# 502 - Auto-Update Server

**Phase:** 23 - Build & Distribution
**Spec ID:** 502
**Status:** Planned
**Dependencies:** 494-electron-packaging
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Set up the server-side infrastructure for distributing application updates, supporting both GitHub Releases and self-hosted update servers for electron-updater.

---

## Acceptance Criteria

- [ ] GitHub Releases configured as update source
- [ ] Release assets properly named for auto-update
- [ ] Update manifest (latest.yml) generated
- [ ] Delta updates supported (where possible)
- [ ] Staging/beta channel support
- [ ] Self-hosted option documented

---

## Implementation Details

### 1. GitHub Releases Configuration

Update `electron/electron-builder.config.js`:

```javascript
// Publishing configuration
publish: {
  provider: 'github',
  owner: 'tachikoma',
  repo: 'tachikoma',
  releaseType: 'release',  // or 'draft' for manual publishing
  private: false,
  vPrefixedTagName: true,  // Use v1.0.0 format for tags
},

// Channel configuration
generateUpdatesFilesForAllChannels: true,
```

### 2. Release Asset Naming

electron-builder generates these files for auto-update:

```
macOS:
  - Tachikoma-{version}-arm64.dmg
  - Tachikoma-{version}-arm64.dmg.blockmap
  - Tachikoma-{version}-x64.dmg
  - Tachikoma-{version}-x64.dmg.blockmap
  - Tachikoma-{version}-arm64-mac.zip
  - Tachikoma-{version}-x64-mac.zip
  - latest-mac.yml

Windows:
  - Tachikoma-Setup-{version}.exe
  - Tachikoma-Setup-{version}.exe.blockmap
  - latest.yml

Linux:
  - Tachikoma-{version}.AppImage
  - latest-linux.yml
```

### 3. Update Manifest Files

Example `latest-mac.yml`:

```yaml
version: 1.0.0
files:
  - url: Tachikoma-1.0.0-arm64-mac.zip
    sha512: <sha512-hash>
    size: 123456789
  - url: Tachikoma-1.0.0-x64-mac.zip
    sha512: <sha512-hash>
    size: 123456789
path: Tachikoma-1.0.0-arm64-mac.zip
sha512: <sha512-hash>
releaseDate: '2024-01-15T10:00:00.000Z'
```

### 4. Release Script

Create `scripts/release.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:?Version required (e.g., 1.0.0)}"
CHANNEL="${2:-stable}"  # stable, beta, alpha

echo "Preparing release v${VERSION} (${CHANNEL} channel)..."

# Verify we're on main branch for stable releases
if [ "$CHANNEL" = "stable" ]; then
    BRANCH=$(git rev-parse --abbrev-ref HEAD)
    if [ "$BRANCH" != "main" ]; then
        echo "Error: Stable releases must be from main branch"
        exit 1
    fi
fi

# Update version in package.json files
echo "Updating version to ${VERSION}..."
npm version "$VERSION" --no-git-tag-version
cd web && npm version "$VERSION" --no-git-tag-version && cd ..
cd electron && npm version "$VERSION" --no-git-tag-version && cd ..

# Update Cargo.toml
sed -i '' "s/^version = \".*\"/version = \"${VERSION}\"/" Cargo.toml

# Build release
echo "Building release..."
make clean
make build-release

# Package for all platforms
echo "Packaging for distribution..."
cd electron
npm run dist:all
cd ..

# Create git tag
echo "Creating git tag..."
git add -A
git commit -m "Release v${VERSION}"
git tag -a "v${VERSION}" -m "Release v${VERSION}"

echo ""
echo "Release v${VERSION} prepared!"
echo ""
echo "Next steps:"
echo "  1. Review the changes"
echo "  2. Push: git push origin main --tags"
echo "  3. GitHub Actions will create the release"
echo ""
echo "Or manually create release:"
echo "  gh release create v${VERSION} electron/out/* --title 'v${VERSION}' --notes-file CHANGELOG.md"
```

### 5. GitHub Actions Release Workflow

