#!/bin/sh
# SR-seed corpus gate: the SR-seed corpus runs byte-identically across
# three independent implementations —
#   1. rustc     (the reference: corpus + BOOT_SHIMS, debug-profile flags)
#   2. sr0i-cc   (sr0i built with host cc, for fast iteration)
#   3. sr0i-m2   (sr0i built from the 256-byte hex0 seed via M2-Planet)
# subrust itself is verified == rustc by `cargo test --test boot_tests`; this
# script adds sr0i as the third leg. floats.rs is skipped: its f_* IEEE-f64
# intrinsics are per-arch assembly backends, not in the C prototype.
#
# Usage: sh verify.sh    (needs: rustc, cc, and the seed tools built)
set -e
here=$(dirname "$0")
cd "$here"
: "${TMPDIR:=/tmp}"
SEED=../tests/seed
SHIMS_SRC=../tests/common/mod.rs
M2BIN="$PWD/vendor/stage0-posix/AMD64/bin"

# extract the BOOT_SHIMS block (kept in ONE place, the Rust harness)
sed -n '/BOOT_SHIMS: &str = r#"/,/"#;/p' "$SHIMS_SRC" | sed '1d;$d' > "$TMPDIR/shims.rs"

# build both sr0i flavors
cc -w -DHOSTCC -o "$TMPDIR/sr0i_cc" sr0i/sr0i.c
sh kaem/build-sr0i.sh >/dev/null

pass=0; fail=0
for rs in "$SEED"/*.rs; do
    p=$(basename "$rs" .rs)
    [ "$p" = "floats" ] && continue   # f_* needs the assembly backend
    inp=/dev/null; [ -f "$SEED/$p.in" ] && inp="$SEED/$p.in"

    cat "$rs" "$TMPDIR/shims.rs" > "$TMPDIR/ref.rs"
    rustc --edition 2021 -O -C overflow-checks=on -A warnings \
        -A arithmetic_overflow -A unconditional_panic \
        "$TMPDIR/ref.rs" -o "$TMPDIR/ref" 2>/dev/null

    # capture the program's real exit status (not the pipe's), then hash
    # bytes; compare trap-NESS (nonzero) since rustc panics 101, sr0i exits 1
    # `&& rr=0 || rr=$?` keeps `set -e` from firing on a legit trap exit
    "$TMPDIR/ref"       < "$inp" > "$TMPDIR/o_r" 2>/dev/null && rr=0 || rr=$?
    "$TMPDIR/sr0i_cc" "$rs" < "$inp" > "$TMPDIR/o_c" 2>/dev/null && cc=0 || cc=$?
    sr0i/sr0i        "$rs" < "$inp" > "$TMPDIR/o_m" 2>/dev/null && mm=0 || mm=$?
    r=$(od -An -tx1 < "$TMPDIR/o_r" | tr -d ' \n')
    c=$(od -An -tx1 < "$TMPDIR/o_c" | tr -d ' \n')
    m=$(od -An -tx1 < "$TMPDIR/o_m" | tr -d ' \n')
    tr=0; [ "$rr" != 0 ] && tr=1
    tc=0; [ "$cc" != 0 ] && tc=1
    tm=0; [ "$mm" != 0 ] && tm=1

    if [ "$r" = "$c" ] && [ "$c" = "$m" ] && [ "$tr" = "$tc" ] && [ "$tc" = "$tm" ]; then
        echo "  ok   $p (trap=$tr)"
        pass=$((pass + 1))
    else
        echo "  FAIL $p : rustc(ec$rr)=$r  cc(ec$cc)=$c  m2(ec$mm)=$m"
        fail=$((fail + 1))
    fi
done
echo "sr0i three-way corpus: $pass ok, $fail fail"
[ "$fail" = 0 ]
