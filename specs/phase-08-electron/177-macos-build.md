# Spec 177: macOS Build

## Phase
8 - Electron Shell

## Spec ID
177

## Status
Planned

## Dependencies
- Spec 175 (Build Configuration)
- Spec 176 (Code Signing)

## Estimated Context
~8%

---

## Objective

Configure and optimize the macOS build process for Tachikoma, including universal binary support (Intel and Apple Silicon), DMG creation, notarization, and Mac App Store preparation.

---

## Acceptance Criteria

- [ ] Universal binary (x64 + arm64) support
- [ ] DMG installer with custom design
- [ ] PKG installer for enterprise deployment
- [ ] Mac App Store build configuration
- [ ] Notarization with hardened runtime
- [ ] Touch Bar support
- [ ] Dock integration
- [ ] macOS-specific menu configuration
- [ ] Proper Info.plist configuration

---

## Implementation Details

### macOS-Specific Electron Builder Config

```typescript
// electron-builder.mac.config.ts
import type { Configuration, MacConfiguration, DmgOptions } from 'electron-builder';

export const macConfig: MacConfiguration = {
  target: [
    {
      target: 'dmg',
      arch: ['universal'],
    },
    {
      target: 'zip',
      arch: ['universal'],
    },
    {
      target: 'mas',
      arch: ['universal'],
    },
  ],

  // Application metadata
  category: 'public.app-category.developer-tools',
  type: 'distribution',

  // Icon
  icon: 'build/icon.icns',

  // Dark mode
  darkModeSupport: true,

  // Security
  hardenedRuntime: true,
  gatekeeperAssess: false,

  // Entitlements
  entitlements: 'build/entitlements.mac.plist',
  entitlementsInherit: 'build/entitlements.mac.inherit.plist',

  // Notarization
  notarize: {
    teamId: process.env.APPLE_TEAM_ID || '',
  },

  // Signing
  identity: process.env.APPLE_IDENTITY || 'Developer ID Application',

  // Bundle ID
  bundleVersion: process.env.BUILD_NUMBER || '1',

  // Extended Info.plist entries
  extendInfo: {
    CFBundleDocumentTypes: [
      {
        CFBundleTypeName: 'Tachikoma Project',
        CFBundleTypeRole: 'Editor',
        CFBundleTypeExtensions: ['tachi', 'tachikoma'],
        CFBundleTypeIconFile: 'file-icon.icns',
        LSHandlerRank: 'Owner',
        LSItemContentTypes: ['io.tachikoma.project'],
      },
    ],
    UTExportedTypeDeclarations: [
      {
        UTTypeIdentifier: 'io.tachikoma.project',
        UTTypeDescription: 'Tachikoma Project',
        UTTypeConformsTo: ['public.data', 'public.content'],
        UTTypeTagSpecification: {
          'public.filename-extension': ['tachi', 'tachikoma'],
          'public.mime-type': ['application/x-tachikoma'],
        },
      },
    ],
    NSMicrophoneUsageDescription: 'Tachikoma requires microphone access for voice input features.',
    NSCameraUsageDescription: 'Tachikoma requires camera access for video features.',
    NSAppleEventsUsageDescription: 'Tachikoma requires Apple Events access for automation.',
    NSDocumentsFolderUsageDescription: 'Tachikoma requires access to your Documents folder.',
    NSDesktopFolderUsageDescription: 'Tachikoma requires access to your Desktop.',
    NSDownloadsFolderUsageDescription: 'Tachikoma requires access to your Downloads folder.',
    LSMinimumSystemVersion: '10.15.0',
    NSSupportsAutomaticGraphicsSwitching: true,
    NSHighResolutionCapable: true,
  },

  // Universal binary options
  x64ArchFiles: '*',
  mergeASARs: true,

  // Binaries to sign
  binaries: [
    'Contents/Frameworks/Tachikoma Helper.app/Contents/MacOS/Tachikoma Helper',
    'Contents/Frameworks/Tachikoma Helper (GPU).app/Contents/MacOS/Tachikoma Helper (GPU)',
    'Contents/Frameworks/Tachikoma Helper (Renderer).app/Contents/MacOS/Tachikoma Helper (Renderer)',
  ],

  // Minimum OS version
  minimumSystemVersion: '10.15.0',

  // Extra files specific to macOS
  extraFiles: [
    {
      from: 'build/macOS',
      to: 'Resources',
      filter: ['**/*'],
    },
  ],
};

export const dmgConfig: DmgOptions = {
  // Window configuration
  window: {
    width: 540,
    height: 380,
  },

  // Contents layout
  contents: [
    {
      x: 130,
      y: 220,
      type: 'file',
    },
    {
      x: 410,
      y: 220,
      type: 'link',
      path: '/Applications',
    },
  ],

  // Background image
  background: 'build/dmg-background.png',
  backgroundColor: '#1a1a1a',

  // Icon settings
  icon: 'build/icon.icns',
  iconSize: 80,

  // Title
  title: '${productName} ${version}',

  // Format
  format: 'ULFO',

  // Sign the DMG
  sign: true,

  // Write protection
  writeUpdateInfo: true,
};

export const masConfig: MacConfiguration = {
  ...macConfig,
  target: [
    {
      target: 'mas',
      arch: ['universal'],
    },
  ],
  type: 'distribution',
  hardenedRuntime: false, // Not needed for MAS
  entitlements: 'build/entitlements.mas.plist',
  entitlementsInherit: 'build/entitlements.mas.inherit.plist',
  provisioningProfile: 'build/embedded.provisionprofile',
};
```

