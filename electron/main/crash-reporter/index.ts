import {
  crashReporter,
  app,
  dialog,
  BrowserWindow,
} from 'electron';
import { promises as fs } from 'fs';
import { join } from 'path';
import { platform, release, arch, cpus, totalmem, freemem } from 'os';
import { Logger } from '../logger';
import { configManager } from '../config';

const logger = new Logger('crash-reporter');

interface CrashReport {
  id: string;
  timestamp: string;
  type: 'crash' | 'exception' | 'rejection';
  process: 'main' | 'renderer';
  error: {
    name: string;
    message: string;
    stack?: string;
  };
  system: {
    platform: string;
    release: string;
    arch: string;
    cpuCount: number;
    totalMemory: number;
    freeMemory: number;
  };
  app: {
    version: string;
    electron: string;
    chrome: string;
    node: string;
  };
  minidumpPath?: string;
  extraData?: Record<string, string>;
}

class CrashReporterService {
  private crashDir: string;
  private isEnabled = false;
  private uploadUrl: string | null = null;

  constructor() {
    this.crashDir = join(app.getPath('userData'), 'crashes');
    this.ensureCrashDir();
  }

  private async ensureCrashDir(): Promise<void> {
    try {
      await fs.mkdir(this.crashDir, { recursive: true });
    } catch (error) {
      logger.error('Failed to create crash directory', { error });
    }
  }

  setup(options: { uploadUrl?: string; companyName?: string } = {}): void {
    this.uploadUrl = options.uploadUrl || null;

    // Enable Electron's built-in crash reporter
    crashReporter.start({
      companyName: options.companyName || 'Tachikoma',
      productName: app.name,
      submitURL: options.uploadUrl || '',
      uploadToServer: !!options.uploadUrl && this.hasUserConsent(),
      ignoreSystemCrashHandler: false,
      extra: {
        version: app.getVersion(),
        platform: platform(),
        arch: arch(),
      },
    });

    // Setup uncaught exception handler
    process.on('uncaughtException', (error) => {
      this.handleException(error, 'main');
    });

    // Setup unhandled rejection handler
    process.on('unhandledRejection', (reason) => {
      const error = reason instanceof Error ? reason : new Error(String(reason));
      this.handleRejection(error, 'main');
    });

    this.isEnabled = true;
    logger.info('Crash reporter initialized', {
      uploadEnabled: !!options.uploadUrl,
      crashDir: this.crashDir,
    });
  }

  private hasUserConsent(): boolean {
    return configManager.get('telemetry') ?? false;
  }

  private async handleException(error: Error, process: 'main' | 'renderer'): Promise<void> {
    logger.error('Uncaught exception', { error: error.message, stack: error.stack });

    const report = await this.createCrashReport(error, 'exception', process);
    await this.saveCrashReport(report);

    if (process === 'main') {
      this.showCrashDialog(error);
    }
  }

  private async handleRejection(error: Error, process: 'main' | 'renderer'): Promise<void> {
    logger.error('Unhandled rejection', { error: error.message, stack: error.stack });

    const report = await this.createCrashReport(error, 'rejection', process);
    await this.saveCrashReport(report);
  }

  private async createCrashReport(
    error: Error,
    type: CrashReport['type'],
    processType: CrashReport['process']
  ): Promise<CrashReport> {
    return {
      id: this.generateId(),
      timestamp: new Date().toISOString(),
      type,
      process: processType,
      error: {
        name: error.name,
        message: error.message,
        stack: error.stack,
      },
      system: {
        platform: platform(),
        release: release(),
        arch: arch(),
        cpuCount: cpus().length,
        totalMemory: totalmem(),
        freeMemory: freemem(),
      },
      app: {
        version: app.getVersion(),
        electron: process.versions.electron,
        chrome: process.versions.chrome,
        node: process.versions.node,
      },
    };
  }

