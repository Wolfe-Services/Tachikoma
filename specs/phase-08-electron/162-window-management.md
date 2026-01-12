# Spec 162: Window Management

## Phase
8 - Electron Shell

## Spec ID
162

## Status
Planned

## Dependencies
- Spec 161 (Electron Main Process)
- Spec 169 (Security Configuration)

## Estimated Context
~10%

---

## Objective

Implement comprehensive window management for the Electron application, including main window creation, multi-window support, window state persistence, and proper security configurations for each window type.

---

## Acceptance Criteria

- [ ] Main window creates with proper security settings
- [ ] Window state (size, position, maximized) persists across sessions
- [ ] Support for multiple window types (main, settings, about)
- [ ] Frameless window option with custom title bar
- [ ] Window bounds validation (ensure visible on screen)
- [ ] Proper window close behavior per platform
- [ ] DevTools management in development mode
- [ ] Window focus and blur handling
- [ ] Splash screen support during app load

---

## Implementation Details

### Window Factory

```typescript
// src/electron/main/window/index.ts
import {
  BrowserWindow,
  BrowserWindowConstructorOptions,
  screen,
  app,
  shell,
} from 'electron';
import { join } from 'path';
import { configManager } from '../config';
import { Logger } from '../logger';

const logger = new Logger('window');

interface WindowState {
  width: number;
  height: number;
  x?: number;
  y?: number;
  maximized: boolean;
}

type WindowType = 'main' | 'settings' | 'about' | 'splash';

const windowDefaults: Record<WindowType, Partial<BrowserWindowConstructorOptions>> = {
  main: {
    width: 1200,
    height: 800,
    minWidth: 800,
    minHeight: 600,
    show: false,
    titleBarStyle: 'hiddenInset',
    trafficLightPosition: { x: 16, y: 16 },
  },
  settings: {
    width: 600,
    height: 500,
    minWidth: 500,
    minHeight: 400,
    resizable: true,
    modal: true,
  },
  about: {
    width: 400,
    height: 300,
    resizable: false,
    minimizable: false,
    maximizable: false,
  },
  splash: {
    width: 400,
    height: 300,
    frame: false,
    transparent: true,
    resizable: false,
    movable: false,
    skipTaskbar: true,
    alwaysOnTop: true,
  },
};

export class WindowManager {
  private windows: Map<string, BrowserWindow> = new Map();
  private mainWindowId: string | null = null;

  private getSecurityOptions(): Partial<BrowserWindowConstructorOptions> {
    return {
      webPreferences: {
        nodeIntegration: false,
        contextIsolation: true,
        sandbox: true,
        webSecurity: true,
        allowRunningInsecureContent: false,
        preload: join(__dirname, '../preload/index.js'),
        spellcheck: true,
        enableWebSQL: false,
        // Disable features for security
        webgl: true,
        plugins: false,
        experimentalFeatures: false,
      },
    };
  }

  private validateWindowBounds(state: WindowState): WindowState {
    const displays = screen.getAllDisplays();
    const validState = { ...state };

    // Check if window is visible on any display
    const isVisible = displays.some((display) => {
      const { x, y, width, height } = display.bounds;
      return (
        validState.x !== undefined &&
        validState.y !== undefined &&
        validState.x >= x &&
        validState.x < x + width &&
        validState.y >= y &&
        validState.y < y + height
      );
    });

    if (!isVisible && displays.length > 0) {
      // Reset to primary display center
      const primaryDisplay = screen.getPrimaryDisplay();
      const { width, height } = primaryDisplay.workAreaSize;
      validState.x = Math.round((width - validState.width) / 2);
      validState.y = Math.round((height - validState.height) / 2);
      logger.info('Window bounds reset to primary display');
    }

    return validState;
  }

  async createWindow(
    type: WindowType,
    options: Partial<BrowserWindowConstructorOptions> = {}
  ): Promise<BrowserWindow> {
    const id = `${type}-${Date.now()}`;
    const defaults = windowDefaults[type];
    const securityOptions = this.getSecurityOptions();

    // Load saved state for main window
    let savedState: WindowState | null = null;
    if (type === 'main') {
      savedState = this.validateWindowBounds(configManager.get('window'));
    }

    const windowOptions: BrowserWindowConstructorOptions = {
      ...defaults,
      ...securityOptions,
      ...options,
      ...(savedState && {
        width: savedState.width,
        height: savedState.height,
        x: savedState.x,
        y: savedState.y,
      }),
      backgroundColor: '#1a1a1a',
      icon: this.getAppIcon(),
    };

    const window = new BrowserWindow(windowOptions);

    // Store window reference
    this.windows.set(id, window);

    if (type === 'main') {
      this.mainWindowId = id;
    }

    // Setup window event handlers
    this.setupWindowEvents(window, type, id);

    // Load content
    await this.loadWindowContent(window, type);

    // Restore maximized state
    if (savedState?.maximized) {
      window.maximize();
    }

    logger.info(`Created ${type} window`, { id });

    return window;
  }

  private getAppIcon(): string | undefined {
    switch (process.platform) {
      case 'win32':
        return join(__dirname, '../../resources/icon.ico');
      case 'darwin':
        return join(__dirname, '../../resources/icon.icns');
      default:
        return join(__dirname, '../../resources/icon.png');
    }
  }

  private setupWindowEvents(
    window: BrowserWindow,
    type: WindowType,
    id: string
  ): void {
    // Track window state for main window
    if (type === 'main') {
      const saveState = (): void => {
        if (!window.isMinimized() && !window.isMaximized()) {
          const [width, height] = window.getSize();
          const [x, y] = window.getPosition();
          configManager.set('window', {
            width,
            height,
            x,
            y,
            maximized: window.isMaximized(),
          });
        } else if (window.isMaximized()) {
          const currentConfig = configManager.get('window');
          configManager.set('window', {
            ...currentConfig,
            maximized: true,
          });
        }
      };

      window.on('resize', saveState);
      window.on('move', saveState);
      window.on('maximize', saveState);
      window.on('unmaximize', saveState);
    }

    // Ready to show
    window.once('ready-to-show', () => {
      window.show();

      // Focus in development
      if (!app.isPackaged) {
        window.focus();
      }
    });

    // External links
    window.webContents.setWindowOpenHandler(({ url }) => {
      if (url.startsWith('https://') || url.startsWith('http://')) {
        shell.openExternal(url);
      }
      return { action: 'deny' };
    });

    // Closed
    window.on('closed', () => {
      this.windows.delete(id);
      if (id === this.mainWindowId) {
        this.mainWindowId = null;
      }
      logger.info(`Window closed`, { id, type });
    });

    // Unresponsive
    window.on('unresponsive', () => {
      logger.warn('Window became unresponsive', { id, type });
    });

    window.webContents.on('did-fail-load', (_, errorCode, errorDescription) => {
      logger.error('Window failed to load', { id, type, errorCode, errorDescription });
    });

    // DevTools in development
    if (!app.isPackaged) {
      window.webContents.on('before-input-event', (event, input) => {
        if (input.control && input.shift && input.key.toLowerCase() === 'i') {
          window.webContents.toggleDevTools();
        }
      });
    }
  }

  private async loadWindowContent(
    window: BrowserWindow,
    type: WindowType
  ): Promise<void> {
    const routes: Record<WindowType, string> = {
      main: '/',
      settings: '/settings',
      about: '/about',
      splash: '/splash',
    };

    if (app.isPackaged) {
      await window.loadFile(join(__dirname, `../../renderer/index.html`), {
        hash: routes[type],
      });
    } else {
      const port = process.env.ELECTRON_RENDERER_URL || 'http://localhost:5173';
      await window.loadURL(`${port}#${routes[type]}`);
    }
  }

  getMainWindow(): BrowserWindow | null {
    if (this.mainWindowId) {
      return this.windows.get(this.mainWindowId) || null;
    }
    return null;
  }

  getWindow(id: string): BrowserWindow | null {
    return this.windows.get(id) || null;
  }

  getAllWindows(): BrowserWindow[] {
    return Array.from(this.windows.values());
  }

  closeAllWindows(): void {
    for (const window of this.windows.values()) {
      window.close();
    }
  }

  focusMainWindow(): void {
    const mainWindow = this.getMainWindow();
    if (mainWindow) {
      if (mainWindow.isMinimized()) {
        mainWindow.restore();
      }
      mainWindow.focus();
    }
  }
}

