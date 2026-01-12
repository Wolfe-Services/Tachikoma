# Spec 494: Electron Packaging

## Phase
23 - Build/Package System

## Spec ID
494

## Status
Planned

## Dependencies
- Spec 491 (Build System Orchestration)
- Spec 492 (Build Configuration)
- Spec 493 (Rust Compilation)
- Spec 161 (Electron Main Process)

## Estimated Context
~12%

---

## Objective

Implement the Electron application packaging system using electron-builder. This includes configuration for all target platforms, auto-update integration, code signing preparation, and distribution artifact generation.

---

## Acceptance Criteria

- [ ] electron-builder configuration for all platforms
- [ ] Development and production build modes
- [ ] Auto-update configuration with electron-updater
- [ ] DMG, NSIS, AppImage, and DEB/RPM generation
- [ ] App icon and resource embedding
- [ ] Native module bundling
- [ ] ASAR archive configuration with unpacking rules
- [ ] Build artifact verification
- [ ] Portable builds for all platforms
- [ ] Delta update support

---

## Implementation Details

### Electron Builder Configuration (electron-builder.config.js)

```javascript
// electron/electron-builder.config.js

const { platform } = require('os');

/**
 * @type {import('electron-builder').Configuration}
 */
const config = {
  appId: 'com.tachikoma.app',
  productName: 'Tachikoma',
  copyright: 'Copyright (c) 2024 Tachikoma Team',

  // Directories
  directories: {
    output: 'release/${version}',
    buildResources: 'resources',
  },

  // Files to include
  files: [
    'dist/**/*',
    'package.json',
    '!**/*.map',
    '!**/node_modules/*/{CHANGELOG.md,README.md,README,readme.md,readme}',
    '!**/node_modules/*/{test,__tests__,tests,powered-test,example,examples}',
    '!**/node_modules/*.d.ts',
    '!**/node_modules/.bin',
    '!**/*.{iml,o,hprof,orig,pyc,pyo,rbc,swp,csproj,sln,xproj}',
    '!.editorconfig',
    '!**/._*',
    '!**/{.DS_Store,.git,.hg,.svn,CVS,RCS,SCCS,.gitignore,.gitattributes}',
  ],

  // Extra resources (native binaries)
  extraResources: [
    {
      from: '../target/release',
      to: 'bin',
      filter: ['tachikoma', 'tachikoma.exe', '!*.pdb', '!*.d'],
    },
  ],

  // ASAR configuration
  asar: true,
  asarUnpack: [
    '**/*.node',
    '**/node_modules/sharp/**/*',
    '**/node_modules/@napi-rs/**/*',
  ],

  // Compression
  compression: 'maximum',

  // Remove package scripts
  removePackageScripts: true,

  // Publish configuration
  publish: {
    provider: 'github',
    owner: 'tachikoma',
    repo: 'tachikoma',
    releaseType: 'release',
  },

  // macOS configuration
  mac: {
    target: [
      { target: 'dmg', arch: ['x64', 'arm64'] },
      { target: 'zip', arch: ['x64', 'arm64'] },
    ],
    category: 'public.app-category.developer-tools',
    icon: 'resources/icon.icns',
    darkModeSupport: true,
    hardenedRuntime: true,
    gatekeeperAssess: false,
    entitlements: 'resources/entitlements.mac.plist',
    entitlementsInherit: 'resources/entitlements.mac.plist',
    extendInfo: {
      NSMicrophoneUsageDescription: 'Tachikoma needs microphone access for voice input features.',
      NSAppleEventsUsageDescription: 'Tachikoma needs automation access to interact with other applications.',
    },
    notarize: false, // Enable in production with proper credentials
  },

  // DMG configuration
  dmg: {
    contents: [
      { x: 130, y: 220 },
      { x: 410, y: 220, type: 'link', path: '/Applications' },
    ],
    icon: 'resources/icon.icns',
    iconSize: 128,
    window: {
      width: 540,
      height: 380,
    },
    background: 'resources/dmg-background.png',
  },

  // Windows configuration
  win: {
    target: [
      { target: 'nsis', arch: ['x64', 'arm64'] },
      { target: 'portable', arch: ['x64'] },
      { target: 'msi', arch: ['x64'] },
    ],
    icon: 'resources/icon.ico',
    publisherName: 'Tachikoma Team',
    verifyUpdateCodeSignature: false, // Enable in production
    requestedExecutionLevel: 'asInvoker',
  },

  // NSIS configuration
  nsis: {
    oneClick: false,
    perMachine: false,
    allowToChangeInstallationDirectory: true,
    allowElevation: true,
    installerIcon: 'resources/icon.ico',
    uninstallerIcon: 'resources/icon.ico',
    installerHeaderIcon: 'resources/icon.ico',
    createDesktopShortcut: true,
    createStartMenuShortcut: true,
    menuCategory: true,
    shortcutName: 'Tachikoma',
    include: 'resources/installer.nsh',
    license: '../LICENSE',
  },

  // Linux configuration
  linux: {
    target: [
      { target: 'AppImage', arch: ['x64', 'arm64'] },
      { target: 'deb', arch: ['x64', 'arm64'] },
      { target: 'rpm', arch: ['x64'] },
      { target: 'snap', arch: ['x64'] },
    ],
    icon: 'resources/icons',
    category: 'Development',
    synopsis: 'AI-Powered Development Assistant',
    description: 'Tachikoma is an AI-powered development assistant that helps you write, review, and improve code.',
    desktop: {
      Name: 'Tachikoma',
      Comment: 'AI-Powered Development Assistant',
      Categories: 'Development;IDE;',
      Keywords: 'ai;coding;development;assistant;',
      StartupWMClass: 'tachikoma',
    },
    mimeTypes: ['x-scheme-handler/tachikoma'],
  },

  // DEB configuration
  deb: {
    depends: ['libgtk-3-0', 'libnotify4', 'libnss3', 'libxss1', 'libxtst6', 'xdg-utils'],
    afterInstall: 'resources/linux/after-install.sh',
    afterRemove: 'resources/linux/after-remove.sh',
  },

  // RPM configuration
  rpm: {
    depends: ['gtk3', 'libnotify', 'nss', 'libXScrnSaver', 'libXtst', 'xdg-utils'],
  },

  // Snap configuration
  snap: {
    confinement: 'classic',
    grade: 'stable',
    plugs: [
      'default',
      'removable-media',
      'home',
      'network',
      'network-bind',
    ],
  },

  // AppImage configuration
  appImage: {
    license: '../LICENSE',
  },

  // Auto-update configuration
  electronUpdaterCompatibility: '>=2.16',

  // Build hooks
  afterSign: 'scripts/notarize.js',
  afterPack: 'scripts/after-pack.js',
  afterAllArtifactBuild: 'scripts/after-build.js',
};

module.exports = config;
```

