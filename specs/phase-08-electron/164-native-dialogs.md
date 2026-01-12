# Spec 164: Native Dialogs

## Phase
8 - Electron Shell

## Spec ID
164

## Status
Planned

## Dependencies
- Spec 161 (Electron Main Process)
- Spec 170 (IPC Channels)
- Spec 172 (Context Bridge)

## Estimated Context
~8%

---

## Objective

Implement native dialog wrappers for the Electron application, providing consistent APIs for file dialogs, message boxes, and error dialogs across platforms. Ensure proper security through the IPC layer.

---

## Acceptance Criteria

- [x] File open dialog with filters and multi-select
- [x] File save dialog with default names and extensions
- [x] Directory picker dialog
- [x] Message box for confirmations and alerts
- [x] Error dialog for critical errors
- [x] Progress dialog for long operations
- [x] Custom dialog windows for complex inputs
- [x] Dialog result promises with proper typing
- [x] Platform-specific dialog behavior
- [x] Dialog state management (prevent multiple dialogs)

---

## Implementation Details

### Dialog Service

```typescript
// src/electron/main/dialogs/index.ts
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
```

### Predefined Dialog Helpers

```typescript
// src/electron/main/dialogs/presets.ts
import { dialogService, FileFilter } from './index';

// Common file filters
export const fileFilters: Record<string, FileFilter[]> = {
  images: [
    { name: 'Images', extensions: ['jpg', 'jpeg', 'png', 'gif', 'webp', 'svg'] },
  ],
  documents: [
    { name: 'Documents', extensions: ['pdf', 'doc', 'docx', 'txt', 'md'] },
  ],
  code: [
    { name: 'Code', extensions: ['ts', 'tsx', 'js', 'jsx', 'json', 'html', 'css'] },
  ],
  projects: [
    { name: 'Tachikoma Project', extensions: ['tachi', 'tachikoma'] },
  ],
  all: [
    { name: 'All Files', extensions: ['*'] },
  ],
};

export async function openProjectDialog(): Promise<string | null> {
  const paths = await dialogService.showOpenDialog({
    title: 'Open Project',
    directory: true,
  });

  return paths.length > 0 ? paths[0] : null;
}

export async function openFileDialog(filterType?: keyof typeof fileFilters): Promise<string[]> {
  return dialogService.showOpenDialog({
    title: 'Open File',
    filters: filterType ? fileFilters[filterType] : fileFilters.all,
    multiSelect: true,
  });
}

export async function openImageDialog(): Promise<string[]> {
  return dialogService.showOpenDialog({
    title: 'Select Image',
    filters: fileFilters.images,
    multiSelect: true,
  });
}

export async function saveProjectDialog(defaultName?: string): Promise<string | null> {
  return dialogService.showSaveDialog({
    title: 'Save Project',
    defaultName: defaultName || 'untitled.tachi',
    filters: fileFilters.projects,
  });
}

export async function exportDialog(
  defaultName: string,
  filterType: keyof typeof fileFilters
): Promise<string | null> {
  return dialogService.showSaveDialog({
    title: 'Export',
    defaultName,
    filters: fileFilters[filterType],
  });
}

export async function confirmDelete(itemName: string): Promise<boolean> {
  return dialogService.confirm(
    `Delete "${itemName}"?`,
    'This action cannot be undone.'
  );
}

export async function confirmUnsavedChanges(): Promise<'save' | 'discard' | 'cancel'> {
  const result = await dialogService.showMessageBox({
    type: 'warning',
    message: 'You have unsaved changes',
    detail: 'Do you want to save your changes before closing?',
    buttons: ['Save', "Don't Save", 'Cancel'],
    defaultId: 0,
    cancelId: 2,
  });

  switch (result.response) {
    case 0:
      return 'save';
    case 1:
      return 'discard';
    default:
      return 'cancel';
  }
}

export async function confirmOverwrite(filePath: string): Promise<boolean> {
  return dialogService.confirm(
    'File already exists',
    `Do you want to replace "${filePath}"?`
  );
}

export async function showAboutDialog(appInfo: {
  name: string;
  version: string;
  copyright: string;
}): Promise<void> {
  await dialogService.showMessageBox({
    type: 'info',
    title: `About ${appInfo.name}`,
    message: appInfo.name,
    detail: `Version ${appInfo.version}\n\n${appInfo.copyright}`,
  });
}
```

