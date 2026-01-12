import { vi } from 'vitest';

// Mock Electron module
vi.mock('electron', () => {
  const mockApp = {
    getPath: vi.fn((name: string) => `/mock/${name}`),
    getVersion: vi.fn(() => '1.0.0-test'),
    getName: vi.fn(() => 'Tachikoma'),
    isPackaged: false,
    on: vi.fn(),
    once: vi.fn(),
    quit: vi.fn(),
    exit: vi.fn(),
    relaunch: vi.fn(),
    requestSingleInstanceLock: vi.fn(() => true),
    setAsDefaultProtocolClient: vi.fn(),
    enableSandbox: vi.fn(),
    whenReady: vi.fn(() => Promise.resolve()),
    dock: {
      setBadge: vi.fn(),
      bounce: vi.fn(),
      setIcon: vi.fn(),
    },
  };

  const mockBrowserWindow = vi.fn().mockImplementation(() => ({
    loadFile: vi.fn().mockResolvedValue(undefined),
    loadURL: vi.fn().mockResolvedValue(undefined),
    on: vi.fn(),
    once: vi.fn(),
    show: vi.fn(),
    hide: vi.fn(),
    close: vi.fn(),
    focus: vi.fn(),
    minimize: vi.fn(),
    maximize: vi.fn(),
    restore: vi.fn(),
    isMinimized: vi.fn(() => false),
    isMaximized: vi.fn(() => false),
    isDestroyed: vi.fn(() => false),
    isVisible: vi.fn(() => true),
    setTitle: vi.fn(),
    getTitle: vi.fn(() => 'Tachikoma'),
    getBounds: vi.fn(() => ({ x: 0, y: 0, width: 1200, height: 800 })),
    setBounds: vi.fn(),
    setProgressBar: vi.fn(),
    webContents: {
      send: vi.fn(),
      on: vi.fn(),
      once: vi.fn(),
      openDevTools: vi.fn(),
      closeDevTools: vi.fn(),
      isDevToolsOpened: vi.fn(() => false),
      setWindowOpenHandler: vi.fn(),
      getURL: vi.fn(() => 'http://localhost:5173'),
      session: {
        clearCache: vi.fn().mockResolvedValue(undefined),
      },
    },
  }));

  (mockBrowserWindow as any).getAllWindows = vi.fn(() => []);
  (mockBrowserWindow as any).fromWebContents = vi.fn();

  return {
    app: mockApp,
    BrowserWindow: mockBrowserWindow,
    ipcMain: {
      handle: vi.fn(),
      on: vi.fn(),
      once: vi.fn(),
      removeHandler: vi.fn(),
      removeListener: vi.fn(),
    },
    ipcRenderer: {
      invoke: vi.fn(),
      on: vi.fn(),
      once: vi.fn(),
      send: vi.fn(),
      removeListener: vi.fn(),
    },
    contextBridge: {
      exposeInMainWorld: vi.fn(),
    },
    session: {
      defaultSession: {
        webRequest: {
          onHeadersReceived: vi.fn(),
          onBeforeRequest: vi.fn(),
        },
        setPermissionRequestHandler: vi.fn(),
        setPermissionCheckHandler: vi.fn(),
        clearCache: vi.fn(),
        loadExtension: vi.fn(),
      },
    },
    protocol: {
      registerSchemesAsPrivileged: vi.fn(),
      handle: vi.fn(),
      interceptFileProtocol: vi.fn(),
      unhandle: vi.fn(),
    },
    dialog: {
      showOpenDialog: vi.fn().mockResolvedValue({ canceled: false, filePaths: [] }),
      showSaveDialog: vi.fn().mockResolvedValue({ canceled: false, filePath: '' }),
      showMessageBox: vi.fn().mockResolvedValue({ response: 0, checkboxChecked: false }),
      showErrorBox: vi.fn(),
    },
    shell: {
      openExternal: vi.fn().mockResolvedValue(undefined),
      openPath: vi.fn().mockResolvedValue(''),
      showItemInFolder: vi.fn(),
    },
    nativeTheme: {
      shouldUseDarkColors: false,
      themeSource: 'system',
      on: vi.fn(),
    },
    Menu: {
      buildFromTemplate: vi.fn(() => ({})),
      setApplicationMenu: vi.fn(),
      getApplicationMenu: vi.fn(),
    },
    Tray: vi.fn().mockImplementation(() => ({
      on: vi.fn(),
      setToolTip: vi.fn(),
      setContextMenu: vi.fn(),
      setImage: vi.fn(),
      destroy: vi.fn(),
    })),
    Notification: vi.fn().mockImplementation(() => ({
      on: vi.fn(),
      show: vi.fn(),
      close: vi.fn(),
    })),
    nativeImage: {
      createFromPath: vi.fn(() => ({
        resize: vi.fn().mockReturnThis(),
        toDataURL: vi.fn(() => ''),
      })),
    },
    powerMonitor: {
      on: vi.fn(),
    },
    powerSaveBlocker: {
      start: vi.fn(() => 1),
      stop: vi.fn(),
    },
    crashReporter: {
      start: vi.fn(),
      getUploadedReports: vi.fn(() => []),
    },
    clipboard: {
      readText: vi.fn(() => ''),
      writeText: vi.fn(),
    },
    screen: {
      getAllDisplays: vi.fn(() => [{ bounds: { x: 0, y: 0, width: 1920, height: 1080 } }]),
      getPrimaryDisplay: vi.fn(() => ({ workAreaSize: { width: 1920, height: 1040 } })),
    },
    net: {
      fetch: vi.fn(),
    },
  };
});

// Mock @electron-toolkit/utils
vi.mock('@electron-toolkit/utils', () => ({
  electronApp: {
    setAppUserModelId: vi.fn(),
  },
  optimizer: {
    watchWindowShortcuts: vi.fn(),
  },
  is: {
    dev: true,
  },
}));

// Mock fs for file system tests
vi.mock('fs', async () => {
  const memfs = await import('memfs');
  return {
    ...memfs.fs,
    promises: memfs.fs.promises,
  };
});

// Mock path module
vi.mock('path', async () => {
  const actualPath = await vi.importActual('path');
  return {
    ...actualPath,
    join: vi.fn((...args: string[]) => args.join('/')),
  };
});

// Global test utilities
globalThis.createMockWindow = () => {
  const { BrowserWindow } = require('electron');
  return new BrowserWindow();
};

globalThis.createMockEvent = () => ({
  sender: {
    id: 1,
    send: vi.fn(),
    isDestroyed: () => false,
  },
});

// Mock native module for testing
vi.mock('../main/native', () => ({
  native: {
    startMission: vi.fn().mockResolvedValue('test-mission-id'),
    stopMission: vi.fn().mockResolvedValue(true),
    getMissionStatus: vi.fn().mockResolvedValue({
      id: 'test-mission',
      status: 'running',
      progress: 0.5,
      message: 'Test mission running',
    }),
    listSpecs: vi.fn().mockResolvedValue({
      specs: [
        { path: '/test/spec1.md', title: 'Test Spec 1' },
        { path: '/test/spec2.md', title: 'Test Spec 2' },
      ],
    }),
    readSpec: vi.fn().mockResolvedValue({
      content: '# Test Spec\n\nTest content',
      metadata: { title: 'Test Spec', id: '001' },
    }),
    getConfig: vi.fn().mockResolvedValue({ value: 'test-value' }),
    setConfig: vi.fn().mockResolvedValue(true),
  },
}));