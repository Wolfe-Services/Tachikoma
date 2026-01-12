import { ipcMain, BrowserWindow } from 'electron';
import { Logger } from '../logger';

const logger = new Logger('ipc-debug');

interface IPCMessage {
  id: string;
  timestamp: number;
  direction: 'in' | 'out';
  channel: string;
  data?: unknown;
  duration?: number;
  error?: string;
  sender?: {
    processId: number;
    frameId: number;
  };
}

class IPCDebugger {
  private messages: IPCMessage[] = [];
  private maxMessages = 1000;
  private subscribers: Set<BrowserWindow> = new Set();
  private isEnabled = false;
  private messageIdCounter = 0;

  enable(): void {
    if (this.isEnabled) return;

    this.isEnabled = true;
    this.setupInterceptors();
    logger.info('IPC debugging enabled');
  }

  disable(): void {
    if (!this.isEnabled) return;

    this.isEnabled = false;
    // Note: Cannot remove interceptors once set in Electron
    logger.info('IPC debugging disabled');
  }

  subscribe(window: BrowserWindow): void {
    this.subscribers.add(window);

    window.on('closed', () => {
      this.subscribers.delete(window);
    });
  }

  unsubscribe(window: BrowserWindow): void {
    this.subscribers.delete(window);
  }

  private setupInterceptors(): void {
    // Intercept main process handle calls
    const originalHandle = ipcMain.handle.bind(ipcMain);
    ipcMain.handle = (channel: string, listener: (...args: any[]) => any) => {
      const wrappedListener = async (event: any, ...args: any[]) => {
        const startTime = Date.now();
        const messageId = this.generateMessageId();

        const incomingMessage: IPCMessage = {
          id: messageId,
          timestamp: startTime,
          direction: 'in',
          channel,
          data: args,
          sender: {
            processId: event.processId,
            frameId: event.frameId,
          },
        };

        this.recordMessage(incomingMessage);

        try {
          const result = await listener(event, ...args);
          const endTime = Date.now();

          const outgoingMessage: IPCMessage = {
            id: messageId + '_response',
            timestamp: endTime,
            direction: 'out',
            channel: `${channel}:response`,
            data: result,
            duration: endTime - startTime,
          };

          this.recordMessage(outgoingMessage);
          return result;
        } catch (error) {
          const endTime = Date.now();

          const errorMessage: IPCMessage = {
            id: messageId + '_error',
            timestamp: endTime,
            direction: 'out',
            channel: `${channel}:error`,
            error: error instanceof Error ? error.message : String(error),
            duration: endTime - startTime,
          };

          this.recordMessage(errorMessage);
          throw error;
        }
      };

      return originalHandle(channel, wrappedListener);
    };

    // Intercept main process on calls
    const originalOn = ipcMain.on.bind(ipcMain);
    ipcMain.on = (channel: string, listener: (...args: any[]) => void) => {
      const wrappedListener = (event: any, ...args: any[]) => {
        const messageId = this.generateMessageId();

        const message: IPCMessage = {
          id: messageId,
          timestamp: Date.now(),
          direction: 'in',
          channel,
          data: args,
          sender: {
            processId: event.processId,
            frameId: event.frameId,
          },
        };

        this.recordMessage(message);
        return listener(event, ...args);
      };

      return originalOn(channel, wrappedListener);
    };
  }

  private generateMessageId(): string {
    return `ipc_${++this.messageIdCounter}_${Date.now()}`;
  }

  private recordMessage(message: IPCMessage): void {
    if (!this.isEnabled) return;

    this.messages.push(message);
    if (this.messages.length > this.maxMessages) {
      this.messages.shift();
    }

    // Broadcast to subscribers
    for (const window of this.subscribers) {
      if (!window.isDestroyed()) {
        window.webContents.send('ipc:debug', message);
      }
    }

    logger.debug('IPC message recorded', {
      channel: message.channel,
      direction: message.direction,
      duration: message.duration,
    });
  }

  getMessages(count?: number): IPCMessage[] {
    if (count) {
      return this.messages.slice(-count);
    }
    return [...this.messages];
  }

  clearMessages(): void {
    this.messages = [];
    logger.debug('IPC debug messages cleared');
  }

  getChannelStats(): Record<string, { count: number; avgDuration: number }> {
    const stats: Record<string, { count: number; totalDuration: number }> = {};

    for (const message of this.messages) {
      if (!stats[message.channel]) {
        stats[message.channel] = { count: 0, totalDuration: 0 };
      }
      
      stats[message.channel].count++;
      if (message.duration) {
        stats[message.channel].totalDuration += message.duration;
      }
    }

    const result: Record<string, { count: number; avgDuration: number }> = {};
    for (const [channel, data] of Object.entries(stats)) {
      result[channel] = {
        count: data.count,
        avgDuration: data.count > 0 ? data.totalDuration / data.count : 0,
      };
    }

    return result;
  }
}

export const ipcDebugger = new IPCDebugger();