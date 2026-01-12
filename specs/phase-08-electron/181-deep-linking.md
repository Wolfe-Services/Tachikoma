# Spec 181: Deep Linking

## Phase
8 - Electron Shell

## Spec ID
181

## Status
Planned

## Dependencies
- Spec 161 (Electron Main Process)
- Spec 166 (App Lifecycle)

## Estimated Context
~8%

---

## Objective

Implement deep linking support for the Tachikoma application, allowing external applications and web pages to open specific content or trigger actions within the app using custom URL schemes.

---

## Acceptance Criteria

- [x] Custom URL scheme (tachikoma://) registration
- [x] Deep link parsing and routing
- [x] Single instance handling of deep links
- [x] OAuth callback support
- [x] File opening via URLs
- [x] Action triggers via URLs
- [x] Cross-platform deep link handling
- [x] Security validation of deep links
- [x] Deep link history tracking

---

## Implementation Details

### Deep Link Manager

```typescript
// src/electron/main/deep-links/index.ts
import { app, BrowserWindow } from 'electron';
import { URL } from 'url';
import { Logger } from '../logger';

const logger = new Logger('deep-links');

interface DeepLinkRoute {
  pattern: RegExp;
  handler: (params: Record<string, string>, url: URL) => void | Promise<void>;
}

interface ParsedDeepLink {
  scheme: string;
  host: string;
  path: string;
  params: Record<string, string>;
  hash: string;
  raw: string;
}

class DeepLinkManager {
  private routes: DeepLinkRoute[] = [];
  private mainWindow: BrowserWindow | null = null;
  private pendingLinks: string[] = [];
  private readonly scheme = 'tachikoma';

  constructor() {
    this.setupProtocolHandler();
  }

  private setupProtocolHandler(): void {
    // Register as default protocol handler
    if (process.defaultApp) {
      if (process.argv.length >= 2) {
        app.setAsDefaultProtocolClient(this.scheme, process.execPath, [
          '-r',
          './.electron-vite/build/electron/main/index.js',
        ]);
      }
    } else {
      app.setAsDefaultProtocolClient(this.scheme);
    }

    // Handle protocol on macOS
    app.on('open-url', (event, url) => {
      event.preventDefault();
      this.handleDeepLink(url);
    });

    // Handle protocol on Windows/Linux (via command line args)
    const gotTheLock = app.requestSingleInstanceLock();

    if (!gotTheLock) {
      app.quit();
      return;
    }

    app.on('second-instance', (event, commandLine) => {
      // Find deep link in command line arguments
      const url = commandLine.find((arg) => arg.startsWith(`${this.scheme}://`));
      if (url) {
        this.handleDeepLink(url);
      }

      // Focus the main window
      if (this.mainWindow) {
        if (this.mainWindow.isMinimized()) {
          this.mainWindow.restore();
        }
        this.mainWindow.focus();
      }
    });

    // Check if app was launched from a deep link
    const launchUrl = this.getLaunchUrl();
    if (launchUrl) {
      this.pendingLinks.push(launchUrl);
    }
  }

  private getLaunchUrl(): string | null {
    // On macOS, we get it from open-url event
    // On Windows/Linux, check command line args
    if (process.platform !== 'darwin') {
      const url = process.argv.find((arg) => arg.startsWith(`${this.scheme}://`));
      return url || null;
    }
    return null;
  }

  setMainWindow(window: BrowserWindow): void {
    this.mainWindow = window;

    // Process any pending deep links
    while (this.pendingLinks.length > 0) {
      const url = this.pendingLinks.shift()!;
      this.handleDeepLink(url);
    }
  }

  registerRoute(pattern: string | RegExp, handler: DeepLinkRoute['handler']): void {
    const regex = typeof pattern === 'string' ? new RegExp(`^${pattern}$`) : pattern;

    this.routes.push({
      pattern: regex,
      handler,
    });

    logger.debug('Registered deep link route', { pattern: pattern.toString() });
  }

  handleDeepLink(url: string): void {
    logger.info('Handling deep link', { url });

    // Validate URL
    if (!this.isValidDeepLink(url)) {
      logger.warn('Invalid deep link', { url });
      return;
    }

    // Parse the URL
    const parsed = this.parseDeepLink(url);
    if (!parsed) {
      logger.error('Failed to parse deep link', { url });
      return;
    }

    // If main window not ready, queue the link
    if (!this.mainWindow) {
      logger.debug('Main window not ready, queueing deep link');
      this.pendingLinks.push(url);
      return;
    }

    // Find matching route
    const fullPath = `${parsed.host}${parsed.path}`;

    for (const route of this.routes) {
      const match = fullPath.match(route.pattern);
      if (match) {
        const params = { ...parsed.params };

        // Add named capture groups to params
        if (match.groups) {
          Object.assign(params, match.groups);
        }

        // Execute handler
        try {
          route.handler(params, new URL(url));
        } catch (error) {
          logger.error('Deep link handler error', { error, url });
        }
        return;
      }
    }

    // No matching route - send to renderer
    this.mainWindow.webContents.send('deep-link', { url, parsed });
    logger.warn('No route matched for deep link', { url });
  }

  private isValidDeepLink(url: string): boolean {
    try {
      const parsed = new URL(url);

      // Check scheme
      if (parsed.protocol !== `${this.scheme}:`) {
        return false;
      }

      // Validate host (prevent malicious URLs)
      const allowedHosts = ['open', 'auth', 'action', 'project', 'file'];
      if (!allowedHosts.includes(parsed.hostname)) {
        return false;
      }

      return true;
    } catch {
      return false;
    }
  }

  private parseDeepLink(url: string): ParsedDeepLink | null {
    try {
      const parsed = new URL(url);

      // Parse query parameters
      const params: Record<string, string> = {};
      parsed.searchParams.forEach((value, key) => {
        params[key] = value;
      });

      return {
        scheme: parsed.protocol.replace(':', ''),
        host: parsed.hostname,
        path: parsed.pathname,
        params,
        hash: parsed.hash,
        raw: url,
      };
    } catch {
      return null;
    }
  }

  // Helper to create deep link URLs
  createDeepLink(
    host: string,
    path: string = '',
    params: Record<string, string> = {}
  ): string {
    const url = new URL(`${this.scheme}://${host}${path}`);

    Object.entries(params).forEach(([key, value]) => {
      url.searchParams.set(key, value);
    });

    return url.toString();
  }
}

