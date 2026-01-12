import { ipcMain } from 'electron';
import { securityManager } from './index';
import { runSecurityAudit } from './audit';

export function setupSecurityIpcHandlers(): void {
  ipcMain.handle('security:getReport', () => {
    return securityManager.getSecurityReport();
  });

  ipcMain.handle('security:runAudit', async () => {
    return runSecurityAudit();
  });

  ipcMain.handle('security:addAllowedOrigin', (_, origin: string) => {
    securityManager.addAllowedOrigin(origin);
  });

  ipcMain.handle('security:removeAllowedOrigin', (_, origin: string) => {
    securityManager.removeAllowedOrigin(origin);
  });
}