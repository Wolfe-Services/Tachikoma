// src/shared/ipc/types.ts
import { IPC_CHANNELS } from './channels';

// Request/Response type mapping
export interface IPCRequestMap {
  // App
  [IPC_CHANNELS.APP.GET_INFO]: void;
  [IPC_CHANNELS.APP.GET_PATH]: { name: string };
  [IPC_CHANNELS.APP.QUIT]: void;
  [IPC_CHANNELS.APP.RESTART]: void;

  // Window
  [IPC_CHANNELS.WINDOW.MINIMIZE]: void;
  [IPC_CHANNELS.WINDOW.MAXIMIZE]: void;
  [IPC_CHANNELS.WINDOW.CLOSE]: void;
  [IPC_CHANNELS.WINDOW.IS_MAXIMIZED]: void;
  [IPC_CHANNELS.WINDOW.SET_TITLE]: { title: string };
  [IPC_CHANNELS.WINDOW.OPEN_SETTINGS]: void;
  [IPC_CHANNELS.WINDOW.OPEN_ABOUT]: void;

  // File system
  [IPC_CHANNELS.FS.EXISTS]: { path: string };
  [IPC_CHANNELS.FS.STAT]: { path: string };
  [IPC_CHANNELS.FS.READ_FILE]: { path: string; options?: { encoding?: string } };
  [IPC_CHANNELS.FS.WRITE_FILE]: { path: string; data: string; options?: { atomic?: boolean } };
  [IPC_CHANNELS.FS.DELETE_FILE]: { path: string };
  [IPC_CHANNELS.FS.READ_DIR]: { path: string };
  [IPC_CHANNELS.FS.CREATE_DIR]: { path: string; recursive?: boolean };
  [IPC_CHANNELS.FS.WATCH]: { path: string; options?: { recursive?: boolean } };
  [IPC_CHANNELS.FS.UNWATCH]: { watchId: string };

  // Dialogs
  [IPC_CHANNELS.DIALOG.OPEN_FILE]: {
    title?: string;
    filters?: Array<{ name: string; extensions: string[] }>;
    multiSelect?: boolean;
  };
  [IPC_CHANNELS.DIALOG.SAVE_FILE]: {
    title?: string;
    defaultPath?: string;
    filters?: Array<{ name: string; extensions: string[] }>;
  };
  [IPC_CHANNELS.DIALOG.MESSAGE]: {
    type?: 'info' | 'error' | 'warning' | 'question';
    title?: string;
    message: string;
    detail?: string;
    buttons?: string[];
  };
  [IPC_CHANNELS.DIALOG.CONFIRM]: { message: string; detail?: string };
  [IPC_CHANNELS.DIALOG.ERROR]: { message: string; detail?: string };

  // Menu
  [IPC_CHANNELS.MENU.UPDATE_STATE]: { menuState: Record<string, boolean> };
  [IPC_CHANNELS.MENU.CONTEXT_SHOW]: { x: number; y: number; template?: any };

  // Updates
  [IPC_CHANNELS.UPDATE.CHECK]: { silent?: boolean };
  [IPC_CHANNELS.UPDATE.DOWNLOAD]: void;
  [IPC_CHANNELS.UPDATE.INSTALL]: void;
  [IPC_CHANNELS.UPDATE.GET_STATE]: void;

  // System
  [IPC_CHANNELS.SYSTEM.SUSPEND]: void;
  [IPC_CHANNELS.SYSTEM.RESUME]: void;
  [IPC_CHANNELS.SYSTEM.BATTERY]: void;

  // Notifications
  [IPC_CHANNELS.NOTIFICATION.SHOW]: {
    title: string;
    body: string;
    icon?: string;
    silent?: boolean;
    urgency?: 'normal' | 'critical' | 'low';
  };

  // Crash reporting
  [IPC_CHANNELS.CRASH.EXCEPTION]: { error: Error };
  [IPC_CHANNELS.CRASH.REJECTION]: { reason: any };
  [IPC_CHANNELS.CRASH.GET_REPORTS]: void;
}

export interface IPCResponseMap {
  // App
  [IPC_CHANNELS.APP.GET_INFO]: {
    name: string;
    version: string;
    electron: string;
    platform: string;
  };
  [IPC_CHANNELS.APP.GET_PATH]: string;
  [IPC_CHANNELS.APP.QUIT]: void;
  [IPC_CHANNELS.APP.RESTART]: void;

  // Window
  [IPC_CHANNELS.WINDOW.MINIMIZE]: void;
  [IPC_CHANNELS.WINDOW.MAXIMIZE]: void;
  [IPC_CHANNELS.WINDOW.CLOSE]: void;
  [IPC_CHANNELS.WINDOW.IS_MAXIMIZED]: boolean;
  [IPC_CHANNELS.WINDOW.SET_TITLE]: void;
  [IPC_CHANNELS.WINDOW.OPEN_SETTINGS]: void;
  [IPC_CHANNELS.WINDOW.OPEN_ABOUT]: void;

