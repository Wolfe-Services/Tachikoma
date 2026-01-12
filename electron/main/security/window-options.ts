import { BrowserWindowConstructorOptions } from 'electron';
import { join } from 'path';

export function getSecureWindowOptions(
  preloadPath?: string
): Partial<BrowserWindowConstructorOptions> {
  return {
    webPreferences: {
      // Security settings
      nodeIntegration: false,
      nodeIntegrationInWorker: false,
      nodeIntegrationInSubFrames: false,
      contextIsolation: true,
      sandbox: true,
      webSecurity: true,
      allowRunningInsecureContent: false,

      // Feature restrictions
      webviewTag: false,
      plugins: false,
      experimentalFeatures: false,
      enableWebSQL: false,
      navigateOnDragDrop: false,

      // Preload script (sandboxed)
      preload: preloadPath || join(__dirname, '../../preload/index.js'),

      // Other settings
      spellcheck: true,
      autoplayPolicy: 'user-gesture-required',

      // Disable unsafe features
      v8CacheOptions: 'none',
      safeDialogs: true,
      safeDialogsMessage: 'Prevent additional dialogs',
    },
  };
}

export function getSecureWebviewOptions(): Record<string, unknown> {
  return {
    nodeIntegration: false,
    nodeIntegrationInSubFrames: false,
    contextIsolation: true,
    sandbox: true,
    webSecurity: true,
    allowRunningInsecureContent: false,
    plugins: false,
    experimentalFeatures: false,
    enableWebSQL: false,
  };
}