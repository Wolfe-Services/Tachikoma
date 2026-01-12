# Spec 499: macOS Packaging

## Phase
23 - Build/Package System

## Spec ID
499

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

Implement macOS-specific packaging including DMG creation, universal binary support, hardened runtime configuration, and App Store preparation (optional). Ensure compliance with Apple's notarization requirements.

---

## Acceptance Criteria

- [ ] DMG creation with custom background and layout
- [ ] Universal binary for Intel and Apple Silicon
- [ ] Hardened runtime with proper entitlements
- [ ] Sparkle integration for auto-updates
- [ ] App sandbox configuration (optional)
- [ ] Dock and menu bar integration
- [ ] File association registration
- [ ] URL scheme registration
- [ ] Launch at login support
- [ ] App Store build variant (optional)

---

## Implementation Details

### macOS Build Configuration (electron/build/mac.config.ts)

```typescript
// electron/build/mac.config.ts
import type { Configuration, MacConfiguration } from 'electron-builder';
import * as path from 'path';

export const macConfig: MacConfiguration = {
  // Target formats
  target: [
    {
      target: 'dmg',
      arch: ['x64', 'arm64', 'universal'],
    },
    {
      target: 'zip',
      arch: ['x64', 'arm64', 'universal'],
    },
  ],

  // Application category
  category: 'public.app-category.developer-tools',

  // Icon
  icon: 'resources/icon.icns',

  // Dark mode support
  darkModeSupport: true,

  // Hardened runtime
  hardenedRuntime: true,
  gatekeeperAssess: false,

  // Entitlements
  entitlements: 'resources/entitlements.mac.plist',
  entitlementsInherit: 'resources/entitlements.mac.plist',

  // Identity
  identity: process.env.APPLE_SIGNING_IDENTITY || null,

  // Notarization
  notarize: process.env.APPLE_ID
    ? {
        teamId: process.env.APPLE_TEAM_ID!,
      }
    : false,

  // Extra info.plist entries
  extendInfo: {
    // Accessibility permissions
    NSAppleEventsUsageDescription:
      'Tachikoma needs automation access to interact with other applications for development workflows.',

    // File system access
    NSDesktopFolderUsageDescription:
      'Tachikoma needs access to your Desktop folder to read and write files.',
    NSDocumentsFolderUsageDescription:
      'Tachikoma needs access to your Documents folder to manage projects.',
    NSDownloadsFolderUsageDescription:
      'Tachikoma needs access to your Downloads folder for file operations.',

    // Custom URL scheme
    CFBundleURLTypes: [
      {
        CFBundleURLSchemes: ['tachikoma'],
        CFBundleURLName: 'com.tachikoma.app',
        CFBundleTypeRole: 'Editor',
      },
    ],

    // File associations
    CFBundleDocumentTypes: [
      {
        CFBundleTypeName: 'Tachikoma Mission',
        CFBundleTypeExtensions: ['tkmission'],
        CFBundleTypeIconFile: 'document.icns',
        CFBundleTypeRole: 'Editor',
        LSHandlerRank: 'Owner',
      },
      {
        CFBundleTypeName: 'Tachikoma Spec',
        CFBundleTypeExtensions: ['tkspec'],
        CFBundleTypeRole: 'Editor',
        LSHandlerRank: 'Owner',
      },
    ],

    // Launch services
    LSMinimumSystemVersion: '10.15.0',
    LSApplicationCategoryType: 'public.app-category.developer-tools',

    // Sparkle updates
    SUPublicEDKey: process.env.SPARKLE_PUBLIC_KEY || '',
    SUFeedURL: 'https://releases.tachikoma.io/appcast.xml',
    SUEnableAutomaticChecks: true,
  },

  // Signing configuration
  signIgnore: [
    // Ignore signing for these patterns during development
    '*.provisionprofile',
  ],

  // Extra resources
  extraResources: [
    {
      from: '../target/universal/release/tachikoma',
      to: 'bin/tachikoma',
    },
  ],

  // Binaries to sign
  binaries: ['resources/bin/tachikoma'],

  // Minimum system version
  minimumSystemVersion: '10.15.0',

  // Artificial delay for signing (helps with keychain race conditions)
  artifactBuildCompleted: async () => {
    await new Promise((resolve) => setTimeout(resolve, 1000));
  },
};
```

