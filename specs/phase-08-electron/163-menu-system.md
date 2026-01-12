# Spec 163: Menu System

## Phase
8 - Electron Shell

## Spec ID
163

## Status
Planned

## Dependencies
- Spec 161 (Electron Main Process)
- Spec 162 (Window Management)
- Spec 170 (IPC Channels)

## Estimated Context
~9%

---

## Objective

Implement a comprehensive menu system for the Electron application including application menu, context menus, and dynamic menu updates. Support platform-specific menu conventions and keyboard shortcuts.

---

## Acceptance Criteria

- [ ] Application menu follows platform conventions (macOS vs Windows/Linux)
- [ ] All menu items have keyboard shortcuts
- [ ] Context menus for common interactions
- [ ] Dynamic menu updates based on app state
- [ ] Menu item enable/disable based on context
- [ ] Recent files menu integration
- [ ] Edit menu with undo/redo support
- [ ] View menu with zoom and fullscreen
- [ ] Help menu with documentation links
- [ ] Developer menu in development mode

---

## Implementation Details

### Menu Builder

```typescript
// src/electron/main/menu/index.ts
import {
  Menu,
  MenuItem,
  MenuItemConstructorOptions,
  BrowserWindow,
  app,
  shell,
  dialog,
} from 'electron';
import { Logger } from '../logger';

const logger = new Logger('menu');

type MenuTemplate = (MenuItemConstructorOptions | MenuItem)[];

interface MenuState {
  canUndo: boolean;
  canRedo: boolean;
  hasSelection: boolean;
  isFullScreen: boolean;
  recentFiles: string[];
}

export class MenuBuilder {
  private mainWindow: BrowserWindow;
  private state: MenuState = {
    canUndo: false,
    canRedo: false,
    hasSelection: false,
    isFullScreen: false,
    recentFiles: [],
  };

  constructor(mainWindow: BrowserWindow) {
    this.mainWindow = mainWindow;
  }

  updateState(newState: Partial<MenuState>): void {
    this.state = { ...this.state, ...newState };
    this.buildMenu();
  }

  buildMenu(): Menu {
    const template = this.buildTemplate();
    const menu = Menu.buildFromTemplate(template);
    Menu.setApplicationMenu(menu);
    return menu;
  }

  private buildTemplate(): MenuTemplate {
    const isMac = process.platform === 'darwin';

    const template: MenuTemplate = [
      ...(isMac ? [this.buildMacAppMenu()] : []),
      this.buildFileMenu(),
      this.buildEditMenu(),
      this.buildViewMenu(),
      this.buildWindowMenu(),
      this.buildHelpMenu(),
    ];

    // Add developer menu in development
    if (!app.isPackaged) {
      template.push(this.buildDevMenu());
    }

    return template;
  }

  private buildMacAppMenu(): MenuItemConstructorOptions {
    return {
      label: app.name,
      submenu: [
        { role: 'about' },
        { type: 'separator' },
        {
          label: 'Preferences...',
          accelerator: 'CmdOrCtrl+,',
          click: () => this.openPreferences(),
        },
        { type: 'separator' },
        { role: 'services' },
        { type: 'separator' },
        { role: 'hide' },
        { role: 'hideOthers' },
        { role: 'unhide' },
        { type: 'separator' },
        { role: 'quit' },
      ],
    };
  }

  private buildFileMenu(): MenuItemConstructorOptions {
    const isMac = process.platform === 'darwin';

    return {
      label: 'File',
      submenu: [
        {
          label: 'New Project',
          accelerator: 'CmdOrCtrl+N',
          click: () => this.newProject(),
        },
        {
          label: 'Open Project...',
          accelerator: 'CmdOrCtrl+O',
          click: () => this.openProject(),
        },
        {
          label: 'Open Recent',
          submenu: this.buildRecentFilesMenu(),
        },
        { type: 'separator' },
        {
          label: 'Save',
          accelerator: 'CmdOrCtrl+S',
          click: () => this.save(),
        },
        {
          label: 'Save As...',
          accelerator: 'CmdOrCtrl+Shift+S',
          click: () => this.saveAs(),
        },
        { type: 'separator' },
        {
          label: 'Export...',
          accelerator: 'CmdOrCtrl+E',
          click: () => this.export(),
        },
        { type: 'separator' },
        ...(isMac
          ? []
          : [
              {
                label: 'Preferences...',
                accelerator: 'CmdOrCtrl+,',
                click: () => this.openPreferences(),
              },
              { type: 'separator' } as MenuItemConstructorOptions,
            ]),
        isMac ? { role: 'close' as const } : { role: 'quit' as const },
      ],
    };
  }

  private buildRecentFilesMenu(): MenuItemConstructorOptions[] {
    if (this.state.recentFiles.length === 0) {
      return [{ label: 'No Recent Files', enabled: false }];
    }

    const recentItems: MenuItemConstructorOptions[] = this.state.recentFiles.map(
      (filePath) => ({
        label: filePath,
        click: () => this.openRecentFile(filePath),
      })
    );

    return [
      ...recentItems,
      { type: 'separator' },
      {
        label: 'Clear Recent',
        click: () => this.clearRecentFiles(),
      },
    ];
  }

  private buildEditMenu(): MenuItemConstructorOptions {
    return {
      label: 'Edit',
      submenu: [
        {
          label: 'Undo',
          accelerator: 'CmdOrCtrl+Z',
          enabled: this.state.canUndo,
          click: () => this.sendToRenderer('edit:undo'),
        },
        {
          label: 'Redo',
          accelerator: 'CmdOrCtrl+Shift+Z',
          enabled: this.state.canRedo,
          click: () => this.sendToRenderer('edit:redo'),
        },
        { type: 'separator' },
        { role: 'cut' },
        { role: 'copy' },
        { role: 'paste' },
        { role: 'pasteAndMatchStyle' },
        { role: 'delete' },
        { role: 'selectAll' },
        { type: 'separator' },
        {
          label: 'Find',
          accelerator: 'CmdOrCtrl+F',
          click: () => this.sendToRenderer('edit:find'),
        },
        {
          label: 'Find and Replace',
          accelerator: 'CmdOrCtrl+H',
          click: () => this.sendToRenderer('edit:findReplace'),
        },
        { type: 'separator' },
        {
          label: 'Speech',
          submenu: [
            { role: 'startSpeaking' },
            { role: 'stopSpeaking' },
          ],
        },
      ],
    };
  }

  private buildViewMenu(): MenuItemConstructorOptions {
    return {
      label: 'View',
      submenu: [
        {
          label: 'Toggle Sidebar',
          accelerator: 'CmdOrCtrl+B',
          click: () => this.sendToRenderer('view:toggleSidebar'),
        },
        {
          label: 'Toggle Panel',
          accelerator: 'CmdOrCtrl+J',
          click: () => this.sendToRenderer('view:togglePanel'),
        },
        { type: 'separator' },
        {
          label: 'Zoom In',
          accelerator: 'CmdOrCtrl+Plus',
          click: () => this.zoomIn(),
        },
        {
          label: 'Zoom Out',
          accelerator: 'CmdOrCtrl+-',
          click: () => this.zoomOut(),
        },
        {
          label: 'Reset Zoom',
          accelerator: 'CmdOrCtrl+0',
          click: () => this.resetZoom(),
        },
        { type: 'separator' },
        {
          label: 'Enter Full Screen',
          accelerator: process.platform === 'darwin' ? 'Ctrl+Cmd+F' : 'F11',
          click: () => this.toggleFullScreen(),
        },
        { type: 'separator' },
        {
          label: 'Appearance',
          submenu: [
            {
              label: 'Light Mode',
              type: 'radio',
              checked: false,
              click: () => this.setTheme('light'),
            },
            {
              label: 'Dark Mode',
              type: 'radio',
              checked: true,
              click: () => this.setTheme('dark'),
            },
            {
              label: 'System',
              type: 'radio',
              checked: false,
              click: () => this.setTheme('system'),
            },
          ],
        },
      ],
    };
  }

  private buildWindowMenu(): MenuItemConstructorOptions {
    const isMac = process.platform === 'darwin';

    return {
      label: 'Window',
      submenu: [
        { role: 'minimize' },
        { role: 'zoom' },
        ...(isMac
          ? [
              { type: 'separator' } as MenuItemConstructorOptions,
              { role: 'front' } as MenuItemConstructorOptions,
              { type: 'separator' } as MenuItemConstructorOptions,
              { role: 'window' } as MenuItemConstructorOptions,
            ]
          : [{ role: 'close' } as MenuItemConstructorOptions]),
      ],
    };
  }

  private buildHelpMenu(): MenuItemConstructorOptions {
    return {
      label: 'Help',
      submenu: [
        {
          label: 'Documentation',
          click: () => shell.openExternal('https://docs.tachikoma.io'),
        },
        {
          label: 'Release Notes',
          click: () =>
            shell.openExternal(
              `https://github.com/tachikoma/releases/tag/v${app.getVersion()}`
            ),
        },
        { type: 'separator' },
        {
          label: 'Report Issue',
          click: () =>
            shell.openExternal('https://github.com/tachikoma/issues/new'),
        },
        {
          label: 'Community Forum',
          click: () => shell.openExternal('https://community.tachikoma.io'),
        },
        { type: 'separator' },
        {
          label: 'Check for Updates...',
          click: () => this.checkForUpdates(),
        },
        { type: 'separator' },
        ...(process.platform !== 'darwin'
          ? [
              {
                label: 'About Tachikoma',
                click: () => this.showAbout(),
              },
            ]
          : []),
      ],
    };
  }

  private buildDevMenu(): MenuItemConstructorOptions {
    return {
      label: 'Developer',
      submenu: [
        { role: 'reload' },
        { role: 'forceReload' },
        { role: 'toggleDevTools' },
        { type: 'separator' },
        {
          label: 'Open User Data Folder',
          click: () => shell.openPath(app.getPath('userData')),
        },
        {
          label: 'Open Logs Folder',
          click: () => shell.openPath(app.getPath('logs')),
        },
        { type: 'separator' },
        {
          label: 'Clear Storage',
          click: () => this.clearStorage(),
        },
        {
          label: 'Simulate Crash',
          click: () => process.crash(),
        },
      ],
    };
  }

  // Action handlers
  private sendToRenderer(channel: string, data?: unknown): void {
    this.mainWindow.webContents.send(channel, data);
  }

  private async newProject(): Promise<void> {
    this.sendToRenderer('project:new');
  }

  private async openProject(): Promise<void> {
    const { filePaths, canceled } = await dialog.showOpenDialog(this.mainWindow, {
      title: 'Open Project',
      properties: ['openDirectory'],
    });

    if (!canceled && filePaths.length > 0) {
      this.sendToRenderer('project:open', { path: filePaths[0] });
    }
  }

  private openRecentFile(filePath: string): void {
    this.sendToRenderer('project:open', { path: filePath });
  }

  private clearRecentFiles(): void {
    this.state.recentFiles = [];
    this.sendToRenderer('recentFiles:clear');
    this.buildMenu();
  }

  private save(): void {
    this.sendToRenderer('project:save');
  }

  private saveAs(): void {
    this.sendToRenderer('project:saveAs');
  }

  private export(): void {
    this.sendToRenderer('project:export');
  }

  private openPreferences(): void {
    this.sendToRenderer('preferences:open');
  }

  private zoomIn(): void {
    const currentZoom = this.mainWindow.webContents.getZoomFactor();
    this.mainWindow.webContents.setZoomFactor(Math.min(currentZoom + 0.1, 3));
  }

  private zoomOut(): void {
    const currentZoom = this.mainWindow.webContents.getZoomFactor();
    this.mainWindow.webContents.setZoomFactor(Math.max(currentZoom - 0.1, 0.5));
  }

  private resetZoom(): void {
    this.mainWindow.webContents.setZoomFactor(1);
  }

  private toggleFullScreen(): void {
    const isFullScreen = this.mainWindow.isFullScreen();
    this.mainWindow.setFullScreen(!isFullScreen);
  }

  private setTheme(theme: 'light' | 'dark' | 'system'): void {
    this.sendToRenderer('theme:set', { theme });
  }

  private checkForUpdates(): void {
    this.sendToRenderer('updates:check');
  }

  private showAbout(): void {
    dialog.showMessageBox(this.mainWindow, {
      type: 'info',
      title: 'About Tachikoma',
      message: 'Tachikoma',
      detail: `Version: ${app.getVersion()}\nElectron: ${process.versions.electron}\nChrome: ${process.versions.chrome}\nNode.js: ${process.versions.node}`,
    });
  }

  private async clearStorage(): Promise<void> {
    const { response } = await dialog.showMessageBox(this.mainWindow, {
      type: 'warning',
      title: 'Clear Storage',
      message: 'Are you sure you want to clear all storage?',
      detail: 'This will remove all saved data and preferences.',
      buttons: ['Cancel', 'Clear Storage'],
      defaultId: 0,
    });

    if (response === 1) {
      await this.mainWindow.webContents.session.clearStorageData();
      logger.info('Storage cleared');
    }
  }
}

