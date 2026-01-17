# 483 - E2E Test Framework

**Phase:** 22 - Testing Infrastructure
**Spec ID:** 483
**Status:** Planned
**Dependencies:** 471-test-harness, 161-electron-main
**Estimated Context:** ~12% of Sonnet window

---

## Objective

Set up end-to-end testing infrastructure using Playwright for testing the complete Tachikoma application including Electron shell, Svelte UI, and Rust backend interactions.

---

## Acceptance Criteria

- [x] Playwright configured for Electron testing
- [x] Test utilities for common E2E operations
- [x] Page object models for main UI areas
- [x] Screenshot and video capture on failures
- [x] Parallel test execution support
- [x] CI integration with artifacts

---

## Implementation Details

### 1. Playwright Configuration

Create `e2e/playwright.config.ts`:

```typescript
import { defineConfig, devices } from '@playwright/test';
import path from 'path';

// Path to the built Electron app
const electronPath = process.platform === 'darwin'
  ? path.join(__dirname, '../electron/out/Tachikoma-darwin-arm64/Tachikoma.app/Contents/MacOS/Tachikoma')
  : process.platform === 'win32'
    ? path.join(__dirname, '../electron/out/Tachikoma-win32-x64/Tachikoma.exe')
    : path.join(__dirname, '../electron/out/Tachikoma-linux-x64/tachikoma');

export default defineConfig({
  testDir: './tests',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: [
    ['html', { outputFolder: 'playwright-report' }],
    ['json', { outputFile: 'test-results/results.json' }],
    ['junit', { outputFile: 'test-results/junit.xml' }],
  ],
  timeout: 60_000,

  use: {
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    video: 'on-first-retry',
  },

  projects: [
    {
      name: 'electron',
      use: {
        // Custom launch for Electron
      },
    },
  ],

  // Web server for development testing
  webServer: process.env.E2E_DEV ? {
    command: 'npm run dev',
    url: 'http://localhost:5173',
    reuseExistingServer: !process.env.CI,
  } : undefined,
});
```

### 2. Electron Test Utilities

Create `e2e/utils/electron.ts`:

```typescript
import { _electron as electron, ElectronApplication, Page } from '@playwright/test';
import path from 'path';

export interface ElectronTestContext {
  app: ElectronApplication;
  page: Page;
}

/**
 * Launch the Electron application for testing
 */
export async function launchElectronApp(): Promise<ElectronTestContext> {
  const electronPath = getElectronPath();

  const app = await electron.launch({
    args: [
      path.join(__dirname, '../../electron/dist/main.js'),
      '--test-mode',
    ],
    env: {
      ...process.env,
      NODE_ENV: 'test',
      E2E_TESTING: 'true',
    },
  });

  // Wait for the first window
  const page = await app.firstWindow();

  // Wait for the app to be ready
  await page.waitForLoadState('domcontentloaded');

  return { app, page };
}

/**
 * Get the path to the Electron executable
 */
function getElectronPath(): string {
  const platform = process.platform;
  const arch = process.arch;

  if (platform === 'darwin') {
    return path.join(
      __dirname,
      `../../electron/out/Tachikoma-darwin-${arch}/Tachikoma.app/Contents/MacOS/Tachikoma`
    );
  } else if (platform === 'win32') {
    return path.join(
      __dirname,
      `../../electron/out/Tachikoma-win32-${arch}/Tachikoma.exe`
    );
  } else {
    return path.join(
      __dirname,
      `../../electron/out/Tachikoma-linux-${arch}/tachikoma`
    );
  }
}

/**
 * Close the Electron application
 */
export async function closeElectronApp(ctx: ElectronTestContext): Promise<void> {
  await ctx.app.close();
}

/**
 * Take a screenshot with a descriptive name
 */
export async function screenshot(page: Page, name: string): Promise<void> {
  await page.screenshot({
    path: `test-results/screenshots/${name}.png`,
    fullPage: true,
  });
}

/**
 * Wait for IPC to complete
 */
export async function waitForIpc(page: Page, timeout = 5000): Promise<void> {
  await page.waitForFunction(
    () => (window as any).__ipcReady === true,
    { timeout }
  );
}

/**
 * Execute an IPC call from the test
 */
export async function invokeIpc<T>(
  ctx: ElectronTestContext,
  channel: string,
  ...args: unknown[]
): Promise<T> {
  return ctx.page.evaluate(
    ({ channel, args }) => {
      return (window as any).tachikoma.invoke(channel, ...args);
    },
    { channel, args }
  );
}
```

