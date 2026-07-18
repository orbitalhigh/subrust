# boot2 libc

Goal: a `tcc-boot2` that runs and produces working binaries. Three
phases:

1. **Phase A** — compile mes-libc to a P1pp library and link it into
   tcc-boot2 itself. Status: linkage green, runtime hardening in
   progress (see [§Phase A status](#phase-a-status)).
2. **Phase B1** — produce a `libc.a` archive on disk at the path
   tcc-boot2 expects (`$LIBDIR/libc.a`). Not started.
3. **Phase B2** — produce a `libtcc1.a` archive on disk at
   `$LIBDIR/tcc/libtcc1.a` (runtime helpers tcc emits calls to:
   `__divdi3`, `__floatundidf`, …). Not started.

Until Phase B lands, tcc-boot2 can only run paths that don't need to
link external archives — `-version`, parse-only smokes.

Strategy in one sentence: **maintain a single hand-collapsed libc
source (`vendor/mes-libc/libc.c`, started from mes-libc 0.24) whose
syscall layer calls our P1pp labelled `sys_*` entry points directly,
then build it three different ways: as P1pp linked into tcc-boot2
(Phase A), as an ELF archive via tcc-boot2 itself (Phase B1), and
tcc's own `lib/libtcc1.c` via tcc-boot2 (Phase B2).**

Anchor: P1pp syscall block at
[P1/P1pp.P1pp:986-1058](../P1/P1pp.P1pp). The header comment in
[`vendor/mes-libc/libc.c`](../vendor/mes-libc/libc.c) is the
authoritative manifest of what's provided / required / ordered.

## Layout

```
vendor/mes-libc/
├── libc.c              single-file libc (~1000 lines).  Includes
│                        syscall wrappers (_read/_write/.../brk),
│                        FILE-globals, malloc/free, printf family,
│                        ctype, string ops, __libc_init, __assert_fail.
└── LICENSE             mes's COPYING; libc subset is GPLv3+.

bootprep/
├── libc-flatten.sh     host: cc -E -nostdinc -I bootprep/headers
│                        libc.c → build/$ARCH/vendor/mes-libc/libc.flat.c.
│                        Prepends build/$ARCH/vendor/tcc/stdarg-bridge.h
│                        (see docs/TCC.md) so tcc itself can later
│                        compile through the flattened libc.
└── headers/            hand-rolled libc headers consumed at flatten
                         time by the host preprocessor; `stdarg.h`
                         here routes va_* through __builtin_va_*.

tests/
├── build-cc.sh         container: cc.scm <flat.c> → P1pp
│                        (CC_LIB=PFX selects --lib= mode; see §Linking)
└── build-p1pp.sh       container: catm + M1pp → P1pp/M1/asm/ELF.

P1/
├── entry-libc.P1pp     :p1_main wrapper (calls __libc_init, main)
└── elf-end.P1pp        single :ELF_end terminator label

tests/cc-libc/          targeted fixtures for cc.scm + libc TDD
```

Memory helpers (`memcpy`, `memmove`, `memset`, `memcmp`) come from
`tcc/cc/mem.c`, not from `libc.c` — they're shared with `-nostdlib`
test/runtime paths.

## Phase A — link tcc-boot2

### P1pp syscall wrappers

`P1/P1pp.P1pp` defines labelled syscall entry points; per-arch
backend macros (`P1/P1-{aarch64,amd64,riscv64}.M1pp`) supply the
syscall numbers. Original wrappers covered tcc's needs partly; we
added three for libc:

| arch    | lseek | brk | unlinkat |
|---------|------:|----:|---------:|
| amd64   | 8     | 12  | 263      |
| aarch64 | 62    | 214 | 35       |
| riscv64 | 62    | 214 | 35       |

`:sys_unlink` always routes through `unlinkat(AT_FDCWD, path, 0)` —
same trick as `:sys_open → openat`, so the C-visible interface is
identical across arches.

Acceptance fixture: `tests/p1/sys_calls.P1pp` exercises all three on
every arch via `make test SUITE=p1`.

### What `libc.c` provides

The header comment in [`vendor/mes-libc/libc.c`](../vendor/mes-libc/libc.c)
is the source of truth; in summary:

- **syscalls** — `_read _write _open3 close lseek brk unlink _exit
  raise abort` plus `environ getenv __libc_init` and ENOSYS stubs
  for `access execve fsync rmdir stat strtod`. Each thin C wrapper
  calls the matching `sys_*` P1pp label.
- **I/O** — `stdin / stdout / stderr` as real `FILE*` symbols (FILE
  is a long-typed alias for the fd), plus `fopen fdopen fclose
  fflush fseek ftell remove fread fwrite fputs fputc fgetc puts
  strdup`, the printf family (`fprintf printf snprintf sprintf
  vfprintf vsnprintf vprintf vsprintf`).
- **stdlib** — `malloc free realloc qsort exit atoi strtol strtoul
  strtoull strtof`. Allocator is a brk-backed free list.
- **string** — `strlen strcmp strcpy strncmp strncpy strchr strrchr
  strstr strcat strdup memmem`.
- **ctype** — `isdigit islower isnumber isspace isxdigit toupper`.
- **assertions** — `__assert_fail`.

`mem*` (`memcpy memmove memset memcmp`) come from `tcc/cc/mem.c`,
not `libc.c` — they're also linked into the `-nostdlib` tcc
runtime and tests.

`__libc_init(argc, argv)` walks argv's NULL terminator to populate
`environ` so the first `getenv()` doesn't dereference a NULL
environment pointer. `P1/entry-libc.P1pp` calls it ahead of `main`.

### Build

`bootprep/libc-flatten.sh --arch <a>` (host): stages
`vendor/mes-libc/libc.c` to `build/$ARCH/vendor/mes-libc/libc-stage/`,
then runs `host_cc -E -P -nostdinc -I bootprep/headers
-D __linux__=1 -D __${MES_ARCH}__=1 -D __riscv_xlen=64
-D HAVE_CONFIG_H=0 -D inline=` against it and prepends
`build/$ARCH/vendor/tcc/stdarg-bridge.h` (guarded by `#ifndef CCSCM`)
to produce `build/$ARCH/vendor/mes-libc/libc.flat.c` (~27 KB).

`MES_ARCH` mapping is `aarch64→riscv64`, `amd64→x86_64`,
`riscv64→riscv64` — `bootprep/headers/` is arch-agnostic, so the
mapping only feeds the `__${MES_ARCH}__` predefine that gates a few
preprocessor branches inside `libc.c`.

`tests/build-cc.sh` (container) then runs `cc.scm` over
`libc.flat.c` to produce `build/$ARCH/vendor/mes-libc/libc.P1pp`.

### Linking — catm chain

cc.scm has a `--lib=PFX` flag that turns its output into a P1pp
library: it suppresses the auto-emitted entry stub
(`%fn(p1_main, 16, { %call(&main) })`) and the trailing `:ELF_end`,
and namespaces anonymous string labels as `PFX+"cc__str_N"` so two
cc.scm outputs in the same link don't collide on `cc__str_0..N`.
String literals emit their bytes plus a NUL terminator, then an
explicit `.align 8` (any TU, lib or exec) so labels following a string
land at an aligned address — without it, aarch64 BLR / 4-byte LDR
SIGBUS once a non-multiple-of-4 string shows up in `.data`.

Wired together, the link is just `catm`:

```
P1/entry-libc.P1pp                      # :p1_main → __libc_init → main
build/$ARCH/vendor/mes-libc/libc.P1pp   # cc.scm --lib=libc__   → libc__cc__str_*
<client>.P1pp                           # cc.scm --lib=<pfx>__  → <pfx>__cc__str_*
P1/elf-end.P1pp                         # :ELF_end
```

`tests/build-p1pp.sh` already cats its inputs in front of the M1pp
expander, so the catm chain is just its source-list arguments. Both
the tcc-boot2 link rule (Makefile) and the cc-libc test suite
(`tests/run-suite.sh`) compose this way; the tcc-boot2 client uses
prefix `tcc__`, every cc-libc fixture uses `app__`.

`__libc_init` (defined in `vendor/mes-libc/libc.c`) walks argv's
NULL terminator to populate `environ`; it must run before any libc
function that reads the environment. That's why the entry fragment
calls it ahead of `main`.

### Wiring

```
make tcc-boot2 ARCH=aarch64    # builds libc.P1pp + tcc.flat.P1pp
                               # (both --lib= mode), then catms with
                               # entry-libc + elf-end into the ELF
```

The tcc-boot2 link rule depends on `build/$ARCH/libc.P1pp`,
`P1/entry-libc.P1pp`, and `P1/elf-end.P1pp`; rebuilds when any
changes.

## Phase A status

`make tcc-boot2 ARCH=aarch64` links cleanly (0 unresolved symbols).
`tcc-boot2 -version` currently segfaults. We're driving the failure
mode through the **cc-libc** test suite (next §) so each cc.scm/libc
bug surfaces as one focused fixture instead of a 1.8 MB binary
diagnostic.

| fixture            | status | exercises                                                                          |
|--------------------|--------|------------------------------------------------------------------------------------|
| 00-exit            | PASS   | bare `int main() { return 7; }`                                                    |
| 01-write-syscall   | PASS   | direct `extern long sys_write` (P1pp label)                                        |
| 02-write-libc      | PASS   | `posix/write.c → _write → sys_write` (errno layer)                                 |
| 03-fputs-stdout    | PASS   | `fputs(s, stdout) → fdputs → write`                                                |
| 04-printf-literal  | FAIL   | `printf("plain literal\n")` — prints, then segfaults on main return                |
| 05-printf-int      | FAIL   | `printf("got %d\n", 42)` — pulls 100 instead of 42 (varargs bug) + segfaults       |
| 06-puts            | FAIL   | `puts("ok")` — silent: `oputs` writes through `__stdout` which reads 0 (cc.scm tentative-vs-initialized merge bug) |
| 07-malloc-roundtrip| PASS   | `malloc → brk → sys_brk` round-trip                                                |

Phase A acceptance lands when 04, 05, 06 turn green: that
demonstrates varargs, return-from-libc, and global-initializer
resolution all work end-to-end. `tcc-boot2 -version` retest follows
naturally; expect more cc-libc fixtures to be born from whatever
breaks at that point.

## Workflow — adding a cc-libc fixture

```
tests/cc-libc/<name>.c                # source, plain C
tests/cc-libc/<name>.expected         # exact stdout match (default empty)
tests/cc-libc/<name>.expected-exit    # exit status (default 0)
```

cc.scm doesn't run a preprocessor, so fixtures use explicit `extern`
decls today. (TODO: thread mes headers through `host_cc -E` like
libc-flatten.sh does for the libc itself, then fixtures can use
`#include <stdio.h>` etc.)

```
make test SUITE=cc-libc ARCH=aarch64
make test SUITE=cc-libc ARCH=aarch64 -- 05-printf-int    # one fixture
```

Per-fixture artefacts:

- `build/$ARCH/tests/cc-libc/<name>` — final ELF.
- `build/$ARCH/.work/tests/cc-libc/<name>/` — scratch:
  - `<name>.client.P1pp` — cc.scm output for the fixture (lib mode,
    prefix `app__`)
  - `cc.log` / `p1pp.log` — captured stdout+stderr from each pipeline
    stage; the suite handler dumps the relevant log under the FAIL
    row when a stage exits non-zero.

When triaging a failure, the catm'd source the M1pp expander sees
lives at `build/$ARCH/.work/tests/cc-libc/<name>/combined.M1pp`
(`tests/build-p1pp.sh` copies it there alongside the rest of the
per-stage scratch outputs; the path is also recorded in the sidecar
`<elf>.workdir` next to the binary). Grep that for the symbol or
sequence in question.

## Phase B — build the on-disk archives tcc-boot2 needs

tcc-boot2 produces ELF binaries via its own codegen (X86_64,
aarch64, riscv64). When it links a user program it auto-appends
`-lc` and resolves `__divdi3` / `__floatundidf` / etc. against
`$LIBDIR/tcc/libtcc1.a`. Both archives have to exist on disk before
tcc-boot2 is useful as a compiler. Phase A's `libc.P1pp` doesn't
help here — that one is linked into tcc-boot2 itself in P1pp form.
The archives are tcc-boot2's *output* world.

Build them with tcc-boot2 itself, mirroring live-bootstrap's
`pass1.kaem` (search for `libtcc1.o`).

### B1. libc.a from the same vendored source

Reuse `vendor/mes-libc/libc.c` (i.e. the *unflattened* source —
or the already-flattened `libc.flat.c`, since it was preprocessed
through our own `bootprep/headers/`). Compile with tcc-boot2 per
arch. The exact tcc invocation will need to match whatever set of
predefines `libc.c` expects (see the `bootprep/libc-flatten.sh`
flags above). Archive with `tcc -ar cr libc.a libc.o`.

Install at `$LIBDIR/libc.a` where `$LIBDIR` is whatever
`CONFIG_TCC_CRTPREFIX` was baked into tcc-boot2 (default
`build/$ARCH/sysroot/lib`; align with the `-D CONFIG_TCC_CRTPREFIX`
in the Makefile). Also produce `crt1.o` if the link needs one — for
static binaries with the existing `_start` (`tcc/libc/$ARCH/start.S`)
it can be skipped; check by linking the smoke test below.

The chicken-and-egg concern is moot: tcc-boot2's codegen for
P1-64 targets does not emit `__divdi3`-class calls when compiling
the libc (long-long is native register width on X86_64 / aarch64 /
riscv64). So building libc.a needs no prior libtcc1.a.

### B2. libtcc1.a from upstream tcc

The file is already vendored implicitly via `stage1-flatten.sh`
(it's `tcc-0.9.26-1147-gee75a10c/lib/libtcc1.c` inside the tarball).
The compile uses tcc's own bundled headers from
`tcc-0.9.26-1147-gee75a10c/include/` (stdarg, stddef, stdbool,
float, varargs); no `bootprep/headers/` involvement at this point.

Sketch:

```sh
TCC_BOOT2=build/$ARCH/tcc-boot2/tcc-boot2
TCC_SRC=build/$ARCH/vendor/tcc/tcc-0.9.26-1147-gee75a10c
$TCC_BOOT2 -c -D HAVE_CONFIG_H=1 -D HAVE_LONG_LONG=1 -D HAVE_FLOAT=1 \
    -I "$TCC_SRC" -I "$TCC_SRC/include" \
    -o build/$ARCH/libtcc1.o \
    $TCC_SRC/lib/libtcc1.c
# aarch64 also pulls in lib-arm64.c per upstream:
if [ "$ARCH" = aarch64 ]; then
    $TCC_BOOT2 -c ... -o build/$ARCH/lib-arm64.o $TCC_SRC/lib/lib-arm64.c
    EXTRA=build/$ARCH/lib-arm64.o
fi
$TCC_BOOT2 -ar cr build/$ARCH/libtcc1.a build/$ARCH/libtcc1.o $EXTRA
```

Install at `$LIBDIR/tcc/libtcc1.a` (matches `tcc_add_support(s1,
"libtcc1.a")` against `tcc_lib_path`).

### B3. Wire into the Makefile

```
make tcc-boot2 ARCH=aarch64           # phase A: links tcc-boot2 itself
make tcc-archives ARCH=aarch64        # phase B: produces libc.a + libtcc1.a
make tcc-smoke ARCH=aarch64           # phase B acceptance, see below
```

`tcc-archives` depends on `tcc-boot2`. `tcc-smoke` depends on
`tcc-archives`.

### B4. Phase B smoke tests

- `tests/tcc/300-hello.c` — `printf("hi\n")`. Compile *with
  tcc-boot2*, run, check output. Exercises libc.a auto-link.
- `tests/tcc/301-longlong.c` — does `long long a = ...; a / b;`
  with values that force a real divmod on 32-bit hosts; on our
  P1-64 targets this should still link (idiv is native), but the
  test confirms libtcc1.a is on the path and gets searched.
- `tests/tcc/302-self-host-fragment.c` — pull a small TU out of
  tcc.c (e.g. one of the smaller pass1 files) and compile it with
  tcc-boot2 to confirm tcc-on-tcc works end-to-end.

Phase B acceptance: `make tcc-smoke ARCH=aarch64` passes all three.

### Looking ahead — milestone 5

With Phase B done, tcc-boot2 can compile arbitrary C through its
own codegen. Milestone 5 (use tcc-boot2 to rebuild tcc.c → tcc-boot0,
checksum-match against the live-bootstrap reference) becomes a
matter of feeding tcc.c through tcc-boot2 with the boot0 defines.
That's tracked in [TCC.md](TCC.md), not here.

## Out of scope

- **Threading, locale, dynamic linker, IEEE-754 math.** `libc.c`'s
  fp paths (`strtof`, etc.) are present but the rest of the boot
  pipeline doesn't exercise them seriously.
- **errno from threads.** `errno` is a single global int. cc.scm has
  no TLS; tcc-boot2 is single-threaded.

## Notes for the engineer

- `libc.c` is a single editable file; if a fixture surfaces a
  missing routine, add it directly there rather than reaching for
  upstream mes-libc.
- cc.scm's `--cc-debug` flag prints per-phase heap usage on stderr.
  `libc.flat.c` is ~27 KB so heap should be flat; if it isn't,
  that's a cc.scm bug, not a libc bug.
- Layout: `vendor/mes-libc/libc.c` is arch-agnostic. The only
  per-arch input feeding the libc build is the `__${MES_ARCH}__`
  predefine that `bootprep/libc-flatten.sh` passes through.
- The flatten-time stdarg shim (`bootprep/headers/stdarg.h`) and
  the per-arch tcc stdarg bridge (generated at
  `build/<arch>/vendor/tcc/stdarg-bridge.h`, prepended to
  `libc.flat.c`) are *both* required and serve different consumers
  — see the comment at the top of `bootprep/headers/stdarg.h` and
  [docs/TCC.md](TCC.md).