export function setupMenu(mainWindow: BrowserWindow): MenuBuilder {
  const menuBuilder = new MenuBuilder(mainWindow);
  menuBuilder.buildMenu();
  return menuBuilder;
}
```

### Context Menu System

```typescript
// src/electron/main/menu/context-menu.ts
import {
  Menu,
  MenuItemConstructorOptions,
  BrowserWindow,
  clipboard,
  shell,
} from 'electron';
import { ipcMain } from 'electron';

interface ContextMenuOptions {
  type: 'text' | 'link' | 'image' | 'file' | 'custom';
  selectionText?: string;
  linkURL?: string;
  imageSrc?: string;
  filePath?: string;
  customItems?: MenuItemConstructorOptions[];
}

export function setupContextMenuHandlers(): void {
  ipcMain.handle(
    'contextMenu:show',
    (event, options: ContextMenuOptions) => {
      const window = BrowserWindow.fromWebContents(event.sender);
      if (!window) return;

      const menu = buildContextMenu(options, event.sender);
      menu.popup({ window });
    }
  );
}

function buildContextMenu(
  options: ContextMenuOptions,
  webContents: Electron.WebContents
): Menu {
  let template: MenuItemConstructorOptions[] = [];

  switch (options.type) {
    case 'text':
      template = buildTextContextMenu(options, webContents);
      break;
    case 'link':
      template = buildLinkContextMenu(options);
      break;
    case 'image':
      template = buildImageContextMenu(options);
      break;
    case 'file':
      template = buildFileContextMenu(options);
      break;
    case 'custom':
      template = options.customItems || [];
      break;
  }

  return Menu.buildFromTemplate(template);
}

