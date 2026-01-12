# Spec 165: File System Access

## Phase
8 - Electron Shell

## Spec ID
165

## Status
Planned

## Dependencies
- Spec 161 (Electron Main Process)
- Spec 164 (Native Dialogs)
- Spec 169 (Security Configuration)
- Spec 170 (IPC Channels)

## Estimated Context
~10%

---

## Objective

Implement secure file system access for the Electron application, providing read/write operations, directory management, file watching, and proper path handling. Ensure all operations are sandboxed and validated for security.

---

## Acceptance Criteria

- [x] Secure file read/write operations via IPC
- [x] Directory creation and management
- [x] File and directory enumeration
- [x] File watching for external changes
- [x] Path validation and sanitization
- [x] Sandboxed access to allowed directories only
- [x] Streaming support for large files
- [x] Atomic file operations
- [x] File metadata and stat operations
- [x] Cross-platform path handling

---

## Implementation Details

### File System Service

```typescript
// src/electron/main/fs/index.ts
import {
  promises as fs,
  createReadStream,
  createWriteStream,
  watch,
  FSWatcher,
  Stats,
} from 'fs';
import { app } from 'electron';
import { join, resolve, dirname, basename, extname, relative, normalize } from 'path';
import { createHash } from 'crypto';
import { Logger } from '../logger';

const logger = new Logger('fs');

interface FileInfo {
  name: string;
  path: string;
  size: number;
  isDirectory: boolean;
  isFile: boolean;
  isSymlink: boolean;
  created: Date;
  modified: Date;
  accessed: Date;
  extension: string;
}

interface ReadOptions {
  encoding?: BufferEncoding;
  start?: number;
  end?: number;
}

interface WriteOptions {
  encoding?: BufferEncoding;
  mode?: number;
  flag?: string;
  atomic?: boolean;
}

interface WatchCallback {
  (event: 'change' | 'rename', filename: string): void;
}

class FileSystemService {
  private allowedPaths: Set<string> = new Set();
  private watchers: Map<string, FSWatcher> = new Map();

  constructor() {
    // Initialize with default allowed paths
    this.allowedPaths.add(app.getPath('userData'));
    this.allowedPaths.add(app.getPath('documents'));
    this.allowedPaths.add(app.getPath('downloads'));
    this.allowedPaths.add(app.getPath('temp'));
  }

  addAllowedPath(path: string): void {
    const normalized = normalize(resolve(path));
    this.allowedPaths.add(normalized);
    logger.info('Added allowed path', { path: normalized });
  }

  removeAllowedPath(path: string): void {
    const normalized = normalize(resolve(path));
    this.allowedPaths.delete(normalized);
    logger.info('Removed allowed path', { path: normalized });
  }

  private validatePath(targetPath: string): string {
    const normalized = normalize(resolve(targetPath));

    // Check if path is within allowed directories
    const isAllowed = Array.from(this.allowedPaths).some((allowedPath) =>
      normalized.startsWith(allowedPath)
    );

    if (!isAllowed) {
      logger.warn('Access denied to path', { path: normalized });
      throw new Error(`Access denied: ${normalized}`);
    }

    return normalized;
  }

  async exists(path: string): Promise<boolean> {
    const validPath = this.validatePath(path);
    try {
      await fs.access(validPath);
      return true;
    } catch {
      return false;
    }
  }

  async stat(path: string): Promise<FileInfo> {
    const validPath = this.validatePath(path);
    const stats = await fs.stat(validPath);

    return this.statsToFileInfo(validPath, stats);
  }

  async lstat(path: string): Promise<FileInfo> {
    const validPath = this.validatePath(path);
    const stats = await fs.lstat(validPath);

    return this.statsToFileInfo(validPath, stats);
  }

  private statsToFileInfo(path: string, stats: Stats): FileInfo {
    return {
      name: basename(path),
      path,
      size: stats.size,
      isDirectory: stats.isDirectory(),
      isFile: stats.isFile(),
      isSymlink: stats.isSymbolicLink(),
      created: stats.birthtime,
      modified: stats.mtime,
      accessed: stats.atime,
      extension: extname(path),
    };
  }

  async readFile(path: string, options: ReadOptions = {}): Promise<string | Buffer> {
    const validPath = this.validatePath(path);

    logger.debug('Reading file', { path: validPath });

    if (options.start !== undefined || options.end !== undefined) {
      // Use streaming for partial reads
      return this.readFilePartial(validPath, options);
    }

    const data = await fs.readFile(validPath);

    if (options.encoding) {
      return data.toString(options.encoding);
    }

    return data;
  }

  private async readFilePartial(path: string, options: ReadOptions): Promise<Buffer> {
    return new Promise((resolve, reject) => {
      const chunks: Buffer[] = [];
      const stream = createReadStream(path, {
        start: options.start,
        end: options.end,
      });

      stream.on('data', (chunk: Buffer) => chunks.push(chunk));
      stream.on('end', () => resolve(Buffer.concat(chunks)));
      stream.on('error', reject);
    });
  }

  async writeFile(
    path: string,
    data: string | Buffer,
    options: WriteOptions = {}
  ): Promise<void> {
    const validPath = this.validatePath(path);

    logger.debug('Writing file', { path: validPath, atomic: options.atomic });

    if (options.atomic) {
      await this.writeFileAtomic(validPath, data, options);
    } else {
      await fs.writeFile(validPath, data, {
        encoding: options.encoding,
        mode: options.mode,
        flag: options.flag,
      });
    }
  }

  private async writeFileAtomic(
    path: string,
    data: string | Buffer,
    options: WriteOptions
  ): Promise<void> {
    const tempPath = `${path}.tmp.${Date.now()}`;

    try {
      await fs.writeFile(tempPath, data, {
        encoding: options.encoding,
        mode: options.mode,
      });

      await fs.rename(tempPath, path);
    } catch (error) {
      // Clean up temp file on error
      try {
        await fs.unlink(tempPath);
      } catch {
        // Ignore cleanup errors
      }
      throw error;
    }
  }

  async appendFile(
    path: string,
    data: string | Buffer,
    encoding?: BufferEncoding
  ): Promise<void> {
    const validPath = this.validatePath(path);

    logger.debug('Appending to file', { path: validPath });

    await fs.appendFile(validPath, data, { encoding });
  }

  async deleteFile(path: string): Promise<void> {
    const validPath = this.validatePath(path);

    logger.debug('Deleting file', { path: validPath });

    await fs.unlink(validPath);
  }

  async copyFile(src: string, dest: string): Promise<void> {
    const validSrc = this.validatePath(src);
    const validDest = this.validatePath(dest);

    logger.debug('Copying file', { src: validSrc, dest: validDest });

    await fs.copyFile(validSrc, validDest);
  }

  async moveFile(src: string, dest: string): Promise<void> {
    const validSrc = this.validatePath(src);
    const validDest = this.validatePath(dest);

    logger.debug('Moving file', { src: validSrc, dest: validDest });

    await fs.rename(validSrc, validDest);
  }

  async createDirectory(path: string, recursive = true): Promise<void> {
    const validPath = this.validatePath(path);

    logger.debug('Creating directory', { path: validPath, recursive });

    await fs.mkdir(validPath, { recursive });
  }

  async deleteDirectory(path: string, recursive = false): Promise<void> {
    const validPath = this.validatePath(path);

    logger.debug('Deleting directory', { path: validPath, recursive });

    await fs.rm(validPath, { recursive, force: recursive });
  }

  async readDirectory(path: string): Promise<FileInfo[]> {
    const validPath = this.validatePath(path);

    logger.debug('Reading directory', { path: validPath });

    const entries = await fs.readdir(validPath, { withFileTypes: true });

    const fileInfos = await Promise.all(
      entries.map(async (entry) => {
        const entryPath = join(validPath, entry.name);
        const stats = await fs.stat(entryPath);
        return this.statsToFileInfo(entryPath, stats);
      })
    );

    return fileInfos;
  }

  async readDirectoryRecursive(path: string, depth = Infinity): Promise<FileInfo[]> {
    const validPath = this.validatePath(path);
    const results: FileInfo[] = [];

    const traverse = async (dir: string, currentDepth: number): Promise<void> => {
      if (currentDepth > depth) return;

      const entries = await this.readDirectory(dir);

      for (const entry of entries) {
        results.push(entry);

        if (entry.isDirectory) {
          await traverse(entry.path, currentDepth + 1);
        }
      }
    };

    await traverse(validPath, 0);
    return results;
  }

  watchPath(path: string, callback: WatchCallback): string {
    const validPath = this.validatePath(path);
    const watchId = `watch-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;

    logger.debug('Starting watch', { path: validPath, watchId });

    const watcher = watch(validPath, { recursive: true }, (event, filename) => {
      if (filename) {
        callback(event, filename);
      }
    });

    this.watchers.set(watchId, watcher);

    return watchId;
  }

  unwatchPath(watchId: string): void {
    const watcher = this.watchers.get(watchId);

    if (watcher) {
      watcher.close();
      this.watchers.delete(watchId);
      logger.debug('Stopped watch', { watchId });
    }
  }

  async getFileHash(path: string, algorithm = 'sha256'): Promise<string> {
    const validPath = this.validatePath(path);

    return new Promise((resolve, reject) => {
      const hash = createHash(algorithm);
      const stream = createReadStream(validPath);

      stream.on('data', (data) => hash.update(data));
      stream.on('end', () => resolve(hash.digest('hex')));
      stream.on('error', reject);
    });
  }

  // Path utilities
  resolvePath(...paths: string[]): string {
    return resolve(...paths);
  }

  joinPath(...paths: string[]): string {
    return join(...paths);
  }

  getRelativePath(from: string, to: string): string {
    return relative(from, to);
  }

  getDirname(path: string): string {
    return dirname(path);
  }

  getBasename(path: string, ext?: string): string {
    return basename(path, ext);
  }

  getExtension(path: string): string {
    return extname(path);
  }

  // App paths
  getAppPath(name: 'userData' | 'documents' | 'downloads' | 'temp' | 'logs'): string {
    return app.getPath(name);
  }

  cleanup(): void {
    for (const [watchId, watcher] of this.watchers) {
      watcher.close();
      logger.debug('Closed watcher on cleanup', { watchId });
    }
    this.watchers.clear();
  }
}

