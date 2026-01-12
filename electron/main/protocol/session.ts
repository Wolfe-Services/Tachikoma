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