# Spec 185: Electron Tests

## Phase
8 - Electron Shell

## Spec ID
185

## Status
Planned

## Dependencies
- Spec 161 (Electron Main Process)
- All Phase 8 Specs

## Estimated Context
~12%

---

## Objective

Implement comprehensive testing infrastructure for the Electron application, including unit tests for main process code, integration tests for IPC communication, end-to-end tests for the complete application, and performance benchmarks.

---

## Acceptance Criteria

- [ ] Unit test framework for main process
- [ ] Integration tests for IPC channels
- [ ] E2E tests with Playwright
- [ ] Visual regression tests
- [ ] Performance benchmarks
- [ ] CI/CD test pipeline
- [ ] Code coverage reporting
- [ ] Snapshot testing
- [ ] Cross-platform test matrix
- [ ] Automated test fixtures

---

## Implementation Details

### Test Configuration

```typescript
// vitest.config.electron.ts
import { defineConfig } from 'vitest/config';
import { resolve } from 'path';

export default defineConfig({
  test: {
    name: 'electron-main',
    root: './src/electron',
    environment: 'node',
    include: ['**/__tests__/**/*.test.ts'],
    exclude: ['**/node_modules/**'],
    globals: true,
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      include: ['main/**/*.ts', 'preload/**/*.ts'],
      exclude: ['**/__tests__/**', '**/*.d.ts'],
    },
    setupFiles: ['./test/setup.ts'],
    mockReset: true,
    restoreMocks: true,
  },
  resolve: {
    alias: {
      '@electron': resolve(__dirname, './src/electron'),
      '@shared': resolve(__dirname, './src/shared'),
    },
  },
});
```

### Test Setup

```typescript
// src/electron/test/setup.ts
import { vi } from 'vitest';

// Mock Electron module
vi.mock('electron', () => {
  const mockApp = {
    getPath: vi.fn((name: string) => `/mock/${name}`),
    getVersion: vi.fn(() => '1.0.0-test'),
    getName: vi.fn(() => 'Tachikoma'),
    isPackaged: false,
    on: vi.fn(),
    once: vi.fn(),
    quit: vi.fn(),
    exit: vi.fn(),
    relaunch: vi.fn(),
    requestSingleInstanceLock: vi.fn(() => true),
    setAsDefaultProtocolClient: vi.fn(),
    enableSandbox: vi.fn(),
    whenReady: vi.fn(() => Promise.resolve()),
    dock: {
      setBadge: vi.fn(),
      bounce: vi.fn(),
      setIcon: vi.fn(),
    },
  };

  const mockBrowserWindow = vi.fn().mockImplementation(() => ({
    loadFile: vi.fn().mockResolvedValue(undefined),
    loadURL: vi.fn().mockResolvedValue(undefined),
    on: vi.fn(),
    once: vi.fn(),
    show: vi.fn(),
    hide: vi.fn(),
    close: vi.fn(),
    focus: vi.fn(),
    minimize: vi.fn(),
    maximize: vi.fn(),
    restore: vi.fn(),
    isMinimized: vi.fn(() => false),
    isMaximized: vi.fn(() => false),
    isDestroyed: vi.fn(() => false),
    isVisible: vi.fn(() => true),
    setTitle: vi.fn(),
    getTitle: vi.fn(() => 'Tachikoma'),
    getBounds: vi.fn(() => ({ x: 0, y: 0, width: 1200, height: 800 })),
    setBounds: vi.fn(),
    setProgressBar: vi.fn(),
    webContents: {
      send: vi.fn(),
      on: vi.fn(),
      once: vi.fn(),
      openDevTools: vi.fn(),
      closeDevTools: vi.fn(),
      isDevToolsOpened: vi.fn(() => false),
      setWindowOpenHandler: vi.fn(),
      getURL: vi.fn(() => 'http://localhost:5173'),
    },
  }));

  (mockBrowserWindow as any).getAllWindows = vi.fn(() => []);
  (mockBrowserWindow as any).fromWebContents = vi.fn();

  return {
    app: mockApp,
    BrowserWindow: mockBrowserWindow,
    ipcMain: {
      handle: vi.fn(),
      on: vi.fn(),
      once: vi.fn(),
      removeHandler: vi.fn(),
      removeListener: vi.fn(),
    },
    ipcRenderer: {
      invoke: vi.fn(),
      on: vi.fn(),
      once: vi.fn(),
      send: vi.fn(),
      removeListener: vi.fn(),
    },
    contextBridge: {
      exposeInMainWorld: vi.fn(),
    },
    session: {
      defaultSession: {
        webRequest: {
          onHeadersReceived: vi.fn(),
          onBeforeRequest: vi.fn(),
        },
        setPermissionRequestHandler: vi.fn(),
        setPermissionCheckHandler: vi.fn(),
        clearCache: vi.fn(),
        loadExtension: vi.fn(),
      },
    },
    protocol: {
      registerSchemesAsPrivileged: vi.fn(),
      handle: vi.fn(),
      interceptFileProtocol: vi.fn(),
      unhandle: vi.fn(),
    },
    dialog: {
      showOpenDialog: vi.fn().mockResolvedValue({ canceled: false, filePaths: [] }),
      showSaveDialog: vi.fn().mockResolvedValue({ canceled: false, filePath: '' }),
      showMessageBox: vi.fn().mockResolvedValue({ response: 0, checkboxChecked: false }),
      showErrorBox: vi.fn(),
    },
    shell: {
      openExternal: vi.fn().mockResolvedValue(undefined),
      openPath: vi.fn().mockResolvedValue(''),
      showItemInFolder: vi.fn(),
    },
    nativeTheme: {
      shouldUseDarkColors: false,
      themeSource: 'system',
      on: vi.fn(),
    },
    Menu: {
      buildFromTemplate: vi.fn(() => ({})),
      setApplicationMenu: vi.fn(),
      getApplicationMenu: vi.fn(),
    },
    Tray: vi.fn().mockImplementation(() => ({
      on: vi.fn(),
      setToolTip: vi.fn(),
      setContextMenu: vi.fn(),
      setImage: vi.fn(),
      destroy: vi.fn(),
    })),
    Notification: vi.fn().mockImplementation(() => ({
      on: vi.fn(),
      show: vi.fn(),
      close: vi.fn(),
    })),
    nativeImage: {
      createFromPath: vi.fn(() => ({
        resize: vi.fn().mockReturnThis(),
        toDataURL: vi.fn(() => ''),
      })),
    },
    powerMonitor: {
      on: vi.fn(),
    },
    powerSaveBlocker: {
      start: vi.fn(() => 1),
      stop: vi.fn(),
    },
    crashReporter: {
      start: vi.fn(),
      getUploadedReports: vi.fn(() => []),
    },
    clipboard: {
      readText: vi.fn(() => ''),
      writeText: vi.fn(),
    },
    screen: {
      getAllDisplays: vi.fn(() => [{ bounds: { x: 0, y: 0, width: 1920, height: 1080 } }]),
      getPrimaryDisplay: vi.fn(() => ({ workAreaSize: { width: 1920, height: 1040 } })),
    },
    net: {
      fetch: vi.fn(),
    },
  };
});

// Mock fs for file system tests
vi.mock('fs', async () => {
  const memfs = await import('memfs');
  return {
    ...memfs.fs,
    promises: memfs.fs.promises,
  };
});

// Global test utilities
globalThis.createMockWindow = () => {
  const { BrowserWindow } = require('electron');
  return new BrowserWindow();
};

globalThis.createMockEvent = () => ({
  sender: {
    id: 1,
    send: vi.fn(),
    isDestroyed: () => false,
  },
});
```