function buildTextContextMenu(
  options: ContextMenuOptions,
  webContents: Electron.WebContents
): MenuItemConstructorOptions[] {
  const hasSelection = !!options.selectionText;

  return [
    {
      label: 'Cut',
      accelerator: 'CmdOrCtrl+X',
      enabled: hasSelection,
      click: () => webContents.cut(),
    },
    {
      label: 'Copy',
      accelerator: 'CmdOrCtrl+C',
      enabled: hasSelection,
      click: () => webContents.copy(),
    },
    {
      label: 'Paste',
      accelerator: 'CmdOrCtrl+V',
      click: () => webContents.paste(),
    },
    { type: 'separator' },
    {
      label: 'Select All',
      accelerator: 'CmdOrCtrl+A',
      click: () => webContents.selectAll(),
    },
    ...(hasSelection
      ? [
          { type: 'separator' } as MenuItemConstructorOptions,
          {
            label: 'Search Google',
            click: () => {
              const url = `https://www.google.com/search?q=${encodeURIComponent(
                options.selectionText!
              )}`;
              shell.openExternal(url);
            },
          },
        ]
      : []),
  ];
}

function buildLinkContextMenu(
  options: ContextMenuOptions
): MenuItemConstructorOptions[] {
  return [
    {
      label: 'Open Link',
      click: () => shell.openExternal(options.linkURL!),
    },
    {
      label: 'Copy Link Address',
      click: () => clipboard.writeText(options.linkURL!),
    },
    { type: 'separator' },
    {
      label: 'Copy',
      accelerator: 'CmdOrCtrl+C',
      click: () => clipboard.writeText(options.linkURL!),
    },
  ];
}

