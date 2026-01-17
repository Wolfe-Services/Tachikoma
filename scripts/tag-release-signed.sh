#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:?Version required}"
MESSAGE="${2:-Release v${VERSION}}"

TAG="v${VERSION}"

# Check for GPG key
if ! git config user.signingkey &> /dev/null; then
    echo "No GPG signing key configured."
    echo "Set with: git config user.signingkey <key-id>"
    exit 1
fi

# Create signed annotated tag
echo "Creating signed tag: $TAG"
git tag -s -a "$TAG" -m "$MESSAGE"

# Verify signature
echo ""
echo "Verifying signature:"
git tag -v "$TAG"

echo ""
echo "Signed tag created successfully!"
echo "Push with: git push origin $TAG"