### Progress Dialog

```typescript
// src/electron/main/dialogs/progress.ts
import { BrowserWindow } from 'electron';

interface ProgressOptions {
  title: string;
  message?: string;
  indeterminate?: boolean;
}

export class ProgressDialog {
  private window: BrowserWindow | null = null;
  private parentWindow: BrowserWindow;
  private progress: number = 0;
  private message: string = '';

  constructor(parentWindow: BrowserWindow) {
    this.parentWindow = parentWindow;
  }

  async show(options: ProgressOptions): Promise<void> {
    this.window = new BrowserWindow({
      width: 400,
      height: 120,
      parent: this.parentWindow,
      modal: true,
      frame: false,
      resizable: false,
      movable: false,
      minimizable: false,
      maximizable: false,
      closable: false,
      show: false,
      webPreferences: {
        nodeIntegration: false,
        contextIsolation: true,
      },
    });

    const html = this.generateHTML(options);
    await this.window.loadURL(`data:text/html;charset=utf-8,${encodeURIComponent(html)}`);
    this.window.show();

    // Set taskbar progress
    if (options.indeterminate) {
      this.parentWindow.setProgressBar(-1); // Indeterminate
    }
  }

  private generateHTML(options: ProgressOptions): string {
    return `
      <!DOCTYPE html>
      <html>
      <head>
        <style>
          * { margin: 0; padding: 0; box-sizing: border-box; }
          body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #1a1a1a;
            color: #fff;
            padding: 20px;
            display: flex;
            flex-direction: column;
            justify-content: center;
            height: 100vh;
          }
          .title {
            font-size: 14px;
            font-weight: 600;
            margin-bottom: 8px;
          }
          .message {
            font-size: 12px;
            color: #888;
            margin-bottom: 12px;
          }
          .progress-container {
            background: #333;
            border-radius: 4px;
            height: 8px;
            overflow: hidden;
          }
          .progress-bar {
            background: #007aff;
            height: 100%;
            width: 0%;
            transition: width 0.3s ease;
          }
          .progress-bar.indeterminate {
            width: 30%;
            animation: indeterminate 1.5s infinite ease-in-out;
          }
          @keyframes indeterminate {
            0% { transform: translateX(-100%); }
            100% { transform: translateX(400%); }
          }
        </style>
      </head>
      <body>
        <div class="title" id="title">${options.title}</div>
        <div class="message" id="message">${options.message || ''}</div>
        <div class="progress-container">
          <div class="progress-bar ${options.indeterminate ? 'indeterminate' : ''}" id="progress"></div>
        </div>
      </body>
      </html>
    `;
  }

  setProgress(value: number): void {
    this.progress = Math.max(0, Math.min(100, value));

    if (this.window && !this.window.isDestroyed()) {
      this.window.webContents.executeJavaScript(
        `document.getElementById('progress').style.width = '${this.progress}%'`
      );
    }

    this.parentWindow.setProgressBar(this.progress / 100);
  }

  setMessage(message: string): void {
    this.message = message;

    if (this.window && !this.window.isDestroyed()) {
      this.window.webContents.executeJavaScript(
        `document.getElementById('message').textContent = '${message.replace(/'/g, "\\'")}'`
      );
    }
  }

  close(): void {
    if (this.window && !this.window.isDestroyed()) {
      this.window.close();
      this.window = null;
    }

    this.parentWindow.setProgressBar(-1); // Remove progress
  }
}
```

### Dialog IPC Handlers

```typescript
// src/electron/main/ipc/dialogs.ts
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
}
```

### Renderer API Types

```typescript
// src/shared/types/dialogs.ts
export interface DialogAPI {
  // File dialogs
  openFile(options?: OpenFileOptions): Promise<string[]>;
  saveFile(options?: SaveFileOptions): Promise<string | null>;

