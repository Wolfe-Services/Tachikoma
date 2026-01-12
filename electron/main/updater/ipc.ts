import { ipcMain } from 'electron';
import { autoUpdaterService, UpdateChannel } from './index';

export function setupUpdaterIpcHandlers(): void {
  ipcMain.handle('updater:check', async (_, silent?: boolean) => {
    await autoUpdaterService.checkForUpdates(silent ?? false);
    return { success: true };
  });

  ipcMain.handle('updater:download', () => {
    autoUpdaterService.downloadUpdate();
    return { success: true };
  });

  ipcMain.handle('updater:install', () => {
    autoUpdaterService.installUpdate();
    return { success: true };
  });

  ipcMain.handle('updater:getState', () => {
    return autoUpdaterService.getState();
  });

  ipcMain.handle('updater:setChannel', (_, channel: UpdateChannel) => {
    autoUpdaterService.setChannel(channel);
    return { success: true };
  });

  ipcMain.handle('updater:clearSkipped', () => {
    autoUpdaterService.clearSkippedVersions();
    return { success: true };
  });

  ipcMain.handle('updater:getHistory', () => {
    return autoUpdaterService.getUpdateHistory();
  });

  ipcMain.handle('updater:startAutoCheck', () => {
    autoUpdaterService.startAutoCheck();
    return { success: true };
  });

  ipcMain.handle('updater:stopAutoCheck', () => {
    autoUpdaterService.stopAutoCheck();
    return { success: true };
  });
}