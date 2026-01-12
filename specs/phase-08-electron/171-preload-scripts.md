# Spec 171: Preload Scripts

## Phase
8 - Electron Shell

## Spec ID
171

## Status
Planned

## Dependencies
- Spec 161 (Electron Main Process)
- Spec 169 (Security Configuration)
- Spec 170 (IPC Channels)

## Estimated Context
~9%

---

## Objective

Implement secure preload scripts that bridge the main process and renderer while maintaining context isolation. The preload script exposes a minimal, type-safe API to the renderer through the context bridge.

---

## Acceptance Criteria

- [ ] Preload script runs in isolated context
- [ ] Minimal API surface exposed to renderer
- [ ] Type-safe bridge API
- [ ] IPC wrapper functions for all channels
- [ ] Event listener registration and cleanup
- [ ] Platform information exposure
- [ ] No direct Node.js or Electron access in renderer
- [ ] Proper error handling in bridge calls
- [ ] Support for multiple preload scripts (per window type)

---

## Implementation Details

### Main Preload Script

```typescript
// src/electron/preload/index.ts
import { contextBridge, ipcRenderer, IpcRendererEvent } from 'electron';
import { IPC_CHANNELS } from '../../shared/ipc/channels';

// Type definitions for the exposed API
interface ElectronAPI {
  // Platform info
  platform: NodeJS.Platform;
  isPackaged: boolean;

  // App operations
  getAppInfo: () => Promise<AppInfo>;
  getAppPath: (name: string) => Promise<string>;
  quit: () => Promise<void>;
  restart: () => Promise<void>;

  // Window operations
  minimizeWindow: () => Promise<void>;
  maximizeWindow: () => Promise<void>;
  closeWindow: () => Promise<void>;
  isWindowMaximized: () => Promise<boolean>;
  setWindowTitle: (title: string) => Promise<void>;
  onWindowMaximize: (callback: (maximized: boolean) => void) => () => void;

  // File system operations
  fs: {
    exists: (path: string) => Promise<boolean>;
    stat: (path: string) => Promise<FileInfo>;
    readFile: (path: string, options?: ReadOptions) => Promise<string | Uint8Array>;
    writeFile: (path: string, data: string | Uint8Array, options?: WriteOptions) => Promise<void>;
    deleteFile: (path: string) => Promise<void>;
    readDirectory: (path: string) => Promise<FileInfo[]>;
    createDirectory: (path: string, recursive?: boolean) => Promise<void>;
    watch: (path: string, callback: (event: WatchEvent) => void) => Promise<() => void>;
    getAppPath: (name: string) => Promise<string>;
  };

  // Dialog operations
  dialog: {
    openFile: (options?: OpenFileOptions) => Promise<string[]>;
    saveFile: (options?: SaveFileOptions) => Promise<string | null>;
    showMessage: (options: MessageOptions) => Promise<MessageResult>;
    confirm: (message: string, detail?: string) => Promise<boolean>;
    alert: (message: string, detail?: string) => Promise<void>;
    error: (message: string, detail?: string) => Promise<void>;
  };

  // Menu operations
  menu: {
    updateState: (state: MenuState) => Promise<void>;
    showContextMenu: (options: ContextMenuOptions) => Promise<void>;
  };

  // Update operations
  updater: {
    check: (silent?: boolean) => Promise<void>;
    download: () => Promise<void>;
    install: () => Promise<void>;
    getState: () => Promise<UpdateState>;
    onChecking: (callback: () => void) => () => void;
    onAvailable: (callback: (info: UpdateInfo) => void) => () => void;
    onNotAvailable: (callback: (info: UpdateInfo) => void) => () => void;
    onProgress: (callback: (progress: ProgressInfo) => void) => () => void;
    onDownloaded: (callback: (info: UpdateInfo) => void) => () => void;
    onError: (callback: (error: { message: string }) => void) => () => void;
  };

  // Notification operations
  notification: {
    show: (options: NotificationOptions) => Promise<string>;
    onClick: (callback: (id: string) => void) => () => void;
    onClose: (callback: (id: string) => void) => () => void;
  };

  // Crash reporting
  reportException: (error: ErrorInfo) => void;
  reportRejection: (error: ErrorInfo) => void;

  // System events
  onSuspend: (callback: () => void) => () => void;
  onResume: (callback: () => void) => () => void;
  onThemeChange: (callback: (isDark: boolean) => void) => () => void;
  onDeepLink: (callback: (url: string) => void) => () => void;
  onConnectivity: (callback: (isOnline: boolean) => void) => () => void;

  // Shell operations
  shell: {
    openExternal: (url: string) => Promise<void>;
    openPath: (path: string) => Promise<void>;
    showItemInFolder: (path: string) => void;
  };

  // Clipboard operations
  clipboard: {
    readText: () => string;
    writeText: (text: string) => void;
    readImage: () => Uint8Array | null;
    writeImage: (data: Uint8Array) => void;
  };
}

// Helper function to create event listeners with cleanup
function createEventListener<T>(
  channel: string,
  callback: (data: T) => void
): () => void {
  const handler = (_event: IpcRendererEvent, data: T) => callback(data);
  ipcRenderer.on(channel, handler);
  return () => ipcRenderer.removeListener(channel, handler);
}

// The API to expose
const electronAPI: ElectronAPI = {
  // Platform info
  platform: process.platform,
  isPackaged: process.env.NODE_ENV === 'production',

  // App operations
  getAppInfo: () => ipcRenderer.invoke(IPC_CHANNELS.APP.GET_INFO),
  getAppPath: (name) => ipcRenderer.invoke(IPC_CHANNELS.APP.GET_PATH, { name }),
  quit: () => ipcRenderer.invoke(IPC_CHANNELS.APP.QUIT),
  restart: () => ipcRenderer.invoke(IPC_CHANNELS.APP.RESTART),

  // Window operations
  minimizeWindow: () => ipcRenderer.invoke(IPC_CHANNELS.WINDOW.MINIMIZE),
  maximizeWindow: () => ipcRenderer.invoke(IPC_CHANNELS.WINDOW.MAXIMIZE),
  closeWindow: () => ipcRenderer.invoke(IPC_CHANNELS.WINDOW.CLOSE),
  isWindowMaximized: () => ipcRenderer.invoke(IPC_CHANNELS.WINDOW.IS_MAXIMIZED),
  setWindowTitle: (title) =>
    ipcRenderer.invoke(IPC_CHANNELS.WINDOW.SET_TITLE, { title }),
  onWindowMaximize: (callback) =>
    createEventListener(IPC_CHANNELS.WINDOW.MAXIMIZE_CHANGED, (data: { maximized: boolean }) =>
      callback(data.maximized)
    ),

  // File system operations
  fs: {
    exists: (path) => ipcRenderer.invoke(IPC_CHANNELS.FS.EXISTS, path),
    stat: (path) => ipcRenderer.invoke(IPC_CHANNELS.FS.STAT, path),
    readFile: async (path, options) => {
      const result = await ipcRenderer.invoke(IPC_CHANNELS.FS.READ_FILE, path, options);
      if (result.type === 'buffer') {
        return Uint8Array.from(atob(result.data), (c) => c.charCodeAt(0));
      }
      return result.data;
    },
    writeFile: async (path, data, options) => {
      let payload: string | { type: 'buffer'; data: string };
      if (data instanceof Uint8Array) {
        payload = {
          type: 'buffer',
          data: btoa(String.fromCharCode(...data)),
        };
      } else {
        payload = data;
      }
      await ipcRenderer.invoke(IPC_CHANNELS.FS.WRITE_FILE, path, payload, options);
    },
    deleteFile: (path) => ipcRenderer.invoke(IPC_CHANNELS.FS.DELETE_FILE, path),
    readDirectory: (path) => ipcRenderer.invoke(IPC_CHANNELS.FS.READ_DIR, path),
    createDirectory: (path, recursive) =>
      ipcRenderer.invoke(IPC_CHANNELS.FS.CREATE_DIR, path, recursive),
    watch: async (path, callback) => {
      const watchId = await ipcRenderer.invoke(IPC_CHANNELS.FS.WATCH, path);
      const handler = (_event: IpcRendererEvent, data: any) => {
        if (data.watchId === watchId) {
          callback(data);
        }
      };
      ipcRenderer.on(IPC_CHANNELS.FS.WATCH_EVENT, handler);

      return () => {
        ipcRenderer.removeListener(IPC_CHANNELS.FS.WATCH_EVENT, handler);
        ipcRenderer.invoke(IPC_CHANNELS.FS.UNWATCH, watchId);
      };
    },
    getAppPath: (name) => ipcRenderer.invoke('fs:getAppPath', name),
  },

  // Dialog operations
  dialog: {
    openFile: (options) =>
      ipcRenderer.invoke(IPC_CHANNELS.DIALOG.OPEN_FILE, options),
    saveFile: (options) =>
      ipcRenderer.invoke(IPC_CHANNELS.DIALOG.SAVE_FILE, options),
    showMessage: (options) =>
      ipcRenderer.invoke(IPC_CHANNELS.DIALOG.MESSAGE, options),
    confirm: (message, detail) =>
      ipcRenderer.invoke(IPC_CHANNELS.DIALOG.CONFIRM, message, detail),
    alert: (message, detail) =>
      ipcRenderer.invoke('dialog:alert', message, detail),
    error: (message, detail) =>
      ipcRenderer.invoke(IPC_CHANNELS.DIALOG.ERROR, message, detail),
  },

  // Menu operations
  menu: {
    updateState: (state) =>
      ipcRenderer.invoke(IPC_CHANNELS.MENU.UPDATE_STATE, state),
    showContextMenu: (options) =>
      ipcRenderer.invoke(IPC_CHANNELS.MENU.CONTEXT_SHOW, options),
  },

  // Update operations
  updater: {
    check: (silent) => ipcRenderer.invoke(IPC_CHANNELS.UPDATE.CHECK, silent),
    download: () => ipcRenderer.invoke(IPC_CHANNELS.UPDATE.DOWNLOAD),
    install: () => ipcRenderer.invoke(IPC_CHANNELS.UPDATE.INSTALL),
    getState: () => ipcRenderer.invoke(IPC_CHANNELS.UPDATE.GET_STATE),
    onChecking: (callback) =>
      createEventListener(IPC_CHANNELS.UPDATE.CHECKING, callback),
    onAvailable: (callback) =>
      createEventListener(IPC_CHANNELS.UPDATE.AVAILABLE, callback),
    onNotAvailable: (callback) =>
      createEventListener(IPC_CHANNELS.UPDATE.NOT_AVAILABLE, callback),
    onProgress: (callback) =>
      createEventListener(IPC_CHANNELS.UPDATE.PROGRESS, callback),
    onDownloaded: (callback) =>
      createEventListener(IPC_CHANNELS.UPDATE.DOWNLOADED, callback),
    onError: (callback) =>
      createEventListener(IPC_CHANNELS.UPDATE.ERROR, callback),
  },

  // Notification operations
  notification: {
    show: (options) =>
      ipcRenderer.invoke(IPC_CHANNELS.NOTIFICATION.SHOW, options),
    onClick: (callback) =>
      createEventListener(IPC_CHANNELS.NOTIFICATION.CLICKED, (data: { id: string }) =>
        callback(data.id)
      ),
    onClose: (callback) =>
      createEventListener(IPC_CHANNELS.NOTIFICATION.CLOSED, (data: { id: string }) =>
        callback(data.id)
      ),
  },

  // Crash reporting
  reportException: (error) =>
    ipcRenderer.send(IPC_CHANNELS.CRASH.EXCEPTION, error),
  reportRejection: (error) =>
    ipcRenderer.send(IPC_CHANNELS.CRASH.REJECTION, error),

  // System events
  onSuspend: (callback) =>
    createEventListener(IPC_CHANNELS.SYSTEM.SUSPEND, callback),
  onResume: (callback) =>
    createEventListener(IPC_CHANNELS.SYSTEM.RESUME, callback),
  onThemeChange: (callback) =>
    createEventListener(IPC_CHANNELS.SYSTEM.THEME_CHANGED, (data: { isDark: boolean }) =>
      callback(data.isDark)
    ),
  onDeepLink: (callback) =>
    createEventListener(IPC_CHANNELS.DEEP_LINK.RECEIVED, (data: { url: string }) =>
      callback(data.url)
    ),
  onConnectivity: (callback) =>
    createEventListener(IPC_CHANNELS.SYSTEM.CONNECTIVITY, (data: { isOnline: boolean }) =>
      callback(data.isOnline)
    ),

  // Shell operations
  shell: {
    openExternal: (url) => ipcRenderer.invoke('shell:openExternal', url),
    openPath: (path) => ipcRenderer.invoke('shell:openPath', path),
    showItemInFolder: (path) => ipcRenderer.send('shell:showItemInFolder', path),
  },

  // Clipboard operations
  clipboard: {
    readText: () => {
      // Note: clipboard access needs special handling
      // This is a simplified version
      return '';
    },
    writeText: (text) => {
      ipcRenderer.send('clipboard:writeText', text);
    },
    readImage: () => null,
    writeImage: (data) => {
      ipcRenderer.send('clipboard:writeImage', btoa(String.fromCharCode(...data)));
    },
  },
};

// Expose the API to the renderer
contextBridge.exposeInMainWorld('electronAPI', electronAPI);

// Type declaration for renderer
declare global {
  interface Window {
    electronAPI: ElectronAPI;
  }
}

// Also export types for use in renderer
export type { ElectronAPI };
```