### E2E Test Configuration

```typescript
// playwright.config.ts
import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './e2e',
  timeout: 60000,
  retries: process.env.CI ? 2 : 0,
  workers: 1, // Electron tests must run sequentially
  reporter: [
    ['html', { open: 'never' }],
    ['json', { outputFile: 'test-results/results.json' }],
  ],
  use: {
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    video: 'on-first-retry',
  },
  projects: [
    {
      name: 'electron',
      testMatch: '**/*.e2e.ts',
    },
  ],
});
```

### E2E Test Utilities

```typescript
// e2e/utils/electron.ts
import { _electron as electron, ElectronApplication, Page } from 'playwright';
import { join } from 'path';

export interface TestContext {
  app: ElectronApplication;
  page: Page;
}

export async function launchApp(): Promise<TestContext> {
  const app = await electron.launch({
    args: [join(__dirname, '../../dist/electron/main/index.js')],
    env: {
      ...process.env,
      NODE_ENV: 'test',
      ELECTRON_IS_DEV: '0',
    },
  });

  const page = await app.firstWindow();
  await page.waitForLoadState('domcontentloaded');

  return { app, page };
}

export async function closeApp(app: ElectronApplication): Promise<void> {
  await app.close();
}

// Helper to evaluate in main process
export async function evaluateInMain<T>(
  app: ElectronApplication,
  fn: (modules: { app: Electron.App; BrowserWindow: typeof Electron.BrowserWindow }) => T
): Promise<T> {
  return app.evaluate(fn);
}

// Helper to wait for IPC event
export async function waitForIpc(
  page: Page,
  channel: string,
  timeout: number = 5000
): Promise<unknown> {
  return page.evaluate(
    ({ channel, timeout }) => {
      return new Promise((resolve, reject) => {
        const timer = setTimeout(() => {
          reject(new Error(`Timeout waiting for IPC: ${channel}`));
        }, timeout);

        window.electronAPI?.once?.(channel, (data: unknown) => {
          clearTimeout(timer);
          resolve(data);
        });
      });
    },
    { channel, timeout }
  );
}
```

### E2E Test Example