function buildImageContextMenu(
  options: ContextMenuOptions
): MenuItemConstructorOptions[] {
  return [
    {
      label: 'Copy Image',
      click: () => {
        // Would need to fetch and copy the image
      },
    },
    {
      label: 'Copy Image Address',
      click: () => clipboard.writeText(options.imageSrc!),
    },
    {
      label: 'Save Image As...',
      click: () => {
        // Would need to implement save dialog
      },
    },
    { type: 'separator' },
    {
      label: 'Open Image in Browser',
      click: () => shell.openExternal(options.imageSrc!),
    },
  ];
}

function buildFileContextMenu(
  options: ContextMenuOptions
): MenuItemConstructorOptions[] {
  return [
    {
      label: 'Open',
      click: () => shell.openPath(options.filePath!),
    },
    {
      label: 'Reveal in Finder',
      click: () => shell.showItemInFolder(options.filePath!),
    },
    { type: 'separator' },
    {
      label: 'Copy Path',
      click: () => clipboard.writeText(options.filePath!),
    },
    { type: 'separator' },
    {
      label: 'Delete',
      click: () => {
        // Would need to implement delete confirmation
      },
    },
  ];
}
```

### Tray Menu

```typescript
// src/electron/main/menu/tray-menu.ts
import { Menu, Tray, app, nativeImage, BrowserWindow } from 'electron';
import { join } from 'path';

