import { Menu, Tray, app, nativeImage, BrowserWindow } from 'electron';
import { join } from 'path';
import { Logger } from '../logger';

const logger = new Logger('tray');

export class TrayManager {
  private tray: Tray | null = null;
  private mainWindow: BrowserWindow;

  constructor(mainWindow: BrowserWindow) {
    this.mainWindow = mainWindow;
  }

  create(): void {
    try {
      const iconPath = this.getIconPath();
      const icon = nativeImage.createFromPath(iconPath);

      // Resize for system tray (16x16 on most platforms)
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

      logger.info('System tray created');
    } catch (error) {
      logger.error('Failed to create system tray', error);
    }
  }

  private getIconPath(): string {
    const iconName =
      process.platform === 'win32' ? 'tray-icon.ico' : 'tray-iconTemplate.png';
    return join(__dirname, '../../resources', iconName);
  }

  updateMenu(status?: 'online' | 'offline' | 'syncing'): void {
    if (!this.tray) return;

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

    this.tray.setContextMenu(contextMenu);
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
    logger.debug('Window shown from tray');
  }

  private hideWindow(): void {
    this.mainWindow.hide();
    logger.debug('Window hidden to tray');
  }

  destroy(): void {
    if (this.tray) {
      this.tray.destroy();
      this.tray = null;
      logger.info('System tray destroyed');
    }
  }
}