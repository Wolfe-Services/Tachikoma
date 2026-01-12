import {
  protocol,
  app,
  net,
  session,
  ProtocolResponse,
} from 'electron';
import { join, extname, normalize } from 'path';
import { existsSync, createReadStream, statSync } from 'fs';
import { URL } from 'url';
import { Logger } from '../logger';

const logger = new Logger('protocol');

interface ProtocolConfig {
  scheme: string;
  privileges: Electron.CustomScheme['privileges'];
}

const MIME_TYPES: Record<string, string> = {
  '.html': 'text/html',
  '.css': 'text/css',
  '.js': 'application/javascript',
  '.mjs': 'application/javascript',
  '.json': 'application/json',
  '.png': 'image/png',
  '.jpg': 'image/jpeg',
  '.jpeg': 'image/jpeg',
  '.gif': 'image/gif',
  '.svg': 'image/svg+xml',
  '.ico': 'image/x-icon',
  '.webp': 'image/webp',
  '.woff': 'font/woff',
  '.woff2': 'font/woff2',
  '.ttf': 'font/ttf',
  '.eot': 'application/vnd.ms-fontobject',
  '.mp3': 'audio/mpeg',
  '.mp4': 'video/mp4',
  '.webm': 'video/webm',
  '.pdf': 'application/pdf',
  '.zip': 'application/zip',
};

class ProtocolManager {
  private registeredSchemes: Set<string> = new Set();

  registerSchemes(): void {
    // Must be called before app is ready
    const schemes: ProtocolConfig[] = [
      {
        scheme: 'tachikoma',
        privileges: {
          standard: true,
          secure: true,
          supportFetchAPI: true,
          corsEnabled: true,
          stream: true,
        },
      },
      {
        scheme: 'tachikoma-asset',
        privileges: {
          standard: false,
          secure: true,
          supportFetchAPI: true,
          corsEnabled: false,
          stream: true,
        },
      },
    ];

    protocol.registerSchemesAsPrivileged(schemes);

    schemes.forEach((s) => this.registeredSchemes.add(s.scheme));
    logger.info('Protocol schemes registered', {
      schemes: schemes.map((s) => s.scheme),
    });
  }

  setupProtocols(): void {
    // Setup main tachikoma:// protocol
    this.setupTachikomaProtocol();

    // Setup asset protocol
    this.setupAssetProtocol();

    // Setup file protocol interception
    this.setupFileProtocolInterception();

    logger.info('Protocol handlers setup complete');
  }

  private setupTachikomaProtocol(): void {
    protocol.handle('tachikoma', async (request) => {
      const url = new URL(request.url);

      logger.debug('Tachikoma protocol request', {
        host: url.hostname,
        path: url.pathname,
      });

      // Route based on hostname
      switch (url.hostname) {
        case 'app':
          return this.handleAppRequest(url);
        case 'api':
          return this.handleApiRequest(url, request);
        case 'resource':
          return this.handleResourceRequest(url);
        default:
          return new Response('Not Found', { status: 404 });
      }
    });
  }

  private async handleAppRequest(url: URL): Promise<Response> {
    // Serve app files from the renderer directory
    const basePath = app.isPackaged
      ? join(process.resourcesPath, 'app.asar', 'dist', 'renderer')
      : join(__dirname, '../../renderer');

    let filePath = normalize(join(basePath, url.pathname));

    // Prevent path traversal
    if (!filePath.startsWith(basePath)) {
      return new Response('Forbidden', { status: 403 });
    }

    // Default to index.html for SPA routing
    if (!existsSync(filePath) || statSync(filePath).isDirectory()) {
      filePath = join(basePath, 'index.html');
    }

    if (!existsSync(filePath)) {
      return new Response('Not Found', { status: 404 });
    }

    const mimeType = MIME_TYPES[extname(filePath)] || 'application/octet-stream';

    return net.fetch(`file://${filePath}`, {
      headers: {
        'Content-Type': mimeType,
      },
    });
  }

  private async handleApiRequest(url: URL, request: Request): Promise<Response> {
    // Handle internal API requests
    const endpoint = url.pathname;

    switch (endpoint) {
      case '/version':
        return Response.json({
          version: app.getVersion(),
          electron: process.versions.electron,
          chrome: process.versions.chrome,
          node: process.versions.node,
        });

      case '/platform':
        return Response.json({
          platform: process.platform,
          arch: process.arch,
          isPackaged: app.isPackaged,
        });

      default:
        return new Response('Not Found', { status: 404 });
    }
  }

