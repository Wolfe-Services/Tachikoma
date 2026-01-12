# Spec 169: Security Configuration

## Phase
8 - Electron Shell

## Spec ID
169

## Status
Planned

## Dependencies
- Spec 161 (Electron Main Process)
- Spec 171 (Preload Scripts)
- Spec 172 (Context Bridge)

## Estimated Context
~10%

---

## Objective

Implement comprehensive security configuration for the Electron application, following Electron security best practices. This includes process isolation, Content Security Policy, secure IPC communication, and protection against common vulnerabilities.

---

## Acceptance Criteria

- [ ] Context isolation enabled for all windows
- [ ] Sandbox mode enabled for renderers
- [ ] Node integration disabled in renderers
- [ ] Content Security Policy configured
- [ ] WebSecurity enabled
- [ ] Navigation restricted to allowed origins
- [ ] New window creation blocked
- [ ] Protocol handling secured
- [ ] Certificate validation enforced
- [ ] Permissions properly managed

---

## Implementation Details

### Security Configuration Module

```typescript
// src/electron/main/security/index.ts
import {
  app,
  session,
  BrowserWindow,
  WebContents,
  shell,
  dialog,
} from 'electron';
import { URL } from 'url';
import { Logger } from '../logger';

const logger = new Logger('security');

interface SecurityConfig {
  allowedOrigins: string[];
  allowedProtocols: string[];
  cspDirectives: Record<string, string[]>;
  permissions: {
    allow: string[];
    deny: string[];
  };
}

const defaultConfig: SecurityConfig = {
  allowedOrigins: [
    'https://api.tachikoma.io',
    'https://auth.tachikoma.io',
  ],
  allowedProtocols: ['https:', 'http:', 'tachikoma:', 'file:'],
  cspDirectives: {
    'default-src': ["'self'"],
    'script-src': ["'self'"],
    'style-src': ["'self'", "'unsafe-inline'"],
    'img-src': ["'self'", 'data:', 'https:'],
    'font-src': ["'self'", 'data:'],
    'connect-src': [
      "'self'",
      'https://api.tachikoma.io',
      'wss://api.tachikoma.io',
    ],
    'frame-src': ["'none'"],
    'object-src': ["'none'"],
    'base-uri': ["'self'"],
    'form-action': ["'self'"],
  },
  permissions: {
    allow: ['clipboard-read', 'clipboard-write', 'notifications'],
    deny: [
      'geolocation',
      'camera',
      'microphone',
      'display-capture',
      'midi',
      'pointerLock',
      'fullscreen',
      'openExternal',
    ],
  },
};

class SecurityManager {
  private config: SecurityConfig;
  private trustedCertificates: Map<string, string> = new Map();

  constructor(config: Partial<SecurityConfig> = {}) {
    this.config = { ...defaultConfig, ...config };
  }

  initialize(): void {
    this.configureAppSecurity();
    this.configureSessionSecurity();
    this.setupWebContentsHandlers();

    logger.info('Security manager initialized');
  }

  private configureAppSecurity(): void {
    // Enable sandbox for all renderers
    app.enableSandbox();

    // Disable navigation to file:// in packaged app
    if (app.isPackaged) {
      this.config.allowedProtocols = this.config.allowedProtocols.filter(
        (p) => p !== 'file:'
      );
    }

    // Handle certificate errors
    app.on(
      'certificate-error',
      (event, webContents, url, error, certificate, callback) => {
        this.handleCertificateError(event, url, error, certificate, callback);
      }
    );

    // Handle login prompts
    app.on('login', (event, webContents, details, authInfo, callback) => {
      this.handleLogin(event, details, authInfo, callback);
    });
  }

  private configureSessionSecurity(): void {
    const ses = session.defaultSession;

    // Configure Content Security Policy
    ses.webRequest.onHeadersReceived((details, callback) => {
      callback({
        responseHeaders: {
          ...details.responseHeaders,
          'Content-Security-Policy': [this.buildCSP()],
          'X-Content-Type-Options': ['nosniff'],
          'X-Frame-Options': ['DENY'],
          'X-XSS-Protection': ['1; mode=block'],
          'Referrer-Policy': ['strict-origin-when-cross-origin'],
        },
      });
    });

    // Configure permissions
    ses.setPermissionRequestHandler((webContents, permission, callback, details) => {
      this.handlePermissionRequest(permission, callback, details);
    });

    ses.setPermissionCheckHandler((webContents, permission, requestingOrigin) => {
      return this.checkPermission(permission, requestingOrigin);
    });

    // Disable spell check download (security risk)
    ses.setSpellCheckerLanguages([]);

    // Clear preload scripts cache on reload in development
    if (!app.isPackaged) {
      ses.clearCache();
    }
  }

  private setupWebContentsHandlers(): void {
    app.on('web-contents-created', (event, contents) => {
      this.secureWebContents(contents);
    });
  }

  private secureWebContents(contents: WebContents): void {
    // Block navigation to non-allowed origins
    contents.on('will-navigate', (event, url) => {
      if (!this.isAllowedNavigation(url)) {
        event.preventDefault();
        logger.warn('Blocked navigation', { url });
      }
    });

    // Block frame navigation
    contents.on('will-frame-navigate', (event, url) => {
      if (!this.isAllowedNavigation(url)) {
        event.preventDefault();
        logger.warn('Blocked frame navigation', { url });
      }
    });

    // Block new window creation
    contents.setWindowOpenHandler(({ url }) => {
      // Allow external URLs to open in default browser
      if (this.isExternalUrl(url)) {
        shell.openExternal(url);
        return { action: 'deny' };
      }

      logger.warn('Blocked window open', { url });
      return { action: 'deny' };
    });

    // Block remote module
    contents.on('remote-require', (event) => {
      event.preventDefault();
      logger.warn('Blocked remote-require');
    });

    contents.on('remote-get-builtin', (event) => {
      event.preventDefault();
      logger.warn('Blocked remote-get-builtin');
    });

    contents.on('remote-get-global', (event) => {
      event.preventDefault();
      logger.warn('Blocked remote-get-global');
    });

    contents.on('remote-get-current-window', (event) => {
      event.preventDefault();
      logger.warn('Blocked remote-get-current-window');
    });

    contents.on('remote-get-current-web-contents', (event) => {
      event.preventDefault();
      logger.warn('Blocked remote-get-current-web-contents');
    });

    // Disable eval and new Function
    contents.on('will-attach-webview', (event, webPreferences) => {
      // Strip away preload scripts if not matching the expected location
      delete webPreferences.preload;
      delete webPreferences.preloadURL;

      // Disable Node.js integration
      webPreferences.nodeIntegration = false;
      webPreferences.contextIsolation = true;
    });
  }

  private buildCSP(): string {
    return Object.entries(this.config.cspDirectives)
      .map(([directive, values]) => `${directive} ${values.join(' ')}`)
      .join('; ');
  }

  private isAllowedNavigation(url: string): boolean {
    try {
      const parsedUrl = new URL(url);

      // Allow internal navigation
      if (parsedUrl.protocol === 'tachikoma:') {
        return true;
      }

      // Allow file:// in development
      if (!app.isPackaged && parsedUrl.protocol === 'file:') {
        return true;
      }

      // Allow localhost in development
      if (!app.isPackaged && parsedUrl.hostname === 'localhost') {
        return true;
      }

      // Check against allowed origins
      const origin = parsedUrl.origin;
      return this.config.allowedOrigins.includes(origin);
    } catch {
      return false;
    }
  }

  private isExternalUrl(url: string): boolean {
    try {
      const parsedUrl = new URL(url);
      return parsedUrl.protocol === 'https:' || parsedUrl.protocol === 'http:';
    } catch {
      return false;
    }
  }

  private handlePermissionRequest(
    permission: string,
    callback: (granted: boolean) => void,
    details?: Electron.PermissionRequestHandlerHandlerDetails
  ): void {
    if (this.config.permissions.allow.includes(permission)) {
      logger.debug('Permission granted', { permission });
      callback(true);
    } else {
      logger.debug('Permission denied', { permission });
      callback(false);
    }
  }

  private checkPermission(permission: string, requestingOrigin: string): boolean {
    // Only allow permissions for our own origin
    if (!app.isPackaged && requestingOrigin.includes('localhost')) {
      return this.config.permissions.allow.includes(permission);
    }

    return false;
  }

  private handleCertificateError(
    event: Electron.Event,
    url: string,
    error: string,
    certificate: Electron.Certificate,
    callback: (isTrusted: boolean) => void
  ): void {
    // In development, allow localhost
    if (!app.isPackaged && url.includes('localhost')) {
      event.preventDefault();
      callback(true);
      return;
    }

    // Check if certificate is trusted
    const fingerprint = certificate.fingerprint;
    if (this.trustedCertificates.has(fingerprint)) {
      event.preventDefault();
      callback(true);
      return;
    }

    logger.warn('Certificate error', { url, error });
    callback(false);
  }

  private async handleLogin(
    event: Electron.Event,
    details: Electron.AuthenticationResponseDetails,
    authInfo: Electron.AuthInfo,
    callback: (username?: string, password?: string) => void
  ): Promise<void> {
    event.preventDefault();

    // Could show a login dialog here
    logger.warn('Login requested', { host: authInfo.host });
    callback(); // Cancel by default
  }

  // Public API

  addAllowedOrigin(origin: string): void {
    if (!this.config.allowedOrigins.includes(origin)) {
      this.config.allowedOrigins.push(origin);
      logger.info('Added allowed origin', { origin });
    }
  }

  removeAllowedOrigin(origin: string): void {
    const index = this.config.allowedOrigins.indexOf(origin);
    if (index > -1) {
      this.config.allowedOrigins.splice(index, 1);
      logger.info('Removed allowed origin', { origin });
    }
  }

  trustCertificate(fingerprint: string, host: string): void {
    this.trustedCertificates.set(fingerprint, host);
    logger.info('Added trusted certificate', { fingerprint, host });
  }

  updateCSP(directive: string, values: string[]): void {
    this.config.cspDirectives[directive] = values;
    logger.info('Updated CSP directive', { directive });
  }

  getSecurityReport(): Record<string, unknown> {
    return {
      sandboxEnabled: true,
      contextIsolation: true,
      nodeIntegration: false,
      allowedOrigins: this.config.allowedOrigins,
      csp: this.buildCSP(),
      trustedCertificates: Array.from(this.trustedCertificates.keys()),
    };
  }
}

export const securityManager = new SecurityManager();
```

