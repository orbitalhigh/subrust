# boot5 musl spec

`boot/boot5.sh <arch>` builds a static musl 1.2.5 libc with the
verified boot4 tcc for the same architecture, then links and runs a
static hello-world smoke binary. Supported architectures are `amd64`,
`aarch64`, and `riscv64`; aarch64 is verified end-to-end every run, and
the same recipe has previously been validated against amd64 and riscv64.

The build runs in `boot2-busybox:$ARCH` (scratch + busybox, no libc, no
`/etc`) and produces only static artifacts. Dynamic linking and `ldso/`
are intentionally out of scope.

The container body is a single line — `sh -eu /work/in/run.sh` — where
`run.sh` is a flat sequence of `tcc -c …` / `cp` / `mkdir` invocations
generated on the host. The container shell needs no control flow,
parameter expansion, or functions; intent is to fit a kaem-class
minimal shell.

## Usage

```sh
boot/boot3.sh        <amd64|aarch64|riscv64>
boot/boot4.sh        <amd64|aarch64|riscv64>
bootprep/boot5-calibrate.sh <amd64|aarch64|riscv64>   # once per arch
boot/boot5.sh        <amd64|aarch64|riscv64>
```

## Inputs

| Path | Purpose |
|------|---------|
| `build/$ARCH/boot4/tcc3` | fixed-point self-host tcc from boot4 |
| `build/$ARCH/boot4/libtcc1.a` | tcc runtime archive produced by boot4 |
| `vendor/musl/1.2.5.tar.gz` | pristine upstream musl source |
| `vendor/musl/overrides/` | post-patch files vendored as a tree (replaces the old patch + `patch` binary) |
| `vendor/musl/deletes.txt` | upstream files removed by the same patch set, one path per line |
| `vendor/musl/generated/$MUSL_ARCH/{alltypes,syscall}.h` | per-arch headers pre-generated at vendor time (replaces musl's mkalltypes.sed + `__NR_`→`SYS_` rewrite, so the container needs no awk) |
| `vendor/musl/skip-$ARCH.txt` | per-arch calibration list — sources tcc 0.9.26 cannot compile, produced by `bootprep/boot5-calibrate.sh` |
| `build/$ARCH/vendor/tcc/stdarg-bridge.h` | per-arch `__builtin_va_list` bridge (byte-identical across arches, three arches gated by `#ifdef`; produced by `bootprep/stage1-flatten.sh`) |
| `bootprep/assets/boot-hello.c` | smoke-test source (shared with boot4) |

Architecture mapping:

| `ARCH` | container platform | musl target |
|--------|--------------------|-------------|
| `amd64` | `linux/amd64` | `x86_64-linux-musl` |
| `aarch64` | `linux/arm64` | `aarch64-linux-musl` |
| `riscv64` | `linux/riscv64` | `riscv64-linux-musl` |

## Outputs

`boot/boot5.sh` writes final artifacts to `build/$ARCH/boot5/`:

| File | Purpose |
|------|---------|
| `libc.a` | static musl libc archive |
| `crt1.o`, `crti.o`, `crtn.o` | static startup and init/fini CRT objects |
| `hello` | static smoke-test ELF linked by boot5 |

Staging lives under `build/$ARCH/.boot5-stage/`, organized as:

| Subdir | Role |
|--------|------|
| `in/` | exactly the files the container reads (bind-mounted as `/work/in`) |
| `_host/` | host-only scratch (source enumeration outputs); not visible to the container |
| `out/` | container writes here; host then copies to `build/$ARCH/boot5/` |

The entire `.boot5-stage` tree is disposable; every `boot5.sh` run rebuilds it.

## Pipeline

1. **Stage inputs (host)**. Copy boot4 `tcc3` and `libtcc1.a` to `in/`.
   Extract the musl tarball into `in/musl-1.2.5/`. Overlay the vendored
   `musl-1.2.5-overrides/` tree on top of it. Remove every path listed
   in `musl-1.2.5-deletes.txt`. The result is the post-patch tree that
   the old `patch + belt-and-braces rm` recipe produced — built without
   a `patch` binary anywhere.
2. **Stage pre-generated headers (host)**. Copy
   `musl-1.2.5-generated/$MUSL_ARCH/alltypes.h` and `syscall.h` into
   `in/`. These were produced by `bootprep/musl-vendor.sh` (a host-only
   helper that runs `awk -f bootprep/mkalltypes.awk` and the SYS_ rewrite
   once per arch when the patch set changes).
3. **Enumerate musl sources (host)**. Walk the prepared tree under
   `in/musl-1.2.5/`, mirror musl's per-arch override rule (per-arch file
   under `src/$d/$MUSL_ARCH/` replaces the same-stem base under
   `src/$d/`), then subtract the calibration skip list. Outputs go to
   `_host/` (`base.txt`, `arch.txt`, `replaced.txt`, `keep.txt`,
   `build-srcs.txt`, `build-objdirs.txt`).