Create `.github/workflows/release.yml`:

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build-and-release:
    strategy:
      matrix:
        include:
          - os: macos-latest
            platform: mac
          - os: windows-latest
            platform: win
          - os: ubuntu-latest
            platform: linux

    runs-on: ${{ matrix.os }}

    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Install Rust
        uses: dtolnay/rust-action@stable

      - name: Install dependencies
        run: npm ci

      - name: Build Rust
        run: cargo build --release --workspace

      - name: Build and package
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          CSC_LINK: ${{ secrets.MAC_CERTIFICATE }}
          CSC_KEY_PASSWORD: ${{ secrets.MAC_CERTIFICATE_PASSWORD }}
          APPLE_ID: ${{ secrets.APPLE_ID }}
          APPLE_APP_SPECIFIC_PASSWORD: ${{ secrets.APPLE_APP_PASSWORD }}
          APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
          WIN_CSC_LINK: ${{ secrets.WIN_CERTIFICATE }}
          WIN_CSC_KEY_PASSWORD: ${{ secrets.WIN_CERTIFICATE_PASSWORD }}
        run: |
          cd electron
          npm run dist -- --${{ matrix.platform }} --publish always

  create-release:
    needs: build-and-release
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Create Release Notes
        run: |
          # Extract changelog for this version
          VERSION=${GITHUB_REF#refs/tags/v}
          sed -n "/## \[${VERSION}\]/,/## \[/p" CHANGELOG.md | head -n -1 > release-notes.md

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v1
        with:
          body_path: release-notes.md
          draft: false
          prerelease: ${{ contains(github.ref, 'beta') || contains(github.ref, 'alpha') }}
```

### 6. Self-Hosted Update Server (Alternative)

Create `update-server/server.ts`:

```typescript
/**
 * Self-hosted update server for Tachikoma
 */

import express from 'express';
import path from 'path';
import fs from 'fs';

const app = express();
const PORT = process.env.PORT || 3001;
const RELEASES_DIR = process.env.RELEASES_DIR || './releases';

interface ReleaseInfo {
  version: string;
  releaseDate: string;
  files: {
    url: string;
    sha512: string;
    size: number;
  }[];
}

// Serve release files
app.use('/releases', express.static(RELEASES_DIR));

// Update endpoint for macOS
app.get('/update/darwin/:arch/:version', (req, res) => {
  const { arch, version } = req.params;
  const manifestPath = path.join(RELEASES_DIR, 'latest-mac.yml');

  if (!fs.existsSync(manifestPath)) {
    return res.status(404).json({ error: 'No updates available' });
  }

  const manifest = fs.readFileSync(manifestPath, 'utf-8');
  res.type('text/yaml').send(manifest);
});

// Update endpoint for Windows
app.get('/update/win32/:arch/:version', (req, res) => {
  const { arch, version } = req.params;
  const manifestPath = path.join(RELEASES_DIR, 'latest.yml');

  if (!fs.existsSync(manifestPath)) {
    return res.status(404).json({ error: 'No updates available' });
  }

  const manifest = fs.readFileSync(manifestPath, 'utf-8');
  res.type('text/yaml').send(manifest);
});

// Update endpoint for Linux
app.get('/update/linux/:arch/:version', (req, res) => {
  const { arch, version } = req.params;
  const manifestPath = path.join(RELEASES_DIR, 'latest-linux.yml');

  if (!fs.existsSync(manifestPath)) {
    return res.status(404).json({ error: 'No updates available' });
  }

  const manifest = fs.readFileSync(manifestPath, 'utf-8');
  res.type('text/yaml').send(manifest);
});

// Health check
app.get('/health', (req, res) => {
  res.json({ status: 'ok' });
});

app.listen(PORT, () => {
  console.log(`Update server running on port ${PORT}`);
});
```

### 7. Staging/Beta Channel Configuration

For beta releases, configure in `electron/electron-builder.config.js`:

```javascript
// For beta channel
const channel = process.env.RELEASE_CHANNEL || 'latest';

module.exports = {
  // ... other config

  publish: {
    provider: 'github',
    owner: 'tachikoma',
    repo: 'tachikoma',
    releaseType: channel === 'beta' ? 'prerelease' : 'release',
  },

  // Generate channel-specific manifest
  // latest.yml for stable, beta.yml for beta
};
```

---

## Testing Requirements

1. Release assets upload to GitHub correctly
2. Update manifests are properly generated
3. Checksums match actual files
4. Version comparison works correctly
5. Channel switching works for beta users

---

## Related Specs

- Depends on: [494-electron-packaging.md](494-electron-packaging.md)
- Next: [503-auto-update-client.md](503-auto-update-client.md)
- Related: [167-auto-updates.md](../phase-08-electron/167-auto-updates.md)
