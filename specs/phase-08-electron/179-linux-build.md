# Spec 179: Linux Build

## Phase
8 - Electron Shell

## Spec ID
179

## Status
Planned

## Dependencies
- Spec 175 (Build Configuration)
- Spec 176 (Code Signing)

## Estimated Context
~8%

---

## Objective

Configure and optimize the Linux build process for Tachikoma, including AppImage, DEB, RPM, and Snap package creation with proper desktop integration, package signing, and distribution preparation.

---

## Acceptance Criteria

- [ ] AppImage with automatic updates
- [ ] DEB package for Debian/Ubuntu
- [ ] RPM package for Fedora/RHEL
- [ ] Snap package for universal distribution
- [ ] Flatpak package configuration
- [ ] GPG package signing
- [ ] Desktop file and icons
- [ ] MIME type associations
- [ ] Wayland and X11 support
- [ ] ARM64 Linux support

---

## Implementation Details

### Linux-Specific Electron Builder Config

```typescript
// electron-builder.linux.config.ts
import type { LinuxConfiguration, SnapOptions, DebOptions, AppImageOptions } from 'electron-builder';

export const linuxConfig: LinuxConfiguration = {
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
      target: 'tar.gz',
      arch: ['x64', 'arm64'],
    },
  ],

  // Icons
  icon: 'build/icons',

  // Category
  category: 'Development',

  // Desktop file
  desktop: {
    Name: 'Tachikoma',
    GenericName: 'Development Environment',
    Comment: 'Modern development environment for building applications',
    Type: 'Application',
    Categories: 'Development;IDE;',
    Keywords: 'development;code;editor;ide;programming;',
    StartupNotify: 'true',
    StartupWMClass: 'tachikoma',
    MimeType: 'application/x-tachikoma;x-scheme-handler/tachikoma;',
    Actions: 'new-window;',
  },

  // Synopsis and description
  synopsis: 'Modern development environment',
  description: 'Tachikoma is a modern development environment designed for building amazing applications with a beautiful, intuitive interface.',

  // Maintainer
  maintainer: 'Tachikoma Team <team@tachikoma.io>',
  vendor: 'Tachikoma',

  // File associations
  fileAssociations: [
    {
      ext: 'tachi',
      name: 'Tachikoma Project',
      description: 'Tachikoma Project File',
      mimeType: 'application/x-tachikoma',
      icon: 'file-icon',
    },
  ],

  // MIME types
  mimeTypes: ['application/x-tachikoma', 'x-scheme-handler/tachikoma'],

  // Extra files
  extraFiles: [
    {
      from: 'build/linux',
      to: '.',
      filter: ['**/*'],
    },
  ],

  // Executable name
  executableName: 'tachikoma',

  // Artifact naming
  artifactName: '${productName}-${version}-${arch}.${ext}',
};

export const appImageConfig: AppImageOptions = {
  // AppImage options
  artifactName: '${productName}-${version}-${arch}.${ext}',

  // Desktop integration
  category: 'Development',

  // License
  license: 'LICENSE',

  // Desktop file entry
  desktop: {
    StartupWMClass: 'tachikoma',
  },
};

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

  // Recommends
  recommends: ['libappindicator3-1'],

  // Category
  category: 'Development',

  // Priority
  priority: 'optional',

  // Package name
  packageName: 'tachikoma',

  // Maintainer scripts
  afterInstall: 'build/linux/after-install.sh',
  afterRemove: 'build/linux/after-remove.sh',

  // File category
  fpm: ['--category', 'Development'],
};

export const rpmConfig = {
  // Dependencies
  depends: [
    'gtk3',
    'libnotify',
    'nss',
    'libXScrnSaver',
    'libXtst',
    'xdg-utils',
    'at-spi2-core',
    'libuuid',
    'libsecret',
  ],

  // Category
  category: 'Development',

  // Package name
  packageName: 'tachikoma',

  // Compression
  compression: 'xz',

  // FPM options
  fpm: ['--rpm-os', 'linux'],
};

export const snapConfig: SnapOptions = {
  // Confinement
  confinement: 'strict',

  // Grade
  grade: 'stable',

  // Base
  base: 'core22',

  // Summary
  summary: 'Modern development environment',

  // Plugs (permissions)
  plugs: [
    'desktop',
    'desktop-legacy',
    'home',
    'x11',
    'wayland',
    'unity7',
    'browser-support',
    'network',
    'network-bind',
    'gsettings',
    'opengl',
    'removable-media',
    'password-manager-service',
  ],

  // Slots
  slots: ['tachikoma-dbus'],

  // Build packages
  buildPackages: ['build-essential', 'libsecret-1-dev'],

  // Stage packages
  stagePackages: ['libsecret-1-0'],

  // Apps
  apps: {
    tachikoma: {
      command: 'tachikoma',
      'desktop': 'share/applications/tachikoma.desktop',
      'common-id': 'io.tachikoma.app',
      plugs: [
        'desktop',
        'desktop-legacy',
        'home',
        'x11',
        'wayland',
        'unity7',
        'browser-support',
        'network',
        'opengl',
      ],
    },
  },
};
```

