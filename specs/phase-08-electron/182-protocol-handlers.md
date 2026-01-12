# Spec 182: Protocol Handlers

## Phase
8 - Electron Shell

## Spec ID
182

## Status
Planned

## Dependencies
- Spec 161 (Electron Main Process)
- Spec 169 (Security Configuration)

## Estimated Context
~8%

---

## Objective

Implement custom protocol handlers for the Tachikoma application, enabling secure handling of custom URL schemes for loading local resources, handling app-specific protocols, and intercepting network requests.

---

## Acceptance Criteria

- [x] Custom protocol registration (tachikoma://)
- [x] Secure resource loading via protocol
- [x] Asset serving from asar archive
- [x] Stream protocol for large files
- [x] Protocol interception and modification
- [x] CORS handling for custom protocols
- [x] Privileged protocol support
- [x] Protocol handler cleanup on quit

---

## Implementation Details

### Protocol Handler Manager

```typescript
// src/electron/main/protocol/index.ts
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
```

### Session Protocol Configuration

```typescript
// src/electron/main/protocol/session.ts
import { session } from 'electron';
import { Logger } from '../logger';

const logger = new Logger('protocol-session');

export function configureSessionProtocols(): void {
  const ses = session.defaultSession;

  // Configure WebRequest to handle custom protocols
  ses.webRequest.onBeforeRequest(
    { urls: ['tachikoma://*/*', 'tachikoma-asset://*/*'] },
    (details, callback) => {
      logger.debug('Custom protocol request', { url: details.url });
      callback({});
    }
  );

  // Set Content Security Policy for custom protocols
  ses.webRequest.onHeadersReceived((details, callback) => {
    const url = new URL(details.url);

    if (url.protocol === 'tachikoma:' || url.protocol === 'tachikoma-asset:') {
      callback({
        responseHeaders: {
          ...details.responseHeaders,
          'Content-Security-Policy': [
            "default-src 'self' tachikoma: tachikoma-asset:; " +
            "script-src 'self' 'unsafe-eval' tachikoma:; " +
            "style-src 'self' 'unsafe-inline' tachikoma:; " +
            "img-src 'self' data: https: tachikoma: tachikoma-asset:; " +
            "connect-src 'self' https: wss: tachikoma:;",
          ],
        },
      });
    } else {
      callback({});
    }
  });

  // Handle protocol errors
  ses.webRequest.onErrorOccurred(
    { urls: ['tachikoma://*/*', 'tachikoma-asset://*/*'] },
    (details) => {
      logger.error('Protocol error', {
        url: details.url,
        error: details.error,
      });
    }
  );
}
```

### Protocol URL Helpers

```typescript
// src/shared/protocol-urls.ts

export const PROTOCOL_SCHEMES = {
  APP: 'tachikoma',
  ASSET: 'tachikoma-asset',
} as const;

export function createAppUrl(path: string): string {
  const cleanPath = path.startsWith('/') ? path : `/${path}`;
  return `${PROTOCOL_SCHEMES.APP}://app${cleanPath}`;
}

export function createApiUrl(endpoint: string): string {
  const cleanEndpoint = endpoint.startsWith('/') ? endpoint : `/${endpoint}`;
  return `${PROTOCOL_SCHEMES.APP}://api${cleanEndpoint}`;
}

export function createResourceUrl(resourcePath: string): string {
  const cleanPath = resourcePath.startsWith('/') ? resourcePath : `/${resourcePath}`;
  return `${PROTOCOL_SCHEMES.APP}://resource${cleanPath}`;
}

export function createAssetUrl(filePath: string): string {
  // Encode the file path
  const encodedPath = encodeURIComponent(filePath);
  return `${PROTOCOL_SCHEMES.ASSET}://${encodedPath}`;
}

export function isProtocolUrl(url: string): boolean {
  return (
    url.startsWith(`${PROTOCOL_SCHEMES.APP}://`) ||
    url.startsWith(`${PROTOCOL_SCHEMES.ASSET}://`)
  );
}

export function parseProtocolUrl(url: string): {
  scheme: string;
  host: string;
  path: string;
  query: Record<string, string>;
} | null {
  try {
    const parsed = new URL(url);
    const query: Record<string, string> = {};
    parsed.searchParams.forEach((v, k) => (query[k] = v));

    return {
      scheme: parsed.protocol.replace(':', ''),
      host: parsed.hostname,
      path: parsed.pathname,
      query,
    };
  } catch {
    return null;
  }
}
```

---

## Testing Requirements

### Unit Tests

```typescript
// src/electron/main/protocol/__tests__/protocol.test.ts
import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('electron', () => ({
  protocol: {
    registerSchemesAsPrivileged: vi.fn(),
    handle: vi.fn(),
    interceptFileProtocol: vi.fn(),
    unhandle: vi.fn(),
  },
  app: {
    getPath: vi.fn().mockReturnValue('/mock/path'),
    getVersion: vi.fn().mockReturnValue('1.0.0'),
    isPackaged: false,
  },
  net: {
    fetch: vi.fn(),
  },
}));

describe('ProtocolManager', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should register protocol schemes', async () => {
    const { protocol } = await import('electron');
    const { protocolManager } = await import('../index');

    protocolManager.registerSchemes();

    expect(protocol.registerSchemesAsPrivileged).toHaveBeenCalled();
  });
});

describe('Protocol URL Helpers', () => {
  it('should create app URLs', async () => {
    const { createAppUrl } = await import('../../../shared/protocol-urls');

    expect(createAppUrl('/index.html')).toBe('tachikoma://app/index.html');
    expect(createAppUrl('styles/main.css')).toBe('tachikoma://app/styles/main.css');
  });

  it('should parse protocol URLs', async () => {
    const { parseProtocolUrl } = await import('../../../shared/protocol-urls');

    const result = parseProtocolUrl('tachikoma://app/index.html?foo=bar');

    expect(result).toEqual({
      scheme: 'tachikoma',
      host: 'app',
      path: '/index.html',
      query: { foo: 'bar' },
    });
  });
});
```

---

## Related Specs

- Spec 161: Electron Main Process
- Spec 169: Security Configuration
- Spec 181: Deep Linking
