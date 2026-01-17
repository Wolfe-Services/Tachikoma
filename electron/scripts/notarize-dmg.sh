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