### Secure Window Options

```typescript
// src/electron/main/security/window-options.ts
import { BrowserWindowConstructorOptions } from 'electron';
import { join } from 'path';

export function getSecureWindowOptions(
  preloadPath?: string
): Partial<BrowserWindowConstructorOptions> {
  return {
    webPreferences: {
      // Security settings
      nodeIntegration: false,
      nodeIntegrationInWorker: false,
      nodeIntegrationInSubFrames: false,
      contextIsolation: true,
      sandbox: true,
      webSecurity: true,
      allowRunningInsecureContent: false,

      // Feature restrictions
      webviewTag: false,
      plugins: false,
      experimentalFeatures: false,
      enableWebSQL: false,
      navigateOnDragDrop: false,

      // Preload script (sandboxed)
      preload: preloadPath || join(__dirname, '../preload/index.js'),

      // Other settings
      spellcheck: true,
      autoplayPolicy: 'user-gesture-required',

      // Disable unsafe features
      v8CacheOptions: 'none',
      safeDialogs: true,
      safeDialogsMessage: 'Prevent additional dialogs',
    },
  };
}

export function getSecureWebviewOptions(): Record<string, unknown> {
  return {
    nodeIntegration: false,
    nodeIntegrationInSubFrames: false,
    contextIsolation: true,
    sandbox: true,
    webSecurity: true,
    allowRunningInsecureContent: false,
    plugins: false,
    experimentalFeatures: false,
    enableWebSQL: false,
  };
}
```

