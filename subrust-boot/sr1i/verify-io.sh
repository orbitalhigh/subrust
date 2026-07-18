#!/bin/sh
# self-hosted-checker gate: the self-hosted checker reading ARBITRARY source from stdin, from the
# seed. Extends verify-b4.sh (which checks one hard-coded sample): here the checker
# reads the program to type-check via `getb`, so a different program on stdin gives
# a different verdict — the seed chain is now a real (tiny-program) type-checker.
#
# check_io.rs = self/check.rs with (a) the source threaded as &[u8] instead of &str
# (src is used only as bytes), and (b) a main that getb-reads the program into a
# byte buffer until EOF (u64::MAX) then reports lexed/parsed/ok + diag_n/root_n as
# "TTT01"-style bytes. It is emitted against BOOT_API and fed `image ++ <program>`;
# we require the seed chain to match the rustc-sr1i reference and the known verdict
# on both a valid program and one with a type error. Tiny pools (CAP_TOKS=32,
# CAP_NODES=28) → programs must be small. Needs rustc, cc, the seed tools, a
# subrust release build.
set -eu
here=$(cd "$(dirname "$0")/.." && pwd)
cd "$here"
: "${TMPDIR:=/tmp}"
SUB="$here/../target/release/subrust"
SHIMS="$here/../tests/common/mod.rs"
SELF="$here/../self"

[ -x "$SUB" ] || (cd "$here/.." && cargo build --release -q)
[ -x sr0i/sr0i ] || sh kaem/build-sr0i.sh >/dev/null

emit_shims() { sed -n '/BOOT_SHIMS: &str = r#"/,/"#;/p' "$SHIMS" | sed '1d;$d'; }

# Derive check_io.rs from the live self/check.rs: thread &[u8], getb-reading main.
python3 - "$SELF/check.rs" "$SELF/check_io.rs" <<'PY'
import sys
src = open(sys.argv[1]).read()
src = src.replace('src: &str', 'src: &[u8]')
src = src.replace('src.as_bytes()', 'src')
i = src.rindex('\nfn main() {')
main = '''
fn main() {
    let mut buf = [0u8; 128];
    let mut n: usize = 0;
    loop {
        let c = getb();
        if c == 18446744073709551615 {
            break;
        }
        if n < 128 {
            buf[n] = c as u8;
            n = n + 1;
        }
    }
    let src = &buf[0..n];
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

cat sr1i/sr1i.rs > "$TMPDIR/sr1i_ref.rs"
emit_shims >> "$TMPDIR/sr1i_ref.rs"
rustc --edition 2021 -O -C overflow-checks=on -A warnings -A arithmetic_overflow \
    -A unconditional_panic "$TMPDIR/sr1i_ref.rs" -o "$TMPDIR/sr1i_ref" 2>/dev/null

"$SUB" emit "$SELF/check_io.rs" boot > "$TMPDIR/check_io.img"

fail=0
check_one() { # $1 = program source, $2 = expected verdict
    printf '%s' "$1" > "$TMPDIR/prog.txt"
    r1=$( { cat "$TMPDIR/check_io.img"; cat "$TMPDIR/prog.txt"; } | "$TMPDIR/sr1i_ref" 2>/dev/null || true )
    s1=$( { cat "$TMPDIR/check_io.img"; cat "$TMPDIR/prog.txt"; } | sr0i/sr0i sr1i/sr1i.rs 2>/dev/null || true )
    if [ "$s1" = "$2" ] && [ "$r1" = "$2" ]; then
        echo "  ok    $1  ->  $s1"
    else
        echo "  FAIL  $1  ->  seed=$s1 rustc-sr1i=$r1 expected=$2"
        fail=1
    fi
}

echo "  self-hosted checker reading arbitrary source from stdin (seed chain):"
check_one 'fn f(x: i64) -> i64 { x }'        TTT01   # valid
check_one 'fn h(a: u64) -> u64 { a + 1 }'    TTT01   # valid
check_one 'fn g() -> bool { 5 }'             TTF11   # type error: 5 is not bool
check_one 'fn e() -> u64 { true }'           TTF11   # type error: true is not u64
[ "$fail" = 0 ] && echo "self-host io: the seed-built checker type-checks arbitrary programs — OK" || echo "self-host io: FAIL"
exit "$fail"
