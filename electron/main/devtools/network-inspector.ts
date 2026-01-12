import { BrowserWindow, session } from 'electron';
import { Logger } from '../logger';

const logger = new Logger('network-inspector');

interface NetworkRequest {
  id: string;
  timestamp: number;
  method: string;
  url: string;
  headers: Record<string, string>;
  status?: number;
  statusText?: string;
  responseHeaders?: Record<string, string>;
  size: {
    request: number;
    response: number;
  };
  timing: {
    start: number;
    end?: number;
    duration?: number;
  };
  error?: string;
}

class NetworkInspector {
  private isEnabled = false;
  private requests: NetworkRequest[] = [];
  private maxRequests = 1000;
  private subscribers: Set<BrowserWindow> = new Set();
  private requestIdCounter = 0;
  private pendingRequests = new Map<string, NetworkRequest>();

  enable(): void {
    if (this.isEnabled) return;

    this.isEnabled = true;
    this.setupInterceptors();
    logger.info('Network inspection enabled');
  }

  disable(): void {
    if (!this.isEnabled) return;

    this.isEnabled = false;
    // Note: Cannot remove interceptors once set in Electron
    logger.info('Network inspection disabled');
  }

  subscribe(window: BrowserWindow): void {
    this.subscribers.add(window);

    window.on('closed', () => {
      this.subscribers.delete(window);
    });
  }

  unsubscribe(window: BrowserWindow): void {
    this.subscribers.delete(window);
  }

  private setupInterceptors(): void {
    const ses = session.defaultSession;

    // Intercept requests before they are sent
    ses.webRequest.onBeforeRequest((details, callback) => {
      if (!this.isEnabled) {
        callback({});
        return;
      }

      const requestId = this.generateRequestId();
      const request: NetworkRequest = {
        id: requestId,
        timestamp: Date.now(),
        method: details.method,
        url: details.url,
        headers: this.parseHeaders(details.requestHeaders || {}),
        size: {
          request: details.uploadData?.reduce((sum, data) => sum + data.bytes.length, 0) || 0,
          response: 0,
        },
        timing: {
          start: Date.now(),
        },
      };

      this.pendingRequests.set(details.id.toString(), request);
      callback({});
    });

    // Intercept request headers
    ses.webRequest.onBeforeSendHeaders((details) => {
      if (!this.isEnabled) return;

      const request = this.pendingRequests.get(details.id.toString());
      if (request) {
        request.headers = this.parseHeaders(details.requestHeaders || {});
      }
    });

    // Intercept response headers
    ses.webRequest.onHeadersReceived((details) => {
      if (!this.isEnabled) return;

      const request = this.pendingRequests.get(details.id.toString());
      if (request) {
        request.status = details.statusCode;
        request.statusText = details.statusLine;
        request.responseHeaders = this.parseHeaders(details.responseHeaders || {});
      }
    });

    // Intercept completed requests
    ses.webRequest.onCompleted((details) => {
      if (!this.isEnabled) return;

      const request = this.pendingRequests.get(details.id.toString());
      if (request) {
        request.timing.end = Date.now();
        request.timing.duration = request.timing.end - request.timing.start;
        request.size.response = details.responseHeaders?.['content-length']
          ? parseInt(details.responseHeaders['content-length'][0], 10)
          : 0;

        this.recordRequest(request);
        this.pendingRequests.delete(details.id.toString());
      }
    });

    // Intercept failed requests
    ses.webRequest.onErrorOccurred((details) => {
      if (!this.isEnabled) return;

      const request = this.pendingRequests.get(details.id.toString());
      if (request) {
        request.timing.end = Date.now();
        request.timing.duration = request.timing.end - request.timing.start;
        request.error = details.error;

        this.recordRequest(request);
        this.pendingRequests.delete(details.id.toString());
      }
    });
  }

