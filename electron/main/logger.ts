import { app } from 'electron';
import { join } from 'path';
import { createWriteStream, WriteStream, mkdirSync, existsSync } from 'fs';

type LogLevel = 'debug' | 'info' | 'warn' | 'error';

interface LogEntry {
  timestamp: string;
  level: LogLevel;
  context: string;
  message: string;
  data?: Record<string, unknown>;
}

export class Logger {
  private static logStream: WriteStream | null = null;
  private static logLevel: LogLevel = 'info';
  private static readonly levels: Record<LogLevel, number> = {
    debug: 0,
    info: 1,
    warn: 2,
    error: 3,
  };

  private context: string;

  constructor(context: string) {
    this.context = context;

    if (!Logger.logStream) {
      Logger.initializeLogStream();
    }
  }

  private static initializeLogStream(): void {
    try {
      const logDir = join(app.getPath('userData'), 'logs');

      if (!existsSync(logDir)) {
        mkdirSync(logDir, { recursive: true });
      }

      const logFile = join(logDir, `tachikoma-${new Date().toISOString().split('T')[0]}.log`);
      Logger.logStream = createWriteStream(logFile, { flags: 'a' });

      // Set log level from environment
      const envLevel = process.env.LOG_LEVEL as LogLevel;
      if (envLevel && Logger.levels[envLevel] !== undefined) {
        Logger.logLevel = envLevel;
      } else if (process.env.NODE_ENV === 'development') {
        Logger.logLevel = 'debug';
      }

      // Handle stream errors
      Logger.logStream.on('error', (error) => {
        console.error('Log stream error:', error);
      });
    } catch (error) {
      console.error('Failed to initialize log stream:', error);
    }
  }

  private shouldLog(level: LogLevel): boolean {
    return Logger.levels[level] >= Logger.levels[Logger.logLevel];
  }

  private formatEntry(entry: LogEntry): string {
    const base = `[${entry.timestamp}] [${entry.level.toUpperCase()}] [${entry.context}] ${entry.message}`;
    if (entry.data) {
      return `${base} ${JSON.stringify(entry.data)}`;
    }
    return base;
  }

  private log(level: LogLevel, message: string, data?: Record<string, unknown>): void {
    if (!this.shouldLog(level)) return;

    const entry: LogEntry = {
      timestamp: new Date().toISOString(),
      level,
      context: this.context,
      message,
      data,
    };

    const formatted = this.formatEntry(entry);

    // Write to console
    switch (level) {
      case 'debug':
        console.debug(formatted);
        break;
      case 'info':
        console.info(formatted);
        break;
      case 'warn':
        console.warn(formatted);
        break;
      case 'error':
        console.error(formatted);
        break;
    }

    // Write to file
    try {
      Logger.logStream?.write(formatted + '\n');
    } catch (error) {
      console.error('Failed to write to log file:', error);
    }
  }

  debug(message: string, data?: Record<string, unknown>): void {
    this.log('debug', message, data);
  }

  info(message: string, data?: Record<string, unknown>): void {
    this.log('info', message, data);
  }

  warn(message: string, data?: Record<string, unknown>): void {
    this.log('warn', message, data);
  }

  error(message: string, data?: Record<string, unknown>): void {
    this.log('error', message, data);
  }

  static setLogLevel(level: LogLevel): void {
    Logger.logLevel = level;
  }

  static async flush(): Promise<void> {
    return new Promise((resolve) => {
      if (Logger.logStream) {
        Logger.logStream.end(resolve);
      } else {
        resolve();
      }
    });
  }
}