# 485 - Visual Regression Tests

**Phase:** 22 - Testing Infrastructure
**Spec ID:** 485
**Status:** Planned
**Dependencies:** 483-e2e-framework
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Implement visual regression testing using Playwright's screenshot comparison capabilities to detect unintended UI changes across the application.

---

## Acceptance Criteria

- [ ] Baseline screenshots captured for key views
- [ ] Visual diffs generated on failures
- [ ] Theme variations tested (light, dark, tachikoma)
- [ ] Responsive breakpoints covered
- [ ] CI integration with artifact storage
- [ ] Update workflow for intentional changes

---

## Implementation Details

### 1. Visual Test Configuration

Create `e2e/visual/playwright.visual.config.ts`:

```typescript
import { defineConfig } from '@playwright/test';
import baseConfig from '../playwright.config';

export default defineConfig({
  ...baseConfig,
  testDir: './visual-tests',

  // Visual test specific settings
  expect: {
    toHaveScreenshot: {
      // Allow slight pixel differences due to font rendering
      maxDiffPixelRatio: 0.01,
      threshold: 0.2,
      animations: 'disabled',
    },
  },

  // Run visual tests serially for consistency
  fullyParallel: false,
  workers: 1,

  // Generate snapshots in specific directory
  snapshotDir: './visual-tests/snapshots',
  snapshotPathTemplate: '{snapshotDir}/{testFilePath}/{arg}{ext}',

  projects: [
    {
      name: 'visual-desktop',
      use: {
        viewport: { width: 1920, height: 1080 },
      },
    },
    {
      name: 'visual-laptop',
      use: {
        viewport: { width: 1440, height: 900 },
      },
    },
    {
      name: 'visual-tablet',
      use: {
        viewport: { width: 1024, height: 768 },
      },
    },
  ],
});
```

### 2. Visual Test Utilities

Create `e2e/visual/utils.ts`:

```typescript
import { Page, expect } from '@playwright/test';

export interface VisualTestOptions {
  /** Mask dynamic elements (timestamps, counters, etc.) */
  mask?: string[];
  /** Wait for animations to complete */
  waitForAnimations?: boolean;
  /** Additional wait time in ms */
  additionalWait?: number;
  /** Clip to specific element */
  clip?: string;
}

/**
 * Take a visual snapshot with common options
 */
export async function visualSnapshot(
  page: Page,
  name: string,
  options: VisualTestOptions = {}
): Promise<void> {
  // Wait for page to stabilize
  await page.waitForLoadState('networkidle');

  if (options.waitForAnimations !== false) {
    // Disable CSS animations
    await page.addStyleTag({
      content: `
        *, *::before, *::after {
          animation-duration: 0s !important;
          animation-delay: 0s !important;
          transition-duration: 0s !important;
          transition-delay: 0s !important;
        }
      `,
    });
  }

  if (options.additionalWait) {
    await page.waitForTimeout(options.additionalWait);
  }

  // Build mask locators
  const maskLocators = options.mask?.map(selector => page.locator(selector)) ?? [];

  // Add default masks for dynamic content
  const defaultMasks = [
    '[data-testid="timestamp"]',
    '[data-testid="duration"]',
    '[data-testid="session-id"]',
  ];

  for (const selector of defaultMasks) {
    if (await page.locator(selector).count() > 0) {
      maskLocators.push(page.locator(selector));
    }
  }

  const screenshotOptions: Parameters<typeof expect>[1] = {
    mask: maskLocators,
  };

  if (options.clip) {
    await expect(page.locator(options.clip)).toHaveScreenshot(`${name}.png`, screenshotOptions);
  } else {
    await expect(page).toHaveScreenshot(`${name}.png`, screenshotOptions);
  }
}

/**
 * Test all theme variations
 */
export async function testAllThemes(
  page: Page,
  testFn: (theme: string) => Promise<void>
): Promise<void> {
  const themes = ['light', 'dark', 'tachikoma'];

  for (const theme of themes) {
    await page.evaluate((t) => {
      document.documentElement.setAttribute('data-theme', t);
    }, theme);

    await testFn(theme);
  }
}

/**
 * Component visual test helper
 */
export async function testComponent(
  page: Page,
  componentSelector: string,
  name: string,
  options: VisualTestOptions = {}
): Promise<void> {
  await expect(page.locator(componentSelector)).toHaveScreenshot(
    `${name}.png`,
    {
      mask: options.mask?.map(s => page.locator(s)),
    }
  );
}
```

### 3. Visual Test Suites

Create `e2e/visual/visual-tests/dashboard.visual.ts`:

```typescript
import { test, expect } from '@playwright/test';
import { visualSnapshot, testAllThemes } from '../utils';

test.describe('Dashboard Visual Tests', () => {
  test.beforeEach(async ({ page }) => {
    // Load test data
    await page.evaluate(() => {
      localStorage.setItem('onboarding_complete', 'true');
    });
    await page.goto('/dashboard');
    await page.waitForLoadState('networkidle');
  });

  test('dashboard empty state', async ({ page }) => {
    await visualSnapshot(page, 'dashboard-empty');
  });

  test('dashboard with missions', async ({ page }) => {
    // Seed mission data
    await page.evaluate(() => {
      localStorage.setItem('missions', JSON.stringify([
        { id: '1', name: 'Test Mission', status: 'complete' },
        { id: '2', name: 'Another Mission', status: 'running' },
      ]));
    });
    await page.reload();

    await visualSnapshot(page, 'dashboard-with-missions', {
      mask: ['[data-testid="mission-timestamp"]'],
    });
  });

  test('dashboard themes', async ({ page }) => {
    await testAllThemes(page, async (theme) => {
      await visualSnapshot(page, `dashboard-${theme}`);
    });
  });
});
```