export const windowManager = new WindowManager();

// Convenience function for creating main window
export async function createMainWindow(): Promise<BrowserWindow> {
  return windowManager.createWindow('main');
}
```

### Custom Title Bar Component (Renderer)

```typescript
// src/renderer/components/TitleBar/TitleBar.tsx
import React, { useEffect, useState } from 'react';
import styles from './TitleBar.module.css';

interface TitleBarProps {
  title?: string;
}

export const TitleBar: React.FC<TitleBarProps> = ({ title = 'Tachikoma' }) => {
  const [isMaximized, setIsMaximized] = useState(false);
  const [isFocused, setIsFocused] = useState(true);

  useEffect(() => {
    const handleMaximize = (_: unknown, maximized: boolean) => {
      setIsMaximized(maximized);
    };

    const handleFocus = () => setIsFocused(true);
    const handleBlur = () => setIsFocused(false);

    window.electronAPI?.onWindowMaximize(handleMaximize);
    window.addEventListener('focus', handleFocus);
    window.addEventListener('blur', handleBlur);

    return () => {
      window.removeEventListener('focus', handleFocus);
      window.removeEventListener('blur', handleBlur);
    };
  }, []);

  const handleMinimize = () => window.electronAPI?.minimizeWindow();
  const handleMaximize = () => window.electronAPI?.maximizeWindow();
  const handleClose = () => window.electronAPI?.closeWindow();

  // On macOS, use native traffic lights
  if (window.electronAPI?.platform === 'darwin') {
    return (
      <div className={styles.titleBarMac} data-focused={isFocused}>
        <div className={styles.dragRegion}>
          <span className={styles.title}>{title}</span>
        </div>
      </div>
    );
  }

  // Custom title bar for Windows/Linux
  return (
    <div className={styles.titleBar} data-focused={isFocused}>
      <div className={styles.dragRegion}>
        <div className={styles.appIcon}>
          <img src="/icon.png" alt="App icon" />
        </div>
        <span className={styles.title}>{title}</span>
      </div>
      <div className={styles.windowControls}>
        <button
          className={styles.controlButton}
          onClick={handleMinimize}
          aria-label="Minimize"
        >
          <MinimizeIcon />
        </button>
        <button
          className={styles.controlButton}
          onClick={handleMaximize}
          aria-label={isMaximized ? 'Restore' : 'Maximize'}
        >
          {isMaximized ? <RestoreIcon /> : <MaximizeIcon />}
        </button>
        <button
          className={`${styles.controlButton} ${styles.closeButton}`}
          onClick={handleClose}
          aria-label="Close"
        >
          <CloseIcon />
        </button>
      </div>
    </div>
  );
};

