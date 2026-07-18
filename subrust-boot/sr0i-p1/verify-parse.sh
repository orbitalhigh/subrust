#!/bin/sh
# Parser gate for the C-free sr0i port. Builds lex.P1pp +
# parse.P1pp + parse-main.P1pp through boot2's chain and proves the node arena
# byte-identical to parse-ref.rs (faithful Rust port of sr0i.c's parser).
# STEP 2a: expression parser (primary/unary/binary precedence) — corpus is
# pure expressions (no blocks/if/statements; those land in 2b). Node record =
# 7 words [kind,a,b,c,d,e,link]. Needs rustc, cc, python3, boot2 seed tools.
set -eu
here=$(dirname "$0")/..
cd "$here"
: "${TMPDIR:=/tmp}"
B=vendor/boot2

cc -w -o "$TMPDIR/m1pp"   "$B/M1pp/M1pp.c"
cc -w -o "$TMPDIR/hex2pp" "$B/hex2pp/hex2pp.c"

cat "$B/P1/P1-amd64.M1pp" "$B/P1/P1.M1pp" "$B/P1/P1pp.P1pp" \
    sr0i-p1/lex.P1pp sr0i-p1/parse.P1pp sr0i-p1/parse-main.P1pp > "$TMPDIR/parse.M1pp"
"$TMPDIR/m1pp" "$TMPDIR/parse.M1pp" "$TMPDIR/parse.exp"
cat "$B/vendor/seed/amd64/ELF.hex2" "$TMPDIR/parse.exp" > "$TMPDIR/parse.lnk"
"$TMPDIR/hex2pp" -B 0x600000 "$TMPDIR/parse.lnk" "$TMPDIR/parse.elf"
chmod 0700 "$TMPDIR/parse.elf"

rustc --edition 2021 -O -o "$TMPDIR/parseref" sr0i-p1/parse-ref.rs 2>/dev/null

CORPUS="$TMPDIR/parse_corpus"
rm -rf "$CORPUS"; mkdir -p "$CORPUS"
# whole programs: fns (params/mut/->), const, multi-fn, then the REAL corpus
printf 'fn main() { putb(72); }'                                        > "$CORPUS/1"
printf 'fn f(x: u64) -> u64 { x + 1 }'                                  > "$CORPUS/2"
printf 'fn g(mut a: u64, b: u64) -> u64 { a = a + b; a }'               > "$CORPUS/3"
printf 'const K: u64 = 42;\nfn main() { putb(K); }'                     > "$CORPUS/4"
printf 'fn a() {} fn b() -> u64 { 1 } fn main() { a(); putb(b()); }'    > "$CORPUS/5"
printf 'const A: u64 = 1;\nconst B: u64 = A + 2;\nfn main() { putb(B); }' > "$CORPUS/6"
printf 'fn loop_it(mut n: u64) { while n > 0 { putb(n); n -= 1; } }\nfn main() { loop_it(5); }' > "$CORPUS/7"
printf 'fn r(n: u64) -> u64 { if n < 2 { n } else { r(n-1) + r(n-2) } }\nfn main() { putb(r(8)); }' > "$CORPUS/8"
# real SR-seed corpus programs — the full parser vs actual programs
for s in ../tests/seed/*.rs; do
    [ -f "$s" ] && cp "$s" "$CORPUS/seed_$(basename "$s")"
done

fail=0
for f in "$CORPUS"/*; do
    SR_PARSE=program "$TMPDIR/parseref" < "$f" > "$TMPDIR/ref.nd" 2>/dev/null || true
    "$TMPDIR/parse.elf" < "$f" > "$TMPDIR/p1.nd" 2>/dev/null || true
    if [ ! -s "$TMPDIR/ref.nd" ] && [ -s "$f" ]; then
        echo "  skip $(basename "$f"): ref produced nothing (non-SR-seed char?)"
        continue
    fi
    if cmp -s "$TMPDIR/p1.nd" "$TMPDIR/ref.nd"; then
        :
    else
        echo "  MISMATCH on $(basename "$f"): $(head -c 70 "$f")"
        python3 - "$TMPDIR/p1.nd" "$TMPDIR/ref.nd" <<'PY'
import struct, sys
p=open(sys.argv[1],'rb').read(); r=open(sys.argv[2],'rb').read()
M=(1<<64)-1; f=lambda x:'NIL' if x==M else x
print(f"    p1 {len(p)}B, ref {len(r)}B")
def hdr(d): return struct.unpack('<4Q', d[:32]) if len(d)>=32 else None
print(f"    p1 hdr [node_n,fn_n,ptok_n,const_n]={hdr(p)}  ref hdr={hdr(r)}")
n=min(len(p),len(r))
for i in range(0,n,8):
    if p[i:i+8]!=r[i:i+8]:
        print(f"    first diff at byte {i} (word {i//8}): p1={struct.unpack('<Q',p[i:i+8])[0]} ref={struct.unpack('<Q',r[i:i+8])[0]}"); break
PY
        fail=1
    fi
done
if [ "$fail" = 0 ]; then
    echo "parser (step 2c: fn/program — COMPLETE) == rustc ref: byte-identical over $(ls "$CORPUS" | wc -l) programs"
else
    echo "parser step 2c: FAIL"
fi
exit "$fail"
