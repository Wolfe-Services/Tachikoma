# Spec 167: Auto Updates

## Phase
8 - Electron Shell

## Spec ID
167

## Status
Planned

## Dependencies
- Spec 161 (Electron Main Process)
- Spec 166 (App Lifecycle)
- Spec 176 (Code Signing)

## Estimated Context
~9%

---

## Objective

Implement automatic update functionality using electron-updater, supporting differential updates, update channels, and user-controlled update behavior. Ensure updates are secure, properly signed, and provide a good user experience.

---

## Acceptance Criteria

- [ ] Automatic update checking on app start
- [ ] Manual update check trigger
- [ ] Download progress indication
- [ ] Update ready notification
- [ ] Install on quit option
- [ ] Update channels (stable, beta, alpha)
- [ ] Differential/delta updates
- [ ] Rollback on failed update
- [ ] Update history tracking
- [ ] Offline handling

---

## Implementation Details

### Auto Updater Service

```typescript
// src/electron/main/updater/index.ts
import { autoUpdater, UpdateInfo, ProgressInfo } from 'electron-updater';
import { BrowserWindow, dialog, app } from 'electron';
import { Logger } from '../logger';
import { configManager } from '../config';

const logger = new Logger('updater');

type UpdateChannel = 'stable' | 'beta' | 'alpha';

interface UpdateState {
  checking: boolean;
  available: boolean;
  downloading: boolean;
  downloaded: boolean;
  error: Error | null;
  updateInfo: UpdateInfo | null;
  progress: ProgressInfo | null;
}

class AutoUpdaterService {
  private mainWindow: BrowserWindow | null = null;
  private state: UpdateState = {
    checking: false,
    available: false,
    downloading: false,
    downloaded: false,
    error: null,
    updateInfo: null,
    progress: null,
  };

  constructor() {
    this.configureUpdater();
    this.setupEventHandlers();
  }

  private configureUpdater(): void {
    // Configure auto-updater
    autoUpdater.autoDownload = false;
    autoUpdater.autoInstallOnAppQuit = true;
    autoUpdater.allowDowngrade = false;
    autoUpdater.allowPrerelease = this.getAllowPrerelease();

    // Set update feed URL (configure for your hosting)
    autoUpdater.setFeedURL({
      provider: 'github',
      owner: 'tachikoma',
      repo: 'tachikoma-desktop',
      private: false,
    });

    // Alternative: Generic server
    // autoUpdater.setFeedURL({
    //   provider: 'generic',
    //   url: 'https://updates.tachikoma.io',
    //   channel: this.getChannel(),
    // });

    logger.info('Auto updater configured', {
      channel: this.getChannel(),
      allowPrerelease: autoUpdater.allowPrerelease,
    });
  }

  private getAllowPrerelease(): boolean {
    const channel = this.getChannel();
    return channel === 'beta' || channel === 'alpha';
  }

  private getChannel(): UpdateChannel {
    return configManager.get('updateChannel' as any) || 'stable';
  }

  private setupEventHandlers(): void {
    autoUpdater.on('checking-for-update', () => {
      logger.info('Checking for updates');
      this.state.checking = true;
      this.notifyRenderer('update:checking');
    });

    autoUpdater.on('update-available', (info: UpdateInfo) => {
      logger.info('Update available', { version: info.version });
      this.state = {
        ...this.state,
        checking: false,
        available: true,
        updateInfo: info,
      };
      this.notifyRenderer('update:available', info);
      this.promptForDownload(info);
    });

    autoUpdater.on('update-not-available', (info: UpdateInfo) => {
      logger.info('No update available', { currentVersion: info.version });
      this.state = {
        ...this.state,
        checking: false,
        available: false,
        updateInfo: info,
      };
      this.notifyRenderer('update:not-available', info);
    });

    autoUpdater.on('download-progress', (progress: ProgressInfo) => {
      logger.debug('Download progress', {
        percent: progress.percent,
        bytesPerSecond: progress.bytesPerSecond,
      });
      this.state.progress = progress;
      this.notifyRenderer('update:progress', progress);

      // Update taskbar progress
      this.mainWindow?.setProgressBar(progress.percent / 100);
    });

    autoUpdater.on('update-downloaded', (info: UpdateInfo) => {
      logger.info('Update downloaded', { version: info.version });
      this.state = {
        ...this.state,
        downloading: false,
        downloaded: true,
        progress: null,
      };
      this.notifyRenderer('update:downloaded', info);

      // Reset taskbar progress
      this.mainWindow?.setProgressBar(-1);

      this.promptForInstall(info);
    });

    autoUpdater.on('error', (error: Error) => {
      logger.error('Update error', { error: error.message });
      this.state = {
        ...this.state,
        checking: false,
        downloading: false,
        error,
      };
      this.notifyRenderer('update:error', { message: error.message });

      // Reset taskbar progress
      this.mainWindow?.setProgressBar(-1);
    });
  }

  private notifyRenderer(channel: string, data?: unknown): void {
    if (this.mainWindow && !this.mainWindow.isDestroyed()) {
      this.mainWindow.webContents.send(channel, data);
    }
  }

  private async promptForDownload(info: UpdateInfo): Promise<void> {
    if (!this.mainWindow) return;

    const releaseNotes = this.formatReleaseNotes(info);

    const { response } = await dialog.showMessageBox(this.mainWindow, {
      type: 'info',
      title: 'Update Available',
      message: `A new version (${info.version}) is available`,
      detail: `Current version: ${app.getVersion()}\n\n${releaseNotes}`,
      buttons: ['Download', 'Later', 'Skip This Version'],
      defaultId: 0,
      cancelId: 1,
    });

    if (response === 0) {
      this.downloadUpdate();
    } else if (response === 2) {
      this.skipVersion(info.version);
    }
  }

  private async promptForInstall(info: UpdateInfo): Promise<void> {
    if (!this.mainWindow) return;

    const { response } = await dialog.showMessageBox(this.mainWindow, {
      type: 'info',
      title: 'Update Ready',
      message: `Version ${info.version} is ready to install`,
      detail: 'The application will restart to apply the update.',
      buttons: ['Restart Now', 'Later'],
      defaultId: 0,
    });

    if (response === 0) {
      this.installUpdate();
    }
  }

  private formatReleaseNotes(info: UpdateInfo): string {
    if (!info.releaseNotes) return '';

    if (typeof info.releaseNotes === 'string') {
      return info.releaseNotes.slice(0, 500);
    }

    // Handle array of release notes
    return info.releaseNotes
      .map((note) => (typeof note === 'string' ? note : note.note))
      .join('\n')
      .slice(0, 500);
  }

  private skipVersion(version: string): void {
    const skipped = configManager.get('skippedVersions' as any) || [];
    configManager.set('skippedVersions' as any, [...skipped, version]);
    logger.info('Skipped version', { version });
  }

  setMainWindow(window: BrowserWindow): void {
    this.mainWindow = window;
  }

  async checkForUpdates(silent = false): Promise<void> {
    if (this.state.checking || this.state.downloading) {
      logger.debug('Update check already in progress');
      return;
    }

    // Check if we should skip this version
    const skipped = configManager.get('skippedVersions' as any) || [];

    try {
      const result = await autoUpdater.checkForUpdates();

      if (result?.updateInfo && skipped.includes(result.updateInfo.version)) {
        logger.info('Skipping version', { version: result.updateInfo.version });
        this.state.available = false;
        return;
      }

      if (!result?.updateInfo && !silent) {
        dialog.showMessageBox(this.mainWindow!, {
          type: 'info',
          title: 'No Updates',
          message: 'You are running the latest version',
        });
      }
    } catch (error) {
      logger.error('Check for updates failed', { error });
      if (!silent) {
        dialog.showErrorBox('Update Error', `Failed to check for updates: ${error}`);
      }
    }
  }

  downloadUpdate(): void {
    if (this.state.downloading) {
      logger.debug('Download already in progress');
      return;
    }

    logger.info('Starting update download');
    this.state.downloading = true;
    autoUpdater.downloadUpdate();
  }

  installUpdate(): void {
    if (!this.state.downloaded) {
      logger.warn('No update downloaded to install');
      return;
    }

    logger.info('Installing update');
    autoUpdater.quitAndInstall(false, true);
  }

  setChannel(channel: UpdateChannel): void {
    configManager.set('updateChannel' as any, channel);
    autoUpdater.allowPrerelease = channel !== 'stable';
    autoUpdater.channel = channel;
    logger.info('Update channel changed', { channel });
  }

  getState(): UpdateState {
    return { ...this.state };
  }

  clearSkippedVersions(): void {
    configManager.set('skippedVersions' as any, []);
    logger.info('Cleared skipped versions');
  }
}

export const autoUpdaterService = new AutoUpdaterService();

export function setupAutoUpdater(mainWindow: BrowserWindow): void {
  autoUpdaterService.setMainWindow(mainWindow);

  // Check for updates on startup (silent)
  if (app.isPackaged) {
    setTimeout(() => {
      autoUpdaterService.checkForUpdates(true);
    }, 5000);

    // Check periodically (every 4 hours)
    setInterval(() => {
      autoUpdaterService.checkForUpdates(true);
    }, 4 * 60 * 60 * 1000);
  }
}
```

