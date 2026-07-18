#!/bin/sh
# emit-on-chain gate: the DIALECT emit (self/emit_self.rs =
# the self-hosted checker + a port of emit.rs) must serialize a checked program
# to the SAME image bytes as the rustc oracle `subrust emit x.rs boot`.
#
# emit_self.rs is DERIVED here from the live self/check.rs (exactly as verify-io.sh
# derives check_io.rs): thread the source as &[u8], then append self/emit_delta.rs
# (the emit serializer + a getb-source / putb-image main). For the rustc DEV diff
# we also (a) give host_name a 'static return (rustc wants it; subrust accepts
# either) and (b) bump the seed pools so the corpus can exercise real constructs —
# neither affects the emitted bytes. The on-chain build (a later step) keeps the
# tiny seed pools (memory-bound, documented). Needs rustc + the subrust release
# build.
set -eu
here=$(cd "$(dirname "$0")/../.." && pwd)   # repo root (…/subrust)
cd "$here"
: "${TMPDIR:=/tmp}"
export TMPDIR
SUB="$here/target/release/subrust"
SELF="$here/self"
SHIMS="$here/tests/common/mod.rs"

[ -x "$SUB" ] || (cargo build --release -q)

# Derive emit_self.rs from the live check.rs + emit_delta.rs.
python3 - "$SELF/check.rs" "$SELF/emit_delta.rs" "$SELF/emit_self.rs" <<'PY'
import sys
chk = open(sys.argv[1]).read()
delta = open(sys.argv[2]).read()
# thread the source as bytes (same transform as verify-io.sh's check_io.rs)
chk = chk.replace('src: &str', 'src: &[u8]')
chk = chk.replace('src.as_bytes()', 'src')
# host support for the emit main's putb: room for [ld,st,getb,putb] + real names
# (host_name is stubbed to b"" in the seed checker). Part of emit_self.rs proper —
# the on-chain checker resolves putb the same way.
chk = chk.replace('const HCAP_HFNS: usize = 1;', 'const HCAP_HFNS: usize = 4;')
chk = chk.replace(
    'fn host_name(id: u32) -> &[u8] { let _ = id; b"" }',
    'fn host_name(id: u32) -> &[u8] {\n'
    '    if id == 0 { return b"ld"; }\n'
    '    if id == 1 { return b"st"; }\n'
    '    if id == 2 { return b"getb"; }\n'
    '    if id == 3 { return b"putb"; }\n'
    '    b""\n'
    '}')
# (The array-`.len()`-through-ref that emit_self's host arm uses — hf.params.len()
# — was mis-lowered by subrust-cli/src/emit.rs, which had no RES_ARRAY_LEN case.
# That is now fixed at the source, so no workaround is needed here.)
# strip check.rs's own reporting main; emit_delta.rs supplies the getb/putb main
i = chk.rindex('\nfn main() {')
open(sys.argv[3], 'w').write(chk[:i] + '\n' + delta)
PY

# rustc dev build: 'static on the no-input-ref borrow return + generous pools
# (capacity only — the image bytes are pool-size-independent).
sed \
  -e "s/fn host_name(id: u32) -> &\[u8\]/fn host_name(id: u32) -> \&'static [u8]/" \
  -e 's/^const CAP_TOKS: usize = 32;/const CAP_TOKS: usize = 4096;/' \
  -e 's/^const CAP_NODES: usize = 28;/const CAP_NODES: usize = 4096;/' \
  -e 's/^const CAP_DIAGS: usize = 6;/const CAP_DIAGS: usize = 64;/' \
  -e 's/^const CAP_CONSTS: usize = 2;/const CAP_CONSTS: usize = 64;/' \
  -e 's/^const CAP_FNS: usize = 4;/const CAP_FNS: usize = 64;/' \
  -e 's/^const CAP_LOCALS: usize = 8;/const CAP_LOCALS: usize = 128;/' \
  -e 's/^const CAP_VALS: usize = 16;/const CAP_VALS: usize = 1024;/' \
  -e 's/^const ECAP_NODES: usize = 96;/const ECAP_NODES: usize = 4096;/' \
  -e 's/^const ECAP_VALS: usize = 48;/const ECAP_VALS: usize = 1024;/' \
  "$SELF/emit_self.rs" > "$TMPDIR/emit_dev_base.rs"

