# Spec 170: IPC Channels

## Phase
8 - Electron Shell

## Spec ID
170

## Status
Planned

## Dependencies
- Spec 161 (Electron Main Process)
- Spec 169 (Security Configuration)
- Spec 171 (Preload Scripts)

## Estimated Context
~10%

---

## Objective

Implement a type-safe, secure Inter-Process Communication (IPC) system between the main process and renderer processes. Define clear channel conventions, validate messages, and provide both synchronous and asynchronous communication patterns.

---

## Acceptance Criteria

- [ ] Type-safe IPC channel definitions
- [ ] Bidirectional communication (main to renderer, renderer to main)
- [ ] Request/response pattern with invoke/handle
- [ ] Event broadcasting from main to renderers
- [ ] Channel validation and sanitization
- [ ] Error handling and propagation
- [ ] IPC logging for debugging
- [ ] Rate limiting for security
- [ ] Message serialization/deserialization
- [ ] Timeout handling for long operations

---

## Implementation Details

### IPC Channel Definitions

```typescript
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
```

### IPC Type Definitions

```typescript
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

  // File system
  [IPC_CHANNELS.FS.EXISTS]: { path: string };
  [IPC_CHANNELS.FS.STAT]: { path: string };
  [IPC_CHANNELS.FS.READ_FILE]: { path: string; options?: { encoding?: string } };
  [IPC_CHANNELS.FS.WRITE_FILE]: { path: string; data: string; options?: { atomic?: boolean } };
  [IPC_CHANNELS.FS.DELETE_FILE]: { path: string };
  [IPC_CHANNELS.FS.READ_DIR]: { path: string };
  [IPC_CHANNELS.FS.CREATE_DIR]: { path: string; recursive?: boolean };

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

  // Dialogs
  [IPC_CHANNELS.DIALOG.OPEN_FILE]: string[];
  [IPC_CHANNELS.DIALOG.SAVE_FILE]: string | null;
  [IPC_CHANNELS.DIALOG.MESSAGE]: { response: number; checkboxChecked: boolean };
  [IPC_CHANNELS.DIALOG.CONFIRM]: boolean;
}

// Event type mapping (main to renderer)
export interface IPCEventMap {
  [IPC_CHANNELS.WINDOW.MAXIMIZE_CHANGED]: { maximized: boolean };
  [IPC_CHANNELS.SYSTEM.SUSPEND]: void;
  [IPC_CHANNELS.SYSTEM.RESUME]: void;
  [IPC_CHANNELS.SYSTEM.BATTERY]: { onBattery: boolean };
  [IPC_CHANNELS.SYSTEM.THEME_CHANGED]: { isDark: boolean };
  [IPC_CHANNELS.UPDATE.CHECKING]: void;
  [IPC_CHANNELS.UPDATE.AVAILABLE]: { version: string; releaseDate: string };
  [IPC_CHANNELS.UPDATE.NOT_AVAILABLE]: { version: string };
  [IPC_CHANNELS.UPDATE.PROGRESS]: { percent: number; bytesPerSecond: number };
  [IPC_CHANNELS.UPDATE.DOWNLOADED]: { version: string };
  [IPC_CHANNELS.UPDATE.ERROR]: { message: string };
  [IPC_CHANNELS.DEEP_LINK.RECEIVED]: { url: string };
  [IPC_CHANNELS.FS.WATCH_EVENT]: { watchId: string; eventType: string; filename: string };
  [IPC_CHANNELS.NOTIFICATION.CLICKED]: { id: string };
  [IPC_CHANNELS.NOTIFICATION.CLOSED]: { id: string };
}
```

### Main Process IPC Handler

