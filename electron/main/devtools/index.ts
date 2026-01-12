import {
  BrowserWindow,
  session,
  app,
  ipcMain,
  WebContents,
} from 'electron';
import { join } from 'path';
import { existsSync, mkdirSync, writeFileSync, readFileSync } from 'fs';
import { Logger } from '../logger';

const logger = new Logger('devtools');

interface DevToolsConfig {
  openOnStart: boolean;
  position: 'right' | 'bottom' | 'undocked' | 'detach';
  width: number;
  height: number;
}

interface ExtensionInfo {
  name: string;
  path: string;
  version: string;
}

class DevToolsManager {
  private config: DevToolsConfig = {
    openOnStart: false,
    position: 'right',
    width: 400,
    height: 600,
  };
  private installedExtensions: ExtensionInfo[] = [];
  private extensionsPath: string;

  constructor() {
    this.extensionsPath = join(app.getPath('userData'), 'devtools-extensions');
    this.ensureExtensionsDirectory();
    this.loadConfig();
  }

  private ensureExtensionsDirectory(): void {
    if (!existsSync(this.extensionsPath)) {
      mkdirSync(this.extensionsPath, { recursive: true });
    }
  }

  private loadConfig(): void {
    const configPath = join(app.getPath('userData'), 'devtools-config.json');
    try {
      if (existsSync(configPath)) {
        this.config = JSON.parse(readFileSync(configPath, 'utf-8'));
      }
    } catch (error) {
      logger.error('Failed to load DevTools config', { error });
    }
  }

  private saveConfig(): void {
    const configPath = join(app.getPath('userData'), 'devtools-config.json');
    try {
      writeFileSync(configPath, JSON.stringify(this.config, null, 2));
    } catch (error) {
      logger.error('Failed to save DevTools config', { error });
    }
  }

  async loadExtensions(): Promise<void> {
    if (app.isPackaged) {
      logger.info('DevTools extensions disabled in production');
      return;
    }

    const ses = session.defaultSession;

    // Load React DevTools
    await this.loadExtension(ses, 'react-devtools', 'REACT_DEVTOOLS_PATH');

    // Load Redux DevTools
    await this.loadExtension(ses, 'redux-devtools', 'REDUX_DEVTOOLS_PATH');

    logger.info('DevTools extensions loaded', {
      extensions: this.installedExtensions.map((e) => e.name),
    });
  }

  private async loadExtension(
    ses: Electron.Session,
    name: string,
    envVar: string
  ): Promise<void> {
    const extensionPath = process.env[envVar];

    if (!extensionPath) {
      logger.debug(`${name} path not set via ${envVar}`);
      return;
    }

    try {
      const extension = await ses.loadExtension(extensionPath, {
        allowFileAccess: true,
      });

      this.installedExtensions.push({
        name,
        path: extensionPath,
        version: extension.version,
      });

      logger.info(`Loaded ${name} extension`, { version: extension.version });
    } catch (error) {
      logger.error(`Failed to load ${name} extension`, { error });
    }
  }

  openDevTools(window: BrowserWindow, options?: Partial<DevToolsConfig>): void {
    const webContents = window.webContents;

    if (webContents.isDevToolsOpened()) {
      webContents.devToolsWebContents?.focus();
      return;
    }

    const mode = options?.position || this.config.position;

    webContents.openDevTools({ mode });
    logger.debug('DevTools opened', { mode });
  }

  closeDevTools(window: BrowserWindow): void {
    if (window.webContents.isDevToolsOpened()) {
      window.webContents.closeDevTools();
      logger.debug('DevTools closed');
    }
  }

  toggleDevTools(window: BrowserWindow): void {
    const webContents = window.webContents;

    if (webContents.isDevToolsOpened()) {
      this.closeDevTools(window);
    } else {
      this.openDevTools(window);
    }
  }

  isDevToolsOpened(window: BrowserWindow): boolean {
    return window.webContents.isDevToolsOpened();
  }

  focusDevTools(window: BrowserWindow): void {
    window.webContents.devToolsWebContents?.focus();
  }

  setDevToolsPosition(position: DevToolsConfig['position']): void {
    this.config.position = position;
    this.saveConfig();
  }

  getInstalledExtensions(): ExtensionInfo[] {
    return [...this.installedExtensions];
  }

  // Inspect element at position
  inspectElement(window: BrowserWindow, x: number, y: number): void {
    window.webContents.inspectElement(x, y);
  }

  // Inspect service worker
  inspectServiceWorker(window: BrowserWindow): void {
    window.webContents.inspectServiceWorker();
  }

  // Inspect shared worker
  inspectSharedWorker(window: BrowserWindow): void {
    window.webContents.inspectSharedWorker();
  }
}

export const devToolsManager = new DevToolsManager();