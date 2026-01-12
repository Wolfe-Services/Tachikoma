# 503 - Auto-Update Client

**Phase:** 23 - Build & Distribution
**Spec ID:** 503
**Status:** Planned
**Dependencies:** 502-auto-update-server
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Implement the client-side auto-update functionality using electron-updater, providing seamless application updates with user notifications and progress tracking.

---

## Acceptance Criteria

- [ ] Automatic update checking on startup
- [ ] User notification of available updates
- [ ] Download progress display
- [ ] Install on quit option
- [ ] Manual update check available
- [ ] Update error handling

---

## Implementation Details

### 1. Auto-Updater Setup

Create `electron/src/updater.ts`:

```typescript
import { autoUpdater, UpdateInfo, ProgressInfo } from 'electron-updater';
import { app, BrowserWindow, dialog, ipcMain } from 'electron';
import log from 'electron-log';

// Configure logging
autoUpdater.logger = log;
(autoUpdater.logger as typeof log).transports.file.level = 'info';

export class AppUpdater {
  private mainWindow: BrowserWindow | null = null;
  private updateAvailable = false;
  private updateDownloaded = false;

  constructor() {
    // Configure auto-updater
    autoUpdater.autoDownload = false;
    autoUpdater.autoInstallOnAppQuit = true;
    autoUpdater.allowDowngrade = false;

    // For development testing
    if (process.env.NODE_ENV === 'development') {
      autoUpdater.forceDevUpdateConfig = true;
    }

    this.setupEventHandlers();
  }

  setMainWindow(window: BrowserWindow) {
    this.mainWindow = window;
  }

  private setupEventHandlers() {
    autoUpdater.on('checking-for-update', () => {
      log.info('Checking for updates...');
      this.sendToRenderer('update-checking');
    });

    autoUpdater.on('update-available', (info: UpdateInfo) => {
      log.info('Update available:', info.version);
      this.updateAvailable = true;
      this.sendToRenderer('update-available', {
        version: info.version,
        releaseDate: info.releaseDate,
        releaseNotes: info.releaseNotes,
      });
    });

    autoUpdater.on('update-not-available', (info: UpdateInfo) => {
      log.info('No update available');
      this.sendToRenderer('update-not-available', {
        version: info.version,
      });
    });

    autoUpdater.on('download-progress', (progress: ProgressInfo) => {
      log.info(`Download progress: ${progress.percent.toFixed(1)}%`);
      this.sendToRenderer('update-download-progress', {
        percent: progress.percent,
        bytesPerSecond: progress.bytesPerSecond,
        transferred: progress.transferred,
        total: progress.total,
      });
    });

    autoUpdater.on('update-downloaded', (info: UpdateInfo) => {
      log.info('Update downloaded:', info.version);
      this.updateDownloaded = true;
      this.sendToRenderer('update-downloaded', {
        version: info.version,
      });

      // Show notification
      this.showUpdateReadyNotification(info);
    });

    autoUpdater.on('error', (error: Error) => {
      log.error('Update error:', error);
      this.sendToRenderer('update-error', {
        message: error.message,
      });
    });

    // IPC handlers
    ipcMain.handle('updater:check', () => this.checkForUpdates());
    ipcMain.handle('updater:download', () => this.downloadUpdate());
    ipcMain.handle('updater:install', () => this.installUpdate());
    ipcMain.handle('updater:get-version', () => app.getVersion());
  }

  async checkForUpdates(): Promise<void> {
    try {
      await autoUpdater.checkForUpdates();
    } catch (error) {
      log.error('Failed to check for updates:', error);
      throw error;
    }
  }

  async downloadUpdate(): Promise<void> {
    if (!this.updateAvailable) {
      throw new Error('No update available');
    }

    try {
      await autoUpdater.downloadUpdate();
    } catch (error) {
      log.error('Failed to download update:', error);
      throw error;
    }
  }

  installUpdate(): void {
    if (!this.updateDownloaded) {
      throw new Error('Update not downloaded');
    }

    // Quit and install
    autoUpdater.quitAndInstall(false, true);
  }

  private sendToRenderer(channel: string, data?: unknown): void {
    if (this.mainWindow && !this.mainWindow.isDestroyed()) {
      this.mainWindow.webContents.send(channel, data);
    }
  }

  private async showUpdateReadyNotification(info: UpdateInfo): Promise<void> {
    const result = await dialog.showMessageBox(this.mainWindow!, {
      type: 'info',
      title: 'Update Ready',
      message: `Version ${info.version} has been downloaded.`,
      detail: 'The update will be installed when you quit the application. Would you like to restart now?',
      buttons: ['Restart Now', 'Later'],
      defaultId: 0,
      cancelId: 1,
    });

    if (result.response === 0) {
      this.installUpdate();
    }
  }
}

// Singleton instance
export const appUpdater = new AppUpdater();
```

