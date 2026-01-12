# Spec 500: Windows Installer

## Phase
23 - Build/Package System

## Spec ID
500

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

Implement Windows-specific packaging including NSIS installer creation, portable executable generation, MSI installer support, and proper Windows integration including registry entries, file associations, and Start menu shortcuts.

---

## Acceptance Criteria

- [ ] NSIS installer with custom UI
- [ ] MSI installer for enterprise deployment
- [ ] Portable executable option
- [ ] Per-user and per-machine installation
- [ ] File association registration
- [ ] URL protocol handler registration
- [ ] Start menu and desktop shortcuts
- [ ] Uninstaller with clean removal
- [ ] Auto-update integration
- [ ] Windows Store package (optional)

---

## Implementation Details

### Windows Build Configuration (electron/build/win.config.ts)

```typescript
// electron/build/win.config.ts
import type { WindowsConfiguration } from 'electron-builder';

export const winConfig: WindowsConfiguration = {
  // Target formats
  target: [
    {
      target: 'nsis',
      arch: ['x64', 'arm64'],
    },
    {
      target: 'msi',
      arch: ['x64'],
    },
    {
      target: 'portable',
      arch: ['x64'],
    },
  ],

  // Application icon
  icon: 'resources/icon.ico',

  // Publisher name (must match certificate)
  publisherName: 'Tachikoma Team',

  // Signing
  sign: './scripts/signing/windows-sign.js',
  signingHashAlgorithms: ['sha256'],

  // Verify signature after signing
  verifyUpdateCodeSignature: true,

  // Certificate
  certificateFile: process.env.WINDOWS_CERTIFICATE_PATH,
  certificatePassword: process.env.WINDOWS_CERTIFICATE_PASSWORD,

  // Request elevation level
  requestedExecutionLevel: 'asInvoker',

  // File associations
  fileAssociations: [
    {
      ext: 'tkmission',
      name: 'Tachikoma Mission',
      description: 'Tachikoma Mission File',
      icon: 'resources/file-mission.ico',
      role: 'Editor',
    },
    {
      ext: 'tkspec',
      name: 'Tachikoma Spec',
      description: 'Tachikoma Specification File',
      icon: 'resources/file-spec.ico',
      role: 'Editor',
    },
  ],

  // Protocol registration
  protocols: [
    {
      name: 'Tachikoma',
      schemes: ['tachikoma'],
    },
  ],

  // Extra resources
  extraResources: [
    {
      from: '../target/x86_64-pc-windows-msvc/release/tachikoma.exe',
      to: 'bin/tachikoma.exe',
    },
  ],
};
```

### NSIS Configuration (electron/build/nsis.config.ts)

```typescript
// electron/build/nsis.config.ts
import type { NsisOptions } from 'electron-builder';

export const nsisConfig: NsisOptions = {
  // One-click installation
  oneClick: false,

  // Per-machine installation
  perMachine: false,

  // Allow custom installation directory
  allowToChangeInstallationDirectory: true,

  // Allow elevation
  allowElevation: true,

  // Custom installer icon
  installerIcon: 'resources/installer.ico',
  uninstallerIcon: 'resources/uninstaller.ico',
  installerHeaderIcon: 'resources/icon.ico',

  // Installer sidebar image (164x314)
  installerSidebar: 'resources/installer-sidebar.bmp',
  uninstallerSidebar: 'resources/installer-sidebar.bmp',

  // Create shortcuts
  createDesktopShortcut: true,
  createStartMenuShortcut: true,

  // Menu category
  menuCategory: true,

  // Shortcut name
  shortcutName: 'Tachikoma',

  // Include custom NSIS script
  include: 'resources/installer.nsh',

  // License file
  license: '../LICENSE',

  // Installer language
  language: 1033, // English

  // Multi-language support
  multiLanguageInstaller: true,

  // Installer languages
  installerLanguages: ['en_US', 'de_DE', 'fr_FR', 'ja_JP', 'zh_CN'],

  // Display language selection dialog
  displayLanguageSelector: true,

  // Uninstall display name
  uninstallDisplayName: '${productName} ${version}',

  // Delete app data on uninstall
  deleteAppDataOnUninstall: false,

  // Run after finish
  runAfterFinish: true,

  // Allow install to network drive
  allowToChangePerMachineInfo: true,

  // Warn on close
  warningsAsErrors: false,

  // Pack elevation helper
  packElevateHelper: true,

  // GUID
  guid: 'com.tachikoma.app',
};
```