export class TrayManager {
  private tray: Tray | null = null;
  private mainWindow: BrowserWindow;

  constructor(mainWindow: BrowserWindow) {
    this.mainWindow = mainWindow;
  }

  create(): void {
    const iconPath = this.getIconPath();
    const icon = nativeImage.createFromPath(iconPath);

    // Resize for system tray (16x16 on macOS, 16x16 on Windows)
    const trayIcon = icon.resize({ width: 16, height: 16 });

    this.tray = new Tray(trayIcon);
    this.tray.setToolTip('Tachikoma');

    this.updateMenu();

    this.tray.on('click', () => {
      this.toggleWindow();
    });

    this.tray.on('right-click', () => {
      this.tray?.popUpContextMenu();
    });
  }

  private getIconPath(): string {
    const iconName =
      process.platform === 'win32' ? 'tray-icon.ico' : 'tray-iconTemplate.png';
    return join(__dirname, '../../resources', iconName);
  }

  updateMenu(status?: 'online' | 'offline' | 'syncing'): void {
    const contextMenu = Menu.buildFromTemplate([
      {
        label: status ? `Status: ${status}` : 'Tachikoma',
        enabled: false,
      },
      { type: 'separator' },
      {
        label: 'Show Window',
        click: () => this.showWindow(),
      },
      {
        label: 'Hide Window',
        click: () => this.hideWindow(),
      },
      { type: 'separator' },
      {
        label: 'Quick Actions',
        submenu: [
          {
            label: 'New Project',
            click: () =>
              this.mainWindow.webContents.send('project:new'),
          },
          {
            label: 'Open Project...',
            click: () =>
              this.mainWindow.webContents.send('project:openDialog'),
          },
        ],
      },
      { type: 'separator' },
      {
        label: 'Preferences',
        click: () =>
          this.mainWindow.webContents.send('preferences:open'),
      },
      { type: 'separator' },
      {
        label: 'Quit',
        accelerator: 'CmdOrCtrl+Q',
        click: () => app.quit(),
      },
    ]);

    this.tray?.setContextMenu(contextMenu);
  }

  private toggleWindow(): void {
    if (this.mainWindow.isVisible()) {
      this.hideWindow();
    } else {
      this.showWindow();
    }
  }

  private showWindow(): void {
    this.mainWindow.show();
    this.mainWindow.focus();
  }

  private hideWindow(): void {
    this.mainWindow.hide();
  }

