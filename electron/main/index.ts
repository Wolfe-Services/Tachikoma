import {
  app,
  BrowserWindow,
  session,
  protocol,
  powerMonitor,
  nativeTheme,
  shell,
  crashReporter
} from 'electron';
import { join } from 'path';
import { electronApp, optimizer, is } from '@electron-toolkit/utils';
import { registerIpcHandlers } from './ipc-handlers';
import { Logger } from './logger';
import { configManager } from './config';
import { registerProtocolSchemes, setupProtocolHandlers, protocolManager } from './protocol';
import { configureSessionProtocols } from './protocol/session';

const logger = new Logger('main');

// Security: Disable hardware acceleration if needed for security
if (!configManager.get('hardwareAcceleration')) {
  app.disableHardwareAcceleration();
}

// Security: Enable sandbox for all renderers
app.enableSandbox();

// Register protocol schemes before app ready
registerProtocolSchemes();

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
        if (!['https:', 'http:', 'file:', 'tachikoma:'].includes(parsedUrl.protocol)) {
          event.preventDefault();
          logger.warn(`Blocked navigation to: ${url}`);
        }
      });

      // Disable new window creation from renderer
      contents.setWindowOpenHandler(({ url }) => {
        // Open external links in default browser
        if (url.startsWith('https://') || url.startsWith('http://')) {
          shell.openExternal(url);
        }
        return { action: 'deny' };
      });

      // Disable remote module access
      contents.on('remote-require' as any, (event: any) => {
        event.preventDefault();
      });

      contents.on('remote-get-builtin' as any, (event: any) => {
        event.preventDefault();
      });

      contents.on('remote-get-global' as any, (event: any) => {
        event.preventDefault();
      });

      contents.on('remote-get-current-window' as any, (event: any) => {
        event.preventDefault();
      });

      contents.on('remote-get-current-web-contents' as any, (event: any) => {
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
    app.whenReady().then(() => this.onReady().catch(logger.error.bind(logger)));

    // Window all closed
    app.on('window-all-closed', () => {
      if (process.platform !== 'darwin') {
        app.quit();
      }
    });

    // Activate (macOS)
    app.on('activate', () => {
      if (BrowserWindow.getAllWindows().length === 0) {
        this.createWindow().catch(logger.error.bind(logger));
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

    // Error handling
    process.on('uncaughtException', (error) => {
      logger.error('Uncaught exception', { error: error.message, stack: error.stack });
      // Don't quit in development
      if (!is.dev) {
        app.quit();
      }
    });

    process.on('unhandledRejection', (reason) => {
      logger.error('Unhandled promise rejection', { reason });
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
      isDev: is.dev,
      isPackaged: app.isPackaged
    });

    // Set app user model id for Windows
    electronApp.setAppUserModelId('com.tachikoma.app');

    // Configure session
    this.configureSession();

    // Setup protocol handlers
    setupProtocolHandlers();
    configureSessionProtocols();

    // Setup crash reporter
    this.setupCrashReporter();

    // Create main window
    await this.createWindow();

    // Register IPC handlers
    registerIpcHandlers();

    // Watch for window shortcuts in development
    if (is.dev) {
      app.on('browser-window-created', (_, window) => {
        optimizer.watchWindowShortcuts(window);
      });

      // Enable remote debugging port for DevTools debugging
      app.commandLine.appendSwitch('remote-debugging-port', '9222');
    }

    // Memory management hints
    this.setupMemoryManagement();

    // Configure GPU process
    this.configureGPU();
  }

  private configureSession(): void {
    const defaultSession = session.defaultSession;

    // Configure Content Security Policy
    // In dev mode, allow inline scripts for SvelteKit HMR
    const scriptSrc = is.dev 
      ? "'self' 'unsafe-inline' 'unsafe-eval'" 
      : "'self' 'unsafe-eval'";
    const connectSrc = is.dev 
      ? "'self' http: https: ws: wss:" 
      : "'self' https: wss:";
    
    defaultSession.webRequest.onHeadersReceived((details, callback) => {
      callback({
        responseHeaders: {
          ...details.responseHeaders,
          'Content-Security-Policy': [
            "default-src 'self'; " +
            `script-src ${scriptSrc}; ` +
            "style-src 'self' 'unsafe-inline'; " +
            "img-src 'self' data: https: http:; " +
            "font-src 'self' data:; " +
            `connect-src ${connectSrc}; ` +
            "frame-src 'none';"
          ],
        },
      });
    });

    // Configure permissions
    defaultSession.setPermissionRequestHandler((_, permission, callback) => {
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
      // In development, allow self-signed certificates for localhost
      if (is.dev && request.hostname === 'localhost') {
        callback(0); // OK
      } else {
        callback(-3); // Use default verification
      }
    });

    // Clear cache periodically (every hour)
    setInterval(() => {
      defaultSession.clearCache();
      logger.debug('Session cache cleared');
    }, 1000 * 60 * 60);

    // Session cookie configuration
    defaultSession.cookies.on('changed', (event, cookie, cause, removed) => {
      logger.debug('Cookie changed', { cookie: cookie.name, cause, removed });
    });
  }

  private setupProtocolHandlers(): void {
    // Register custom protocol
    if (!protocol.isProtocolRegistered('tachikoma')) {
      protocol.registerFileProtocol('tachikoma', (request, callback) => {
        const url = request.url.substr('tachikoma://'.length);
        const filePath = join(__dirname, '../../resources', url);
        callback({ path: filePath });
      });
    }

    // Handle protocol on Windows/Linux
    app.setAsDefaultProtocolClient('tachikoma');

    // Handle protocol on macOS
    app.on('open-url', (event, url) => {
      event.preventDefault();
      this.handleDeepLink(url);
    });
  }

  private setupCrashReporter(): void {
    if (configManager.get('telemetry')) {
      crashReporter.start({
        productName: 'Tachikoma',
        companyName: 'Tachikoma Team',
        submitURL: 'https://crashes.tachikoma.io/submit',
        uploadToServer: app.isPackaged,
        ignoreSystemCrashHandler: false,
        rateLimit: true,
        compress: true
      });
      
      logger.info('Crash reporter enabled');
    }
  }

  private setupMemoryManagement(): void {
    // Garbage collection hints
    setInterval(() => {
      const memUsage = process.memoryUsage();
      logger.debug('Memory usage', {
        rss: Math.round(memUsage.rss / 1024 / 1024) + ' MB',
        heapUsed: Math.round(memUsage.heapUsed / 1024 / 1024) + ' MB',
        heapTotal: Math.round(memUsage.heapTotal / 1024 / 1024) + ' MB',
        external: Math.round(memUsage.external / 1024 / 1024) + ' MB'
      });

      // Force GC if memory usage is high
      if (global.gc && memUsage.heapUsed > 200 * 1024 * 1024) {
        global.gc();
        logger.info('Garbage collection triggered');
      }
    }, 60000); // Every minute

    // Clear renderer caches periodically
    setInterval(() => {
      this.mainWindow?.webContents.session.clearCache();
    }, 5 * 60 * 1000); // Every 5 minutes
  }

  private configureGPU(): void {
    // GPU process configuration for optimal rendering
    if (configManager.get('hardwareAcceleration')) {
      app.commandLine.appendSwitch('enable-gpu-rasterization');
      app.commandLine.appendSwitch('enable-zero-copy');
      app.commandLine.appendSwitch('ignore-gpu-blacklist');
      
      logger.info('Hardware acceleration enabled');
    } else {
      app.commandLine.appendSwitch('disable-gpu');
      app.commandLine.appendSwitch('disable-gpu-compositing');
      
      logger.info('Hardware acceleration disabled');
    }

    // Optimize for high DPI displays
    app.commandLine.appendSwitch('high-dpi-support', '1');
    app.commandLine.appendSwitch('force-device-scale-factor', '1');
  }

  private async createWindow(): Promise<void> {
    const windowConfig = configManager.get('window');

    this.mainWindow = new BrowserWindow({
      width: windowConfig.width,
      height: windowConfig.height,
      x: windowConfig.x,
      y: windowConfig.y,
      minWidth: 800,
      minHeight: 600,
      show: false,
      autoHideMenuBar: false,
      titleBarStyle: 'hiddenInset',
      trafficLightPosition: { x: 15, y: 10 },
      webPreferences: {
        preload: join(__dirname, '../preload/index.js'),
        sandbox: true,
        contextIsolation: true,
        nodeIntegration: false,
        webSecurity: true,
        devTools: is.dev,
        allowRunningInsecureContent: false,
        experimentalFeatures: false
      }
    });

    this.mainWindow.on('ready-to-show', () => {
      logger.info('ready-to-show event fired');
      if (windowConfig.maximized) {
        this.mainWindow?.maximize();
      }
      this.mainWindow?.show();
      
      // Open DevTools in development
      if (is.dev) {
        this.mainWindow?.webContents.openDevTools();
      }

      logger.info('Main window shown');
    });

    // Fallback: show window after timeout if ready-to-show doesn't fire
    setTimeout(() => {
      if (this.mainWindow && !this.mainWindow.isVisible()) {
        logger.warn('Window not visible after timeout, forcing show');
        this.mainWindow.show();
        if (is.dev) {
          this.mainWindow.webContents.openDevTools();
        }
      }
    }, 5000);

    this.mainWindow.on('close', (event) => {
      // Save window state
      const bounds = this.mainWindow?.getBounds();
      const isMaximized = this.mainWindow?.isMaximized() || false;
      
      if (bounds) {
        configManager.set('window', {
          ...bounds,
          maximized: isMaximized
        });
      }

      if (!this.isQuitting && process.platform === 'darwin') {
        event.preventDefault();
        this.mainWindow?.hide();
      }
    });

    this.mainWindow.on('closed', () => {
      this.mainWindow = null;
      logger.info('Main window closed');
    });

    // Log web contents errors
    this.mainWindow.webContents.on('did-fail-load', (event, errorCode, errorDescription, validatedURL) => {
      logger.error('Failed to load', { errorCode, errorDescription, validatedURL });
    });

    this.mainWindow.webContents.on('did-finish-load', () => {
      logger.info('Page finished loading');
    });

    // Load the renderer
    if (is.dev) {
      // In dev mode, load from the Vite dev server (port 5173 for web, 1420 for Tauri)
      const devServerUrl = process.env['ELECTRON_RENDERER_URL'] || 'http://localhost:5173';
      logger.info('Loading dev server URL', { devServerUrl });
      await this.mainWindow.loadURL(devServerUrl);
    } else {
      await this.mainWindow.loadFile(join(__dirname, '../../web/dist/index.html'));
    }
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