### DMG Configuration (electron/build/dmg.config.ts)

```typescript
// electron/build/dmg.config.ts
import type { DmgOptions } from 'electron-builder';

export const dmgConfig: DmgOptions = {
  // Custom background image
  background: 'resources/dmg-background.png',

  // Background color (fallback)
  backgroundColor: '#ffffff',

  // Icon configuration
  icon: 'resources/icon.icns',
  iconSize: 100,

  // Window configuration
  window: {
    x: 400,
    y: 400,
    width: 660,
    height: 480,
  },

  // Icon positions
  contents: [
    {
      x: 180,
      y: 220,
      type: 'file',
    },
    {
      x: 480,
      y: 220,
      type: 'link',
      path: '/Applications',
    },
  ],

  // Title
  title: '${productName} ${version}',

  // Format
  format: 'ULFO', // ULFO for better compression, UDBZ for compatibility

  // Internet-enabled
  internetEnabled: true,

  // Write update info
  writeUpdateInfo: true,
};
```

### Entitlements (resources/entitlements.mac.plist)

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <!-- Allow JIT compilation for V8 -->
    <key>com.apple.security.cs.allow-jit</key>
    <true/>

    <!-- Allow unsigned executable memory (required for Electron) -->
    <key>com.apple.security.cs.allow-unsigned-executable-memory</key>
    <true/>

    <!-- Disable library validation (for native modules) -->
    <key>com.apple.security.cs.disable-library-validation</key>
    <true/>

    <!-- Allow dyld environment variables (for development) -->
    <key>com.apple.security.cs.allow-dyld-environment-variables</key>
    <true/>

    <!-- Network access -->
    <key>com.apple.security.network.client</key>
    <true/>
    <key>com.apple.security.network.server</key>
    <true/>

    <!-- File access (for drag and drop, open/save dialogs) -->
    <key>com.apple.security.files.user-selected.read-write</key>
    <true/>
    <key>com.apple.security.files.downloads.read-write</key>
    <true/>

    <!-- Automation (for AppleScript support) -->
    <key>com.apple.security.automation.apple-events</key>
    <true/>

    <!-- Device access (for hardware info) -->
    <key>com.apple.security.device.usb</key>
    <true/>

    <!-- Personal information access (for calendar/contacts integration) -->
    <!-- Only enable if needed -->
    <!--
    <key>com.apple.security.personal-information.addressbook</key>
    <true/>
    -->
</dict>
</plist>
```

### App Sandbox Entitlements (resources/entitlements.mac.sandbox.plist)

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <!-- Enable App Sandbox -->
    <key>com.apple.security.app-sandbox</key>
    <true/>

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

    <!-- Temporary exception for Electron -->
    <key>com.apple.security.temporary-exception.mach-lookup.global-name</key>
    <array>
        <string>com.apple.coreservices.launchservicesd</string>
    </array>

    <!-- Allow JIT (requires special entitlement from Apple) -->
    <key>com.apple.security.cs.allow-jit</key>
    <true/>
</dict>
</plist>
```

### Universal Binary Build Script (scripts/build-macos-universal.ts)