### Custom NSIS Script (resources/installer.nsh)

```nsis
; resources/installer.nsh
; Custom NSIS script for Tachikoma installer

!include "MUI2.nsh"
!include "FileFunc.nsh"
!include "x64.nsh"

; Custom installer pages
!macro customHeader
  !system "echo '!define MUI_HEADERIMAGE_BITMAP resources\\header.bmp'"
!macroend

; Pre-install tasks
!macro preInit
  ; Check for running instances
  nsExec::ExecToStack 'tasklist /FI "IMAGENAME eq Tachikoma.exe" /FO CSV'
  Pop $0
  Pop $1
  ${If} $1 != ""
    StrCpy $1 $1 14
    ${If} $1 == '"Tachikoma.exe"'
      MessageBox MB_OKCANCEL|MB_ICONEXCLAMATION \
        "Tachikoma is currently running. Please close it before continuing." \
        IDOK closeApp IDCANCEL abortInstall
      closeApp:
        nsExec::Exec 'taskkill /F /IM Tachikoma.exe'
        Sleep 2000
        Goto done
      abortInstall:
        Abort
      done:
    ${EndIf}
  ${EndIf}
!macroend

; Custom uninstall tasks
!macro customUnInit
  ; Check for running instances before uninstall
  nsExec::ExecToStack 'tasklist /FI "IMAGENAME eq Tachikoma.exe" /FO CSV'
  Pop $0
  Pop $1
  ${If} $1 != ""
    StrCpy $1 $1 14
    ${If} $1 == '"Tachikoma.exe"'
      MessageBox MB_OKCANCEL|MB_ICONEXCLAMATION \
        "Tachikoma is currently running. Please close it before uninstalling." \
        IDOK closeAppUn IDCANCEL abortUninstall
      closeAppUn:
        nsExec::Exec 'taskkill /F /IM Tachikoma.exe'
        Sleep 2000
        Goto doneUn
      abortUninstall:
        Abort
      doneUn:
    ${EndIf}
  ${EndIf}
!macroend

; Custom install tasks
!macro customInstall
  ; Add to PATH (optional)
  ; EnVar::AddValue "PATH" "$INSTDIR\resources\bin"

  ; Register context menu handler
  WriteRegStr HKCR "*\shell\Open with Tachikoma" "" "Open with Tachikoma"
  WriteRegStr HKCR "*\shell\Open with Tachikoma" "Icon" "$INSTDIR\${APP_EXECUTABLE_FILENAME}"
  WriteRegStr HKCR "*\shell\Open with Tachikoma\command" "" '"$INSTDIR\${APP_EXECUTABLE_FILENAME}" "%1"'

  ; Register directory context menu
  WriteRegStr HKCR "Directory\shell\Open with Tachikoma" "" "Open with Tachikoma"
  WriteRegStr HKCR "Directory\shell\Open with Tachikoma" "Icon" "$INSTDIR\${APP_EXECUTABLE_FILENAME}"
  WriteRegStr HKCR "Directory\shell\Open with Tachikoma\command" "" '"$INSTDIR\${APP_EXECUTABLE_FILENAME}" "%1"'

  ; Register directory background context menu
  WriteRegStr HKCR "Directory\Background\shell\Open with Tachikoma" "" "Open with Tachikoma"
  WriteRegStr HKCR "Directory\Background\shell\Open with Tachikoma" "Icon" "$INSTDIR\${APP_EXECUTABLE_FILENAME}"
  WriteRegStr HKCR "Directory\Background\shell\Open with Tachikoma\command" "" '"$INSTDIR\${APP_EXECUTABLE_FILENAME}" "%V"'

  ; Write install info for size calculation
  ${GetSize} "$INSTDIR" "/S=0K" $0 $1 $2
  IntFmt $0 "0x%08X" $0
  WriteRegDWORD SHCTX "Software\Microsoft\Windows\CurrentVersion\Uninstall\${UNINSTALL_APP_KEY}" \
    "EstimatedSize" "$0"
!macroend

; Custom uninstall tasks
!macro customUnInstall
  ; Remove from PATH
  ; EnVar::DeleteValue "PATH" "$INSTDIR\resources\bin"

  ; Remove context menu entries
  DeleteRegKey HKCR "*\shell\Open with Tachikoma"
  DeleteRegKey HKCR "Directory\shell\Open with Tachikoma"
  DeleteRegKey HKCR "Directory\Background\shell\Open with Tachikoma"

  ; Ask about removing app data
  MessageBox MB_YESNO "Would you like to remove your Tachikoma settings and data?" IDNO skipRemoveData
    RMDir /r "$APPDATA\Tachikoma"
    RMDir /r "$LOCALAPPDATA\Tachikoma"
  skipRemoveData:
!macroend
```

