# boot2 vs live-bootstrap: seed → tcc

A line-count comparison of the path from `hex0-seed` to a working
`tcc 0.9.26`. Both projects share the same opaque seed and the same
stage0-posix hex/M0 layer; both terminate this segment at the same tcc
release, so the segment is apples-to-apples.

## Summary

| Scope | boot2 | live-bootstrap | Reduction |
|---|---:|---:|---:|
| Single arch, hand-written | 21,468 | 30,813 | 30% |
| Single arch, incl. generated tables | 22,047 | 34,624 | 36% |
| 3 arches, hand-written | 22,695 | 41,053 | 45% |
| 3 arches, incl. generated tables | 24,434 | 44,864 | 46% |
| Per-arch swap set (3-arch sum) | 3,649 | 15,595 | 77% |

- **Audit surface, seed → tcc**: boot2 is ~30% smaller for a single
  architecture, ~45% smaller across three. The shared-once portion
  is ~29% smaller; the per-arch portion (which scales with the number
  of supported arches) is ~77% smaller.
- **Per-arch porting cost**: ~1,200 lines/arch in boot2 vs ~5,200 in
  live-bootstrap. The dominant per-arch cost in live-bootstrap is
  `cc_<arch>.M1` — a hand-written, arch-specific implementation of
  M2-Planet (a C-subset compiler) in M1 macro assembly, ~3,800–5,800
  lines per arch. boot2 has no equivalent because P1, a portable
  pseudo-ISA, sits immediately above the M0 macro stage; the entire
  arch surface is the P1 backend (`P1-<arch>.M1pp` + `P1-<arch>.M1`)
  plus a small entry/syscall shim.
- **Distinct upstream projects in the audit path**: boot2 = 1
  (stage0-posix vendored seed bytes only); live-bootstrap = 5
  (stage0-posix + per-arch `cc_<arch>.M1`, mescc-tools, M2-Planet,
  mes, nyacc).
- **Caveats**: the 3-arch comparison uses live-bootstrap's
  {x86, x86_64, riscv64} vs boot2's {aarch64, amd64, riscv64}
  (live-bootstrap has no complete aarch64 mes-m2 layer in this
  checkout). live-bootstrap's audit surface is amortized across
  multiple downstream consumers (e.g. Guix); boot2's is bespoke.
  Vendored seed bytes (identical between projects), build orchestration
  scripts, and the `tcc 0.9.26` source itself are excluded from all
  totals.

## Scope

- **Window**: from the vendored `hex0-seed` byte stream up to the first
  `tcc 0.9.26` binary. boot2's musl, seed-kernel, and DRIVER=seed
  loop are out of scope; live-bootstrap's downstream binutils, gcc,
  glibc, and userland are out of scope.
- **Architecture**: amd64 / x86_64 (single-arch totals). Per-arch swap
  sets shown separately.
- **Vendored seed bytes**: the seven stage0-posix files (`hex0-seed`,
  `hex0.hex0`, `hex1.hex0`, `hex2.hex1`, `catm.hex2`, `M0.hex2`,
  `ELF.hex2`) are byte-identical between the two projects and are
  excluded from the line counts (counted as shared trust base).

## Methodology

Comments and blank lines stripped per file dialect:

| Dialect | File extensions | Comments stripped |
|---|---|---|
| asm-mix | `.P1`, `.P1pp`, `.M1pp`, `.M1`, `.hex2`, `.hex0`, `.hex1` | `;` (line + inline), `#` (line-leading) |
| Scheme  | `.scm`                                                      | `;` only (`#` is syntax: `#t`, `#f`, `#:kw`) |
| C       | `.c`, `.h`, `.S`                                            | `//` to EOL, `/* … */` (multi-line, string-literal aware) |

Counts cross-checked against `cloc` and against `tools/count-lines.sh`
where applicable, and verified by sampling 10 random kept lines per
representative file (every sample was real code, no comments slipped
through, no code wrongly dropped).

## Hand-written code, seed → tcc, amd64

| Component | Code lines |
|---|---:|
| **boot2** | |
| `M1pp/M1pp.P1` | 5,141 |
| `hex2pp/hex2pp.P1` | 3,231 |
| `P1/P1.M1pp` | 184 |
| `P1/P1pp.P1pp` | 1,298 |
| `P1/P1-amd64.M1pp` (M1pp-style backend, used boot2 onward) | 683 |
| `P1/{elf-end,entry-libc,entry-plain}.P1pp` | 12 |
| `catm/catm.P1pp` | 105 |
| `scheme1/scheme1.P1pp` | 4,334 |
| `scheme1/prelude.scm` | 512 |
| `cc/cc.scm` | 5,186 |
| `vendor/mes-libc/libc.c` | 782 |
| **boot2 hand-written total** | **21,468** |
| `P1/P1-amd64.M1` (flat M0-compatible backend table, used by boot1 before M1pp exists; checked-in output of `P1/gen/p1_gen.py` → `bootprep/prune-p1-table.sh`) | 579 |
| **boot2 incl. generated table** | **22,047** |
| | |
| **live-bootstrap** | |
| stage0-posix `cc_amd64.M1` (M2-Planet seed compiler in M1 macro asm) | 3,848 |
| stage0-posix `amd64_defs.M1`, `libc-core.M1`, `ELF-amd64.hex2` | 127 |
| `mescc-tools` (8 C files: `catm`, `hex2*`, `M1-macro`, `blood-elf`, `get_machine`, `stringify`) | 2,445 |
| `M2-Planet` (8 C + 3 H) | 6,807 |
| mes-m2 inputs (108 files: `mes/src/`, `mes/lib/`, `mes/include/` — interpreter + mini-libc) | 8,351 |
| mes Scheme: `module/mescc/` (x86_64 only) + `module/mes/` (13 files) | 4,770 |
| nyacc 1.00.2-lb1 hand-written core (13 files: `lalr`, `lex`, `parse`, `util`, `lang/{util,sx-util}`, `lang/c99/{parser,pprint,cpp,cppmach,util,body}`) | 4,465 |
| **live-bootstrap hand-written total** | **30,813** |
| nyacc generated parser tables (`lang/c99/mach.d/{c99,c99x,cpp}-{tab,act}.scm`, emitted by `gen-c99-files.scm`, runtime-loaded) | 3,811 |
| **live-bootstrap incl. generated tables** | **34,624** |

