import { ipcMain, BrowserWindow } from 'electron';
import {
  dialogService,
  OpenFileOptions,
  SaveFileOptions,
  MessageOptions,
} from '../dialogs';
import {
  openProjectDialog,
  openFileDialog,
  saveProjectDialog,
  confirmUnsavedChanges,
  confirmDelete,
  showAboutDialog,
} from '../dialogs/presets';
import { ProgressDialog } from '../dialogs/progress';

const progressDialogs = new Map<string, ProgressDialog>();

export function setupDialogIpcHandlers(): void {
  // Generic dialogs
  ipcMain.handle(
    'dialog:openFile',
    async (_, options: OpenFileOptions) => {
      return dialogService.showOpenDialog(options);
    }
  );

  ipcMain.handle(
    'dialog:saveFile',
    async (_, options: SaveFileOptions) => {
      return dialogService.showSaveDialog(options);
    }
  );

  ipcMain.handle(
    'dialog:message',
    async (_, options: MessageOptions) => {
      return dialogService.showMessageBox(options);
    }
  );

  ipcMain.handle('dialog:confirm', async (_, message: string, detail?: string) => {
    return dialogService.confirm(message, detail);
  });

  ipcMain.handle('dialog:alert', async (_, message: string, detail?: string) => {
    return dialogService.alert(message, detail);
  });

  ipcMain.handle('dialog:error', async (_, message: string, detail?: string) => {
    return dialogService.error(message, detail);
  });

  // Preset dialogs
  ipcMain.handle('dialog:openProject', async () => {
    return openProjectDialog();
  });

  ipcMain.handle('dialog:openFiles', async (_, filterType?: string) => {
    return openFileDialog(filterType as any);
  });

  ipcMain.handle('dialog:saveProject', async (_, defaultName?: string) => {
    return saveProjectDialog(defaultName);
  });

  ipcMain.handle('dialog:confirmUnsavedChanges', async () => {
    return confirmUnsavedChanges();
  });

  ipcMain.handle('dialog:confirmDelete', async (_, itemName: string) => {
    return confirmDelete(itemName);
  });

  ipcMain.handle('dialog:showAbout', async (_, appInfo: { name: string; version: string; copyright: string }) => {
    return showAboutDialog(appInfo);
  });

  // Progress dialogs
  ipcMain.handle(
    'dialog:progress:create',
    async (event, id: string, options: { title: string; message?: string; indeterminate?: boolean }) => {
      const window = BrowserWindow.fromWebContents(event.sender);
      if (!window) return;

      const progress = new ProgressDialog(window);
      await progress.show(options);
      progressDialogs.set(id, progress);
    }
  );

  ipcMain.handle('dialog:progress:update', (_, id: string, value: number) => {
    const progress = progressDialogs.get(id);
    progress?.setProgress(value);
  });

  ipcMain.handle('dialog:progress:message', (_, id: string, message: string) => {
    const progress = progressDialogs.get(id);
    progress?.setMessage(message);
  });

  ipcMain.handle('dialog:progress:close', (_, id: string) => {
    const progress = progressDialogs.get(id);
    progress?.close();
    progressDialogs.delete(id);
  });

  // Dialog state management
  ipcMain.handle('dialog:hasActive', async () => {
    return dialogService.hasActiveDialogs();
  });
}