### Type Definitions

```typescript
// src/electron/preload/types.ts

export interface AppInfo {
  name: string;
  version: string;
  electron: string;
  chrome: string;
  node: string;
  platform: NodeJS.Platform;
  arch: string;
  isPackaged: boolean;
}

export interface FileInfo {
  name: string;
  path: string;
  size: number;
  isDirectory: boolean;
  isFile: boolean;
  isSymlink: boolean;
  created: Date;
  modified: Date;
  accessed: Date;
  extension: string;
}

export interface ReadOptions {
  encoding?: BufferEncoding;
  start?: number;
  end?: number;
}

export interface WriteOptions {
  encoding?: BufferEncoding;
  atomic?: boolean;
}

export interface WatchEvent {
  watchId: string;
  eventType: 'change' | 'rename';
  filename: string;
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

export interface FileFilter {
  name: string;
  extensions: string[];
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

export interface MenuState {
  canUndo: boolean;
  canRedo: boolean;
  hasSelection: boolean;
  isFullScreen: boolean;
  recentFiles: string[];
}

export interface ContextMenuOptions {
  type: 'text' | 'link' | 'image' | 'file' | 'custom';
  selectionText?: string;
  linkURL?: string;
  imageSrc?: string;
  filePath?: string;
  customItems?: MenuItemOptions[];
}

export interface MenuItemOptions {
  label: string;
  type?: 'normal' | 'separator' | 'checkbox' | 'radio';
  enabled?: boolean;
  checked?: boolean;
  accelerator?: string;
  click?: () => void;
}

export interface UpdateState {
  checking: boolean;
  available: boolean;
  downloading: boolean;
  downloaded: boolean;
  error: string | null;
  updateInfo: UpdateInfo | null;
  progress: ProgressInfo | null;
}

export interface UpdateInfo {
  version: string;
  releaseDate: string;
  releaseNotes?: string;
}

export interface ProgressInfo {
  percent: number;
  bytesPerSecond: number;
  total: number;
  transferred: number;
}

export interface NotificationOptions {
  title: string;
  body?: string;
  icon?: string;
  silent?: boolean;
  urgency?: 'normal' | 'critical' | 'low';
  timeoutType?: 'default' | 'never';
  actions?: Array<{ type: 'button'; text: string }>;
}

export interface ErrorInfo {
  name: string;
  message: string;
  stack?: string;
}
```

