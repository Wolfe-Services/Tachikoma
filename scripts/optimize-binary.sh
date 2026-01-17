#!/usr/bin/env bash
set -euo pipefail

BINARY="${1:?Binary path required}"

echo "Optimizing binary: $BINARY"

# Strip debug symbols (if not already stripped)
if command -v strip &> /dev/null; then
    strip "$BINARY" 2>/dev/null || true
fi

# UPX compression (optional, can affect startup time)
if command -v upx &> /dev/null && [ "${USE_UPX:-false}" = "true" ]; then
    upx --best --lzma "$BINARY"
fi

# Report final size
SIZE=$(du -h "$BINARY" | cut -f1)
echo "Final size: $SIZE"