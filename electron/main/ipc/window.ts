// src/electron/main/ipc/window.ts
import { BrowserWindow, app } from 'electron';
import { ipcHandler } from './handler';
import { IPC_CHANNELS } from '../../shared/ipc/channels';
import { validate, schemas } from './validation';
import { Logger } from '../logger';

const logger = new Logger('ipc:window');

export function setupWindowIpcHandlers(mainWindow?: BrowserWindow): void {
  logger.info('Setting up window IPC handlers');

  // Get current window or main window
  const getCurrentWindow = (event: any): BrowserWindow | null => {
    return BrowserWindow.fromWebContents(event.sender) || mainWindow || null;
  };

  // Window minimize
  ipcHandler.register(
    IPC_CHANNELS.WINDOW.MINIMIZE,
    async (event) => {
      const window = getCurrentWindow(event);
      if (window) {
        window.minimize();
      }
    },
    { rateLimit: { maxCalls: 10, windowMs: 60000 } }
  );

  // Window maximize/unmaximize
  ipcHandler.register(
    IPC_CHANNELS.WINDOW.MAXIMIZE,
    async (event) => {
      const window = getCurrentWindow(event);
      if (window) {
        if (window.isMaximized()) {
          window.unmaximize();
        } else {
          window.maximize();
        }
        
        // Send maximize changed event
        ipcHandler.send(window, IPC_CHANNELS.WINDOW.MAXIMIZE_CHANGED, {
          maximized: window.isMaximized(),
        });
      }
    },
    { rateLimit: { maxCalls: 10, windowMs: 60000 } }
  );

  // Window close
  ipcHandler.register(
    IPC_CHANNELS.WINDOW.CLOSE,
    async (event) => {
      const window = getCurrentWindow(event);
      if (window) {
        window.close();
      }
    },
    { rateLimit: { maxCalls: 5, windowMs: 60000 } }
  );

  // Check if window is maximized
  ipcHandler.register(
    IPC_CHANNELS.WINDOW.IS_MAXIMIZED,
    async (event) => {
      const window = getCurrentWindow(event);
      return window ? window.isMaximized() : false;
    }
  );

  // Set window title
  ipcHandler.register(
    IPC_CHANNELS.WINDOW.SET_TITLE,
    async (event, args) => {
      const validatedArgs = validate(schemas.titleRequest, args);
      const window = getCurrentWindow(event);
      if (window) {
        window.setTitle(validatedArgs.title);
      }
    },
    { rateLimit: { maxCalls: 20, windowMs: 60000 } }
  );

  // Open settings window
  ipcHandler.register(
    IPC_CHANNELS.WINDOW.OPEN_SETTINGS,
    async (event) => {
      // TODO: Implement settings window
      logger.info('Settings window requested');
    },
    { rateLimit: { maxCalls: 5, windowMs: 60000 } }
  );

  // Open about window
  ipcHandler.register(
    IPC_CHANNELS.WINDOW.OPEN_ABOUT,
    async (event) => {
      // TODO: Implement about window
      logger.info('About window requested');
    },
    { rateLimit: { maxCalls: 5, windowMs: 60000 } }
  );

  logger.info('Window IPC handlers setup complete');
}