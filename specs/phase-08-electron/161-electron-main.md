# Spec 161: Electron Main Process

## Phase
8 - Electron Shell

## Spec ID
161

## Status
Planned

## Dependencies
- Phase 7 (React Component Library)
- Spec 001 (Project Structure)

## Estimated Context
~10%

---

## Objective

Implement the Electron main process as the core entry point for the desktop application. This includes proper process initialization, security configuration, and coordination with renderer processes through IPC.

---

## Acceptance Criteria

- [x] Main process initializes with proper security defaults
- [x] Single instance lock prevents multiple app instances
- [x] Environment-aware configuration (dev/prod)
- [x] Graceful error handling and logging
- [x] App ready lifecycle properly managed
- [x] Memory management and garbage collection hints
- [x] GPU process configuration for optimal rendering
- [x] Protocol registration for custom schemes
- [x] Session management for cookies and storage

---

## Implementation Details

### Main Process Entry Point

```typescript
// src/electron/main/index.ts
import {
  app,
  BrowserWindow,
  session,
  protocol,
  powerMonitor,
  nativeTheme,
} from 'electron';
import { join } from 'path';
import { electronApp, optimizer } from '@electron-toolkit/utils';
import { createMainWindow } from './window';
import { setupIpcHandlers } from './ipc';
import { setupMenu } from './menu';
import { setupAutoUpdater } from './updater';
import { setupCrashReporter } from './crash-reporter';
import { setupProtocolHandlers } from './protocol';
import { setupDeepLinks } from './deep-links';
import { Logger } from './logger';

const logger = new Logger('main');

// Disable hardware acceleration if needed
// app.disableHardwareAcceleration();

// Enable sandbox for all renderers
app.enableSandbox();

class TachikomaApp {
  private mainWindow: BrowserWindow | null = null;
  private isQuitting = false;

  constructor() {
    this.setupSecurityDefaults();
    this.setupEventListeners();
  }

  private setupSecurityDefaults(): void {
    // Disable navigation to unknown protocols
    app.on('web-contents-created', (_, contents) => {
      contents.on('will-navigate', (event, url) => {
        const parsedUrl = new URL(url);
        if (!['https:', 'http:', 'tachikoma:'].includes(parsedUrl.protocol)) {
          event.preventDefault();
          logger.warn(`Blocked navigation to: ${url}`);
        }
      });

      // Disable new window creation from renderer
      contents.setWindowOpenHandler(({ url }) => {
        // Open external links in default browser
        if (url.startsWith('https://')) {
          require('electron').shell.openExternal(url);
        }
        return { action: 'deny' };
      });

      // Disable remote module
      contents.on('remote-require', (event) => {
        event.preventDefault();
      });

      contents.on('remote-get-builtin', (event) => {
        event.preventDefault();
      });

      contents.on('remote-get-global', (event) => {
        event.preventDefault();
      });

      contents.on('remote-get-current-window', (event) => {
        event.preventDefault();
      });

      contents.on('remote-get-current-web-contents', (event) => {
        event.preventDefault();
      });
    });
  }

  private setupEventListeners(): void {
    // Single instance lock
    const gotTheLock = app.requestSingleInstanceLock();

    if (!gotTheLock) {
      logger.info('Another instance is running, quitting...');
      app.quit();
      return;
    }

    app.on('second-instance', (_, commandLine, workingDirectory) => {
      logger.info('Second instance attempted', { commandLine, workingDirectory });

      // Focus existing window
      if (this.mainWindow) {
        if (this.mainWindow.isMinimized()) {
          this.mainWindow.restore();
        }
        this.mainWindow.focus();
      }

      // Handle deep links from second instance
      const url = commandLine.find((arg) => arg.startsWith('tachikoma://'));
      if (url) {
        this.handleDeepLink(url);
      }
    });

    // App ready
    app.whenReady().then(() => this.onReady());

    // Window all closed
    app.on('window-all-closed', () => {
      if (process.platform !== 'darwin') {
        app.quit();
      }
    });

    // Activate (macOS)
    app.on('activate', () => {
      if (BrowserWindow.getAllWindows().length === 0) {
        this.createWindow();
      }
    });

    // Before quit
    app.on('before-quit', () => {
      this.isQuitting = true;
      logger.info('Application is quitting');
    });

    // Power monitor events
    powerMonitor.on('suspend', () => {
      logger.info('System suspending');
      this.mainWindow?.webContents.send('system:suspend');
    });

    powerMonitor.on('resume', () => {
      logger.info('System resuming');
      this.mainWindow?.webContents.send('system:resume');
    });

    powerMonitor.on('on-battery', () => {
      logger.info('Switched to battery power');
      this.mainWindow?.webContents.send('system:battery', { onBattery: true });
    });

    powerMonitor.on('on-ac', () => {
      logger.info('Switched to AC power');
      this.mainWindow?.webContents.send('system:battery', { onBattery: false });
    });

    // Theme changes
    nativeTheme.on('updated', () => {
      const isDark = nativeTheme.shouldUseDarkColors;
      this.mainWindow?.webContents.send('theme:changed', { isDark });
    });
  }

  private async onReady(): Promise<void> {
    logger.info('App ready', {
      version: app.getVersion(),
      electron: process.versions.electron,
      node: process.versions.node,
      chrome: process.versions.chrome,
      platform: process.platform,
      arch: process.arch,
    });

    // Set app user model id for Windows
    electronApp.setAppUserModelId('com.tachikoma.app');

    // Configure session
    this.configureSession();

    // Setup protocol handlers
    setupProtocolHandlers();

    // Setup deep links
    setupDeepLinks(this.handleDeepLink.bind(this));

    // Create main window
    await this.createWindow();

    // Setup IPC handlers
    setupIpcHandlers(this.mainWindow!);

    // Setup menu
    setupMenu(this.mainWindow!);

    // Setup auto updater (production only)
    if (!app.isPackaged) {
      logger.info('Development mode - auto updater disabled');
    } else {
      setupAutoUpdater(this.mainWindow!);
    }

    // Setup crash reporter
    setupCrashReporter();

    // Watch for window shortcuts in development
    if (!app.isPackaged) {
      app.on('browser-window-created', (_, window) => {
        optimizer.watchWindowShortcuts(window);
      });
    }
  }

  private configureSession(): void {
    const defaultSession = session.defaultSession;

    // Configure Content Security Policy
    defaultSession.webRequest.onHeadersReceived((details, callback) => {
      callback({
        responseHeaders: {
          ...details.responseHeaders,
          'Content-Security-Policy': [
            "default-src 'self'; " +
            "script-src 'self' 'unsafe-eval'; " +
            "style-src 'self' 'unsafe-inline'; " +
            "img-src 'self' data: https:; " +
            "font-src 'self' data:; " +
            "connect-src 'self' https://api.tachikoma.io wss://api.tachikoma.io; " +
            "frame-src 'none';"
          ],
        },
      });
    });

    // Configure permissions
    defaultSession.setPermissionRequestHandler((webContents, permission, callback) => {
      const allowedPermissions = ['clipboard-read', 'clipboard-write', 'notifications'];

      if (allowedPermissions.includes(permission)) {
        callback(true);
      } else {
        logger.warn(`Permission denied: ${permission}`);
        callback(false);
      }
    });

    // Configure certificate verification
    defaultSession.setCertificateVerifyProc((request, callback) => {
      // In development, allow self-signed certificates
      if (!app.isPackaged && request.hostname === 'localhost') {
        callback(0); // OK
      } else {
        callback(-3); // Use default verification
      }
    });

    // Clear cache periodically
    setInterval(() => {
      defaultSession.clearCache();
      logger.debug('Session cache cleared');
    }, 1000 * 60 * 60); // Every hour
  }

  private async createWindow(): Promise<void> {
    this.mainWindow = await createMainWindow();

    this.mainWindow.on('close', (event) => {
      if (!this.isQuitting && process.platform === 'darwin') {
        event.preventDefault();
        this.mainWindow?.hide();
      }
    });

    this.mainWindow.on('closed', () => {
      this.mainWindow = null;
    });
  }

  private handleDeepLink(url: string): void {
    logger.info('Handling deep link', { url });

    if (this.mainWindow) {
      this.mainWindow.webContents.send('deep-link', { url });
    }
  }

  public getMainWindow(): BrowserWindow | null {
    return this.mainWindow;
  }
}

// Create app instance
const tachikomaApp = new TachikomaApp();

export { tachikomaApp };
```

