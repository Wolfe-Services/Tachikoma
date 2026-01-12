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