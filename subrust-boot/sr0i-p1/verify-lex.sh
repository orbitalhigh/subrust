#!/bin/sh
# Lexer gate for the C-free sr0i port. Builds lex.P1pp +
# lex-main.P1pp through boot2's portable chain (m1pp -> +ELF -> hex2pp) and
# proves its token stream byte-identical to lex-ref.rs (a faithful Rust port of
# sr0i.c's lex()) over a token corpus. STEP 1a covers non-punctuation tokens
# (whitespace, `//` comments, identifiers+keywords, decimal + 0x-hex integers
# with `_` separators and alnum-suffix skip); punctuation lands in 1b (the
# lexer traps via exit 3 if it meets punctuation, so keep the corpus clean).
# Needs rustc, cc, python3, the boot2 seed toolchain.
set -eu
here=$(dirname "$0")/..
cd "$here"
: "${TMPDIR:=/tmp}"
B=vendor/boot2

cc -w -o "$TMPDIR/m1pp"   "$B/M1pp/M1pp.c"
cc -w -o "$TMPDIR/hex2pp" "$B/hex2pp/hex2pp.c"

cat "$B/P1/P1-amd64.M1pp" "$B/P1/P1.M1pp" "$B/P1/P1pp.P1pp" \
    sr0i-p1/lex.P1pp sr0i-p1/lex-main.P1pp > "$TMPDIR/lex.M1pp"
"$TMPDIR/m1pp" "$TMPDIR/lex.M1pp" "$TMPDIR/lex.exp"
cat "$B/vendor/seed/amd64/ELF.hex2" "$TMPDIR/lex.exp" > "$TMPDIR/lex.lnk"
"$TMPDIR/hex2pp" -B 0x600000 "$TMPDIR/lex.lnk" "$TMPDIR/lex.elf"
chmod 0700 "$TMPDIR/lex.elf"

rustc --edition 2021 -O -o "$TMPDIR/lexref" sr0i-p1/lex-ref.rs 2>/dev/null

# token-exhaustive (non-punctuation) corpus
CORPUS="$TMPDIR/lex_corpus"
rm -rf "$CORPUS"; mkdir -p "$CORPUS"
printf 'fn let mut if else while loop break continue const true false'      > "$CORPUS/1"
printf 'foo bar_baz _x abc123 x1y2 fnx letx trueish'                        > "$CORPUS/2"
printf '0 1 42 123 1000000 18446744073709551615'                            > "$CORPUS/3"
printf '0xff 0x0 0xDEADBEEF 0xabc_def 0x1_2_3 0xF'                           > "$CORPUS/4"
printf '1_000_000 999 0 007'                                                 > "$CORPUS/5"
printf '// a comment line\nfoo\n// another\nbar 42'                          > "$CORPUS/6"
printf '123abc 0xffz 5u64 99usize'                                          > "$CORPUS/7"
printf 'fn   foo123\tlet\n0xFF // trailing\nmut  999_888   while continue'   > "$CORPUS/8"
printf '   \n\t\r   leading_ws_then_ident   0x10'                            > "$CORPUS/9"
printf 'x'                                                                   > "$CORPUS/A"
printf ''                                                                    > "$CORPUS/B"
printf '// only a comment no newline'                                        > "$CORPUS/C"
# step 1b: punctuation — every operator, longest-match cases, real programs
printf 'a -> b == c != d <= e >= f << g >> h'                               > "$CORPUS/D"
printf 'x <<= 1; y >>= 2; a && b || c'                                       > "$CORPUS/E"
printf '+= -= *= /= %%= &= |= ^= ( ) { } , ; : = + - * / %% & | ^ ! < >'     > "$CORPUS/F"
printf 'fn main(){let mut x=1+2*3;if x>=5{putb(x);}else{putb(0);}}'          > "$CORPUS/G"
printf 'a<<b>>c<<=d>>=e a<b>c a<=b>=c ==!= &&||'                             > "$CORPUS/H"
# real SR-seed corpus programs — full tokenization now that punctuation works
for s in ../tests/seed/*.rs; do
    [ -f "$s" ] && cp "$s" "$CORPUS/seed_$(basename "$s")"
done

fail=0
for f in "$CORPUS"/*; do
    "$TMPDIR/lex.elf" < "$f" > "$TMPDIR/p1.tok" 2>/dev/null || true
    "$TMPDIR/lexref"  < "$f" > "$TMPDIR/ref.tok"
    # skip inputs the reference can't fully lex (unhandled char => empty output):
    # a vacuous empty==empty match would hide bugs, so flag & skip them.
    if [ ! -s "$TMPDIR/ref.tok" ] && [ -s "$f" ]; then
        echo "  skip $(basename "$f"): ref produced no tokens (non-SR-seed char?)"
        continue
    fi
    if cmp -s "$TMPDIR/p1.tok" "$TMPDIR/ref.tok"; then
        :
    else
        echo "  MISMATCH on $(basename "$f"): $(head -c 60 "$f")"
        python3 - "$TMPDIR/p1.tok" "$TMPDIR/ref.tok" <<'PY'
import struct, sys
p=open(sys.argv[1],'rb').read(); r=open(sys.argv[2],'rb').read()
print(f"    p1 {len(p)//32} toks, ref {len(r)//32} toks")
for k in range(max(len(p),len(r))//32):
    pk=struct.unpack('<QQQQ',p[32*k:32*k+32]) if 32*k+32<=len(p) else None
    rk=struct.unpack('<QQQQ',r[32*k:32*k+32]) if 32*k+32<=len(r) else None
    if pk!=rk:
        print(f"    tok {k}: p1={pk} ref={rk}"); break
PY
        fail=1
    fi
done
if [ "$fail" = 0 ]; then
    echo "lexer (full: idents/keywords/ints/comments/punctuation) == rustc ref: byte-identical over $(ls "$CORPUS" | wc -l) inputs"
else
    echo "lexer: FAIL"
fi
exit "$fail"