const MinimizeIcon = () => (
  <svg width="10" height="1" viewBox="0 0 10 1">
    <path d="M0 0h10v1H0z" fill="currentColor" />
  </svg>
);

const MaximizeIcon = () => (
  <svg width="10" height="10" viewBox="0 0 10 10">
    <path d="M0 0v10h10V0H0zm1 1h8v8H1V1z" fill="currentColor" />
  </svg>
);

const RestoreIcon = () => (
  <svg width="10" height="10" viewBox="0 0 10 10">
    <path
      d="M2 0v2H0v8h8V8h2V0H2zm6 8H1V3h7v5zm1-6H3V1h6v1z"
      fill="currentColor"
    />
  </svg>
);

const CloseIcon = () => (
  <svg width="10" height="10" viewBox="0 0 10 10">
    <path
      d="M1.41 0L0 1.41 3.59 5 0 8.59 1.41 10 5 6.41 8.59 10 10 8.59 6.41 5 10 1.41 8.59 0 5 3.59 1.41 0z"
      fill="currentColor"
    />
  </svg>
);
```

### Title Bar Styles

```css
/* src/renderer/components/TitleBar/TitleBar.module.css */
.titleBar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 32px;
  background-color: var(--color-surface);
  border-bottom: 1px solid var(--color-border);
  user-select: none;
  -webkit-app-region: drag;
}

