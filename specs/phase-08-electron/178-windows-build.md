# Spec 178: Windows Build

## Phase
8 - Electron Shell

## Spec ID
178

## Status
Planned

## Dependencies
- Spec 175 (Build Configuration)
- Spec 176 (Code Signing)

## Estimated Context
~8%

---

## Objective

Configure and optimize the Windows build process for Tachikoma, including NSIS installer creation, portable executable, Microsoft Store preparation, and proper Windows integration features.

---

## Acceptance Criteria

- [x] NSIS installer with custom UI
- [x] Portable executable option
- [x] Microsoft Store (MSIX) package
- [x] Authenticode code signing
- [x] Windows Defender SmartScreen compatibility
- [x] Start menu and desktop shortcuts
- [x] File associations and protocol handlers
- [x] Windows notification integration
- [x] Taskbar jump list support
- [x] ARM64 Windows support

---

## Implementation Details

### Windows-Specific Electron Builder Config

```typescript
// electron-builder.win.config.ts
import type { Configuration, WindowsConfiguration, NsisOptions } from 'electron-builder';

export const winConfig: WindowsConfiguration = {
  target: [
    {
      target: 'nsis',
      arch: ['x64', 'arm64'],
    },
    {
      target: 'portable',
      arch: ['x64'],
    },
    {
      target: 'appx',
      arch: ['x64', 'arm64'],
    },
  ],

  // Icon
  icon: 'build/icon.ico',

  // Publisher
  publisherName: 'Tachikoma Team',

  // Signing
  signAndEditExecutable: true,
  signDlls: true,
  verifyUpdateCodeSignature: true,

  // Certificate
  certificateFile: process.env.WIN_CSC_LINK,
  certificatePassword: process.env.WIN_CSC_KEY_PASSWORD,
  certificateSubjectName: 'Tachikoma Team',

  // Timestamp server
  timeStampServer: 'http://timestamp.digicert.com',
  rfc3161TimeStampServer: 'http://timestamp.digicert.com',

  // Request elevation
  requestedExecutionLevel: 'asInvoker',

  // Extra files
  extraFiles: [
    {
      from: 'build/windows',
      to: '.',
      filter: ['**/*'],
    },
  ],

  // File associations
  fileAssociations: [
    {
      ext: 'tachi',
      name: 'Tachikoma Project',
      description: 'Tachikoma Project File',
      icon: 'build/file-icon.ico',
      role: 'Editor',
    },
  ],

  // Protocol handlers (URL schemes)
  protocols: [
    {
      name: 'Tachikoma',
      schemes: ['tachikoma'],
    },
  ],
};

export const nsisConfig: NsisOptions = {
  // Installer type
  oneClick: false,
  perMachine: false,
  allowToChangeInstallationDirectory: true,
  allowElevation: true,

  // Shortcuts
  createDesktopShortcut: true,
  createStartMenuShortcut: true,
  shortcutName: 'Tachikoma',

  // Uninstall
  uninstallDisplayName: '${productName}',
  deleteAppDataOnUninstall: false,

  // Icons
  installerIcon: 'build/installer.ico',
  uninstallerIcon: 'build/uninstaller.ico',
  installerHeaderIcon: 'build/installer-header.ico',

  // License
  license: 'LICENSE',

  // Custom scripts
  include: 'build/windows/installer.nsh',
  script: 'build/windows/installer-script.nsi',

  // UI
  installerSidebar: 'build/windows/installer-sidebar.bmp',
  uninstallerSidebar: 'build/windows/uninstaller-sidebar.bmp',

  // Language
  installerLanguages: ['en_US'],

  // Run after install
  runAfterFinish: true,

  // Display name in Add/Remove Programs
  displayLanguageSelector: false,

  // Multi-language
  unicode: true,

  // Warnings
  warningsAsErrors: false,

  // Compression
  differentialPackage: true,
};

export const portableConfig = {
  artifactName: '${productName}-${version}-portable.${ext}',
  requestExecutionLevel: 'user',
};

export const appxConfig = {
  applicationId: 'Tachikoma',
  identityName: 'TachikomaTeam.Tachikoma',
  publisher: 'CN=Tachikoma Team',
  publisherDisplayName: 'Tachikoma Team',
  displayName: 'Tachikoma',
  languages: ['en-US'],
  backgroundColor: '#1a1a1a',
  showNameOnTiles: true,
};
```