### 2. Main Process Integration

Update `electron/src/main.ts`:

```typescript
import { app, BrowserWindow } from 'electron';
import { appUpdater } from './updater';

let mainWindow: BrowserWindow | null = null;

async function createWindow() {
  mainWindow = new BrowserWindow({
    // ... window config
  });

  // Set up auto-updater
  appUpdater.setMainWindow(mainWindow);

  // Check for updates after window is ready (non-blocking)
  mainWindow.once('ready-to-show', () => {
    // Delay update check to not slow down startup
    setTimeout(() => {
      appUpdater.checkForUpdates().catch(() => {
        // Silently fail on initial check
      });
    }, 3000);
  });
}

app.whenReady().then(createWindow);
```

### 3. Renderer Process Integration

Create `web/src/lib/stores/updater.ts`:

```typescript
import { writable, derived } from 'svelte/store';

interface UpdateState {
  checking: boolean;
  available: boolean;
  downloading: boolean;
  downloaded: boolean;
  error: string | null;
  progress: {
    percent: number;
    bytesPerSecond: number;
    transferred: number;
    total: number;
  } | null;
  updateInfo: {
    version: string;
    releaseDate?: string;
    releaseNotes?: string;
  } | null;
}

function createUpdaterStore() {
  const { subscribe, update, set } = writable<UpdateState>({
    checking: false,
    available: false,
    downloading: false,
    downloaded: false,
    error: null,
    progress: null,
    updateInfo: null,
  });

  // Listen to IPC events
  if (typeof window !== 'undefined' && window.tachikoma) {
    window.tachikoma.on('update-checking', () => {
      update(s => ({ ...s, checking: true, error: null }));
    });

    window.tachikoma.on('update-available', (info) => {
      update(s => ({
        ...s,
        checking: false,
        available: true,
        updateInfo: info,
      }));
    });

    window.tachikoma.on('update-not-available', () => {
      update(s => ({ ...s, checking: false, available: false }));
    });

    window.tachikoma.on('update-download-progress', (progress) => {
      update(s => ({ ...s, downloading: true, progress }));
    });

    window.tachikoma.on('update-downloaded', (info) => {
      update(s => ({
        ...s,
        downloading: false,
        downloaded: true,
        updateInfo: info,
        progress: null,
      }));
    });

    window.tachikoma.on('update-error', (error) => {
      update(s => ({
        ...s,
        checking: false,
        downloading: false,
        error: error.message,
      }));
    });
  }

  return {
    subscribe,

    async checkForUpdates() {
      try {
        await window.tachikoma.invoke('updater:check');
      } catch (error) {
        update(s => ({ ...s, error: (error as Error).message }));
      }
    },

    async downloadUpdate() {
      try {
        update(s => ({ ...s, downloading: true }));
        await window.tachikoma.invoke('updater:download');
      } catch (error) {
        update(s => ({ ...s, downloading: false, error: (error as Error).message }));
      }
    },

    async installUpdate() {
      await window.tachikoma.invoke('updater:install');
    },

    clearError() {
      update(s => ({ ...s, error: null }));
    },
  };
}

export const updater = createUpdaterStore();

// Derived stores for convenience
export const hasUpdate = derived(updater, $u => $u.available && !$u.downloaded);
export const updateReady = derived(updater, $u => $u.downloaded);
export const isUpdating = derived(updater, $u => $u.checking || $u.downloading);
```