### Specialized Preload for Settings Window

```typescript
// src/electron/preload/settings.ts
import { contextBridge, ipcRenderer } from 'electron';

interface SettingsAPI {
  getConfig: () => Promise<Record<string, unknown>>;
  setConfig: (key: string, value: unknown) => Promise<void>;
  resetConfig: () => Promise<void>;
  getThemes: () => Promise<string[]>;
  setTheme: (theme: string) => Promise<void>;
  getLanguages: () => Promise<Array<{ code: string; name: string }>>;
  setLanguage: (code: string) => Promise<void>;
}

const settingsAPI: SettingsAPI = {
  getConfig: () => ipcRenderer.invoke('settings:getConfig'),
  setConfig: (key, value) => ipcRenderer.invoke('settings:setConfig', { key, value }),
  resetConfig: () => ipcRenderer.invoke('settings:resetConfig'),
  getThemes: () => ipcRenderer.invoke('settings:getThemes'),
  setTheme: (theme) => ipcRenderer.invoke('settings:setTheme', { theme }),
  getLanguages: () => ipcRenderer.invoke('settings:getLanguages'),
  setLanguage: (code) => ipcRenderer.invoke('settings:setLanguage', { code }),
};

contextBridge.exposeInMainWorld('settingsAPI', settingsAPI);
```

