import { BrowserWindow, webContents } from 'electron';
import { Logger } from '../logger';

const logger = new Logger('console-aggregator');

interface ConsoleMessage {
  id: string;
  timestamp: number;
  level: 'log' | 'info' | 'warn' | 'error' | 'debug' | 'trace';
  message: string;
  source: 'main' | 'renderer' | 'preload';
  processId?: number;
  frameId?: number;
  location?: {
    file: string;
    line: number;
    column: number;
  };
  stackTrace?: string[];
}

class ConsoleAggregator {
  private messages: ConsoleMessage[] = [];
  private maxMessages = 2000;
  private subscribers: Set<BrowserWindow> = new Set();
  private messageIdCounter = 0;
  private isEnabled = false;

  enable(): void {
    if (this.isEnabled) return;

    this.isEnabled = true;
    this.setupInterceptors();
    logger.info('Console aggregation enabled');
  }

  disable(): void {
    if (!this.isEnabled) return;

    this.isEnabled = false;
    logger.info('Console aggregation disabled');
  }

  subscribe(window: BrowserWindow): void {
    this.subscribers.add(window);

    window.on('closed', () => {
      this.subscribers.delete(window);
    });

    // Listen to console messages from this window
    window.webContents.on('console-message', (event, level, message, line, sourceId) => {
      if (!this.isEnabled) return;

      this.recordMessage({
        id: this.generateMessageId(),
        timestamp: Date.now(),
        level: this.mapLogLevel(level),
        message,
        source: 'renderer',
        processId: window.webContents.getProcessId(),
        frameId: event.frameId,
        location: sourceId ? {
          file: sourceId,
          line,
          column: 0,
        } : undefined,
      });
    });
  }

  unsubscribe(window: BrowserWindow): void {
    this.subscribers.delete(window);
  }

  private setupInterceptors(): void {
    // Intercept main process console methods
    const originalConsole = {
      log: console.log.bind(console),
      info: console.info.bind(console),
      warn: console.warn.bind(console),
      error: console.error.bind(console),
      debug: console.debug.bind(console),
      trace: console.trace.bind(console),
    };

    // Wrap console methods
    console.log = (...args: any[]) => {
      originalConsole.log(...args);
      this.recordMainProcessMessage('log', args);
    };

    console.info = (...args: any[]) => {
      originalConsole.info(...args);
      this.recordMainProcessMessage('info', args);
    };

    console.warn = (...args: any[]) => {
      originalConsole.warn(...args);
      this.recordMainProcessMessage('warn', args);
    };

    console.error = (...args: any[]) => {
      originalConsole.error(...args);
      this.recordMainProcessMessage('error', args);
    };

    console.debug = (...args: any[]) => {
      originalConsole.debug(...args);
      this.recordMainProcessMessage('debug', args);
    };

    console.trace = (...args: any[]) => {
      originalConsole.trace(...args);
      this.recordMainProcessMessage('trace', args);
    };
  }

  private mapLogLevel(electronLevel: number): ConsoleMessage['level'] {
    switch (electronLevel) {
      case 0: return 'info';
      case 1: return 'warn';
      case 2: return 'error';
      case 3: return 'debug';
      default: return 'log';
    }
  }

  private recordMainProcessMessage(level: ConsoleMessage['level'], args: any[]): void {
    if (!this.isEnabled) return;

    // Format message from arguments
    const message = args.map(arg => {
      if (typeof arg === 'string') return arg;
      if (arg instanceof Error) return `${arg.name}: ${arg.message}`;
      try {
        return JSON.stringify(arg, null, 2);
      } catch {
        return String(arg);
      }
    }).join(' ');

    // Get stack trace for error context
    let stackTrace: string[] | undefined;
    if (level === 'error' || level === 'trace') {
      const stack = new Error().stack;
      if (stack) {
        stackTrace = stack.split('\n').slice(2); // Remove Error and current function
      }
    }

    this.recordMessage({
      id: this.generateMessageId(),
      timestamp: Date.now(),
      level,
      message,
      source: 'main',
      stackTrace,
    });
  }