### Auto-Updater Configuration (electron/src/main/updater.ts)

```typescript
// electron/src/main/updater.ts
import { app, BrowserWindow, dialog } from 'electron';
import {
  autoUpdater,
  UpdateCheckResult,
  UpdateInfo,
  ProgressInfo,
} from 'electron-updater';
import log from 'electron-log';

interface UpdaterConfig {
  autoDownload: boolean;
  autoInstallOnAppQuit: boolean;
  allowPrerelease: boolean;
  checkInterval: number; // milliseconds
}

const defaultConfig: UpdaterConfig = {
  autoDownload: true,
  autoInstallOnAppQuit: true,
  allowPrerelease: false,
  checkInterval: 1000 * 60 * 60 * 4, // 4 hours
};

class AppUpdater {
  private config: UpdaterConfig;
  private mainWindow: BrowserWindow | null = null;
  private checkInterval: NodeJS.Timeout | null = null;

  constructor(config: Partial<UpdaterConfig> = {}) {
    this.config = { ...defaultConfig, ...config };
    this.setupUpdater();
  }

  private setupUpdater(): void {
    // Configure logger
    log.transports.file.level = 'info';
    autoUpdater.logger = log;

    // Configure updater
    autoUpdater.autoDownload = this.config.autoDownload;
    autoUpdater.autoInstallOnAppQuit = this.config.autoInstallOnAppQuit;
    autoUpdater.allowPrerelease = this.config.allowPrerelease;

    // Set up event handlers
    autoUpdater.on('checking-for-update', () => {
      log.info('Checking for updates...');
      this.sendStatusToWindow('checking-for-update');
    });

    autoUpdater.on('update-available', (info: UpdateInfo) => {
      log.info('Update available:', info);
      this.sendStatusToWindow('update-available', info);
      this.showUpdateNotification(info);
    });

    autoUpdater.on('update-not-available', (info: UpdateInfo) => {
      log.info('Update not available:', info);
      this.sendStatusToWindow('update-not-available', info);
    });

    autoUpdater.on('error', (err: Error) => {
      log.error('Update error:', err);
      this.sendStatusToWindow('error', { message: err.message });
    });

    autoUpdater.on('download-progress', (progress: ProgressInfo) => {
      log.info(`Download progress: ${progress.percent.toFixed(2)}%`);
      this.sendStatusToWindow('download-progress', progress);
    });

    autoUpdater.on('update-downloaded', (info: UpdateInfo) => {
      log.info('Update downloaded:', info);
      this.sendStatusToWindow('update-downloaded', info);
      this.promptInstall(info);
    });
  }

  setWindow(window: BrowserWindow): void {
    this.mainWindow = window;
  }

  private sendStatusToWindow(status: string, data?: unknown): void {
    if (this.mainWindow?.webContents) {
      this.mainWindow.webContents.send('updater:status', { status, data });
    }
  }

  private async showUpdateNotification(info: UpdateInfo): Promise<void> {
    if (!this.mainWindow) return;

    const { response } = await dialog.showMessageBox(this.mainWindow, {
      type: 'info',
      title: 'Update Available',
      message: `A new version (${info.version}) is available.`,
      detail: info.releaseNotes
        ? `Release Notes:\n${typeof info.releaseNotes === 'string' ? info.releaseNotes : info.releaseNotes.map((n) => n.note).join('\n')}`
        : undefined,
      buttons: ['Download Now', 'Later'],
      defaultId: 0,
      cancelId: 1,
    });

    if (response === 0 && !this.config.autoDownload) {
      autoUpdater.downloadUpdate();
    }
  }

  private async promptInstall(info: UpdateInfo): Promise<void> {
    if (!this.mainWindow) return;

    const { response } = await dialog.showMessageBox(this.mainWindow, {
      type: 'info',
      title: 'Update Ready',
      message: `Version ${info.version} has been downloaded.`,
      detail: 'The update will be installed when you quit the application. Would you like to restart now?',
      buttons: ['Restart Now', 'Later'],
      defaultId: 0,
      cancelId: 1,
    });

    if (response === 0) {
      autoUpdater.quitAndInstall(false, true);
    }
  }

  async checkForUpdates(): Promise<UpdateCheckResult | null> {
    try {
      return await autoUpdater.checkForUpdates();
    } catch (error) {
      log.error('Failed to check for updates:', error);
      return null;
    }
  }

  startAutoCheck(): void {
    this.stopAutoCheck();
    this.checkInterval = setInterval(() => {
      this.checkForUpdates();
    }, this.config.checkInterval);

    // Check immediately on start
    setTimeout(() => this.checkForUpdates(), 3000);
  }

  stopAutoCheck(): void {
    if (this.checkInterval) {
      clearInterval(this.checkInterval);
      this.checkInterval = null;
    }
  }

  downloadUpdate(): void {
    autoUpdater.downloadUpdate();
  }

  installUpdate(): void {
    autoUpdater.quitAndInstall(false, true);
  }

  getUpdateInfo(): UpdateInfo | null {
    return autoUpdater.updateInfo;
  }
}

export function setupAutoUpdater(mainWindow: BrowserWindow): AppUpdater {
  const updater = new AppUpdater({
    autoDownload: true,
    allowPrerelease: false,
  });

  updater.setWindow(mainWindow);
  updater.startAutoCheck();

  return updater;
}

export { AppUpdater, UpdaterConfig };
```

