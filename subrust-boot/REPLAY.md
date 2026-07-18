# subrust-boot — replay log and pins

Bootstrap track. Status: the vendored tools, the SR-seed spec, and the C
`sr0i` interpreter are done; the P1pp (no-C) `sr0i` is in progress — the
SR-seed interpreter builds from the 256-byte seed and runs the corpus
byte-identically to rustc and subrust. 2026-07.

## Build everything from hex0, timed

`sh bootstrap-all.sh [--clean] [--no-rustc]` compiles from the 229-byte
hex0-seed and prints a per-phase timing table, tagging each phase's
provenance (`[SEED]` = whole tool chain is the seed; `[boot2]` = boot2's
portable chain, toolchain host-cc; `[rustc]` = depends on rustc).

Measured (this host, x86-64):

```
[SEED ] stage0-posix: seed -> hex0/1/2 -> M0 -> M2-Planet + tools   ~16.8s
[SEED ] sr0i: SR-seed interpreter built from the seed (M2-Planet)    ~0.5s
[rustc] sr0i corpus: 3-way byte-identical (rustc / cc / seed-sr0i)   ~2.7s
[boot2] P1pp arith-core == rustc over 10,400 vectors                 ~0.3s
[rustc] subrust + subrust-cli (cargo release build)                 ~2.3s
[rustc] full subrust: check the example script against its host API  ~0.0s
```

**From the seed to a running SR-seed interpreter: ~17.3s** (cold, `--clean`).
That is the honest "from hex0" number today — it reaches `sr0i`, the SR-seed
interpreter. "Full subrust" (the real checker+machine on the example script)
is still the rustc-built phases above; reaching it FROM THE SEED needs the
two-level stack (`sr1i`) plus subrust's own source on the chain, not yet
built. When those land, the rustc phases are replaced by `sr1i`-on-`sr0i`
running subrust's own source, and this same script measures the true
seed→full-subrust time. (For scale: the community rustc bootstrap —
live-bootstrap→GCC→mrustc→rustc — is days, versus tens of seconds here to the
SR-seed stage.)

## Vendored tools and replay

### stage0-posix (vendor/stage0-posix)

Pinned: `stage0-posix @ 643598041bf7639883874fe2cdc9d9693c9b03d5` with
submodules (git submodule status):

```
9015b9e0 AArch64          82efa0d6 AMD64            4b011a85 M2-Mesoplanet
bd2fe4b0 M2-Planet        68a23cfd M2libc           4b1ff94c armv7l
cedec6b8 bootstrap-seeds  5adfbf33 mescc-tools      a151c245 mescc-tools-extra
261c6727 riscv32          4688bc66 riscv64          3b9c2bb6 x86
```

Binary seeds (the ONLY binaries in the chain), SHA-256:

```
66c95985e668f20f2465c2b876f83fef066fd7c8c2dd3adb51a969f2d7120c8b  hex0-seed          (256 B)
153b8915b73bd07132b59538d10fe53d26578eb160a67db72af07aaa61c51b3b  kaem-optional-seed (618 B)
```

Replay (x86-64 Linux, no shell needed beyond exec):

```
cd vendor/stage0-posix
./bootstrap-seeds/POSIX/AMD64/kaem-optional-seed kaem.amd64
```

Replayed 2026-07-16: exit 0; every artifact verified `OK` against
`amd64.answers` by the chain's own bootstrapped sha256sum. Products in
`AMD64/bin/`: hex2, M1, kaem, blood-elf, catm, M2-Planet, M2-Mesoplanet,
sha256sum, and the file utilities. `cleanup.sh` restores pristine.

The sr0i build uses only the *early* products: M1 (macro assembler),
hex2 (assembler/linker), blood-elf (ELF debug stubs), kaem (build driver) —
i.e. nothing past stage0's phase 10; M2-Planet is replayed for hash
verification but not used by our stages.

### boot2 (vendored)

boot2 — Ryan Sepassi's P1/M1pp/P1pp portable layer — pinned snapshot in
`vendor/boot2/` (sources + docs + amd64/aarch64/riscv64 seed ELF headers;
`COMMIT.txt` = 47e13ea04f3c6287b8aab1f6d125a7596d309047). See
<https://ryansepassi.com/notes/boot2> for boot2 and its source. GPL/boot2-
licensed build tools, run as separate processes — nothing links.