### Preload Build Configuration

```typescript
// electron.vite.config.ts (partial)
import { defineConfig, externalizeDepsPlugin } from 'electron-vite';
import { resolve } from 'path';

export default defineConfig({
  main: {
    plugins: [externalizeDepsPlugin()],
    build: {
      rollupOptions: {
        input: {
          index: resolve(__dirname, 'src/electron/main/index.ts'),
        },
      },
    },
  },
  preload: {
    plugins: [externalizeDepsPlugin()],
    build: {
      rollupOptions: {
        input: {
          index: resolve(__dirname, 'src/electron/preload/index.ts'),
          settings: resolve(__dirname, 'src/electron/preload/settings.ts'),
        },
      },
    },
  },
  renderer: {
    // renderer config
  },
});
```

---

## Testing Requirements

### Unit Tests

```typescript
// src/electron/preload/__tests__/preload.test.ts
import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock electron modules
vi.mock('electron', () => ({
  contextBridge: {
    exposeInMainWorld: vi.fn(),
  },
  ipcRenderer: {
    invoke: vi.fn(),
    on: vi.fn(),
    removeListener: vi.fn(),
    send: vi.fn(),
  },
}));

describe('Preload Script', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should expose electronAPI to renderer', async () => {
    const { contextBridge } = await import('electron');

    // Import preload to trigger setup
    await import('../index');

    expect(contextBridge.exposeInMainWorld).toHaveBeenCalledWith(
      'electronAPI',
      expect.any(Object)
    );
  });

  it('should create proper event listeners', async () => {
    const { ipcRenderer } = await import('electron');

    // Get the exposed API
    const exposedAPI = (await import('electron')).contextBridge
      .exposeInMainWorld as any;

    // Import preload
    await import('../index');

    // Verify event listener pattern
    const api = exposedAPI.mock.calls[0][1];

    const cleanup = api.onSuspend(vi.fn());
    expect(ipcRenderer.on).toHaveBeenCalled();

    cleanup();
    expect(ipcRenderer.removeListener).toHaveBeenCalled();
  });

  it('should handle file data conversion', async () => {
    const { ipcRenderer } = await import('electron');
    (ipcRenderer.invoke as any).mockResolvedValue({
      type: 'buffer',
      data: btoa('test data'),
    });

    await import('../index');

    const api = (await import('electron')).contextBridge.exposeInMainWorld as any;
    const exposedAPI = api.mock.calls[0][1];

    const result = await exposedAPI.fs.readFile('/test/path');
    expect(result).toBeInstanceOf(Uint8Array);
  });
});
```

---

## Related Specs

- Spec 161: Electron Main Process
- Spec 169: Security Configuration
- Spec 170: IPC Channels
- Spec 172: Context Bridge
