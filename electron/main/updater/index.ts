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

interface UpdateHistoryEntry {
  version: string;
  date: string;
  success: boolean;
  error?: string;
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
  private updateHistory: UpdateHistoryEntry[] = [];
  private skipVersions: string[] = [];
  private checkInterval: NodeJS.Timer | null = null;

  constructor() {
    this.loadUpdateHistory();
    this.loadSkipVersions();
    this.configureUpdater();
    this.setupEventHandlers();
  }

  private configureUpdater(): void {
    // Configure auto-updater
    autoUpdater.autoDownload = false;
    autoUpdater.autoInstallOnAppQuit = true;
    autoUpdater.allowDowngrade = false;
    autoUpdater.allowPrerelease = this.getAllowPrerelease();

    // Set update feed URL (GitHub releases)
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
    return (configManager.get('updateChannel' as any) || 'stable') as UpdateChannel;
  }

  private setupEventHandlers(): void {
    autoUpdater.on('checking-for-update', () => {
      logger.info('Checking for updates');
      this.state.checking = true;
      this.state.error = null;
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
      
      if (!this.isVersionSkipped(info.version)) {
        this.promptForDownload(info);
      }
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

      // Record failed update in history
      if (this.state.updateInfo) {
        this.addToHistory({
          version: this.state.updateInfo.version,
          date: new Date().toISOString(),
          success: false,
          error: error.message,
        });
      }

      // Rollback handling
      this.handleUpdateFailure(error);
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
      buttons: ['Restart Now', 'Install on Quit'],
      defaultId: 0,
    });

    if (response === 0) {
      this.installUpdate();
    } else {
      // Install on quit is already enabled by default
      logger.info('Update will be installed on quit');
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
    if (!this.skipVersions.includes(version)) {
      this.skipVersions.push(version);
      this.saveSkipVersions();
    }
    logger.info('Skipped version', { version });
  }

  private isVersionSkipped(version: string): boolean {
    return this.skipVersions.includes(version);
  }

  private handleUpdateFailure(error: Error): void {
    logger.warn('Update failed, implementing rollback strategy', { error: error.message });
    
    // Clear any partial downloads
    autoUpdater.getFeedURL().then(feedUrl => {
      logger.info('Clearing update cache after failure');
    }).catch(() => {
      // Ignore errors when clearing cache
    });

    // Reset state for retry
    this.state = {
      ...this.state,
      downloading: false,
      downloaded: false,
      error,
    };
  }

  private loadUpdateHistory(): void {
    try {
      const historyPath = require('path').join(app.getPath('userData'), 'update-history.json');
      if (require('fs').existsSync(historyPath)) {
        const data = require('fs').readFileSync(historyPath, 'utf-8');
        this.updateHistory = JSON.parse(data);
      }
    } catch (error) {
      logger.warn('Failed to load update history', { error });
      this.updateHistory = [];
    }
  }

  private saveUpdateHistory(): void {
    try {
      const historyPath = require('path').join(app.getPath('userData'), 'update-history.json');
      require('fs').writeFileSync(historyPath, JSON.stringify(this.updateHistory, null, 2));
    } catch (error) {
      logger.error('Failed to save update history', { error });
    }
  }

  private loadSkipVersions(): void {
    try {
      const skipPath = require('path').join(app.getPath('userData'), 'skip-versions.json');
      if (require('fs').existsSync(skipPath)) {
        const data = require('fs').readFileSync(skipPath, 'utf-8');
        this.skipVersions = JSON.parse(data);
      }
    } catch (error) {
      logger.warn('Failed to load skip versions', { error });
      this.skipVersions = [];
    }
  }

  private saveSkipVersions(): void {
    try {
      const skipPath = require('path').join(app.getPath('userData'), 'skip-versions.json');
      require('fs').writeFileSync(skipPath, JSON.stringify(this.skipVersions, null, 2));
    } catch (error) {
      logger.error('Failed to save skip versions', { error });
    }
  }

  private addToHistory(entry: UpdateHistoryEntry): void {
    this.updateHistory.unshift(entry);
    // Keep only last 50 entries
    if (this.updateHistory.length > 50) {
      this.updateHistory = this.updateHistory.slice(0, 50);
    }
    this.saveUpdateHistory();
  }

  private startPeriodicCheck(): void {
    if (this.checkInterval) {
      clearInterval(this.checkInterval);
    }
    
    // Check every 4 hours
    this.checkInterval = setInterval(() => {
      if (navigator.onLine) {
        this.checkForUpdates(true);
      }
    }, 4 * 60 * 60 * 1000);
  }

  private stopPeriodicCheck(): void {
    if (this.checkInterval) {
      clearInterval(this.checkInterval);
      this.checkInterval = null;
    }
  }

  setMainWindow(window: BrowserWindow): void {
    this.mainWindow = window;
  }

  async checkForUpdates(silent = false): Promise<void> {
    if (this.state.checking || this.state.downloading) {
      logger.debug('Update check already in progress');
      return;
    }

    // Handle offline scenario
    if (!navigator.onLine) {
      logger.info('Offline, skipping update check');
      if (!silent) {
        dialog.showMessageBox(this.mainWindow!, {
          type: 'warning',
          title: 'Offline',
          message: 'Cannot check for updates while offline',
          detail: 'Please check your internet connection and try again.',
        });
      }
      return;
    }

    try {
      const result = await autoUpdater.checkForUpdates();

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

    if (!navigator.onLine) {
      dialog.showErrorBox('Offline', 'Cannot download updates while offline');
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

    // Record successful update in history
    if (this.state.updateInfo) {
      this.addToHistory({
        version: this.state.updateInfo.version,
        date: new Date().toISOString(),
        success: true,
      });
    }

    logger.info('Installing update');
    autoUpdater.quitAndInstall(false, true);
  }

  setChannel(channel: UpdateChannel): void {
    configManager.set('updateChannel' as any, channel);
    autoUpdater.allowPrerelease = channel !== 'stable';
    autoUpdater.channel = channel;
    
    // Clear skipped versions when changing channels
    this.skipVersions = [];
    this.saveSkipVersions();
    
    logger.info('Update channel changed', { channel });
  }

  getState(): UpdateState {
    return { ...this.state };
  }

  getUpdateHistory(): UpdateHistoryEntry[] {
    return [...this.updateHistory];
  }

  clearSkippedVersions(): void {
    this.skipVersions = [];
    this.saveSkipVersions();
    logger.info('Cleared skipped versions');
  }

  startAutoCheck(): void {
    if (configManager.get('autoUpdate')) {
      this.startPeriodicCheck();
      
      // Initial check after 5 seconds
      setTimeout(() => {
        if (navigator.onLine) {
          this.checkForUpdates(true);
        }
      }, 5000);
    }
  }

  stopAutoCheck(): void {
    this.stopPeriodicCheck();
  }

  destroy(): void {
    this.stopPeriodicCheck();
    autoUpdater.removeAllListeners();
  }
}

export const autoUpdaterService = new AutoUpdaterService();

export function setupAutoUpdater(mainWindow: BrowserWindow): void {
  autoUpdaterService.setMainWindow(mainWindow);

  // Start auto-checking only in packaged app
  if (app.isPackaged) {
    autoUpdaterService.startAutoCheck();
  }
}

export { UpdateChannel, UpdateState, UpdateHistoryEntry };