  // File system
  [IPC_CHANNELS.FS.EXISTS]: boolean;
  [IPC_CHANNELS.FS.STAT]: {
    name: string;
    path: string;
    size: number;
    isDirectory: boolean;
    isFile: boolean;
    modified: string;
  };
  [IPC_CHANNELS.FS.READ_FILE]: { type: 'string' | 'buffer'; data: string };
  [IPC_CHANNELS.FS.WRITE_FILE]: void;
  [IPC_CHANNELS.FS.DELETE_FILE]: void;
  [IPC_CHANNELS.FS.READ_DIR]: Array<{
    name: string;
    path: string;
    isDirectory: boolean;
  }>;
  [IPC_CHANNELS.FS.CREATE_DIR]: void;
  [IPC_CHANNELS.FS.WATCH]: { watchId: string };
  [IPC_CHANNELS.FS.UNWATCH]: { success: boolean };

  // Dialogs
  [IPC_CHANNELS.DIALOG.OPEN_FILE]: string[];
  [IPC_CHANNELS.DIALOG.SAVE_FILE]: string | null;
  [IPC_CHANNELS.DIALOG.MESSAGE]: { response: number; checkboxChecked: boolean };
  [IPC_CHANNELS.DIALOG.CONFIRM]: boolean;
  [IPC_CHANNELS.DIALOG.ERROR]: void;

  // Menu
  [IPC_CHANNELS.MENU.UPDATE_STATE]: void;
  [IPC_CHANNELS.MENU.CONTEXT_SHOW]: void;

  // Updates
  [IPC_CHANNELS.UPDATE.CHECK]: { hasUpdate: boolean; version?: string };
  [IPC_CHANNELS.UPDATE.DOWNLOAD]: { success: boolean };
  [IPC_CHANNELS.UPDATE.INSTALL]: { success: boolean };
  [IPC_CHANNELS.UPDATE.GET_STATE]: {
    state: 'idle' | 'checking' | 'available' | 'downloading' | 'downloaded' | 'error';
    version?: string;
    progress?: number;
  };

  // System
  [IPC_CHANNELS.SYSTEM.SUSPEND]: void;
  [IPC_CHANNELS.SYSTEM.RESUME]: void;
  [IPC_CHANNELS.SYSTEM.BATTERY]: { onBattery: boolean; level?: number };

  // Notifications
  [IPC_CHANNELS.NOTIFICATION.SHOW]: { id: string };

  // Crash reporting
  [IPC_CHANNELS.CRASH.EXCEPTION]: void;
  [IPC_CHANNELS.CRASH.REJECTION]: void;
  [IPC_CHANNELS.CRASH.GET_REPORTS]: Array<{ id: string; date: string; error: string }>;
}

// Event type mapping (main to renderer)
export interface IPCEventMap {
  [IPC_CHANNELS.APP.READY]: void;
  [IPC_CHANNELS.WINDOW.MAXIMIZE_CHANGED]: { maximized: boolean };
  [IPC_CHANNELS.SYSTEM.SUSPEND]: void;
  [IPC_CHANNELS.SYSTEM.RESUME]: void;
  [IPC_CHANNELS.SYSTEM.BATTERY]: { onBattery: boolean; level?: number };
  [IPC_CHANNELS.SYSTEM.THEME_CHANGED]: { isDark: boolean };
  [IPC_CHANNELS.SYSTEM.CONNECTIVITY]: { online: boolean };
  [IPC_CHANNELS.UPDATE.CHECKING]: void;
  [IPC_CHANNELS.UPDATE.AVAILABLE]: { version: string; releaseDate: string };
  [IPC_CHANNELS.UPDATE.NOT_AVAILABLE]: { version: string };
  [IPC_CHANNELS.UPDATE.PROGRESS]: { percent: number; bytesPerSecond: number };
  [IPC_CHANNELS.UPDATE.DOWNLOADED]: { version: string };
  [IPC_CHANNELS.UPDATE.ERROR]: { message: string };
  [IPC_CHANNELS.DEEP_LINK.RECEIVED]: { url: string };
  [IPC_CHANNELS.DEEP_LINK.OPEN_FILES]: { files: string[] };
  [IPC_CHANNELS.FS.WATCH_EVENT]: { watchId: string; eventType: string; filename: string };
  [IPC_CHANNELS.NOTIFICATION.CLICKED]: { id: string };
  [IPC_CHANNELS.NOTIFICATION.CLOSED]: { id: string };
}