```typescript
// src/electron/main/ipc/handler.ts
import { ipcMain, IpcMainInvokeEvent, BrowserWindow } from 'electron';
import { Logger } from '../logger';
import { IPC_CHANNELS } from '../../../shared/ipc/channels';

const logger = new Logger('ipc');

interface HandlerConfig {
  timeout?: number;
  rateLimit?: {
    maxCalls: number;
    windowMs: number;
  };
}

const rateLimitMap = new Map<string, Map<number, number[]>>();

class IPCHandler {
  private handlers = new Map<string, Function>();

  register<T, R>(
    channel: string,
    handler: (event: IpcMainInvokeEvent, args: T) => Promise<R> | R,
    config: HandlerConfig = {}
  ): void {
    if (this.handlers.has(channel)) {
      logger.warn('Handler already registered, overwriting', { channel });
    }

    const wrappedHandler = async (
      event: IpcMainInvokeEvent,
      args: T
    ): Promise<R> => {
      const startTime = Date.now();
      const senderId = event.sender.id;

      try {
        // Rate limiting
        if (config.rateLimit) {
          this.checkRateLimit(channel, senderId, config.rateLimit);
        }

        logger.debug('IPC invoke', { channel, senderId });

        // Execute with timeout
        const result = config.timeout
          ? await this.withTimeout(handler(event, args), config.timeout)
          : await handler(event, args);

        logger.debug('IPC response', {
          channel,
          duration: Date.now() - startTime,
        });

        return result;
      } catch (error) {
        logger.error('IPC error', {
          channel,
          error: error instanceof Error ? error.message : String(error),
        });
        throw error;
      }
    };

    ipcMain.handle(channel, wrappedHandler);
    this.handlers.set(channel, wrappedHandler);
  }

  private checkRateLimit(
    channel: string,
    senderId: number,
    config: { maxCalls: number; windowMs: number }
  ): void {
    if (!rateLimitMap.has(channel)) {
      rateLimitMap.set(channel, new Map());
    }

    const channelMap = rateLimitMap.get(channel)!;
    const now = Date.now();
    const calls = channelMap.get(senderId) || [];

    // Remove old calls outside the window
    const recentCalls = calls.filter(
      (timestamp) => now - timestamp < config.windowMs
    );

    if (recentCalls.length >= config.maxCalls) {
      throw new Error(`Rate limit exceeded for channel: ${channel}`);
    }

    recentCalls.push(now);
    channelMap.set(senderId, recentCalls);
  }

  private withTimeout<T>(promise: Promise<T>, ms: number): Promise<T> {
    return Promise.race([
      promise,
      new Promise<T>((_, reject) =>
        setTimeout(() => reject(new Error('IPC timeout')), ms)
      ),
    ]);
  }

  unregister(channel: string): void {
    if (this.handlers.has(channel)) {
      ipcMain.removeHandler(channel);
      this.handlers.delete(channel);
      logger.debug('Handler unregistered', { channel });
    }
  }

  send(window: BrowserWindow, channel: string, data?: unknown): void {
    if (!window.isDestroyed()) {
      window.webContents.send(channel, data);
      logger.debug('IPC send', { channel, windowId: window.id });
    }
  }

  broadcast(channel: string, data?: unknown): void {
    const windows = BrowserWindow.getAllWindows();
    for (const window of windows) {
      this.send(window, channel, data);
    }
    logger.debug('IPC broadcast', { channel, windowCount: windows.length });
  }
}

export const ipcHandler = new IPCHandler();
```

### IPC Setup Module

```typescript
// src/electron/main/ipc/index.ts
import { BrowserWindow } from 'electron';
import { ipcHandler } from './handler';
import { setupWindowIpcHandlers } from './window';
import { setupFsIpcHandlers } from './fs';
import { setupDialogIpcHandlers } from './dialogs';
import { setupMenuIpcHandlers } from './menu';
import { setupUpdaterIpcHandlers } from './updater';
import { setupLifecycleIpcHandlers } from './lifecycle';
import { setupCrashReporterIpcHandlers } from './crash-reporter';
import { setupNotificationIpcHandlers } from './notifications';
import { setupSecurityIpcHandlers } from './security';
import { Logger } from '../logger';

const logger = new Logger('ipc');

export function setupIpcHandlers(mainWindow: BrowserWindow): void {
  logger.info('Setting up IPC handlers');

  // Setup all domain handlers
  setupWindowIpcHandlers();
  setupFsIpcHandlers();
  setupDialogIpcHandlers();
  setupMenuIpcHandlers(mainWindow);
  setupUpdaterIpcHandlers();
  setupLifecycleIpcHandlers();
  setupCrashReporterIpcHandlers();
  setupNotificationIpcHandlers();
  setupSecurityIpcHandlers();

  logger.info('IPC handlers setup complete');
}

export { ipcHandler };
```

### Typed IPC Utility for Renderer

```typescript
// src/shared/ipc/client.ts
import type { IPCRequestMap, IPCResponseMap, IPCEventMap } from './types';

export type IPCClient = {
  invoke<K extends keyof IPCRequestMap>(
    channel: K,
    args?: IPCRequestMap[K]
  ): Promise<IPCResponseMap[K]>;

  on<K extends keyof IPCEventMap>(
    channel: K,
    callback: (data: IPCEventMap[K]) => void
  ): () => void;

  once<K extends keyof IPCEventMap>(
    channel: K,
    callback: (data: IPCEventMap[K]) => void
  ): void;

  send(channel: string, data?: unknown): void;
};

// This will be implemented in the preload script
```

### IPC Validation

```typescript
// src/electron/main/ipc/validation.ts
import { z } from 'zod';

// Validation schemas for IPC arguments
export const schemas = {
  path: z.string().min(1).max(4096),
  title: z.string().max(256),
  message: z.string().max(4096),

  fileOptions: z.object({
    title: z.string().max(256).optional(),
    filters: z
      .array(
        z.object({
          name: z.string(),
          extensions: z.array(z.string()),
        })
      )
      .optional(),
    multiSelect: z.boolean().optional(),
  }),

  writeFileArgs: z.object({
    path: z.string().min(1),
    data: z.string(),
    options: z
      .object({
        encoding: z.string().optional(),
        atomic: z.boolean().optional(),
      })
      .optional(),
  }),

  messageBoxArgs: z.object({
    type: z.enum(['info', 'error', 'warning', 'question', 'none']).optional(),
    title: z.string().max(256).optional(),
    message: z.string().max(4096),
    detail: z.string().max(4096).optional(),
    buttons: z.array(z.string()).max(10).optional(),
  }),
};

export function validate<T>(schema: z.ZodSchema<T>, data: unknown): T {
  return schema.parse(data);
}

export function validateAsync<T>(
  schema: z.ZodSchema<T>,
  data: unknown
): Promise<T> {
  return schema.parseAsync(data);
}
```

