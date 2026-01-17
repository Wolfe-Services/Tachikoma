#!/usr/bin/env bash
set -euo pipefail

APP_OR_DMG="${1:?App or DMG path required}"

echo "=== Notarization Status Check ==="
echo "File: $APP_OR_DMG"
echo ""

# Check if stapled
echo "1. Checking stapled ticket..."
if xcrun stapler validate "$APP_OR_DMG" 2>&1; then
    echo "   ✓ Notarization ticket is stapled"
else
    echo "   ✗ No notarization ticket found"
fi

# Check Gatekeeper assessment
echo ""
echo "2. Gatekeeper assessment..."
if [ -d "$APP_OR_DMG" ]; then
    # App bundle
    if spctl --assess --type execute "$APP_OR_DMG" 2>&1; then
        echo "   ✓ Gatekeeper approved (app)"
    else
        echo "   ✗ Gatekeeper rejected"
    fi
else
    # DMG
    if spctl --assess --type open --context context:primary-signature "$APP_OR_DMG" 2>&1; then
        echo "   ✓ Gatekeeper approved (dmg)"
    else
        echo "   ✗ Gatekeeper rejected"
    fi
fi

# Check code signature
echo ""
echo "3. Code signature..."
codesign --verify --deep --strict "$APP_OR_DMG" && echo "   ✓ Valid signature" || echo "   ✗ Invalid signature"

echo ""
echo "=== Check Complete ==="