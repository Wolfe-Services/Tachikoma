# 498 - Windows Installer

**Phase:** 23 - Build & Distribution
**Spec ID:** 498
**Status:** Planned
**Dependencies:** 494-electron-packaging
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Configure NSIS installer for Windows distribution with custom branding, installation options, and proper Windows integration including Start Menu, desktop shortcuts, and file associations.

---

## Acceptance Criteria

- [ ] NSIS installer with custom UI
- [ ] Installation directory selection
- [ ] Desktop and Start Menu shortcuts
- [ ] File associations registered
- [ ] Protocol handler registered
- [ ] Uninstaller included
- [ ] Silent installation supported

---

## Implementation Details

### 1. NSIS Configuration

Update `electron/electron-builder.config.js`:

```javascript
// Windows configuration
win: {
  icon: 'build/icon.ico',
  target: [
    {
      target: 'nsis',
      arch: ['x64'],
    },
  ],
  publisherName: 'Tachikoma',
  legalTrademarks: 'Tachikoma',

  // File associations
  fileAssociations: [
    {
      ext: 'tspec',
      name: 'Tachikoma Spec',
      description: 'Tachikoma Specification File',
      icon: 'build/file-icon.ico',
      role: 'Editor',
    },
  ],

  // Protocol handler
  protocols: [
    {
      name: 'Tachikoma URL',
      schemes: ['tachikoma'],
    },
  ],

  // Request admin elevation only if needed
  requestedExecutionLevel: 'asInvoker',
},

// NSIS installer configuration
nsis: {
  // Installation options
  oneClick: false,
  allowToChangeInstallationDirectory: true,
  allowElevation: true,
  perMachine: false,

  // Shortcuts
  createDesktopShortcut: true,
  createStartMenuShortcut: true,
  shortcutName: 'Tachikoma',
  menuCategory: 'Development',

  // Artifacts
  artifactName: '${productName}-Setup-${version}.${ext}',

  // Custom NSIS script
  include: 'build/installer.nsh',

  // UI customization
  installerIcon: 'build/icon.ico',
  uninstallerIcon: 'build/icon.ico',
  installerHeader: 'build/installer-header.bmp',
  installerSidebar: 'build/installer-sidebar.bmp',
  installerHeaderIcon: 'build/icon.ico',

  // Install modes
  packElevateHelper: true,

  // Uninstaller
  uninstallDisplayName: '${productName}',

  // License
  license: '../LICENSE',

  // Installer language
  language: '1033', // English

  // Multi-language support
  installerLanguages: ['en_US'],

  // Delete app data on uninstall (optional)
  deleteAppDataOnUninstall: false,

  // Run after installation
  runAfterFinish: true,

  // Warn on uninstall
  warningsAsErrors: false,

  // Display version
  displayLanguageSelector: false,
},
```

### 2. Custom NSIS Script

Create `electron/build/installer.nsh`:

