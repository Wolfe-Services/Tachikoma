#!/usr/bin/env bash
set -euo pipefail

TAG="${1:?Tag required}"

echo "=== Verifying Release Tag: $TAG ==="

# Check tag exists
if ! git rev-parse "$TAG" &> /dev/null; then
    echo "Error: Tag $TAG does not exist"
    exit 1
fi

# Show tag details
echo ""
echo "Tag Details:"
git show "$TAG" --no-patch

# Verify signature if signed
echo ""
echo "Signature Verification:"
if git tag -v "$TAG" 2>/dev/null; then
    echo "Tag is signed and verified"
else
    echo "Tag is not signed (or signature could not be verified)"
fi

# Check associated commits
echo ""
echo "Commits since previous tag:"
PREV_TAG=$(git describe --tags --abbrev=0 "$TAG^" 2>/dev/null || echo "")
if [ -n "$PREV_TAG" ]; then
    git log --oneline "$PREV_TAG..$TAG"
else
    echo "(No previous tag found)"
fi

echo ""
echo "=== Verification Complete ==="