### Custom NSIS Script

```nsis
; build/windows/installer.nsh

!macro customHeader
  !system "echo Custom NSIS header"
!macroend

!macro preInit
  ; Set installation directory based on architecture
  ${If} ${RunningX64}
    StrCpy $INSTDIR "$PROGRAMFILES64\${PRODUCT_NAME}"
  ${Else}
    StrCpy $INSTDIR "$PROGRAMFILES\${PRODUCT_NAME}"
  ${EndIf}
!macroend

!macro customInit
  ; Check for running instance
  System::Call 'kernel32::CreateMutex(p 0, i 0, t "TachikomaMutex") p .r1 ?e'
  Pop $R0
  ${If} $R0 != 0
    MessageBox MB_OK|MB_ICONEXCLAMATION "Tachikoma is already running. Please close it before installing." /SD IDOK
    Abort
  ${EndIf}
!macroend

!macro customInstall
  ; Create registry entries for file associations
  WriteRegStr SHCTX "Software\Classes\.tachi" "" "TachikomaProject"
  WriteRegStr SHCTX "Software\Classes\.tachi" "Content Type" "application/x-tachikoma"
  WriteRegStr SHCTX "Software\Classes\TachikomaProject" "" "Tachikoma Project"
  WriteRegStr SHCTX "Software\Classes\TachikomaProject\DefaultIcon" "" "$INSTDIR\${APP_EXECUTABLE_FILENAME},0"
  WriteRegStr SHCTX "Software\Classes\TachikomaProject\shell" "" "open"
  WriteRegStr SHCTX "Software\Classes\TachikomaProject\shell\open\command" "" '"$INSTDIR\${APP_EXECUTABLE_FILENAME}" "%1"'

  ; Register protocol handler
  WriteRegStr SHCTX "Software\Classes\tachikoma" "" "URL:Tachikoma Protocol"
  WriteRegStr SHCTX "Software\Classes\tachikoma" "URL Protocol" ""
  WriteRegStr SHCTX "Software\Classes\tachikoma\DefaultIcon" "" "$INSTDIR\${APP_EXECUTABLE_FILENAME},1"
  WriteRegStr SHCTX "Software\Classes\tachikoma\shell" "" "open"
  WriteRegStr SHCTX "Software\Classes\tachikoma\shell\open\command" "" '"$INSTDIR\${APP_EXECUTABLE_FILENAME}" "%1"'

  ; Register application capabilities
  WriteRegStr SHCTX "Software\RegisteredApplications" "Tachikoma" "Software\Tachikoma\Capabilities"
  WriteRegStr SHCTX "Software\Tachikoma\Capabilities" "ApplicationDescription" "Modern development environment"
  WriteRegStr SHCTX "Software\Tachikoma\Capabilities" "ApplicationName" "Tachikoma"
  WriteRegStr SHCTX "Software\Tachikoma\Capabilities\FileAssociations" ".tachi" "TachikomaProject"
  WriteRegStr SHCTX "Software\Tachikoma\Capabilities\UrlAssociations" "tachikoma" "TachikomaProject"

  ; Add to App Paths for command line access
  WriteRegStr SHCTX "Software\Microsoft\Windows\CurrentVersion\App Paths\tachikoma.exe" "" "$INSTDIR\${APP_EXECUTABLE_FILENAME}"
  WriteRegStr SHCTX "Software\Microsoft\Windows\CurrentVersion\App Paths\tachikoma.exe" "Path" "$INSTDIR"

  ; Refresh shell icons
  System::Call 'Shell32::SHChangeNotify(i 0x8000000, i 0, p 0, p 0)'
!macroend

!macro customUnInstall
  ; Remove registry entries
  DeleteRegKey SHCTX "Software\Classes\.tachi"
  DeleteRegKey SHCTX "Software\Classes\TachikomaProject"
  DeleteRegKey SHCTX "Software\Classes\tachikoma"
  DeleteRegKey SHCTX "Software\Tachikoma"
  DeleteRegValue SHCTX "Software\RegisteredApplications" "Tachikoma"
  DeleteRegKey SHCTX "Software\Microsoft\Windows\CurrentVersion\App Paths\tachikoma.exe"

  ; Refresh shell icons
  System::Call 'Shell32::SHChangeNotify(i 0x8000000, i 0, p 0, p 0)'
!macroend

!macro customInstallMode
  ; Set default install mode
  StrCpy $isForceCurrentInstall "1"
!macroend
```