## Per-arch swap set

Lines that differ when the bootstrap targets a different architecture.

### boot2

| File | aarch64 | amd64 | riscv64 |
|---|---:|---:|---:|
| `P1/P1-<arch>.M1` (flat backend table for boot1; generated, checked in) | 580 | 579 | 580 |
| `P1/P1-<arch>.M1pp` (M1pp-style backend, used boot2 onward) | 543 | 683 | 516 |
| `tcc/libc/<arch>/start.S` | 10 | 13 | 21 |
| `tcc/libc/<arch>/sys_stubs.S` | 41 | 41 | 42 |
| **Per-arch total** | **1,174** | **1,316** | **1,159** |

### live-bootstrap

| File | x86_64 | riscv64 |
|---|---:|---:|
| stage0-posix `cc_<arch>.M1` (M2-Planet seed) | 3,848 | 3,944 |
| stage0-posix `<arch>_defs.M1`, `libc-core.M1`, `ELF-<arch>.hex2` | 127 | 248 |
| mes `lib/<arch>-mes/<arch>.M1` | 225 | 200 |
| mes `lib/m2/<arch>/<arch>_defs.M1` | 199 | 200 |
| mes `lib/m2/<arch>/ELF-<arch>.hex2` | 32 | 33 |
| mes `lib/linux/<arch>-mes-m2/{crt1,_exit,_write,syscall}.c` | 131 | 161 |
| mes `lib/linux/<arch>-mes-m2/crt1.M1` | 22 | 21 |
| mes `include/linux/<arch>/syscall.h` | 60 | 60 |
| mes `module/mescc/<arch>/{as,info}.scm` | 678 | 554 |
| M2-Planet `if(<ARCH> == Architecture)` branches inside `cc_core.c`, `cc_emit.c`, `cc_types.c` | ~35 | ~10 |
| **Per-arch total** | **~5,360** | **~5,430** |

## Structural notes

- **Why live-bootstrap has more per-arch code**: the first C compiler
  above stage0-posix's hex/M0 stage is `cc_<arch>.M1`, a hand-written
  arch-specific implementation of M2-Planet in M1 macro assembly. It
  is the bridge between M0 and the C-written M2-Planet, and is a
  separate from-scratch port per architecture (~3,800–5,800 lines
  each). After M2-Planet exists, mes is also written as C-subset
  source compiled by M2-Planet, but it carries an additional per-arch
  Scheme codegen (`mescc/<arch>/{as,info}.scm`) and per-arch macro
  tables (`<arch>.M1`, `<arch>_defs.M1`).

- **Why boot2 has less per-arch code**: boot2 introduces P1, a
  portable pseudo-ISA, immediately above the M0 macro stage. The
  P1 backend (`P1-<arch>.M1pp`, ~500–700 lines) plus a flat
  M0-compatible variant (`P1-<arch>.M1`, ~580 lines, used in boot1
  before M1pp exists) are the only arch-specific code-generation
  surfaces; everything above them (`M1pp`, `hex2pp`, `scheme1`,
  `cc.scm`) is one source compiled per arch via that backend. There
  is no separate per-arch C compiler.

- **Distinct upstream projects in the audit path**:
  - boot2: stage0-posix (vendored seed bytes only)
  - live-bootstrap: stage0-posix (seed + per-arch `cc_<arch>.M1`),
    mescc-tools, M2-Planet, mes, nyacc — five projects.

- **Generated tables in the audit surface**: both projects ship
  machine-generated tables that are loaded at bootstrap time:
  - boot2: `P1/P1-<arch>.M1` (~580 lines per arch), produced by
    `P1/gen/p1_gen.py` + `P1/gen/<arch>.py` and pruned by
    `bootprep/prune-p1-table.sh`. Consumed by M0 in boot1.
  - live-bootstrap: nyacc's `lang/c99/mach.d/*.scm` (3,811 lines),
    produced by `gen-c99-files.scm` from the LALR grammar in
    `body.scm`. Loaded by mescc at runtime.

  Generated tables are counted separately from hand-written totals
  above. The host-side generators themselves (`P1/gen/*.py`,
  `gen-c99-files.scm`) are excluded from both counts.

- **Build orchestration scripts** (boot2's `boot/boot*.sh`,
  live-bootstrap's `rootfs.py` and `kaem` files) are not counted on
  either side.