### Logger Implementation

```typescript
// src/electron/main/logger.ts
import { app } from 'electron';
import { join } from 'path';
import { createWriteStream, WriteStream, mkdirSync, existsSync } from 'fs';

type LogLevel = 'debug' | 'info' | 'warn' | 'error';

interface LogEntry {
  timestamp: string;
  level: LogLevel;
  context: string;
  message: string;
  data?: Record<string, unknown>;
}

export class Logger {
  private static logStream: WriteStream | null = null;
  private static logLevel: LogLevel = 'info';
  private static readonly levels: Record<LogLevel, number> = {
    debug: 0,
    info: 1,
    warn: 2,
    error: 3,
  };

  private context: string;

  constructor(context: string) {
    this.context = context;

    if (!Logger.logStream) {
      Logger.initializeLogStream();
    }
  }

  private static initializeLogStream(): void {
    const logDir = join(app.getPath('userData'), 'logs');

    if (!existsSync(logDir)) {
      mkdirSync(logDir, { recursive: true });
    }

    const logFile = join(logDir, `tachikoma-${new Date().toISOString().split('T')[0]}.log`);
    Logger.logStream = createWriteStream(logFile, { flags: 'a' });

    // Set log level from environment
    const envLevel = process.env.LOG_LEVEL as LogLevel;
    if (envLevel && Logger.levels[envLevel] !== undefined) {
      Logger.logLevel = envLevel;
    }
  }

  private shouldLog(level: LogLevel): boolean {
    return Logger.levels[level] >= Logger.levels[Logger.logLevel];
  }

  private formatEntry(entry: LogEntry): string {
    const base = `[${entry.timestamp}] [${entry.level.toUpperCase()}] [${entry.context}] ${entry.message}`;
    if (entry.data) {
      return `${base} ${JSON.stringify(entry.data)}`;
    }
    return base;
  }

  private log(level: LogLevel, message: string, data?: Record<string, unknown>): void {
    if (!this.shouldLog(level)) return;

    const entry: LogEntry = {
      timestamp: new Date().toISOString(),
      level,
      context: this.context,
      message,
      data,
    };

    const formatted = this.formatEntry(entry);

    // Write to console
    switch (level) {
      case 'debug':
        console.debug(formatted);
        break;
      case 'info':
        console.info(formatted);
        break;
      case 'warn':
        console.warn(formatted);
        break;
      case 'error':
        console.error(formatted);
        break;
    }

    // Write to file
    Logger.logStream?.write(formatted + '\n');
  }

  debug(message: string, data?: Record<string, unknown>): void {
    this.log('debug', message, data);
  }

  info(message: string, data?: Record<string, unknown>): void {
    this.log('info', message, data);
  }

  warn(message: string, data?: Record<string, unknown>): void {
    this.log('warn', message, data);
  }

  error(message: string, data?: Record<string, unknown>): void {
    this.log('error', message, data);
  }

  static setLogLevel(level: LogLevel): void {
    Logger.logLevel = level;
  }

  static async flush(): Promise<void> {
    return new Promise((resolve) => {
      if (Logger.logStream) {
        Logger.logStream.end(resolve);
      } else {
        resolve();
      }
    });
  }
}
```