### Build Scripts

```typescript
// electron/scripts/after-pack.js
const { execSync } = require('child_process');
const path = require('path');
const fs = require('fs');

/**
 * @param {import('electron-builder').AfterPackContext} context
 */
exports.default = async function afterPack(context) {
  const { appOutDir, packager, electronPlatformName } = context;

  console.log(`After pack: ${electronPlatformName} - ${appOutDir}`);

  // Copy native binaries
  const binDir = path.join(appOutDir, 'resources', 'bin');
  if (!fs.existsSync(binDir)) {
    fs.mkdirSync(binDir, { recursive: true });
  }

  // Platform-specific binary handling
  const platform = electronPlatformName;
  const arch = packager.arch;

  let sourceDir;
  if (platform === 'darwin') {
    // For macOS, binaries are inside the app bundle
    sourceDir = path.join(
      __dirname,
      '..',
      '..',
      'target',
      arch === 'arm64' ? 'aarch64-apple-darwin' : 'x86_64-apple-darwin',
      'release'
    );
  } else if (platform === 'linux') {
    sourceDir = path.join(
      __dirname,
      '..',
      '..',
      'target',
      arch === 'arm64' ? 'aarch64-unknown-linux-gnu' : 'x86_64-unknown-linux-gnu',
      'release'
    );
  } else if (platform === 'win32') {
    sourceDir = path.join(
      __dirname,
      '..',
      '..',
      'target',
      arch === 'arm64' ? 'aarch64-pc-windows-msvc' : 'x86_64-pc-windows-msvc',
      'release'
    );
  }

  // Copy binaries
  const binaryName = platform === 'win32' ? 'tachikoma.exe' : 'tachikoma';
  const sourceBin = path.join(sourceDir, binaryName);

  if (fs.existsSync(sourceBin)) {
    fs.copyFileSync(sourceBin, path.join(binDir, binaryName));
    console.log(`Copied ${binaryName} to ${binDir}`);
  } else {
    console.warn(`Binary not found: ${sourceBin}`);
  }
};
```