### Updater IPC Handlers

```typescript
// src/electron/main/ipc/updater.ts
import { ipcMain } from 'electron';
import { autoUpdaterService } from '../updater';

export function setupUpdaterIpcHandlers(): void {
  ipcMain.handle('updater:check', async (_, silent?: boolean) => {
    await autoUpdaterService.checkForUpdates(silent ?? false);
  });

  ipcMain.handle('updater:download', () => {
    autoUpdaterService.downloadUpdate();
  });

  ipcMain.handle('updater:install', () => {
    autoUpdaterService.installUpdate();
  });

  ipcMain.handle('updater:getState', () => {
    return autoUpdaterService.getState();
  });

  ipcMain.handle('updater:setChannel', (_, channel: 'stable' | 'beta' | 'alpha') => {
    autoUpdaterService.setChannel(channel);
  });

  ipcMain.handle('updater:clearSkipped', () => {
    autoUpdaterService.clearSkippedVersions();
  });
}
```

### Update UI Component (Renderer)

```typescript
// src/renderer/components/UpdateNotification/UpdateNotification.tsx
import React, { useEffect, useState } from 'react';
import styles from './UpdateNotification.module.css';

interface UpdateInfo {
  version: string;
  releaseDate: string;
  releaseNotes?: string;
}

interface ProgressInfo {
  percent: number;
  bytesPerSecond: number;
  total: number;
  transferred: number;
}

type UpdateStatus = 'idle' | 'checking' | 'available' | 'downloading' | 'ready' | 'error';

export const UpdateNotification: React.FC = () => {
  const [status, setStatus] = useState<UpdateStatus>('idle');
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  const [progress, setProgress] = useState<ProgressInfo | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [dismissed, setDismissed] = useState(false);

  useEffect(() => {
    const unsubscribers = [
      window.electronAPI?.onUpdateChecking(() => {
        setStatus('checking');
        setDismissed(false);
      }),
      window.electronAPI?.onUpdateAvailable((info: UpdateInfo) => {
        setStatus('available');
        setUpdateInfo(info);
      }),
      window.electronAPI?.onUpdateProgress((progressInfo: ProgressInfo) => {
        setStatus('downloading');
        setProgress(progressInfo);
      }),
      window.electronAPI?.onUpdateDownloaded((info: UpdateInfo) => {
        setStatus('ready');
        setUpdateInfo(info);
        setProgress(null);
      }),
      window.electronAPI?.onUpdateError((err: { message: string }) => {
        setStatus('error');
        setError(err.message);
      }),
    ];

    return () => {
      unsubscribers.forEach((unsub) => unsub?.());
    };
  }, []);

  const handleDownload = () => {
    window.electronAPI?.downloadUpdate();
  };

  const handleInstall = () => {
    window.electronAPI?.installUpdate();
  };

  const handleDismiss = () => {
    setDismissed(true);
  };

  if (dismissed || status === 'idle' || status === 'checking') {
    return null;
  }

  const formatBytes = (bytes: number): string => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  const formatSpeed = (bytesPerSecond: number): string => {
    return `${formatBytes(bytesPerSecond)}/s`;
  };

  return (
    <div className={styles.notification} data-status={status}>
      <div className={styles.content}>
        {status === 'available' && (
          <>
            <div className={styles.icon}>
              <UpdateIcon />
            </div>
            <div className={styles.text}>
              <strong>Update Available</strong>
              <span>Version {updateInfo?.version} is ready to download</span>
            </div>
            <div className={styles.actions}>
              <button className={styles.primaryButton} onClick={handleDownload}>
                Download
              </button>
              <button className={styles.secondaryButton} onClick={handleDismiss}>
                Later
              </button>
            </div>
          </>
        )}

        {status === 'downloading' && progress && (
          <>
            <div className={styles.icon}>
              <DownloadIcon />
            </div>
            <div className={styles.text}>
              <strong>Downloading Update</strong>
              <span>
                {formatBytes(progress.transferred)} / {formatBytes(progress.total)}
                {' - '}
                {formatSpeed(progress.bytesPerSecond)}
              </span>
            </div>
            <div className={styles.progressBar}>
              <div
                className={styles.progressFill}
                style={{ width: `${progress.percent}%` }}
              />
            </div>
          </>
        )}

        {status === 'ready' && (
          <>
            <div className={styles.icon}>
              <CheckIcon />
            </div>
            <div className={styles.text}>
              <strong>Update Ready</strong>
              <span>Version {updateInfo?.version} will be installed on restart</span>
            </div>
            <div className={styles.actions}>
              <button className={styles.primaryButton} onClick={handleInstall}>
                Restart Now
              </button>
              <button className={styles.secondaryButton} onClick={handleDismiss}>
                Later
              </button>
            </div>
          </>
        )}

        {status === 'error' && (
          <>
            <div className={styles.icon}>
              <ErrorIcon />
            </div>
            <div className={styles.text}>
              <strong>Update Error</strong>
              <span>{error}</span>
            </div>
            <div className={styles.actions}>
              <button className={styles.secondaryButton} onClick={handleDismiss}>
                Dismiss
              </button>
            </div>
          </>
        )}
      </div>
    </div>
  );
};

// Icon components
const UpdateIcon = () => (
  <svg viewBox="0 0 24 24" width="24" height="24" fill="currentColor">
    <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 15v-4H8l4-4 4 4h-3v4h-2z" />
  </svg>
);

const DownloadIcon = () => (
  <svg viewBox="0 0 24 24" width="24" height="24" fill="currentColor">
    <path d="M19 9h-4V3H9v6H5l7 7 7-7zM5 18v2h14v-2H5z" />
  </svg>
);

const CheckIcon = () => (
  <svg viewBox="0 0 24 24" width="24" height="24" fill="currentColor">
    <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
  </svg>
);

const ErrorIcon = () => (
  <svg viewBox="0 0 24 24" width="24" height="24" fill="currentColor">
    <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z" />
  </svg>
);
```

