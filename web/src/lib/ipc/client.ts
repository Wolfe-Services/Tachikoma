import type { IpcChannels, IpcEvents } from './types';
import { handleIpcError, isIpcAvailable } from './errors';

class TachikomaIpc {
  private listeners = new Map<string, Set<Function>>();

  async invoke<K extends keyof IpcChannels>(
    channel: K,
    request: IpcChannels[K]['request']
  ): Promise<IpcChannels[K]['response']> {
    if (!isIpcAvailable()) {
      throw new Error('Tachikoma IPC not available');
    }
    
    try {
      return window.tachikoma.invoke(channel, request) as Promise<IpcChannels[K]['response']>;
    } catch (error) {
      throw handleIpcError(channel, error);
    }
  }

  on<K extends keyof IpcEvents>(
    channel: K,
    callback: (data: IpcEvents[K]) => void
  ): () => void {
    if (!isIpcAvailable()) {
      return () => {};
    }

    if (!this.listeners.has(channel)) {
      this.listeners.set(channel, new Set());
    }
    this.listeners.get(channel)!.add(callback);

    window.tachikoma.on(channel, callback as any);

    // Return unsubscribe function
    return () => {
      this.listeners.get(channel)?.delete(callback);
      window.tachikoma.off(channel, callback as any);
    };
  }
}

export const ipc = new TachikomaIpc();