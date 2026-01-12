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
          label: this.state.isFullScreen ? 'Exit Full Screen' : 'Enter Full Screen',
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
          accelerator: 'F1',
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
        { 
          role: 'reload',
          accelerator: 'CmdOrCtrl+R',
        },
        { 
          role: 'forceReload',
          accelerator: 'CmdOrCtrl+Shift+R',
        },
        { 
          role: 'toggleDevTools',
          accelerator: 'F12',
        },
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
    this.state.isFullScreen = !isFullScreen;
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