# Spec 183: System Tray Integration

## Phase
8 - Electron Shell

## Spec ID
183

## Status
Planned

## Dependencies
- Spec 161 (Electron Main Process)
- Spec 163 (Menu System)

## Estimated Context
~8%

---

## Objective

Implement system tray integration for the Tachikoma application, providing quick access to common actions, status indicators, and background operation support across all platforms.

---

## Acceptance Criteria

- [ ] System tray icon display
- [ ] Context menu on tray icon
- [ ] Click/double-click handlers
- [ ] Dynamic icon updates (status indicators)
- [ ] Tooltip display
- [ ] Platform-specific behavior (macOS menu bar)
- [ ] Minimize to tray option
- [ ] Notification badge support
- [ ] Animation support for status changes

---

## Implementation Details

### Tray Manager

```typescript
// src/electron/main/tray/index.ts
import {
  Tray,
  Menu,
  MenuItemConstructorOptions,
  nativeImage,
  app,
  BrowserWindow,
  nativeTheme,
} from 'electron';
import { join } from 'path';
import { Logger } from '../logger';
import { configManager } from '../config';

const logger = new Logger('tray');

type TrayStatus = 'idle' | 'syncing' | 'error' | 'offline' | 'notification';

interface TrayConfig {
  showInTray: boolean;
  minimizeToTray: boolean;
  closeToTray: boolean;
  showBadge: boolean;
}

class TrayManager {
  private tray: Tray | null = null;
  private mainWindow: BrowserWindow | null = null;
  private status: TrayStatus = 'idle';
  private badgeCount: number = 0;
  private animationInterval: NodeJS.Timeout | null = null;
  private config: TrayConfig = {
    showInTray: true,
    minimizeToTray: true,
    closeToTray: process.platform === 'darwin',
    showBadge: true,
  };

  setMainWindow(window: BrowserWindow): void {
    this.mainWindow = window;
  }

  create(): void {
    if (this.tray) {
      return;
    }

    const iconPath = this.getIconPath('idle');
    const icon = nativeImage.createFromPath(iconPath);

    // Resize for tray (16x16 on macOS, 16x16-32x32 on Windows/Linux)
    const resized = icon.resize({
      width: 16,
      height: 16,
    });

    this.tray = new Tray(resized);

    // Set tooltip
    this.tray.setToolTip('Tachikoma');

    // Setup event handlers
    this.setupEventHandlers();

    // Build initial menu
    this.updateMenu();

    logger.info('Tray created');
  }

  private getIconPath(status: TrayStatus): string {
    const isDark = nativeTheme.shouldUseDarkColors;
    const platform = process.platform;

    // macOS uses template images (auto-adapts to dark/light)
    if (platform === 'darwin') {
      return join(__dirname, `../../resources/tray/iconTemplate.png`);
    }

    // Windows/Linux use themed icons
    const theme = isDark ? 'dark' : 'light';
    const statusSuffix = status === 'idle' ? '' : `-${status}`;

    return join(__dirname, `../../resources/tray/icon-${theme}${statusSuffix}.png`);
  }

  private setupEventHandlers(): void {
    if (!this.tray) return;

    // Click handler
    this.tray.on('click', () => {
      this.handleClick();
    });

    // Double-click handler (Windows)
    this.tray.on('double-click', () => {
      this.handleDoubleClick();
    });

    // Right-click handler (shows menu on all platforms)
    this.tray.on('right-click', () => {
      this.showContextMenu();
    });

    // Balloon click (Windows)
    this.tray.on('balloon-click', () => {
      this.showWindow();
    });

    // Theme change
    nativeTheme.on('updated', () => {
      this.updateIcon();
    });
  }

  private handleClick(): void {
    // On macOS, click opens the menu
    // On Windows/Linux, click toggles window
    if (process.platform === 'darwin') {
      this.showContextMenu();
    } else {
      this.toggleWindow();
    }
  }

  private handleDoubleClick(): void {
    this.showWindow();
  }

  private showContextMenu(): void {
    if (this.tray) {
      this.tray.popUpContextMenu();
    }
  }

  private toggleWindow(): void {
    if (!this.mainWindow) return;

    if (this.mainWindow.isVisible()) {
      if (this.mainWindow.isFocused()) {
        this.hideWindow();
      } else {
        this.mainWindow.focus();
      }
    } else {
      this.showWindow();
    }
  }

  private showWindow(): void {
    if (!this.mainWindow) return;

    if (this.mainWindow.isMinimized()) {
      this.mainWindow.restore();
    }
    this.mainWindow.show();
    this.mainWindow.focus();
  }

  private hideWindow(): void {
    if (!this.mainWindow) return;

    if (process.platform === 'darwin') {
      app.hide();
    } else {
      this.mainWindow.hide();
    }
  }

  updateMenu(customItems?: MenuItemConstructorOptions[]): void {
    if (!this.tray) return;

    const template: MenuItemConstructorOptions[] = [
      {
        label: this.getStatusLabel(),
        enabled: false,
      },
      { type: 'separator' },
      {
        label: 'Show Tachikoma',
        click: () => this.showWindow(),
      },
      {
        label: 'Hide Tachikoma',
        click: () => this.hideWindow(),
        visible: process.platform === 'darwin',
      },
      { type: 'separator' },
      {
        label: 'Quick Actions',
        submenu: [
          {
            label: 'New Project',
            click: () => {
              this.showWindow();
              this.mainWindow?.webContents.send('project:new');
            },
          },
          {
            label: 'Open Project...',
            click: () => {
              this.showWindow();
              this.mainWindow?.webContents.send('project:openDialog');
            },
          },
        ],
      },
      ...(customItems || []),
      { type: 'separator' },
      {
        label: 'Preferences',
        click: () => {
          this.showWindow();
          this.mainWindow?.webContents.send('preferences:open');
        },
      },
      { type: 'separator' },
      {
        label: 'Quit Tachikoma',
        click: () => {
          app.quit();
        },
      },
    ];

    const contextMenu = Menu.buildFromTemplate(template);
    this.tray.setContextMenu(contextMenu);
  }

  private getStatusLabel(): string {
    switch (this.status) {
      case 'syncing':
        return 'Syncing...';
      case 'error':
        return 'Error - Click for details';
      case 'offline':
        return 'Offline';
      case 'notification':
        return `${this.badgeCount} notifications`;
      default:
        return 'Tachikoma';
    }
  }

  setStatus(status: TrayStatus): void {
    this.status = status;
    this.updateIcon();
    this.updateMenu();

    // Start animation for syncing status
    if (status === 'syncing') {
      this.startAnimation();
    } else {
      this.stopAnimation();
    }

    logger.debug('Tray status updated', { status });
  }

  private updateIcon(): void {
    if (!this.tray) return;

    const iconPath = this.getIconPath(this.status);
    const icon = nativeImage.createFromPath(iconPath);
    const resized = icon.resize({ width: 16, height: 16 });

    this.tray.setImage(resized);
  }

  private startAnimation(): void {
    if (this.animationInterval) return;

    let frame = 0;
    const frames = ['syncing-1', 'syncing-2', 'syncing-3'];

    this.animationInterval = setInterval(() => {
      const iconPath = join(
        __dirname,
        `../../resources/tray/icon-${frames[frame]}.png`
      );
      const icon = nativeImage.createFromPath(iconPath);
      this.tray?.setImage(icon.resize({ width: 16, height: 16 }));

      frame = (frame + 1) % frames.length;
    }, 300);
  }

  private stopAnimation(): void {
    if (this.animationInterval) {
      clearInterval(this.animationInterval);
      this.animationInterval = null;
    }
  }

  setBadgeCount(count: number): void {
    this.badgeCount = count;

    if (count > 0 && this.config.showBadge) {
      this.setStatus('notification');

      // Platform-specific badge
      if (process.platform === 'darwin') {
        app.dock?.setBadge(count > 99 ? '99+' : String(count));
      }
    } else {
      this.setStatus('idle');

      if (process.platform === 'darwin') {
        app.dock?.setBadge('');
      }
    }

    this.updateMenu();
  }

  showBalloon(title: string, content: string): void {
    if (!this.tray || process.platform !== 'win32') return;

    this.tray.displayBalloon({
      title,
      content,
      iconType: 'info',
    });
  }

  setConfig(config: Partial<TrayConfig>): void {
    this.config = { ...this.config, ...config };

    if (!this.config.showInTray) {
      this.destroy();
    } else if (!this.tray) {
      this.create();
    }
  }

  getConfig(): TrayConfig {
    return { ...this.config };
  }

  destroy(): void {
    this.stopAnimation();

    if (this.tray) {
      this.tray.destroy();
      this.tray = null;
      logger.info('Tray destroyed');
    }
  }

  isCreated(): boolean {
    return this.tray !== null;
  }
}

export const trayManager = new TrayManager();
```