### macOS Entitlements (Distribution)

```xml
<!-- build/entitlements.mac.plist -->
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <!-- Hardened Runtime -->
    <key>com.apple.security.cs.allow-jit</key>
    <true/>
    <key>com.apple.security.cs.allow-unsigned-executable-memory</key>
    <true/>
    <key>com.apple.security.cs.disable-library-validation</key>
    <true/>
    <key>com.apple.security.cs.allow-dyld-environment-variables</key>
    <true/>

    <!-- App Sandbox disabled for direct distribution -->
    <key>com.apple.security.app-sandbox</key>
    <false/>

    <!-- Network -->
    <key>com.apple.security.network.client</key>
    <true/>
    <key>com.apple.security.network.server</key>
    <true/>

    <!-- Files -->
    <key>com.apple.security.files.user-selected.read-write</key>
    <true/>
    <key>com.apple.security.files.bookmarks.app-scope</key>
    <true/>
    <key>com.apple.security.files.bookmarks.document-scope</key>
    <true/>

    <!-- Devices -->
    <key>com.apple.security.device.audio-input</key>
    <true/>

    <!-- Automation -->
    <key>com.apple.security.automation.apple-events</key>
    <true/>
</dict>
</plist>
```

### Mac App Store Entitlements

```xml
<!-- build/entitlements.mas.plist -->
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <!-- App Sandbox required for MAS -->
    <key>com.apple.security.app-sandbox</key>
    <true/>

    <!-- App Groups for shared data -->
    <key>com.apple.security.application-groups</key>
    <array>
        <string>$(TeamIdentifierPrefix)io.tachikoma.app</string>
    </array>

    <!-- Network -->
    <key>com.apple.security.network.client</key>
    <true/>

    <!-- Files -->
    <key>com.apple.security.files.user-selected.read-write</key>
    <true/>
    <key>com.apple.security.files.bookmarks.app-scope</key>
    <true/>

    <!-- Temporary exception for Electron -->
    <key>com.apple.security.temporary-exception.mach-lookup.global-name</key>
    <array>
        <string>com.apple.accessibility.AXSpeechManager</string>
    </array>
</dict>
</plist>
```

### Touch Bar Integration

```typescript
// src/electron/main/touchbar/index.ts
import { TouchBar, BrowserWindow, nativeImage } from 'electron';

const { TouchBarButton, TouchBarSpacer, TouchBarSlider, TouchBarSegmentedControl } = TouchBar;

export function createTouchBar(window: BrowserWindow): TouchBar {
  const newProjectButton = new TouchBarButton({
    label: 'New Project',
    icon: nativeImage.createFromPath('build/touchbar/new.png'),
    click: () => {
      window.webContents.send('menu:newProject');
    },
  });

  const openButton = new TouchBarButton({
    label: 'Open',
    icon: nativeImage.createFromPath('build/touchbar/open.png'),
    click: () => {
      window.webContents.send('menu:open');
    },
  });

  const saveButton = new TouchBarButton({
    label: 'Save',
    icon: nativeImage.createFromPath('build/touchbar/save.png'),
    click: () => {
      window.webContents.send('menu:save');
    },
  });

  const runButton = new TouchBarButton({
    label: 'Run',
    backgroundColor: '#28a745',
    click: () => {
      window.webContents.send('action:run');
    },
  });

  const stopButton = new TouchBarButton({
    label: 'Stop',
    backgroundColor: '#dc3545',
    click: () => {
      window.webContents.send('action:stop');
    },
  });

  const viewSegments = new TouchBarSegmentedControl({
    segments: [
      { label: 'Code' },
      { label: 'Preview' },
      { label: 'Terminal' },
    ],
    change: (selectedIndex) => {
      window.webContents.send('view:change', selectedIndex);
    },
  });

  return new TouchBar({
    items: [
      newProjectButton,
      openButton,
      saveButton,
      new TouchBarSpacer({ size: 'large' }),
      runButton,
      stopButton,
      new TouchBarSpacer({ size: 'flexible' }),
      viewSegments,
    ],
  });
}

export function setupTouchBar(window: BrowserWindow): void {
  if (process.platform !== 'darwin') {
    return;
  }

  const touchBar = createTouchBar(window);
  window.setTouchBar(touchBar);
}
```

### Dock Integration