  private generateId(): string {
    return `crash-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }

  private async saveCrashReport(report: CrashReport): Promise<void> {
    const reportPath = join(this.crashDir, `${report.id}.json`);

    try {
      await fs.writeFile(reportPath, JSON.stringify(report, null, 2));
      logger.info('Crash report saved', { path: reportPath });

      // Upload if enabled
      if (this.uploadUrl && this.hasUserConsent()) {
        await this.uploadCrashReport(report);
      }
    } catch (error) {
      logger.error('Failed to save crash report', { error });
    }
  }

  private async uploadCrashReport(report: CrashReport): Promise<void> {
    if (!this.uploadUrl) return;

    try {
      const response = await fetch(this.uploadUrl, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(report),
      });

      if (response.ok) {
        logger.info('Crash report uploaded', { id: report.id });
      } else {
        logger.warn('Failed to upload crash report', {
          id: report.id,
          status: response.status,
        });
      }
    } catch (error) {
      logger.error('Crash report upload failed', { error });
    }
  }

  private showCrashDialog(error: Error): void {
    dialog.showErrorBox(
      'An error occurred',
      `The application encountered an unexpected error:\n\n${error.message}\n\nThe application will now restart.`
    );

    // Restart the app
    app.relaunch();
    app.quit();
  }

  // Setup renderer crash handling
  setupRenderer(window: BrowserWindow): void {
    const webContents = window.webContents;

    webContents.on('crashed', async (event, killed) => {
      logger.error('Renderer crashed', { killed });

      const report = await this.createCrashReport(
        new Error(`Renderer process ${killed ? 'killed' : 'crashed'}`),
        'crash',
        'renderer'
      );
      await this.saveCrashReport(report);

      const { response } = await dialog.showMessageBox({
        type: 'error',
        title: 'Renderer Crashed',
        message: 'The window has stopped responding',
        detail: killed
          ? 'The process was killed by the operating system.'
          : 'An unexpected error occurred.',
        buttons: ['Reload', 'Close'],
        defaultId: 0,
      });

      if (response === 0) {
        webContents.reload();
      } else {
        window.close();
      }
    });

    webContents.on('render-process-gone', async (event, details) => {
      logger.error('Render process gone', { reason: details.reason });

      const report = await this.createCrashReport(
        new Error(`Render process gone: ${details.reason}`),
        'crash',
        'renderer'
      );
      report.extraData = { reason: details.reason, exitCode: String(details.exitCode) };
      await this.saveCrashReport(report);
    });

    // Handle renderer exceptions via IPC
    const { ipcMain } = require('electron');
    ipcMain.on('crash-reporter:exception', async (event, errorData) => {
      const error = new Error(errorData.message);
      error.name = errorData.name;
      error.stack = errorData.stack;
      await this.handleException(error, 'renderer');
    });

    ipcMain.on('crash-reporter:rejection', async (event, errorData) => {
      const error = new Error(errorData.message);
      error.name = errorData.name;
      error.stack = errorData.stack;
      await this.handleRejection(error, 'renderer');
    });
  }

  async getCrashReports(): Promise<CrashReport[]> {
    try {
      const files = await fs.readdir(this.crashDir);
      const reports: CrashReport[] = [];

      for (const file of files) {
        if (file.endsWith('.json')) {
          const content = await fs.readFile(join(this.crashDir, file), 'utf-8');
          reports.push(JSON.parse(content));
        }
      }

      return reports.sort(
        (a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()
      );
    } catch (error) {
      logger.error('Failed to get crash reports', { error });
      return [];
    }
  }

  async clearCrashReports(): Promise<void> {
    try {
      const files = await fs.readdir(this.crashDir);

      for (const file of files) {
        await fs.unlink(join(this.crashDir, file));
      }

      logger.info('Crash reports cleared');
    } catch (error) {
      logger.error('Failed to clear crash reports', { error });
    }
  }

  async deleteCrashReport(id: string): Promise<void> {
    const reportPath = join(this.crashDir, `${id}.json`);

    try {
      await fs.unlink(reportPath);
      logger.info('Crash report deleted', { id });
    } catch (error) {
      logger.error('Failed to delete crash report', { error });
    }
  }

  getMinidumpPath(): string {
    return crashReporter.getUploadedReports()[0]?.date
      ? join(app.getPath('crashDumps'))
      : '';
  }

  setEnabled(enabled: boolean): void {
    this.isEnabled = enabled;
    // Note: crashReporter cannot be disabled once started
    // This controls our custom exception handling
  }
}

export const crashReporterService = new CrashReporterService();

export function setupCrashReporter(): void {
  crashReporterService.setup({
    uploadUrl: process.env.CRASH_REPORT_URL,
    companyName: 'Tachikoma',
  });
}

export type { CrashReport };