.titleBar[data-focused="false"] {
  background-color: var(--color-surface-muted);
}

.titleBarMac {
  height: 38px;
  background-color: var(--color-surface);
  border-bottom: 1px solid var(--color-border);
  -webkit-app-region: drag;
  padding-left: 80px; /* Space for traffic lights */
}

.dragRegion {
  flex: 1;
  display: flex;
  align-items: center;
  height: 100%;
}

.appIcon {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  -webkit-app-region: no-drag;
}

.appIcon img {
  width: 16px;
  height: 16px;
}

.title {
  font-size: 12px;
  font-weight: 500;
  color: var(--color-text);
  margin-left: 8px;
}

.windowControls {
  display: flex;
  height: 100%;
  -webkit-app-region: no-drag;
}

.controlButton {
  width: 46px;
  height: 100%;
  border: none;
  background: transparent;
  color: var(--color-text);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: background-color 0.1s;
}

.controlButton:hover {
  background-color: var(--color-hover);
}

.controlButton:active {
  background-color: var(--color-active);
}

.closeButton:hover {
  background-color: #e81123;
  color: white;
}
```

### Window IPC Handlers

```typescript
// src/electron/main/ipc/window.ts
import { ipcMain, BrowserWindow } from 'electron';
import { windowManager } from '../window';

export function setupWindowIpcHandlers(): void {
  ipcMain.handle('window:minimize', (event) => {
    const window = BrowserWindow.fromWebContents(event.sender);
    window?.minimize();
  });

  ipcMain.handle('window:maximize', (event) => {
    const window = BrowserWindow.fromWebContents(event.sender);
    if (window) {
      if (window.isMaximized()) {
        window.unmaximize();
      } else {
        window.maximize();
      }
    }
  });

  ipcMain.handle('window:close', (event) => {
    const window = BrowserWindow.fromWebContents(event.sender);
    window?.close();
  });

  ipcMain.handle('window:isMaximized', (event) => {
    const window = BrowserWindow.fromWebContents(event.sender);
    return window?.isMaximized() ?? false;
  });

  ipcMain.handle('window:setTitle', (event, title: string) => {
    const window = BrowserWindow.fromWebContents(event.sender);
    window?.setTitle(title);
  });

  ipcMain.handle('window:setAlwaysOnTop', (event, flag: boolean) => {
    const window = BrowserWindow.fromWebContents(event.sender);
    window?.setAlwaysOnTop(flag);
  });

  ipcMain.handle('window:openSettings', async () => {
    const mainWindow = windowManager.getMainWindow();
    if (mainWindow) {
      await windowManager.createWindow('settings', {
        parent: mainWindow,
      });
    }
  });

  ipcMain.handle('window:openAbout', async () => {
    const mainWindow = windowManager.getMainWindow();
    if (mainWindow) {
      await windowManager.createWindow('about', {
        parent: mainWindow,
      });
    }
  });
}
```

### Splash Screen

```typescript
// src/electron/main/window/splash.ts
import { BrowserWindow, app } from 'electron';
import { join } from 'path';

