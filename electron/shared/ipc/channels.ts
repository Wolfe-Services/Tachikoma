// src/shared/ipc/channels.ts

// Channel naming convention: domain:action
export const IPC_CHANNELS = {
  // App lifecycle
  APP: {
    GET_INFO: 'app:getInfo',
    GET_PATH: 'app:getPath',
    QUIT: 'app:quit',
    RESTART: 'app:restart',
    READY: 'app:ready',
  },

  // Window operations
  WINDOW: {
    MINIMIZE: 'window:minimize',
    MAXIMIZE: 'window:maximize',
    CLOSE: 'window:close',
    IS_MAXIMIZED: 'window:isMaximized',
    SET_TITLE: 'window:setTitle',
    OPEN_SETTINGS: 'window:openSettings',
    OPEN_ABOUT: 'window:openAbout',
    MAXIMIZE_CHANGED: 'window:maximizeChanged',
  },

  // File system
  FS: {
    EXISTS: 'fs:exists',
    STAT: 'fs:stat',
    READ_FILE: 'fs:readFile',
    WRITE_FILE: 'fs:writeFile',
    DELETE_FILE: 'fs:deleteFile',
    READ_DIR: 'fs:readDirectory',
    CREATE_DIR: 'fs:createDirectory',
    WATCH: 'fs:watch',
    UNWATCH: 'fs:unwatch',
    WATCH_EVENT: 'fs:watchEvent',
  },

  // Dialogs
  DIALOG: {
    OPEN_FILE: 'dialog:openFile',
    SAVE_FILE: 'dialog:saveFile',
    MESSAGE: 'dialog:message',
    CONFIRM: 'dialog:confirm',
    ERROR: 'dialog:error',
  },

  // Menu
  MENU: {
    UPDATE_STATE: 'menu:updateState',
    CONTEXT_SHOW: 'contextMenu:show',
  },

  // Updates
  UPDATE: {
    CHECK: 'updater:check',
    DOWNLOAD: 'updater:download',
    INSTALL: 'updater:install',
    GET_STATE: 'updater:getState',
    CHECKING: 'update:checking',
    AVAILABLE: 'update:available',
    NOT_AVAILABLE: 'update:not-available',
    PROGRESS: 'update:progress',
    DOWNLOADED: 'update:downloaded',
    ERROR: 'update:error',
  },

  // System
  SYSTEM: {
    SUSPEND: 'system:suspend',
    RESUME: 'system:resume',
    BATTERY: 'system:battery',
    THEME_CHANGED: 'theme:changed',
    CONNECTIVITY: 'lifecycle:connectivity',
  },

  // Notifications
  NOTIFICATION: {
    SHOW: 'notification:show',
    CLICKED: 'notification:clicked',
    CLOSED: 'notification:closed',
  },

  // Crash reporting
  CRASH: {
    EXCEPTION: 'crash-reporter:exception',
    REJECTION: 'crash-reporter:rejection',
    GET_REPORTS: 'crash-reporter:getReports',
  },

  // Deep linking
  DEEP_LINK: {
    RECEIVED: 'deep-link',
    OPEN_FILES: 'open-files',
  },
} as const;

// Type utilities
export type IPCChannels = typeof IPC_CHANNELS;
export type ChannelKey<T> = T extends Record<string, infer V>
  ? V extends string
    ? V
    : V extends Record<string, string>
    ? ChannelKey<V>
    : never
  : never;

export type AllChannels = ChannelKey<IPCChannels>;