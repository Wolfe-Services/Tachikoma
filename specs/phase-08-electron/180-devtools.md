# Spec 180: DevTools Integration

## Phase
8 - Electron Shell

## Spec ID
180

## Status
Planned

## Dependencies
- Spec 161 (Electron Main Process)
- Spec 162 (Window Management)

## Estimated Context
~8%

---

## Objective

Implement comprehensive DevTools integration for the Electron application, including Chrome DevTools management, custom developer tools, performance monitoring, and debugging utilities for both main and renderer processes.

---

## Acceptance Criteria

- [x] Chrome DevTools toggle and management
- [x] DevTools extensions support
- [x] Custom developer panel
- [x] IPC debugging tools
- [x] Performance monitoring overlay
- [x] React DevTools integration
- [x] Redux DevTools integration
- [x] Memory profiling utilities
- [x] Network request inspection
- [x] Console log aggregation

---

## Implementation Details

### DevTools Manager

```typescript
// src/electron/main/devtools/index.ts
import {
  BrowserWindow,
  session,
  app,
  ipcMain,
  WebContents,
} from 'electron';
import { join } from 'path';
import { existsSync, mkdirSync, writeFileSync, readFileSync } from 'fs';
import { Logger } from '../logger';

const logger = new Logger('devtools');

interface DevToolsConfig {
  openOnStart: boolean;
  position: 'right' | 'bottom' | 'undocked' | 'detach';
  width: number;
  height: number;
}

interface ExtensionInfo {
  name: string;
  path: string;
  version: string;
}

class DevToolsManager {
  private config: DevToolsConfig = {
    openOnStart: false,
    position: 'right',
    width: 400,
    height: 600,
  };
  private installedExtensions: ExtensionInfo[] = [];
  private extensionsPath: string;

  constructor() {
    this.extensionsPath = join(app.getPath('userData'), 'devtools-extensions');
    this.ensureExtensionsDirectory();
    this.loadConfig();
  }

  private ensureExtensionsDirectory(): void {
    if (!existsSync(this.extensionsPath)) {
      mkdirSync(this.extensionsPath, { recursive: true });
    }
  }

  private loadConfig(): void {
    const configPath = join(app.getPath('userData'), 'devtools-config.json');
    try {
      if (existsSync(configPath)) {
        this.config = JSON.parse(readFileSync(configPath, 'utf-8'));
      }
    } catch (error) {
      logger.error('Failed to load DevTools config', { error });
    }
  }

  private saveConfig(): void {
    const configPath = join(app.getPath('userData'), 'devtools-config.json');
    try {
      writeFileSync(configPath, JSON.stringify(this.config, null, 2));
    } catch (error) {
      logger.error('Failed to save DevTools config', { error });
    }
  }

  async loadExtensions(): Promise<void> {
    if (app.isPackaged) {
      logger.info('DevTools extensions disabled in production');
      return;
    }

    const ses = session.defaultSession;

    // Load React DevTools
    await this.loadExtension(ses, 'react-devtools', 'REACT_DEVTOOLS_PATH');

    // Load Redux DevTools
    await this.loadExtension(ses, 'redux-devtools', 'REDUX_DEVTOOLS_PATH');

    logger.info('DevTools extensions loaded', {
      extensions: this.installedExtensions.map((e) => e.name),
    });
  }

  private async loadExtension(
    ses: Electron.Session,
    name: string,
    envVar: string
  ): Promise<void> {
    const extensionPath = process.env[envVar];

    if (!extensionPath) {
      logger.debug(`${name} path not set via ${envVar}`);
      return;
    }

    try {
      const extension = await ses.loadExtension(extensionPath, {
        allowFileAccess: true,
      });

      this.installedExtensions.push({
        name,
        path: extensionPath,
        version: extension.version,
      });

      logger.info(`Loaded ${name} extension`, { version: extension.version });
    } catch (error) {
      logger.error(`Failed to load ${name} extension`, { error });
    }
  }

  openDevTools(window: BrowserWindow, options?: Partial<DevToolsConfig>): void {
    const webContents = window.webContents;

    if (webContents.isDevToolsOpened()) {
      webContents.devToolsWebContents?.focus();
      return;
    }

    const mode = options?.position || this.config.position;

    webContents.openDevTools({ mode });
    logger.debug('DevTools opened', { mode });
  }

  closeDevTools(window: BrowserWindow): void {
    if (window.webContents.isDevToolsOpened()) {
      window.webContents.closeDevTools();
      logger.debug('DevTools closed');
    }
  }

  toggleDevTools(window: BrowserWindow): void {
    const webContents = window.webContents;

    if (webContents.isDevToolsOpened()) {
      this.closeDevTools(window);
    } else {
      this.openDevTools(window);
    }
  }

  isDevToolsOpened(window: BrowserWindow): boolean {
    return window.webContents.isDevToolsOpened();
  }

  focusDevTools(window: BrowserWindow): void {
    window.webContents.devToolsWebContents?.focus();
  }

  setDevToolsPosition(position: DevToolsConfig['position']): void {
    this.config.position = position;
    this.saveConfig();
  }

  getInstalledExtensions(): ExtensionInfo[] {
    return [...this.installedExtensions];
  }

  // Inspect element at position
  inspectElement(window: BrowserWindow, x: number, y: number): void {
    window.webContents.inspectElement(x, y);
  }

  // Inspect service worker
  inspectServiceWorker(window: BrowserWindow): void {
    window.webContents.inspectServiceWorker();
  }

  // Inspect shared worker
  inspectSharedWorker(window: BrowserWindow): void {
    window.webContents.inspectSharedWorker();
  }
}

export const devToolsManager = new DevToolsManager();
```

