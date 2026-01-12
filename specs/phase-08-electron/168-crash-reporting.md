# Spec 168: Crash Reporting

## Phase
8 - Electron Shell

## Spec ID
168

## Status
Planned

## Dependencies
- Spec 161 (Electron Main Process)
- Spec 166 (App Lifecycle)

## Estimated Context
~8%

---

## Objective

Implement comprehensive crash reporting for both main and renderer processes, capturing minidumps, stack traces, and diagnostic information. Support both local crash storage and remote reporting to a crash collection service.

---

## Acceptance Criteria

- [ ] Capture crashes in main process
- [ ] Capture crashes in renderer process
- [ ] Capture uncaught exceptions
- [ ] Capture unhandled promise rejections
- [ ] Generate minidump files
- [ ] Collect diagnostic information (system info, app state)
- [ ] Optional remote crash reporting
- [ ] Local crash log storage
- [ ] User consent for crash reporting
- [ ] Crash history viewing

---

## Implementation Details

### Crash Reporter Service

```typescript
// src/electron/main/crash-reporter/index.ts
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
```

### Renderer Error Handler

```typescript
// src/renderer/utils/error-handler.ts
class RendererErrorHandler {
  private initialized = false;

  initialize(): void {
    if (this.initialized) return;

    // Catch unhandled errors
    window.addEventListener('error', (event) => {
      this.reportException({
        name: event.error?.name || 'Error',
        message: event.error?.message || event.message,
        stack: event.error?.stack,
      });
    });

    // Catch unhandled promise rejections
    window.addEventListener('unhandledrejection', (event) => {
      const error =
        event.reason instanceof Error
          ? event.reason
          : new Error(String(event.reason));

      this.reportRejection({
        name: error.name,
        message: error.message,
        stack: error.stack,
      });
    });

    // Catch React errors via error boundary
    this.setupReactErrorBoundary();

    this.initialized = true;
  }

  private reportException(error: {
    name: string;
    message: string;
    stack?: string;
  }): void {
    window.electronAPI?.reportException(error);
  }

  private reportRejection(error: {
    name: string;
    message: string;
    stack?: string;
  }): void {
    window.electronAPI?.reportRejection(error);
  }

  private setupReactErrorBoundary(): void {
    // Will be used with React Error Boundary component
  }

  reportError(error: Error, context?: Record<string, unknown>): void {
    this.reportException({
      name: error.name,
      message: error.message,
      stack: error.stack,
    });
  }
}

export const errorHandler = new RendererErrorHandler();
```

### React Error Boundary

```typescript
// src/renderer/components/ErrorBoundary/ErrorBoundary.tsx
import React, { Component, ReactNode } from 'react';
import { errorHandler } from '../../utils/error-handler';
import styles from './ErrorBoundary.module.css';

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo): void {
    errorHandler.reportError(error, {
      componentStack: errorInfo.componentStack,
    });
  }

  handleReload = (): void => {
    window.location.reload();
  };

  handleReset = (): void => {
    this.setState({ hasError: false, error: null });
  };

  render(): ReactNode {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback;
      }

      return (
        <div className={styles.errorContainer}>
          <div className={styles.errorContent}>
            <div className={styles.errorIcon}>
              <ErrorIcon />
            </div>
            <h2>Something went wrong</h2>
            <p className={styles.errorMessage}>
              {this.state.error?.message || 'An unexpected error occurred'}
            </p>
            <div className={styles.errorActions}>
              <button className={styles.primaryButton} onClick={this.handleReload}>
                Reload Application
              </button>
              <button className={styles.secondaryButton} onClick={this.handleReset}>
                Try Again
              </button>
            </div>
            {process.env.NODE_ENV === 'development' && this.state.error?.stack && (
              <details className={styles.errorDetails}>
                <summary>Error Details</summary>
                <pre>{this.state.error.stack}</pre>
              </details>
            )}
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}

const ErrorIcon = () => (
  <svg viewBox="0 0 24 24" width="48" height="48" fill="currentColor">
    <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z" />
  </svg>
);
```

### Crash Reporter IPC Handlers

```typescript
// src/electron/main/ipc/crash-reporter.ts
import { ipcMain } from 'electron';
import { crashReporterService } from '../crash-reporter';

export function setupCrashReporterIpcHandlers(): void {
  ipcMain.handle('crash-reporter:getReports', async () => {
    return crashReporterService.getCrashReports();
  });

  ipcMain.handle('crash-reporter:clearReports', async () => {
    await crashReporterService.clearCrashReports();
  });

  ipcMain.handle('crash-reporter:deleteReport', async (_, id: string) => {
    await crashReporterService.deleteCrashReport(id);
  });

  ipcMain.handle('crash-reporter:setEnabled', (_, enabled: boolean) => {
    crashReporterService.setEnabled(enabled);
  });

  // Receive errors from renderer
  ipcMain.on('crash-reporter:exception', () => {
    // Handled in setupRenderer
  });

  ipcMain.on('crash-reporter:rejection', () => {
    // Handled in setupRenderer
  });
}
```

---

## Testing Requirements

### Unit Tests

```typescript
// src/electron/main/crash-reporter/__tests__/crash-reporter.test.ts
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { vol } from 'memfs';

vi.mock('fs', async () => {
  const memfs = await import('memfs');
  return { ...memfs.fs, promises: memfs.fs.promises };
});

vi.mock('electron', () => ({
  crashReporter: {
    start: vi.fn(),
    getUploadedReports: vi.fn().mockReturnValue([]),
  },
  app: {
    getPath: vi.fn().mockReturnValue('/mock/userData'),
    getVersion: vi.fn().mockReturnValue('1.0.0'),
    name: 'Tachikoma',
    relaunch: vi.fn(),
    quit: vi.fn(),
  },
  dialog: {
    showErrorBox: vi.fn(),
    showMessageBox: vi.fn().mockResolvedValue({ response: 0 }),
  },
  BrowserWindow: vi.fn(),
  ipcMain: {
    on: vi.fn(),
  },
}));

describe('CrashReporterService', () => {
  beforeEach(() => {
    vol.reset();
    vol.mkdirSync('/mock/userData/crashes', { recursive: true });
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  it('should initialize crash reporter', async () => {
    const { crashReporter } = await import('electron');
    const { crashReporterService } = await import('../index');

    crashReporterService.setup({ uploadUrl: 'https://crash.example.com' });

    expect(crashReporter.start).toHaveBeenCalled();
  });

  it('should save crash reports', async () => {
    const { crashReporterService } = await import('../index');

    crashReporterService.setup({});

    // Trigger a mock crash
    // The report should be saved to the crash directory
  });

  it('should retrieve crash reports', async () => {
    const report = {
      id: 'test-crash-1',
      timestamp: new Date().toISOString(),
      type: 'exception',
      process: 'main',
      error: { name: 'Error', message: 'Test error' },
      system: { platform: 'darwin', release: '21.0.0', arch: 'x64', cpuCount: 8, totalMemory: 16000000000, freeMemory: 8000000000 },
      app: { version: '1.0.0', electron: '25.0.0', chrome: '114.0.0', node: '18.0.0' },
    };

    vol.writeFileSync(
      '/mock/userData/crashes/test-crash-1.json',
      JSON.stringify(report)
    );

    const { crashReporterService } = await import('../index');
    const reports = await crashReporterService.getCrashReports();

    expect(reports).toHaveLength(1);
    expect(reports[0].id).toBe('test-crash-1');
  });
});
```

---

## Related Specs

- Spec 161: Electron Main Process
- Spec 166: App Lifecycle
- Spec 170: IPC Channels