4. **Emit `in/run.sh` (host)**. Produce a single flat shell script: cd
   into the working tree, copy in the pre-generated headers + version
   stamp, `mkdir -p` every obj directory, then one literal
   `tcc -c …` per source (~1300 lines for aarch64), the per-arch CRT
   commands (resolved against `crt/$MUSL_ARCH/crti.s` if present,
   otherwise `crt/crti.c`), the literal `tcc -ar rcs lib/libc.a obj1 obj2 …`
   archive line, the `cp` to `/work/out`, and the `tcc -static …`
   link + run of `hello`. No control flow inside the script — every
   condition is resolved at host emission time.
5. **Run (container)**. `podman run … sh -eu /work/in/run.sh`. The
   container `cp -R`s the prepared tree into tmpfs (its bind-mounted
   `/work/in` is logically read-only) and executes `run.sh` straight
   through.
6. **Verify (host)**. Copy outputs into `build/$ARCH/boot5/`. The
   smoke-test `hello` was already executed inside the container as
   the last line of `run.sh`.

`musl`'s own `configure` script is **not run** — it only produces
`config.mak`, which we don't read. boot5 supplies its own hardcoded
`CFLAGS_BASE`.

Assembler inputs must not receive the va-list shim. tcc 0.9.26 applies
`-include` to `.s`/`.S` as well as `.c`, so boot5 keeps separate
`CFLAGS_C` and `CFLAGS_ASM`.

## Compatibility Surface

The musl overrides keep upstream musl mostly intact and replace only the
surfaces tcc 0.9.26 cannot compile:

| Area | Rule |
|------|------|
| syscalls | replace GCC register-asm-variable wrappers with per-arch asm trampolines |
| atomics / thread pointer | replace inline asm operands with extern asm helpers on aarch64 (true atomic via raw `.long` LL/SC) and riscv64 (single-threaded C-inline a_cas — sufficient for the boot5 hello smoke binary; tcc-asm has no LR/SC mnemonics) |
| crt entry trampoline | aarch64 + riscv64: replace upstream `crt_arch.h` with a minimal `_start` that passes `sp` to `_start_c` and tail-jumps. Drops `.option`, `lla gp`, and the `tail` pseudo (none parseable by tcc-asm). |
| weak aliases | implement `weak_alias` via assembler `.weak`/`.set` directives |
| C99 array parameters | remove `[static N]` qualifiers tcc does not parse |
| `_Complex` | stub `complex.h` and remove complex sources |
| arch asm overrides | delete unsupported fenv, signal, setjmp, thread, string, math overrides as needed |
| varargs | pre-include `build/$ARCH/vendor/tcc/stdarg-bridge.h` (the post-patch tcc `<stdarg.h>`) for C translation units |

Required tcc fixes live under `vendor/tcc/patches/`.
The musl build depends on the aarch64 literal-address load/store fixes
and the LP64 `L`-suffix constant fix.

## Calibration

`bootprep/boot5-calibrate.sh <arch>` produces
`vendor/musl/skip-$arch.txt`, the list of musl sources
tcc 0.9.26 cannot compile for that arch. It runs the legacy skip-on-fail
loop in the container once and captures the failures.

Re-run calibration whenever any of these change:
- the tcc patches under `vendor/tcc/patches/`;
- the musl overrides or deletes;
- the vendored tcc or musl source tarballs.

The calibration list lets `boot5.sh` emit a flat `run.sh` whose compile
loop has no `if $TCC … ; then ok else skip fi` branch — every emitted
command is expected to succeed.

## Status

| arch | calibration vendored | skipped sources |
|------|----------------------|-----------------|
| `aarch64` | yes | 8 |
| `amd64` | yes | 12 |
| `riscv64` | yes | 3 |

Skipped sources are outside the boot5 hello closure. They fall into two
categories:

- long-double constant-folding files that tcc 0.9.26 cannot compile;
- thread exit / low-level asm files needing inline-asm operand support.

Anything that references a skipped function may fail to link. The boot5
contract is a static libc sufficient to link and run the included hello
smoke program, not full musl conformance.

## Smoke Output

Successful boot5 ends by running:

```text
hello from tcc-built libc; argc=4
strdup: works, strlen: 5
```

(The same `hello` source, `bootprep/assets/boot-hello.c`, is also linked and
run by boot4 against the mes-libc closure — proving both libc closures
are exec-correct under their respective build systems.)
