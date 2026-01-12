// src/electron/main/ipc/handler.ts
import { ipcMain, IpcMainInvokeEvent, BrowserWindow } from 'electron';
import { Logger } from '../logger';
import { IPC_CHANNELS } from '../../shared/ipc/channels';
import type { IPCRequestMap, IPCResponseMap, IPCEventMap } from '../../shared/ipc/types';

const logger = new Logger('ipc');

interface HandlerConfig {
  timeout?: number;
  rateLimit?: {
    maxCalls: number;
    windowMs: number;
  };
  validate?: boolean;
}

const rateLimitMap = new Map<string, Map<number, number[]>>();

class IPCHandler {
  private handlers = new Map<string, Function>();

  register<K extends keyof IPCRequestMap>(
    channel: K,
    handler: (event: IpcMainInvokeEvent, args: IPCRequestMap[K]) => Promise<IPCResponseMap[K]> | IPCResponseMap[K],
    config: HandlerConfig = {}
  ): void {
    if (this.handlers.has(channel as string)) {
      logger.warn('Handler already registered, overwriting', { channel });
    }

    const wrappedHandler = async (
      event: IpcMainInvokeEvent,
      args: IPCRequestMap[K]
    ): Promise<IPCResponseMap[K]> => {
      const startTime = Date.now();
      const senderId = event.sender.id;

      try {
        // Rate limiting
        if (config.rateLimit) {
          this.checkRateLimit(channel as string, senderId, config.rateLimit);
        }

        logger.debug('IPC invoke', { channel, senderId, args });

        // Execute with timeout
        const result = config.timeout
          ? await this.withTimeout(handler(event, args), config.timeout)
          : await handler(event, args);

        logger.debug('IPC response', {
          channel,
          duration: Date.now() - startTime,
          success: true,
        });

        return result;
      } catch (error) {
        logger.error('IPC error', {
          channel,
          error: error instanceof Error ? error.message : String(error),
          duration: Date.now() - startTime,
        });
        throw error;
      }
    };

    ipcMain.handle(channel as string, wrappedHandler);
    this.handlers.set(channel as string, wrappedHandler);
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
        setTimeout(() => reject(new Error(`IPC timeout after ${ms}ms`)), ms)
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

  send<K extends keyof IPCEventMap>(window: BrowserWindow, channel: K, data: IPCEventMap[K]): void {
    if (!window.isDestroyed()) {
      window.webContents.send(channel as string, data);
      logger.debug('IPC send', { channel, windowId: window.id });
    }
  }

  broadcast<K extends keyof IPCEventMap>(channel: K, data: IPCEventMap[K]): void {
    const windows = BrowserWindow.getAllWindows();
    for (const window of windows) {
      this.send(window, channel, data);
    }
    logger.debug('IPC broadcast', { channel, windowCount: windows.length });
  }

  // Serialization helpers
  serialize(data: unknown): string {
    try {
      return JSON.stringify(data, (key, value) => {
        // Handle special cases like Date, Error, etc.
        if (value instanceof Error) {
          return {
            __type: 'Error',
            name: value.name,
            message: value.message,
            stack: value.stack,
          };
        }
        if (value instanceof Date) {
          return {
            __type: 'Date',
            value: value.toISOString(),
          };
        }
        return value;
      });
    } catch (error) {
      logger.error('Serialization error', { error });
      throw new Error('Failed to serialize IPC data');
    }
  }

  deserialize<T = unknown>(data: string): T {
    try {
      return JSON.parse(data, (key, value) => {
        if (value && typeof value === 'object' && value.__type) {
          switch (value.__type) {
            case 'Error':
              const error = new Error(value.message);
              error.name = value.name;
              error.stack = value.stack;
              return error;
            case 'Date':
              return new Date(value.value);
          }
        }
        return value;
      });
    } catch (error) {
      logger.error('Deserialization error', { error });
      throw new Error('Failed to deserialize IPC data');
    }
  }

  // Get handler statistics
  getStats(): Record<string, { calls: number; errors: number; avgDuration: number }> {
    const stats: Record<string, { calls: number; errors: number; avgDuration: number }> = {};
    
    // This would need to be implemented with proper tracking
    // For now, return empty stats
    return stats;
  }
}

export const ipcHandler = new IPCHandler();