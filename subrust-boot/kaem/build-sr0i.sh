#!/bin/sh
# Build sr0i (the SR-seed interpreter) using ONLY tools
# bootstrapped from the 256-byte hex0 seed — no host compiler.
#
# Prerequisite: the stage0-posix replay (REPLAY.md) has produced
# vendor/stage0-posix/AMD64/bin/ (M2-Mesoplanet, M2-Planet, blood-elf, M1,
# hex2). This script drives the seed-built M2-Mesoplanet, which orchestrates
# M2-Planet -> blood-elf -> M1 -> hex2 internally (all seed-built).
#
# Output: sr0i/sr0i, an amd64 ELF whose entire tool provenance is the seed.
set -e
here=$(dirname "$0")/..
cd "$here"
ST=vendor/stage0-posix
BIN="$PWD/$ST/AMD64/bin"

if [ ! -x "$BIN/M2-Mesoplanet" ]; then
    echo "error: seed tools missing; run the seed replay first (see REPLAY.md)" >&2
    echo "  cd $ST && ./bootstrap-seeds/POSIX/AMD64/kaem-optional-seed kaem.amd64" >&2
    exit 1
fi

: "${TMPDIR:=/tmp}"
export TMPDIR
PATH="$BIN:$PATH" M2LIBC_PATH="$PWD/$ST/M2libc" \
    "$BIN/M2-Mesoplanet" --architecture amd64 -f sr0i/sr0i.c -o sr0i/sr0i

echo "built sr0i/sr0i from the seed toolchain:"
"$BIN/sha256sum" sr0i/sr0i 2>/dev/null || sha256sum sr0i/sr0i