```typescript
// src/electron/main/dock/index.ts
import { app, Menu, nativeImage } from 'electron';

interface DockBounceOptions {
  type: 'critical' | 'informational';
}

export function setupDock(): void {
  if (process.platform !== 'darwin') {
    return;
  }

  // Set dock menu
  const dockMenu = Menu.buildFromTemplate([
    {
      label: 'New Project',
      click: () => {
        // Emit to renderer
      },
    },
    {
      label: 'Open Recent',
      submenu: [
        { label: 'No Recent Projects', enabled: false },
      ],
    },
  ]);

  app.dock.setMenu(dockMenu);
}

export function setBadgeCount(count: number): void {
  if (process.platform !== 'darwin') {
    return;
  }

  app.dock.setBadge(count > 0 ? String(count) : '');
}

export function bounce(options: DockBounceOptions = { type: 'informational' }): number {
  if (process.platform !== 'darwin') {
    return -1;
  }

  return app.dock.bounce(options.type);
}

export function cancelBounce(id: number): void {
  if (process.platform !== 'darwin') {
    return;
  }

  app.dock.cancelBounce(id);
}

export function setDockIcon(imagePath: string): void {
  if (process.platform !== 'darwin') {
    return;
  }

  const icon = nativeImage.createFromPath(imagePath);
  app.dock.setIcon(icon);
}

export function showDock(): void {
  if (process.platform !== 'darwin') {
    return;
  }

  app.dock.show();
}

export function hideDock(): void {
  if (process.platform !== 'darwin') {
    return;
  }

  app.dock.hide();
}

export function isVisible(): boolean {
  if (process.platform !== 'darwin') {
    return true;
  }

  return app.dock.isVisible();
}
```

### Universal Binary Build Script

```bash
#!/bin/bash
# scripts/build-mac-universal.sh

set -e

echo "Building universal macOS application..."

# Clean previous builds
rm -rf dist release

# Build the application
npm run build

# Build for x64
echo "Building x64..."
npm run build:mac -- --arch x64

# Build for arm64
echo "Building arm64..."
npm run build:mac -- --arch arm64

# Create universal binary
echo "Creating universal binary..."
npm run build:mac -- --universal

# Verify universal binary
APP_PATH="release/mac-universal/Tachikoma.app/Contents/MacOS/Tachikoma"
if file "$APP_PATH" | grep -q "universal"; then
    echo "Universal binary created successfully!"
    file "$APP_PATH"
else
    echo "Warning: Binary may not be universal"
    file "$APP_PATH"
fi

echo "Build complete!"
```

---

## Testing Requirements

### macOS Build Tests

```typescript
// scripts/test-mac-build.ts
import { execSync } from 'child_process';
import { existsSync } from 'fs';
import { join } from 'path';

interface TestResult {
  name: string;
  passed: boolean;
  message: string;
}

const results: TestResult[] = [];

function test(name: string, fn: () => boolean | string): void {
  try {
    const result = fn();
    const passed = result === true || typeof result === 'string';
    results.push({
      name,
      passed,
      message: typeof result === 'string' ? result : passed ? 'OK' : 'Failed',
    });
  } catch (error: any) {
    results.push({
      name,
      passed: false,
      message: error.message,
    });
  }
}

const APP_PATH = 'release/mac-universal/Tachikoma.app';

// Test app bundle exists
test('App bundle exists', () => existsSync(APP_PATH));

// Test code signature
test('Code signature valid', () => {
  execSync(`codesign --verify --deep --strict "${APP_PATH}"`, { stdio: 'pipe' });
  return true;
});

// Test notarization
test('Notarization status', () => {
  const output = execSync(`spctl --assess --type execute "${APP_PATH}" 2>&1`, {
    encoding: 'utf-8',
  });
  return output.includes('accepted') || output.includes('source=Notarized');
});

// Test universal binary
test('Universal binary', () => {
  const binaryPath = join(APP_PATH, 'Contents/MacOS/Tachikoma');
  const output = execSync(`file "${binaryPath}"`, { encoding: 'utf-8' });
  return output.includes('universal') || (output.includes('x86_64') && output.includes('arm64'));
});

// Test minimum OS version
test('Minimum OS version', () => {
  const plistPath = join(APP_PATH, 'Contents/Info.plist');
  const output = execSync(`/usr/libexec/PlistBuddy -c "Print :LSMinimumSystemVersion" "${plistPath}"`, {
    encoding: 'utf-8',
  });
  return output.trim();
});

// Print results
console.log('\nmacOS Build Test Results:');
console.log('='.repeat(50));
for (const result of results) {
  const status = result.passed ? '✓' : '✗';
  console.log(`${status} ${result.name}: ${result.message}`);
}

const passed = results.filter((r) => r.passed).length;
console.log(`\n${passed}/${results.length} tests passed`);
```

---

## Related Specs

- Spec 175: Build Configuration
- Spec 176: Code Signing
- Spec 167: Auto Updates
