# Tag Rollback Procedures

## Overview

This document outlines the procedures for rolling back git tags and releases in case of issues with a published release.

## Prerequisites

Before performing a rollback, ensure you have:
- Admin access to the GitHub repository
- Local git repository with proper credentials
- Understanding of the impact (users may have already downloaded the release)

## Rollback Scenarios

### 1. Tag Not Yet Pushed (Local Only)

If the tag was created locally but not yet pushed:

```bash
# Delete the local tag
git tag -d v1.2.3

# Recreate with different commit if needed
git tag -a v1.2.3 -m "Release v1.2.3" <commit-hash>
```

### 2. Tag Pushed but Release Not Created

If the tag was pushed but the GitHub Actions release workflow hasn't completed:

```bash
# Delete the remote tag
git push origin :refs/tags/v1.2.3

# Delete the local tag
git tag -d v1.2.3

# Cancel the running GitHub Actions workflow if needed
# (Go to GitHub > Actions > Cancel the running workflow)
```

### 3. Full Release Published

If the release is already published on GitHub:

#### Step 1: Mark Release as Pre-release
1. Go to GitHub Releases page
2. Edit the problematic release
3. Check "This is a pre-release"
4. Update description to indicate issues

#### Step 2: Delete GitHub Release
```bash
# Using GitHub CLI (if available)
gh release delete v1.2.3 --yes

# Or manually via GitHub web interface:
# Go to Releases > Click on release > Delete release
```

#### Step 3: Delete Git Tag
```bash
# Delete remote tag
git push origin :refs/tags/v1.2.3

# Delete local tag
git tag -d v1.2.3

# Verify tag is deleted
git ls-remote --tags origin
```

#### Step 4: Notify Users (if applicable)
- Send announcement about the rollback
- Update any documentation that referenced the problematic version
- Consider creating a hotfix release if critical

## Emergency Rollback Script

For emergency situations, use the rollback script:

```bash
#!/usr/bin/env bash
# Usage: ./scripts/rollback-tag.sh v1.2.3

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
if gh release view "$TAG" >/dev/null 2>&1; then
    echo "Deleting GitHub release: $TAG"
    gh release delete "$TAG" --yes
    echo "GitHub release deleted"
else
    echo "No GitHub release found for $TAG"
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
```

## Prevention Best Practices

To minimize the need for rollbacks:

### Pre-Release Checklist
- [ ] All tests pass locally and in CI
- [ ] Version number is correct and follows semver
- [ ] Release notes are accurate and complete
- [ ] All artifacts build successfully
- [ ] Security scans pass
- [ ] Breaking changes are documented

### Release Process
1. Create release candidate tags first (e.g., v1.2.3-rc.1)
2. Test release candidates thoroughly
3. Only create final tags from tested release candidates
4. Use signed tags for production releases

### Monitoring
- Set up alerts for failed release workflows
- Monitor download metrics for unusual patterns
- Have rollback procedures readily available
- Document incident response procedures

## Recovery After Rollback

After rolling back a problematic release:

1. **Investigate**: Understand what went wrong
2. **Fix**: Address the root cause
3. **Test**: Verify the fix thoroughly
4. **Document**: Update procedures to prevent recurrence
5. **Release**: Create a new version with the fix

### Version Strategy After Rollback

- **Patch release**: If rollback was due to bug (increment patch version)
- **Same version**: Only if no one downloaded the problematic release
- **Skip version**: Consider skipping the problematic version number

## Support Contacts

For assistance with rollbacks:
- Development team lead
- DevOps/Release engineer
- Project maintainers

## Related Documentation

- [Release Process](../build-pipeline.md)
- [Git Workflow](../guides/git-workflow.md)
- [Emergency Procedures](../guides/emergency-procedures.md)