```typescript
// scripts/build-macos-universal.ts
import { spawn } from 'child_process';
import * as fs from 'fs';
import * as path from 'path';

interface UniversalBuildConfig {
  appName: string;
  x64AppPath: string;
  arm64AppPath: string;
  outputPath: string;
}

async function createUniversalApp(config: UniversalBuildConfig): Promise<void> {
  const { appName, x64AppPath, arm64AppPath, outputPath } = config;

  console.log('Creating universal app bundle...');

  // Ensure output directory exists
  fs.mkdirSync(path.dirname(outputPath), { recursive: true });

  // Copy arm64 app as base (it's usually smaller)
  await runCommand('cp', ['-R', arm64AppPath, outputPath]);

  // Find all Mach-O binaries in the app
  const binaries = await findMachOBinaries(arm64AppPath);

  for (const binary of binaries) {
    const relativePath = path.relative(arm64AppPath, binary);
    const x64Binary = path.join(x64AppPath, relativePath);
    const outputBinary = path.join(outputPath, relativePath);

    if (fs.existsSync(x64Binary)) {
      console.log(`Creating universal binary: ${relativePath}`);
      await createUniversalBinary(x64Binary, binary, outputBinary);
    }
  }

  console.log('Universal app created successfully!');
}

async function findMachOBinaries(appPath: string): Promise<string[]> {
  const binaries: string[] = [];

  async function walk(dir: string): Promise<void> {
    const entries = fs.readdirSync(dir, { withFileTypes: true });

    for (const entry of entries) {
      const fullPath = path.join(dir, entry.name);

      if (entry.isDirectory()) {
        await walk(fullPath);
      } else if (entry.isFile()) {
        // Check if it's a Mach-O binary
        const isMachO = await checkMachO(fullPath);
        if (isMachO) {
          binaries.push(fullPath);
        }
      }
    }
  }

  await walk(appPath);
  return binaries;
}

async function checkMachO(filePath: string): Promise<boolean> {
  return new Promise((resolve) => {
    const proc = spawn('file', [filePath]);
    let output = '';

    proc.stdout.on('data', (data) => {
      output += data.toString();
    });

    proc.on('close', () => {
      resolve(output.includes('Mach-O'));
    });
  });
}

async function createUniversalBinary(
  x64Path: string,
  arm64Path: string,
  outputPath: string
): Promise<void> {
  // Create temporary file for the universal binary
  const tempPath = `${outputPath}.tmp`;

  await runCommand('lipo', [
    '-create',
    '-output', tempPath,
    x64Path,
    arm64Path,
  ]);

  // Replace original with universal binary
  fs.renameSync(tempPath, outputPath);
}

async function runCommand(command: string, args: string[]): Promise<void> {
  return new Promise((resolve, reject) => {
    const proc = spawn(command, args, { stdio: 'inherit' });

    proc.on('close', (code) => {
      if (code === 0) {
        resolve();
      } else {
        reject(new Error(`${command} failed with code ${code}`));
      }
    });

    proc.on('error', (err) => {
      reject(err);
    });
  });
}

// Export for programmatic use
export { createUniversalApp, UniversalBuildConfig };

// CLI entry point
if (require.main === module) {
  const args = process.argv.slice(2);

  if (args.length < 3) {
    console.error('Usage: ts-node build-macos-universal.ts <x64-app> <arm64-app> <output>');
    process.exit(1);
  }

  createUniversalApp({
    appName: 'Tachikoma',
    x64AppPath: args[0],
    arm64AppPath: args[1],
    outputPath: args[2],
  }).catch((error) => {
    console.error(error);
    process.exit(1);
  });
}
```

### Launch at Login Helper (electron/src/main/launch-at-login.ts)

```typescript
// electron/src/main/launch-at-login.ts
import { app } from 'electron';

interface LaunchSettings {
  openAtLogin: boolean;
  openAsHidden: boolean;
}

export function getLaunchAtLoginSettings(): LaunchSettings {
  const settings = app.getLoginItemSettings();

  return {
    openAtLogin: settings.openAtLogin,
    openAsHidden: settings.openAsHidden ?? false,
  };
}

export function setLaunchAtLogin(settings: Partial<LaunchSettings>): void {
  app.setLoginItemSettings({
    openAtLogin: settings.openAtLogin ?? false,
    openAsHidden: settings.openAsHidden ?? false,
    // For macOS MAS builds
    // path: process.execPath,
    // args: ['--hidden'],
  });
}

export function toggleLaunchAtLogin(): boolean {
  const current = getLaunchAtLoginSettings();
  const newValue = !current.openAtLogin;

  setLaunchAtLogin({ openAtLogin: newValue });

  return newValue;
}
```

### Sparkle Update Configuration (electron/src/main/sparkle-updater.ts)

