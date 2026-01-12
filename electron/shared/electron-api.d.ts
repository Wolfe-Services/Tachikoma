// src/shared/electron-api.d.ts
import type { ElectronAPI } from '../preload/types';

declare global {
  interface Window {
    electronAPI?: ElectronAPI;
  }
}

export {};

// Type-safe API exposed via contextBridge
export type { ElectronAPI } from '../preload/types';