### Desktop Entry File

```ini
# build/linux/tachikoma.desktop
[Desktop Entry]
Name=Tachikoma
GenericName=Development Environment
Comment=Modern development environment for building applications
Exec=tachikoma %F
Icon=tachikoma
Type=Application
Categories=Development;IDE;
Keywords=development;code;editor;ide;programming;
StartupNotify=true
StartupWMClass=tachikoma
MimeType=application/x-tachikoma;x-scheme-handler/tachikoma;
Actions=new-window;

[Desktop Action new-window]
Name=New Window
Exec=tachikoma --new-window
```

### MIME Type Definition

```xml
<!-- build/linux/tachikoma-mime.xml -->
<?xml version="1.0" encoding="UTF-8"?>
<mime-info xmlns="http://www.freedesktop.org/standards/shared-mime-info">
  <mime-type type="application/x-tachikoma">
    <comment>Tachikoma Project</comment>
    <comment xml:lang="en">Tachikoma Project File</comment>
    <icon name="application-x-tachikoma"/>
    <glob pattern="*.tachi"/>
    <glob pattern="*.tachikoma"/>
  </mime-type>
</mime-info>
```

### Post-Install Script

```bash
#!/bin/bash
# build/linux/after-install.sh

set -e

# Update desktop database
if command -v update-desktop-database &> /dev/null; then
    update-desktop-database /usr/share/applications || true
fi

# Update MIME database
if command -v update-mime-database &> /dev/null; then
    update-mime-database /usr/share/mime || true
fi

# Update icon cache
for dir in /usr/share/icons/hicolor /usr/share/icons/Papirus /usr/share/icons/Adwaita; do
    if [ -d "$dir" ] && command -v gtk-update-icon-cache &> /dev/null; then
        gtk-update-icon-cache -f -t "$dir" || true
    fi
done

# Create symlink for CLI access
ln -sf /opt/Tachikoma/tachikoma /usr/local/bin/tachikoma 2>/dev/null || true

# Register MIME types
if [ -f /opt/Tachikoma/resources/tachikoma-mime.xml ]; then
    mkdir -p /usr/share/mime/packages
    cp /opt/Tachikoma/resources/tachikoma-mime.xml /usr/share/mime/packages/
    update-mime-database /usr/share/mime || true
fi

# Set capabilities for sandbox (if needed)
if command -v setcap &> /dev/null; then
    setcap cap_sys_admin+ep /opt/Tachikoma/chrome-sandbox 2>/dev/null || true
fi

echo "Tachikoma has been installed successfully."
```

### Post-Remove Script

```bash
#!/bin/bash
# build/linux/after-remove.sh

set -e

# Remove symlink
rm -f /usr/local/bin/tachikoma 2>/dev/null || true

# Remove MIME type
rm -f /usr/share/mime/packages/tachikoma-mime.xml 2>/dev/null || true

# Update databases
if command -v update-desktop-database &> /dev/null; then
    update-desktop-database /usr/share/applications 2>/dev/null || true
fi

if command -v update-mime-database &> /dev/null; then
    update-mime-database /usr/share/mime 2>/dev/null || true
fi

echo "Tachikoma has been removed."
```

### Flatpak Manifest

