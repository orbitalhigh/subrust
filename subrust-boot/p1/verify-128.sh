#!/bin/sh
# P1pp 128-bit core gate (the "no C rung" ideal): assemble
# arith128-core.P1pp through boot2's portable chain and prove its 128-bit
# two-slot arithmetic byte-identical to a rustc reference (arith128-ref.rs =
# ce_bin128/wrap_prim128/sat_prim128/un_op128 semantics) over every trap edge
# plus random vectors. This is what the C prototype punts on (128-bit = KTRAP
# serialize-only); proving it here in portable P1pp is a down-payment on a
# C-free sr0i. Same toolchain notes as p1/verify.sh. Needs rustc, cc, python3.
set -eu
here=$(dirname "$0")/..
cd "$here"
: "${TMPDIR:=/tmp}"
B=vendor/boot2

cc -w -o "$TMPDIR/m1pp"   "$B/M1pp/M1pp.c"
cc -w -o "$TMPDIR/hex2pp" "$B/hex2pp/hex2pp.c"

# assemble: backend + frontend + libp1pp + src -> m1pp -> +ELF hdr -> hex2pp
cat "$B/P1/P1-amd64.M1pp" "$B/P1/P1.M1pp" "$B/P1/P1pp.P1pp" p1/arith128-core.P1pp \
    > "$TMPDIR/a128.M1pp"
"$TMPDIR/m1pp" "$TMPDIR/a128.M1pp" "$TMPDIR/a128.exp"
cat "$B/vendor/seed/amd64/ELF.hex2" "$TMPDIR/a128.exp" > "$TMPDIR/a128.lnk"
"$TMPDIR/hex2pp" -B 0x600000 "$TMPDIR/a128.lnk" "$TMPDIR/arith128-core.elf"
chmod 0700 "$TMPDIR/arith128-core.elf"

rustc --edition 2021 -O -o "$TMPDIR/arith128_ref" p1/arith128-ref.rs 2>/dev/null

# deterministic vector set: every implemented op x every 128-bit edge pair,
# plus seeded-LCG randoms. Edit OPS as groups land in arith128-core.P1pp.
OPS="${OPS:-1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32 33 34 35 36 37 38 39}"
python3 - "$OPS" > "$TMPDIR/vec128.bin" <<'PY'
import struct, sys
M = (1 << 64) - 1
ops = [int(x) for x in sys.argv[1].split()]
# interesting 128-bit values as (lo, hi) word pairs
def w(v): return (v & M, (v >> 64) & M)
vals = [w(x) for x in [
    0, 1, 2, 3, 0xff, 1 << 32, (1 << 32) - 1, 1 << 63, (1 << 63) - 1,
    1 << 64, (1 << 64) - 1, 1 << 100, 1 << 126, 1 << 127, (1 << 127) - 1,
    (1 << 127) + 1, (1 << 128) - 1, (1 << 128) - 2, 0x8000000000000000,
    (1 << 64) | 1, (1 << 127) | 7, 0x0123456789abcdef, 12345678901234567890,
]]
st = 0x1234567890abcdef
def nxt():
    global st; st = (st * 6364136223846793005 + 1442695040888963407) & M; return st
r = bytearray()
for op in ops:
    for (alo, ahi) in vals:
        for (blo, bhi) in vals:
            r += struct.pack('<QQQQQ', op, alo, ahi, blo, bhi)
for _ in range(6000):
    op = ops[nxt() % len(ops)]
    r += struct.pack('<QQQQQ', op, nxt(), nxt(), nxt(), nxt())
sys.stdout.buffer.write(r)
PY

n=$(( $(wc -c < "$TMPDIR/vec128.bin") / 40 ))
"$TMPDIR/arith128-core.elf" < "$TMPDIR/vec128.bin" > "$TMPDIR/p128.out"
"$TMPDIR/arith128_ref"      < "$TMPDIR/vec128.bin" > "$TMPDIR/ref128.out"
if cmp -s "$TMPDIR/p128.out" "$TMPDIR/ref128.out"; then
    echo "P1pp arith128-core == rustc: byte-identical over $n records  (ops: $OPS)"
else
    echo "MISMATCH (first diff byte):"; cmp "$TMPDIR/p128.out" "$TMPDIR/ref128.out" | head -1
    # show the offending record for debugging
    python3 - "$TMPDIR/p128.out" "$TMPDIR/ref128.out" "$TMPDIR/vec128.bin" <<'PY'
import struct, sys
p = open(sys.argv[1],'rb').read(); r = open(sys.argv[2],'rb').read(); v = open(sys.argv[3],'rb').read()
for k in range(min(len(p),len(r))//24):
    if p[24*k:24*k+24] != r[24*k:24*k+24]:
        op,alo,ahi,blo,bhi = struct.unpack('<QQQQQ', v[40*k:40*k+40])
        pf = struct.unpack('<QQQ', p[24*k:24*k+24]); rf = struct.unpack('<QQQ', r[24*k:24*k+24])
        print(f"  record {k}: op={op} a=({alo:#x},{ahi:#x}) b=({blo:#x},{bhi:#x})")
        print(f"    p1 =(flag={pf[0]} lo={pf[1]:#x} hi={pf[2]:#x})")
        print(f"    ref=(flag={rf[0]} lo={rf[1]:#x} hi={rf[2]:#x})")
        break
PY
    exit 1
fi