### Jump List Integration

```typescript
// src/electron/main/windows/jumplist.ts
import { app, JumpListCategory } from 'electron';

interface RecentProject {
  path: string;
  name: string;
  lastOpened: Date;
}

export function setupJumpList(recentProjects: RecentProject[] = []): void {
  if (process.platform !== 'win32') {
    return;
  }

  const tasks: JumpListCategory = {
    type: 'tasks',
    items: [
      {
        type: 'task',
        title: 'New Project',
        description: 'Create a new Tachikoma project',
        program: process.execPath,
        args: '--new-project',
        iconPath: process.execPath,
        iconIndex: 0,
      },
      {
        type: 'task',
        title: 'Open Project',
        description: 'Open an existing project',
        program: process.execPath,
        args: '--open-project',
        iconPath: process.execPath,
        iconIndex: 0,
      },
    ],
  };

  const recent: JumpListCategory = {
    type: 'custom',
    name: 'Recent Projects',
    items: recentProjects.slice(0, 10).map((project) => ({
      type: 'task' as const,
      title: project.name,
      description: project.path,
      program: process.execPath,
      args: `"${project.path}"`,
      iconPath: process.execPath,
      iconIndex: 0,
    })),
  };

  try {
    app.setJumpList([tasks, recent]);
  } catch (error) {
    console.error('Failed to set jump list:', error);
  }
}

export function clearJumpList(): void {
  if (process.platform !== 'win32') {
    return;
  }

  try {
    app.setJumpList([]);
  } catch (error) {
    console.error('Failed to clear jump list:', error);
  }
}

export function addToRecentDocuments(path: string): void {
  if (process.platform !== 'win32') {
    return;
  }

  app.addRecentDocument(path);
}

export function clearRecentDocuments(): void {
  app.clearRecentDocuments();
}
```

### Taskbar Progress and Overlay

```typescript
// src/electron/main/windows/taskbar.ts
import { BrowserWindow, nativeImage } from 'electron';

type ProgressMode = 'none' | 'normal' | 'indeterminate' | 'error' | 'paused';

export function setTaskbarProgress(
  window: BrowserWindow,
  progress: number,
  mode: ProgressMode = 'normal'
): void {
  if (process.platform !== 'win32') {
    return;
  }

  switch (mode) {
    case 'none':
      window.setProgressBar(-1);
      break;
    case 'indeterminate':
      window.setProgressBar(2, { mode: 'indeterminate' });
      break;
    case 'error':
      window.setProgressBar(progress, { mode: 'error' });
      break;
    case 'paused':
      window.setProgressBar(progress, { mode: 'paused' });
      break;
    default:
      window.setProgressBar(progress);
  }
}

export function setTaskbarOverlay(
  window: BrowserWindow,
  overlayPath: string | null,
  description: string = ''
): void {
  if (process.platform !== 'win32') {
    return;
  }

  if (overlayPath) {
    const overlay = nativeImage.createFromPath(overlayPath);
    window.setOverlayIcon(overlay, description);
  } else {
    window.setOverlayIcon(null, '');
  }
}

export function flashTaskbar(window: BrowserWindow, flash: boolean = true): void {
  if (process.platform !== 'win32') {
    return;
  }

  window.flashFrame(flash);
}

export function setThumbarButtons(
  window: BrowserWindow,
  buttons: Electron.ThumbarButton[]
): void {
  if (process.platform !== 'win32') {
    return;
  }

  window.setThumbarButtons(buttons);
}

// Create thumbnail toolbar for media controls
export function setupMediaThumbar(
  window: BrowserWindow,
  callbacks: {
    onPrevious: () => void;
    onPlayPause: () => void;
    onNext: () => void;
  }
): void {
  const buttons: Electron.ThumbarButton[] = [
    {
      tooltip: 'Previous',
      icon: nativeImage.createFromPath('build/windows/thumbar-prev.png'),
      click: callbacks.onPrevious,
    },
    {
      tooltip: 'Play/Pause',
      icon: nativeImage.createFromPath('build/windows/thumbar-play.png'),
      click: callbacks.onPlayPause,
    },
    {
      tooltip: 'Next',
      icon: nativeImage.createFromPath('build/windows/thumbar-next.png'),
      click: callbacks.onNext,
    },
  ];

  setThumbarButtons(window, buttons);
}
```