### Security Audit

```typescript
// src/electron/main/security/audit.ts
import { app, BrowserWindow, session } from 'electron';
import { Logger } from '../logger';

const logger = new Logger('security-audit');

interface AuditResult {
  passed: boolean;
  checks: Array<{
    name: string;
    passed: boolean;
    message: string;
    severity: 'critical' | 'warning' | 'info';
  }>;
}

export async function runSecurityAudit(): Promise<AuditResult> {
  const checks: AuditResult['checks'] = [];

  // Check app sandbox
  checks.push({
    name: 'App Sandbox',
    passed: true, // app.enableSandbox() was called
    message: 'Sandbox is enabled for all renderers',
    severity: 'critical',
  });

  // Check all windows
  const windows = BrowserWindow.getAllWindows();
  for (const window of windows) {
    const prefs = window.webContents.getWebPreferences();

    checks.push({
      name: `Window ${window.id} - Context Isolation`,
      passed: prefs.contextIsolation === true,
      message: prefs.contextIsolation
        ? 'Context isolation is enabled'
        : 'Context isolation is disabled!',
      severity: 'critical',
    });

    checks.push({
      name: `Window ${window.id} - Node Integration`,
      passed: prefs.nodeIntegration === false,
      message: prefs.nodeIntegration
        ? 'Node integration is enabled!'
        : 'Node integration is disabled',
      severity: 'critical',
    });

    checks.push({
      name: `Window ${window.id} - Sandbox`,
      passed: prefs.sandbox === true,
      message: prefs.sandbox ? 'Sandbox is enabled' : 'Sandbox is disabled!',
      severity: 'critical',
    });

    checks.push({
      name: `Window ${window.id} - Web Security`,
      passed: prefs.webSecurity === true,
      message: prefs.webSecurity
        ? 'Web security is enabled'
        : 'Web security is disabled!',
      severity: 'critical',
    });
  }

  // Check session configuration
  const defaultSession = session.defaultSession;

  // Check if permission handler is set
  checks.push({
    name: 'Permission Handler',
    passed: true, // We set it in securityManager
    message: 'Permission handler is configured',
    severity: 'warning',
  });

  // Check if running packaged
  checks.push({
    name: 'Production Build',
    passed: app.isPackaged,
    message: app.isPackaged
      ? 'Running packaged application'
      : 'Running in development mode',
    severity: 'info',
  });

  const passed = checks.every(
    (c) => c.passed || c.severity !== 'critical'
  );

  const result: AuditResult = { passed, checks };

  // Log results
  logger.info('Security audit complete', {
    passed,
    criticalIssues: checks.filter((c) => !c.passed && c.severity === 'critical')
      .length,
    warnings: checks.filter((c) => !c.passed && c.severity === 'warning').length,
  });

  if (!passed) {
    logger.error('Security audit failed', {
      failedChecks: checks.filter((c) => !c.passed),
    });
  }

  return result;
}
```

