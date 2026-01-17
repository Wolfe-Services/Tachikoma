#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:?Version required}"
MESSAGE="${2:-Release v${VERSION}}"

# Validate version format
if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-z]+\.[0-9]+)?$ ]]; then
    echo "Invalid version format: $VERSION"
    echo "Expected: X.Y.Z or X.Y.Z-prerelease.N"
    exit 1
fi

TAG="v${VERSION}"

# Check if tag already exists
if git rev-parse "$TAG" &> /dev/null; then
    echo "Tag $TAG already exists!"
    exit 1
fi

# Verify we're on the right branch
BRANCH=$(git rev-parse --abbrev-ref HEAD)
if [[ "$VERSION" =~ -beta || "$VERSION" =~ -alpha ]]; then
    # Pre-releases can be from develop or feature branches
    echo "Creating pre-release tag from branch: $BRANCH"
else
    # Stable releases should be from main
    if [ "$BRANCH" != "main" ]; then
        echo "Warning: Creating stable release from non-main branch ($BRANCH)"
        read -p "Continue? [y/N] " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
fi

# Verify working tree is clean
if [ -n "$(git status --porcelain)" ]; then
    echo "Working tree is not clean. Commit or stash changes first."
    exit 1
fi

# Create annotated tag
echo "Creating tag: $TAG"
git tag -a "$TAG" -m "$MESSAGE"

# Show tag info
echo ""
echo "Tag created:"
git show "$TAG" --no-patch

echo ""
echo "To push this tag:"
echo "  git push origin $TAG"
echo ""
echo "To delete this tag (if needed):"
echo "  git tag -d $TAG"