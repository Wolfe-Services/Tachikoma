import { ipcMain, app } from 'electron';
import { lifecycleManager } from '../lifecycle';

export function setupLifecycleIpcHandlers(): void {
  ipcMain.handle('lifecycle:getState', () => {
    return lifecycleManager.getState();
  });

  ipcMain.handle('lifecycle:restart', async () => {
    await lifecycleManager.restart();
  });

  ipcMain.handle('lifecycle:quit', () => {
    app.quit();
  });

  ipcMain.handle('lifecycle:preventQuit', (_, prevent: boolean) => {
    lifecycleManager.preventAppQuit(prevent);
  });

  ipcMain.handle('lifecycle:setLoginItem', (_, openAtLogin: boolean) => {
    lifecycleManager.setLoginItemSettings(openAtLogin);
  });

  ipcMain.handle('lifecycle:getLoginItem', () => {
    return lifecycleManager.getLoginItemSettings();
  });

  ipcMain.handle('lifecycle:startPowerSaveBlocker', (_, reason) => {
    lifecycleManager.startPowerSaveBlocker(reason);
  });

  ipcMain.handle('lifecycle:stopPowerSaveBlocker', () => {
    lifecycleManager.stopPowerSaveBlocker();
  });

  // macOS specific
  ipcMain.handle('lifecycle:setBadgeCount', (_, count: number) => {
    lifecycleManager.setBadgeCount(count);
  });

  ipcMain.handle('lifecycle:bounce', (_, type?: 'critical' | 'informational') => {
    lifecycleManager.bounce(type);
  });

  // Unsaved changes response from renderer
  ipcMain.on('lifecycle:unsavedChangesResponse', () => {
    // Handled in lifecycle manager
  });

  ipcMain.on('lifecycle:saveComplete', () => {
    // Handled in lifecycle manager
  });
}