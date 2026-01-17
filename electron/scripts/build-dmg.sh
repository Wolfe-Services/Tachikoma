#!/usr/bin/env bash
set -euo pipefail

ARCH="${1:-$(uname -m)}"
VERSION=$(node -p "require('./package.json').version")

echo "Building DMG for macOS ($ARCH)..."

# Map architecture names
case "$ARCH" in
    x86_64|x64)
        ELECTRON_ARCH="x64"
        ;;
    arm64|aarch64)
        ELECTRON_ARCH="arm64"
        ;;
    universal)
        ELECTRON_ARCH="universal"
        ;;
    *)
        echo "Unknown architecture: $ARCH"
        exit 1
        ;;
esac

# Build DMG
if [ "$ELECTRON_ARCH" = "universal" ]; then
    # Build universal binary
    npx electron-builder --mac --universal
else
    npx electron-builder --mac dmg --arch "$ELECTRON_ARCH"
fi

# Verify output
DMG_FILE="release/${VERSION}/Tachikoma-${VERSION}-${ELECTRON_ARCH}.dmg"
if [ -f "$DMG_FILE" ]; then
    echo "DMG created: $DMG_FILE"
    ls -lh "$DMG_FILE"

    # Verify DMG
    echo "Verifying DMG..."
    hdiutil verify "$DMG_FILE"

    echo "DMG contents:"
    hdiutil attach "$DMG_FILE" -mountpoint /tmp/dmg-verify
    ls -la /tmp/dmg-verify/
    hdiutil detach /tmp/dmg-verify
else
    echo "Error: DMG not found at $DMG_FILE"
    exit 1
fi

echo "DMG build complete!"