export const deepLinkManager = new DeepLinkManager();
```

### Default Route Handlers

```typescript
// src/electron/main/deep-links/routes.ts
import { deepLinkManager } from './index';
import { windowManager } from '../window';
import { dialogService } from '../dialogs';
import { fsService } from '../fs';
import { shell } from 'electron';
import { Logger } from '../logger';

const logger = new Logger('deep-link-routes');

export function setupDeepLinkRoutes(): void {
  // Open project
  // tachikoma://open/project?path=/path/to/project
  deepLinkManager.registerRoute('open/project', async (params) => {
    const { path } = params;

    if (!path) {
      logger.warn('Missing path parameter for open/project');
      return;
    }

    const exists = await fsService.exists(path);
    if (!exists) {
      await dialogService.error('Project Not Found', `The project at "${path}" does not exist.`);
      return;
    }

    const mainWindow = windowManager.getMainWindow();
    mainWindow?.webContents.send('project:open', { path });
  });

  // Open file
  // tachikoma://open/file?path=/path/to/file
  deepLinkManager.registerRoute('open/file', async (params) => {
    const { path } = params;

    if (!path) {
      logger.warn('Missing path parameter for open/file');
      return;
    }

    const exists = await fsService.exists(path);
    if (!exists) {
      await dialogService.error('File Not Found', `The file at "${path}" does not exist.`);
      return;
    }

    const mainWindow = windowManager.getMainWindow();
    mainWindow?.webContents.send('file:open', { path });
  });

  // OAuth callback
  // tachikoma://auth/callback?code=xxx&state=xxx
  deepLinkManager.registerRoute('auth/callback', (params) => {
    const { code, state, error } = params;

    const mainWindow = windowManager.getMainWindow();

    if (error) {
      mainWindow?.webContents.send('auth:error', { error });
      return;
    }

    if (!code) {
      mainWindow?.webContents.send('auth:error', { error: 'Missing authorization code' });
      return;
    }

    mainWindow?.webContents.send('auth:callback', { code, state });
  });

  // Action trigger
  // tachikoma://action/new-project
  // tachikoma://action/open-settings
  deepLinkManager.registerRoute('action/(?<action>[a-z-]+)', (params) => {
    const { action } = params;

    const mainWindow = windowManager.getMainWindow();

    switch (action) {
      case 'new-project':
        mainWindow?.webContents.send('project:new');
        break;
      case 'open-settings':
        mainWindow?.webContents.send('settings:open');
        break;
      case 'check-updates':
        mainWindow?.webContents.send('updates:check');
        break;
      default:
        logger.warn('Unknown action', { action });
    }
  });

  // Install extension/plugin
  // tachikoma://install?url=https://example.com/plugin.zip
  deepLinkManager.registerRoute('install', async (params, url) => {
    const { url: installUrl, name } = params;

    if (!installUrl) {
      logger.warn('Missing install URL');
      return;
    }

    // Validate the install URL
    if (!installUrl.startsWith('https://')) {
      await dialogService.error('Invalid URL', 'Extension URLs must use HTTPS.');
      return;
    }

    const confirmed = await dialogService.confirm(
      'Install Extension',
      `Do you want to install the extension "${name || 'Unknown'}" from ${installUrl}?`
    );

    if (confirmed) {
      const mainWindow = windowManager.getMainWindow();
      mainWindow?.webContents.send('extension:install', { url: installUrl, name });
    }
  });

  logger.info('Deep link routes registered');
}
```

### Deep Link IPC Handlers

```typescript
// src/electron/main/ipc/deep-links.ts
import { ipcMain } from 'electron';
import { deepLinkManager } from '../deep-links';