### MSI Configuration (electron/build/msi.config.ts)

```typescript
// electron/build/msi.config.ts
import type { MsiOptions } from 'electron-builder';

export const msiConfig: MsiOptions = {
  // One click
  oneClick: false,

  // Per machine
  perMachine: true,

  // Run after finish
  runAfterFinish: false,

  // Create desktop shortcut
  createDesktopShortcut: false,

  // Create start menu shortcut
  createStartMenuShortcut: true,

  // WiX upgrade code (must be consistent across versions)
  upgradeCode: 'YOUR-UPGRADE-CODE-GUID-HERE',

  // WiX options
  warningsAsErrors: false,
};
```

### Windows Sign Script (scripts/signing/windows-sign.js)

```javascript
// scripts/signing/windows-sign.js
const { execSync } = require('child_process');
const path = require('path');
const fs = require('fs');

/**
 * @param {import('electron-builder').CustomWindowsSignTaskConfiguration} configuration
 */
exports.default = async function sign(configuration) {
  const { path: filePath, hash, isNest } = configuration;

  // Skip if no certificate configured
  if (!process.env.WINDOWS_CERTIFICATE_PATH) {
    console.log('Skipping signing - no certificate configured');
    return;
  }

  // Skip if already signed (for nested signatures)
  if (isNest) {
    console.log(`Adding nested signature to ${filePath}`);
  }

  const certPath = process.env.WINDOWS_CERTIFICATE_PATH;
  const certPassword = process.env.WINDOWS_CERTIFICATE_PASSWORD;
  const timestampServer = 'http://timestamp.digicert.com';

  const args = [
    'sign',
    '/fd', hash || 'SHA256',
    '/f', certPath,
    '/p', certPassword,
    '/tr', timestampServer,
    '/td', hash || 'SHA256',
    '/d', 'Tachikoma',
    '/du', 'https://tachikoma.io',
  ];

  if (isNest) {
    args.push('/as'); // Append signature
  }

  args.push(filePath);

  console.log(`Signing ${filePath}...`);

  try {
    // Find signtool
    const signtoolPath = findSignTool();

    execSync(`"${signtoolPath}" ${args.join(' ')}`, {
      stdio: 'inherit',
    });

    console.log('Signing successful');
  } catch (error) {
    console.error('Signing failed:', error.message);
    throw error;
  }
};

function findSignTool() {
  // Common locations for signtool
  const locations = [
    'C:\\Program Files (x86)\\Windows Kits\\10\\bin\\x64\\signtool.exe',
    'C:\\Program Files (x86)\\Windows Kits\\10\\bin\\10.0.22621.0\\x64\\signtool.exe',
    'C:\\Program Files (x86)\\Windows Kits\\10\\bin\\10.0.19041.0\\x64\\signtool.exe',
    'C:\\Program Files (x86)\\Microsoft SDKs\\ClickOnce\\SignTool\\signtool.exe',
  ];

  // Check PATH first
  try {
    execSync('where signtool', { stdio: 'pipe' });
    return 'signtool';
  } catch {}

  // Check common locations
  for (const location of locations) {
    if (fs.existsSync(location)) {
      return location;
    }
  }

  throw new Error('signtool not found');
}
```

