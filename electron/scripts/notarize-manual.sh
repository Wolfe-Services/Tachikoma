#!/usr/bin/env bash
set -euo pipefail

APP_PATH="${1:?App path required}"
BUNDLE_ID="${2:-io.tachikoma.app}"

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