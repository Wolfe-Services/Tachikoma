# Spec 501: Linux Packages

## Phase
23 - Build/Package System

## Spec ID
501

## Status
Planned

## Dependencies
- Spec 494 (Electron Packaging)
- Spec 498 (Code Signing)
- Spec 493 (Rust Compilation)

## Estimated Context
~10%

---

## Objective

Implement Linux-specific packaging including AppImage, DEB, RPM, Snap, and Flatpak formats. Ensure proper desktop integration, repository hosting, and package signing for distribution through various Linux package managers.

---

## Acceptance Criteria

- [ ] AppImage universal package
- [ ] DEB package for Debian/Ubuntu
- [ ] RPM package for Fedora/RHEL
- [ ] Snap package for Ubuntu Store
- [ ] Flatpak package for Flathub
- [ ] Desktop file and icon installation
- [ ] MIME type and file associations
- [ ] Auto-update via AppImageUpdate
- [ ] Package signing with GPG
- [ ] Repository configuration files

---

## Implementation Details

### Linux Build Configuration (electron/build/linux.config.ts)

```typescript
// electron/build/linux.config.ts
import type { LinuxConfiguration } from 'electron-builder';

export const linuxConfig: LinuxConfiguration = {
  // Target formats
  target: [
    {
      target: 'AppImage',
      arch: ['x64', 'arm64'],
    },
    {
      target: 'deb',
      arch: ['x64', 'arm64'],
    },
    {
      target: 'rpm',
      arch: ['x64'],
    },
    {
      target: 'snap',
      arch: ['x64'],
    },
    {
      target: 'flatpak',
      arch: ['x64'],
    },
    {
      target: 'tar.gz',
      arch: ['x64', 'arm64'],
    },
  ],

  // Application category
  category: 'Development',

  // Icon
  icon: 'resources/icons',

  // Synopsis (short description)
  synopsis: 'AI-Powered Development Assistant',

  // Description
  description:
    'Tachikoma is an AI-powered development assistant that helps you write, review, and improve code through natural language interactions.',

  // Desktop file
  desktop: {
    Name: 'Tachikoma',
    Comment: 'AI-Powered Development Assistant',
    Categories: 'Development;IDE;TextEditor;',
    Keywords: 'ai;coding;development;assistant;llm;',
    StartupWMClass: 'tachikoma',
    MimeType: 'x-scheme-handler/tachikoma;application/x-tachikoma-mission;application/x-tachikoma-spec;',
    Terminal: 'false',
  },

  // MIME types
  mimeTypes: [
    'x-scheme-handler/tachikoma',
    'application/x-tachikoma-mission',
    'application/x-tachikoma-spec',
  ],

  // File associations
  fileAssociations: [
    {
      ext: 'tkmission',
      name: 'Tachikoma Mission',
      mimeType: 'application/x-tachikoma-mission',
      icon: 'file-mission',
    },
    {
      ext: 'tkspec',
      name: 'Tachikoma Spec',
      mimeType: 'application/x-tachikoma-spec',
      icon: 'file-spec',
    },
  ],

  // Extra resources
  extraResources: [
    {
      from: '../target/x86_64-unknown-linux-gnu/release/tachikoma',
      to: 'bin/tachikoma',
    },
  ],

  // Executable name
  executableName: 'tachikoma',

  // Maintainer
  maintainer: 'Tachikoma Team <team@tachikoma.io>',

  // Vendor
  vendor: 'Tachikoma',

  // Publish to Snapcraft
  publish: {
    provider: 'snapStore',
    channels: ['stable'],
  },
};
```

### DEB Configuration (electron/build/deb.config.ts)

```typescript
// electron/build/deb.config.ts
import type { DebOptions } from 'electron-builder';

export const debConfig: DebOptions = {
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
  recommends: ['libappindicator3-1', 'gnome-keyring'],

  // Package category
  category: 'devel',

  // Priority
  priority: 'optional',

  // Section
  section: 'devel',

  // Post-install script
  afterInstall: 'resources/linux/after-install.sh',

  // Pre-remove script
  afterRemove: 'resources/linux/after-remove.sh',

  // File associations
  fpm: [
    '--deb-no-default-config-files',
  ],
};
```

### RPM Configuration (electron/build/rpm.config.ts)