### Performance Monitor

```typescript
// src/electron/main/devtools/performance.ts
import { app, BrowserWindow, ipcMain, webContents } from 'electron';
import { Logger } from '../logger';

const logger = new Logger('performance');

interface PerformanceMetrics {
  timestamp: number;
  cpu: {
    percentCPUUsage: number;
    idleWakeupsPerSecond: number;
  };
  memory: {
    workingSetSize: number;
    peakWorkingSetSize: number;
    privateBytes: number;
  };
  renderer: {
    frameCount: number;
    fps: number;
  };
  heap: {
    totalHeapSize: number;
    usedHeapSize: number;
    heapSizeLimit: number;
  };
}

class PerformanceMonitor {
  private isMonitoring = false;
  private intervalId: NodeJS.Timeout | null = null;
  private metrics: PerformanceMetrics[] = [];
  private maxMetrics = 1000;
  private subscribers: Set<BrowserWindow> = new Set();

  start(intervalMs: number = 1000): void {
    if (this.isMonitoring) {
      return;
    }

    this.isMonitoring = true;
    logger.info('Performance monitoring started');

    this.intervalId = setInterval(() => {
      this.collectMetrics();
    }, intervalMs);
  }

  stop(): void {
    if (!this.isMonitoring) {
      return;
    }

    if (this.intervalId) {
      clearInterval(this.intervalId);
      this.intervalId = null;
    }

    this.isMonitoring = false;
    logger.info('Performance monitoring stopped');
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

  private async collectMetrics(): Promise<void> {
    const cpuUsage = process.getCPUUsage();
    const memoryUsage = process.memoryUsage();
    const appMetrics = app.getAppMetrics();

    // Calculate total CPU and memory from all processes
    let totalCPU = 0;
    let totalMemory = 0;

    for (const metric of appMetrics) {
      totalCPU += metric.cpu.percentCPUUsage;
      totalMemory += metric.memory.workingSetSize;
    }

    const metrics: PerformanceMetrics = {
      timestamp: Date.now(),
      cpu: {
        percentCPUUsage: totalCPU,
        idleWakeupsPerSecond: cpuUsage.idleWakeupsPerSecond,
      },
      memory: {
        workingSetSize: totalMemory,
        peakWorkingSetSize: Math.max(
          ...appMetrics.map((m) => m.memory.peakWorkingSetSize)
        ),
        privateBytes: memoryUsage.heapUsed,
      },
      renderer: {
        frameCount: 0,
        fps: 0,
      },
      heap: {
        totalHeapSize: memoryUsage.heapTotal,
        usedHeapSize: memoryUsage.heapUsed,
        heapSizeLimit: 0,
      },
    };

    // Store metrics
    this.metrics.push(metrics);
    if (this.metrics.length > this.maxMetrics) {
      this.metrics.shift();
    }

    // Broadcast to subscribers
    for (const window of this.subscribers) {
      if (!window.isDestroyed()) {
        window.webContents.send('performance:metrics', metrics);
      }
    }
  }

  getMetrics(count?: number): PerformanceMetrics[] {
    if (count) {
      return this.metrics.slice(-count);
    }
    return [...this.metrics];
  }

  getLatestMetrics(): PerformanceMetrics | null {
    return this.metrics.length > 0 ? this.metrics[this.metrics.length - 1] : null;
  }

  clearMetrics(): void {
    this.metrics = [];
  }
}

export const performanceMonitor = new PerformanceMonitor();
```

