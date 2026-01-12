# 496 - macOS Code Signing

**Phase:** 23 - Build & Distribution
**Spec ID:** 496
**Status:** Planned
**Dependencies:** 495-macos-dmg
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Configure macOS code signing using Apple Developer ID certificates to enable distribution outside the Mac App Store while satisfying Gatekeeper requirements.

---

## Acceptance Criteria

- [ ] Developer ID Application certificate configured
- [ ] All binaries and frameworks signed
- [ ] Entitlements properly applied
- [ ] Hardened runtime enabled
- [ ] Signature verification passes
- [ ] CI/CD signing automated

---

## Implementation Details

### 1. Signing Configuration

Update `electron/electron-builder.config.js`:

```javascript
// macOS signing configuration
mac: {
  // ... other mac config

  // Code signing
  identity: 'Developer ID Application: Your Company Name (TEAM_ID)',

  // Hardened runtime (required for notarization)
  hardenedRuntime: true,

  // Entitlements
  entitlements: 'build/entitlements.mac.plist',
  entitlementsInherit: 'build/entitlements.mac.plist',

  // Sign all nested code
  signIgnore: [],

  // Timestamp server
  timestamp: 'http://timestamp.apple.com/ts01',

  // Gatekeeper assess (local verification)
  gatekeeperAssess: true,

  // Strict verification
  strictVerify: true,
},
```

### 2. Environment Variables

Required environment variables for signing:

```bash
# Certificate (base64-encoded .p12 file)
CSC_LINK=base64-encoded-certificate

# Certificate password
CSC_KEY_PASSWORD=certificate-password

# Or use keychain identity
CSC_NAME="Developer ID Application: Your Company Name (TEAM_ID)"

# Optional: Disable signing (for testing)
# CSC_IDENTITY_AUTO_DISCOVERY=false
```

### 3. Signing Script

Create `electron/scripts/sign-macos.sh`:

```bash
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
```

### 4. Entitlements File

Update `electron/build/entitlements.mac.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <!-- Hardened Runtime -->
    <key>com.apple.security.cs.allow-jit</key>
    <true/>

    <key>com.apple.security.cs.allow-unsigned-executable-memory</key>
    <true/>

    <key>com.apple.security.cs.disable-library-validation</key>
    <true/>

    <!-- Network -->
    <key>com.apple.security.network.client</key>
    <true/>

    <key>com.apple.security.network.server</key>
    <true/>

    <!-- File access -->
    <key>com.apple.security.files.user-selected.read-write</key>
    <true/>

    <key>com.apple.security.files.downloads.read-write</key>
    <true/>

    <!-- Automation -->
    <key>com.apple.security.automation.apple-events</key>
    <true/>

    <!-- Device access (optional) -->
    <!-- <key>com.apple.security.device.audio-input</key>
    <true/> -->
</dict>
</plist>
```

### 5. CI/CD Signing Setup

Create `.github/workflows/sign-macos.yml`:

```yaml
name: Sign macOS

on:
  workflow_call:
    inputs:
      artifact-name:
        required: true
        type: string
    secrets:
      APPLE_CERTIFICATE:
        required: true
      APPLE_CERTIFICATE_PASSWORD:
        required: true

jobs:
  sign:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4

      - name: Download artifact
        uses: actions/download-artifact@v4
        with:
          name: ${{ inputs.artifact-name }}
          path: ./unsigned

      - name: Import certificate
        env:
          CERTIFICATE: ${{ secrets.APPLE_CERTIFICATE }}
          PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
        run: |
          # Create temporary keychain
          KEYCHAIN_PATH=$RUNNER_TEMP/signing.keychain-db
          KEYCHAIN_PASSWORD=$(openssl rand -base64 32)

          security create-keychain -p "$KEYCHAIN_PASSWORD" "$KEYCHAIN_PATH"
          security set-keychain-settings -lut 21600 "$KEYCHAIN_PATH"
          security unlock-keychain -p "$KEYCHAIN_PASSWORD" "$KEYCHAIN_PATH"

          # Import certificate
          echo "$CERTIFICATE" | base64 --decode > $RUNNER_TEMP/certificate.p12
          security import $RUNNER_TEMP/certificate.p12 \
            -P "$PASSWORD" \
            -A \
            -t cert \
            -f pkcs12 \
            -k "$KEYCHAIN_PATH"

          security list-keychain -d user -s "$KEYCHAIN_PATH"

          # Allow codesign to access keychain
          security set-key-partition-list -S apple-tool:,apple:,codesign: \
            -s -k "$KEYCHAIN_PASSWORD" "$KEYCHAIN_PATH"

      - name: Sign app
        run: |
          APP_PATH=$(find ./unsigned -name "*.app" -type d | head -1)
          ./electron/scripts/sign-macos.sh "$APP_PATH"

      - name: Verify signature
        run: |
          APP_PATH=$(find ./unsigned -name "*.app" -type d | head -1)
          codesign --verify --deep --strict "$APP_PATH"
          spctl --assess --type execute "$APP_PATH"

      - name: Upload signed artifact
        uses: actions/upload-artifact@v4
        with:
          name: ${{ inputs.artifact-name }}-signed
          path: ./unsigned
```

### 6. Signature Verification Script

Create `electron/scripts/verify-signature.sh`:

```bash
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
```

---

## Testing Requirements

1. App signature verifies with `codesign --verify`
2. All nested code is properly signed
3. Entitlements are correctly applied
4. Hardened runtime is enabled
5. CI signing works with imported certificate

---

## Related Specs

- Depends on: [495-macos-dmg.md](495-macos-dmg.md)
- Next: [497-macos-notarize.md](497-macos-notarize.md)
- Related: [169-security-config.md](../phase-08-electron/169-security-config.md)
