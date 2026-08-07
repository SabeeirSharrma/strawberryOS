#!/usr/bin/env bash
# build-iso.sh — Build the Strawberry OS headless ISO using archiso.
# Run on an Arch Linux system with archiso installed.
#
# Usage: ./scripts/build-iso.sh [--gui]
#
# Prerequisites:
#   sudo pacman -S archiso

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
PROFILE_DIR="$PROJECT_DIR/archiso"
BUILD_DIR="/tmp/strawberry-iso-build"
OUTPUT_DIR="$PROJECT_DIR/archiso/out"

echo "=== Strawberry OS ISO Builder ==="
echo ""

# Check for archiso
if ! command -v mkarchiso &>/dev/null; then
    echo "ERROR: mkarchiso not found."
    echo "Install archiso: sudo pacman -S archiso"
    exit 1
fi

# Check for root
if [ "$EUID" -ne 0 ]; then
    echo "ERROR: This script must be run as root (archiso requires it)."
    echo "Usage: sudo ./scripts/build-iso.sh"
    exit 1
fi

# Clean previous build
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR" "$OUTPUT_DIR"

echo "Profile:     $PROFILE_DIR"
echo "Build dir:   $BUILD_DIR"
echo "Output dir:  $OUTPUT_DIR"
echo ""

# Build the ISO
mkarchiso \
    -w "$BUILD_DIR" \
    -o "$OUTPUT_DIR" \
    "$PROFILE_DIR"

echo ""
echo "=== Build complete ==="
echo "ISO written to: $OUTPUT_DIR/"
ls -lh "$OUTPUT_DIR"/*.iso 2>/dev/null || echo "(no ISO found — check build logs)"