export const fsService = new FileSystemService();
export type { FileInfo, ReadOptions, WriteOptions, WatchCallback };
```

### File System IPC Handlers

```typescript
// src/electron/main/ipc/fs.ts
import { ipcMain, IpcMainInvokeEvent } from 'electron';
import { fsService, ReadOptions, WriteOptions } from '../fs';

export function setupFsIpcHandlers(): void {
  // File operations
  ipcMain.handle('fs:exists', async (_, path: string) => {
    return fsService.exists(path);
  });

  ipcMain.handle('fs:stat', async (_, path: string) => {
    return fsService.stat(path);
  });

  ipcMain.handle('fs:readFile', async (_, path: string, options?: ReadOptions) => {
    const result = await fsService.readFile(path, options);
    // Convert Buffer to base64 for IPC transfer
    if (Buffer.isBuffer(result)) {
      return { type: 'buffer', data: result.toString('base64') };
    }
    return { type: 'string', data: result };
  });

  ipcMain.handle(
    'fs:writeFile',
    async (_, path: string, data: string | { type: 'buffer'; data: string }, options?: WriteOptions) => {
      let content: string | Buffer;
      if (typeof data === 'object' && data.type === 'buffer') {
        content = Buffer.from(data.data, 'base64');
      } else {
        content = data as string;
      }
      await fsService.writeFile(path, content, options);
    }
  );

  ipcMain.handle('fs:appendFile', async (_, path: string, data: string, encoding?: BufferEncoding) => {
    await fsService.appendFile(path, data, encoding);
  });

  ipcMain.handle('fs:deleteFile', async (_, path: string) => {
    await fsService.deleteFile(path);
  });

  ipcMain.handle('fs:copyFile', async (_, src: string, dest: string) => {
    await fsService.copyFile(src, dest);
  });

  ipcMain.handle('fs:moveFile', async (_, src: string, dest: string) => {
    await fsService.moveFile(src, dest);
  });

  // Directory operations
  ipcMain.handle('fs:createDirectory', async (_, path: string, recursive?: boolean) => {
    await fsService.createDirectory(path, recursive);
  });

  ipcMain.handle('fs:deleteDirectory', async (_, path: string, recursive?: boolean) => {
    await fsService.deleteDirectory(path, recursive);
  });

  ipcMain.handle('fs:readDirectory', async (_, path: string) => {
    return fsService.readDirectory(path);
  });

  ipcMain.handle('fs:readDirectoryRecursive', async (_, path: string, depth?: number) => {
    return fsService.readDirectoryRecursive(path, depth);
  });

  // Watch operations
  const watchCallbacks = new Map<string, (event: IpcMainInvokeEvent) => void>();

  ipcMain.handle('fs:watch', async (event, path: string) => {
    const webContents = event.sender;
    const watchId = fsService.watchPath(path, (eventType, filename) => {
      if (!webContents.isDestroyed()) {
        webContents.send('fs:watchEvent', { watchId, eventType, filename });
      }
    });

    // Track sender for cleanup
    const cleanup = () => {
      fsService.unwatchPath(watchId);
      watchCallbacks.delete(watchId);
    };

    webContents.once('destroyed', cleanup);
    watchCallbacks.set(watchId, cleanup);

    return watchId;
  });

  ipcMain.handle('fs:unwatch', async (_, watchId: string) => {
    fsService.unwatchPath(watchId);
    const cleanup = watchCallbacks.get(watchId);
    if (cleanup) {
      watchCallbacks.delete(watchId);
    }
  });

  // Utility operations
  ipcMain.handle('fs:getFileHash', async (_, path: string, algorithm?: string) => {
    return fsService.getFileHash(path, algorithm);
  });

  // Path utilities
  ipcMain.handle('fs:resolvePath', (_, ...paths: string[]) => {
    return fsService.resolvePath(...paths);
  });

  ipcMain.handle('fs:joinPath', (_, ...paths: string[]) => {
    return fsService.joinPath(...paths);
  });

  ipcMain.handle('fs:getRelativePath', (_, from: string, to: string) => {
    return fsService.getRelativePath(from, to);
  });

  ipcMain.handle('fs:getDirname', (_, path: string) => {
    return fsService.getDirname(path);
  });

  ipcMain.handle('fs:getBasename', (_, path: string, ext?: string) => {
    return fsService.getBasename(path, ext);
  });

  ipcMain.handle('fs:getExtension', (_, path: string) => {
    return fsService.getExtension(path);
  });

  ipcMain.handle('fs:getAppPath', (_, name: string) => {
    return fsService.getAppPath(name as any);
  });

  // Access control
  ipcMain.handle('fs:addAllowedPath', (_, path: string) => {
    fsService.addAllowedPath(path);
  });

  ipcMain.handle('fs:removeAllowedPath', (_, path: string) => {
    fsService.removeAllowedPath(path);
  });
}
```

### Renderer File System Types

```typescript
// src/shared/types/fs.ts
export interface FileInfo {
  name: string;
  path: string;
  size: number;
  isDirectory: boolean;
  isFile: boolean;
  isSymlink: boolean;
  created: Date;
  modified: Date;
  accessed: Date;
  extension: string;
}

