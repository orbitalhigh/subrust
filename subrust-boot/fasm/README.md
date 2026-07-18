# `fasm/` — the amd64 assembly backend for the BOOT_API f64 intrinsics

The SR-seed interpreter `sr0i` (a M2-Planet C program, built from the 229-byte
hex0 seed) implements the integer BOOT_API — `ld` / `st` / `getb` / `putb` —
but it **cannot** do the IEEE-f64 intrinsics. It aborts on them:

```c
/* f_* : IEEE f64 intrinsics live in per-arch assembly; the
 * integer M2-Planet prototype cannot do them. Abort loudly. */
die("f_* intrinsics need the assembly backend (not in the C prototype)");
```

This directory **is** that assembly backend for amd64 — the per-arch assembly
the C prototype defers on. It executes real hardware IEEE-754 f64 (SSE2 +
one x87 `fprem` loop for the remainder), built with **no host assembler in the
build path**: only the seed-built `hex2`.

## The nine intrinsics

Exactly the BOOT_SHIMS f64 shims from `subrust/tests/common/mod.rs`:

| op | name       | semantics                                   | instruction |
|----|------------|---------------------------------------------|-------------|
| 0  | `f_add`    | `(from_bits(a) + from_bits(b)).to_bits()`   | `addsd`     |
| 1  | `f_sub`    | `... - ...`                                  | `subsd`     |
| 2  | `f_mul`    | `... * ...`                                  | `mulsd`     |
| 3  | `f_div`    | `... / ...`                                  | `divsd`     |
| 4  | `f_rem`    | Rust `%` = C `fmod` (sign of dividend)       | x87 `fprem` |
| 5  | `f_lt`     | `from_bits(a) < from_bits(b)` (NaN→false)    | `comisd`    |
| 6  | `f_eq`     | `from_bits(a) == from_bits(b)` (NaN→false)   | `ucomisd`   |
| 7  | `f_from_i` | `((a as i64) as f64).to_bits()`              | `cvtsi2sd`  |
| 8  | `f_to_i`   | `(from_bits(a) as i64) as u64` — SATURATING  | `cvttsd2si` + saturate |

`f_to_i` reproduces Rust's saturating float→int cast by hand: `cvttsd2si`
yields the x86 "integer indefinite" `0x8000000000000000` on NaN / ±∞ /
out-of-range, which the code then resolves to `0` (NaN), `i64::MAX` (positive
overflow) or `i64::MIN` (negative overflow), while leaving a genuine
`-2^63` untouched.

## Protocol

Same byte discipline (`getb`/`putb`) as the rest of the seed chain, so it is
driven and tested through stdin/stdout like every other rung:

```
stdin:  repeated 17-byte records — op:u8, a:u64-LE, b:u64-LE
stdout: 8 bytes per record       — result:u64-LE
EOF at a record boundary -> exit 0
```

## Files

- **`f_amd64.s`** — human-readable GAS source; the thing you audit.
- **`f_amd64.hex2`** — the audited machine-code bytes (each line = bytes +
  disassembly comment). This is what actually gets built; it is concatenated
  after `vendor/stage0-posix/AMD64/ELF-amd64.hex2` (the standard stage0 64-bit
  ELF header, `e_entry = &_start`) and linked by the seed `hex2`. The code is
  position-independent (relative jumps, register/stack operands, immediate
  loads — no absolute or RIP-relative data refs), so these bytes are wrapped
  verbatim with no relocation.
- **`build-fasm.sh`** — seed build: `hex2` + the ELF header → `f_amd64`. No
  host compiler.
- **`verify-fasm.sh`** — the gate (below).
- **`fref.rs`, `harness.py`** — the differential test (reference + driver).

## Verification (`verify-fasm.sh`)

1. Build `f_amd64` from `f_amd64.hex2` with the **seed** `hex2` — provenance is
   seed-only.
2. Build a rustc reference straight from the **live** BOOT_SHIMS f64 bodies in
   `tests/common/mod.rs` (so it cannot drift from the language's own shims).
3. Feed both the same corpus — all 9 ops over a cross-product of ~50 special
   bit-patterns (0, ±0, ±∞, NaN, sNaN, subnormals, DBL_MAX/MIN, 2^53, ±2^63,
   i64::MIN/MAX, …) plus 20 000 pseudo-random records (~48 k total) — and
   require **byte-identical** results.
4. If a host `as` is present, cross-check that `f_amd64.s` still assembles to
   the exact bytes committed in `f_amd64.hex2` (a sync check only — never part
   of the build).

**NaN-payload leniency** applies to the arithmetic ops (`add`..`rem`) only:
IEEE leaves NaN payloads unspecified, and because `fadd` is commutative LLVM
may place either operand as the SSE destination, so Rust's NaN payload can
differ from this backend's fixed operand order. Two NaN results therefore count
as equal there. `f_lt`/`f_eq`/`f_from_i`/`f_to_i` never yield NaN and stay
strictly bit-exact.

## Scope — what this is and isn't

This backend supplies the **f64 intrinsics** the C prototype defers on, and
proves they run as real IEEE-754 from seed-assembled machine code. It is a
standalone module exercised through the same byte protocol `sr0i` uses; wiring
it *inside* a full assembly `sr0i` (porting the ~1200-line C interpreter loop
to amd64 assembly, which would let f64 programs run end-to-end on a pure-asm
chain) remains the larger deferred work — `sr0i.c` itself stays frozen. The
128-bit integer intrinsics (today's `KTRAP` serialize-only stubs) are a natural
next set of ops in the same shape (two-slot add/sub/mul with carry) but are not
built here.

Only amd64 is implemented. Other arches (aarch64, riscv64) would each get a
sibling `f_<arch>.s` / `.hex2` verified by the same harness.