```yaml
# build/linux/io.tachikoma.app.yaml
app-id: io.tachikoma.app
runtime: org.freedesktop.Platform
runtime-version: '23.08'
sdk: org.freedesktop.Sdk
base: org.electronjs.Electron2.BaseApp
base-version: '23.08'
command: tachikoma
separate-locales: false

finish-args:
  - --share=ipc
  - --share=network
  - --socket=x11
  - --socket=wayland
  - --socket=pulseaudio
  - --device=dri
  - --filesystem=home
  - --filesystem=xdg-documents
  - --filesystem=xdg-download
  - --filesystem=/tmp
  - --talk-name=org.freedesktop.Notifications
  - --talk-name=org.freedesktop.secrets
  - --talk-name=org.gnome.SessionManager
  - --env=ELECTRON_TRASH=gio

modules:
  - name: tachikoma
    buildsystem: simple
    build-commands:
      - install -Dm755 tachikoma -t /app/bin/
      - install -Dm644 tachikoma.desktop /app/share/applications/io.tachikoma.app.desktop
      - install -Dm644 tachikoma.svg /app/share/icons/hicolor/scalable/apps/io.tachikoma.app.svg
      - install -Dm644 tachikoma-mime.xml /app/share/mime/packages/io.tachikoma.app.xml
    sources:
      - type: archive
        url: https://github.com/tachikoma/releases/download/v${VERSION}/Tachikoma-${VERSION}-linux-x64.tar.gz
        sha256: ${SHA256}
```

### Linux-Specific Features

```typescript
// src/electron/main/linux/index.ts
import { app, nativeTheme } from 'electron';

export function setupLinuxIntegration(): void {
  if (process.platform !== 'linux') {
    return;
  }

  // Set app name for desktop integration
  app.setName('Tachikoma');

  // Set desktop name for grouping
  app.setDesktopName('tachikoma.desktop');

  // Handle dark mode on Linux
  setupLinuxTheme();

  // Handle Wayland-specific setup
  if (isWayland()) {
    setupWayland();
  }
}

function setupLinuxTheme(): void {
  // GTK theme detection
  const gtkTheme = process.env.GTK_THEME || '';
  const preferDark = gtkTheme.toLowerCase().includes('dark') ||
    nativeTheme.shouldUseDarkColors;

  if (preferDark) {
    nativeTheme.themeSource = 'dark';
  }
}

function isWayland(): boolean {
  return process.env.XDG_SESSION_TYPE === 'wayland' ||
    process.env.WAYLAND_DISPLAY !== undefined;
}

function setupWayland(): void {
  // Wayland-specific configurations
  app.commandLine.appendSwitch('enable-features', 'UseOzonePlatform');
  app.commandLine.appendSwitch('ozone-platform', 'wayland');

  // Enable Wayland IME
  app.commandLine.appendSwitch('enable-wayland-ime');
}

// Unity launcher integration
export function setUnityLauncherCount(count: number): void {
  if (process.platform !== 'linux') {
    return;
  }

  try {
    app.setBadgeCount(count);
  } catch {
    // Badge count not supported on this desktop
  }
}

export function setUnityLauncherProgress(progress: number): void {
  if (process.platform !== 'linux') {
    return;
  }

  // Unity launcher progress via DBus
  // Implementation would require dbus bindings
}
```

---

## Testing Requirements

### Linux Build Tests

```typescript
// scripts/test-linux-build.ts
import { execSync } from 'child_process';
import { existsSync } from 'fs';

const results: Array<{ name: string; passed: boolean; message: string }> = [];

function test(name: string, fn: () => boolean | string): void {
  try {
    const result = fn();
    results.push({
      name,
      passed: result === true || typeof result === 'string',
      message: typeof result === 'string' ? result : result ? 'OK' : 'Failed',
    });
  } catch (error: any) {
    results.push({ name, passed: false, message: error.message });
  }
}

// Test AppImage
test('AppImage exists', () => existsSync('release/Tachikoma-x86_64.AppImage'));

test('AppImage executable', () => {
  const output = execSync('file release/Tachikoma-x86_64.AppImage', { encoding: 'utf-8' });
  return output.includes('executable');
});

// Test DEB
test('DEB package exists', () => existsSync('release/tachikoma_amd64.deb'));

test('DEB package valid', () => {
  execSync('dpkg-deb --info release/tachikoma_amd64.deb', { stdio: 'pipe' });
  return true;
});

// Test RPM
test('RPM package exists', () => existsSync('release/tachikoma.x86_64.rpm'));

test('RPM package valid', () => {
  execSync('rpm -qpi release/tachikoma.x86_64.rpm', { stdio: 'pipe' });
  return true;
});

// Test desktop file
test('Desktop file valid', () => {
  execSync('desktop-file-validate build/linux/tachikoma.desktop', { stdio: 'pipe' });
  return true;
});

console.log('\nLinux Build Test Results:');
results.forEach((r) => console.log(`${r.passed ? '✓' : '✗'} ${r.name}: ${r.message}`));
```

---

## Related Specs

- Spec 175: Build Configuration
- Spec 176: Code Signing
- Spec 167: Auto Updates
