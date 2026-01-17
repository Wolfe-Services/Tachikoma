#!/usr/bin/env bash
set -euo pipefail

APP_PATH="${1:?App path required}"
IDENTITY="${CSC_NAME:-Developer ID Application}"

echo "Signing macOS app: $APP_PATH"
echo "Using identity: $IDENTITY"

# Sign all frameworks and helpers first (deep signing)
find "$APP_PATH" -name "*.framework" -o -name "*.dylib" -o -name "*.node" | while read -r item; do
    echo "Signing: $item"
    codesign --force --options runtime \
        --entitlements "build/entitlements.mac.plist" \
        --sign "$IDENTITY" \
        --timestamp \
        "$item"
done

# Sign helper apps
find "$APP_PATH/Contents/Frameworks" -name "*.app" | while read -r helper; do
    echo "Signing helper: $helper"
    codesign --force --options runtime \
        --entitlements "build/entitlements.mac.plist" \
        --sign "$IDENTITY" \
        --timestamp \
        --deep \
        "$helper"
done

# Sign the main app
echo "Signing main app..."
codesign --force --options runtime \
    --entitlements "build/entitlements.mac.plist" \
    --sign "$IDENTITY" \
    --timestamp \
    --deep \
    "$APP_PATH"

# Verify signature
echo "Verifying signature..."
codesign --verify --deep --strict --verbose=2 "$APP_PATH"

# Display signature info
echo "Signature info:"
codesign --display --verbose=2 "$APP_PATH"

echo "Code signing complete!"