### Notarization Script (macOS)

```javascript
// electron/scripts/notarize.js
const { notarize } = require('@electron/notarize');
const path = require('path');

/**
 * @param {import('electron-builder').AfterSignContext} context
 */
exports.default = async function notarizing(context) {
  const { electronPlatformName, appOutDir } = context;

  if (electronPlatformName !== 'darwin') {
    return;
  }

  // Check if we have notarization credentials
  if (!process.env.APPLE_ID || !process.env.APPLE_ID_PASSWORD) {
    console.log('Skipping notarization - no credentials');
    return;
  }

  const appName = context.packager.appInfo.productFilename;
  const appPath = path.join(appOutDir, `${appName}.app`);

  console.log(`Notarizing ${appPath}...`);

  try {
    await notarize({
      tool: 'notarytool',
      appPath,
      appleId: process.env.APPLE_ID,
      appleIdPassword: process.env.APPLE_ID_PASSWORD,
      teamId: process.env.APPLE_TEAM_ID,
    });

    console.log('Notarization complete');
  } catch (error) {
    console.error('Notarization failed:', error);
    throw error;
  }
};
```

### macOS Entitlements (resources/entitlements.mac.plist)

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>com.apple.security.cs.allow-jit</key>
    <true/>
    <key>com.apple.security.cs.allow-unsigned-executable-memory</key>
    <true/>
    <key>com.apple.security.cs.disable-library-validation</key>
    <true/>
    <key>com.apple.security.network.client</key>
    <true/>
    <key>com.apple.security.network.server</key>
    <true/>
    <key>com.apple.security.files.user-selected.read-write</key>
    <true/>
    <key>com.apple.security.files.downloads.read-write</key>
    <true/>
    <key>com.apple.security.automation.apple-events</key>
    <true/>
</dict>
</plist>
```

### Package.json Scripts

```json
{
  "scripts": {
    "build": "electron-vite build",
    "build:unpack": "electron-builder --dir",
    "package": "electron-builder",
    "package:mac": "electron-builder --mac",
    "package:mac-universal": "electron-builder --mac --universal",
    "package:win": "electron-builder --win",
    "package:linux": "electron-builder --linux",
    "package:all": "electron-builder -mwl",
    "publish": "electron-builder --publish always",
    "publish:mac": "electron-builder --mac --publish always",
    "publish:win": "electron-builder --win --publish always",
    "publish:linux": "electron-builder --linux --publish always"
  }
}
```

---

## Testing Requirements

### Unit Tests

```typescript
// electron/src/main/__tests__/updater.test.ts
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { AppUpdater } from '../updater';

vi.mock('electron', () => ({
  app: { getVersion: vi.fn().mockReturnValue('1.0.0') },
  dialog: { showMessageBox: vi.fn().mockResolvedValue({ response: 1 }) },
}));

vi.mock('electron-updater', () => ({
  autoUpdater: {
    on: vi.fn(),
    checkForUpdates: vi.fn().mockResolvedValue(null),
    downloadUpdate: vi.fn(),
    quitAndInstall: vi.fn(),
    logger: null,
    autoDownload: false,
    autoInstallOnAppQuit: false,
    allowPrerelease: false,
    updateInfo: null,
  },
}));

describe('AppUpdater', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should create updater with default config', () => {
    const updater = new AppUpdater();
    expect(updater).toBeDefined();
  });

  it('should accept custom configuration', () => {
    const updater = new AppUpdater({
      autoDownload: false,
      allowPrerelease: true,
    });
    expect(updater).toBeDefined();
  });

  it('should check for updates', async () => {
    const updater = new AppUpdater();
    const result = await updater.checkForUpdates();
    expect(result).toBeNull(); // Mocked to return null
  });
});
```

### Integration Tests

```typescript
// electron/__tests__/build.integration.test.ts
import { describe, it, expect } from 'vitest';
import * as fs from 'fs';
import * as path from 'path';

describe('Electron Build Output', () => {
  const releaseDir = path.join(__dirname, '..', 'release');

  it('should have electron-builder config', () => {
    const configPath = path.join(__dirname, '..', 'electron-builder.config.js');
    expect(fs.existsSync(configPath)).toBe(true);
  });

  it('should have entitlements file for macOS', () => {
    const entitlementsPath = path.join(
      __dirname,
      '..',
      'resources',
      'entitlements.mac.plist'
    );
    expect(fs.existsSync(entitlementsPath)).toBe(true);
  });
});
```

---

## Related Specs

- Spec 491: Build System Orchestration
- Spec 498: Code Signing
- Spec 499: macOS Packaging
- Spec 500: Windows Installer
- Spec 501: Linux Packages
- Spec 503: Release Workflow