export function setupDeepLinkIpcHandlers(): void {
  ipcMain.handle('deepLink:create', (_, host: string, path: string, params: Record<string, string>) => {
    return deepLinkManager.createDeepLink(host, path, params);
  });

  ipcMain.handle('deepLink:handle', (_, url: string) => {
    deepLinkManager.handleDeepLink(url);
  });
}
```

### Renderer Deep Link Hook

```typescript
// src/renderer/hooks/useDeepLink.ts
import { useEffect, useCallback } from 'react';

interface DeepLinkData {
  url: string;
  parsed: {
    scheme: string;
    host: string;
    path: string;
    params: Record<string, string>;
  };
}

type DeepLinkHandler = (data: DeepLinkData) => void;

export function useDeepLink(handler: DeepLinkHandler): void {
  const handleDeepLink = useCallback(
    (_: unknown, data: DeepLinkData) => {
      handler(data);
    },
    [handler]
  );

  useEffect(() => {
    const cleanup = window.electronAPI?.onDeepLink?.((url: string) => {
      // Parse the URL in renderer
      try {
        const parsed = new URL(url);
        const params: Record<string, string> = {};
        parsed.searchParams.forEach((v, k) => (params[k] = v));

        handler({
          url,
          parsed: {
            scheme: parsed.protocol.replace(':', ''),
            host: parsed.hostname,
            path: parsed.pathname,
            params,
          },
        });
      } catch (error) {
        console.error('Failed to parse deep link:', error);
      }
    });

    return cleanup;
  }, [handler]);
}

// Helper to create deep links
export function createDeepLink(
  host: string,
  path: string = '',
  params: Record<string, string> = {}
): Promise<string> {
  return window.electronAPI?.invoke('deepLink:create', host, path, params) ||
    Promise.resolve(`tachikoma://${host}${path}`);
}
```

---

## Testing Requirements

### Unit Tests

```typescript
// src/electron/main/deep-links/__tests__/deep-links.test.ts
import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('electron', () => ({
  app: {
    setAsDefaultProtocolClient: vi.fn(),
    requestSingleInstanceLock: vi.fn().mockReturnValue(true),
    on: vi.fn(),
  },
  BrowserWindow: vi.fn(),
}));

describe('DeepLinkManager', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should parse deep links correctly', async () => {
    const { deepLinkManager } = await import('../index');

    const url = 'tachikoma://open/project?path=/test/path&name=test';
    const parsed = (deepLinkManager as any).parseDeepLink(url);

    expect(parsed).toEqual({
      scheme: 'tachikoma',
      host: 'open',
      path: '/project',
      params: { path: '/test/path', name: 'test' },
      hash: '',
      raw: url,
    });
  });

  it('should validate deep links', async () => {
    const { deepLinkManager } = await import('../index');

    expect((deepLinkManager as any).isValidDeepLink('tachikoma://open/file')).toBe(true);
    expect((deepLinkManager as any).isValidDeepLink('tachikoma://auth/callback')).toBe(true);
    expect((deepLinkManager as any).isValidDeepLink('http://example.com')).toBe(false);
    expect((deepLinkManager as any).isValidDeepLink('tachikoma://malicious/path')).toBe(false);
  });

  it('should create deep links', async () => {
    const { deepLinkManager } = await import('../index');

    const url = deepLinkManager.createDeepLink('open', '/project', { path: '/test' });
    expect(url).toBe('tachikoma://open/project?path=%2Ftest');
  });
});
```

---

## Related Specs

- Spec 161: Electron Main Process
- Spec 166: App Lifecycle
- Spec 182: Protocol Handlers
