#!/bin/sh
# build-fasm.sh — assemble the amd64 IEEE-f64 BOOT_API backend using ONLY the
# seed-built hex2 (seed-built tools) — no host assembler in the build
# path. f_amd64.hex2 holds the audited machine-code bytes; it is concatenated
# after AMD64/ELF-amd64.hex2 (the standard stage0 64-bit ELF header, whose
# e_entry is &_start) and linked by the seed hex2 into a static ELF.
#
# Output: fasm/f_amd64, an amd64 ELF whose whole tool provenance is the seed.
set -eu
here=$(cd "$(dirname "$0")/.." && pwd)   # subrust-boot/
cd "$here"
ST=vendor/stage0-posix
HEX2="$PWD/$ST/AMD64/bin/hex2"
HDR="$PWD/$ST/AMD64/ELF-amd64.hex2"

if [ ! -x "$HEX2" ]; then
    echo "error: seed hex2 missing; run the seed replay first (see REPLAY.md)" >&2
    exit 1
fi

"$HEX2" --little-endian --architecture amd64 --base-address 0x00600000 \
    -f "$HDR" -f fasm/f_amd64.hex2 -o fasm/f_amd64
chmod +x fasm/f_amd64

echo "built fasm/f_amd64 from the seed hex2:"
"$ST/AMD64/bin/sha256sum" fasm/f_amd64 2>/dev/null || sha256sum fasm/f_amd64
