#!/bin/sh
# End-to-end gate for the C-free sr0i. Builds the P1pp sr0i
# (lex+parse+eval) through boot2's chain and proves its program OUTPUT
# byte-identical to the FROZEN C sr0i (the oracle) over a corpus. The C sr0i
# reads the program from argv[1]; the P1pp sr0i reads it from stdin — same
# output. STEP 3a: putb + straight-line integer arithmetic (no vars/control/
# user-calls yet). Needs cc, python3, boot2 seed tools, and the seed-built C
# sr0i (built via kaem/build-sr0i.sh if absent).
set -eu
here=$(dirname "$0")/..
cd "$here"
: "${TMPDIR:=/tmp}"
B=vendor/boot2

cc -w -o "$TMPDIR/m1pp"   "$B/M1pp/M1pp.c"
cc -w -o "$TMPDIR/hex2pp" "$B/hex2pp/hex2pp.c"
cat "$B/P1/P1-amd64.M1pp" "$B/P1/P1.M1pp" "$B/P1/P1pp.P1pp" \
    sr0i-p1/lex.P1pp sr0i-p1/parse.P1pp sr0i-p1/eval.P1pp sr0i-p1/main.P1pp > "$TMPDIR/sr0ip1.M1pp"
"$TMPDIR/m1pp" "$TMPDIR/sr0ip1.M1pp" "$TMPDIR/sr0ip1.exp"
cat "$B/vendor/seed/amd64/ELF.hex2" "$TMPDIR/sr0ip1.exp" > "$TMPDIR/sr0ip1.lnk"
"$TMPDIR/hex2pp" -B 0x600000 "$TMPDIR/sr0ip1.lnk" "$TMPDIR/p1-sr0i"
chmod 0700 "$TMPDIR/p1-sr0i"

# the frozen C sr0i (oracle) — build from the seed toolchain if not present
[ -x sr0i/sr0i ] || sh kaem/build-sr0i.sh >/dev/null 2>&1
CSR0I="$PWD/sr0i/sr0i"

