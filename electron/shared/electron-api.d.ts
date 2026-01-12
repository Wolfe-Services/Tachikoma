// src/shared/electron-api.d.ts
import type { ElectronAPI } from '../preload/types';

declare global {
  interface Window {
    electronAPI?: ElectronAPI;
  }
}

export {};