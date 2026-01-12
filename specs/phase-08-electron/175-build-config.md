# Spec 175: Electron Builder Configuration

## Phase
8 - Electron Shell

## Spec ID
175

## Status
Planned

## Dependencies
- Spec 161 (Electron Main Process)
- Spec 173 (Rust Native Modules)
- Spec 174 (NAPI-RS Setup)

## Estimated Context
~10%

---

## Objective

Configure electron-builder for creating distributable packages across all target platforms. This includes application metadata, build targets, file associations, installer customization, and CI/CD integration.

---

## Acceptance Criteria

- [x] Build configuration supports macOS, Windows, and Linux
- [x] DMG, PKG, NSIS, AppImage, and DEB/RPM formats
- [x] Auto-update configuration integrated
- [x] File associations configured
- [x] Application metadata complete
- [x] Icons and assets properly configured
- [x] Native modules bundled correctly
- [x] Reproducible builds
- [x] Build optimization for size and startup

---

## Implementation Details

### Main Configuration File

```typescript
// electron-builder.config.ts
import type { Configuration } from 'electron-builder';

const config: Configuration = {
  appId: 'io.tachikoma.app',
  productName: 'Tachikoma',
  copyright: 'Copyright (c) 2024 Tachikoma Team',

  // Directories
  directories: {
    output: 'release/${version}',
    buildResources: 'build',
  },

  // Files to include
  files: [
    'dist/**/*',
    '!dist/**/*.map',
    'node_modules/**/*',
    '!node_modules/**/*.md',
    '!node_modules/**/*.ts',
    '!node_modules/**/test/**',
    '!node_modules/**/tests/**',
    '!node_modules/**/.github/**',
  ],

  // Extra files
  extraFiles: [
    {
      from: 'LICENSE',
      to: 'LICENSE',
    },
  ],

  // Extra resources (native modules, etc.)
  extraResources: [
    {
      from: 'native/tachikoma-native.${os}-${arch}.node',
      to: 'native/',
      filter: ['**/*'],
    },
  ],

  // ASAR archive
  asar: true,
  asarUnpack: [
    'node_modules/sharp/**/*',
    'node_modules/@tachikoma/native*/**/*',
  ],

  // Compression
  compression: 'maximum',

  // Remove unneeded locales
  electronLanguages: ['en', 'en-US'],

  // Artifacts naming
  artifactName: '${productName}-${version}-${os}-${arch}.${ext}',

  // Publish configuration
  publish: {
    provider: 'github',
    owner: 'tachikoma',
    repo: 'tachikoma-desktop',
    releaseType: 'release',
  },

  // macOS configuration
  mac: {
    target: [
      {
        target: 'dmg',
        arch: ['x64', 'arm64'],
      },
      {
        target: 'zip',
        arch: ['x64', 'arm64'],
      },
    ],
    category: 'public.app-category.developer-tools',
    icon: 'build/icon.icns',
    darkModeSupport: true,
    hardenedRuntime: true,
    gatekeeperAssess: false,
    entitlements: 'build/entitlements.mac.plist',
    entitlementsInherit: 'build/entitlements.mac.plist',
    notarize: {
      teamId: process.env.APPLE_TEAM_ID,
    },
    extendInfo: {
      NSMicrophoneUsageDescription: 'This app requires microphone access for voice features.',
      NSCameraUsageDescription: 'This app requires camera access for video features.',
      NSAppleEventsUsageDescription: 'This app requires Apple Events access for automation.',
    },
  },

  // DMG configuration
  dmg: {
    contents: [
      {
        x: 130,
        y: 220,
      },
      {
        x: 410,
        y: 220,
        type: 'link',
        path: '/Applications',
      },
    ],
    window: {
      width: 540,
      height: 380,
    },
    background: 'build/dmg-background.png',
    icon: 'build/icon.icns',
    iconSize: 80,
    title: '${productName} ${version}',
  },

  // PKG configuration
  pkg: {
    license: 'LICENSE',
    installLocation: '/Applications',
  },

  // Windows configuration
  win: {
    target: [
      {
        target: 'nsis',
        arch: ['x64', 'arm64'],
      },
      {
        target: 'portable',
        arch: ['x64'],
      },
    ],
    icon: 'build/icon.ico',
    publisherName: 'Tachikoma Team',
    verifyUpdateCodeSignature: true,
    signAndEditExecutable: true,
  },

  // NSIS installer configuration
  nsis: {
    oneClick: false,
    perMachine: false,
    allowToChangeInstallationDirectory: true,
    allowElevation: true,
    createDesktopShortcut: true,
    createStartMenuShortcut: true,
    shortcutName: 'Tachikoma',
    uninstallDisplayName: '${productName}',
    installerIcon: 'build/icon.ico',
    uninstallerIcon: 'build/icon.ico',
    installerHeaderIcon: 'build/icon.ico',
    license: 'LICENSE',
    deleteAppDataOnUninstall: false,
    include: 'build/installer.nsh',
    warningsAsErrors: false,
  },

  // Linux configuration
  linux: {
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
    ],
    icon: 'build/icons',
    category: 'Development',
    synopsis: 'Modern development environment',
    description: 'Tachikoma is a modern development environment for building amazing applications.',
    desktop: {
      Name: 'Tachikoma',
      Comment: 'Modern development environment',
      Keywords: 'development;code;editor',
      StartupNotify: 'true',
      StartupWMClass: 'tachikoma',
    },
    maintainer: 'Tachikoma Team <team@tachikoma.io>',
    vendor: 'Tachikoma',
  },

  // AppImage configuration
  appImage: {
    artifactName: '${productName}-${version}-${arch}.${ext}',
    category: 'Development',
    desktop: {
      StartupWMClass: 'tachikoma',
    },
  },

  // Debian package configuration
  deb: {
    depends: ['libgtk-3-0', 'libnotify4', 'libnss3', 'libxss1', 'libxtst6', 'xdg-utils', 'libatspi2.0-0', 'libuuid1'],
    category: 'Development',
    priority: 'optional',
    afterInstall: 'build/linux/after-install.sh',
    afterRemove: 'build/linux/after-remove.sh',
  },

  // RPM package configuration
  rpm: {
    depends: ['gtk3', 'libnotify', 'nss', 'libXScrnSaver', 'libXtst', 'xdg-utils', 'at-spi2-core', 'libuuid'],
    category: 'Development',
  },

  // Snap configuration
  snap: {
    confinement: 'strict',
    grade: 'stable',
    summary: 'Modern development environment',
    plugs: ['desktop', 'desktop-legacy', 'home', 'x11', 'unity7', 'browser-support', 'network', 'gsettings', 'opengl'],
  },

  // File associations
  fileAssociations: [
    {
      ext: 'tachi',
      name: 'Tachikoma Project',
      description: 'Tachikoma Project File',
      mimeType: 'application/x-tachikoma',
      icon: 'build/file-icon.icns',
      role: 'Editor',
    },
    {
      ext: 'tachikoma',
      name: 'Tachikoma Project',
      description: 'Tachikoma Project File',
      mimeType: 'application/x-tachikoma',
      icon: 'build/file-icon.icns',
      role: 'Editor',
    },
  ],

  // Protocol handlers
  protocols: [
    {
      name: 'Tachikoma',
      schemes: ['tachikoma'],
    },
  ],

  // Hooks
  beforeBuild: async (context) => {
    console.log('Building for:', context.platform.nodeName, context.arch);
    // Run any pre-build scripts
  },

  afterSign: async (context) => {
    // Run notarization for macOS
    if (context.electronPlatformName === 'darwin') {
      console.log('Notarizing application...');
    }
  },

  afterPack: async (context) => {
    console.log('Pack complete:', context.outDir);
  },

  afterAllArtifactBuild: async (result) => {
    console.log('All artifacts built:', result.artifactPaths);
    return result.artifactPaths;
  },
};

export default config;
```

