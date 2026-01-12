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
  x?: number;
  y?: number;
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

// Main ElectronAPI interface
export interface ElectronAPI {
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

  // Legacy Tachikoma API for backward compatibility
  tachikoma: {
    platform: NodeJS.Platform;
    invoke: (channel: string, ...args: unknown[]) => Promise<unknown>;
    on: (channel: string, callback: (...args: unknown[]) => void) => void;
    off: (channel: string, callback: (...args: unknown[]) => void) => void;
  };
}

// Global type declarations
declare global {
  interface Window {
    electronAPI: ElectronAPI;
    tachikoma: ElectronAPI['tachikoma'];
  }
}

export {};