```nsis
!macro customHeader
  ; Custom header - can add branding
  !define MUI_WELCOMEFINISHPAGE_BITMAP "installer-sidebar.bmp"
  !define MUI_HEADERIMAGE
  !define MUI_HEADERIMAGE_BITMAP "installer-header.bmp"
  !define MUI_HEADERIMAGE_RIGHT
!macroend

!macro customInit
  ; Check for previous installation
  ReadRegStr $0 HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\${UNINSTALL_APP_KEY}" "UninstallString"
  ${If} $0 != ""
    MessageBox MB_YESNO|MB_ICONQUESTION "A previous version of ${PRODUCT_NAME} is installed. Would you like to uninstall it first?" IDYES uninst IDNO done
    uninst:
      ExecWait '$0 /S'
    done:
  ${EndIf}
!macroend

!macro customInstall
  ; Register file association
  WriteRegStr HKCU "Software\Classes\.tspec" "" "TachikomaSpec"
  WriteRegStr HKCU "Software\Classes\TachikomaSpec" "" "Tachikoma Specification"
  WriteRegStr HKCU "Software\Classes\TachikomaSpec\DefaultIcon" "" "$INSTDIR\Tachikoma.exe,0"
  WriteRegStr HKCU "Software\Classes\TachikomaSpec\shell\open\command" "" '"$INSTDIR\Tachikoma.exe" "%1"'

  ; Register protocol handler
  WriteRegStr HKCU "Software\Classes\tachikoma" "" "URL:Tachikoma Protocol"
  WriteRegStr HKCU "Software\Classes\tachikoma" "URL Protocol" ""
  WriteRegStr HKCU "Software\Classes\tachikoma\DefaultIcon" "" "$INSTDIR\Tachikoma.exe,0"
  WriteRegStr HKCU "Software\Classes\tachikoma\shell\open\command" "" '"$INSTDIR\Tachikoma.exe" "%1"'

  ; Add to PATH (optional)
  ; EnVar::AddValue "PATH" "$INSTDIR"

  ; Notify shell of changes
  System::Call 'Shell32::SHChangeNotify(i 0x8000000, i 0, p 0, p 0)'
!macroend

!macro customUnInstall
  ; Remove file association
  DeleteRegKey HKCU "Software\Classes\.tspec"
  DeleteRegKey HKCU "Software\Classes\TachikomaSpec"

  ; Remove protocol handler
  DeleteRegKey HKCU "Software\Classes\tachikoma"

  ; Remove from PATH (if added)
  ; EnVar::DeleteValue "PATH" "$INSTDIR"

  ; Notify shell of changes
  System::Call 'Shell32::SHChangeNotify(i 0x8000000, i 0, p 0, p 0)'
!macroend

!macro customRemoveFiles
  ; Custom file removal (if needed)
  RMDir /r "$INSTDIR\resources"
  RMDir /r "$INSTDIR\locales"
!macroend
```

### 3. Installer Graphics

Required graphics files in `electron/build/`:

```
installer-header.bmp    - 150x57 pixels, Windows installer header
installer-sidebar.bmp   - 164x314 pixels, Welcome/Finish page sidebar
icon.ico               - Multi-resolution icon (16, 32, 48, 64, 128, 256)
file-icon.ico          - Icon for .tspec files
```

### 4. Build Script for Windows

Create `electron/scripts/build-windows.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

echo "Building Windows installer..."

# Ensure we're in the electron directory
cd "$(dirname "$0")/.."

# Build TypeScript
echo "Compiling TypeScript..."
npm run build

# Create NSIS installer
echo "Creating NSIS installer..."
npx electron-builder --win nsis

# List output
echo "Build complete! Artifacts:"
ls -la out/*.exe

echo "Done!"
```

### 5. Windows Build on CI

For building on non-Windows platforms:

Create `electron/scripts/build-windows-ci.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

echo "Building Windows installer (cross-compile)..."

# Install Wine for cross-compilation (if on Linux)
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo "Installing Wine..."
    sudo dpkg --add-architecture i386
    sudo apt-get update
    sudo apt-get install -y wine64 wine32
fi

# Build
npm run dist:win

echo "Windows build complete!"
```

### 6. MSI Alternative Configuration

For enterprise deployment, add MSI support:

```javascript
// In electron-builder.config.js
win: {
  target: [
    {
      target: 'nsis',
      arch: ['x64'],
    },
    {
      target: 'msi',
      arch: ['x64'],
    },
  ],
},

msi: {
  artifactName: '${productName}-${version}.${ext}',
  createDesktopShortcut: true,
  createStartMenuShortcut: true,
  perMachine: true,  // Install for all users
  runAfterFinish: false,  // MSI best practice
},
```

### 7. Portable Version

Add portable version for users who don't want to install:

```javascript
win: {
  target: [
    { target: 'nsis', arch: ['x64'] },
    { target: 'portable', arch: ['x64'] },
  ],
},

portable: {
  artifactName: '${productName}-Portable-${version}.${ext}',
  // Use app data in portable directory
  useAppIdAsSubfolder: true,
},
```

---

## Testing Requirements

1. Installer runs on clean Windows system
2. Installation to custom directory works
3. Shortcuts are created correctly
4. File associations work
5. Protocol handler opens app
6. Uninstaller removes all components

---

## Related Specs

- Depends on: [494-electron-packaging.md](494-electron-packaging.md)
- Next: [499-windows-signing.md](499-windows-signing.md)
- Related: [182-protocol-handlers.md](../phase-08-electron/182-protocol-handlers.md)
