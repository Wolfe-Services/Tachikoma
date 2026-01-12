import { isTauri } from '@utils/environment';
import { ipc } from './client';
import type { IpcChannels } from './types';

export interface InvokeOptions {
  timeout?: number;
  retries?: number;
  retryDelay?: number;
  cache?: boolean;
  cacheTime?: number;
}

export interface InvokeError {
  code: string;
  message: string;
  details?: Record<string, unknown>;
}

const DEFAULT_OPTIONS: InvokeOptions = {
  timeout: 30000,
  retries: 0,
  retryDelay: 1000,
  cache: false,
  cacheTime: 60000
};

// Simple in-memory cache
const cache = new Map<string, { data: unknown; timestamp: number }>();

function getCacheKey(command: string, args?: Record<string, unknown>): string {
  return `${command}:${JSON.stringify(args || {})}`;
}

export async function invoke<K extends keyof IpcChannels>(
  command: K,
  args: IpcChannels[K]['request'],
  options: InvokeOptions = {}
): Promise<IpcChannels[K]['response']> {
  const opts = { ...DEFAULT_OPTIONS, ...options };

  // Check cache first
  if (opts.cache) {
    const cacheKey = getCacheKey(command as string, args as Record<string, unknown>);
    const cached = cache.get(cacheKey);

    if (cached && Date.now() - cached.timestamp < opts.cacheTime!) {
      return cached.data as IpcChannels[K]['response'];
    }
  }

  // Execute with retries
  let lastError: Error | null = null;

  for (let attempt = 0; attempt <= opts.retries!; attempt++) {
    try {
      const result = await executeInvoke(command, args, opts.timeout!);

      // Cache successful result
      if (opts.cache) {
        cache.set(getCacheKey(command as string, args as Record<string, unknown>), {
          data: result,
          timestamp: Date.now()
        });
      }

      return result;
    } catch (error) {
      lastError = error as Error;

      // Don't retry on certain errors
      if (isNonRetryableError(error)) {
        throw transformError(error);
      }

      // Wait before retry
      if (attempt < opts.retries!) {
        await new Promise(resolve => setTimeout(resolve, opts.retryDelay!));
      }
    }
  }

  throw transformError(lastError);
}

async function executeInvoke<K extends keyof IpcChannels>(
  command: K,
  args: IpcChannels[K]['request'],
  timeout: number
): Promise<IpcChannels[K]['response']> {
  if (!isTauri()) {
    throw new Error('Tachikoma IPC is not available');
  }

  // Create timeout promise
  const timeoutPromise = new Promise<never>((_, reject) => {
    setTimeout(() => {
      reject(new Error(`IPC timeout after ${timeout}ms`));
    }, timeout);
  });

  // Race between invoke and timeout
  return Promise.race([
    ipc.invoke(command, args),
    timeoutPromise
  ]);
}

function isNonRetryableError(error: unknown): boolean {
  if (error instanceof Error) {
    const nonRetryable = [
      'NotFound',
      'Unauthorized',
      'Forbidden',
      'ValidationError'
    ];
    return nonRetryable.some(code => error.message.includes(code));
  }
  return false;
}

function transformError(error: unknown): InvokeError {
  if (error instanceof Error) {
    // Parse error format
    const match = error.message.match(/^(\w+):\s*(.+)$/);
    if (match) {
      return {
        code: match[1],
        message: match[2]
      };
    }
    return {
      code: 'UnknownError',
      message: error.message
    };
  }
  return {
    code: 'UnknownError',
    message: String(error)
  };
}

// Invalidate cache for specific command
export function invalidateCache(command?: string): void {
  if (command) {
    for (const key of cache.keys()) {
      if (key.startsWith(`${command}:`)) {
        cache.delete(key);
      }
    }
  } else {
    cache.clear();
  }
}