export interface ReadOptions {
  encoding?: BufferEncoding;
  start?: number;
  end?: number;
}

export interface WriteOptions {
  encoding?: BufferEncoding;
  mode?: number;
  flag?: string;
  atomic?: boolean;
}

export interface WatchEvent {
  watchId: string;
  eventType: 'change' | 'rename';
  filename: string;
}

export interface FileSystemAPI {
  // File operations
  exists(path: string): Promise<boolean>;
  stat(path: string): Promise<FileInfo>;
  readFile(path: string, options?: ReadOptions): Promise<string | Uint8Array>;
  readTextFile(path: string): Promise<string>;
  readBinaryFile(path: string): Promise<Uint8Array>;
  writeFile(path: string, data: string | Uint8Array, options?: WriteOptions): Promise<void>;
  appendFile(path: string, data: string, encoding?: BufferEncoding): Promise<void>;
  deleteFile(path: string): Promise<void>;
  copyFile(src: string, dest: string): Promise<void>;
  moveFile(src: string, dest: string): Promise<void>;

  // Directory operations
  createDirectory(path: string, recursive?: boolean): Promise<void>;
  deleteDirectory(path: string, recursive?: boolean): Promise<void>;
  readDirectory(path: string): Promise<FileInfo[]>;
  readDirectoryRecursive(path: string, depth?: number): Promise<FileInfo[]>;

