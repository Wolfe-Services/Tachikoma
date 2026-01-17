# 497 - macOS Notarization

**Phase:** 23 - Build & Distribution
**Spec ID:** 497
**Status:** Planned
**Dependencies:** 496-macos-signing
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Configure Apple notarization for the macOS application to satisfy Gatekeeper requirements on macOS 10.15+ and enable users to run the app without security warnings.

---

## Acceptance Criteria

- [x] App submitted to Apple notarization service
- [x] Notarization ticket stapled to app
- [x] DMG notarization configured
- [x] Notarization status monitoring
- [x] CI/CD notarization automated
- [x] Error handling for notarization failures

---

## Implementation Details

### 1. Notarization Configuration

Update `electron/electron-builder.config.js`:

```javascript
// Add notarization hook
afterSign: 'scripts/notarize.js',

// macOS specific
mac: {
  // ... existing config

  // Notarization requires hardened runtime
  hardenedRuntime: true,

  // Gatekeeper assess after notarization
  gatekeeperAssess: true,
},
```

### 2. Notarization Script

Create `electron/scripts/notarize.js`:

```javascript
/**
 * Notarization script for macOS builds
 */

const { notarize } = require('@electron/notarize');
const path = require('path');

exports.default = async function notarizing(context) {
  const { electronPlatformName, appOutDir } = context;

  // Only notarize macOS builds
  if (electronPlatformName !== 'darwin') {
    console.log('Skipping notarization (not macOS)');
    return;
  }

  // Skip if no credentials
  if (!process.env.APPLE_ID || !process.env.APPLE_APP_SPECIFIC_PASSWORD) {
    console.log('Skipping notarization (no credentials)');
    return;
  }

  const appName = context.packager.appInfo.productFilename;
  const appPath = path.join(appOutDir, `${appName}.app`);

  console.log(`Notarizing ${appPath}...`);

  try {
    await notarize({
      tool: 'notarytool',
      appPath,
      appleId: process.env.APPLE_ID,
      appleIdPassword: process.env.APPLE_APP_SPECIFIC_PASSWORD,
      teamId: process.env.APPLE_TEAM_ID,
    });

    console.log('Notarization complete!');
  } catch (error) {
    console.error('Notarization failed:', error);
    throw error;
  }
};
```

### 3. Manual Notarization Script

Create `electron/scripts/notarize-manual.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

APP_PATH="${1:?App path required}"
BUNDLE_ID="${2:-com.tachikoma.app}"

# Required environment variables
: "${APPLE_ID:?APPLE_ID required}"
: "${APPLE_APP_SPECIFIC_PASSWORD:?APPLE_APP_SPECIFIC_PASSWORD required}"
: "${APPLE_TEAM_ID:?APPLE_TEAM_ID required}"

echo "Notarizing: $APP_PATH"
echo "Bundle ID: $BUNDLE_ID"
echo "Team ID: $APPLE_TEAM_ID"

# Create ZIP for submission
ZIP_PATH="${APP_PATH%.app}.zip"
echo "Creating ZIP for submission..."
ditto -c -k --keepParent "$APP_PATH" "$ZIP_PATH"

# Submit for notarization
echo "Submitting to Apple..."
SUBMISSION_OUTPUT=$(xcrun notarytool submit "$ZIP_PATH" \
    --apple-id "$APPLE_ID" \
    --password "$APPLE_APP_SPECIFIC_PASSWORD" \
    --team-id "$APPLE_TEAM_ID" \
    --wait \
    --output-format json)

echo "$SUBMISSION_OUTPUT"

# Parse submission ID
SUBMISSION_ID=$(echo "$SUBMISSION_OUTPUT" | python3 -c "import sys,json; print(json.load(sys.stdin)['id'])")
STATUS=$(echo "$SUBMISSION_OUTPUT" | python3 -c "import sys,json; print(json.load(sys.stdin)['status'])")

echo "Submission ID: $SUBMISSION_ID"
echo "Status: $STATUS"

if [ "$STATUS" != "Accepted" ]; then
    echo "Notarization failed!"

    # Get detailed log
    echo "Fetching notarization log..."
    xcrun notarytool log "$SUBMISSION_ID" \
        --apple-id "$APPLE_ID" \
        --password "$APPLE_APP_SPECIFIC_PASSWORD" \
        --team-id "$APPLE_TEAM_ID"

    exit 1
fi

# Staple the ticket
echo "Stapling notarization ticket..."
xcrun stapler staple "$APP_PATH"

# Verify stapling
echo "Verifying stapling..."
xcrun stapler validate "$APP_PATH"

# Clean up
rm -f "$ZIP_PATH"

echo "Notarization complete!"
```