### Windows Notifications

```typescript
// src/electron/main/windows/notifications.ts
import { Notification, nativeImage, app } from 'electron';

interface ToastOptions {
  title: string;
  body: string;
  icon?: string;
  silent?: boolean;
  actions?: Array<{ type: 'button'; text: string }>;
  urgency?: 'normal' | 'critical' | 'low';
  timeoutType?: 'default' | 'never';
}

export function showToast(options: ToastOptions): Notification {
  const notification = new Notification({
    title: options.title,
    body: options.body,
    icon: options.icon ? nativeImage.createFromPath(options.icon) : undefined,
    silent: options.silent,
    urgency: options.urgency,
    timeoutType: options.timeoutType,
    actions: options.actions,
    toastXml: buildToastXml(options),
  });

  notification.show();
  return notification;
}

function buildToastXml(options: ToastOptions): string | undefined {
  if (process.platform !== 'win32') {
    return undefined;
  }

  // Build Windows-specific toast XML for advanced features
  const actions = options.actions
    ?.map(
      (action, index) =>
        `<action content="${action.text}" arguments="action=${index}" />`
    )
    .join('');

  return `
    <toast>
      <visual>
        <binding template="ToastGeneric">
          <text>${escapeXml(options.title)}</text>
          <text>${escapeXml(options.body)}</text>
        </binding>
      </visual>
      ${actions ? `<actions>${actions}</actions>` : ''}
    </toast>
  `;
}

function escapeXml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&apos;');
}

// Set app user model ID for notifications
export function setAppUserModelId(): void {
  if (process.platform === 'win32') {
    app.setAppUserModelId('io.tachikoma.app');
  }
}
```

---

## Testing Requirements

### Windows Build Tests

```typescript
// scripts/test-win-build.ts
import { execSync } from 'child_process';
import { existsSync } from 'fs';

interface TestResult {
  name: string;
  passed: boolean;
  message: string;
}

const results: TestResult[] = [];

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

const EXE_PATH = 'release/Tachikoma Setup.exe';
const PORTABLE_PATH = 'release/Tachikoma-portable.exe';

test('Installer exists', () => existsSync(EXE_PATH));
test('Portable exists', () => existsSync(PORTABLE_PATH));

test('Authenticode signature', () => {
  const output = execSync(
    `powershell -Command "Get-AuthenticodeSignature '${EXE_PATH}' | Select-Object -ExpandProperty Status"`,
    { encoding: 'utf-8' }
  );
  return output.trim() === 'Valid';
});

test('Version info', () => {
  const output = execSync(
    `powershell -Command "(Get-Item '${EXE_PATH}').VersionInfo.FileVersion"`,
    { encoding: 'utf-8' }
  );
  return output.trim();
});

console.log('\nWindows Build Test Results:');
results.forEach((r) => console.log(`${r.passed ? '✓' : '✗'} ${r.name}: ${r.message}`));
```

---

## Related Specs

- Spec 175: Build Configuration
- Spec 176: Code Signing
- Spec 167: Auto Updates
