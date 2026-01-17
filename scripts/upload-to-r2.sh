#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:?Version required}"
BUCKET="${R2_BUCKET:-tachikoma-releases}"

# Required environment variables
: "${R2_ACCESS_KEY_ID:?R2_ACCESS_KEY_ID required}"
: "${R2_SECRET_ACCESS_KEY:?R2_SECRET_ACCESS_KEY required}"
: "${R2_ENDPOINT:?R2_ENDPOINT required}"

echo "Uploading release v${VERSION} to R2..."

# Configure AWS CLI for R2
export AWS_ACCESS_KEY_ID="$R2_ACCESS_KEY_ID"
export AWS_SECRET_ACCESS_KEY="$R2_SECRET_ACCESS_KEY"

# Check if AWS CLI is installed
if ! command -v aws &> /dev/null; then
    echo "AWS CLI is not installed. Please install it first."
    exit 1
fi

# Upload release artifacts
UPLOAD_COUNT=0
for file in electron/release/*; do
    if [ -f "$file" ]; then
        filename=$(basename "$file")
        # Skip blockmap files and other temporary files
        if [[ "$filename" == *.blockmap ]] || [[ "$filename" == builder-* ]]; then
            continue
        fi
        
        echo "Uploading: $filename"
        
        # Determine content type based on extension
        content_type="application/octet-stream"
        case "${filename##*.}" in
            yml|yaml)
                content_type="text/yaml"
                ;;
            dmg)
                content_type="application/x-apple-diskimage"
                ;;
            exe)
                content_type="application/x-msdos-program"
                ;;
            AppImage)
                content_type="application/x-executable"
                ;;
            deb)
                content_type="application/vnd.debian.binary-package"
                ;;
            rpm)
                content_type="application/x-rpm"
                ;;
        esac

        aws s3 cp "$file" "s3://${BUCKET}/releases/v${VERSION}/${filename}" \
            --endpoint-url "$R2_ENDPOINT" \
            --content-type "$content_type" \
            --cache-control "public, max-age=31536000" # 1 year for versioned files

        UPLOAD_COUNT=$((UPLOAD_COUNT + 1))
    fi
done

# Upload latest.yml files for auto-update
for yml in electron/release/*.yml; do
    if [ -f "$yml" ]; then
        filename=$(basename "$yml")
        echo "Uploading latest manifest: $filename"

        aws s3 cp "$yml" "s3://${BUCKET}/releases/latest/${filename}" \
            --endpoint-url "$R2_ENDPOINT" \
            --content-type "text/yaml" \
            --cache-control "public, max-age=300" # 5 minutes for update manifests

        UPLOAD_COUNT=$((UPLOAD_COUNT + 1))
    fi
done

echo ""
echo "Upload complete! Uploaded $UPLOAD_COUNT files."
echo "Download URL: https://releases.tachikoma.dev/releases/v${VERSION}/"
echo "Latest URL: https://releases.tachikoma.dev/latest"