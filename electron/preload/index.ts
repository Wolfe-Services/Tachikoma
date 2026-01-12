import { contextBridge, ipcRenderer, IpcRendererEvent } from 'electron';
import { IPC_CHANNELS } from '../shared/ipc/channels';
import type { ElectronAPI } from './types';

// Helper function to create event listeners with cleanup
function createEventListener<T>(
  channel: string,
  callback: (data: T) => void
): () => void {
  const handler = (_event: IpcRendererEvent, data: T) => callback(data);
  ipcRenderer.on(channel, handler);
  return () => ipcRenderer.removeListener(channel, handler);
}

// Helper function to wrap IPC calls with error handling
async function safeInvoke<T>(channel: string, ...args: unknown[]): Promise<T> {
  try {
    return await ipcRenderer.invoke(channel, ...args);
  } catch (error) {
    console.error(`IPC invoke failed for channel ${channel}:`, error);
    throw error;
  }
}

// The API to expose
const electronAPI: ElectronAPI = {
  // Platform info
  platform: process.platform,
  isPackaged: process.env.NODE_ENV === 'production',

  // App operations
  getAppInfo: () => safeInvoke(IPC_CHANNELS.APP.GET_INFO),
  getAppPath: (name) => safeInvoke(IPC_CHANNELS.APP.GET_PATH, { name }),
  quit: () => safeInvoke(IPC_CHANNELS.APP.QUIT),
  restart: () => safeInvoke(IPC_CHANNELS.APP.RESTART),

  // Window operations
  minimizeWindow: () => safeInvoke(IPC_CHANNELS.WINDOW.MINIMIZE),
  maximizeWindow: () => safeInvoke(IPC_CHANNELS.WINDOW.MAXIMIZE),
  closeWindow: () => safeInvoke(IPC_CHANNELS.WINDOW.CLOSE),
  isWindowMaximized: () => safeInvoke(IPC_CHANNELS.WINDOW.IS_MAXIMIZED),
  setWindowTitle: (title) =>
    safeInvoke(IPC_CHANNELS.WINDOW.SET_TITLE, { title }),
  onWindowMaximize: (callback) =>
    createEventListener(IPC_CHANNELS.WINDOW.MAXIMIZE_CHANGED, (data: { maximized: boolean }) =>
      callback(data.maximized)
    ),

  // File system operations
  fs: {
    exists: (path) => safeInvoke(IPC_CHANNELS.FS.EXISTS, { path }),
    stat: (path) => safeInvoke(IPC_CHANNELS.FS.STAT, { path }),
    readFile: async (path, options) => {
      const result = await safeInvoke<{ type: 'string' | 'buffer'; data: string }>(
        IPC_CHANNELS.FS.READ_FILE, 
        { path, options }
      );
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
      await safeInvoke(IPC_CHANNELS.FS.WRITE_FILE, { path, data: payload, options });
    },
    deleteFile: (path) => safeInvoke(IPC_CHANNELS.FS.DELETE_FILE, { path }),
    readDirectory: (path) => safeInvoke(IPC_CHANNELS.FS.READ_DIR, { path }),
    createDirectory: (path, recursive) =>
      safeInvoke(IPC_CHANNELS.FS.CREATE_DIR, { path, recursive }),
    watch: async (path, callback) => {
      const { watchId } = await safeInvoke<{ watchId: string }>(IPC_CHANNELS.FS.WATCH, { path });
      const handler = (_event: IpcRendererEvent, data: any) => {
        if (data.watchId === watchId) {
          callback(data);
        }
      };
      ipcRenderer.on(IPC_CHANNELS.FS.WATCH_EVENT, handler);

      return () => {
        ipcRenderer.removeListener(IPC_CHANNELS.FS.WATCH_EVENT, handler);
        safeInvoke(IPC_CHANNELS.FS.UNWATCH, { watchId });
      };
    },
    getAppPath: (name) => safeInvoke(IPC_CHANNELS.APP.GET_PATH, { name }),
  },

  // Dialog operations
  dialog: {
    openFile: (options) =>
      safeInvoke(IPC_CHANNELS.DIALOG.OPEN_FILE, options),
    saveFile: (options) =>
      safeInvoke(IPC_CHANNELS.DIALOG.SAVE_FILE, options),
    showMessage: (options) =>
      safeInvoke(IPC_CHANNELS.DIALOG.MESSAGE, options),
    confirm: (message, detail) =>
      safeInvoke(IPC_CHANNELS.DIALOG.CONFIRM, { message, detail }),
    alert: async (message, detail) => {
      await safeInvoke(IPC_CHANNELS.DIALOG.MESSAGE, {
        type: 'info',
        message,
        detail,
        buttons: ['OK']
      });
    },
    error: (message, detail) =>
      safeInvoke(IPC_CHANNELS.DIALOG.ERROR, { message, detail }),
  },

  // Menu operations
  menu: {
    updateState: (state) =>
      safeInvoke(IPC_CHANNELS.MENU.UPDATE_STATE, { menuState: state }),
    showContextMenu: (options) =>
      safeInvoke(IPC_CHANNELS.MENU.CONTEXT_SHOW, options),
  },

  // Update operations
  updater: {
    check: (silent) => safeInvoke(IPC_CHANNELS.UPDATE.CHECK, { silent }),
    download: () => safeInvoke(IPC_CHANNELS.UPDATE.DOWNLOAD),
    install: () => safeInvoke(IPC_CHANNELS.UPDATE.INSTALL),
    getState: () => safeInvoke(IPC_CHANNELS.UPDATE.GET_STATE),
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
      safeInvoke(IPC_CHANNELS.NOTIFICATION.SHOW, options),
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
    ipcRenderer.send(IPC_CHANNELS.CRASH.EXCEPTION, { error }),
  reportRejection: (error) =>
    ipcRenderer.send(IPC_CHANNELS.CRASH.REJECTION, { reason: error }),

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
    createEventListener(IPC_CHANNELS.SYSTEM.CONNECTIVITY, (data: { online: boolean }) =>
      callback(data.online)
    ),

  // Shell operations
  shell: {
    openExternal: (url) => safeInvoke('shell:openExternal', url),
    openPath: (path) => safeInvoke('shell:openPath', path),
    showItemInFolder: (path) => ipcRenderer.send('shell:showItemInFolder', path),
  },

  // Clipboard operations
  clipboard: {
    readText: () => {
      // Note: Synchronous clipboard access is limited in sandboxed renderer
      // Consider implementing async version via IPC if needed
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

  // Legacy Tachikoma API for backward compatibility
  tachikoma: {
    platform: process.platform,
    invoke: (channel: string, ...args: unknown[]) => {
      const validChannels = [
        'mission:start',
        'mission:stop',
        'mission:status',
        'spec:list',
        'spec:read',
        'config:get',
        'config:set'
      ];
      if (validChannels.includes(channel)) {
        return safeInvoke(channel, ...args);
      }
      throw new Error(`Invalid channel: ${channel}`);
    },
    on: (channel: string, callback: (...args: unknown[]) => void) => {
      const validChannels = [
        'mission:progress',
        'mission:log',
        'mission:complete',
        'mission:error'
      ];
      if (validChannels.includes(channel)) {
        ipcRenderer.on(channel, (_event, ...args) => callback(...args));
      }
    },
    off: (channel: string, callback: (...args: unknown[]) => void) => {
      ipcRenderer.removeListener(channel, callback);
    }
  }
};

// Expose the API to the renderer
contextBridge.exposeInMainWorld('electronAPI', electronAPI);

// Also expose legacy tachikoma API for compatibility
contextBridge.exposeInMainWorld('tachikoma', electronAPI.tachikoma);

// Handle uncaught errors in preload context
process.on('uncaughtException', (error) => {
  console.error('Preload uncaught exception:', error);
  electronAPI.reportException({
    name: error.name,
    message: error.message,
    stack: error.stack
  });
});

process.on('unhandledRejection', (reason) => {
  console.error('Preload unhandled rejection:', reason);
  electronAPI.reportRejection({
    name: 'UnhandledRejection',
    message: String(reason),
    stack: reason instanceof Error ? reason.stack : undefined
  });
});

// Export types for TypeScript
export type { ElectronAPI };