#!/bin/sh
# bootstrap-all.sh — build everything from the 256-byte hex0 seed and time it.
#
# Honest accounting: what genuinely runs FROM THE SEED today
# is the chain up to `sr0i`, the SR-seed interpreter. "Full subrust"
# — the real checker+machine running hvac.rs — is still rustc-built: reaching
# it from the seed needs the two-level interpreter stack (sr1i) and subrust
# source on the chain, which are not built yet. This script times every phase
# that exists and marks
# each one's provenance:
#   [SEED]  artifact's whole tool provenance is the 256-byte seed
#   [boot2] boot2's portable chain; toolchain built with host cc (the
#           self-hosted-from-seed m1pp/hex2pp are a later purity upgrade)
#   [rustc] depends on rustc — the reference oracle, and today's real subrust
#
# Usage: sh bootstrap-all.sh [--clean] [--no-rustc]
#   --clean     reset stage0-posix to pristine first (true cold from-seed)
#   --no-rustc  skip the rustc-dependent phases (verification refs + subrust)
set -eu

CLEAN=0
WITH_RUSTC=1
for a in "$@"; do
    case "$a" in
        --clean) CLEAN=1 ;;
        --no-rustc) WITH_RUSTC=0 ;;
        *) echo "usage: $0 [--clean] [--no-rustc]" >&2; exit 2 ;;
    esac
done

here=$(cd "$(dirname "$0")" && pwd)
cd "$here"
: "${TMPDIR:=/tmp}"
ST=vendor/stage0-posix
STBIN="$here/$ST/AMD64/bin"
LOG="$TMPDIR/bootstrap-all.log"
: > "$LOG"

# ---- timing table ----------------------------------------------------------
ROWS="$TMPDIR/boot_rows.txt"; : > "$ROWS"
now() { date +%s.%N; }
elapsed() { echo "$2 - $1" | bc; }
seed_total=0; rustc_total=0

phase() { # phase "TAG" "label" cmd...
    tag=$1; label=$2; shift 2
    printf '  >> [%s] %s ... ' "$tag" "$label"
    s=$(now)
    if "$@" >>"$LOG" 2>&1; then ok=OK; else ok=FAIL; fi
    e=$(now); d=$(elapsed "$s" "$e")
    printf '%s (%.2fs)\n' "$ok" "$d"
    printf '%s\t%s\t%s\t%.2f\n' "$tag" "$label" "$ok" "$d" >> "$ROWS"
    case "$tag" in
        SEED) seed_total=$(echo "$seed_total + $d" | bc) ;;
        rustc) rustc_total=$(echo "$rustc_total + $d" | bc) ;;
    esac
    [ "$ok" = OK ] || { echo "FAILED at: $label (see $LOG)"; exit 1; }
}

# ---- phase implementations -------------------------------------------------
p_clean() { cd "$here/$ST"; git clean -xdfq . >/dev/null 2>&1 || true; cd "$here"; }

p_stage0() { # 256-byte seed -> hex0/1/2 -> M0 -> M2-Planet + mescc-tools
    cd "$here/$ST"
    ./bootstrap-seeds/POSIX/AMD64/kaem-optional-seed kaem.amd64
    cd "$here"
}

p_sr0i() { sh kaem/build-sr0i.sh; }              # M2-Planet (seed-built) -> sr0i

p_sr0i_verify() { sh verify.sh; }                # corpus: rustc == cc == seed-sr0i

p_p1_arith() { sh p1/verify.sh; }                # boot2 P1pp arith core == rustc

p_p1_arith128() { sh p1/verify-128.sh; }         # boot2 P1pp 128-bit core == rustc

p_sr0ip1() { sh sr0i-p1/verify-sr0i.sh; }        # C-FREE P1pp sr0i == frozen C sr0i

p_subrust_build() { cd "$here/.."; cargo build --release -q; cd "$here"; }

p_example() {   # full subrust (rustc-built): check + run the hvac script
    "$here/../target/release/subrust" check "$here/../tests/data/hvac.rs" hvac
}

p_sr1i() {   # two-level stack: seed-sr0i -> sr1i -> recurse.rs, from the seed
    "$here/../target/release/subrust" emit "$here/../tests/seed/recurse.rs" boot > "$TMPDIR/recurse.img"
    out=$(sr0i/sr0i sr1i/sr1i.rs < "$TMPDIR/recurse.img")
    [ "$out" = "$(cat "$here/../tests/seed/recurse.out")" ]
}

p_fasm() { sh fasm/build-fasm.sh; }              # seed hex2 -> f_* IEEE-f64 backend

p_fasm_verify() { sh fasm/verify-fasm.sh; }      # 48k f64 ops == rustc BOOT_SHIMS

# ---- run -------------------------------------------------------------------
echo "=== subrust bootstrap: from the 256-byte hex0 seed ==="
seed_sha=$(sha256sum "$ST/bootstrap-seeds/POSIX/AMD64/hex0-seed" 2>/dev/null | cut -c1-16)
echo "    seed: hex0-seed sha256 ${seed_sha}...  ($(wc -c < "$ST/bootstrap-seeds/POSIX/AMD64/hex0-seed") bytes)"
echo

[ "$CLEAN" = 1 ] && phase SEED "reset stage0 to pristine" p_clean
phase SEED  "stage0-posix: seed -> hex0/1/2 -> M0 -> M2-Planet + tools" p_stage0
phase SEED  "sr0i: SR-seed interpreter built from the seed (M2-Planet)" p_sr0i
phase SEED  "f_* backend: real IEEE f64 from seed-assembled amd64 (hex2)" p_fasm

if [ "$WITH_RUSTC" = 1 ]; then
    phase rustc "sr0i corpus: 3-way byte-identical (rustc / cc / seed-sr0i)" p_sr0i_verify
    phase boot2 "P1pp arith-core == rustc over 10,400 vectors" p_p1_arith
    phase boot2 "P1pp 128-bit core (39 ops) == rustc over 26k vectors" p_p1_arith128
    phase boot2 "C-FREE P1pp sr0i == frozen C sr0i (seed corpus + sr1i chain)" p_sr0ip1
    phase rustc "subrust + subrust-cli (cargo release build)" p_subrust_build
    phase rustc "full subrust: check hvac.rs against HVAC_API" p_example
    phase SEED  "two-level stack: seed-sr0i -> sr1i -> recurse (fib/gcd/pow)" p_sr1i
    phase rustc "f_* backend: 48k IEEE f64 ops == rustc BOOT_SHIMS" p_fasm_verify
fi

# ---- summary ---------------------------------------------------------------
echo
echo "=== timing ==="
awk -F'\t' '{printf "  [%-5s] %-58s %6ss  %s\n", $1, $2, $4, $3}' "$ROWS"
echo "  ------------------------------------------------------------------------"
printf '  FROM THE SEED (hex0 -> sr0i, SR-seed running): %ss\n' "$seed_total"
if [ "$WITH_RUSTC" = 1 ]; then
    printf '  rustc-dependent phases (verify refs + full subrust): %ss\n' "$rustc_total"
fi
echo
echo "  Reached from the seed: sr0i (SR-seed) AND the two-level stack"
echo "  (seed -> sr0i -> sr1i -> program) — an interpreter written in SR-seed"
echo "  running recurse/fib on the seed-built interpreter (the two-level mechanism)."
echo "  Full subrust (checker+machine on hvac.rs) is still rustc-built above;"
echo "  the full dialect on the chain needs sr1i grown to the full dialect + subrust v0.2-0.4 language work."