### 3. Page Object Models

Create `e2e/pages/BasePage.ts`:

```typescript
import { Page, Locator } from '@playwright/test';

export abstract class BasePage {
  protected page: Page;

  constructor(page: Page) {
    this.page = page;
  }

  /**
   * Wait for page to be ready
   */
  async waitForReady(): Promise<void> {
    await this.page.waitForLoadState('networkidle');
  }

  /**
   * Get element by test ID
   */
  getByTestId(testId: string): Locator {
    return this.page.locator(`[data-testid="${testId}"]`);
  }

  /**
   * Take a screenshot
   */
  async screenshot(name: string): Promise<void> {
    await this.page.screenshot({
      path: `test-results/screenshots/${name}.png`,
    });
  }

  /**
   * Wait for toast notification
   */
  async waitForToast(text: string): Promise<void> {
    await this.page.locator('.toast').filter({ hasText: text }).waitFor();
  }
}
```

Create `e2e/pages/MissionPage.ts`:

```typescript
import { Page, Locator, expect } from '@playwright/test';
import { BasePage } from './BasePage';

export class MissionPage extends BasePage {
  // Locators
  readonly newMissionButton: Locator;
  readonly promptInput: Locator;
  readonly backendSelect: Locator;
  readonly startButton: Locator;
  readonly stopButton: Locator;
  readonly progressBar: Locator;
  readonly logViewer: Locator;
  readonly contextMeter: Locator;

  constructor(page: Page) {
    super(page);
    this.newMissionButton = this.getByTestId('new-mission-btn');
    this.promptInput = this.getByTestId('prompt-input');
    this.backendSelect = this.getByTestId('backend-select');
    this.startButton = this.getByTestId('start-mission-btn');
    this.stopButton = this.getByTestId('stop-mission-btn');
    this.progressBar = this.getByTestId('mission-progress');
    this.logViewer = this.getByTestId('log-viewer');
    this.contextMeter = this.getByTestId('context-meter');
  }

  async goto(): Promise<void> {
    await this.page.goto('/mission');
    await this.waitForReady();
  }

  async createNewMission(): Promise<void> {
    await this.newMissionButton.click();
  }

  async setPrompt(prompt: string): Promise<void> {
    await this.promptInput.fill(prompt);
  }

  async selectBackend(backend: string): Promise<void> {
    await this.backendSelect.selectOption(backend);
  }

  async startMission(): Promise<void> {
    await this.startButton.click();
  }

  async stopMission(): Promise<void> {
    await this.stopButton.click();
  }

  async waitForMissionStart(): Promise<void> {
    await expect(this.progressBar).toBeVisible();
  }

  async waitForMissionComplete(timeout = 120000): Promise<void> {
    await this.page.waitForSelector('[data-testid="mission-complete"]', {
      timeout,
    });
  }

  async getLogContent(): Promise<string> {
    return this.logViewer.textContent() ?? '';
  }

  async getContextUsage(): Promise<number> {
    const text = await this.contextMeter.textContent();
    const match = text?.match(/(\d+)%/);
    return match ? parseInt(match[1]) : 0;
  }
}
```

Create `e2e/pages/SettingsPage.ts`:

```typescript
import { Page, Locator } from '@playwright/test';
import { BasePage } from './BasePage';

export class SettingsPage extends BasePage {
  readonly themeSelect: Locator;
  readonly apiKeyInput: Locator;
  readonly saveButton: Locator;
  readonly backendList: Locator;

  constructor(page: Page) {
    super(page);
    this.themeSelect = this.getByTestId('theme-select');
    this.apiKeyInput = this.getByTestId('api-key-input');
    this.saveButton = this.getByTestId('save-settings-btn');
    this.backendList = this.getByTestId('backend-list');
  }

  async goto(): Promise<void> {
    await this.page.goto('/settings');
    await this.waitForReady();
  }

  async setTheme(theme: 'light' | 'dark' | 'tachikoma'): Promise<void> {
    await this.themeSelect.selectOption(theme);
  }

  async setApiKey(key: string): Promise<void> {
    await this.apiKeyInput.fill(key);
  }

  async saveSettings(): Promise<void> {
    await this.saveButton.click();
    await this.waitForToast('Settings saved');
  }
}
```

