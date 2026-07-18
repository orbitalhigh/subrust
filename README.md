# subrust

A tiny, dependency-free interpreter for a strict subset of Rust — every accepted program is valid Rust and behaves identically under rustc.

*Built in part in preparation for the first issue of [Orbital High](https://orbitalhigh.com), a digital magazine.*

Every program subrust accepts is **valid Rust that behaves identically under
`rustc`** — two laws the test suite enforces differentially:

- **L1** — accepted source compiles under `rustc`.
- **L2** — it produces the same result under `rustc` (debug-profile semantics:
  arithmetic overflow, bounds, and division traps).

subrust is `#![no_std]`, no-alloc, `#![forbid(unsafe_code)]`, and has **zero
dependencies**. It is built for embedding user-editable control scripts in
sandboxed, deterministic, air-gapped systems.

## Applications

Because the same source is *both* an interpretable script and a native Rust
program, subrust fits wherever you want editable logic without giving up
compiled behavior or safety:

- **Edit-time interpretation, release-time compilation.** Iterate on a running
  device — hot-reload a script and see the change immediately through the
  interpreter — then compile the *identical, unchanged* source with `rustc` for
  release: same result (L1/L2), native speed, and no interpreter in the shipped
  binary.
- **Safely running user-supplied scripts.** The machine is sandboxed: no ambient
  clock, RNG, filesystem, or network; every effect goes through host functions
  the embedder registers; and fuel, call-depth, and memory caps bound every run.
  Untrusted logic can only do what you hand it, and always terminates.
- **Embedded / microcontrollers.** `no_std`, no-alloc, `forbid(unsafe)`, and
  zero dependencies mean the whole interpreter fits alongside firmware — a
  field-editable control script runs on the device itself within a fixed, known
  memory budget.
- **Kernel and bare-metal.** For the same reason (no heap, no std, no unsafe) the
  library can run in kernel space or other constrained environments where `alloc`
  is unavailable — user-editable policy where a general-purpose scripting runtime
  could never go.

## Use as a library

```rust
use subrust::{CHK_INIT, MEM_INIT};
use subrust::apis::TEST_API;

let src = "fn main() { print_u64(2 + 2); }";
let mut mem = Box::new(MEM_INIT);
let mut chk = Box::new(CHK_INIT);
assert!(subrust::check_source(src, &mut mem, &mut chk, &TEST_API));
```

A program is loaded with `check_source` (lex + parse + type-check against a host
API), then run through the resumable machine (`subrust::call`), which suspends on
every host call so the embedder stays in control of all effects. Values cross the
boundary as untagged `u64` slots; there is no ambient clock, RNG, filesystem, or
network.

## Command-line tool

```
cargo install subrust
subrust check  file.rs [api]   # type-check against a host API (none|test|hvac)
subrust run    file.rs         # run main() with print_* host functions
subrust lex    file.rs         # dump tokens
subrust ast    file.rs         # dump the parsed tree
subrust emit   file.rs boot    # serialize a checked program to a portable image
```

## The example: a house HVAC controller

`tests/data/hvac.rs` is a self-contained, user-editable control script — an HVAC
thermostat whose operating mode is an `enum`, so the exhaustive `match` the
checker enforces surfaces every site that must change when a mode is added. A
stale or missing sensor reading (`Sensor.known`) holds the system safely idle.

## Language: three nested subsets

subrust is a tower of subsets, each strictly inside the one above, and every rung
is valid Rust:

```
SR-seed  ⊂  SR-intermediate  ⊂  subrust  ⊂  Rust
```

**SR-seed** — the minimal rung. `u64`/`bool` values; arithmetic, comparison, and
bitwise operators; `if` / `while` / `loop` / `break` / `continue`; functions,
`const`s, and recursion; and host calls. No structs, arrays, strings, or enums
(sum types are hand-encoded). This is the language the seed-built interpreter runs.

**SR-intermediate** adds everything needed to *write a type-checker* in it: `Copy`
structs and fixed arrays, `&str` and `&[u8]` with byte indexing, `&`/`&mut`
reference and slice parameters, every integer width (`i8`…`u128`, `usize`),
`T::MAX`/`T::MIN`, named-`const` `match` patterns, `.len()`/`.as_bytes()`, and the
value-model bit builtins (`f64::from_bits`/`.to_bits`, `.wrapping_*`,
`.saturating_*`, `.is_nan`). subrust's own front-end is written entirely here.

**subrust** — the full user-facing language. Everything above, plus `f64`
arithmetic, tuples, inherent `impl` methods, slice indexing and sub-slicing,
`match` with or-patterns, **field-less `enum`s with exhaustive `match`**, and
**`assert!(cond, "msg")`**. Deliberately excluded (and
likely to stay so): closures, generics, traits, the heap, `async`, and `unsafe`.

**Roadmap (short term): `Option<T>` / `Result<T, E>` with full pattern matching.**
Enums landed field-less first; next are payload-carrying variants
(`enum E { A(u64), B }` with binding patterns), and on top of them the built-in
`Option`/`Result` plus `Some`/`None`/`Ok`/`Err`. Then a host API can hand back
`Option<Sensor>` and "forgot to handle the missing case" becomes a compile-time
error instead of a silent `0.0`.

## Bootstrap: reproducible from a 229-byte seed

The whole tower is reconstructed from a **229-byte hex0 seed**, offline, trusting
no pre-existing compiler — driven by `subrust-boot/bootstrap-all.sh`:

1. **Seed → an SR-seed interpreter, two independent ways.** Both start from the
   same 229-byte amd64 hex0 monitor seed (stage0 additionally drives its build
   with a 618-byte `kaem` seed). The **C path** grows the seed through
   stage0-posix → M2-Planet (the classic hex0 → C bootstrap) and builds the
   interpreter in C. The **C-free path** reimplements that same interpreter in
   `boot2`'s portable **P1pp** pseudo-ISA — no C in the interpreter itself — and
   the two builds come out byte-identical, both cross-checked against `rustc`.
2. **SR-seed → subrust.** On top of the interpreter a small meta-interpreter runs
   subrust's own front-end (lexer, parser, type-checker — written in
   SR-intermediate), reproducing subrust's checker from the seed.
3. **subrust → native.** For release the same source is compiled by `rustc`
   (L1/L2) — nothing from the seed chain is in the shipped binary.

```
sh subrust-boot/bootstrap-all.sh
```

Vendored bootstrap toolchains live under `subrust-boot/vendor/` and keep their own
upstream licenses — see [NOTICE](NOTICE).

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your
option. Vendored bootstrap toolchains under `subrust-boot/vendor/` keep their own
licenses — see [NOTICE](NOTICE).
