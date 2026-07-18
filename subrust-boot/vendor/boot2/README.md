# boot2 — a bootstrap chain you can read

`boot2` brings a Linux/POSIX system up from a few hundred bytes of seed
machine code to `tcc` + `musl` running on a tcc-built kernel. Every
intermediate stage is small enough to read end-to-end. The compiler that
builds the C compiler is in this repository. So is the kernel that runs
it.

## Writing

A series walking through the chain, one rung at a time:

1. [Playing with the bootstrap](https://ryansepassi.com/notes/boot2.html) —
   the portable pseudo-ISA (P1) and the macro layer (M1++).
2. [boot2 Scheme](https://ryansepassi.com/notes/boot2-scheme.html) —
   a Scheme interpreter (scheme1) hosted in P1++.
3. [boot2 C](https://ryansepassi.com/notes/boot2-c.html) —
   a C compiler (cc.scm) hosted in scheme1, compiling tcc to a
   self-hosted fixed point on three arches.

## The chain

```
;; ── boot0.sh ── Bootstrap from seed ──────────────────────────────────
(define hex0 (hex0-seed hex0.hex0))
(define hex1 (hex0 hex1.hex0))
(define hex2 (hex1 hex2.hex1))
(define catm (hex2 catm.hex2))
(define M0   (hex2 (catm ELF.hex2 M0.hex2)))

;; ── boot1.sh ── Self-host m1pp + hex2pp ──────────────────────────────
;; Compile+Link for arch-specific M1 source.
(defn exe (M1-src) (hex2 (catm ELF.hex2 (M0 M1-src))))

;; P1 — portable pseudo-ISA at the M1 level.
;; P1A.M1 is the arch-specific backend.
;; m1pp and hex2pp are themselves P1 programs; after these stages they
;; replace M0 + hex2 for everything downstream.
(define m1pp   (exe (catm P1A.M1 m1pp.P1)))
(define hex2pp (exe (catm P1A.M1 hex2pp.P1)))

;; ── boot2.sh ── Self-host catm + Scheme ──────────────────────────────
;; P1pp — P1 rewritten with m1pp macros. Assemble any P1pp source via m1pp.
;; P1A.M1pp is the arch-specific backend, rewritten to use M1pp.
;; P1.M1pp is the arch-agnostic interface.
;; P1pp.P1pp is "libp1pp", P1pp's standard library, niceties and utilities
;; for programming in P1pp.
(defn ppexe (src)
  (hex2pp (catm ELF.hex2 (m1pp (catm P1A.M1pp P1.M1pp P1pp.P1pp src)))))

;; Rebuild catm from P1pp; after this stage the seed boot0 catm is
;; no longer needed and boot3+ run with only boot1 + boot2 binaries.
(define catm    (ppexe catm.P1pp))
(define scheme  (ppexe scheme1.P1pp))

;; ── boot3.sh / boot4.sh ── C ─────────────────────────────────────────
(defn scc (C-src) (ppexe (scheme cc.scm C-src)))
(define tcc0 (scc tcc.c))   ;; boot3: compiler is scheme cc.scm
(define tcc1 (tcc0 tcc.c))  ;; boot4: compiler is scheme-compiled tcc
(define tcc  (tcc1 tcc.c))  ;; boot4: compiler is tcc-compiled tcc

;; ── boot5.sh ── musl ──────────────────────────────────────────────────
;; tcc + mes-libc compile musl-1.2.5 to produce libc.a + crt{1,i,n}.o.

;; ── boot6.sh ── seed-kernel ───────────────────────────────────────────
;; tcc links the seed-kernel ELF (Image on aarch64). That kernel is the
;; runtime for DRIVER=seed re-runs, closing the bootstrap loop.
```

For a stage-by-stage walk-through of what is built, what becomes
trustable, and what becomes unused at each step, see
[docs/TOUR.md](docs/TOUR.md).

## What you have to trust

The trust boundary has two parts: a small set of **vendored bytes** that
the chain starts from, and a small body of **hand-written source** that
turns those bytes into everything else.

### Vendored bytes (per architecture)

Per arch, seven files from
[live-bootstrap](https://github.com/fosslinux/live-bootstrap)'s
stage0-posix; full provenance in [vendor/seed/README.md](vendor/seed/README.md).
Sizes for `aarch64 / amd64 / riscv64`:

| file        | role                                      | bytes (a/x/r) |
| ----------- | ----------------------------------------- | ------------- |
| `hex0-seed` | the only opaque ELF; assembles `hex0.hex0` | 526 / 229 / 392 |
| `hex0.hex0` | hex assembler — source of `hex1`           | 9763 / 6387 / 8065 |
| `hex1.hex0` | hex assembler with labels                  | 18971 / 10784 / 27080 |
| `hex2.hex1` | hex assembler with ELF-aware linking       | 31017 / 24767 / 39860 |
| `catm.hex2` | concatenates files                         | 6456 / 5468 / 6231 |
| `M0.hex2`   | macro stage above hex2                     | 50189 / 43551 / 65364 |
| `ELF.hex2`  | ELF header preamble                        | 2981 / 2672 / 2661 |

Every one of these except `hex0-seed` is a textual hex file you can
read. `hex0-seed` itself is a few hundred bytes; it is the smallest
opaque artifact in the trust path, and the vendored copies are the same
bytes used by other live-bootstrap consumers.

### Host envelope

You also trust your runtime environment: `sh`, `podman` (or rootless
equivalents), and the `qemu-user-static` binaries for any cross arches. With
`DRIVER=seed`, you trust `qemu-system-<arch>` — but not your host's toolchain,
since the seed driver runs each stage *inside* the kernel that boot6 produced.
Of course, you can also run these directly (without podman) or on bare metal
which would reduce your trust base.

### Hand-written source (LoC)

Lines of code (comments and blanks stripped) that are loaded and
executed during the bootstrap, not counting vendored sources flattened
later in the chain (`tcc-0.9.26`, `musl-1.2.5`):

| layer         | files                                        | LoC   |
| ------------- | -------------------------------------------- | ----- |
| M1pp          | `M1pp/M1pp.P1`                               |  5000 |
| hex2pp        | `hex2pp/hex2pp.P1`                           |  3087 |
| P1            | `P1/{P1.M1pp, P1pp.P1pp, P1-<arch>.M1pp, …}` |  3236 |
| catm          | `catm/catm.P1pp`                             |   105 |
| scheme1       | `scheme1/{scheme1.P1pp, prelude.scm}`        |  4842 |
| cc            | `cc/cc.scm`                                  |  5173 |
| mes-libc      | `vendor/mes-libc/libc.c`                     |  1019 |
| seed-kernel   | `seed-kernel/{kernel.c, arch/<arch>/*}`      |  ~1700 (incl. asm) |

Every layer is small enough to read in an afternoon. The full list of
files crossed by the chain, and the order in which to read them, is in
[docs/TOUR.md](docs/TOUR.md).

## Reading order

* **5 minutes** — the chain pseudocode above, plus
  [docs/TOUR.md §0 "Map"](docs/TOUR.md).
* **An hour** — [docs/TOUR.md](docs/TOUR.md) end to end, then skim
  [boot/boot0.sh](boot/boot0.sh) … [boot/boot6.sh](boot/boot6.sh).
* **A day** — the component specs in dependency order:
  [docs/P1.md](docs/P1.md), [docs/M1PP.md](docs/M1PP.md),
  [docs/HEX2pp.md](docs/HEX2pp.md), [docs/LIBP1PP.md](docs/LIBP1PP.md),
  [docs/SCHEME1.md](docs/SCHEME1.md), [docs/CC.md](docs/CC.md),
  [docs/CCSCM.md](docs/CCSCM.md),
  [docs/LIBC.md](docs/LIBC.md), [docs/MUSL.md](docs/MUSL.md),
  [docs/TCC.md](docs/TCC.md), [docs/OS.md](docs/OS.md).

## Architectures × drivers

`DRIVER={podman,seed} × ARCH={aarch64,amd64,riscv64}`

`DRIVER` selects the runtime that executes each `bootN` stage:

* **podman** (default) — each stage runs in a container with access only to its
  input binaries and sources.
* **seed** — each stage runs inside the tcc-built seed-kernel under
  `qemu-system-<arch>`. Closes the loop: the kernel built by
  `DRIVER=podman` boot6 is the runtime for the next pass. First-time
  setup therefore requires one prior `DRIVER=podman` pass per arch.
  Host needs `qemu-system-<arch>` for the target.

Both drivers write to disjoint trees (`build/<arch>/<driver>/...`), so
they coexist.

## Building

End-to-end via the driver script:

```sh
./boot/boot.sh aarch64                # default DRIVER=podman
DRIVER=seed ./boot/boot.sh aarch64    # re-run on the tcc-built kernel
./boot/boot.sh --help                 # env vars (DRIVER, BOOT*_TIMEOUT, …)
```

Or via path-based Make targets — outputs are the targets, so deps walk
the chain back to source-prep:

```sh
make build/aarch64/podman/boot6/Image            # full chain
make build/amd64/podman/boot6/kernel.elf
make build/riscv64/podman/boot1/M1pp             # only prep-src + boot0 + boot1
make all ARCH=aarch64 DRIVER=podman              # convenience: boot6 kernel
make help                                        # target list
```

Per-stage outputs land at `build/<arch>/<driver>/boot{0..6}/`; the
canonical generated source tree (used by every stage) is at
`build/<arch>/src/`.

## Tests

Suites live in `tests/` with their own Makefile (included from the
top-level). Each suite picks fixtures by `<NNN>-<name>.<ext>` and
diff-checks `<NNN>-<name>.expected` (and optional `.expected-exit`).

```sh
make test                                  # all suites, default arch
make test SUITE=cc                         # one suite
make test SUITE=cc NAMES='001 042'         # filter by fixture prefix
make test SUITE=tcc-cc ARCH=amd64 STAGE=2  # tcc-built test runner
```

Suites: `m1pp`, `p1`, `scheme1`, `cc-util`, `cc-lex`, `cc-pp`, `cc-cg`,
`cc`, `cc-libc`, `cc-ext`, `tcc-cc`, `tcc-libc`. Full per-suite contract
in [tests/README.md](tests/README.md).

`tests/seed-accept.sh` is a separate seed-driver acceptance harness that
diffs seed-built vs podman-built artifacts for byte equivalence; see
that script's header for modes.

## Repository layout

```
boot/        bootN.sh stage drivers + the shared shell DSL
bootprep/    source-tree prep: vendor flatten, run.scm gen, calibration
catm/        catm.P1pp  — concatenator, second tier
M1pp/        M1pp.P1    — macro expander (M1pp.c is the readable C reference)
hex2pp/      hex2pp.P1  — assembler/linker (hex2pp.c is the readable C reference)
P1/          P1 frontend, libp1pp, per-arch backends
scheme1/     scheme1.P1pp + prelude.scm — the Scheme interpreter and its R7RS layer
cc/          cc.scm — the C compiler, in scheme1
tcc/         tcc 0.9.26 patches, mem.c, host-cross asm fallback
seed-kernel/ kernel.c + per-arch boot/MMU + user-mode tests
docs/        component specs and the TOUR
tests/       suite fixtures + harness
vendor/      seed bytes, tcc/mes-libc/musl tarballs (provenance in vendor/*/README.md)
tools/       small helper scripts (count-lines, disasm-elf, …)
```

If you'd like to chat, email me at hi at ryansepassi.com.
