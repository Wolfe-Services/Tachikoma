# 501 - Linux Debian Package

**Phase:** 23 - Build & Distribution
**Spec ID:** 501
**Status:** Planned
**Dependencies:** 494-electron-packaging
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Configure Debian package (.deb) creation for installation on Debian, Ubuntu, and derivative distributions with proper dependency management and system integration.

---

## Acceptance Criteria

- [ ] .deb package created with correct metadata
- [ ] Dependencies properly declared
- [ ] Post-install scripts configure system integration
- [ ] Desktop entry and icons installed
- [ ] File associations registered
- [ ] APT repository compatible

---

## Implementation Details

### 1. Debian Package Configuration

Update `electron/electron-builder.config.js`:

```javascript
// Debian package configuration
deb: {
  artifactName: '${productName}_${version}_${arch}.${ext}',

  // Package metadata
  packageName: 'tachikoma',
  category: 'devel',
  priority: 'optional',
  section: 'devel',

  // Dependencies
  depends: [
    'libgtk-3-0',
    'libnotify4',
    'libnss3',
    'libxss1',
    'libxtst6',
    'xdg-utils',
    'libatspi2.0-0',
    'libuuid1',
    'libsecret-1-0',
  ],

  // Recommended packages
  recommends: [
    'git',
  ],

  // Package scripts
  afterInstall: 'build/deb-scripts/postinst',
  afterRemove: 'build/deb-scripts/postrm',

  // Desktop file
  desktop: {
    Name: 'Tachikoma',
    GenericName: 'AI Development Environment',
    Comment: 'AI-powered autonomous development tool',
    Exec: '/opt/Tachikoma/tachikoma %U',
    Icon: 'tachikoma',
    Type: 'Application',
    Categories: 'Development;IDE;',
    MimeType: 'application/x-tachikoma-spec;x-scheme-handler/tachikoma;',
    StartupNotify: 'true',
    StartupWMClass: 'tachikoma',
  },

  // Package maintainer
  maintainer: 'Tachikoma Team <support@tachikoma.dev>',

  // Vendor
  vendor: 'Tachikoma',

  // Homepage
  homepage: 'https://tachikoma.dev',

  // Compression
  compression: 'xz',

  // Fpm options
  fpm: [
    '--deb-priority', 'optional',
  ],
},
```

### 2. Post-Install Script

Create `electron/build/deb-scripts/postinst`:

```bash
#!/bin/bash
set -e

# Update desktop database
if command -v update-desktop-database &> /dev/null; then
    update-desktop-database -q /usr/share/applications || true
fi

# Update icon cache
if command -v gtk-update-icon-cache &> /dev/null; then
    gtk-update-icon-cache -f -t /usr/share/icons/hicolor || true
fi

# Update MIME database
if command -v update-mime-database &> /dev/null; then
    update-mime-database /usr/share/mime || true
fi

# Register protocol handler
if command -v xdg-mime &> /dev/null; then
    xdg-mime default tachikoma.desktop x-scheme-handler/tachikoma || true
fi

# Create symlink in /usr/bin
ln -sf /opt/Tachikoma/tachikoma /usr/bin/tachikoma || true

# Set capabilities for sandbox (if needed)
# setcap cap_net_bind_service=+ep /opt/Tachikoma/tachikoma || true

echo "Tachikoma installed successfully!"
exit 0
```

### 3. Post-Remove Script

Create `electron/build/deb-scripts/postrm`:

```bash
#!/bin/bash
set -e

case "$1" in
    purge|remove)
        # Remove symlink
        rm -f /usr/bin/tachikoma || true

        # Update databases
        if command -v update-desktop-database &> /dev/null; then
            update-desktop-database -q /usr/share/applications || true
        fi

        if command -v gtk-update-icon-cache &> /dev/null; then
            gtk-update-icon-cache -f -t /usr/share/icons/hicolor || true
        fi

        if command -v update-mime-database &> /dev/null; then
            update-mime-database /usr/share/mime || true
        fi

        # Remove config directory on purge
        if [ "$1" = "purge" ]; then
            rm -rf /etc/tachikoma || true
        fi
        ;;
esac

exit 0
```

### 4. Icon Installation

Icons should be placed in standard XDG directories:

```
/usr/share/icons/hicolor/16x16/apps/tachikoma.png
/usr/share/icons/hicolor/32x32/apps/tachikoma.png
/usr/share/icons/hicolor/48x48/apps/tachikoma.png
/usr/share/icons/hicolor/64x64/apps/tachikoma.png
/usr/share/icons/hicolor/128x128/apps/tachikoma.png
/usr/share/icons/hicolor/256x256/apps/tachikoma.png
/usr/share/icons/hicolor/512x512/apps/tachikoma.png
/usr/share/icons/hicolor/scalable/apps/tachikoma.svg
```