### Portable Configuration (electron/build/portable.config.ts)

```typescript
// electron/build/portable.config.ts
import type { PortableOptions } from 'electron-builder';

export const portableConfig: PortableOptions = {
  // Request elevation level
  requestExecutionLevel: 'user',

  // Unpack directory name
  unpackDirName: '${productName}',

  // Splash screen
  splashImage: 'resources/splash.bmp',

  // Use temp directory
  useTemp: true,
};
```

### Windows Store Configuration (electron/build/appx.config.ts)

```typescript
// electron/build/appx.config.ts
import type { AppXOptions } from 'electron-builder';

export const appxConfig: AppXOptions = {
  // Application ID
  applicationId: 'Tachikoma',

  // Publisher
  publisher: process.env.APPX_PUBLISHER || 'CN=TachikomaTeam',

  // Publisher display name
  publisherDisplayName: 'Tachikoma Team',

  // Display name
  displayName: 'Tachikoma',

  // Identity name
  identityName: process.env.APPX_IDENTITY_NAME || 'TachikomaTeam.Tachikoma',

  // Languages
  languages: ['en-US', 'de-DE', 'fr-FR', 'ja-JP', 'zh-CN'],

  // Add auto launch
  addAutoLaunchExtension: true,

  // Show name on tiles
  showNameOnTiles: true,

  // Background color
  backgroundColor: '#1a1a2e',

  // Set min version
  // minVersion: '10.0.17763.0',
};
```

---

## Testing Requirements

### Unit Tests

```typescript
// electron/build/__tests__/win.config.test.ts
import { describe, it, expect } from 'vitest';
import { winConfig, nsisConfig, msiConfig } from '../win.config';

describe('Windows Build Configuration', () => {
  it('should have valid target configuration', () => {
    expect(winConfig.target).toBeDefined();
    expect(Array.isArray(winConfig.target)).toBe(true);
  });

  it('should have NSIS target', () => {
    const nsisTarget = winConfig.target?.find(
      (t: any) => typeof t === 'object' && t.target === 'nsis'
    );
    expect(nsisTarget).toBeDefined();
  });

  it('should have file associations configured', () => {
    expect(winConfig.fileAssociations).toBeDefined();
    expect(winConfig.fileAssociations?.length).toBeGreaterThan(0);
  });

  it('should have protocol handler configured', () => {
    expect(winConfig.protocols).toBeDefined();
    expect(winConfig.protocols?.length).toBeGreaterThan(0);
  });
});

describe('NSIS Configuration', () => {
  it('should allow installation directory change', () => {
    expect(nsisConfig.allowToChangeInstallationDirectory).toBe(true);
  });

  it('should create shortcuts', () => {
    expect(nsisConfig.createDesktopShortcut).toBe(true);
    expect(nsisConfig.createStartMenuShortcut).toBe(true);
  });

  it('should have license file', () => {
    expect(nsisConfig.license).toBeDefined();
  });
});
```

### Integration Tests

```typescript
// electron/build/__tests__/win.integration.test.ts
import { describe, it, expect } from 'vitest';
import * as fs from 'fs';
import * as path from 'path';

describe('Windows Build Integration', () => {
  it('should have Windows icon file', () => {
    const iconPath = path.join(__dirname, '..', '..', 'resources', 'icon.ico');
    if (process.platform === 'win32') {
      expect(fs.existsSync(iconPath)).toBe(true);
    }
  });

  it('should have installer script', () => {
    const scriptPath = path.join(__dirname, '..', '..', 'resources', 'installer.nsh');
    expect(fs.existsSync(scriptPath)).toBe(true);
  });

  it('should have signing script', () => {
    const scriptPath = path.join(__dirname, '..', '..', '..', 'scripts', 'signing', 'windows-sign.js');
    expect(fs.existsSync(scriptPath)).toBe(true);
  });
});
```

---

## Related Specs

- Spec 494: Electron Packaging
- Spec 498: Code Signing
- Spec 493: Rust Compilation
- Spec 503: Release Workflow
