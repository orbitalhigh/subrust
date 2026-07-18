#!/bin/sh
# Dialect-checker gate: subrust's OWN checker, running from the seed.
#
# The self-hosted checker (subrust/self/check.rs — the real src/check.rs adapted
# by generate_check.py, which already CHECKS+RUNS under subrust in the selfhost
# tests) is given a putb-reporting main (check_b4.rs), serialized by `subrust
# emit ... boot` into an image, and evaluated by:
#   seed -> sr0i(seed-built) -> sr1i -> check_b4 image
# It lexes, parses, and type-checks the sample `fn f(x: i64) -> i64 { x }` and
# reports lexed/parsed/ok as T/F bytes plus diag_n/root_n as digits: "TTT01".
# We require the seed chain to match the rustc-sr1i reference AND the known-good
# answer. (A rustc-native reference of check.rs is not available — check.rs uses
# subrust-dialect idioms rustc rejects; it is meant to be checked by subrust.)
#
# f64/128-bit paths in the checker are dead for this integer sample (KTRAP stubs
# never fire). Needs: rustc, cc, the seed tools, a subrust release build.
set -eu
here=$(cd "$(dirname "$0")/.." && pwd)
cd "$here"
: "${TMPDIR:=/tmp}"
SUB="$here/../target/release/subrust"
SHIMS="$here/../tests/common/mod.rs"
SELF="$here/../self"
EXPECT="TTT01"

[ -x "$SUB" ] || (cd "$here/.." && cargo build --release -q)
[ -x sr0i/sr0i ] || sh kaem/build-sr0i.sh >/dev/null

emit_shims() { sed -n '/BOOT_SHIMS: &str = r#"/,/"#;/p' "$SHIMS" | sed '1d;$d'; }

# Derive check_b4.rs from the current self/check.rs (swap the print_* main for a
# putb-reporting one) so the gate always tracks the live self-hosted checker.
python3 - "$SELF/check.rs" "$SELF/check_b4.rs" <<'PY'
import sys
src = open(sys.argv[1]).read()
i = src.rindex('\nfn main() {')
main = '''
fn main() {
    let src = "fn f(x: i64) -> i64 { x }";
    let mut m = Mem {
        toks: [TOK_NONE; CAP_TOKS], tok_n: 0,
        nodes: [NODE_NONE; CAP_NODES], node_n: 0,
        diags: [Diag { code: 0, lo: 0, hi: 0, a: 0, b: 0 }; CAP_DIAGS], diag_n: 0,
        diag_lost: 0, overflow: false, root_first: NODE_NIL, root_n: 0,
    };
    let lexed = lex(src, &mut m);
    let parsed = parse(src, &mut m);
    let mut chk = CHK_INIT;
    let host = EMPTY_HOST;
    let ok = check(src, &mut m, &mut chk, &host);
    if lexed { putb(84); } else { putb(70); }
    if parsed { putb(84); } else { putb(70); }
    if ok { putb(84); } else { putb(70); }
    putb(48 + m.diag_n as u64);
    putb(48 + m.root_n as u64);
}
'''
open(sys.argv[2], 'w').write(src[:i] + main)
PY

# sr1i under rustc (valid SR-seed => valid Rust)
cat sr1i/sr1i.rs > "$TMPDIR/sr1i_ref.rs"
emit_shims >> "$TMPDIR/sr1i_ref.rs"
rustc --edition 2021 -O -C overflow-checks=on -A warnings -A arithmetic_overflow \
    -A unconditional_panic "$TMPDIR/sr1i_ref.rs" -o "$TMPDIR/sr1i_ref" 2>/dev/null

# serialize the self-hosted checker to a seed-chain image
"$SUB" emit "$SELF/check_b4.rs" boot > "$TMPDIR/check_b4.img"

r1=$("$TMPDIR/sr1i_ref" < "$TMPDIR/check_b4.img" 2>/dev/null || true)          # rustc-sr1i
s1=$(sr0i/sr0i sr1i/sr1i.rs < "$TMPDIR/check_b4.img" 2>/dev/null || true)      # seed chain

echo "  self-hosted checker on 'fn f(x: i64) -> i64 { x }':"
echo "    expected            : $EXPECT  (lexed=T parsed=T ok=T  diag_n=0 root_n=1)"
echo "    rustc-sr1i(image)   : $r1"
echo "    seed-sr0i -> sr1i   : $s1"
if [ "$s1" = "$EXPECT" ] && [ "$r1" = "$EXPECT" ]; then
    echo "dialect checker: subrust's own checker runs from the hex0 seed — OK"
    exit 0
fi
echo "dialect checker: FAIL"
exit 1
