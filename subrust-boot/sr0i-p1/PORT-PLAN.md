# sr0i-p1 — a C-free `sr0i` in portable P1pp

> **STATUS: DONE (2026-07-17).** The P1pp sr0i (lex+parse+eval+main, ~1400 lines,
> built by boot2 m1pp→hex2pp — no C) runs the whole SR-seed test corpus AND the
> sr1i meta-interpreter on emitted images **byte-identical to the frozen C sr0i**
> (`verify-sr0i.sh`: 38 programs + the sr1i image chain). seed→P1pp-sr0i→sr1i, no C.
> Remaining optional finale: boot2-only seed (wire boot2 from-seed m1pp/hex2pp,
> drop stage0/M2-Planet). sr0i.c stays frozen as the oracle.

Port the frozen `subrust-boot/sr0i/sr0i.c` (SR-seed interpreter, 1191 lines,
built by M2-Planet C today) to **P1pp** (boot2's portable pseudo-ISA), so the
whole chain reaches `sr0i` with **no C compiler** — only boot2's seed-built
`m1pp`/`hex2pp`. Portable amd64/aarch64/riscv64 from one source. The frozen
`sr0i.c` is never modified — it is the **byte-exact differential oracle**.

## Strategy: component-first, differentially verified, bounded steps

`sr0i.c` = `lex()` → `parse_*()` (AST via `mk()`) → `eval()`/`run_main()`, over
shared global arrays (tokens, nodes, fns, locals, `wmem`). Port one component per
iteration, each with a component-level rustc reference + a dump-harness, then
assemble `sr0i.P1pp` and verify **end-to-end vs the frozen C sr0i** on the corpus.

Token model: `(kind, ival, pos, len)`. 52 token kinds (T_EOF=0 .. T_KW_CONST=52).
Node model: 15 kinds (N_INT=1 .. N_EXPR=15), fields `a,b,c,d,e,link`. Op ids
OP_ADD=1.. . Memory: MEM_WORDS=2^20; caps SRC 262144, TOK/NODE 65536, FN 1024.

## Steps (one bounded verified step per loop iteration)

1. **Lexer** — `lex.P1pp` (source bytes → token array). Gate `verify-lex.sh`:
   dump `[kind,ival,pos,len]` per token, diff vs `lex-ref.rs` (a faithful Rust
   port of `lex()`) over a token-exhaustive corpus. ← START HERE
2. **Parser** — `parse.P1pp` (tokens → AST). Gate `verify-parse.sh`: dump the
   node arena `[kind,a,b,c,d,e,link]`, diff vs `parse-ref.rs`.
3. **Evaluator core** — `eval.P1pp` for the compute/control/recursion core
   (N_INT/BOOL/NAME/CALL/UNARY/BIN/IF/BLOCK/LET/ASSIGN/WHILE/LOOP/BREAK/CONT),
   plus `do_bin` (reuse the proven `arith-core.P1pp` u64 trap-arithmetic + its
   `udivmod`) and `host_call` (ld/st/getb/putb). Software w_mul/w_div/w_rem.
4. **run_main + const-eval + file I/O main** — read argv[1] via `sys_open`,
   assemble `sr0i.P1pp`.
5. **End-to-end gate** `verify-sr0i.sh`: `sr0i.P1pp` output byte-identical to the
   frozen C `sr0i` (and rustc) over the whole SR-seed corpus + the sr1i image.
   This is the payoff: the two-level stack `seed → P1pp-sr0i → sr1i` with no C.

## Notes / deferrals

- f_* IEEE intrinsics: the C prototype aborts; amd64 lives in `fasm/` (SSE). A
  P1pp sr0i can fold in the per-arch f_* later; integer corpus doesn't need them.
- 128-bit: `p1/arith128-core.P1pp` already done (portable), foldable if needed.
- Build (per component): `cat P1-amd64.M1pp P1.M1pp P1pp.P1pp <mods> | m1pp →
  +ELF.hex2 → hex2pp -B 0x600000`. rustc refs need `--edition 2021`.