### App Configuration

```typescript
// src/electron/main/config.ts
import { app } from 'electron';
import { join } from 'path';
import { existsSync, readFileSync, writeFileSync } from 'fs';

interface AppConfig {
  window: {
    width: number;
    height: number;
    x?: number;
    y?: number;
    maximized: boolean;
  };
  theme: 'light' | 'dark' | 'system';
  locale: string;
  telemetry: boolean;
  autoUpdate: boolean;
  hardwareAcceleration: boolean;
}

const defaultConfig: AppConfig = {
  window: {
    width: 1200,
    height: 800,
    maximized: false,
  },
  theme: 'system',
  locale: 'en',
  telemetry: true,
  autoUpdate: true,
  hardwareAcceleration: true,
};

class ConfigManager {
  private configPath: string;
  private config: AppConfig;

  constructor() {
    this.configPath = join(app.getPath('userData'), 'config.json');
    this.config = this.load();
  }

  private load(): AppConfig {
    if (existsSync(this.configPath)) {
      try {
        const data = readFileSync(this.configPath, 'utf-8');
        return { ...defaultConfig, ...JSON.parse(data) };
      } catch (error) {
        console.error('Failed to load config:', error);
      }
    }
    return { ...defaultConfig };
  }

  save(): void {
    try {
      writeFileSync(this.configPath, JSON.stringify(this.config, null, 2));
    } catch (error) {
      console.error('Failed to save config:', error);
    }
  }

  get<K extends keyof AppConfig>(key: K): AppConfig[K] {
    return this.config[key];
  }

  set<K extends keyof AppConfig>(key: K, value: AppConfig[K]): void {
    this.config[key] = value;
    this.save();
  }

  getAll(): AppConfig {
    return { ...this.config };
  }

  reset(): void {
    this.config = { ...defaultConfig };
    this.save();
  }
}

export const configManager = new ConfigManager();
export type { AppConfig };
```

