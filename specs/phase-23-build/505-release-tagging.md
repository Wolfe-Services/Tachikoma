# 505 - Release Tagging

**Phase:** 23 - Build & Distribution
**Spec ID:** 505
**Status:** Planned
**Dependencies:** 504-version-management
**Estimated Context:** ~6% of Sonnet window

---

## Objective

Implement a consistent git tagging strategy for releases that triggers CI/CD pipelines and maintains a clear release history.

---

## Acceptance Criteria

- [x] Git tags follow v{version} format
- [x] Tags are annotated with release info
- [x] Tags trigger release workflows
- [x] Branch protection for release tags
- [x] Tag signing for verified releases
- [x] Tag rollback procedure documented

---

## Implementation Details

### 1. Tagging Script

Create `scripts/tag-release.sh`:

```bash
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
```

### 2. Signed Tags Script

Create `scripts/tag-release-signed.sh`:

```bash
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
```

### 3. Release Workflow Trigger

Create `.github/workflows/release-on-tag.yml`:

```yaml
name: Release on Tag

on:
  push:
    tags:
      - 'v*'

jobs:
  validate-tag:
    runs-on: ubuntu-latest
    outputs:
      version: ${{ steps.parse.outputs.version }}
      prerelease: ${{ steps.parse.outputs.prerelease }}
    steps:
      - name: Parse tag
        id: parse
        run: |
          TAG=${GITHUB_REF#refs/tags/v}
          echo "version=$TAG" >> $GITHUB_OUTPUT

          if [[ "$TAG" =~ -(alpha|beta|rc)\. ]]; then
            echo "prerelease=true" >> $GITHUB_OUTPUT
          else
            echo "prerelease=false" >> $GITHUB_OUTPUT
          fi

      - name: Validate version format
        run: |
          VERSION=${{ steps.parse.outputs.version }}
          if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-z]+\.[0-9]+)?$ ]]; then
            echo "Invalid version format: $VERSION"
            exit 1
          fi

  build:
    needs: validate-tag
    uses: ./.github/workflows/build-all.yml
    with:
      version: ${{ needs.validate-tag.outputs.version }}
    secrets: inherit

  release:
    needs: [validate-tag, build]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Download all artifacts
        uses: actions/download-artifact@v4
        with:
          path: release-artifacts

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v1
        with:
          files: release-artifacts/**/*
          prerelease: ${{ needs.validate-tag.outputs.prerelease == 'true' }}
          generate_release_notes: true
```

### 4. Branch Protection Rules

Configure in GitHub repository settings:

```yaml
# .github/settings.yml (if using probot/settings)
branches:
  - name: main
    protection:
      required_pull_request_reviews:
        required_approving_review_count: 1
      required_status_checks:
        strict: true
        contexts:
          - test
          - lint
      enforce_admins: false
      restrictions: null

# Tag protection (via GitHub UI or API)
# Protect tags matching: v*
```

### 5. Tag Management Commands

Add to `Makefile`:

```makefile
# Release tagging
.PHONY: tag tag-signed tag-list tag-delete

tag:
	@read -p "Version: " VERSION && ./scripts/tag-release.sh $$VERSION

tag-signed:
	@read -p "Version: " VERSION && ./scripts/tag-release-signed.sh $$VERSION

tag-list:
	@git tag -l "v*" --sort=-version:refname | head -10

tag-delete:
	@read -p "Tag to delete (e.g., v1.0.0): " TAG && \
		git tag -d $$TAG && \
		echo "Local tag deleted. To delete remote: git push origin :refs/tags/$$TAG"
```

### 6. Tag Verification Script

Create `scripts/verify-release-tag.sh`:

```bash
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
```

---

## Testing Requirements

1. Tags are created in correct format
2. CI triggers on tag push
3. Signed tags verify correctly
4. Pre-release tags marked appropriately
5. Tag deletion works for rollback

---

## Related Specs

- Depends on: [504-version-management.md](504-version-management.md)
- Next: [506-changelog.md](506-changelog.md)
- Related: [502-auto-update-server.md](502-auto-update-server.md)
