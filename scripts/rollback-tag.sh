#!/usr/bin/env bash
set -euo pipefail

TAG="${1:?Tag required (e.g., v1.2.3)}"

echo "=== EMERGENCY TAG ROLLBACK: $TAG ==="
echo "This will:"
echo "  1. Delete the GitHub release (if exists)"
echo "  2. Delete the remote git tag"
echo "  3. Delete the local git tag"
echo ""
read -p "Are you sure you want to rollback $TAG? [y/N] " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Rollback cancelled"
    exit 1
fi

echo "Step 1: Checking if GitHub release exists..."
if command -v gh >/dev/null 2>&1; then
    if gh release view "$TAG" >/dev/null 2>&1; then
        echo "Deleting GitHub release: $TAG"
        gh release delete "$TAG" --yes
        echo "GitHub release deleted"
    else
        echo "No GitHub release found for $TAG"
    fi
else
    echo "GitHub CLI not installed - skipping release deletion"
    echo "Manually delete release at: https://github.com/owner/repo/releases"
fi

echo "Step 2: Deleting remote git tag..."
if git ls-remote --exit-code --tags origin "refs/tags/$TAG" >/dev/null 2>&1; then
    git push origin ":refs/tags/$TAG"
    echo "Remote tag deleted"
else
    echo "Remote tag $TAG not found"
fi

echo "Step 3: Deleting local git tag..."
if git rev-parse --verify "$TAG" >/dev/null 2>&1; then
    git tag -d "$TAG"
    echo "Local tag deleted"
else
    echo "Local tag $TAG not found"
fi

echo ""
echo "=== ROLLBACK COMPLETE ==="
echo "Tag $TAG has been completely removed"
echo ""
echo "Next steps:"
echo "  1. Fix the issues that caused the rollback"
echo "  2. Create a new tag with incremented version"
echo "  3. Test thoroughly before release"
echo "  4. Communicate rollback to users if necessary"