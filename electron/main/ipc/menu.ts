import { ipcMain, BrowserWindow } from 'electron';
import { MenuBuilder } from '../menu';
import { Logger } from '../logger';

const logger = new Logger('menu-ipc');

let menuBuilder: MenuBuilder | null = null;

export function setupMenuIpcHandlers(mainWindow: BrowserWindow): MenuBuilder {
  menuBuilder = new MenuBuilder(mainWindow);
  menuBuilder.buildMenu();

  ipcMain.handle('menu:updateState', (_, state) => {
    if (menuBuilder) {
      menuBuilder.updateState(state);
      logger.debug('Menu state updated', state);
    }
  });

  ipcMain.handle('menu:addRecentFile', (_, filePath: string) => {
    if (menuBuilder) {
      const currentState = (menuBuilder as any).state;
      const recentFiles = [
        filePath,
        ...currentState.recentFiles.filter((f: string) => f !== filePath),
      ].slice(0, 10);

      menuBuilder.updateState({ recentFiles });
      logger.info('Recent file added', { filePath });
    }
  });

  ipcMain.handle('menu:clearRecentFiles', () => {
    if (menuBuilder) {
      menuBuilder.updateState({ recentFiles: [] });
      logger.info('Recent files cleared');
    }
  });

  ipcMain.handle('menu:setFullScreenState', (_, isFullScreen: boolean) => {
    if (menuBuilder) {
      menuBuilder.updateState({ isFullScreen });
      logger.debug('Fullscreen state updated', { isFullScreen });
    }
  });

  logger.info('Menu IPC handlers registered');
  return menuBuilder;
}