```typescript
// e2e/app.e2e.ts
import { test, expect } from '@playwright/test';
import { launchApp, closeApp, TestContext, evaluateInMain } from './utils/electron';

let context: TestContext;

test.beforeAll(async () => {
  context = await launchApp();
});

test.afterAll(async () => {
  await closeApp(context.app);
});

test.describe('Application Launch', () => {
  test('should launch with correct title', async () => {
    const title = await context.page.title();
    expect(title).toContain('Tachikoma');
  });

  test('should have single window', async () => {
    const windowCount = await evaluateInMain(context.app, ({ BrowserWindow }) => {
      return BrowserWindow.getAllWindows().length;
    });

    expect(windowCount).toBe(1);
  });

  test('should be in development mode', async () => {
    const isPackaged = await evaluateInMain(context.app, ({ app }) => {
      return app.isPackaged;
    });

    expect(isPackaged).toBe(false);
  });
});

test.describe('Window Management', () => {
  test('should minimize and restore', async () => {
    await evaluateInMain(context.app, ({ BrowserWindow }) => {
      const win = BrowserWindow.getAllWindows()[0];
      win.minimize();
    });

    const isMinimized = await evaluateInMain(context.app, ({ BrowserWindow }) => {
      return BrowserWindow.getAllWindows()[0].isMinimized();
    });

    expect(isMinimized).toBe(true);

    await evaluateInMain(context.app, ({ BrowserWindow }) => {
      BrowserWindow.getAllWindows()[0].restore();
    });
  });

  test('should handle maximize', async () => {
    await evaluateInMain(context.app, ({ BrowserWindow }) => {
      BrowserWindow.getAllWindows()[0].maximize();
    });

    const isMaximized = await evaluateInMain(context.app, ({ BrowserWindow }) => {
      return BrowserWindow.getAllWindows()[0].isMaximized();
    });

    expect(isMaximized).toBe(true);
  });
});

test.describe('IPC Communication', () => {
  test('should get app info via IPC', async () => {
    const appInfo = await context.page.evaluate(async () => {
      return window.electronAPI?.getAppInfo();
    });

    expect(appInfo).toHaveProperty('version');
    expect(appInfo).toHaveProperty('electron');
    expect(appInfo).toHaveProperty('platform');
  });
});
```

### Performance Benchmark Tests

```typescript
// e2e/performance.e2e.ts
import { test, expect } from '@playwright/test';
import { launchApp, closeApp, TestContext, evaluateInMain } from './utils/electron';

let context: TestContext;

test.beforeAll(async () => {
  context = await launchApp();
});

test.afterAll(async () => {
  await closeApp(context.app);
});

test.describe('Performance', () => {
  test('should launch within acceptable time', async () => {
    const startTime = Date.now();
    const newContext = await launchApp();
    const launchTime = Date.now() - startTime;

    expect(launchTime).toBeLessThan(5000); // 5 seconds max

    await closeApp(newContext.app);
  });

  test('should have acceptable memory usage', async () => {
    const metrics = await evaluateInMain(context.app, ({ app }) => {
      return app.getAppMetrics();
    });

    const totalMemory = metrics.reduce((sum: number, m: any) => {
      return sum + m.memory.workingSetSize;
    }, 0);

    // Less than 500MB total
    expect(totalMemory).toBeLessThan(500 * 1024 * 1024);
  });

  test('should render within acceptable time', async () => {
    const metrics = await context.page.evaluate(() => {
      const entries = performance.getEntriesByType('navigation') as PerformanceNavigationTiming[];
      const nav = entries[0];
      return {
        domContentLoaded: nav.domContentLoadedEventEnd - nav.startTime,
        load: nav.loadEventEnd - nav.startTime,
      };
    });

    expect(metrics.domContentLoaded).toBeLessThan(2000);
    expect(metrics.load).toBeLessThan(3000);
  });
});
```

### CI/CD Test Pipeline

```yaml
# .github/workflows/test.yml
name: Tests

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  unit-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
      - run: npm ci
      - run: npm run test:unit
      - uses: codecov/codecov-action@v3
        with:
          files: ./coverage/lcov.info

  e2e-tests:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
      - run: npm ci
      - run: npx playwright install --with-deps
      - run: npm run build
      - run: npm run test:e2e
        env:
          CI: true
      - uses: actions/upload-artifact@v4
        if: failure()
        with:
          name: test-results-${{ matrix.os }}
          path: test-results/
```

### Test Scripts

```json
{
  "scripts": {
    "test": "npm run test:unit && npm run test:e2e",
    "test:unit": "vitest run --config vitest.config.electron.ts",
    "test:unit:watch": "vitest --config vitest.config.electron.ts",
    "test:unit:coverage": "vitest run --config vitest.config.electron.ts --coverage",
    "test:e2e": "playwright test",
    "test:e2e:headed": "playwright test --headed",
    "test:e2e:debug": "playwright test --debug",
    "test:e2e:report": "playwright show-report"
  }
}
```

---

## Related Specs

- All Phase 8 Specs
- Spec 161: Electron Main Process
- Spec 170: IPC Channels
