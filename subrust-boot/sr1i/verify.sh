#!/bin/sh
# Two-level interpreter-stack gate: the TWO-LEVEL interpreter stack.
# sr1i is a meta-circular SR-seed interpreter WRITTEN IN SR-seed; it reads a
# `subrust emit`-serialized checked program image and evaluates it. This gate
# runs the compute/control/recursion corpus through:
#   seed -> sr0i(seed-built) -> sr1i -> program
# and diffs against the program run directly on sr0i, requiring identical
# output and trap outcome. It also cross-checks sr1i under rustc (sr1i is
# valid SR-seed, hence valid Rust).
#
# Scope: the compute/control/recursion core + putb (getb/ld/st/f_* deferred);
# structs/arrays/f64 are dialect, beyond SR-seed. See REPLAY.md.
#
# Needs: rustc, cc, the seed tools, and a subrust release build.
set -eu
here=$(cd "$(dirname "$0")/.." && pwd)
cd "$here"
: "${TMPDIR:=/tmp}"
SUB="$here/../target/release/subrust"
SEED="$here/../tests/seed"
SHIMS="$here/../tests/common/mod.rs"

[ -x "$SUB" ] || (cd "$here/.." && cargo build --release -q)

cc -w -DHOSTCC -o "$TMPDIR/sr0i_cc" sr0i/sr0i.c
[ -x sr0i/sr0i ] || sh kaem/build-sr0i.sh >/dev/null

# sr1i under rustc (valid SR-seed => valid Rust)
cat sr1i/sr1i.rs > "$TMPDIR/sr1i_ref.rs"
sed -n '/BOOT_SHIMS: &str = r#"/,/"#;/p' "$SHIMS" | sed '1d;$d' >> "$TMPDIR/sr1i_ref.rs"
rustc --edition 2021 -O -C overflow-checks=on -A warnings -A arithmetic_overflow \
    -A unconditional_panic "$TMPDIR/sr1i_ref.rs" -o "$TMPDIR/sr1i_ref" 2>/dev/null

# programs sr1i handles at this rung (no guest getb/ld/st/f_*)
PROGS="hello arith loops bools recurse trap_overflow trap_div0 trap_shift"
pass=0; fail=0
for p in $PROGS; do
    "$SUB" emit "$SEED/$p.rs" boot > "$TMPDIR/$p.img"

    # reference: program run directly on sr0i
    sr0i/sr0i "$SEED/$p.rs" < /dev/null > "$TMPDIR/ref" 2>/dev/null && rec=0 || rec=$?
    # rustc-sr1i on the image
    "$TMPDIR/sr1i_ref" < "$TMPDIR/$p.img" > "$TMPDIR/r1" 2>/dev/null && r1c=0 || r1c=$?
    # seed-sr0i -> sr1i on the image
    sr0i/sr0i sr1i/sr1i.rs < "$TMPDIR/$p.img" > "$TMPDIR/s1" 2>/dev/null && s1c=0 || s1c=$?

    rt=0; [ "$rec" != 0 ] && rt=1
    a1=0; [ "$r1c" != 0 ] && a1=1
    b1=0; [ "$s1c" != 0 ] && b1=1
    if cmp -s "$TMPDIR/ref" "$TMPDIR/r1" && cmp -s "$TMPDIR/r1" "$TMPDIR/s1" \
        && [ "$rt" = "$a1" ] && [ "$a1" = "$b1" ]; then
        echo "  ok   $p (trap=$rt)"
        pass=$((pass + 1))
    else
        echo "  FAIL $p : ref(ec$rec) rustc-sr1i(ec$r1c) seed-sr0i->sr1i(ec$s1c)"
        fail=$((fail + 1))
    fi
done
echo "sr1i two-level stack: $pass ok, $fail fail"
[ "$fail" = 0 ]