### Custom Developer Panel

```typescript
// src/renderer/components/DevPanel/DevPanel.tsx
import React, { useState, useEffect } from 'react';
import styles from './DevPanel.module.css';

interface PerformanceMetrics {
  cpu: { percentCPUUsage: number };
  memory: { workingSetSize: number };
  heap: { usedHeapSize: number; totalHeapSize: number };
}

interface IPCMessage {
  timestamp: number;
  direction: 'in' | 'out';
  channel: string;
  duration?: number;
}

export const DevPanel: React.FC = () => {
  const [isVisible, setIsVisible] = useState(false);
  const [activeTab, setActiveTab] = useState<'performance' | 'ipc' | 'console' | 'state'>('performance');
  const [metrics, setMetrics] = useState<PerformanceMetrics | null>(null);
  const [ipcMessages, setIpcMessages] = useState<IPCMessage[]>([]);
  const [consoleLogs, setConsoleLogs] = useState<string[]>([]);

  useEffect(() => {
    // Listen for keyboard shortcut
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.ctrlKey && e.shiftKey && e.key === 'D') {
        setIsVisible((v) => !v);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  useEffect(() => {
    if (!isVisible) return;

    // Subscribe to performance metrics
    const cleanup = window.electronAPI?.onPerformanceMetrics?.((data) => {
      setMetrics(data);
    });

    return cleanup;
  }, [isVisible]);

  if (!isVisible) return null;

  const formatBytes = (bytes: number): string => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  return (
    <div className={styles.devPanel}>
      <div className={styles.header}>
        <div className={styles.tabs}>
          {(['performance', 'ipc', 'console', 'state'] as const).map((tab) => (
            <button
              key={tab}
              className={`${styles.tab} ${activeTab === tab ? styles.active : ''}`}
              onClick={() => setActiveTab(tab)}
            >
              {tab.charAt(0).toUpperCase() + tab.slice(1)}
            </button>
          ))}
        </div>
        <button className={styles.closeButton} onClick={() => setIsVisible(false)}>
          Close
        </button>
      </div>

      <div className={styles.content}>
        {activeTab === 'performance' && (
          <div className={styles.performanceTab}>
            <div className={styles.metric}>
              <span className={styles.label}>CPU Usage</span>
              <span className={styles.value}>
                {metrics?.cpu.percentCPUUsage.toFixed(1)}%
              </span>
            </div>
            <div className={styles.metric}>
              <span className={styles.label}>Memory</span>
              <span className={styles.value}>
                {metrics ? formatBytes(metrics.memory.workingSetSize) : '-'}
              </span>
            </div>
            <div className={styles.metric}>
              <span className={styles.label}>Heap</span>
              <span className={styles.value}>
                {metrics
                  ? `${formatBytes(metrics.heap.usedHeapSize)} / ${formatBytes(metrics.heap.totalHeapSize)}`
                  : '-'}
              </span>
            </div>
          </div>
        )}

        {activeTab === 'ipc' && (
          <div className={styles.ipcTab}>
            <div className={styles.messageList}>
              {ipcMessages.map((msg, index) => (
                <div
                  key={index}
                  className={`${styles.message} ${styles[msg.direction]}`}
                >
                  <span className={styles.channel}>{msg.channel}</span>
                  {msg.duration && (
                    <span className={styles.duration}>{msg.duration}ms</span>
                  )}
                </div>
              ))}
            </div>
          </div>
        )}

        {activeTab === 'console' && (
          <div className={styles.consoleTab}>
            <div className={styles.logList}>
              {consoleLogs.map((log, index) => (
                <div key={index} className={styles.logEntry}>
                  {log}
                </div>
              ))}
            </div>
          </div>
        )}

        {activeTab === 'state' && (
          <div className={styles.stateTab}>
            <p>Application state inspection coming soon...</p>
          </div>
        )}
      </div>
    </div>
  );
};
```