### 4. Update UI Component

Create `web/src/lib/components/UpdateNotification.svelte`:

```svelte
<script lang="ts">
  import { updater, hasUpdate, updateReady, isUpdating } from '$lib/stores/updater';
  import { fade } from 'svelte/transition';

  $: state = $updater;
</script>

{#if $hasUpdate || $updateReady || state.downloading}
  <div class="update-notification" transition:fade>
    {#if state.downloading}
      <div class="update-progress">
        <span>Downloading update...</span>
        <div class="progress-bar">
          <div
            class="progress-fill"
            style="width: {state.progress?.percent ?? 0}%"
          />
        </div>
        <span class="progress-text">
          {state.progress?.percent.toFixed(0)}%
        </span>
      </div>
    {:else if $updateReady}
      <div class="update-ready">
        <span>Update ready! Version {state.updateInfo?.version}</span>
        <button class="install-btn" on:click={() => updater.installUpdate()}>
          Restart to Update
        </button>
      </div>
    {:else if $hasUpdate}
      <div class="update-available">
        <span>Update available: v{state.updateInfo?.version}</span>
        <button class="download-btn" on:click={() => updater.downloadUpdate()}>
          Download
        </button>
        <button class="dismiss-btn" on:click={() => { /* dismiss */ }}>
          Later
        </button>
      </div>
    {/if}
  </div>
{/if}

{#if state.error}
  <div class="update-error" transition:fade>
    <span>Update error: {state.error}</span>
    <button on:click={() => updater.clearError()}>Dismiss</button>
  </div>
{/if}

<style>
  .update-notification {
    position: fixed;
    bottom: 20px;
    right: 20px;
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    padding: 16px;
    box-shadow: var(--shadow-lg);
    z-index: 1000;
  }

  .progress-bar {
    width: 200px;
    height: 4px;
    background: var(--bg-tertiary);
    border-radius: 2px;
    overflow: hidden;
    margin: 8px 0;
  }

  .progress-fill {
    height: 100%;
    background: var(--accent-color);
    transition: width 0.3s ease;
  }

  button {
    margin-left: 8px;
    padding: 4px 12px;
    border-radius: 4px;
    cursor: pointer;
  }

  .install-btn {
    background: var(--accent-color);
    color: white;
    border: none;
  }

  .download-btn {
    background: var(--accent-color);
    color: white;
    border: none;
  }

  .dismiss-btn {
    background: transparent;
    border: 1px solid var(--border-color);
  }

  .update-error {
    position: fixed;
    bottom: 20px;
    right: 20px;
    background: var(--error-bg);
    color: var(--error-color);
    padding: 12px 16px;
    border-radius: 8px;
    z-index: 1000;
  }
</style>
```

### 5. Settings Integration

Add to Settings page for manual update check:

```svelte
<script lang="ts">
  import { updater, isUpdating } from '$lib/stores/updater';

  let currentVersion = '';

  onMount(async () => {
    currentVersion = await window.tachikoma.invoke('updater:get-version');
  });
</script>

<section class="update-settings">
  <h3>Updates</h3>
  <p>Current version: {currentVersion}</p>

  <button
    on:click={() => updater.checkForUpdates()}
    disabled={$isUpdating}
  >
    {$isUpdating ? 'Checking...' : 'Check for Updates'}
  </button>
</section>
```

---

## Testing Requirements

1. Update check runs on startup
2. Update notification displays correctly
3. Download progress updates in real-time
4. Install on quit works correctly
5. Manual update check functions
6. Error states are handled gracefully

---

## Related Specs

- Depends on: [502-auto-update-server.md](502-auto-update-server.md)
- Next: [504-version-management.md](504-version-management.md)
- Related: [167-auto-updates.md](../phase-08-electron/167-auto-updates.md)