```typescript
// electron/build/rpm.config.ts
import type { RpmOptions } from 'electron-builder';

export const rpmConfig: RpmOptions = {
  // Dependencies
  depends: [
    'gtk3',
    'libnotify',
    'nss',
    'libXScrnSaver',
    'libXtst',
    'xdg-utils',
    'at-spi2-atk',
    'libuuid',
    'libsecret',
  ],

  // Package category
  category: 'Development/Tools',

  // Post-install script
  afterInstall: 'resources/linux/after-install.sh',

  // Pre-remove script
  afterRemove: 'resources/linux/after-remove.sh',

  // Additional fpm arguments
  fpm: [
    '--rpm-os', 'linux',
  ],
};
```

### Snap Configuration (electron/build/snap.config.ts)

```typescript
// electron/build/snap.config.ts
import type { SnapOptions } from 'electron-builder';

export const snapConfig: SnapOptions = {
  // Confinement level
  confinement: 'classic', // 'strict', 'devmode', or 'classic'

  // Grade
  grade: 'stable', // 'stable' or 'devel'

  // Summary
  summary: 'AI-Powered Development Assistant',

  // Plugs (permissions)
  plugs: [
    'default',
    'removable-media',
    'home',
    'network',
    'network-bind',
    'desktop',
    'desktop-legacy',
    'unity7',
    'wayland',
    'x11',
    'browser-support',
    'password-manager-service',
  ],

  // Environment variables
  environment: {
    TMPDIR: '$XDG_RUNTIME_DIR',
    DISABLE_WAYLAND: '1', // Force X11 for better compatibility
  },

  // Build packages
  buildPackages: ['build-essential', 'libx11-dev', 'libxkbfile-dev'],

  // Stage packages
  stagePackages: [
    'libnspr4',
    'libnss3',
    'libxss1',
    'libappindicator3-1',
    'libsecret-1-0',
    'libnotify4',
  ],

  // Auto connect
  autoStart: false,

  // Allow interfaces
  allowNativeWayland: false,
};
```

### Flatpak Configuration (electron/build/flatpak.config.ts)

```typescript
// electron/build/flatpak.config.ts
import type { FlatpakOptions } from 'electron-builder';

export const flatpakConfig: FlatpakOptions = {
  // Runtime
  runtime: 'org.freedesktop.Platform',
  runtimeVersion: '23.08',

  // SDK
  sdk: 'org.freedesktop.Sdk',

  // Base
  base: 'org.electronjs.Electron2.BaseApp',
  baseVersion: '23.08',

  // Finish args (sandbox permissions)
  finishArgs: [
    '--share=ipc',
    '--share=network',
    '--socket=x11',
    '--socket=wayland',
    '--socket=pulseaudio',
    '--device=dri',
    '--filesystem=home',
    '--filesystem=xdg-run/keyring',
    '--talk-name=org.freedesktop.Notifications',
    '--talk-name=org.freedesktop.secrets',
    '--talk-name=org.kde.StatusNotifierWatcher',
    '--talk-name=com.canonical.AppMenu.Registrar',
    '--talk-name=com.canonical.indicator.application',
    '--env=XCURSOR_PATH=/run/host/user-share/icons:/run/host/share/icons',
  ],

  // Modules (additional dependencies)
  modules: [],
};
```

### AppImage Configuration (electron/build/appimage.config.ts)

```typescript
// electron/build/appimage.config.ts
import type { AppImageOptions } from 'electron-builder';

export const appImageConfig: AppImageOptions = {
  // License file
  license: '../LICENSE',

  // Desktop integration
  // systemIntegration: 'ask', // Deprecated in newer AppImage

  // Category
  category: 'Development',
};
```

### Post-Install Script (resources/linux/after-install.sh)

```bash
#!/bin/bash
# resources/linux/after-install.sh
# Post-installation script for Tachikoma

set -e

# Update desktop database
if command -v update-desktop-database &> /dev/null; then
    update-desktop-database -q /usr/share/applications || true
fi

# Update icon cache
if command -v gtk-update-icon-cache &> /dev/null; then
    gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor || true
fi

# Update MIME database
if command -v update-mime-database &> /dev/null; then
    update-mime-database /usr/share/mime || true
fi

# Register URL handler
if command -v xdg-mime &> /dev/null; then
    xdg-mime default tachikoma.desktop x-scheme-handler/tachikoma || true
fi

# Create symbolic link in PATH (optional)
# ln -sf /opt/Tachikoma/tachikoma /usr/local/bin/tachikoma || true

echo "Tachikoma installation completed successfully."
```

### Post-Remove Script (resources/linux/after-remove.sh)

