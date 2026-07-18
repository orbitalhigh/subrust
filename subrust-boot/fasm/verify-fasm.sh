#!/bin/sh
# f_* assembly-backend gate. The sr0i C prototype cannot do the BOOT_API IEEE
# f64 intrinsics — it aborts: die("f_* intrinsics need the assembly backend").
# This gate builds that backend for amd64 FROM THE SEED (seed hex2 + the stage0
# ELF header, no host assembler) and proves it computes real IEEE f64
# byte-for-byte identically to the rustc BOOT_SHIMS reference over a large
# corpus (all 9 ops x specials cross-product + 20k pseudo-random records).
#
# Ops: 0 f_add 1 f_sub 2 f_mul 3 f_div 4 f_rem 5 f_lt 6 f_eq 7 f_from_i 8 f_to_i
# via the same op:u8,a:u64-LE,b:u64-LE / result:u64-LE byte protocol the chain
# already uses (getb/putb discipline). NaN-payload leniency applies ONLY to the
# arithmetic ops (add..rem): IEEE leaves NaN payloads unspecified and a
# commutative fadd lets LLVM pick either operand as the SSE destination, so
# both-NaN counts as equal there; f_lt/f_eq/f_from_i/f_to_i stay strictly
# bit-exact (they never yield NaN).
#
# If `as` (host assembler) is present it is used ONLY as a cross-check that
# f_amd64.s and the committed f_amd64.hex2 bytes are in sync — never for the
# build. Needs: rustc, the seed tools (hex2), python3.
set -eu
here=$(cd "$(dirname "$0")" && pwd)       # subrust-boot/fasm
cd "$here"
: "${TMPDIR:=/tmp}"
BOOT=$(cd "$here/.." && pwd)              # subrust-boot
SHIMS="$BOOT/../tests/common/mod.rs"

# 1. Build the backend from the seed hex2 (no host assembler).
sh build-fasm.sh >/dev/null
echo "  built f_amd64 from the seed hex2 (provenance: seed only)"

# 2. Build the rustc reference. Prefer the live BOOT_SHIMS f_* bodies so the
#    reference can never drift from the language's own float shims; fall back
#    to the vendored fref.rs if the shims file is unavailable.
if [ -f "$SHIMS" ]; then
    {
        echo 'use std::io::{Read, Write};'
        sed -n '/fn f_add(a: u64/,/fn f_to_i(a: u64[^;]*}$/p' "$SHIMS"
        cat <<'RS'
fn main() {
    let mut inp = Vec::new();
    std::io::stdin().read_to_end(&mut inp).unwrap();
    let out = std::io::stdout(); let mut o = out.lock();
    let mut i = 0;
    while i + 17 <= inp.len() {
        let op = inp[i];
        let a = u64::from_le_bytes(inp[i+1..i+9].try_into().unwrap());
        let b = u64::from_le_bytes(inp[i+9..i+17].try_into().unwrap());
        let r = match op {
            0 => f_add(a,b), 1 => f_sub(a,b), 2 => f_mul(a,b), 3 => f_div(a,b),
            4 => f_rem(a,b), 5 => f_lt(a,b) as u64, 6 => f_eq(a,b) as u64,
            7 => f_from_i(a), 8 => f_to_i(a), _ => 0,
        };
        o.write_all(&r.to_le_bytes()).unwrap();
        i += 17;
    }
}
RS
    } > "$TMPDIR/fref_gen.rs"
    REF_SRC="$TMPDIR/fref_gen.rs"
else
    REF_SRC="fref.rs"
fi
rustc --edition 2021 -O "$REF_SRC" -o "$TMPDIR/fref" 2>/dev/null

# 3. Differential: seed-built backend vs rustc reference over the full corpus.
echo "  seed backend vs rustc BOOT_SHIMS reference:"
if python3 harness.py "$TMPDIR/fref" ./f_amd64 > "$TMPDIR/fasm_diff.txt" 2>&1; then
    diff_ok=1
else
    diff_ok=0
fi
sed 's/^/    /' "$TMPDIR/fasm_diff.txt"

# 4. Cross-check (optional): the committed hex2 bytes == `as f_amd64.s` .text.
sync_note="skipped (no host 'as')"
if command -v as >/dev/null 2>&1 && command -v objcopy >/dev/null 2>&1; then
    as --64 f_amd64.s -o "$TMPDIR/f.o" 2>/dev/null
    ld -o "$TMPDIR/f.elf" "$TMPDIR/f.o" 2>/dev/null
    objcopy -j .text -O binary "$TMPDIR/f.elf" "$TMPDIR/f.text" 2>/dev/null
    # code region of the seed ELF = after 64B ELF header + 56B program header.
    n=$(wc -c < "$TMPDIR/f.text")
    if tail -c +121 f_amd64 | head -c "$n" | cmp -s - "$TMPDIR/f.text"; then
        sync_note="OK (f_amd64.s .text == committed f_amd64.hex2 bytes)"
    else
        sync_note="MISMATCH — f_amd64.s and f_amd64.hex2 have diverged"
        diff_ok=0
    fi
fi
echo "  source/hex sync: $sync_note"

if [ "$diff_ok" = 1 ]; then
    echo "f_* assembly backend (amd64): real IEEE f64 from the seed — OK"
    exit 0
else
    echo "f_* assembly backend (amd64): FAIL"
    exit 1
fi
