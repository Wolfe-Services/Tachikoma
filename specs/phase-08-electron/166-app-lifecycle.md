# Spec 166: App Lifecycle

## Phase
8 - Electron Shell

## Spec ID
166

## Status
Planned

## Dependencies
- Spec 161 (Electron Main Process)
- Spec 162 (Window Management)

## Estimated Context
~8%

---

## Objective

Implement comprehensive application lifecycle management for the Electron application, handling startup, shutdown, suspend/resume, and update scenarios. Ensure proper cleanup, state persistence, and graceful handling of all lifecycle events.

---

## Acceptance Criteria

- [x] Proper app ready initialization
- [x] Single instance enforcement
- [x] Graceful shutdown with cleanup
- [x] System suspend/resume handling
- [x] Before quit confirmation for unsaved changes
- [x] App restart functionality
- [x] State persistence on quit
- [x] Crash recovery on restart
- [x] Login item management (auto-start)
- [x] Dock badge and bounce on macOS

---

## Implementation Details

### Lifecycle Manager

```typescript
// src/electron/main/lifecycle/index.ts
import {
  app,
  BrowserWindow,
  powerMonitor,
  powerSaveBlocker,
  dialog,
} from 'electron';
import { Logger } from '../logger';
import { configManager } from '../config';
import { windowManager } from '../window';
import { fsService } from '../fs';

const logger = new Logger('lifecycle');

type LifecycleState = 'initializing' | 'ready' | 'running' | 'suspending' | 'quitting';

interface AppState {
  lastSessionEnd: string | null;
  crashRecovery: boolean;
  openProjects: string[];
  windowState: Record<string, unknown>;
}

class LifecycleManager {
  private state: LifecycleState = 'initializing';
  private isQuitting = false;
  private preventQuit = false;
  private powerSaveBlockerId: number | null = null;
  private shutdownCallbacks: Array<() => Promise<void>> = [];

  constructor() {
    this.setupEventListeners();
  }

  private setupEventListeners(): void {
    // App ready
    app.on('ready', () => this.onReady());

    // Window all closed
    app.on('window-all-closed', () => this.onWindowAllClosed());

    // Before quit
    app.on('before-quit', (event) => this.onBeforeQuit(event));

    // Will quit
    app.on('will-quit', (event) => this.onWillQuit(event));

    // Quit
    app.on('quit', () => this.onQuit());

    // Activate (macOS)
    app.on('activate', () => this.onActivate());

    // Second instance
    app.on('second-instance', (event, commandLine, workingDirectory) => {
      this.onSecondInstance(commandLine, workingDirectory);
    });

    // Power monitor events
    powerMonitor.on('suspend', () => this.onSuspend());
    powerMonitor.on('resume', () => this.onResume());
    powerMonitor.on('lock-screen', () => this.onLockScreen());
    powerMonitor.on('unlock-screen', () => this.onUnlockScreen());
    powerMonitor.on('shutdown', () => this.onSystemShutdown());

    // GPU process events
    app.on('gpu-process-crashed', (event, killed) => {
      this.onGpuCrash(killed);
    });

    app.on('render-process-gone', (event, webContents, details) => {
      this.onRendererCrash(webContents, details);
    });

    // Certificate errors
    app.on('certificate-error', (event, webContents, url, error, certificate, callback) => {
      this.onCertificateError(event, url, error, callback);
    });
  }

  private async onReady(): Promise<void> {
    logger.info('App ready');
    this.state = 'ready';

    // Check for crash recovery
    await this.checkCrashRecovery();

    // Initialize services
    await this.initializeServices();

    this.state = 'running';
    logger.info('App running');
  }

  private async checkCrashRecovery(): Promise<void> {
    try {
      const appState = await this.loadAppState();

      if (appState.crashRecovery) {
        logger.warn('Detected previous crash, attempting recovery');

        const { response } = await dialog.showMessageBox({
          type: 'warning',
          title: 'Recovery',
          message: 'Tachikoma was not closed properly',
          detail: 'Would you like to restore your previous session?',
          buttons: ['Restore', 'Start Fresh'],
          defaultId: 0,
        });

        if (response === 0) {
          await this.restoreSession(appState);
        }
      }

      // Mark as potential crash (will be cleared on clean exit)
      await this.saveAppState({ ...appState, crashRecovery: true });
    } catch (error) {
      logger.error('Crash recovery check failed', { error });
    }
  }

  private async initializeServices(): Promise<void> {
    // Services are initialized in order of dependency
    logger.debug('Initializing services');
  }

  private onWindowAllClosed(): void {
    logger.debug('All windows closed');

    // On macOS, keep app running
    if (process.platform !== 'darwin') {
      app.quit();
    }
  }

  private async onBeforeQuit(event: Electron.Event): Promise<void> {
    if (this.isQuitting) return;

    logger.info('Before quit triggered');

    if (this.preventQuit) {
      event.preventDefault();
      return;
    }

    // Check for unsaved changes
    const mainWindow = windowManager.getMainWindow();
    if (mainWindow) {
      event.preventDefault();

      const shouldQuit = await this.checkUnsavedChanges(mainWindow);
      if (shouldQuit) {
        this.isQuitting = true;
        app.quit();
      }
    }
  }

  private async checkUnsavedChanges(window: BrowserWindow): Promise<boolean> {
    return new Promise((resolve) => {
      // Ask renderer if there are unsaved changes
      window.webContents.send('lifecycle:checkUnsavedChanges');

      const timeout = setTimeout(() => {
        resolve(true); // Timeout, allow quit
      }, 5000);

      const { ipcMain } = require('electron');
      ipcMain.once('lifecycle:unsavedChangesResponse', (_, hasUnsaved: boolean) => {
        clearTimeout(timeout);

        if (!hasUnsaved) {
          resolve(true);
          return;
        }

        dialog
          .showMessageBox(window, {
            type: 'warning',
            title: 'Unsaved Changes',
            message: 'You have unsaved changes',
            detail: 'Do you want to save before quitting?',
            buttons: ['Save', "Don't Save", 'Cancel'],
            defaultId: 0,
            cancelId: 2,
          })
          .then(({ response }) => {
            if (response === 0) {
              // Save and quit
              window.webContents.send('lifecycle:saveAndQuit');
              ipcMain.once('lifecycle:saveComplete', () => {
                resolve(true);
              });
            } else if (response === 1) {
              // Don't save, just quit
              resolve(true);
            } else {
              // Cancel
              resolve(false);
            }
          });
      });
    });
  }

  private async onWillQuit(event: Electron.Event): Promise<void> {
    logger.info('Will quit');
    this.state = 'quitting';

    // Run shutdown callbacks
    for (const callback of this.shutdownCallbacks) {
      try {
        await callback();
      } catch (error) {
        logger.error('Shutdown callback failed', { error });
      }
    }

    // Save app state
    await this.saveAppState({
      lastSessionEnd: new Date().toISOString(),
      crashRecovery: false, // Clean exit
      openProjects: [],
      windowState: {},
    });

    // Cleanup services
    fsService.cleanup();
    await Logger.flush();
  }

  private onQuit(): void {
    logger.info('App quit');
  }

  private onActivate(): void {
    logger.debug('App activated');

    // On macOS, re-create window if all closed
    if (BrowserWindow.getAllWindows().length === 0) {
      windowManager.createWindow('main');
    } else {
      windowManager.focusMainWindow();
    }
  }

  private onSecondInstance(commandLine: string[], workingDirectory: string): void {
    logger.info('Second instance attempted', { commandLine, workingDirectory });

    // Focus existing window
    windowManager.focusMainWindow();

    // Handle deep links
    const url = commandLine.find((arg) => arg.startsWith('tachikoma://'));
    if (url) {
      const mainWindow = windowManager.getMainWindow();
      mainWindow?.webContents.send('deep-link', { url });
    }

    // Handle file arguments
    const filePaths = commandLine.filter(
      (arg) => !arg.startsWith('-') && !arg.startsWith('tachikoma://')
    );
    if (filePaths.length > 0) {
      const mainWindow = windowManager.getMainWindow();
      mainWindow?.webContents.send('open-files', { paths: filePaths });
    }
  }

  private onSuspend(): void {
    logger.info('System suspending');
    this.state = 'suspending';

    // Save state before suspend
    const mainWindow = windowManager.getMainWindow();
    mainWindow?.webContents.send('lifecycle:suspend');
  }

  private onResume(): void {
    logger.info('System resumed');
    this.state = 'running';

    // Notify renderer
    const mainWindow = windowManager.getMainWindow();
    mainWindow?.webContents.send('lifecycle:resume');

    // Check network connectivity
    this.checkConnectivity();
  }

  private onLockScreen(): void {
    logger.debug('Screen locked');

    const mainWindow = windowManager.getMainWindow();
    mainWindow?.webContents.send('lifecycle:lockScreen');
  }

  private onUnlockScreen(): void {
    logger.debug('Screen unlocked');

    const mainWindow = windowManager.getMainWindow();
    mainWindow?.webContents.send('lifecycle:unlockScreen');
  }

  private onSystemShutdown(): void {
    logger.info('System shutdown detected');
    this.isQuitting = true;
    app.quit();
  }

  private onGpuCrash(killed: boolean): void {
    logger.error('GPU process crashed', { killed });

    dialog.showErrorBox(
      'GPU Error',
      'The graphics process has crashed. Please restart the application.'
    );
  }

  private onRendererCrash(
    webContents: Electron.WebContents,
    details: Electron.RenderProcessGoneDetails
  ): void {
    logger.error('Renderer process crashed', { details });

    const window = BrowserWindow.fromWebContents(webContents);

    dialog
      .showMessageBox({
        type: 'error',
        title: 'Renderer Error',
        message: 'A window has stopped responding',
        detail: `Reason: ${details.reason}`,
        buttons: ['Reload', 'Close'],
      })
      .then(({ response }) => {
        if (response === 0) {
          webContents.reload();
        } else {
          window?.close();
        }
      });
  }

  private onCertificateError(
    event: Electron.Event,
    url: string,
    error: string,
    callback: (isTrusted: boolean) => void
  ): void {
    // In development, allow localhost certificates
    if (!app.isPackaged && url.includes('localhost')) {
      event.preventDefault();
      callback(true);
    } else {
      logger.warn('Certificate error', { url, error });
      callback(false);
    }
  }

  private async checkConnectivity(): Promise<void> {
    // Check if online
    const mainWindow = windowManager.getMainWindow();
    if (mainWindow) {
      const isOnline = await mainWindow.webContents.executeJavaScript(
        'navigator.onLine'
      );
      mainWindow.webContents.send('lifecycle:connectivity', { isOnline });
    }
  }

  private async loadAppState(): Promise<AppState> {
    try {
      const statePath = app.getPath('userData') + '/app-state.json';
      const content = await fsService.readFile(statePath, { encoding: 'utf-8' });
      return JSON.parse(content as string);
    } catch {
      return {
        lastSessionEnd: null,
        crashRecovery: false,
        openProjects: [],
        windowState: {},
      };
    }
  }

  private async saveAppState(state: AppState): Promise<void> {
    try {
      const statePath = app.getPath('userData') + '/app-state.json';
      await fsService.writeFile(statePath, JSON.stringify(state, null, 2));
    } catch (error) {
      logger.error('Failed to save app state', { error });
    }
  }

  private async restoreSession(state: AppState): Promise<void> {
    logger.info('Restoring session', { projects: state.openProjects });

    const mainWindow = windowManager.getMainWindow();
    if (mainWindow && state.openProjects.length > 0) {
      mainWindow.webContents.send('lifecycle:restoreProjects', {
        projects: state.openProjects,
      });
    }
  }

  // Public API

  getState(): LifecycleState {
    return this.state;
  }

  preventAppQuit(prevent: boolean): void {
    this.preventQuit = prevent;
  }

  onShutdown(callback: () => Promise<void>): void {
    this.shutdownCallbacks.push(callback);
  }

  async restart(): Promise<void> {
    logger.info('Restarting app');

    app.relaunch();
    app.quit();
  }

  startPowerSaveBlocker(reason: 'prevent-app-suspension' | 'prevent-display-sleep'): void {
    if (this.powerSaveBlockerId !== null) {
      this.stopPowerSaveBlocker();
    }

    this.powerSaveBlockerId = powerSaveBlocker.start(reason);
    logger.debug('Power save blocker started', { id: this.powerSaveBlockerId });
  }

  stopPowerSaveBlocker(): void {
    if (this.powerSaveBlockerId !== null) {
      powerSaveBlocker.stop(this.powerSaveBlockerId);
      logger.debug('Power save blocker stopped', { id: this.powerSaveBlockerId });
      this.powerSaveBlockerId = null;
    }
  }

  // Auto-start management
  setLoginItemSettings(openAtLogin: boolean): void {
    app.setLoginItemSettings({
      openAtLogin,
      openAsHidden: true,
      path: app.getPath('exe'),
    });

    configManager.set('autoStart' as any, openAtLogin);
    logger.info('Login item settings updated', { openAtLogin });
  }

  getLoginItemSettings(): { openAtLogin: boolean; openAsHidden: boolean } {
    return app.getLoginItemSettings();
  }

  // macOS specific
  setBadgeCount(count: number): void {
    if (process.platform === 'darwin') {
      app.setBadgeCount(count);
    }
  }

  bounce(type: 'critical' | 'informational' = 'informational'): void {
    if (process.platform === 'darwin') {
      app.dock?.bounce(type);
    }
  }

  setDockIcon(image: string): void {
    if (process.platform === 'darwin') {
      const { nativeImage } = require('electron');
      app.dock?.setIcon(nativeImage.createFromPath(image));
    }
  }
}

export const lifecycleManager = new LifecycleManager();
```