### 5. MIME Type Installation

Create `electron/build/mime/tachikoma.xml`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<mime-info xmlns="http://www.freedesktop.org/standards/shared-mime-info">
  <mime-type type="application/x-tachikoma-spec">
    <comment>Tachikoma Specification</comment>
    <comment xml:lang="en">Tachikoma Specification File</comment>
    <glob pattern="*.tspec"/>
    <icon name="application-x-tachikoma-spec"/>
  </mime-type>
</mime-info>
```

### 6. Build Script

Create `electron/scripts/build-deb.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

ARCH="${1:-amd64}"
VERSION=$(node -p "require('./package.json').version")

echo "Building Debian package ($ARCH)..."

# Map architecture names
case "$ARCH" in
    x64|x86_64|amd64)
        ELECTRON_ARCH="x64"
        DEB_ARCH="amd64"
        ;;
    arm64|aarch64)
        ELECTRON_ARCH="arm64"
        DEB_ARCH="arm64"
        ;;
    *)
        echo "Unknown architecture: $ARCH"
        exit 1
        ;;
esac

# Build package
npx electron-builder --linux deb --arch "$ELECTRON_ARCH"

# Verify package
DEB_FILE="out/tachikoma_${VERSION}_${DEB_ARCH}.deb"

if [ -f "$DEB_FILE" ]; then
    echo "Package created: $DEB_FILE"
    ls -lh "$DEB_FILE"

    # Show package info
    echo ""
    echo "Package info:"
    dpkg-deb --info "$DEB_FILE"

    # Show package contents
    echo ""
    echo "Package contents (first 20 files):"
    dpkg-deb --contents "$DEB_FILE" | head -20

    # Lint package
    if command -v lintian &> /dev/null; then
        echo ""
        echo "Linting package..."
        lintian "$DEB_FILE" || true
    fi
else
    echo "Error: Package not found at $DEB_FILE"
    exit 1
fi

echo "Debian package build complete!"
```

### 7. Test Installation Script

Create `electron/scripts/test-deb-install.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

DEB_FILE="${1:?DEB file path required}"

echo "Testing Debian package installation..."

# Create test container
CONTAINER_ID=$(docker run -d --rm ubuntu:22.04 sleep 3600)

cleanup() {
    docker stop "$CONTAINER_ID" &> /dev/null || true
}
trap cleanup EXIT

# Copy package to container
docker cp "$DEB_FILE" "$CONTAINER_ID:/tmp/tachikoma.deb"

# Install package and dependencies
docker exec "$CONTAINER_ID" bash -c "
    apt-get update
    apt-get install -y /tmp/tachikoma.deb
"

# Verify installation
docker exec "$CONTAINER_ID" bash -c "
    # Check binary exists
    ls -la /opt/Tachikoma/tachikoma

    # Check symlink
    ls -la /usr/bin/tachikoma

    # Check desktop file
    cat /usr/share/applications/tachikoma.desktop

    # Check version
    /opt/Tachikoma/tachikoma --version || echo 'Version check requires display'
"

# Test uninstall
docker exec "$CONTAINER_ID" bash -c "
    apt-get remove -y tachikoma
    apt-get purge -y tachikoma
"

echo "Debian package test complete!"
```

### 8. APT Repository Setup (Optional)

For hosting your own APT repository:

```bash
# Generate GPG key for signing
gpg --full-generate-key

# Export public key
gpg --armor --export your@email.com > tachikoma.gpg.key

# Sign package
dpkg-sig --sign builder -k your@email.com tachikoma_*.deb

# Create repository structure
mkdir -p repo/pool/main repo/dists/stable/main/binary-amd64

# Copy packages
cp tachikoma_*.deb repo/pool/main/

# Generate Packages file
cd repo
dpkg-scanpackages pool/main > dists/stable/main/binary-amd64/Packages
gzip -k dists/stable/main/binary-amd64/Packages

# Generate Release file
apt-ftparchive release dists/stable > dists/stable/Release

# Sign Release file
gpg --armor --sign --detach-sign -o dists/stable/Release.gpg dists/stable/Release
gpg --armor --clearsign -o dists/stable/InRelease dists/stable/Release
```

---

## Testing Requirements

1. Package installs on Ubuntu 20.04, 22.04, 24.04
2. Package installs on Debian 11, 12
3. Dependencies are correctly resolved
4. Desktop entry appears in application menu
5. File associations work correctly

---

## Related Specs

- Depends on: [494-electron-packaging.md](494-electron-packaging.md)
- Next: [502-auto-update-server.md](502-auto-update-server.md)
- Related: [500-linux-appimage.md](500-linux-appimage.md)
