import {
  dialog,
  BrowserWindow,
  MessageBoxOptions,
  OpenDialogOptions,
  SaveDialogOptions,
  MessageBoxReturnValue,
  OpenDialogReturnValue,
  SaveDialogReturnValue,
} from 'electron';
import { Logger } from '../logger';

const logger = new Logger('dialogs');

export interface FileFilter {
  name: string;
  extensions: string[];
}

export interface OpenFileOptions {
  title?: string;
  defaultPath?: string;
  filters?: FileFilter[];
  multiSelect?: boolean;
  directory?: boolean;
  showHiddenFiles?: boolean;
}

export interface SaveFileOptions {
  title?: string;
  defaultPath?: string;
  filters?: FileFilter[];
  defaultName?: string;
}

export interface MessageOptions {
  type?: 'none' | 'info' | 'error' | 'question' | 'warning';
  title?: string;
  message: string;
  detail?: string;
  buttons?: string[];
  defaultId?: number;
  cancelId?: number;
  checkboxLabel?: string;
  checkboxChecked?: boolean;
}

export interface MessageResult {
  response: number;
  checkboxChecked: boolean;
}

class DialogService {
  private activeDialogs: Set<string> = new Set();

  private getWindow(): BrowserWindow | undefined {
    return BrowserWindow.getFocusedWindow() || BrowserWindow.getAllWindows()[0];
  }

  private generateDialogId(): string {
    return `dialog-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }

  async showOpenDialog(options: OpenFileOptions = {}): Promise<string[]> {
    const dialogId = this.generateDialogId();
    this.activeDialogs.add(dialogId);

    try {
      const window = this.getWindow();

      const dialogOptions: OpenDialogOptions = {
        title: options.title || 'Open',
        defaultPath: options.defaultPath,
        filters: options.filters,
        properties: this.buildOpenProperties(options),
      };

      logger.debug('Showing open dialog', { dialogId, options: dialogOptions });

      const result: OpenDialogReturnValue = window
        ? await dialog.showOpenDialog(window, dialogOptions)
        : await dialog.showOpenDialog(dialogOptions);

      if (result.canceled) {
        logger.debug('Open dialog canceled', { dialogId });
        return [];
      }

      logger.debug('Open dialog result', { dialogId, filePaths: result.filePaths });
      return result.filePaths;
    } finally {
      this.activeDialogs.delete(dialogId);
    }
  }

  private buildOpenProperties(
    options: OpenFileOptions
  ): OpenDialogOptions['properties'] {
    const properties: OpenDialogOptions['properties'] = [];

    if (options.directory) {
      properties.push('openDirectory');
    } else {
      properties.push('openFile');
    }

    if (options.multiSelect) {
      properties.push('multiSelections');
    }

    if (options.showHiddenFiles) {
      properties.push('showHiddenFiles');
    }

    // macOS specific
    if (process.platform === 'darwin') {
      properties.push('createDirectory');
    }

    return properties;
  }

  async showSaveDialog(options: SaveFileOptions = {}): Promise<string | null> {
    const dialogId = this.generateDialogId();
    this.activeDialogs.add(dialogId);

    try {
      const window = this.getWindow();

      const dialogOptions: SaveDialogOptions = {
        title: options.title || 'Save',
        defaultPath: options.defaultPath || options.defaultName,
        filters: options.filters,
        properties: ['createDirectory', 'showOverwriteConfirmation'],
      };

      logger.debug('Showing save dialog', { dialogId, options: dialogOptions });

      const result: SaveDialogReturnValue = window
        ? await dialog.showSaveDialog(window, dialogOptions)
        : await dialog.showSaveDialog(dialogOptions);

      if (result.canceled || !result.filePath) {
        logger.debug('Save dialog canceled', { dialogId });
        return null;
      }

      logger.debug('Save dialog result', { dialogId, filePath: result.filePath });
      return result.filePath;
    } finally {
      this.activeDialogs.delete(dialogId);
    }
  }

  async showMessageBox(options: MessageOptions): Promise<MessageResult> {
    const dialogId = this.generateDialogId();
    this.activeDialogs.add(dialogId);

    try {
      const window = this.getWindow();

      const dialogOptions: MessageBoxOptions = {
        type: options.type || 'info',
        title: options.title,
        message: options.message,
        detail: options.detail,
        buttons: options.buttons || ['OK'],
        defaultId: options.defaultId ?? 0,
        cancelId: options.cancelId,
        checkboxLabel: options.checkboxLabel,
        checkboxChecked: options.checkboxChecked,
        noLink: true,
      };

      logger.debug('Showing message box', { dialogId, options: dialogOptions });

      const result: MessageBoxReturnValue = window
        ? await dialog.showMessageBox(window, dialogOptions)
        : await dialog.showMessageBox(dialogOptions);

      logger.debug('Message box result', { dialogId, response: result.response });

      return {
        response: result.response,
        checkboxChecked: result.checkboxChecked,
      };
    } finally {
      this.activeDialogs.delete(dialogId);
    }
  }

  async confirm(message: string, detail?: string): Promise<boolean> {
    const result = await this.showMessageBox({
      type: 'question',
      message,
      detail,
      buttons: ['Cancel', 'OK'],
      defaultId: 1,
      cancelId: 0,
    });

    return result.response === 1;
  }

  async alert(message: string, detail?: string): Promise<void> {
    await this.showMessageBox({
      type: 'info',
      message,
      detail,
      buttons: ['OK'],
    });
  }

  async warn(message: string, detail?: string): Promise<void> {
    await this.showMessageBox({
      type: 'warning',
      message,
      detail,
      buttons: ['OK'],
    });
  }

  async error(message: string, detail?: string): Promise<void> {
    await this.showMessageBox({
      type: 'error',
      message,
      detail,
      buttons: ['OK'],
    });
  }

  showErrorBox(title: string, content: string): void {
    dialog.showErrorBox(title, content);
  }

  hasActiveDialogs(): boolean {
    return this.activeDialogs.size > 0;
  }
}

export const dialogService = new DialogService();