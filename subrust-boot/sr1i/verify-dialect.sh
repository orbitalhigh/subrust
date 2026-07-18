#!/bin/sh
# Dialect gate: dialect programs — structs, arrays,
# slices, match, ... beyond SR-seed — run through the two-level stack
#   seed -> sr0i(seed-built) -> sr1i -> program-image
# and match a NATIVE reference. sr0i cannot run dialect SOURCE (it interprets
# SR-seed source only), so the reference is the dialect program compiled + run by
# rustc with the BOOT shims — not sr0i-direct as in verify.sh. Each program in
# sr1i/dialect/*.rs is emitted (`subrust emit`), then compared across:
#   rustc-native-reference == rustc-sr1i(image) == seed-sr0i -> sr1i(image)
# requiring identical output and trap outcome.
set -eu
here=$(cd "$(dirname "$0")/.." && pwd)
cd "$here"
: "${TMPDIR:=/tmp}"
SUB="$here/../target/release/subrust"
SHIMS="$here/../tests/common/mod.rs"
DIA="$here/sr1i/dialect"

[ -x "$SUB" ] || (cd "$here/.." && cargo build --release -q)

cc -w -DHOSTCC -o "$TMPDIR/sr0i_cc" sr0i/sr0i.c
[ -x sr0i/sr0i ] || sh kaem/build-sr0i.sh >/dev/null

emit_shims() { sed -n '/BOOT_SHIMS: &str = r#"/,/"#;/p' "$SHIMS" | sed '1d;$d'; }

# sr1i under rustc (it is valid SR-seed, hence valid Rust)
cat sr1i/sr1i.rs > "$TMPDIR/sr1i_ref.rs"
emit_shims >> "$TMPDIR/sr1i_ref.rs"
rustc --edition 2021 -O -C overflow-checks=on -A warnings -A arithmetic_overflow \
    -A unconditional_panic "$TMPDIR/sr1i_ref.rs" -o "$TMPDIR/sr1i_ref" 2>/dev/null

pass=0
fail=0
for f in "$DIA"/*.rs; do
    p=$(basename "$f" .rs)
    "$SUB" emit "$f" boot > "$TMPDIR/$p.img"

    # reference: the dialect program itself, compiled + run by rustc.
    # (underscore, not dot, in the stem — rustc derives the crate name from it)
    cat "$f" > "$TMPDIR/${p}_ref.rs"
    emit_shims >> "$TMPDIR/${p}_ref.rs"
    rustc --edition 2021 -O -C overflow-checks=on -A warnings -A arithmetic_overflow \
        -A unconditional_panic "$TMPDIR/${p}_ref.rs" -o "$TMPDIR/${p}_refbin" 2>/dev/null
    "$TMPDIR/${p}_refbin" < /dev/null > "$TMPDIR/ref" 2>/dev/null && rec=0 || rec=$?

    # image through rustc-sr1i, and through seed-built sr0i -> sr1i
    "$TMPDIR/sr1i_ref" < "$TMPDIR/$p.img" > "$TMPDIR/r1" 2>/dev/null && r1c=0 || r1c=$?
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
echo "sr1i dialect: $pass ok, $fail fail"
[ "$fail" = 0 ]