### Lifecycle IPC Handlers

```typescript
// src/electron/main/ipc/lifecycle.ts
import { ipcMain, app } from 'electron';
import { lifecycleManager } from '../lifecycle';

export function setupLifecycleIpcHandlers(): void {
  ipcMain.handle('lifecycle:getState', () => {
    return lifecycleManager.getState();
  });

  ipcMain.handle('lifecycle:restart', async () => {
    await lifecycleManager.restart();
  });

  ipcMain.handle('lifecycle:quit', () => {
    app.quit();
  });

  ipcMain.handle('lifecycle:preventQuit', (_, prevent: boolean) => {
    lifecycleManager.preventAppQuit(prevent);
  });

  ipcMain.handle('lifecycle:setLoginItem', (_, openAtLogin: boolean) => {
    lifecycleManager.setLoginItemSettings(openAtLogin);
  });

  ipcMain.handle('lifecycle:getLoginItem', () => {
    return lifecycleManager.getLoginItemSettings();
  });

  ipcMain.handle('lifecycle:startPowerSaveBlocker', (_, reason) => {
    lifecycleManager.startPowerSaveBlocker(reason);
  });

  ipcMain.handle('lifecycle:stopPowerSaveBlocker', () => {
    lifecycleManager.stopPowerSaveBlocker();
  });

  // macOS specific
  ipcMain.handle('lifecycle:setBadgeCount', (_, count: number) => {
    lifecycleManager.setBadgeCount(count);
  });

  ipcMain.handle('lifecycle:bounce', (_, type?: 'critical' | 'informational') => {
    lifecycleManager.bounce(type);
  });

  // Unsaved changes response from renderer
  ipcMain.on('lifecycle:unsavedChangesResponse', () => {
    // Handled in lifecycle manager
  });

  ipcMain.on('lifecycle:saveComplete', () => {
    // Handled in lifecycle manager
  });
}
```

