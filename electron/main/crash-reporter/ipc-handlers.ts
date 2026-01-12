import { ipcMain } from 'electron';
import { crashReporterService } from './index';

export function setupCrashReporterIpcHandlers(): void {
  ipcMain.handle('crash-reporter:getReports', async () => {
    return crashReporterService.getCrashReports();
  });

  ipcMain.handle('crash-reporter:clearReports', async () => {
    await crashReporterService.clearCrashReports();
  });

  ipcMain.handle('crash-reporter:deleteReport', async (_, id: string) => {
    await crashReporterService.deleteCrashReport(id);
  });

  ipcMain.handle('crash-reporter:setEnabled', (_, enabled: boolean) => {
    crashReporterService.setEnabled(enabled);
  });

  // Receive errors from renderer
  ipcMain.on('crash-reporter:exception', () => {
    // Handled in setupRenderer
  });

  ipcMain.on('crash-reporter:rejection', () => {
    // Handled in setupRenderer
  });
}