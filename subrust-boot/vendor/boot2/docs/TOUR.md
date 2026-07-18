# Tour

A walk through the bootstrap, stage by stage. Each stage takes about
ten minutes to read. By the end, you'll have seen every binary the
chain produces and how each one is built from the binaries before it.

The READMEs and per-component specs in docs/ cover the *what* and
*why* in depth. This tour is the *order* — the path that ties them
together.

## §0. Map

The chain is a sequence of seven shell scripts under [boot/](../boot).
Each `bootN.sh` produces one or two binaries from the binaries the
prior stages produced, plus source from the canonical
`build/<arch>/src/` tree (prepared once by `bootprep/prep-src.sh`).

| Stage | Driver script           | Produces                                  | Trust extension                                               |
| ----- | ----------------------- | ----------------------------------------- | ------------------------------------------------------------- |
| 0     | [boot0.sh](../boot/boot0.sh) | `hex2`, `catm`, `M0`                  | hex0-seed → hex assemblers → file concatenator → macro stage  |
| 1     | [boot1.sh](../boot/boot1.sh) | `M1pp`, `hex2pp`                      | first programs in the portable P1 pseudo-ISA    |
| 2     | [boot2.sh](../boot/boot2.sh) | `catm` (rebuilt), `scheme1`           | seed `catm` retired; Scheme interpreter arrives               |
| 3     | [boot3.sh](../boot/boot3.sh) | `tcc0`                                | C arrives — `cc.scm` (in scheme1) compiles upstream tcc       |
| 4     | [boot4.sh](../boot/boot4.sh) | `tcc1`, `tcc2`, `tcc3`, `libc.a`, `libtcc1.a` | tcc self-host, byte-identical fixed point `tcc2 == tcc3`, minimal libc |
| 5     | [boot5.sh](../boot/boot5.sh) | `libc.a`, `crt{1,i,n}.o`              | musl-1.2.5 built by the self-hosted tcc                       |
| 6     | [boot6.sh](../boot/boot6.sh) | `Image` (aarch64) / `kernel.elf`      | a minimal kernel that can host the chain (`DRIVER=seed`)     |

Drivers (`DRIVER=podman` default, `DRIVER=seed` for the loop pass) only
change *where* each stage executes; the inputs, outputs, and shell
scripts are identical.

## §1. boot0 — from a hex seed to a macro assembler

**You arrive with**: nothing of ours. Just `sh`, `podman` or
`qemu-user-static`, and the seven [vendored seed
files](../vendor/seed/) per arch. `hex0-seed` is the only opaque
artifact; it is a few hundred bytes (526 / 229 / 392 for
aarch64 / amd64 / riscv64).

**boot0 builds**: `hex2`, `catm`, `M0`.

**How**:

```
hex0-seed  hex0.hex0   →  hex0   (hex assembler, no labels)
hex0       hex1.hex0   →  hex1   (hex assembler with labels)
hex1       hex2.hex1   →  hex2   (hex with ELF-aware linking)
hex2       catm.hex2   →  catm   (concatenate files)
catm       ELF.hex2 + M0.hex2  →  M0.combined.hex2
hex2       M0.combined.hex2    →  M0     (macro stage above hex2)
```

Each line is one `stage` call in [boot0.sh](../boot/boot0.sh). The
script is 48 lines. Read it.

**Trust extension**: from this point on you have a file concatenator
(`catm`), a hex-with-labels assembler/linker (`hex2`), and a macro
preprocessor (`M0`). Everything later is derived from these three.