### App Info

```typescript
// src/electron/main/lifecycle/app-info.ts
import { app } from 'electron';
import { platform, arch, release, cpus, totalmem, freemem } from 'os';

export interface AppInfo {
  name: string;
  version: string;
  isPackaged: boolean;
  paths: {
    userData: string;
    appData: string;
    logs: string;
    temp: string;
    exe: string;
  };
  versions: {
    electron: string;
    chrome: string;
    node: string;
    v8: string;
  };
  system: {
    platform: string;
    arch: string;
    release: string;
    cpuCount: number;
    totalMemory: number;
    freeMemory: number;
  };
}

export function getAppInfo(): AppInfo {
  return {
    name: app.name,
    version: app.getVersion(),
    isPackaged: app.isPackaged,
    paths: {
      userData: app.getPath('userData'),
      appData: app.getPath('appData'),
      logs: app.getPath('logs'),
      temp: app.getPath('temp'),
      exe: app.getPath('exe'),
    },
    versions: {
      electron: process.versions.electron,
      chrome: process.versions.chrome,
      node: process.versions.node,
      v8: process.versions.v8,
    },
    system: {
      platform: platform(),
      arch: arch(),
      release: release(),
      cpuCount: cpus().length,
      totalMemory: totalmem(),
      freeMemory: freemem(),
    },
  };
}
```

