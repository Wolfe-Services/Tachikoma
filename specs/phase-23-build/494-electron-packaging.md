# 494 - Electron Packaging

**Phase:** 23 - Build & Distribution
**Spec ID:** 494
**Status:** Planned
**Dependencies:** 491-build-overview, 175-build-config
**Estimated Context:** ~12% of Sonnet window

---

## Objective

Configure electron-builder for packaging the Tachikoma application into distributable formats for macOS, Windows, and Linux platforms.

---

## Acceptance Criteria

- [ ] electron-builder configuration for all platforms
- [ ] Native dependencies properly bundled
- [ ] Assets and resources correctly included
- [ ] Auto-update configuration prepared
- [ ] File associations configured
- [ ] Deep linking protocol registered

---

## Implementation Details

### 1. electron-builder Configuration

Create `electron/electron-builder.config.js`:

```javascript
/**
 * electron-builder configuration for Tachikoma
 * @type {import('electron-builder').Configuration}
 */
module.exports = {
  appId: 'com.tachikoma.app',
  productName: 'Tachikoma',
  copyright: 'Copyright (c) 2024 Tachikoma',

  // Directories
  directories: {
    output: 'out',
    buildResources: 'build',
  },

  // Files to include
  files: [
    'dist/**/*',
    'node_modules/**/*',
    '!node_modules/**/test/**',
    '!node_modules/**/*.md',
    '!node_modules/**/CHANGELOG*',
    '!node_modules/**/LICENSE*',
  ],

  // Extra resources (not packed into asar)
  extraResources: [
    {
      from: '../web/dist',
      to: 'web',
      filter: ['**/*'],
    },
    {
      from: '../crates/tachikoma-native/tachikoma-native.node',
      to: 'native/tachikoma-native.node',
    },
  ],

  // ASAR archive
  asar: true,
  asarUnpack: [
    '**/node_modules/sharp/**',
    '**/node_modules/@electron/**',
    '**/*.node',
  ],

  // Compression
  compression: 'maximum',

  // macOS configuration
  mac: {
    category: 'public.app-category.developer-tools',
    icon: 'build/icon.icns',
    hardenedRuntime: true,
    gatekeeperAssess: false,
    entitlements: 'build/entitlements.mac.plist',
    entitlementsInherit: 'build/entitlements.mac.plist',

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

    // File associations
    fileAssociations: [
      {
        ext: 'tspec',
        name: 'Tachikoma Spec',
        role: 'Editor',
        icon: 'build/file-icon.icns',
      },
    ],

    // Protocol
    protocols: [
      {
        name: 'Tachikoma',
        schemes: ['tachikoma'],
      },
    ],
  },

  // DMG configuration
  dmg: {
    artifactName: '${productName}-${version}-${arch}.${ext}',
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
    iconSize: 128,
  },

  // Windows configuration
  win: {
    icon: 'build/icon.ico',
    target: [
      {
        target: 'nsis',
        arch: ['x64'],
      },
    ],

    // File associations
    fileAssociations: [
      {
        ext: 'tspec',
        name: 'Tachikoma Spec',
        icon: 'build/file-icon.ico',
      },
    ],

    // Protocol
    protocols: [
      {
        name: 'Tachikoma',
        schemes: ['tachikoma'],
      },
    ],
  },

  // NSIS installer configuration
  nsis: {
    oneClick: false,
    allowToChangeInstallationDirectory: true,
    createDesktopShortcut: true,
    createStartMenuShortcut: true,
    shortcutName: 'Tachikoma',
    artifactName: '${productName}-Setup-${version}.${ext}',
    include: 'build/installer.nsh',
    installerIcon: 'build/icon.ico',
    uninstallerIcon: 'build/icon.ico',
    installerHeader: 'build/installer-header.bmp',
    installerSidebar: 'build/installer-sidebar.bmp',
  },

  // Linux configuration
  linux: {
    icon: 'build/icons',
    category: 'Development',
    target: [
      {
        target: 'AppImage',
        arch: ['x64'],
      },
      {
        target: 'deb',
        arch: ['x64'],
      },
    ],

    // File associations
    fileAssociations: [
      {
        ext: 'tspec',
        name: 'Tachikoma Spec',
        mimeType: 'application/x-tachikoma-spec',
      },
    ],

    // Desktop entry
    desktop: {
      MimeType: 'x-scheme-handler/tachikoma;application/x-tachikoma-spec',
    },
  },

  // AppImage configuration
  appImage: {
    artifactName: '${productName}-${version}.${ext}',
    license: 'LICENSE',
  },

  // Debian package configuration
  deb: {
    artifactName: '${productName}_${version}_${arch}.${ext}',
    depends: [
      'libgtk-3-0',
      'libnotify4',
      'libnss3',
      'libxss1',
      'libxtst6',
      'xdg-utils',
      'libatspi2.0-0',
      'libuuid1',
    ],
  },

  // Auto-update configuration
  publish: {
    provider: 'github',
    owner: 'tachikoma',
    repo: 'tachikoma',
    releaseType: 'release',
  },

  // Build hooks
  afterSign: 'scripts/notarize.js',
  afterPack: 'scripts/after-pack.js',
};
```

