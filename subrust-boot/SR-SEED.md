# SR-seed — the bootstrap rung language (spec)

SR-seed is subrust v0.1 *restricted*, plus the BOOT_API host functions: every
SR-seed program is a valid subrust program
and a valid Rust program, so it runs under rustc (shims appended), under
subrust (`apis::BOOT_API`), and — the point — under `sr0i`, the assembly
interpreter. This spec is frozen; growth happens in subrust proper, never
here.

## Language

- items: `fn name(p: u64, ...) -> u64 | bool | ()` — free functions,
  by-value params, recursion allowed; `const NAME: u64|bool = expr;` — scalar
  constants (added so interpreters like sr1i can be written readably);
- types: `u64`, `bool`, `()` only;
- statements: `let` / `let mut` (initializer required, annotate or let the
  literal adapt), assignment and compound assignment on locals, `if`/`else`,
  `while`, `loop`/`break`/`continue`, expression statements;
- expressions: integer literals (decimal/hex, `u64` range), `true`/`false`,
  calls, parens, `if`/`else` as expression, blocks;
  operators on u64: `+ - * / % & | ^ << >>` (rustc debug semantics: overflow,
  div-by-zero and shift ≥ 64 trap); comparisons; on bool: `&& || ! & | ^`.

Excluded (available one rung up, in the dialect): structs, arrays, `&str`
and all literals of it, `match`, `for`, casts (`as`), `const` items, f64 as
a *type* (bit patterns travel as u64 through the f_* API), every other
integer width, negation (nothing signed exists).

Program shape: any set of functions; execution begins at `fn main()`.
Termination = returning from main. Trap = abort (exit code 1 side effect;
output emitted before the trap stands).

## BOOT_API (host functions; ids = table order)

| id | signature | semantics |
|---|---|---|
| 0 | `fn ld(a: u64) -> u64` | word read; `a >= 2^20` traps |
| 1 | `fn st(a: u64, v: u64)` | word write; same bound |
| 2 | `fn getb() -> u64` | next input byte; `2^64-1` at EOF |
| 3 | `fn putb(b: u64)` | write byte `b & 0xFF` to output |
| 4 | `fn f_add(a: u64, b: u64) -> u64` | IEEE f64 on bit patterns |
| 5 | `fn f_sub(a: u64, b: u64) -> u64` | |
| 6 | `fn f_mul(a: u64, b: u64) -> u64` | |
| 7 | `fn f_div(a: u64, b: u64) -> u64` | |
| 8 | `fn f_rem(a: u64, b: u64) -> u64` | Rust `%` on f64 |
| 9 | `fn f_lt(a: u64, b: u64) -> bool` | IEEE `<` (false on NaN) |
| 10 | `fn f_eq(a: u64, b: u64) -> bool` | IEEE `==` (false on NaN) |
| 11 | `fn f_from_i(a: u64) -> u64` | `(a as i64) as f64`, bits |
| 12 | `fn f_to_i(a: u64) -> u64` | `f64 as i64` (saturating, NaN→0), bits |

Memory: one flat array of 2^20 u64 words (8 MiB), zero-initialized.
f64 negation needs no intrinsic: `x ^ (1 << 63)` is IEEE negate.

## Conformance

- Corpus: `subrust/tests/seed/*.rs`; files named `trap_*` must trap, with
  the output prefix still compared. Optional `.in` = input bytes, optional
  `.out` = hand-verified golden output.
- Harness: `subrust/tests/boot_tests.rs` — every program byte-diffed between
  rustc (shims + same flags as the parity suite) and subrust+BOOT_API.
  `sr0i` joins as the third implementation (`subrust-boot/kaem/`).
- Fuzz: `seed_fuzz_*` in `subrust/tests/fuzz_tests.rs` — the generator's
  SR-seed mode (u64/bool, ld/st round-trips, f_* chains, trap parity).