**Worth reading**: `boot0.sh` itself, then the
[live-bootstrap stage0-posix
documentation](https://github.com/oriansj/stage0-posix) for what each
seed file is.

## §2. boot1 — first self-hosted programs

**You arrive with**: `hex2`, `catm`, `M0` from boot0.

**boot1 builds**: `M1pp` and `hex2pp` — the M1 expander and
hex2 assembler that all later P1 / P1pp source uses.

**How**: a small build function, applied once each:

```sh
build_p1() {  # $1 = source .P1, $2 = output binary name
    stage catm combined.M1 P1.M1 "$1"           -- P1.M1 "$1"            -- combined.M1
    stage M0   combined.M1 prog.hex2            -- combined.M1           -- prog.hex2
    stage catm linked.hex2 ELF.hex2 prog.hex2   -- ELF.hex2 prog.hex2    -- linked.hex2
    stage hex2 linked.hex2 "$2"                 -- linked.hex2           -- "$2"
}
```

`P1.M1` is the per-arch backend that turns portable P1 instruction
mnemonics into native machine code. `M1pp.P1` and `hex2pp.P1` are
~5000 and ~3100 lines of P1 source. They are the first programs in
this chain written in our own pseudo-ISA, and are the first sources that
are naturally human-readable (ie not hex bytes) and portable.

**Trust extension**: M1pp accepts the macro flavour every later
`.P1pp` file uses (function-like macros, struct/enum synthesis,
compile-time integer eval, token paste, hygienic intra-macro labels).
hex2pp adds nestable scopes, alignment, and pointer-size directives on
top of hex2. Together they are enough to compile every later stage's
source.

**Worth reading**: `M1pp/M1pp.c` is a 2110-line C reference implementation kept
in sync with `M1pp.P1`. Its preamble lays out the syntax in 60 lines:

```
/*
 * Tiny single-pass M1pp macro expander. Output is consumed directly by
 * hex2pp -- there is no intermediate M0/hex2 stage. All emission is in
 * the byte/label/directive vocabulary hex2pp accepts.
 *
 * Syntax:
 *   %macro NAME(a, b)
 *   ... body ...
 *   %endm
 *   …
```

The actual bootstrap loads `M1pp.P1`; the C version is for reference.
Same arrangement for `hex2pp/hex2pp.c` vs `hex2pp/hex2pp.P1`.

The full M1pp / hex2pp specs are [docs/M1PP.md](M1PP.md) and
[docs/HEX2pp.md](HEX2pp.md).

## §3. boot2 — closing on catm, then a Scheme

**You arrive with**: M1pp, hex2pp from boot1; the seed `catm` from
boot0 (one last use).

**boot2 builds**: `catm` (rebuilt from `catm.P1pp`) and `scheme1`.

**How**: the universal P1pp build function appears here for the first
time —

```sh
build_p1pp() {  # $1 = catm-bin, $2 = src .P1pp, $3 = out
    stage "$1"   combined.M1pp backend.M1pp frontend.M1pp libp1pp.P1pp "$2" \
                 -- backend.M1pp frontend.M1pp libp1pp.P1pp "$2"  --  combined.M1pp
    stage M1pp   combined.M1pp expanded.hex2pp                    --  combined.M1pp  -- expanded.hex2pp
    stage "$1"   linked.hex2pp ELF.hex2 expanded.hex2pp           --  ELF.hex2 expanded.hex2pp -- linked.hex2pp
    stage hex2pp -B 0x600000 linked.hex2pp "$3"                   --  linked.hex2pp -- "$3"
}
```

The four files concatenated into `combined.M1pp` are:
`P1-<arch>.M1pp` (per-arch backend), `P1.M1pp` (portable frontend),
`P1pp.P1pp` ("libp1pp" — standard macros and helpers), and the
program source. M1pp expands all the macros; hex2pp turns the
resulting bytes into an ELF.

`catm.P1pp` is built first (using the seed `catm` one last time, then
discarded). After that, `scheme1.P1pp` is built using the new `catm`.
From here forward, stage0 is retired, and all binaries have been built from
portable sources (no arch-specific code and no hex).

**Trust extension**: `catm` is now self-built. `scheme1` is a small
R7RS-subset Scheme — fixnums, pairs, symbols, bytevectors,
closures, records. ~4300 LoC of P1pp; full surface in
[docs/SCHEME1.md](SCHEME1.md). It is not a teaching toy: it runs the C
compiler in the next stage.

**Worth reading**: `catm/catm.P1pp` is 105 lines and is the most
readable single P1pp file in the project. It demonstrates the whole
P1pp surface in one ~100-line program. After that, eval/apply and the
dispatcher headers of
[scheme1/scheme1.P1pp](../scheme1/scheme1.P1pp).

## §4. boot3 — C arrives via a Scheme

**You arrive with**: `M1pp`, `hex2pp` from boot1; `catm`, `scheme1`
from boot2.

**boot3 builds**: `tcc0` — a tcc-0.9.26 binary, compiled by
`cc.scm` running inside `scheme1`, and assembled by M1pp and hex2pp.

**How**: the boot3 driver hands `scheme1` a generated `run.scm` that
loads `cc/cc.scm`, hands it the flattened tcc translation unit
`tcc.flat.c`, captures the emitted P1pp, and runs the standard
M1pp+hex2pp pipeline to turn that into an ELF. `tcc.flat.c` is one
big TU produced by `bootprep/stage1-flatten.sh` from upstream
tcc-0.9.26 plus a small set of patches (search for `our-patches/` in
that script).

`cc.scm` is the central piece. ~5200 LoC of Scheme implementing a
streaming C compiler: lexer → preprocessor → parser → codegen →
P1pp emission. Full code map in [docs/CCSCM.md](CCSCM.md); the
accepted C subset in [docs/CC.md](CC.md).

**Trust extension**: a C compiler that you can read end to end in an
afternoon is now part of the chain. Its output, `tcc0`, is a real tcc
— with all of tcc's faults plus whatever divergence cc.scm's codegen
contributes; boot4 will iron that out.

**Worth reading**: the cc.scm overview block, then the codegen
section header (search for `;; ── Code generator ──`). The
phase-1 milestone test
[tests/cc/000-return-argc.c](../tests/cc/000-return-argc.c) is the
smallest end-to-end exercise of the whole pipeline.

## §5. boot4 — self-host to a fixed point

**You arrive with**: `tcc0` from boot3.

**boot4 builds**: `tcc1`, `tcc2`, `tcc3`, plus `crt1.o`,
`libc.a` (mes-libc), and `libtcc1.a`.

**How**: tcc compiles tcc, three more times.

```
tcc0 = tcc-source compiled by cc.scm        ← boot3
tcc1 = tcc-source compiled by tcc0          ← here
tcc2 = tcc-source compiled by tcc1          ← here
tcc3 = tcc-source compiled by tcc2          ← here
```

After the third bounce, the script asserts `tcc2 == tcc3` byte for
byte. That's the fixed point: tcc compiling itself with no help from
cc.scm reaches a stable image, and any future build that walks through
this stage will produce the same `tcc3` from the same sources.

Why four stages, not two? `cc.scm` and tcc's own codegen produce
different — but both correct — code for the same source, so `tcc0`
and `tcc1` differ. `tcc1` and `tcc2` are both built by tcc, but `tcc1`
itself was a cc.scm-shaped binary, so its codegen choices in `tcc2`
need one more bounce to reach the tcc-shaped fixed point.

**Trust extension**: a self-built tcc that demonstrates determinism
under self-application. Plus mes-libc, a small libc that's good
enough for tcc's own runtime, and `libtcc1.a` — tcc's helper archive
(division/intrinsics that tcc emits calls to).

**Worth reading**: the fixed-point check in
[boot4.sh:114–123](../boot/boot4.sh) is the audit point of the entire
chain.

## §6. boot5 — a real libc

**You arrive with**: `tcc3` and `libtcc1.a` from boot4; `catm` and
`scheme1` from boot2.

**boot5 builds**: `libc.a` and `crt{1,i,n}.o` from upstream
musl-1.2.5, plus a `hello` smoke binary linked statically against the
result.

**How**: musl is patched lightly during prep (`bootprep/musl-vendor.sh`
+ `bootprep/prep-src.sh`) to work around tcc's missing GCC extensions:
register-asm-variable syscalls, `__attribute__((alias))` weak refs,
`_Complex`, x86_64 SSE/x87 inline asm. The patch list lives in the
musl prep script. Then a generated `run.scm` walks the per-source
list and invokes `tcc -c` ~1300 times inside the driver, producing a
static archive.

**Trust extension**: a real libc. The chain no longer depends on
mes-libc for anything but the seed-kernel's `hello` link path; every
later binary you build with this `tcc3` can use musl.

**Worth reading**: the musl skip / patch lists under
[vendor/musl/](../vendor/musl/) and the calibration script
[bootprep/boot5-calibrate.sh](../bootprep/boot5-calibrate.sh).
Background: [docs/MUSL.md](MUSL.md), [docs/LIBC.md](LIBC.md).

## §7. boot6 — a kernel that runs the chain

**You arrive with**: `tcc3` from boot4 and `scheme1` from boot2.

**boot6 builds**: a minimal kernel image — `Image` on aarch64, `kernel.elf`
on amd64 / riscv64.

**How**: tcc3 compiles `seed-kernel/kernel.c` plus the per-arch entry
in `seed-kernel/arch/<arch>/{kernel.S, mmu.c, arch.h}` and `tcc/cc/mem.c`,
linking through tcc directly — no separate linker, no objcopy. On
aarch64, tcc's flat-binary mode produces an arm64 `Image`.

The kernel is a simple OS satisfying [docs/OS.md](OS.md) Tier 1: it
boots through an arch backend with two virtio-blk-MMIO disks, parses
the DTB, brings up a polling virtio-blk driver, reads a cpio newc
archive into an in-memory tmpfs, loads `/init` (a static target ELF),
enters it through the trap-return path, and serialises the tmpfs to
the second disk on exit. ~1300 lines of C plus per-arch entry / MMU
in another ~250.

**Trust extension**: a kernel built by the same chain it needs to
run. With this kernel, `DRIVER=seed` lets you re-run *every* prior
stage inside the kernel that boot6 just produced.

**Worth reading**:
[seed-kernel/kernel.c](../seed-kernel/kernel.c) — preamble plus the
console / fs / trap dispatcher headers.
The full OS contract is [docs/OS.md](OS.md).

## §8. The loop

The chain has now produced a kernel. The kernel is enough to run any
of the earlier stages: under `DRIVER=seed`, each `bootN.sh` packs its
inputs into a cpio, boots the kernel under `qemu-system-<arch>`, the
kernel loads an init ELF that runs the same shell pipeline as the
podman driver, and the resulting outputs are extracted from the
write-disk on shutdown.

```sh
DRIVER=seed ./boot/boot.sh aarch64
```

`tests/seed-accept.sh` then diffs the seed-driver outputs against the
podman-driver outputs byte for byte. When that diff is empty, the
chain has run on itself and produced the same artifacts. That is the
loop closure — the smallest interesting fact this project tries to
demonstrate.

## What to read next

* [docs/P1.md](P1.md) — the portable pseudo-ISA, the abstraction
  every later stage rests on.
* [docs/CCSCM.md](CCSCM.md) — code map for `cc.scm`, the C compiler.
* [docs/OS.md](OS.md) — the seed-kernel / target-userland contract.
* The tests under [tests/](../tests). Most suites are
  fixture-driven; each fixture is a small program that demonstrates
  one feature. `tests/cc-cg/`, `tests/cc/`, and `tests/cc-libc/`
  together are a guided tour of the C surface.