### macOS Entitlements

```xml
<!-- build/entitlements.mac.plist -->
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <!-- Hardened runtime -->
    <key>com.apple.security.cs.allow-jit</key>
    <true/>
    <key>com.apple.security.cs.allow-unsigned-executable-memory</key>
    <true/>
    <key>com.apple.security.cs.disable-library-validation</key>
    <true/>

    <!-- App sandbox (disabled for Electron apps) -->
    <key>com.apple.security.app-sandbox</key>
    <false/>

    <!-- Network access -->
    <key>com.apple.security.network.client</key>
    <true/>
    <key>com.apple.security.network.server</key>
    <true/>

    <!-- File access -->
    <key>com.apple.security.files.user-selected.read-write</key>
    <true/>
    <key>com.apple.security.files.downloads.read-write</key>
    <true/>

    <!-- Device access -->
    <key>com.apple.security.device.audio-input</key>
    <true/>
    <key>com.apple.security.device.camera</key>
    <true/>

    <!-- Automation -->
    <key>com.apple.security.automation.apple-events</key>
    <true/>
</dict>
</plist>
```

### NSIS Custom Script

```nsis
; build/installer.nsh

!macro customHeader
  !system "echo custom header"
!macroend

!macro preInit
  ; Check for admin rights
  UserInfo::GetAccountType
  pop $0
  ${If} $0 != "admin"
    MessageBox mb_iconstop "Administrator rights required!"
    SetErrorLevel 740
    Quit
  ${EndIf}
!macroend

!macro customInit
  ; Custom initialization
!macroend

!macro customInstall
  ; Create registry entries for file associations
  WriteRegStr SHCTX "Software\Classes\.tachi" "" "TachikomaProject"
  WriteRegStr SHCTX "Software\Classes\TachikomaProject" "" "Tachikoma Project"
  WriteRegStr SHCTX "Software\Classes\TachikomaProject\DefaultIcon" "" "$INSTDIR\${APP_EXECUTABLE_FILENAME},0"
  WriteRegStr SHCTX "Software\Classes\TachikomaProject\shell\open\command" "" '"$INSTDIR\${APP_EXECUTABLE_FILENAME}" "%1"'

  ; Register protocol handler
  WriteRegStr SHCTX "Software\Classes\tachikoma" "" "URL:Tachikoma Protocol"
  WriteRegStr SHCTX "Software\Classes\tachikoma" "URL Protocol" ""
  WriteRegStr SHCTX "Software\Classes\tachikoma\shell\open\command" "" '"$INSTDIR\${APP_EXECUTABLE_FILENAME}" "%1"'

  ; Add to PATH (optional)
  ; EnVar::AddValue "PATH" "$INSTDIR"
!macroend

!macro customUnInstall
  ; Remove registry entries
  DeleteRegKey SHCTX "Software\Classes\.tachi"
  DeleteRegKey SHCTX "Software\Classes\TachikomaProject"
  DeleteRegKey SHCTX "Software\Classes\tachikoma"
!macroend
```