```bash
#!/bin/bash
# resources/linux/after-remove.sh
# Post-removal script for Tachikoma

set -e

# Update desktop database
if command -v update-desktop-database &> /dev/null; then
    update-desktop-database -q /usr/share/applications || true
fi

# Update icon cache
if command -v gtk-update-icon-cache &> /dev/null; then
    gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor || true
fi

# Update MIME database
if command -v update-mime-database &> /dev/null; then
    update-mime-database /usr/share/mime || true
fi

# Remove symbolic link
# rm -f /usr/local/bin/tachikoma || true

# Remove user data (optional - ask user)
# rm -rf "$HOME/.config/Tachikoma" || true
# rm -rf "$HOME/.local/share/Tachikoma" || true

echo "Tachikoma has been removed."
```

### Desktop File (resources/linux/tachikoma.desktop)

```ini
[Desktop Entry]
Name=Tachikoma
Comment=AI-Powered Development Assistant
Exec=/opt/Tachikoma/tachikoma %U
Terminal=false
Type=Application
Icon=tachikoma
Categories=Development;IDE;TextEditor;
Keywords=ai;coding;development;assistant;llm;
StartupWMClass=tachikoma
MimeType=x-scheme-handler/tachikoma;application/x-tachikoma-mission;application/x-tachikoma-spec;
Actions=new-window;

[Desktop Action new-window]
Name=New Window
Exec=/opt/Tachikoma/tachikoma --new-window
```

### MIME Types (resources/linux/tachikoma-mime.xml)

```xml
<?xml version="1.0" encoding="UTF-8"?>
<mime-info xmlns="http://www.freedesktop.org/standards/shared-mime-info">
  <mime-type type="application/x-tachikoma-mission">
    <comment>Tachikoma Mission File</comment>
    <glob pattern="*.tkmission"/>
    <icon name="application-x-tachikoma-mission"/>
  </mime-type>

  <mime-type type="application/x-tachikoma-spec">
    <comment>Tachikoma Specification File</comment>
    <glob pattern="*.tkspec"/>
    <icon name="application-x-tachikoma-spec"/>
  </mime-type>
</mime-info>
```

### Repository Setup Script (scripts/linux/setup-repo.sh)

```bash
#!/bin/bash
# scripts/linux/setup-repo.sh
# Script to generate APT/YUM repository metadata

set -euo pipefail

REPO_DIR="${1:-.}"
GPG_KEY_ID="${GPG_KEY_ID:-}"

echo "Setting up Linux package repository in $REPO_DIR"

# Create directory structure
mkdir -p "$REPO_DIR/deb/pool/main"
mkdir -p "$REPO_DIR/deb/dists/stable/main/binary-amd64"
mkdir -p "$REPO_DIR/deb/dists/stable/main/binary-arm64"
mkdir -p "$REPO_DIR/rpm"

# Function to generate APT repository
generate_apt_repo() {
    echo "Generating APT repository..."

    cd "$REPO_DIR/deb"

    # Generate Packages file
    for arch in amd64 arm64; do
        dpkg-scanpackages --arch "$arch" pool/main > "dists/stable/main/binary-$arch/Packages"
        gzip -9c "dists/stable/main/binary-$arch/Packages" > "dists/stable/main/binary-$arch/Packages.gz"
    done

    # Generate Release file
    cat > dists/stable/Release << EOF
Origin: Tachikoma
Label: Tachikoma
Suite: stable
Codename: stable
Architectures: amd64 arm64
Components: main
Description: Tachikoma APT Repository
EOF

    # Add checksums
    {
        echo "MD5Sum:"
        find dists/stable/main -type f -name "Packages*" -exec md5sum {} \; | sed 's|dists/stable/||'
        echo "SHA256:"
        find dists/stable/main -type f -name "Packages*" -exec sha256sum {} \; | sed 's|dists/stable/||'
    } >> dists/stable/Release

    # Sign Release file
    if [ -n "$GPG_KEY_ID" ]; then
        gpg --default-key "$GPG_KEY_ID" --armor --detach-sign -o dists/stable/Release.gpg dists/stable/Release
        gpg --default-key "$GPG_KEY_ID" --armor --clearsign -o dists/stable/InRelease dists/stable/Release
    fi

    cd -
}

# Function to generate YUM repository
generate_yum_repo() {
    echo "Generating YUM repository..."

    cd "$REPO_DIR/rpm"

    # Generate repository metadata
    createrepo_c .

    # Sign repository metadata
    if [ -n "$GPG_KEY_ID" ]; then
        gpg --default-key "$GPG_KEY_ID" --armor --detach-sign repodata/repomd.xml
    fi

    cd -
}

# Generate repositories
generate_apt_repo
generate_yum_repo

echo "Repository setup complete!"

# Generate repo configuration files
cat > "$REPO_DIR/tachikoma.list" << 'EOF'
# /etc/apt/sources.list.d/tachikoma.list
deb [signed-by=/usr/share/keyrings/tachikoma-archive-keyring.gpg] https://packages.tachikoma.io/deb stable main
EOF

cat > "$REPO_DIR/tachikoma.repo" << 'EOF'
# /etc/yum.repos.d/tachikoma.repo
[tachikoma]
name=Tachikoma
baseurl=https://packages.tachikoma.io/rpm
enabled=1
gpgcheck=1
gpgkey=https://packages.tachikoma.io/gpg-key.asc
EOF

echo "Repository configuration files generated."
```