Create `e2e/visual/visual-tests/components.visual.ts`:

```typescript
import { test, expect } from '@playwright/test';
import { testComponent, testAllThemes } from '../utils';

test.describe('Component Visual Tests', () => {
  test.describe('Buttons', () => {
    test.beforeEach(async ({ page }) => {
      await page.goto('/storybook/button');
    });

    test('button variants', async ({ page }) => {
      await testComponent(page, '[data-testid="button-primary"]', 'button-primary');
      await testComponent(page, '[data-testid="button-secondary"]', 'button-secondary');
      await testComponent(page, '[data-testid="button-danger"]', 'button-danger');
      await testComponent(page, '[data-testid="button-ghost"]', 'button-ghost');
    });

    test('button states', async ({ page }) => {
      await testComponent(page, '[data-testid="button-disabled"]', 'button-disabled');
      await testComponent(page, '[data-testid="button-loading"]', 'button-loading');
    });
  });

  test.describe('Inputs', () => {
    test.beforeEach(async ({ page }) => {
      await page.goto('/storybook/input');
    });

    test('input variants', async ({ page }) => {
      await testComponent(page, '[data-testid="input-default"]', 'input-default');
      await testComponent(page, '[data-testid="input-error"]', 'input-error');
      await testComponent(page, '[data-testid="input-disabled"]', 'input-disabled');
    });
  });

  test.describe('Cards', () => {
    test.beforeEach(async ({ page }) => {
      await page.goto('/storybook/card');
    });

    test('card variants', async ({ page }) => {
      await testComponent(page, '[data-testid="card-default"]', 'card-default');
      await testComponent(page, '[data-testid="card-elevated"]', 'card-elevated');
      await testComponent(page, '[data-testid="card-interactive"]', 'card-interactive');
    });
  });

  test.describe('Modals', () => {
    test('modal appearance', async ({ page }) => {
      await page.goto('/storybook/modal');
      await page.getByTestId('open-modal-btn').click();
      await page.waitForSelector('[data-testid="modal"]');

      await expect(page).toHaveScreenshot('modal-open.png', {
        mask: [page.locator('[data-testid="modal-backdrop"]')],
      });
    });
  });
});
```

Create `e2e/visual/visual-tests/pages.visual.ts`:

```typescript
import { test, expect } from '@playwright/test';
import { visualSnapshot, testAllThemes } from '../utils';

test.describe('Page Visual Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.evaluate(() => {
      localStorage.setItem('onboarding_complete', 'true');
    });
  });

  test('mission panel', async ({ page }) => {
    await page.goto('/mission');
    await visualSnapshot(page, 'mission-panel');
  });

  test('spec browser', async ({ page }) => {
    await page.goto('/specs');
    await visualSnapshot(page, 'spec-browser');
  });

  test('forge panel', async ({ page }) => {
    await page.goto('/forge');
    await visualSnapshot(page, 'forge-panel');
  });

  test('settings page', async ({ page }) => {
    await page.goto('/settings');
    await visualSnapshot(page, 'settings-page');
  });

  test('all pages in all themes', async ({ page }) => {
    const pages = ['/dashboard', '/mission', '/specs', '/forge', '/settings'];

    for (const pagePath of pages) {
      await page.goto(pagePath);
      const pageName = pagePath.replace('/', '') || 'home';

      await testAllThemes(page, async (theme) => {
        await visualSnapshot(page, `${pageName}-${theme}`);
      });
    }
  });
});
```

### 4. CI Integration

Add to `.github/workflows/visual-tests.yml`:

```yaml
name: Visual Regression Tests

on:
  pull_request:
    branches: [main]

jobs:
  visual-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Install dependencies
        run: npm ci

      - name: Install Playwright
        run: npx playwright install --with-deps chromium

      - name: Build app
        run: npm run build

      - name: Run visual tests
        run: npm run test:visual

      - name: Upload visual diff artifacts
        if: failure()
        uses: actions/upload-artifact@v4
        with:
          name: visual-diffs
          path: |
            e2e/visual/test-results/
            e2e/visual/visual-tests/snapshots/

      - name: Comment on PR with visual changes
        if: failure()
        uses: actions/github-script@v7
        with:
          script: |
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: '## Visual Regression Detected\n\nVisual changes were detected. Please review the artifacts.'
            })
```

### 5. Update Script

Create `scripts/update-visual-snapshots.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

echo "Updating visual snapshots..."

# Run visual tests with update flag
npx playwright test --config=e2e/visual/playwright.visual.config.ts --update-snapshots

echo "Snapshots updated. Review changes with:"
echo "  git diff e2e/visual/visual-tests/snapshots/"
```

---

## Testing Requirements

1. All key views have baseline screenshots
2. Theme variations are covered
3. Responsive breakpoints are tested
4. CI detects visual regressions
5. Update workflow is documented

---

## Related Specs

- Depends on: [483-e2e-framework.md](483-e2e-framework.md)
- Next: [486-benchmarks.md](486-benchmarks.md)
- Related: [193-color-system.md](../phase-09-ui-foundation/193-color-system.md)