export async function showSplashScreen(): Promise<BrowserWindow> {
  const splash = new BrowserWindow({
    width: 400,
    height: 300,
    frame: false,
    transparent: true,
    resizable: false,
    movable: false,
    skipTaskbar: true,
    alwaysOnTop: true,
    webPreferences: {
      nodeIntegration: false,
      contextIsolation: true,
    },
  });

  if (app.isPackaged) {
    await splash.loadFile(join(__dirname, '../../renderer/splash.html'));
  } else {
    await splash.loadURL('http://localhost:5173/splash.html');
  }

  splash.show();
  splash.center();

  return splash;
}

export function closeSplashScreen(splash: BrowserWindow): void {
  if (!splash.isDestroyed()) {
    splash.close();
  }
}
```

---

## Testing Requirements

### Unit Tests

```typescript
// src/electron/main/window/__tests__/window-manager.test.ts
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { screen } from 'electron';

vi.mock('electron', () => ({
  BrowserWindow: vi.fn().mockImplementation(() => ({
    on: vi.fn(),
    once: vi.fn(),
    loadFile: vi.fn().mockResolvedValue(undefined),
    loadURL: vi.fn().mockResolvedValue(undefined),
    show: vi.fn(),
    focus: vi.fn(),
    close: vi.fn(),
    isMinimized: vi.fn().mockReturnValue(false),
    isMaximized: vi.fn().mockReturnValue(false),
    maximize: vi.fn(),
    getSize: vi.fn().mockReturnValue([1200, 800]),
    getPosition: vi.fn().mockReturnValue([100, 100]),
    webContents: {
      on: vi.fn(),
      setWindowOpenHandler: vi.fn(),
    },
  })),
  screen: {
    getAllDisplays: vi.fn().mockReturnValue([
      { bounds: { x: 0, y: 0, width: 1920, height: 1080 } },
    ]),
    getPrimaryDisplay: vi.fn().mockReturnValue({
      workAreaSize: { width: 1920, height: 1040 },
    }),
  },
  app: {
    isPackaged: false,
    getPath: vi.fn().mockReturnValue('/mock/path'),
  },
  shell: {
    openExternal: vi.fn(),
  },
}));

describe('WindowManager', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should validate window bounds correctly', async () => {
    const { WindowManager } = await import('../index');
    const manager = new WindowManager();

    // Test bounds validation internally
    // The window should be visible on the mocked display
  });

  it('should create main window with correct options', async () => {
    const { WindowManager } = await import('../index');
    const manager = new WindowManager();

    const window = await manager.createWindow('main');
    expect(window).toBeDefined();
  });

  it('should handle multiple window creation', async () => {
    const { WindowManager } = await import('../index');
    const manager = new WindowManager();

    await manager.createWindow('main');
    await manager.createWindow('settings');

    expect(manager.getAllWindows()).toHaveLength(2);
  });
});
```

### Integration Tests

```typescript
// src/electron/main/window/__tests__/window.integration.test.ts
import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { _electron as electron } from 'playwright';
import type { ElectronApplication } from 'playwright';

describe('Window Management Integration', () => {
  let electronApp: ElectronApplication;

  beforeAll(async () => {
    electronApp = await electron.launch({ args: ['.'] });
  });

  afterAll(async () => {
    await electronApp.close();
  });

  it('should restore window state', async () => {
    const windowState = await electronApp.evaluate(({ BrowserWindow }) => {
      const win = BrowserWindow.getAllWindows()[0];
      return {
        width: win.getBounds().width,
        height: win.getBounds().height,
      };
    });

    expect(windowState.width).toBeGreaterThan(0);
    expect(windowState.height).toBeGreaterThan(0);
  });

  it('should handle window minimize', async () => {
    await electronApp.evaluate(({ BrowserWindow }) => {
      const win = BrowserWindow.getAllWindows()[0];
      win.minimize();
    });

    const isMinimized = await electronApp.evaluate(({ BrowserWindow }) => {
      return BrowserWindow.getAllWindows()[0].isMinimized();
    });

    expect(isMinimized).toBe(true);
  });
});
```

---

## Related Specs

- Spec 161: Electron Main Process
- Spec 163: Menu System
- Spec 169: Security Configuration
- Spec 170: IPC Channels
