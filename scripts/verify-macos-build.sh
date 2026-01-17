#!/usr/bin/env bash
set -euo pipefail

DMG_FILE="${1:?DMG file required}"

echo "=== macOS Build Verification ==="
echo "File: ${DMG_FILE}"

ERRORS=0
MOUNT_POINT="/Volumes/Tachikoma-Test-$$"

cleanup() {
    if [ -d "$MOUNT_POINT" ]; then
        hdiutil detach "$MOUNT_POINT" -quiet || true
    fi
}
trap cleanup EXIT

# Mount DMG
echo ""
echo "--- Mounting DMG ---"
hdiutil attach "$DMG_FILE" -mountpoint "$MOUNT_POINT" -nobrowse -quiet

if [ ! -d "$MOUNT_POINT/Tachikoma.app" ]; then
    echo "ERROR: Tachikoma.app not found in DMG"
    exit 1
fi
echo "OK: App bundle found"

APP_PATH="$MOUNT_POINT/Tachikoma.app"

# Check code signing
echo ""
echo "--- Verifying Code Signature ---"
if codesign -v --deep --strict "$APP_PATH" 2>&1; then
    echo "OK: Code signature valid"
else
    echo "ERROR: Code signature verification failed"
    ((ERRORS++))
fi

# Check signing identity
echo ""
echo "--- Checking Signing Identity ---"
IDENTITY=$(codesign -dv "$APP_PATH" 2>&1 | grep "Authority" | head -1)
echo "$IDENTITY"

if [[ "$IDENTITY" == *"Developer ID Application"* ]]; then
    echo "OK: Signed with Developer ID"
else
    echo "WARNING: Not signed with Developer ID (may be development build)"
fi

# Check notarization
echo ""
echo "--- Checking Notarization ---"
if spctl -a -v "$APP_PATH" 2>&1 | grep -q "accepted"; then
    echo "OK: App is notarized and accepted by Gatekeeper"
else
    echo "WARNING: App may not be notarized"
fi

# Check entitlements
echo ""
echo "--- Checking Entitlements ---"
ENTITLEMENTS=$(codesign -d --entitlements :- "$APP_PATH" 2>&1)
echo "$ENTITLEMENTS" | head -20

# Required entitlements
REQUIRED_ENTITLEMENTS=(
    "com.apple.security.cs.allow-jit"
    "com.apple.security.cs.allow-unsigned-executable-memory"
)

for ent in "${REQUIRED_ENTITLEMENTS[@]}"; do
    if echo "$ENTITLEMENTS" | grep -q "$ent"; then
        echo "OK: Has entitlement $ent"
    else
        echo "WARNING: Missing entitlement $ent"
    fi
done

# Check app structure
echo ""
echo "--- Checking App Structure ---"

REQUIRED_FILES=(
    "Contents/MacOS/Tachikoma"
    "Contents/Info.plist"
    "Contents/Resources/app.asar"
    "Contents/Frameworks/Electron Framework.framework"
)

for file in "${REQUIRED_FILES[@]}"; do
    if [ -e "$APP_PATH/$file" ]; then
        echo "OK: Found $file"
    else
        echo "ERROR: Missing $file"
        ((ERRORS++))
    fi
done

# Check Info.plist
echo ""
echo "--- Checking Info.plist ---"
BUNDLE_ID=$(/usr/libexec/PlistBuddy -c "Print CFBundleIdentifier" "$APP_PATH/Contents/Info.plist")
VERSION=$(/usr/libexec/PlistBuddy -c "Print CFBundleShortVersionString" "$APP_PATH/Contents/Info.plist")
echo "Bundle ID: $BUNDLE_ID"
echo "Version: $VERSION"

# Check native modules
echo ""
echo "--- Checking Native Modules ---"
NATIVE_MODULES=$(find "$APP_PATH" -name "*.node" 2>/dev/null || true)
if [ -n "$NATIVE_MODULES" ]; then
    echo "Found native modules:"
    echo "$NATIVE_MODULES"

    # Verify each is signed
    while IFS= read -r module; do
        if codesign -v "$module" 2>&1; then
            echo "OK: $module signed"
        else
            echo "ERROR: $module not properly signed"
            ((ERRORS++))
        fi
    done <<< "$NATIVE_MODULES"
fi

echo ""
echo "=== macOS Verification Complete ==="
if [ "$ERRORS" -gt 0 ]; then
    echo "FAILED: ${ERRORS} error(s)"
    exit 1
else
    echo "PASSED"
fi