#!/usr/bin/env bash
# build-iso.sh — Build Strawberry OS ISO(s) using archiso.
# Run on an Arch Linux system with archiso installed.
#
# Usage:
#   sudo ./scripts/build-iso.sh          # Build headless ISO
#   sudo ./scripts/build-iso.sh --gui    # Build GUI ISO
#   sudo ./scripts/build-iso.sh --all    # Build both

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
BUILD_DIR="/tmp/strawberry-build"
OUTPUT_DIR="$PROJECT_DIR/archiso/out"

build_iso() {
    local profile_dir="$1"
    local profile_name="$2"

    echo ""
    echo "=== Building: $profile_name ==="
    echo "Profile:     $profile_dir"
    echo "Build dir:   $BUILD_DIR"
    echo "Output dir:  $OUTPUT_DIR"
    echo ""

    rm -rf "$BUILD_DIR"
    mkdir -p "$OUTPUT_DIR"

    mkarchiso \
        -v \
        -w "$BUILD_DIR" \
        -o "$OUTPUT_DIR" \
        "$profile_dir"

    echo ""
    echo "=== $profile_name build complete ==="
}

echo "=== Strawberry OS ISO Builder ==="

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

MODE="${1:-}"

case "$MODE" in
    --gui)
        build_iso "$PROJECT_DIR/archiso-gui" "GUI (Cinnamon Desktop)"
        ;;
    --all)
        build_iso "$PROJECT_DIR/archiso" "Headless (CLI only)"
        build_iso "$PROJECT_DIR/archiso-gui" "GUI (Cinnamon Desktop)"
        ;;
    *)
        build_iso "$PROJECT_DIR/archiso" "Headless (CLI only)"
        ;;
esac

echo ""
echo "=== All builds complete ==="
ls -lh "$OUTPUT_DIR"/*.iso 2>/dev/null || echo "(no ISOs found — check build logs)"