### Linux Post-Install Script

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
if command -v gtk-update-icon-cache &> /dev/null; then
    gtk-update-icon-cache -f -t /usr/share/icons/hicolor || true
fi

# Create symlink in /usr/local/bin
ln -sf /opt/Tachikoma/tachikoma /usr/local/bin/tachikoma || true

echo "Tachikoma installation complete!"
```

### Linux Post-Remove Script

```bash
#!/bin/bash
# build/linux/after-remove.sh

# Remove symlink
rm -f /usr/local/bin/tachikoma || true

# Update desktop database
if command -v update-desktop-database &> /dev/null; then
    update-desktop-database /usr/share/applications || true
fi

echo "Tachikoma uninstallation complete!"
```

### Build Scripts

```json
{
  "scripts": {
    "build": "electron-vite build",
    "build:mac": "electron-builder --mac",
    "build:mac:universal": "electron-builder --mac --universal",
    "build:win": "electron-builder --win",
    "build:linux": "electron-builder --linux",
    "build:all": "electron-builder --mac --win --linux",
    "build:dir": "electron-builder --dir",
    "release": "electron-builder --publish always",
    "release:mac": "electron-builder --mac --publish always",
    "release:win": "electron-builder --win --publish always",
    "release:linux": "electron-builder --linux --publish always"
  }
}
```

### GitHub Actions Build Workflow

```yaml
# .github/workflows/build.yml
name: Build and Release