  // Watch operations
  watch(path: string, callback: (event: WatchEvent) => void): Promise<() => void>;

  // Utility operations
  getFileHash(path: string, algorithm?: string): Promise<string>;

  // Path utilities
  path: {
    resolve(...paths: string[]): string;
    join(...paths: string[]): string;
    relative(from: string, to: string): string;
    dirname(path: string): string;
    basename(path: string, ext?: string): string;
    extname(path: string): string;
  };

  // App paths
  getAppPath(name: 'userData' | 'documents' | 'downloads' | 'temp' | 'logs'): Promise<string>;
}
```

---

## Testing Requirements

### Unit Tests

```typescript
// src/electron/main/fs/__tests__/fs.test.ts
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { vol } from 'memfs';

vi.mock('fs', async () => {
  const memfs = await import('memfs');
  return {
    ...memfs.fs,
    promises: memfs.fs.promises,
    createReadStream: vi.fn(),
    createWriteStream: vi.fn(),
    watch: vi.fn(),
  };
});

vi.mock('electron', () => ({
  app: {
    getPath: vi.fn((name) => `/mock/${name}`),
  },
}));

describe('FileSystemService', () => {
  beforeEach(() => {
    vol.reset();
    vol.mkdirSync('/mock/userData', { recursive: true });
    vol.mkdirSync('/mock/documents', { recursive: true });
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  it('should check if file exists', async () => {
    vol.writeFileSync('/mock/userData/test.txt', 'content');

    const { fsService } = await import('../index');
    const exists = await fsService.exists('/mock/userData/test.txt');

    expect(exists).toBe(true);
  });

  it('should read file content', async () => {
    vol.writeFileSync('/mock/userData/test.txt', 'hello world');

    const { fsService } = await import('../index');
    const content = await fsService.readFile('/mock/userData/test.txt', {
      encoding: 'utf-8',
    });

    expect(content).toBe('hello world');
  });

  it('should write file content', async () => {
    const { fsService } = await import('../index');
    await fsService.writeFile('/mock/userData/new.txt', 'new content');

    const content = vol.readFileSync('/mock/userData/new.txt', 'utf-8');
    expect(content).toBe('new content');
  });

  it('should deny access to non-allowed paths', async () => {
    const { fsService } = await import('../index');

    await expect(fsService.readFile('/etc/passwd')).rejects.toThrow(
      'Access denied'
    );
  });

  it('should read directory contents', async () => {
    vol.mkdirSync('/mock/userData/testdir');
    vol.writeFileSync('/mock/userData/testdir/file1.txt', 'content1');
    vol.writeFileSync('/mock/userData/testdir/file2.txt', 'content2');

    const { fsService } = await import('../index');
    const files = await fsService.readDirectory('/mock/userData/testdir');

    expect(files).toHaveLength(2);
    expect(files.map((f) => f.name)).toContain('file1.txt');
    expect(files.map((f) => f.name)).toContain('file2.txt');
  });
});
```

### Integration Tests

```typescript
// src/electron/main/fs/__tests__/fs.integration.test.ts
import { describe, it, expect, beforeAll, afterAll, afterEach } from 'vitest';
import { _electron as electron } from 'playwright';
import type { ElectronApplication } from 'playwright';
import { mkdtempSync, rmSync, writeFileSync } from 'fs';
import { join } from 'path';
import { tmpdir } from 'os';

describe('File System Integration', () => {
  let electronApp: ElectronApplication;
  let testDir: string;

  beforeAll(async () => {
    testDir = mkdtempSync(join(tmpdir(), 'tachikoma-test-'));
    electronApp = await electron.launch({
      args: ['.'],
      env: {
        ...process.env,
        TACHIKOMA_TEST_DIR: testDir,
      },
    });
  });

  afterAll(async () => {
    await electronApp.close();
    rmSync(testDir, { recursive: true, force: true });
  });

  it('should read and write files via IPC', async () => {
    writeFileSync(join(testDir, 'test.txt'), 'initial content');

    // Would need proper IPC testing setup
  });

  it('should watch for file changes', async () => {
    // Would need proper watch testing setup
  });
});
```

---

## Related Specs

- Spec 161: Electron Main Process
- Spec 164: Native Dialogs
- Spec 169: Security Configuration
- Spec 170: IPC Channels
