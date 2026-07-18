#!/bin/sh
# P1pp track gate (the "no C rung" ideal): assemble
# arith-core.P1pp through boot2's portable chain and prove its u64
# trap-arithmetic byte-identical to a rustc reference over every trap edge
# plus random vectors. This is the load-bearing semantic core of a P1pp sr0i;
# proving it here de-risks the full interpreter port.
#
# Toolchain: boot2's reference m1pp.c / hex2pp.c, host-built for iteration
# (they are byte-deterministic; the self-hosted-from-seed m1pp/hex2pp that
# boot2's boot0/boot1 produce are the purity upgrade — same output). Needs
# rustc, cc, python3.
set -eu
here=$(dirname "$0")/..
cd "$here"
: "${TMPDIR:=/tmp}"
B=vendor/boot2

cc -w -o "$TMPDIR/m1pp"   "$B/M1pp/M1pp.c"
cc -w -o "$TMPDIR/hex2pp" "$B/hex2pp/hex2pp.c"

# assemble: backend + frontend + libp1pp + src -> m1pp -> +ELF hdr -> hex2pp
cat "$B/P1/P1-amd64.M1pp" "$B/P1/P1.M1pp" "$B/P1/P1pp.P1pp" p1/arith-core.P1pp \
    > "$TMPDIR/ac.M1pp"
"$TMPDIR/m1pp" "$TMPDIR/ac.M1pp" "$TMPDIR/ac.exp"
cat "$B/vendor/seed/amd64/ELF.hex2" "$TMPDIR/ac.exp" > "$TMPDIR/ac.lnk"
"$TMPDIR/hex2pp" -B 0x600000 "$TMPDIR/ac.lnk" "$TMPDIR/arith-core.elf"
chmod 0700 "$TMPDIR/arith-core.elf"

rustc -O -o "$TMPDIR/arith_ref" p1/arith-ref.rs 2>/dev/null

# deterministic vector set: every op x every edge, plus seeded-LCG randoms
python3 - > "$TMPDIR/vectors.bin" <<'PY'
import struct, sys
M=(1<<64)-1
ops=[1,2,3,4,5,8,9,10,11,12,13,14,15,16,17,18]
edge=[0,1,2,7,10,31,63,64,65,255,1000000007,1<<32,(1<<32)-1,1<<63,
      (1<<63)-1,(1<<63)+1,M,M-1,9223372036854775808,18446744073709551557]
st=0x1234567890abcdef
def nxt():
    global st; st=(st*6364136223846793005+1442695040888963407)&M; return st
r=bytearray()
for op in ops:
    for a in edge:
        for b in edge:
            r+=struct.pack('<QQQ',op,a,b)
for _ in range(4000):
    r+=struct.pack('<QQQ',ops[nxt()%len(ops)],nxt(),nxt())
sys.stdout.buffer.write(r)
PY

n=$(( $(wc -c < "$TMPDIR/vectors.bin") / 24 ))
"$TMPDIR/arith-core.elf" < "$TMPDIR/vectors.bin" > "$TMPDIR/p1.out"
"$TMPDIR/arith_ref"      < "$TMPDIR/vectors.bin" > "$TMPDIR/ref.out"
if cmp -s "$TMPDIR/p1.out" "$TMPDIR/ref.out"; then
    echo "P1pp arith-core == rustc: byte-identical over $n records"
else
    echo "MISMATCH (first diff byte):"; cmp "$TMPDIR/p1.out" "$TMPDIR/ref.out" | head -1
    exit 1
fi