### 4. Test Fixtures

Create `e2e/fixtures/index.ts`:

```typescript
import { test as base, expect } from '@playwright/test';
import { ElectronTestContext, launchElectronApp, closeElectronApp } from '../utils/electron';
import { MissionPage } from '../pages/MissionPage';
import { SettingsPage } from '../pages/SettingsPage';

// Extend base test with custom fixtures
export const test = base.extend<{
  electronApp: ElectronTestContext;
  missionPage: MissionPage;
  settingsPage: SettingsPage;
}>({
  electronApp: async ({}, use) => {
    const ctx = await launchElectronApp();
    await use(ctx);
    await closeElectronApp(ctx);
  },

  missionPage: async ({ electronApp }, use) => {
    const page = new MissionPage(electronApp.page);
    await use(page);
  },

  settingsPage: async ({ electronApp }, use) => {
    const page = new SettingsPage(electronApp.page);
    await use(page);
  },
});

export { expect };
```

### 5. Example E2E Tests

Create `e2e/tests/mission.spec.ts`:

```typescript
import { test, expect } from '../fixtures';

test.describe('Mission Panel', () => {
  test('should create and start a new mission', async ({ missionPage }) => {
    await missionPage.goto();
    await missionPage.createNewMission();
    await missionPage.setPrompt('Test mission prompt');
    await missionPage.selectBackend('mock');
    await missionPage.startMission();
    await missionPage.waitForMissionStart();

    await expect(missionPage.progressBar).toBeVisible();
  });

  test('should display logs during mission', async ({ missionPage }) => {
    await missionPage.goto();
    await missionPage.createNewMission();
    await missionPage.setPrompt('Simple test');
    await missionPage.selectBackend('mock');
    await missionPage.startMission();

    // Wait for some logs to appear
    await missionPage.page.waitForTimeout(2000);
    const logs = await missionPage.getLogContent();

    expect(logs.length).toBeGreaterThan(0);
  });

  test('should stop mission when requested', async ({ missionPage }) => {
    await missionPage.goto();
    await missionPage.createNewMission();
    await missionPage.setPrompt('Long running test');
    await missionPage.selectBackend('mock');
    await missionPage.startMission();
    await missionPage.waitForMissionStart();

    await missionPage.stopMission();

    await expect(missionPage.stopButton).not.toBeVisible();
  });
});

test.describe('Settings', () => {
  test('should change theme', async ({ settingsPage }) => {
    await settingsPage.goto();
    await settingsPage.setTheme('dark');
    await settingsPage.saveSettings();

    // Verify theme applied
    const body = settingsPage.page.locator('body');
    await expect(body).toHaveClass(/dark-theme/);
  });
});
```

### 6. CI Configuration

Add to `.github/workflows/e2e.yml`:

```yaml
name: E2E Tests

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  e2e:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]

    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Install dependencies
        run: npm ci

      - name: Build Electron app
        run: npm run build:electron

      - name: Install Playwright
        run: npx playwright install --with-deps

      - name: Run E2E tests
        run: npm run test:e2e

      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: e2e-results-${{ matrix.os }}
          path: |
            e2e/playwright-report/
            e2e/test-results/
```

---

## Testing Requirements

1. Playwright launches Electron app successfully
2. Page objects encapsulate UI interactions
3. Tests run reliably in CI
4. Screenshots capture on failure
5. Video recording works on first retry

---

## Related Specs

- Depends on: [471-test-harness.md](471-test-harness.md), [161-electron-main.md](../phase-08-electron/161-electron-main.md)
- Next: [484-e2e-scenarios.md](484-e2e-scenarios.md)
- Related: [485-visual-regression.md](485-visual-regression.md)