  destroy(): void {
    this.tray?.destroy();
    this.tray = null;
  }
}
```

### Menu IPC Handlers

```typescript
// src/electron/main/ipc/menu.ts
import { ipcMain, BrowserWindow } from 'electron';
import { MenuBuilder } from '../menu';

let menuBuilder: MenuBuilder | null = null;

export function setupMenuIpcHandlers(mainWindow: BrowserWindow): void {
  menuBuilder = new MenuBuilder(mainWindow);
  menuBuilder.buildMenu();

  ipcMain.handle('menu:updateState', (_, state) => {
    menuBuilder?.updateState(state);
  });

  ipcMain.handle('menu:addRecentFile', (_, filePath: string) => {
    if (menuBuilder) {
      const currentState = (menuBuilder as any).state;
      const recentFiles = [
        filePath,
        ...currentState.recentFiles.filter((f: string) => f !== filePath),
      ].slice(0, 10);

      menuBuilder.updateState({ recentFiles });
    }
  });
}
```

---

## Testing Requirements

### Unit Tests

```typescript
// src/electron/main/menu/__tests__/menu.test.ts
import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('electron', () => ({
  Menu: {
    buildFromTemplate: vi.fn().mockReturnValue({
      popup: vi.fn(),
    }),
    setApplicationMenu: vi.fn(),
  },
  app: {
    name: 'Tachikoma',
    getVersion: vi.fn().mockReturnValue('1.0.0'),
    isPackaged: false,
  },
  shell: {
    openExternal: vi.fn(),
    openPath: vi.fn(),
  },
  dialog: {
    showOpenDialog: vi.fn().mockResolvedValue({ canceled: true, filePaths: [] }),
    showMessageBox: vi.fn().mockResolvedValue({ response: 0 }),
  },
}));

describe('MenuBuilder', () => {
  let mockWindow: any;

  beforeEach(() => {
    mockWindow = {
      webContents: {
        send: vi.fn(),
        getZoomFactor: vi.fn().mockReturnValue(1),
        setZoomFactor: vi.fn(),
        session: {
          clearStorageData: vi.fn(),
        },
      },
      isFullScreen: vi.fn().mockReturnValue(false),
      setFullScreen: vi.fn(),
    };
  });

  it('should build menu with correct structure', async () => {
    const { MenuBuilder } = await import('../index');
    const builder = new MenuBuilder(mockWindow);

    const menu = builder.buildMenu();
    expect(menu).toBeDefined();
  });

  it('should update menu state', async () => {
    const { MenuBuilder } = await import('../index');
    const { Menu } = await import('electron');

    const builder = new MenuBuilder(mockWindow);
    builder.updateState({ canUndo: true });

    expect(Menu.buildFromTemplate).toHaveBeenCalled();
  });

  it('should handle recent files', async () => {
    const { MenuBuilder } = await import('../index');
    const builder = new MenuBuilder(mockWindow);

    builder.updateState({ recentFiles: ['/path/to/file1', '/path/to/file2'] });

    // Menu should be rebuilt with recent files
  });
});
```

### Integration Tests

```typescript
// src/electron/main/menu/__tests__/menu.integration.test.ts
import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { _electron as electron } from 'playwright';
import type { ElectronApplication } from 'playwright';

describe('Menu Integration', () => {
  let electronApp: ElectronApplication;

  beforeAll(async () => {
    electronApp = await electron.launch({ args: ['.'] });
  });

  afterAll(async () => {
    await electronApp.close();
  });

  it('should have application menu', async () => {
    const menuItemCount = await electronApp.evaluate(({ Menu }) => {
      const menu = Menu.getApplicationMenu();
      return menu?.items.length ?? 0;
    });

    expect(menuItemCount).toBeGreaterThan(0);
  });

  it('should handle menu keyboard shortcuts', async () => {
    const page = await electronApp.firstWindow();

    // Trigger save shortcut
    await page.keyboard.press('Control+S');

    // Verify IPC message was sent
    // This would need proper setup to verify
  });
});
```

---

## Related Specs

- Spec 161: Electron Main Process
- Spec 162: Window Management
- Spec 170: IPC Channels
- Spec 183: Tray Integration