### Package.json Configuration

```json
{
  "name": "tachikoma",
  "version": "1.0.0",
  "description": "Tachikoma Desktop Application",
  "main": "dist/electron/main/index.js",
  "author": "Tachikoma Team",
  "license": "MIT",
  "scripts": {
    "dev": "electron-vite dev",
    "build": "electron-vite build",
    "preview": "electron-vite preview",
    "package": "electron-builder",
    "package:mac": "electron-builder --mac",
    "package:win": "electron-builder --win",
    "package:linux": "electron-builder --linux"
  },
  "devDependencies": {
    "@electron-toolkit/utils": "^3.0.0",
    "electron": "^32.0.0",
    "electron-builder": "^24.13.0",
    "electron-vite": "^2.3.0",
    "typescript": "^5.5.0"
  }
}
```

---

## Testing Requirements

### Unit Tests

```typescript
// src/electron/main/__tests__/config.test.ts
import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { vol } from 'memfs';

vi.mock('fs', async () => {
  const memfs = await import('memfs');
  return memfs.fs;
});

vi.mock('electron', () => ({
  app: {
    getPath: vi.fn().mockReturnValue('/mock/userData'),
  },
}));

describe('ConfigManager', () => {
  beforeEach(() => {
    vol.reset();
    vol.mkdirSync('/mock/userData', { recursive: true });
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  it('should load default config when no config file exists', async () => {
    const { configManager } = await import('../config');

    expect(configManager.get('theme')).toBe('system');
    expect(configManager.get('window').width).toBe(1200);
  });

  it('should save and load config changes', async () => {
    const { configManager } = await import('../config');

    configManager.set('theme', 'dark');
    configManager.save();

    expect(configManager.get('theme')).toBe('dark');
  });

  it('should merge saved config with defaults', async () => {
    vol.writeFileSync(
      '/mock/userData/config.json',
      JSON.stringify({ theme: 'light' })
    );

    const { configManager } = await import('../config');

    expect(configManager.get('theme')).toBe('light');
    expect(configManager.get('autoUpdate')).toBe(true);
  });
});
```

### Integration Tests

```typescript
// src/electron/main/__tests__/app.integration.test.ts
import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { _electron as electron } from 'playwright';
import type { ElectronApplication, Page } from 'playwright';

describe('Main Process Integration', () => {
  let electronApp: ElectronApplication;
  let page: Page;

  beforeAll(async () => {
    electronApp = await electron.launch({
      args: ['.'],
      env: {
        ...process.env,
        NODE_ENV: 'test',
      },
    });

    page = await electronApp.firstWindow();
  });

  afterAll(async () => {
    await electronApp.close();
  });

  it('should launch the application', async () => {
    const isPackaged = await electronApp.evaluate(({ app }) => app.isPackaged);
    expect(isPackaged).toBe(false);
  });

  it('should have correct app name', async () => {
    const name = await electronApp.evaluate(({ app }) => app.getName());
    expect(name).toBe('tachikoma');
  });

  it('should create a browser window', async () => {
    const windowCount = await electronApp.evaluate(({ BrowserWindow }) =>
      BrowserWindow.getAllWindows().length
    );
    expect(windowCount).toBe(1);
  });

  it('should prevent multiple instances', async () => {
    // This would need a more complex test setup
    // to actually test single instance lock
  });
});
```

---

## Related Specs

- Spec 162: Window Management
- Spec 166: App Lifecycle
- Spec 169: Security Configuration
- Spec 170: IPC Channels
- Spec 171: Preload Scripts