### Update Notification Styles

```css
/* src/renderer/components/UpdateNotification/UpdateNotification.module.css */
.notification {
  position: fixed;
  bottom: 20px;
  right: 20px;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  padding: 16px;
  max-width: 400px;
  z-index: 1000;
  animation: slideIn 0.3s ease-out;
}

@keyframes slideIn {
  from {
    transform: translateY(20px);
    opacity: 0;
  }
  to {
    transform: translateY(0);
    opacity: 1;
  }
}

.notification[data-status="error"] {
  border-color: var(--color-error);
}

.content {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 12px;
}

.icon {
  color: var(--color-primary);
}

.notification[data-status="error"] .icon {
  color: var(--color-error);
}

.notification[data-status="ready"] .icon {
  color: var(--color-success);
}

.text {
  flex: 1;
  min-width: 200px;
}

.text strong {
  display: block;
  font-size: 14px;
  margin-bottom: 4px;
}

.text span {
  font-size: 12px;
  color: var(--color-text-secondary);
}

.actions {
  display: flex;
  gap: 8px;
  width: 100%;
  margin-top: 8px;
}

.primaryButton {
  flex: 1;
  padding: 8px 16px;
  background: var(--color-primary);
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-weight: 500;
}

.primaryButton:hover {
  background: var(--color-primary-hover);
}

.secondaryButton {
  padding: 8px 16px;
  background: transparent;
  color: var(--color-text);
  border: 1px solid var(--color-border);
  border-radius: 4px;
  cursor: pointer;
}

.secondaryButton:hover {
  background: var(--color-hover);
}

.progressBar {
  width: 100%;
  height: 4px;
  background: var(--color-border);
  border-radius: 2px;
  overflow: hidden;
  margin-top: 8px;
}

.progressFill {
  height: 100%;
  background: var(--color-primary);
  transition: width 0.3s ease;
}
```

