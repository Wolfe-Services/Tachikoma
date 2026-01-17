#!/usr/bin/env bash
set -euo pipefail

APP_PATH="${1:?App path required}"

echo "=== Code Signature Verification ==="
echo "App: $APP_PATH"
echo ""

# Basic verification
echo "1. Basic verification..."
if codesign --verify --deep --strict "$APP_PATH"; then
    echo "   ✓ Signature is valid"
else
    echo "   ✗ Signature is INVALID"
    exit 1
fi

# Detailed verification
echo ""
echo "2. Detailed verification..."
codesign --verify --deep --strict --verbose=2 "$APP_PATH" 2>&1 | head -20

# Display signing info
echo ""
echo "3. Signing information..."
codesign --display --verbose=2 "$APP_PATH" 2>&1

# Check entitlements
echo ""
echo "4. Entitlements..."
codesign --display --entitlements - "$APP_PATH" 2>/dev/null || echo "   No entitlements"

# Gatekeeper assessment
echo ""
echo "5. Gatekeeper assessment..."
if spctl --assess --type execute "$APP_PATH" 2>&1; then
    echo "   ✓ Gatekeeper approved"
else
    echo "   ⚠ Gatekeeper rejected (may need notarization)"
fi

echo ""
echo "=== Verification Complete ==="