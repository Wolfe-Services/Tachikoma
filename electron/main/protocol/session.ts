import { session } from 'electron';
import { Logger } from '../logger';

const logger = new Logger('protocol-session');

export function configureSessionProtocols(): void {
  const ses = session.defaultSession;

  // Configure WebRequest to handle custom protocols
  // Note: URL patterns for custom schemes aren't supported, so we filter in callback
  ses.webRequest.onBeforeRequest((details, callback) => {
    if (details.url.startsWith('tachikoma:') || details.url.startsWith('tachikoma-asset:')) {
      logger.debug('Custom protocol request', { url: details.url });
    }
    callback({});
  });

  // Set Content Security Policy for custom protocols
  ses.webRequest.onHeadersReceived((details, callback) => {
    try {
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
    } catch {
      callback({});
    }
  });

  // Handle protocol errors
  ses.webRequest.onErrorOccurred((details) => {
    if (details.url.startsWith('tachikoma:') || details.url.startsWith('tachikoma-asset:')) {
      logger.error('Protocol error', {
        url: details.url,
        error: details.error,
      });
    }
  });
}