```typescript
// electron/src/main/sparkle-updater.ts
import { NativeImage, nativeImage } from 'electron';

// Note: This requires the Sparkle framework to be integrated
// For Electron apps, electron-updater is typically preferred

interface SparkleConfig {
  feedURL: string;
  publicEdKey?: string;
  automaticChecks: boolean;
  checkInterval: number; // seconds
}

// Appcast XML format for Sparkle
export function generateAppcastItem(options: {
  version: string;
  buildNumber: string;
  releaseNotes: string;
  pubDate: Date;
  downloadURL: string;
  signature: string;
  length: number;
}): string {
  const pubDateString = options.pubDate.toUTCString();

  return `
    <item>
      <title>Version ${options.version}</title>
      <sparkle:releaseNotesLink>https://tachikoma.io/releases/${options.version}/notes</sparkle:releaseNotesLink>
      <pubDate>${pubDateString}</pubDate>
      <enclosure
        url="${options.downloadURL}"
        sparkle:version="${options.buildNumber}"
        sparkle:shortVersionString="${options.version}"
        sparkle:edSignature="${options.signature}"
        length="${options.length}"
        type="application/octet-stream"
      />
      <sparkle:minimumSystemVersion>10.15</sparkle:minimumSystemVersion>
    </item>
  `;
}

export function generateAppcast(
  items: Array<Parameters<typeof generateAppcastItem>[0]>
): string {
  const itemsXml = items.map(generateAppcastItem).join('\n');

  return `<?xml version="1.0" encoding="utf-8"?>
<rss version="2.0" xmlns:sparkle="http://www.andymatuschak.org/xml-namespaces/sparkle" xmlns:dc="http://purl.org/dc/elements/1.1/">
  <channel>
    <title>Tachikoma Updates</title>
    <link>https://tachikoma.io</link>
    <description>Updates for Tachikoma</description>
    <language>en</language>
    ${itemsXml}
  </channel>
</rss>`;
}
```

---

## Testing Requirements

### Unit Tests

```typescript
// scripts/__tests__/macos-build.test.ts
import { describe, it, expect, vi } from 'vitest';
import { macConfig, dmgConfig } from '../build/mac.config';

describe('macOS Build Configuration', () => {
  it('should have valid target configuration', () => {
    expect(macConfig.target).toBeDefined();
    expect(Array.isArray(macConfig.target)).toBe(true);
  });

  it('should have hardened runtime enabled', () => {
    expect(macConfig.hardenedRuntime).toBe(true);
  });

  it('should have entitlements configured', () => {
    expect(macConfig.entitlements).toBeDefined();
    expect(macConfig.entitlementsInherit).toBeDefined();
  });

  it('should have DMG configuration', () => {
    expect(dmgConfig.window).toBeDefined();
    expect(dmgConfig.contents).toBeDefined();
    expect(dmgConfig.contents.length).toBe(2);
  });
});
```

### Integration Tests

```typescript
// scripts/__tests__/macos-build.integration.test.ts
import { describe, it, expect } from 'vitest';
import * as fs from 'fs';
import * as path from 'path';
import { spawn } from 'child_process';

describe('macOS Build Integration', () => {
  it('should have entitlements file', () => {
    const entitlementsPath = path.join(
      __dirname,
      '..',
      '..',
      'resources',
      'entitlements.mac.plist'
    );

    if (process.platform === 'darwin') {
      expect(fs.existsSync(entitlementsPath)).toBe(true);
    }
  });

  it('should validate entitlements plist', async () => {
    if (process.platform !== 'darwin') return;

    const entitlementsPath = path.join(
      __dirname,
      '..',
      '..',
      'resources',
      'entitlements.mac.plist'
    );

    const result = await new Promise<boolean>((resolve) => {
      const proc = spawn('plutil', ['-lint', entitlementsPath]);
      proc.on('close', (code) => resolve(code === 0));
    });

    expect(result).toBe(true);
  });

  it('should have lipo available for universal builds', async () => {
    if (process.platform !== 'darwin') return;

    const result = await new Promise<boolean>((resolve) => {
      const proc = spawn('which', ['lipo']);
      proc.on('close', (code) => resolve(code === 0));
    });

    expect(result).toBe(true);
  });
});
```

---

## Related Specs

- Spec 494: Electron Packaging
- Spec 498: Code Signing
- Spec 493: Rust Compilation
- Spec 503: Release Workflow