### 4. Notarize DMG Script

Create `electron/scripts/notarize-dmg.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

DMG_PATH="${1:?DMG path required}"

# Required environment variables
: "${APPLE_ID:?APPLE_ID required}"
: "${APPLE_APP_SPECIFIC_PASSWORD:?APPLE_APP_SPECIFIC_PASSWORD required}"
: "${APPLE_TEAM_ID:?APPLE_TEAM_ID required}"

echo "Notarizing DMG: $DMG_PATH"

# Submit DMG for notarization
echo "Submitting to Apple..."
xcrun notarytool submit "$DMG_PATH" \
    --apple-id "$APPLE_ID" \
    --password "$APPLE_APP_SPECIFIC_PASSWORD" \
    --team-id "$APPLE_TEAM_ID" \
    --wait

# Staple the ticket to DMG
echo "Stapling notarization ticket..."
xcrun stapler staple "$DMG_PATH"

# Verify
echo "Verifying..."
xcrun stapler validate "$DMG_PATH"
spctl --assess --type open --context context:primary-signature "$DMG_PATH"

echo "DMG notarization complete!"
```

### 5. CI/CD Notarization

Create `.github/workflows/notarize.yml`:

```yaml
name: Notarize macOS

on:
  workflow_call:
    inputs:
      artifact-name:
        required: true
        type: string
    secrets:
      APPLE_ID:
        required: true
      APPLE_APP_SPECIFIC_PASSWORD:
        required: true
      APPLE_TEAM_ID:
        required: true

jobs:
  notarize:
    runs-on: macos-latest
    timeout-minutes: 30
    steps:
      - uses: actions/checkout@v4

      - name: Download signed artifact
        uses: actions/download-artifact@v4
        with:
          name: ${{ inputs.artifact-name }}
          path: ./signed

      - name: Notarize app
        env:
          APPLE_ID: ${{ secrets.APPLE_ID }}
          APPLE_APP_SPECIFIC_PASSWORD: ${{ secrets.APPLE_APP_SPECIFIC_PASSWORD }}
          APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
        run: |
          APP_PATH=$(find ./signed -name "*.app" -type d | head -1)
          ./electron/scripts/notarize-manual.sh "$APP_PATH"

      - name: Notarize DMG (if exists)
        env:
          APPLE_ID: ${{ secrets.APPLE_ID }}
          APPLE_APP_SPECIFIC_PASSWORD: ${{ secrets.APPLE_APP_SPECIFIC_PASSWORD }}
          APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
        run: |
          DMG_PATH=$(find ./signed -name "*.dmg" | head -1)
          if [ -n "$DMG_PATH" ]; then
            ./electron/scripts/notarize-dmg.sh "$DMG_PATH"
          fi

      - name: Verify Gatekeeper
        run: |
          APP_PATH=$(find ./signed -name "*.app" -type d | head -1)
          spctl --assess --type execute "$APP_PATH"

      - name: Upload notarized artifact
        uses: actions/upload-artifact@v4
        with:
          name: ${{ inputs.artifact-name }}-notarized
          path: ./signed
```

### 6. Notarization Status Check

Create `electron/scripts/check-notarization.sh`:

```bash
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
```

---

## Testing Requirements

1. Notarization submission succeeds
2. Ticket is properly stapled
3. Gatekeeper approves the app
4. DMG notarization works
5. CI workflow completes successfully

---

## Related Specs

- Depends on: [496-macos-signing.md](496-macos-signing.md)
- Next: [498-windows-installer.md](498-windows-installer.md)
- Related: [502-auto-update-server.md](502-auto-update-server.md)