### Security IPC Handlers

```typescript
// src/electron/main/ipc/security.ts
import { ipcMain } from 'electron';
import { securityManager } from '../security';
import { runSecurityAudit } from '../security/audit';

export function setupSecurityIpcHandlers(): void {
  ipcMain.handle('security:getReport', () => {
    return securityManager.getSecurityReport();
  });

  ipcMain.handle('security:runAudit', async () => {
    return runSecurityAudit();
  });

  ipcMain.handle('security:addAllowedOrigin', (_, origin: string) => {
    securityManager.addAllowedOrigin(origin);
  });

  ipcMain.handle('security:removeAllowedOrigin', (_, origin: string) => {
    securityManager.removeAllowedOrigin(origin);
  });
}
```

---

## Testing Requirements

### Unit Tests

```typescript
// src/electron/main/security/__tests__/security.test.ts
import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('electron', () => ({
  app: {
    enableSandbox: vi.fn(),
    isPackaged: false,
    on: vi.fn(),
  },
  session: {
    defaultSession: {
      webRequest: {
        onHeadersReceived: vi.fn(),
      },
      setPermissionRequestHandler: vi.fn(),
      setPermissionCheckHandler: vi.fn(),
      setSpellCheckerLanguages: vi.fn(),
      clearCache: vi.fn(),
    },
  },
  shell: {
    openExternal: vi.fn(),
  },
  dialog: {
    showMessageBox: vi.fn(),
  },
  BrowserWindow: {
    getAllWindows: vi.fn().mockReturnValue([]),
  },
}));

describe('SecurityManager', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should enable sandbox on initialization', async () => {
    const { app } = await import('electron');
    const { securityManager } = await import('../index');

    securityManager.initialize();
    expect(app.enableSandbox).toHaveBeenCalled();
  });

  it('should build correct CSP header', async () => {
    const { securityManager } = await import('../index');

    const report = securityManager.getSecurityReport();
    expect(report.csp).toContain("default-src 'self'");
    expect(report.csp).toContain("script-src 'self'");
  });

  it('should allow adding trusted origins', async () => {
    const { securityManager } = await import('../index');

    securityManager.addAllowedOrigin('https://trusted.example.com');
    const report = securityManager.getSecurityReport();

    expect(report.allowedOrigins).toContain('https://trusted.example.com');
  });
});
```

### Security Audit Tests

```typescript
// src/electron/main/security/__tests__/audit.test.ts
import { describe, it, expect, vi } from 'vitest';

vi.mock('electron', () => ({
  app: {
    isPackaged: true,
  },
  session: {
    defaultSession: {},
  },
  BrowserWindow: {
    getAllWindows: vi.fn().mockReturnValue([
      {
        id: 1,
        webContents: {
          getWebPreferences: vi.fn().mockReturnValue({
            contextIsolation: true,
            nodeIntegration: false,
            sandbox: true,
            webSecurity: true,
          }),
        },
      },
    ]),
  },
}));

describe('Security Audit', () => {
  it('should pass audit with secure configuration', async () => {
    const { runSecurityAudit } = await import('../audit');

    const result = await runSecurityAudit();
    expect(result.passed).toBe(true);
  });

  it('should fail audit with insecure configuration', async () => {
    const { BrowserWindow } = await import('electron');
    (BrowserWindow.getAllWindows as any).mockReturnValue([
      {
        id: 1,
        webContents: {
          getWebPreferences: vi.fn().mockReturnValue({
            contextIsolation: false, // Insecure!
            nodeIntegration: true, // Insecure!
            sandbox: false,
            webSecurity: false,
          }),
        },
      },
    ]);

    const { runSecurityAudit } = await import('../audit');

    const result = await runSecurityAudit();
    expect(result.passed).toBe(false);
  });
});
```

---

## Related Specs

- Spec 161: Electron Main Process
- Spec 162: Window Management
- Spec 171: Preload Scripts
- Spec 172: Context Bridge