CORPUS="$TMPDIR/sr0i_corpus"
rm -rf "$CORPUS"; mkdir -p "$CORPUS"
printf 'fn main() { putb(72); }'                            > "$CORPUS/1"
printf 'fn main() { putb(48 + 1 * 2); }'                    > "$CORPUS/2"
printf 'fn main() { putb(65); putb(66); putb(67); }'        > "$CORPUS/3"
printf 'fn main() { putb(100 - 55); putb(7 * 7); }'         > "$CORPUS/4"
printf 'fn main() { putb(255 & 72); putb(64 | 8); }'        > "$CORPUS/5"
printf 'fn main() { putb(200 / 3); putb(200 %% 7); }'        > "$CORPUS/6"
printf 'fn main() { putb(1 << 6); putb(256 >> 2); }'        > "$CORPUS/7"
printf 'fn main() { putb(10 ^ 3); putb(!0 & 74); }'         > "$CORPUS/8"
printf 'fn main() { putb((5 + 3) * 9); }'                   > "$CORPUS/9"
printf 'fn main() { putb(72); putb(10); }'                  > "$CORPUS/A"
# step 3b: locals + control flow
printf 'fn main() { let x = 65; putb(x); }'                                       > "$CORPUS/B"
printf 'fn main() { let mut x = 65; x = x + 1; putb(x); }'                        > "$CORPUS/C"
printf 'fn main() { let mut s = 0; s += 10; s += 5; putb(s + 33); }'             > "$CORPUS/D"
printf 'fn main() { let x = 5; if x > 3 { putb(89); } else { putb(78); } }'      > "$CORPUS/E"
printf 'fn main() { let mut i = 65; while i < 70 { putb(i); i = i + 1; } }'      > "$CORPUS/F"
printf 'fn main() { let mut n = 5; while n > 0 { putb(48 + n); n -= 1; } }'      > "$CORPUS/G"
printf 'fn main() { let mut i = 0; loop { putb(65 + i); i += 1; if i == 3 { break; } } }' > "$CORPUS/H"
printf 'fn main() { let mut i = 0; while i < 6 { i += 1; if i == 3 { continue; } putb(64 + i); } }' > "$CORPUS/I"
printf 'fn main() { let mut r = 0; let mut i = 1; while i <= 4 { r += i; i += 1; } putb(r + 64); }' > "$CORPUS/J"
printf 'fn main() { let x = 2; if x == 1 { putb(65); } else if x == 2 { putb(66); } else { putb(67); } }' > "$CORPUS/K"
# step 3c: user function calls + recursion
printf 'fn emit(c: u64) { putb(c); } fn main() { emit(72); emit(73); }'                          > "$CORPUS/L"
printf 'fn add(a: u64, b: u64) -> u64 { a + b } fn main() { putb(add(40, 32)); }'                > "$CORPUS/M"
printf 'fn fac(n: u64) -> u64 { if n == 0 { 1 } else { n * fac(n - 1) } } fn main() { putb(fac(4) + 42); }' > "$CORPUS/N"
printf 'fn fib(n: u64) -> u64 { if n < 2 { n } else { fib(n-1) + fib(n-2) } } fn main() { putb(fib(10) + 10); }' > "$CORPUS/O"
printf 'fn dbl(x: u64) -> u64 { x + x } fn main() { putb(dbl(dbl(18))); }'                       > "$CORPUS/P"
printf 'fn g(a: u64, b: u64) -> u64 { if b == 0 { a } else { g(b, a %% b) } } fn main() { putb(g(1000, 24) + 56); }' > "$CORPUS/Q"
# step 3d: the WHOLE real SR-seed corpus (ld/st/getb/const). floats.rs uses '.'
# which neither sr0i handles, so skip it.
for s in ../tests/seed/*.rs; do
    case "$(basename "$s")" in floats.rs) continue ;; esac
    [ -f "$s" ] && cp "$s" "$CORPUS/seed_$(basename "$s" .rs)"
done

fail=0
for f in "$CORPUS"/*; do
    # per-program stdin: a seed_X program uses tests/seed/X.in if it exists.
    inp=/dev/null
    bn=$(basename "$f")
    case "$bn" in seed_*) cand="../tests/seed/${bn#seed_}.in"; [ -f "$cand" ] && inp="$cand" ;; esac
    co=$("$CSR0I" "$f" < "$inp" 2>/dev/null | od -An -tu1 | tr -s ' '); crc=$?
    po=$("$TMPDIR/p1-sr0i" "$f" < "$inp" 2>/dev/null | od -An -tu1 | tr -s ' '); prc=$?
    if [ "$co" = "$po" ] && [ "$crc" = "$prc" ]; then
        :
    else
        echo "  MISMATCH on $bn: $(head -c 60 "$f")"
        echo "    C sr0i  (rc=$crc): $co"
        echo "    P1 sr0i (rc=$prc): $po"
        fail=1
    fi
done
# THE PRIZE: the two-level stack seed -> P1pp-sr0i -> sr1i -> program, no C.
# sr1i.rs is the meta-circular SR-seed interpreter; feed it a subrust-emitted
# image and require the P1pp sr0i to match the C sr0i running the same.
SUB="$PWD/../target/release/subrust"
if [ -x "$SUB" ] && [ -f sr1i/sr1i.rs ]; then
    "$SUB" emit ../tests/seed/recurse.rs boot > "$TMPDIR/pz.img" 2>/dev/null || true
    if [ -s "$TMPDIR/pz.img" ]; then
        "$CSR0I"            sr1i/sr1i.rs < "$TMPDIR/pz.img" > "$TMPDIR/pz.c"  2>/dev/null; cz=$?
        "$TMPDIR/p1-sr0i"   sr1i/sr1i.rs < "$TMPDIR/pz.img" > "$TMPDIR/pz.p1" 2>/dev/null; pz=$?
        if cmp -s "$TMPDIR/pz.c" "$TMPDIR/pz.p1" && [ "$cz" = "$pz" ]; then
            echo "  PRIZE: seed->P1pp-sr0i->sr1i == seed->C-sr0i->sr1i (byte-identical, no C)"
        else
            echo "  PRIZE FAIL: sr1i chain differs (C rc=$cz P1 rc=$pz)"; fail=1
        fi
    fi
fi
if [ "$fail" = 0 ]; then
    echo "P1pp sr0i == frozen C sr0i: byte-identical over $(ls "$CORPUS" | wc -l) programs + the sr1i image chain (C-free sr0i DONE)"
else
    echo "sr0i eval: FAIL"
fi
exit "$fail"