sed -n '/BOOT_SHIMS: &str = r#"/,/"#;/p' "$SHIMS" | sed '1d;$d' > "$TMPDIR/emit_shims.rs"
cat "$TMPDIR/emit_dev_base.rs" "$TMPDIR/emit_shims.rs" > "$TMPDIR/emit_dev.rs"
rustc --edition 2021 -O -A warnings "$TMPDIR/emit_dev.rs" -o "$TMPDIR/emit_dev" 2>/dev/null

# compute-core + putb corpus: literals, names/locals, binary/unary, if/while/
# loop/break/continue, let/assign, return, user calls, and putb host calls. Each
# must emit byte-for-byte what `subrust emit` produces.
corpus='
fn main() {}
:::
fn main() -> u64 { 1 + 2 }
:::
fn main() -> u64 { let x = 5; x }
:::
fn main() -> u64 { if 1 < 2 { 3 } else { 4 } }
:::
fn main() -> bool { !false }
:::
fn main() -> u64 { let mut x: u64 = 0; while x < 3 { x = x + 1; } x }
:::
fn main() -> u64 { let mut s: u64 = 0; let mut i: u64 = 0; loop { if i >= 4 { break; } s = s + i; i = i + 1; } s }
:::
fn f(a: u64, b: u64) -> u64 { a * b + 1 } fn main() -> u64 { f(6, 7) }
:::
fn main() -> u64 { let a = 10u64; let b = 3u64; (a - b) * 2 }
:::
fn main() { putb(72); }
:::
fn main() { putb(72); putb(73); }
:::
fn main() { let mut i: u64 = 0; while i < 5 { putb(65 + i); i = i + 1; } }
:::
fn f(n: u64) { putb(48 + n); } fn main() { f(7); }
'

# split on the ':::' delimiter without mangling program text
python3 - "$corpus" > "$TMPDIR/emit_progs.txt" <<'PY'
import sys
progs = [p.strip() for p in sys.argv[1].split(':::')]
progs = [p for p in progs if p]
print(len(progs))
for i, p in enumerate(progs):
    open(f"{__import__('os').environ['TMPDIR']}/emit_p{i}.rs", 'w').write(p + '\n')
    print(p)
PY

total=$(sed -n '1p' "$TMPDIR/emit_progs.txt")
fail=0
i=0
while [ "$i" -lt "$total" ]; do
    prog="$TMPDIR/emit_p$i.rs"
    "$TMPDIR/emit_dev" < "$prog" > "$TMPDIR/emit_self.img" 2>/dev/null || true
    "$SUB" emit "$prog" boot > "$TMPDIR/emit_ref.img" 2>/dev/null || true
    # the corpus must be valid, emittable programs — an empty reference means the
    # oracle rejected it (0-byte == 0-byte would otherwise be a false pass)
    if [ ! -s "$TMPDIR/emit_ref.img" ]; then
        printf '  BAD CORPUS (oracle emitted nothing): %s\n' "$(tr -d '\n' < "$prog")"
        "$SUB" emit "$prog" boot 2>&1 1>/dev/null | head -1 | sed 's/^/    /'
        fail=1
        i=$((i + 1))
        continue
    fi
    if cmp -s "$TMPDIR/emit_self.img" "$TMPDIR/emit_ref.img"; then
        sz=$(wc -c < "$TMPDIR/emit_self.img")
        printf '  ok  [%2d B] %s\n' "$sz" "$(tr -d '\n' < "$prog")"
    else
        printf '  MISMATCH: %s\n' "$(tr -d '\n' < "$prog")"
        cmp "$TMPDIR/emit_self.img" "$TMPDIR/emit_ref.img" | head -1 || true
        printf '    self=%s ref=%s bytes\n' "$(wc -c < "$TMPDIR/emit_self.img")" "$(wc -c < "$TMPDIR/emit_ref.img")"
        fail=1
    fi
    i=$((i + 1))
done

if [ "$fail" -eq 0 ]; then
    echo "emit_self (dialect) == subrust emit (rustc): byte-identical across $total programs (compute core + putb)"