What P1 gives that M2-Planet didn't: a portable pseudo-ISA with **native
64-bit mul/div/rem** and `bltu` (unsigned compare), registers a0-a3/t0-t2/
s0-s3, a fixed calling convention, and M1pp macros (named params, structs,
stack frames, compile-time expressions, hygienic local labels). Verified in
this sandbox: `hello.P1pp` and a 64-bit mul/div/rem program assemble via
boot2's reference `m1pp.c`/`hex2pp.c` (host-built) and run correctly. One
gap: P1 `div`/`rem` are SIGNED only (no `divu`/`remu`), so SR-seed's unsigned
division needs software binary long division (via `bltu`) — see the P1pp
arith core below.

## SR-seed

- Spec: `SR-SEED.md` (this directory).
- Host API: `apis::BOOT_API` in the subrust crate (ld/st/getb/putb + f_*).
- Corpus: `subrust/tests/seed/*.rs` (+ `.in`/`.out`), harness
  `subrust/tests/boot_tests.rs` — every program runs under rustc (shim
  appended) AND under subrust+BootHost, byte-diffed, trap-diffed.
- Fuzz: `seed_fuzz_*` in `subrust/tests/fuzz_tests.rs` — SR-seed-restricted
  generator, same differential harness.

## sr0i — the SR-seed interpreter (done)

`sr0i/sr0i.c` — an SR-seed interpreter in the M2-Planet C subset (~900
lines): fixed pools + integer indices (subrust's own dialect style, which is
also what keeps it in-subset), tree-walking, rustc-debug trap semantics.

Scope decisions, documented honestly:
- **Written in C, not P1pp assembly.** The ideal is P1pp (no C rung at all),
  but boot2 was not vendorable at first. M2-Planet is seed-built and already
  in the trusted base, so using its C subset does not grow that base; the
  P1pp rewrite stays a north-star refinement. Develop with host `cc`, then
  cross-build the identical source with seed-built M2-Planet.
- **f_* deferred.** M2-Planet is integer-only, and the IEEE-f64 intrinsics
  are per-arch *assembly* backends anyway. sr0i covers the SR-seed language +
  ld/st/getb/putb; `floats.rs` stays a rustc/subrust-only differential until
  the backend intrinsics land.
- **Software 64-bit mul/div/rem.** Probed constraint: M2-Planet amd64 makes
  add/sub/shift/bitwise/unsigned-compare correct at 64-bit, but emits 32-bit
  **mul/div**. So sr0i implements u64 multiply (16-bit limbs) and divide
  (binary long division) with only the 64-bit-correct primitives — one code
  path, right on host cc and M2-Planet alike (`sr0i.c` `w_mul`/`w_div`/
  `w_rem`). This is the bootstrap in miniature: the seed compiler gives you
  32-bit arithmetic; you build 64-bit on top.

Build (seed tools only): `sh kaem/build-sr0i.sh` (drives seed-built
M2-Mesoplanet → M2-Planet → blood-elf → M1 → hex2). Seed-built artifact
`sr0i/sr0i`, amd64 ELF, ~100 KB.

**Exit criterion met.** `sh verify.sh`: the 12 non-float corpus programs are
byte-identical with matching trap outcomes across THREE implementations —
rustc (BOOT_SHIMS), sr0i built with host cc, and **sr0i built from the seed
via M2-Planet**. subrust itself is verified == rustc by `cargo test --test
boot_tests`; the seed-fuzz mode (`cargo test --release seed_fuzz_soak --
--ignored`, 40 seeds) keeps subrust == rustc broad. The seed-built sr0i and
its host-cc twin agree, so all four executions of SR-seed coincide.

Note: `sr0i/sr0i` and `vendor/*/AMD64/bin` are build products (gitignored);
rebuild via the vendored-tools replay + `build-sr0i.sh`. sha256 of the
seed-built binary is toolchain/host-deterministic; recorded in-run by
`build-sr0i.sh`.

## sr0i in P1pp (the "no C rung" ideal — done)

Now that boot2 is vendored, sr0i has been rewritten in P1pp. It replaces the
M2-Planet C prototype with portable pseudo-ISA source assembled by boot2's
`M1pp → hex2pp` chain — no C compiler in the interpreter's provenance, and
portable to amd64/aarch64/riscv64 by swapping the P1 backend file.