  private generateMessageId(): string {
    return `console_${++this.messageIdCounter}_${Date.now()}`;
  }

  private recordMessage(message: ConsoleMessage): void {
    if (!this.isEnabled) return;

    this.messages.push(message);
    if (this.messages.length > this.maxMessages) {
      this.messages.shift();
    }

    // Broadcast to subscribers
    for (const window of this.subscribers) {
      if (!window.isDestroyed()) {
        window.webContents.send('console:message', message);
      }
    }

    logger.debug('Console message recorded', {
      level: message.level,
      source: message.source,
      message: message.message.substring(0, 100),
    });
  }

  getMessages(count?: number): ConsoleMessage[] {
    if (count) {
      return this.messages.slice(-count);
    }
    return [...this.messages];
  }

  getMessagesByLevel(level: ConsoleMessage['level']): ConsoleMessage[] {
    return this.messages.filter(m => m.level === level);
  }

  getMessagesBySource(source: ConsoleMessage['source']): ConsoleMessage[] {
    return this.messages.filter(m => m.source === source);
  }

  getErrorMessages(): ConsoleMessage[] {
    return this.messages.filter(m => m.level === 'error');
  }

  getWarningMessages(): ConsoleMessage[] {
    return this.messages.filter(m => m.level === 'warn');
  }

  searchMessages(query: string): ConsoleMessage[] {
    const lowerQuery = query.toLowerCase();
    return this.messages.filter(m => 
      m.message.toLowerCase().includes(lowerQuery) ||
      m.location?.file.toLowerCase().includes(lowerQuery)
    );
  }

  clearMessages(): void {
    this.messages = [];
    logger.debug('Console messages cleared');
  }

  getStats(): {
    totalMessages: number;
    byLevel: Record<ConsoleMessage['level'], number>;
    bySource: Record<ConsoleMessage['source'], number>;
    errorCount: number;
    warningCount: number;
  } {
    const byLevel: Record<ConsoleMessage['level'], number> = {
      log: 0,
      info: 0,
      warn: 0,
      error: 0,
      debug: 0,
      trace: 0,
    };

    const bySource: Record<ConsoleMessage['source'], number> = {
      main: 0,
      renderer: 0,
      preload: 0,
    };

    for (const message of this.messages) {
      byLevel[message.level]++;
      bySource[message.source]++;
    }

    return {
      totalMessages: this.messages.length,
      byLevel,
      bySource,
      errorCount: byLevel.error,
      warningCount: byLevel.warn,
    };
  }

  exportLogs(): string {
    let output = '# Console Log Export\n\n';
    output += `Generated at: ${new Date().toISOString()}\n`;
    output += `Total messages: ${this.messages.length}\n\n`;

    const stats = this.getStats();
    output += '## Statistics\n\n';
    output += `- Errors: ${stats.errorCount}\n`;
    output += `- Warnings: ${stats.warningCount}\n`;
    output += `- Main process: ${stats.bySource.main}\n`;
    output += `- Renderer process: ${stats.bySource.renderer}\n`;
    output += `- Preload script: ${stats.bySource.preload}\n\n`;

    output += '## Messages\n\n';
    for (const message of this.messages) {
      const timestamp = new Date(message.timestamp).toISOString();
      const location = message.location 
        ? ` (${message.location.file}:${message.location.line})` 
        : '';
      
      output += `**${timestamp}** [${message.level.toUpperCase()}] [${message.source}]${location}\n`;
      output += `${message.message}\n`;
      
      if (message.stackTrace) {
        output += '```\n';
        output += message.stackTrace.join('\n');
        output += '\n```\n';
      }
      
      output += '\n---\n\n';
    }

    return output;
  }

  // Create custom log method for DevTools internal logging
  log(level: ConsoleMessage['level'], message: string, location?: ConsoleMessage['location']): void {
    this.recordMessage({
      id: this.generateMessageId(),
      timestamp: Date.now(),
      level,
      message: `[DevTools] ${message}`,
      source: 'main',
      location,
    });
  }
}

export const consoleAggregator = new ConsoleAggregator();