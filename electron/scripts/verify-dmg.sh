#!/usr/bin/env bash
set -euo pipefail

DMG_FILE="${1:?DMG file path required}"

echo "Verifying DMG: $DMG_FILE"

# Check file exists
if [ ! -f "$DMG_FILE" ]; then
    echo "Error: DMG file not found"
    exit 1
fi

# Verify DMG integrity
echo "Checking DMG integrity..."
hdiutil verify "$DMG_FILE"

# Mount and inspect
MOUNT_POINT=$(mktemp -d)
echo "Mounting DMG to $MOUNT_POINT..."
hdiutil attach "$DMG_FILE" -mountpoint "$MOUNT_POINT" -nobrowse

# Check app exists
APP_PATH="$MOUNT_POINT/Tachikoma.app"
if [ ! -d "$APP_PATH" ]; then
    echo "Error: App not found in DMG"
    hdiutil detach "$MOUNT_POINT"
    exit 1
fi

# Check code signature
echo "Checking code signature..."
codesign --verify --deep --strict "$APP_PATH"
if [ $? -eq 0 ]; then
    echo "Code signature: Valid"
else
    echo "Warning: Code signature invalid or missing"
fi

# Check architecture
echo "Checking architecture..."
MAIN_BINARY="$APP_PATH/Contents/MacOS/Tachikoma"
file "$MAIN_BINARY"
lipo -info "$MAIN_BINARY"

# Check entitlements
echo "Checking entitlements..."
codesign -d --entitlements - "$APP_PATH" 2>/dev/null || echo "No entitlements found"

# Check notarization
echo "Checking notarization..."
spctl --assess --type execute "$APP_PATH" 2>&1 || echo "Not notarized or Gatekeeper disabled"

# Cleanup
hdiutil detach "$MOUNT_POINT"
rmdir "$MOUNT_POINT"

echo "DMG verification complete!"