---

## Testing Requirements

### Unit Tests

```typescript
// electron/build/__tests__/linux.config.test.ts
import { describe, it, expect } from 'vitest';
import { linuxConfig, debConfig, rpmConfig, snapConfig } from '../linux.config';

describe('Linux Build Configuration', () => {
  it('should have valid target configuration', () => {
    expect(linuxConfig.target).toBeDefined();
    expect(Array.isArray(linuxConfig.target)).toBe(true);
  });

  it('should have AppImage target', () => {
    const appImageTarget = linuxConfig.target?.find(
      (t: any) => typeof t === 'object' && t.target === 'AppImage'
    );
    expect(appImageTarget).toBeDefined();
  });

  it('should have desktop entry configured', () => {
    expect(linuxConfig.desktop).toBeDefined();
    expect(linuxConfig.desktop?.Name).toBe('Tachikoma');
    expect(linuxConfig.desktop?.Categories).toContain('Development');
  });

  it('should have MIME types configured', () => {
    expect(linuxConfig.mimeTypes).toBeDefined();
    expect(linuxConfig.mimeTypes?.length).toBeGreaterThan(0);
  });
});

describe('DEB Configuration', () => {
  it('should have dependencies', () => {
    expect(debConfig.depends).toBeDefined();
    expect(debConfig.depends?.length).toBeGreaterThan(0);
  });

  it('should have post-install script', () => {
    expect(debConfig.afterInstall).toBeDefined();
  });
});

describe('Snap Configuration', () => {
  it('should have confinement level', () => {
    expect(snapConfig.confinement).toBeDefined();
    expect(['strict', 'classic', 'devmode']).toContain(snapConfig.confinement);
  });

  it('should have plugs defined', () => {
    expect(snapConfig.plugs).toBeDefined();
    expect(snapConfig.plugs?.length).toBeGreaterThan(0);
  });
});
```

### Integration Tests

```typescript
// electron/build/__tests__/linux.integration.test.ts
import { describe, it, expect } from 'vitest';
import * as fs from 'fs';
import * as path from 'path';

describe('Linux Build Integration', () => {
  it('should have icon files', () => {
    const iconsDir = path.join(__dirname, '..', '..', 'resources', 'icons');

    if (process.platform === 'linux') {
      expect(fs.existsSync(iconsDir)).toBe(true);

      // Check for various sizes
      const sizes = ['16x16', '32x32', '48x48', '128x128', '256x256', '512x512'];
      for (const size of sizes) {
        const iconPath = path.join(iconsDir, size + '.png');
        // Icons should exist if directory exists
        if (fs.existsSync(iconsDir)) {
          expect(fs.existsSync(iconPath)).toBe(true);
        }
      }
    }
  });

  it('should have desktop file', () => {
    const desktopPath = path.join(
      __dirname,
      '..',
      '..',
      'resources',
      'linux',
      'tachikoma.desktop'
    );
    expect(fs.existsSync(desktopPath)).toBe(true);
  });

  it('should have post-install scripts', () => {
    const afterInstallPath = path.join(
      __dirname,
      '..',
      '..',
      'resources',
      'linux',
      'after-install.sh'
    );
    expect(fs.existsSync(afterInstallPath)).toBe(true);
  });
});
```

---

## Related Specs

- Spec 494: Electron Packaging
- Spec 498: Code Signing
- Spec 493: Rust Compilation
- Spec 503: Release Workflow