---

## Testing Requirements

### Unit Tests

```typescript
// src/electron/main/updater/__tests__/updater.test.ts
import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('electron-updater', () => ({
  autoUpdater: {
    on: vi.fn(),
    setFeedURL: vi.fn(),
    checkForUpdates: vi.fn().mockResolvedValue({ updateInfo: { version: '2.0.0' } }),
    downloadUpdate: vi.fn(),
    quitAndInstall: vi.fn(),
    autoDownload: false,
    autoInstallOnAppQuit: true,
    allowDowngrade: false,
    allowPrerelease: false,
    channel: 'stable',
  },
}));

vi.mock('electron', () => ({
  app: {
    getVersion: vi.fn().mockReturnValue('1.0.0'),
    isPackaged: true,
  },
  dialog: {
    showMessageBox: vi.fn().mockResolvedValue({ response: 0 }),
    showErrorBox: vi.fn(),
  },
  BrowserWindow: vi.fn(),
}));

describe('AutoUpdaterService', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should check for updates', async () => {
    const { autoUpdater } = await import('electron-updater');
    const { autoUpdaterService } = await import('../index');

    await autoUpdaterService.checkForUpdates(true);
    expect(autoUpdater.checkForUpdates).toHaveBeenCalled();
  });

  it('should download update when available', async () => {
    const { autoUpdater } = await import('electron-updater');
    const { autoUpdaterService } = await import('../index');

    autoUpdaterService.downloadUpdate();
    expect(autoUpdater.downloadUpdate).toHaveBeenCalled();
  });

  it('should install update', async () => {
    const { autoUpdater } = await import('electron-updater');
    const { autoUpdaterService } = await import('../index');

    // Simulate downloaded state
    (autoUpdaterService as any).state.downloaded = true;
    autoUpdaterService.installUpdate();
    expect(autoUpdater.quitAndInstall).toHaveBeenCalled();
  });
});
```

---

## Related Specs

- Spec 161: Electron Main Process
- Spec 166: App Lifecycle
- Spec 175: Build Configuration
- Spec 176: Code Signing