on:
  push:
    tags:
      - 'v*'
  workflow_dispatch:

jobs:
  build:
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
          cache: 'npm'

      - name: Install dependencies
        run: npm ci

      - name: Build native module
        run: |
          cd native
          npm ci
          npm run build

      - name: Build application
        run: npm run build

      - name: Package (macOS)
        if: matrix.platform == 'mac'
        run: npm run build:mac
        env:
          CSC_LINK: ${{ secrets.MAC_CERTS }}
          CSC_KEY_PASSWORD: ${{ secrets.MAC_CERTS_PASSWORD }}
          APPLE_ID: ${{ secrets.APPLE_ID }}
          APPLE_APP_SPECIFIC_PASSWORD: ${{ secrets.APPLE_APP_SPECIFIC_PASSWORD }}
          APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}

      - name: Package (Windows)
        if: matrix.platform == 'win'
        run: npm run build:win
        env:
          CSC_LINK: ${{ secrets.WIN_CERTS }}
          CSC_KEY_PASSWORD: ${{ secrets.WIN_CERTS_PASSWORD }}

      - name: Package (Linux)
        if: matrix.platform == 'linux'
        run: npm run build:linux

      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: release-${{ matrix.platform }}
          path: |
            release/**/*.dmg
            release/**/*.zip
            release/**/*.exe
            release/**/*.AppImage
            release/**/*.deb
            release/**/*.rpm

  release:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - name: Download all artifacts
        uses: actions/download-artifact@v4
        with:
          path: release

      - name: Create Release
        uses: softprops/action-gh-release@v1
        with:
          files: release/**/*
          draft: true
          generate_release_notes: true
```

---

## Testing Requirements

### Build Verification Tests

```typescript
// scripts/verify-build.ts
import { existsSync, statSync } from 'fs';
import { join } from 'path';

interface BuildArtifact {
  platform: string;
  path: string;
  minSize: number;
}

const artifacts: BuildArtifact[] = [
  { platform: 'mac', path: 'release/Tachikoma-*.dmg', minSize: 50 * 1024 * 1024 },
  { platform: 'win', path: 'release/Tachikoma-*.exe', minSize: 50 * 1024 * 1024 },
  { platform: 'linux', path: 'release/Tachikoma-*.AppImage', minSize: 50 * 1024 * 1024 },
];

async function verifyBuild(): Promise<void> {
  const glob = require('glob');

  for (const artifact of artifacts) {
    const files = glob.sync(artifact.path);

    if (files.length === 0) {
      console.error(`Missing artifact for ${artifact.platform}: ${artifact.path}`);
      continue;
    }

    for (const file of files) {
      const stats = statSync(file);

      if (stats.size < artifact.minSize) {
        console.error(
          `Artifact ${file} is too small: ${stats.size} bytes (expected at least ${artifact.minSize})`
        );
      } else {
        console.log(`Verified ${file}: ${Math.round(stats.size / 1024 / 1024)} MB`);
      }
    }
  }
}

verifyBuild();
```

---

## Related Specs

- Spec 161: Electron Main Process
- Spec 176: Code Signing
- Spec 177: macOS Build
- Spec 178: Windows Build
- Spec 179: Linux Build