**The load-bearing semantic core.** `p1/arith-core.P1pp` implements SR-seed's
u64 trap-arithmetic (`do_bin`): add/sub/mul/div/rem/shift/bitwise/compare with
rustc debug-profile trap semantics, including software **unsigned divmod**
(binary long division via `bltu`, since P1 div/rem are signed). `sh
p1/verify.sh`: byte-identical to a rustc reference (`p1/arith-ref.rs`) over
**10,400 records** — every op × every trap edge (0/1/2³²/2⁶³/2⁶⁴−1, shift
63/64/65, div0, mul overflow) + 4000 seeded-random triples. This is the piece
most likely to hide subtle bugs; nailing it in P1pp de-risked the port.

**The full interpreter.** `sr0i-p1/` completes the port: a P1pp lexer, a
recursive-descent parser, and a tree-walking evaluator plus `main` (~1400
lines total, reusing the arith core above), assembled entirely by boot2's
`m1pp`/`hex2pp` — no C anywhere in its provenance. The frozen `sr0i.c` stays as
the byte-exact differential oracle. `sh sr0i-p1/verify-sr0i.sh`: the P1pp sr0i
is byte-identical to the seed-built C sr0i over **38 programs** — the whole
SR-seed corpus plus the two-level `sr1i` image chain (seed → P1pp-sr0i → sr1i,
no C). f_* intrinsics stay per-arch backends (deferred, as in C); the integer
corpus doesn't need them. What remains is an optional finale: driving boot2's
own `m1pp`/`hex2pp` from its seed so the chain is boot2-only and
stage0/M2-Planet become droppable.

Purity note: the arith-core gate uses boot2's reference `m1pp.c`/`hex2pp.c`
(host-built for iteration). The self-hosted-from-seed `m1pp`/`hex2pp` that
boot2's boot0/boot1 pipeline produces are byte-deterministic equivalents; the
seed-provenance upgrade is running boot2's boot0/boot1 (needs its seed kernel
/ container), tracked but not blocking the semantic work.

## sr1i — the two-level interpreter stack (mechanism done)

`sr1i` is a meta-circular interpreter for SR-seed, **written in SR-seed**
(sr1i/sr1i.rs, ~300 lines): it reads a checked program image (`subrust emit`,
subrust-cli/src/emit.rs) from stdin and evaluates it, so the stack is
**seed -> sr0i -> sr1i -> program**. Because sr1i is itself SR-seed it is
also valid Rust, verified under rustc too — the same triple differential.

`sh sr1i/verify.sh`: 8 programs (hello/arith/loops/bools/recurse +
trap_overflow/div0/shift) run byte-identically with matching trap outcomes
across program-direct-on-sr0i, rustc-sr1i, and **seed-built-sr0i -> sr1i**.
recurse (fib/gcd/pow) exercises recursion + traps through TWO interpreter
levels; the trap_* programs confirm sr1i predicts guest overflow/div0/shift
and signals it correctly.

This proves the load-bearing mechanism — an interpreter written in SR-seed,
running on sr0i. Two enabling changes: `const` added to SR-seed
(SR-SEED.md) + sr0i.c (with the fix that `const_val` must be the 64-bit WORD,
not 32-bit `unsigned` — a truncation bug the two-level stack surfaced); and
`subrust emit`, which serializes a *checked* SR-seed program to a flat image
(the program-image format) so sr1i needs no lexer/parser/checker.

Scope at this stage: the compute/control/recursion core + putb. getb/ld/st
and f_* are deferred (guest-memory virtualization); structs/arrays/f64 are
dialect, beyond SR-seed. So sr1i is NOT yet the full-dialect interpreter that
subrust's own source needs — see below.

## subrust's own source on the chain (not yet built)

The remaining distance to "full subrust from the seed": sr1i must grow from
the SR-seed subset to the full bootstrap DIALECT (structs, fixed arrays,
`&`/`&mut`/slices, methods, `match`, all integer widths, f64 via intrinsics,
exact f64 decimal parsing) — which subrust's ~7k-line source is written in —
AND subrust must first implement the additional language features that source
uses. That is the main remaining work, genuinely months of it; the mechanism
above is the proof that the interpreter-on-the-chain approach works, not a
shortcut around it.