### IPC Debug Tools

```typescript
// src/electron/main/ipc/debug.ts
import { ipcMain, BrowserWindow } from 'electron';
import { Logger } from '../logger';

const logger = new Logger('ipc-debug');

interface IPCMessage {
  timestamp: number;
  direction: 'in' | 'out';
  channel: string;
  args?: unknown;
  response?: unknown;
  error?: string;
  duration?: number;
}

const messageLog: IPCMessage[] = [];
const MAX_LOG_SIZE = 1000;

export function enableIPCLogging(): void {
  // Intercept ipcMain handlers
  const originalHandle = ipcMain.handle.bind(ipcMain);

  ipcMain.handle = (channel: string, listener: Function) => {
    const wrappedListener = async (event: any, ...args: any[]) => {
      const startTime = Date.now();

      logMessage({
        timestamp: startTime,
        direction: 'in',
        channel,
        args: args[0],
      });

      try {
        const result = await listener(event, ...args);

        logMessage({
          timestamp: Date.now(),
          direction: 'out',
          channel,
          response: result,
          duration: Date.now() - startTime,
        });

        return result;
      } catch (error) {
        logMessage({
          timestamp: Date.now(),
          direction: 'out',
          channel,
          error: error instanceof Error ? error.message : String(error),
          duration: Date.now() - startTime,
        });
        throw error;
      }
    };

    return originalHandle(channel, wrappedListener);
  };

  logger.info('IPC logging enabled');
}

function logMessage(message: IPCMessage): void {
  messageLog.push(message);

  if (messageLog.length > MAX_LOG_SIZE) {
    messageLog.shift();
  }

  if (message.direction === 'in') {
    logger.debug(`IPC <- ${message.channel}`, { args: message.args });
  } else {
    logger.debug(`IPC -> ${message.channel}`, {
      response: message.response,
      error: message.error,
      duration: message.duration,
    });
  }
}

export function getIPCLog(): IPCMessage[] {
  return [...messageLog];
}

export function clearIPCLog(): void {
  messageLog.length = 0;
}

export function sendIPCLogToDevTools(window: BrowserWindow): void {
  window.webContents.send('ipc-debug:log', messageLog);
}
```

---

## Testing Requirements

### Unit Tests

```typescript
// src/electron/main/ipc/__tests__/handler.test.ts
import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('electron', () => ({
  ipcMain: {
    handle: vi.fn(),
    removeHandler: vi.fn(),
  },
  BrowserWindow: {
    getAllWindows: vi.fn().mockReturnValue([]),
    fromWebContents: vi.fn(),
  },
}));

describe('IPCHandler', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should register handlers', async () => {
    const { ipcMain } = await import('electron');
    const { IPCHandler } = await import('../handler');

    const handler = new (IPCHandler as any)();
    handler.register('test:channel', async () => 'response');

    expect(ipcMain.handle).toHaveBeenCalledWith(
      'test:channel',
      expect.any(Function)
    );
  });

  it('should enforce rate limits', async () => {
    const { IPCHandler } = await import('../handler');

    const handler = new (IPCHandler as any)();
    const mockHandler = vi.fn().mockResolvedValue('ok');

    handler.register('test:limited', mockHandler, {
      rateLimit: { maxCalls: 2, windowMs: 1000 },
    });

    // This test would need to simulate multiple calls
  });

  it('should handle timeouts', async () => {
    const { IPCHandler } = await import('../handler');

    const handler = new (IPCHandler as any)();
    const slowHandler = () =>
      new Promise((resolve) => setTimeout(() => resolve('slow'), 5000));

    handler.register('test:slow', slowHandler, { timeout: 100 });

    // This test would verify timeout behavior
  });
});
```

### Integration Tests

```typescript
// src/electron/main/ipc/__tests__/ipc.integration.test.ts
import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { _electron as electron } from 'playwright';
import type { ElectronApplication, Page } from 'playwright';

describe('IPC Integration', () => {
  let electronApp: ElectronApplication;
  let page: Page;

  beforeAll(async () => {
    electronApp = await electron.launch({ args: ['.'] });
    page = await electronApp.firstWindow();
  });

  afterAll(async () => {
    await electronApp.close();
  });

  it('should handle invoke/handle pattern', async () => {
    const result = await electronApp.evaluate(async ({ ipcRenderer }) => {
      return ipcRenderer.invoke('app:getInfo');
    });

    expect(result).toHaveProperty('version');
    expect(result).toHaveProperty('electron');
  });

  it('should receive events from main process', async () => {
    // Would need to test event broadcasting
  });
});
```

---

## Related Specs

- Spec 161: Electron Main Process
- Spec 169: Security Configuration
- Spec 171: Preload Scripts
- Spec 172: Context Bridge