### Tray IPC Handlers

```typescript
// src/electron/main/ipc/tray.ts
import { ipcMain } from 'electron';
import { trayManager } from '../tray';

export function setupTrayIpcHandlers(): void {
  ipcMain.handle('tray:setStatus', (_, status: string) => {
    trayManager.setStatus(status as any);
  });

  ipcMain.handle('tray:setBadgeCount', (_, count: number) => {
    trayManager.setBadgeCount(count);
  });

  ipcMain.handle('tray:showBalloon', (_, title: string, content: string) => {
    trayManager.showBalloon(title, content);
  });

  ipcMain.handle('tray:setConfig', (_, config: any) => {
    trayManager.setConfig(config);
  });

  ipcMain.handle('tray:getConfig', () => {
    return trayManager.getConfig();
  });
}
```

---

## Testing Requirements

### Unit Tests

```typescript
// src/electron/main/tray/__tests__/tray.test.ts
import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('electron', () => ({
  Tray: vi.fn().mockImplementation(() => ({
    on: vi.fn(),
    setToolTip: vi.fn(),
    setContextMenu: vi.fn(),
    setImage: vi.fn(),
    popUpContextMenu: vi.fn(),
    displayBalloon: vi.fn(),
    destroy: vi.fn(),
  })),
  Menu: {
    buildFromTemplate: vi.fn().mockReturnValue({}),
  },
  nativeImage: {
    createFromPath: vi.fn().mockReturnValue({
      resize: vi.fn().mockReturnThis(),
    }),
  },
  nativeTheme: {
    shouldUseDarkColors: false,
    on: vi.fn(),
  },
  app: {
    dock: {
      setBadge: vi.fn(),
    },
    quit: vi.fn(),
    hide: vi.fn(),
  },
}));

describe('TrayManager', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should create tray', async () => {
    const { Tray } = await import('electron');
    const { trayManager } = await import('../index');

    trayManager.create();
    expect(Tray).toHaveBeenCalled();
    expect(trayManager.isCreated()).toBe(true);
  });

  it('should update status', async () => {
    const { trayManager } = await import('../index');

    trayManager.create();
    trayManager.setStatus('syncing');
    // Status should be updated
  });

  it('should set badge count', async () => {
    const { app } = await import('electron');
    const { trayManager } = await import('../index');

    trayManager.create();
    trayManager.setBadgeCount(5);

    expect(app.dock?.setBadge).toHaveBeenCalled();
  });
});
```

---

## Related Specs

- Spec 161: Electron Main Process
- Spec 163: Menu System
- Spec 184: Notifications
