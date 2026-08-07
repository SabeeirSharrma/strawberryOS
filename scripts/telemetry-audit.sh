#!/usr/bin/env bash
# telemetry-audit.sh — Verify the zero-telemetry baseline.
# Scans the project for known telemetry patterns and reports findings.
#
# Usage: ./scripts/telemetry-audit.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "=== Strawberry OS Telemetry Audit ==="
echo ""
echo "Scanning: $PROJECT_DIR"
echo ""

ISSUES=0

# Known telemetry/analytics patterns to flag
PATTERNS=(
    "analytics"
    "telemetry"
    "tracking"
    "phone.home"
    "metrics\.collect"
    "google.analytics"
    "sentry"
    "mixpanel"
    "amplitude"
    "segment\.io"
    "datadog\.com"
    "crashlytics"
)

for pattern in "${PATTERNS[@]}"; do
    # Search source files, excluding this script, .git, target/, and known-good patterns
    # Known-good: 'telemetry = false' enforcement, README/docs mentioning zero-telemetry
    MATCHES=$(rg -i --glob '!.git' --glob '!telemetry-audit.sh' --glob '!target/' --glob '!*.lock' --glob '!README.md' "$pattern" "$PROJECT_DIR" 2>/dev/null | \
        grep -v 'telemetry.*false' | \
        grep -v 'Forcing.*false' | \
        grep -v 'enforces telemetry' | \
        grep -v 'telemetry = false' | \
        grep -v 'pub telemetry' | \
        grep -v 'config.general.telemetry' | \
        grep -v 'Telemetry:' | \
        grep -v 'telemetry: {}' | \
        grep -v 'Telemetry flag' || true)
    if [ -n "$MATCHES" ]; then
        echo -e "${YELLOW}FLAG:${NC} Pattern '$pattern' found in:"
        echo "$MATCHES" | head -10
        echo ""
        ISSUES=$((ISSUES + 1))
    fi
done

# Check for hardcoded phone-home URLs (exclude known-good ones)
URL_MATCHES=$(rg -i --glob '!.git' --glob '!telemetry-audit.sh' --glob '!target/' 'https?://' "$PROJECT_DIR" 2>/dev/null | \
    grep -vi 'github.com' | \
    grep -vi 'crates.io' | \
    grep -vi 'example.com' | \
    grep -vi 'pool\.example' || true)

if [ -n "$URL_MATCHES" ]; then
    echo -e "${YELLOW}INFO:${NC} Non-GitHub URLs found (review these):"
    echo "$URL_MATCHES" | head -20
    echo ""
fi

# Verify global.toml has telemetry = false
GLOBAL_CONFIG="$PROJECT_DIR/archiso/airootfs/etc/strawberry/global.toml"
if [ -f "$GLOBAL_CONFIG" ]; then
    if grep -q 'telemetry = false' "$GLOBAL_CONFIG"; then
        echo -e "${GREEN}PASS:${NC} global.toml has telemetry = false"
    else
        echo -e "${RED}FAIL:${NC} global.toml missing 'telemetry = false'"
        ISSUES=$((ISSUES + 1))
    fi
fi

# Verify strawberryd enforces telemetry off at load
DAEMON_SRC="$PROJECT_DIR/strawberryd/src/config.rs"
if [ -f "$DAEMON_SRC" ]; then
    if grep -q 'telemetry.*false\|Forcing.*false' "$DAEMON_SRC"; then
        echo -e "${GREEN}PASS:${NC} strawberryd enforces telemetry off at config load"
    else
        echo -e "${RED}FAIL:${NC} strawberryd does not enforce telemetry = false"
        ISSUES=$((ISSUES + 1))
    fi
fi

# Summary
echo ""
echo "=== Audit Summary ==="
if [ "$ISSUES" -eq 0 ]; then
    echo -e "${GREEN}PASS:${NC} No telemetry issues found."
else
    echo -e "${RED}WARN:${NC} $ISSUES issue(s) found — review above."
fi

exit $ISSUES