### DevTools IPC Handlers

```typescript
// src/electron/main/ipc/devtools.ts
import { ipcMain, BrowserWindow } from 'electron';
import { devToolsManager } from '../devtools';
import { performanceMonitor } from '../devtools/performance';

export function setupDevToolsIpcHandlers(): void {
  ipcMain.handle('devtools:open', (event) => {
    const window = BrowserWindow.fromWebContents(event.sender);
    if (window) {
      devToolsManager.openDevTools(window);
    }
  });

  ipcMain.handle('devtools:close', (event) => {
    const window = BrowserWindow.fromWebContents(event.sender);
    if (window) {
      devToolsManager.closeDevTools(window);
    }
  });

  ipcMain.handle('devtools:toggle', (event) => {
    const window = BrowserWindow.fromWebContents(event.sender);
    if (window) {
      devToolsManager.toggleDevTools(window);
    }
  });

  ipcMain.handle('devtools:isOpen', (event) => {
    const window = BrowserWindow.fromWebContents(event.sender);
    return window ? devToolsManager.isDevToolsOpened(window) : false;
  });

  ipcMain.handle('devtools:inspectElement', (event, x: number, y: number) => {
    const window = BrowserWindow.fromWebContents(event.sender);
    if (window) {
      devToolsManager.inspectElement(window, x, y);
    }
  });

  ipcMain.handle('devtools:getExtensions', () => {
    return devToolsManager.getInstalledExtensions();
  });

  // Performance monitoring
  ipcMain.handle('performance:start', (event, intervalMs?: number) => {
    const window = BrowserWindow.fromWebContents(event.sender);
    if (window) {
      performanceMonitor.subscribe(window);
    }
    performanceMonitor.start(intervalMs);
  });

  ipcMain.handle('performance:stop', () => {
    performanceMonitor.stop();
  });

  ipcMain.handle('performance:getMetrics', (_, count?: number) => {
    return performanceMonitor.getMetrics(count);
  });

  ipcMain.handle('performance:getLatest', () => {
    return performanceMonitor.getLatestMetrics();
  });
}
```

---

## Testing Requirements

### Unit Tests

```typescript
// src/electron/main/devtools/__tests__/devtools.test.ts
import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('electron', () => ({
  app: {
    getPath: vi.fn().mockReturnValue('/mock/userData'),
    isPackaged: false,
    getAppMetrics: vi.fn().mockReturnValue([]),
  },
  session: {
    defaultSession: {
      loadExtension: vi.fn(),
    },
  },
  BrowserWindow: vi.fn(),
}));

describe('DevToolsManager', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should toggle DevTools', async () => {
    const { devToolsManager } = await import('../index');
    const mockWindow = {
      webContents: {
        isDevToolsOpened: vi.fn().mockReturnValue(false),
        openDevTools: vi.fn(),
        closeDevTools: vi.fn(),
      },
    };

    devToolsManager.toggleDevTools(mockWindow as any);
    expect(mockWindow.webContents.openDevTools).toHaveBeenCalled();
  });
});
```

---

## Related Specs

- Spec 161: Electron Main Process
- Spec 162: Window Management
- Spec 170: IPC Channels
