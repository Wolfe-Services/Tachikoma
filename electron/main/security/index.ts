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