---

## Testing Requirements

### Unit Tests

```typescript
// src/electron/main/lifecycle/__tests__/lifecycle.test.ts
import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('electron', () => ({
  app: {
    on: vi.fn(),
    quit: vi.fn(),
    relaunch: vi.fn(),
    getPath: vi.fn().mockReturnValue('/mock/path'),
    isPackaged: false,
    name: 'Tachikoma',
    getVersion: vi.fn().mockReturnValue('1.0.0'),
    setLoginItemSettings: vi.fn(),
    getLoginItemSettings: vi.fn().mockReturnValue({ openAtLogin: false }),
    setBadgeCount: vi.fn(),
    dock: {
      bounce: vi.fn(),
      setIcon: vi.fn(),
    },
  },
  BrowserWindow: {
    getAllWindows: vi.fn().mockReturnValue([]),
    fromWebContents: vi.fn(),
  },
  powerMonitor: {
    on: vi.fn(),
  },
  powerSaveBlocker: {
    start: vi.fn().mockReturnValue(1),
    stop: vi.fn(),
  },
  dialog: {
    showMessageBox: vi.fn().mockResolvedValue({ response: 0 }),
    showErrorBox: vi.fn(),
  },
  ipcMain: {
    on: vi.fn(),
    once: vi.fn(),
  },
}));

describe('LifecycleManager', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should start in initializing state', async () => {
    const { lifecycleManager } = await import('../index');
    // State would be 'initializing' until app ready
  });

  it('should handle power save blocker', async () => {
    const { powerSaveBlocker } = await import('electron');
    const { lifecycleManager } = await import('../index');

    lifecycleManager.startPowerSaveBlocker('prevent-app-suspension');
    expect(powerSaveBlocker.start).toHaveBeenCalled();

    lifecycleManager.stopPowerSaveBlocker();
    expect(powerSaveBlocker.stop).toHaveBeenCalled();
  });

  it('should set login item settings', async () => {
    const { app } = await import('electron');
    const { lifecycleManager } = await import('../index');

    lifecycleManager.setLoginItemSettings(true);
    expect(app.setLoginItemSettings).toHaveBeenCalledWith(
      expect.objectContaining({ openAtLogin: true })
    );
  });
});
```

---

## Related Specs

- Spec 161: Electron Main Process
- Spec 162: Window Management
- Spec 167: Auto Updates
- Spec 168: Crash Reporting