### 2. macOS Entitlements

Create `electron/build/entitlements.mac.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <!-- Allow JIT compilation for V8 -->
    <key>com.apple.security.cs.allow-jit</key>
    <true/>

    <!-- Allow unsigned executable memory -->
    <key>com.apple.security.cs.allow-unsigned-executable-memory</key>
    <true/>

    <!-- Network access -->
    <key>com.apple.security.network.client</key>
    <true/>
    <key>com.apple.security.network.server</key>
    <true/>

    <!-- File access -->
    <key>com.apple.security.files.user-selected.read-write</key>
    <true/>

    <!-- Automation (for AppleScript) -->
    <key>com.apple.security.automation.apple-events</key>
    <true/>
</dict>
</plist>
```

### 3. Package Scripts

Update `electron/package.json`:

```json
{
  "name": "tachikoma-electron",
  "version": "0.1.0",
  "private": true,
  "main": "dist/main.js",
  "scripts": {
    "build": "tsc",
    "watch": "tsc -w",
    "start": "electron .",
    "package": "electron-builder --dir",
    "dist": "electron-builder",
    "dist:mac": "electron-builder --mac",
    "dist:win": "electron-builder --win",
    "dist:linux": "electron-builder --linux",
    "dist:all": "electron-builder -mwl",
    "clean": "rm -rf dist out"
  },
  "devDependencies": {
    "electron": "^28.0.0",
    "electron-builder": "^24.9.0",
    "typescript": "^5.3.0"
  },
  "dependencies": {
    "electron-updater": "^6.1.0"
  },
  "build": {
    "extends": "./electron-builder.config.js"
  }
}
```

### 4. After-Pack Hook

Create `electron/scripts/after-pack.js`:

```javascript
/**
 * Post-pack hook for additional processing
 */

const fs = require('fs');
const path = require('path');

exports.default = async function (context) {
  const { appOutDir, packager } = context;
  const platform = packager.platform.name;

  console.log(`After pack for ${platform}`);

  // Platform-specific post-processing
  if (platform === 'mac') {
    // macOS specific
    const resourcesPath = path.join(appOutDir, 'Tachikoma.app', 'Contents', 'Resources');

    // Copy native module to correct location
    const nativeModuleSrc = path.join(__dirname, '../../crates/tachikoma-native/tachikoma-native.node');
    const nativeModuleDst = path.join(resourcesPath, 'native', 'tachikoma-native.node');

    if (fs.existsSync(nativeModuleSrc)) {
      fs.mkdirSync(path.dirname(nativeModuleDst), { recursive: true });
      fs.copyFileSync(nativeModuleSrc, nativeModuleDst);
      console.log('Copied native module');
    }
  }

  if (platform === 'win') {
    // Windows specific
  }

  if (platform === 'linux') {
    // Linux specific
  }

  console.log('After pack complete');
};
```

### 5. Build Resources Structure

```
electron/build/
├── icon.icns              # macOS app icon
├── icon.ico               # Windows app icon
├── icons/                 # Linux icons
│   ├── 16x16.png
│   ├── 32x32.png
│   ├── 48x48.png
│   ├── 64x64.png
│   ├── 128x128.png
│   ├── 256x256.png
│   └── 512x512.png
├── file-icon.icns         # macOS file association icon
├── file-icon.ico          # Windows file association icon
├── dmg-background.png     # macOS DMG background
├── installer-header.bmp   # Windows installer header
├── installer-sidebar.bmp  # Windows installer sidebar
├── installer.nsh          # NSIS custom script
└── entitlements.mac.plist # macOS entitlements
```

### 6. CI Build Integration

Create `electron/scripts/build-ci.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

PLATFORM="${1:-}"

echo "Building Electron app for: ${PLATFORM:-all platforms}"

# Install dependencies
npm ci

# Build TypeScript
npm run build

# Package based on platform
case "$PLATFORM" in
    mac|darwin)
        npm run dist:mac
        ;;
    win|windows)
        npm run dist:win
        ;;
    linux)
        npm run dist:linux
        ;;
    all|"")
        npm run dist:all
        ;;
    *)
        echo "Unknown platform: $PLATFORM"
        exit 1
        ;;
esac

echo "Build complete! Artifacts in electron/out/"
ls -la out/
```

---

## Testing Requirements

1. `npm run package` creates unpacked app
2. `npm run dist` creates distributable packages
3. App launches correctly from packaged version
4. Native module loads in packaged app
5. File associations work on all platforms

---

## Related Specs

- Depends on: [491-build-overview.md](491-build-overview.md), [175-build-config.md](../phase-08-electron/175-build-config.md)
- Next: [495-macos-dmg.md](495-macos-dmg.md)
- Related: [167-auto-updates.md](../phase-08-electron/167-auto-updates.md)