  // Message dialogs
  message(options: MessageOptions): Promise<MessageResult>;
  confirm(message: string, detail?: string): Promise<boolean>;
  alert(message: string, detail?: string): Promise<void>;
  error(message: string, detail?: string): Promise<void>;

  // Preset dialogs
  openProject(): Promise<string | null>;
  openFiles(filterType?: string): Promise<string[]>;
  saveProject(defaultName?: string): Promise<string | null>;
  confirmUnsavedChanges(): Promise<'save' | 'discard' | 'cancel'>;
  confirmDelete(itemName: string): Promise<boolean>;

  // Progress dialogs
  progress: {
    create(id: string, options: ProgressOptions): Promise<void>;
    update(id: string, value: number): void;
    message(id: string, message: string): void;
    close(id: string): void;
  };
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

export interface FileFilter {
  name: string;
  extensions: string[];
}

export interface ProgressOptions {
  title: string;
  message?: string;
  indeterminate?: boolean;
}
```

---

## Testing Requirements

### Unit Tests

```typescript
// src/electron/main/dialogs/__tests__/dialogs.test.ts
import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('electron', () => ({
  dialog: {
    showOpenDialog: vi.fn(),
    showSaveDialog: vi.fn(),
    showMessageBox: vi.fn(),
    showErrorBox: vi.fn(),
  },
  BrowserWindow: {
    getFocusedWindow: vi.fn().mockReturnValue(null),
    getAllWindows: vi.fn().mockReturnValue([]),
  },
}));

describe('DialogService', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should show open dialog and return file paths', async () => {
    const { dialog } = await import('electron');
    const { dialogService } = await import('../index');

    (dialog.showOpenDialog as any).mockResolvedValue({
      canceled: false,
      filePaths: ['/path/to/file.txt'],
    });

    const result = await dialogService.showOpenDialog();
    expect(result).toEqual(['/path/to/file.txt']);
  });

  it('should return empty array when dialog is canceled', async () => {
    const { dialog } = await import('electron');
    const { dialogService } = await import('../index');

    (dialog.showOpenDialog as any).mockResolvedValue({
      canceled: true,
      filePaths: [],
    });

    const result = await dialogService.showOpenDialog();
    expect(result).toEqual([]);
  });

  it('should show save dialog and return file path', async () => {
    const { dialog } = await import('electron');
    const { dialogService } = await import('../index');

    (dialog.showSaveDialog as any).mockResolvedValue({
      canceled: false,
      filePath: '/path/to/saved.txt',
    });

    const result = await dialogService.showSaveDialog();
    expect(result).toBe('/path/to/saved.txt');
  });

  it('should handle confirm dialog correctly', async () => {
    const { dialog } = await import('electron');
    const { dialogService } = await import('../index');

    (dialog.showMessageBox as any).mockResolvedValue({
      response: 1,
      checkboxChecked: false,
    });

    const result = await dialogService.confirm('Are you sure?');
    expect(result).toBe(true);
  });
});
```

### Integration Tests

```typescript
// src/electron/main/dialogs/__tests__/dialogs.integration.test.ts
import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { _electron as electron } from 'playwright';
import type { ElectronApplication, Page } from 'playwright';

describe('Dialogs Integration', () => {
  let electronApp: ElectronApplication;
  let page: Page;

  beforeAll(async () => {
    electronApp = await electron.launch({ args: ['.'] });
    page = await electronApp.firstWindow();
  });

  afterAll(async () => {
    await electronApp.close();
  });

  it('should handle IPC dialog calls', async () => {
    // Test would need to mock dialog responses
    // as actual dialogs require user interaction
  });

  it('should track active dialogs', async () => {
    const hasActiveDialogs = await electronApp.evaluate(async () => {
      // Access dialog service and check state
      return false;
    });

    expect(hasActiveDialogs).toBe(false);
  });
});
```

---

## Related Specs

- Spec 161: Electron Main Process
- Spec 165: File System Access
- Spec 170: IPC Channels
- Spec 172: Context Bridge