  private async handleResourceRequest(url: URL): Promise<Response> {
    // Serve resources from the resources directory
    const resourcesPath = app.isPackaged
      ? process.resourcesPath
      : join(__dirname, '../../resources');

    const filePath = normalize(join(resourcesPath, url.pathname));

    // Prevent path traversal
    if (!filePath.startsWith(resourcesPath)) {
      return new Response('Forbidden', { status: 403 });
    }

    if (!existsSync(filePath)) {
      return new Response('Not Found', { status: 404 });
    }

    const mimeType = MIME_TYPES[extname(filePath)] || 'application/octet-stream';

    return net.fetch(`file://${filePath}`, {
      headers: {
        'Content-Type': mimeType,
      },
    });
  }

  private setupAssetProtocol(): void {
    // Stream-based protocol for large files
    protocol.handle('tachikoma-asset', async (request) => {
      const url = new URL(request.url);
      const filePath = decodeURIComponent(url.pathname);

      if (!existsSync(filePath)) {
        return new Response('Not Found', { status: 404 });
      }

      const stats = statSync(filePath);
      const mimeType = MIME_TYPES[extname(filePath)] || 'application/octet-stream';

      // Handle range requests for streaming
      const rangeHeader = request.headers.get('range');

      if (rangeHeader) {
        return this.handleRangeRequest(filePath, rangeHeader, stats.size, mimeType);
      }

      // Full file response
      const stream = createReadStream(filePath);

      return new Response(stream as any, {
        headers: {
          'Content-Type': mimeType,
          'Content-Length': String(stats.size),
          'Accept-Ranges': 'bytes',
        },
      });
    });
  }

  private handleRangeRequest(
    filePath: string,
    rangeHeader: string,
    fileSize: number,
    mimeType: string
  ): Response {
    const parts = rangeHeader.replace(/bytes=/, '').split('-');
    const start = parseInt(parts[0], 10);
    const end = parts[1] ? parseInt(parts[1], 10) : fileSize - 1;

    if (start >= fileSize || end >= fileSize) {
      return new Response('Range Not Satisfiable', {
        status: 416,
        headers: {
          'Content-Range': `bytes */${fileSize}`,
        },
      });
    }

    const chunkSize = end - start + 1;
    const stream = createReadStream(filePath, { start, end });

    return new Response(stream as any, {
      status: 206,
      headers: {
        'Content-Type': mimeType,
        'Content-Length': String(chunkSize),
        'Content-Range': `bytes ${start}-${end}/${fileSize}`,
        'Accept-Ranges': 'bytes',
      },
    });
  }

  private setupFileProtocolInterception(): void {
    // Intercept file:// protocol for security
    protocol.interceptFileProtocol('file', (request, callback) => {
      const url = new URL(request.url);
      let filePath = decodeURIComponent(url.pathname);

      // On Windows, remove leading slash
      if (process.platform === 'win32' && filePath.startsWith('/')) {
        filePath = filePath.slice(1);
      }

      // Security: Only allow files in specific directories
      const allowedPaths = [
        app.getPath('userData'),
        app.getPath('temp'),
        app.isPackaged
          ? process.resourcesPath
          : join(__dirname, '../../'),
      ];

      const isAllowed = allowedPaths.some((allowed) =>
        normalize(filePath).startsWith(normalize(allowed))
      );

      if (!isAllowed) {
        logger.warn('Blocked file protocol request', { path: filePath });
        callback({ error: -10 }); // NET::ERR_ACCESS_DENIED
        return;
      }

      callback({ path: filePath });
    });
  }

  // Cleanup
  cleanup(): void {
    for (const scheme of this.registeredSchemes) {
      try {
        protocol.unhandle(scheme);
      } catch {
        // Scheme might not be registered
      }
    }
  }
}

export const protocolManager = new ProtocolManager();

// Must be called before app ready
export function registerProtocolSchemes(): void {
  protocolManager.registerSchemes();
}

// Must be called after app ready
export function setupProtocolHandlers(): void {
  protocolManager.setupProtocols();
}