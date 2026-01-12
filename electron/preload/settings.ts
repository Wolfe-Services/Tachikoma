// src/electron/preload/settings.ts
import { contextBridge, ipcRenderer } from 'electron';
import type { SettingsAPI } from './types';

// Helper function for safe IPC invocation with error handling
async function safeInvoke<T>(channel: string, ...args: unknown[]): Promise<T> {
  try {
    return await ipcRenderer.invoke(channel, ...args);
  } catch (error) {
    console.error(`Settings IPC invoke failed for channel ${channel}:`, error);
    throw error;
  }
}

// Settings-specific API
const settingsAPI: SettingsAPI = {
  getConfig: () => safeInvoke('settings:getConfig'),
  setConfig: (key, value) => safeInvoke('settings:setConfig', { key, value }),
  resetConfig: () => safeInvoke('settings:resetConfig'),
  getThemes: () => safeInvoke('settings:getThemes'),
  setTheme: (theme) => safeInvoke('settings:setTheme', { theme }),
  getLanguages: () => safeInvoke('settings:getLanguages'),
  setLanguage: (code) => safeInvoke('settings:setLanguage', { code }),
  
  // Additional settings operations
  exportConfig: () => safeInvoke('settings:exportConfig'),
  importConfig: (config) => safeInvoke('settings:importConfig', config),
  validateConfig: (config) => safeInvoke('settings:validateConfig', config),
  
  // Settings categories
  getCategories: () => safeInvoke('settings:getCategories'),
  getCategorySettings: (category) => safeInvoke('settings:getCategorySettings', { category }),
  
  // Watch for settings changes
  onSettingChanged: (callback: (key: string, value: unknown, oldValue: unknown) => void) => {
    const handler = (_event: any, data: { key: string; value: unknown; oldValue: unknown }) => {
      callback(data.key, data.value, data.oldValue);
    };
    ipcRenderer.on('settings:changed', handler);
    
    // Return cleanup function
    return () => {
      ipcRenderer.removeListener('settings:changed', handler);
    };
  },
  
  // Keybinding management
  getKeybindings: () => safeInvoke('settings:getKeybindings'),
  setKeybinding: (action, keys) => safeInvoke('settings:setKeybinding', { action, keys }),
  resetKeybindings: () => safeInvoke('settings:resetKeybindings'),
  
  // Plugin settings
  getPluginSettings: (pluginId) => safeInvoke('settings:getPluginSettings', { pluginId }),
  setPluginSetting: (pluginId, key, value) => 
    safeInvoke('settings:setPluginSetting', { pluginId, key, value }),
};

// Expose settings API to renderer
contextBridge.exposeInMainWorld('settingsAPI', settingsAPI);

// Handle errors in settings preload
process.on('uncaughtException', (error) => {
  console.error('Settings preload uncaught exception:', error);
  ipcRenderer.send('crash-reporter:exception', {
    error: {
      name: error.name,
      message: error.message,
      stack: error.stack,
      context: 'settings-preload'
    }
  });
});

process.on('unhandledRejection', (reason) => {
  console.error('Settings preload unhandled rejection:', reason);
  ipcRenderer.send('crash-reporter:rejection', {
    reason: {
      name: 'UnhandledRejection',
      message: String(reason),
      stack: reason instanceof Error ? reason.stack : undefined,
      context: 'settings-preload'
    }
  });
});

export type { SettingsAPI };