  private parseHeaders(headers: Record<string, string[]>): Record<string, string> {
    const parsed: Record<string, string> = {};
    for (const [key, values] of Object.entries(headers)) {
      parsed[key] = values.join(', ');
    }
    return parsed;
  }

  private generateRequestId(): string {
    return `req_${++this.requestIdCounter}_${Date.now()}`;
  }

  private recordRequest(request: NetworkRequest): void {
    if (!this.isEnabled) return;

    this.requests.push(request);
    if (this.requests.length > this.maxRequests) {
      this.requests.shift();
    }

    // Broadcast to subscribers
    for (const window of this.subscribers) {
      if (!window.isDestroyed()) {
        window.webContents.send('network:request', request);
      }
    }

    logger.debug('Network request recorded', {
      method: request.method,
      url: request.url,
      status: request.status,
      duration: request.timing.duration,
    });
  }

  getRequests(count?: number): NetworkRequest[] {
    if (count) {
      return this.requests.slice(-count);
    }
    return [...this.requests];
  }

  getRequestById(id: string): NetworkRequest | null {
    return this.requests.find(r => r.id === id) || null;
  }

  getRequestsByDomain(domain: string): NetworkRequest[] {
    return this.requests.filter(r => {
      try {
        const url = new URL(r.url);
        return url.hostname.includes(domain);
      } catch {
        return false;
      }
    });
  }

  getRequestsByStatus(status: number): NetworkRequest[] {
    return this.requests.filter(r => r.status === status);
  }

  getFailedRequests(): NetworkRequest[] {
    return this.requests.filter(r => r.error || (r.status && r.status >= 400));
  }

  clearRequests(): void {
    this.requests = [];
    this.pendingRequests.clear();
    logger.debug('Network requests cleared');
  }

  getNetworkStats(): {
    totalRequests: number;
    successfulRequests: number;
    failedRequests: number;
    averageResponseTime: number;
    totalDataTransferred: number;
  } {
    const totalRequests = this.requests.length;
    const successfulRequests = this.requests.filter(r => !r.error && r.status && r.status < 400).length;
    const failedRequests = totalRequests - successfulRequests;
    
    const responseTimes = this.requests
      .filter(r => r.timing.duration)
      .map(r => r.timing.duration!);
    const averageResponseTime = responseTimes.length > 0 
      ? responseTimes.reduce((sum, time) => sum + time, 0) / responseTimes.length 
      : 0;

    const totalDataTransferred = this.requests.reduce((sum, r) => 
      sum + r.size.request + r.size.response, 0);

    return {
      totalRequests,
      successfulRequests,
      failedRequests,
      averageResponseTime,
      totalDataTransferred,
    };
  }

  exportHAR(): string {
    // Export requests in HAR (HTTP Archive) format
    const har = {
      log: {
        version: '1.2',
        creator: {
          name: 'Tachikoma Network Inspector',
          version: '1.0.0',
        },
        entries: this.requests.map(request => ({
          startedDateTime: new Date(request.timestamp).toISOString(),
          time: request.timing.duration || 0,
          request: {
            method: request.method,
            url: request.url,
            headers: Object.entries(request.headers).map(([name, value]) => ({
              name,
              value,
            })),
            queryString: [],
            postData: undefined,
            headersSize: -1,
            bodySize: request.size.request,
          },
          response: {
            status: request.status || 0,
            statusText: request.statusText || '',
            headers: Object.entries(request.responseHeaders || {}).map(([name, value]) => ({
              name,
              value,
            })),
            content: {
              size: request.size.response,
              mimeType: request.responseHeaders?.['content-type'] || 'application/octet-stream',
            },
            redirectURL: '',
            headersSize: -1,
            bodySize: request.size.response,
          },
          cache: {},
          timings: {
            send: 0,
            wait: request.timing.duration || 0,
            receive: 0,
          },
        })),
      },
    };

    return JSON.stringify(har, null, 2);
  }
}

export const networkInspector = new NetworkInspector();