else
    echo "verify-emit: rustc-diff FAIL"
    exit 1
fi

# ── CHAIN VALIDATION ─────────────────────────────────────────────────────────
# The real rung closure: run emit_self ITSELF on the seed chain (sr0i ⟨sr1i
# ⟨emit_self ⟨program⟩⟩⟩) and require its emitted image to byte-match `subrust
# emit`. No rustc in this path — `subrust emit emit_self.rs boot` compiles the
# emitter to an sr1i image (it fits sr0i's node region: ~24.1k of ~28.5k), which
# the seed binary interprets. The committed (seed-pool) emit_self.rs is used, so
# programs must fit its tiny caps (CAP_NODES=28). Covers the compute core AND putb
# host calls — the host arm's `hf.params.len()` exercised array `.len()`, which
# exposed a real emit bug (emit.rs had no RES_ARRAY_LEN case) now fixed at the
# source. Needs the seed tools to build sr0i.
BOOT="$here/subrust-boot"
SR0I="$BOOT/sr0i/sr0i"
SR1I="$BOOT/sr1i/sr1i.rs"
[ -x "$SR0I" ] || (cd "$BOOT" && sh kaem/build-sr0i.sh >/dev/null 2>&1) || true
if [ ! -x "$SR0I" ]; then
    echo "verify-emit: chain stage SKIPPED (sr0i unavailable — needs the seed toolchain)"
    exit 0
fi

"$SUB" emit "$SELF/emit_self.rs" boot > "$TMPDIR/emitself.img" 2>/dev/null || true
if [ ! -s "$TMPDIR/emitself.img" ]; then
    echo "verify-emit: chain stage FAIL (subrust could not emit emit_self.rs)"
    exit 1
fi

# host-free programs that fit emit_self's seed caps (CAP_NODES=28). Cover
# KL/KB/KT/KN/KI/KF — the compute core that runs end-to-end with no rustc.
chain_corpus='fn main() -> u64 { 1 + 2 }
:::
fn main() -> u64 { let x: u64 = 5; x + x }
:::
fn main() -> u64 { if 1 < 2 { 3 } else { 4 } }
:::
fn f(a: u64) -> u64 { a * 3 } fn main() -> u64 { f(4) }
:::
fn main() { putb(72); }
:::
fn main() { putb(72); putb(73); }
:::
fn f(n: u64) { putb(n); } fn main() { f(72); }'

python3 - "$chain_corpus" > "$TMPDIR/chain_progs.txt" <<'PY'
import sys, os
progs = [p.strip() for p in sys.argv[1].split(':::') if p.strip()]
print(len(progs))
for i, p in enumerate(progs):
    open(f"{os.environ['TMPDIR']}/chain_p{i}.rs", 'w').write(p)
    print(p)
PY

ctotal=$(sed -n '1p' "$TMPDIR/chain_progs.txt")
cfail=0
i=0
while [ "$i" -lt "$ctotal" ]; do
    prog="$TMPDIR/chain_p$i.rs"
    "$SUB" emit "$prog" boot > "$TMPDIR/chain_ref.img" 2>/dev/null || true
    { cat "$TMPDIR/emitself.img"; cat "$prog"; } | "$SR0I" "$SR1I" > "$TMPDIR/chain_seed.img" 2>/dev/null || true
    if [ -s "$TMPDIR/chain_ref.img" ] && cmp -s "$TMPDIR/chain_seed.img" "$TMPDIR/chain_ref.img"; then
        printf '  ok  [%4d B] %s\n' "$(wc -c < "$TMPDIR/chain_seed.img")" "$(cat "$prog")"
    else
        printf '  CHAIN MISMATCH: %s  (seed=%s ref=%s)\n' "$(cat "$prog")" "$(wc -c < "$TMPDIR/chain_seed.img")" "$(wc -c < "$TMPDIR/chain_ref.img")"
        cfail=1
    fi
    i=$((i + 1))
done

if [ "$cfail" -eq 0 ]; then
    echo "CHAIN-VALIDATED (sr0i, NO RUSTC): emit_self on the seed chain == subrust emit across $ctotal programs"
else
    echo "verify-emit: chain FAIL"
    exit 1
fi
