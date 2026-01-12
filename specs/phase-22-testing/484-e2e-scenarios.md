# 484 - E2E Test Scenarios

**Phase:** 22 - Testing Infrastructure
**Spec ID:** 484
**Status:** Planned
**Dependencies:** 483-e2e-framework
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Define comprehensive end-to-end test scenarios covering critical user journeys, edge cases, and integration points across the Tachikoma application.

---

## Acceptance Criteria

- [ ] Core user journey tests implemented
- [ ] Error handling scenarios covered
- [ ] Multi-step workflow tests
- [ ] Cross-feature integration tests
- [ ] Performance boundary tests
- [ ] Accessibility checks included

---

## Implementation Details

### 1. Core User Journey Tests

Create `e2e/tests/journeys/onboarding.spec.ts`:

```typescript
import { test, expect } from '../../fixtures';

test.describe('User Onboarding Journey', () => {
  test('should complete first-time setup', async ({ electronApp }) => {
    const { page } = electronApp;

    // Step 1: Welcome screen
    await expect(page.getByTestId('welcome-screen')).toBeVisible();
    await page.getByTestId('get-started-btn').click();

    // Step 2: API key configuration
    await expect(page.getByTestId('api-config-screen')).toBeVisible();
    await page.getByTestId('api-key-input').fill('sk-ant-test-key');
    await page.getByTestId('validate-key-btn').click();

    // Wait for validation
    await expect(page.getByTestId('key-valid-indicator')).toBeVisible();
    await page.getByTestId('continue-btn').click();

    // Step 3: Workspace selection
    await expect(page.getByTestId('workspace-screen')).toBeVisible();
    await page.getByTestId('select-workspace-btn').click();

    // Step 4: Complete
    await expect(page.getByTestId('setup-complete')).toBeVisible();
    await page.getByTestId('start-using-btn').click();

    // Verify landed on dashboard
    await expect(page.getByTestId('dashboard')).toBeVisible();
  });

  test('should handle invalid API key gracefully', async ({ electronApp }) => {
    const { page } = electronApp;

    await page.getByTestId('get-started-btn').click();
    await page.getByTestId('api-key-input').fill('invalid-key');
    await page.getByTestId('validate-key-btn').click();

    await expect(page.getByTestId('error-message')).toContainText('Invalid API key');
    await expect(page.getByTestId('continue-btn')).toBeDisabled();
  });
});
```

Create `e2e/tests/journeys/mission-complete.spec.ts`:

```typescript
import { test, expect } from '../../fixtures';

test.describe('Complete Mission Journey', () => {
  test.beforeEach(async ({ electronApp }) => {
    // Skip onboarding for these tests
    await electronApp.page.evaluate(() => {
      localStorage.setItem('onboarding_complete', 'true');
    });
    await electronApp.page.reload();
  });

  test('should complete a full mission cycle', async ({ missionPage }) => {
    await missionPage.goto();

    // Create mission
    await missionPage.createNewMission();
    await missionPage.setPrompt('Create a simple test function');
    await missionPage.selectBackend('mock');

    // Start and monitor
    await missionPage.startMission();
    await missionPage.waitForMissionStart();

    // Verify progress indicators
    await expect(missionPage.progressBar).toBeVisible();
    await expect(missionPage.contextMeter).toBeVisible();

    // Wait for completion (with mock backend)
    await missionPage.waitForMissionComplete(30000);

    // Verify completion state
    await expect(missionPage.getByTestId('mission-complete')).toBeVisible();
    await expect(missionPage.getByTestId('mission-summary')).toBeVisible();
  });

  test('should handle mission with tool calls', async ({ missionPage }) => {
    await missionPage.goto();
    await missionPage.createNewMission();
    await missionPage.setPrompt('Read and modify a file');
    await missionPage.selectBackend('mock-with-tools');
    await missionPage.startMission();

    // Verify tool calls appear in log
    await missionPage.page.waitForSelector('[data-testid="tool-call-read_file"]');
    await missionPage.page.waitForSelector('[data-testid="tool-call-edit_file"]');
  });

  test('should show and allow checkpoint approval in attended mode', async ({
    missionPage,
  }) => {
    await missionPage.goto();
    await missionPage.createNewMission();
    await missionPage.setPrompt('Make code changes');
    await missionPage.getByTestId('attended-mode-toggle').check();
    await missionPage.selectBackend('mock-with-checkpoints');
    await missionPage.startMission();

    // Wait for checkpoint
    const checkpoint = missionPage.getByTestId('checkpoint-approval');
    await expect(checkpoint).toBeVisible({ timeout: 30000 });

    // Approve checkpoint
    await checkpoint.getByTestId('approve-btn').click();

    // Mission continues
    await expect(missionPage.progressBar).toBeVisible();
  });
});
```

### 2. Error Handling Scenarios

Create `e2e/tests/scenarios/error-handling.spec.ts`:

```typescript
import { test, expect } from '../../fixtures';

test.describe('Error Handling Scenarios', () => {
  test('should handle network disconnection during mission', async ({
    missionPage,
    electronApp,
  }) => {
    await missionPage.goto();
    await missionPage.createNewMission();
    await missionPage.setPrompt('Long running task');
    await missionPage.selectBackend('mock-slow');
    await missionPage.startMission();

    // Simulate network disconnection
    await electronApp.page.context().setOffline(true);

    // Verify error handling
    await expect(missionPage.getByTestId('network-error')).toBeVisible({
      timeout: 10000,
    });
    await expect(missionPage.getByTestId('retry-btn')).toBeVisible();

    // Reconnect
    await electronApp.page.context().setOffline(false);
    await missionPage.getByTestId('retry-btn').click();

    // Mission should resume
    await expect(missionPage.progressBar).toBeVisible();
  });

  test('should handle rate limit errors', async ({ missionPage }) => {
    await missionPage.goto();
    await missionPage.createNewMission();
    await missionPage.setPrompt('Trigger rate limit');
    await missionPage.selectBackend('mock-rate-limited');
    await missionPage.startMission();

    // Verify rate limit notification
    await expect(missionPage.getByTestId('rate-limit-warning')).toBeVisible();
    await expect(missionPage.getByTestId('retry-countdown')).toBeVisible();
  });

  test('should handle context overflow', async ({ missionPage }) => {
    await missionPage.goto();
    await missionPage.createNewMission();
    await missionPage.setPrompt('Fill context window');
    await missionPage.selectBackend('mock-context-overflow');
    await missionPage.startMission();

    // Wait for redline warning
    await expect(missionPage.getByTestId('redline-warning')).toBeVisible({
      timeout: 30000,
    });

    // Verify auto-reboot option
    await expect(missionPage.getByTestId('reboot-btn')).toBeVisible();
  });

  test('should handle backend unavailable', async ({ missionPage }) => {
    await missionPage.goto();
    await missionPage.createNewMission();
    await missionPage.setPrompt('Test unavailable');
    await missionPage.selectBackend('unavailable-backend');
    await missionPage.startMission();

    await expect(missionPage.getByTestId('backend-error')).toContainText(
      'Backend unavailable'
    );
  });
});
```

### 3. Multi-Feature Integration Tests

Create `e2e/tests/scenarios/integration.spec.ts`:

```typescript
import { test, expect } from '../../fixtures';

test.describe('Cross-Feature Integration', () => {
  test('should create spec and use it in mission', async ({ electronApp }) => {
    const { page } = electronApp;

    // Navigate to spec browser
    await page.getByTestId('nav-specs').click();
    await expect(page.getByTestId('spec-browser')).toBeVisible();

    // Create new spec
    await page.getByTestId('new-spec-btn').click();
    await page.getByTestId('spec-title-input').fill('Test Feature');
    await page.getByTestId('spec-content-editor').fill('## Objective\n\nTest objective');
    await page.getByTestId('save-spec-btn').click();

    // Verify spec created
    await expect(page.getByTestId('spec-saved-toast')).toBeVisible();

    // Navigate to mission panel
    await page.getByTestId('nav-mission').click();
    await page.getByTestId('new-mission-btn').click();

    // Select the spec
    await page.getByTestId('select-spec-btn').click();
    await page.getByText('Test Feature').click();

    // Verify spec selected
    await expect(page.getByTestId('selected-spec')).toContainText('Test Feature');
  });

  test('should forge spec and track in dashboard', async ({ electronApp }) => {
    const { page } = electronApp;

    // Start forge session
    await page.getByTestId('nav-forge').click();
    await page.getByTestId('new-forge-btn').click();
    await page.getByTestId('forge-goal-input').fill('Design test system');
    await page.getByTestId('start-forge-btn').click();

    // Wait for forge to complete (mock backend)
    await expect(page.getByTestId('forge-complete')).toBeVisible({ timeout: 60000 });

    // Navigate to dashboard
    await page.getByTestId('nav-dashboard').click();

    // Verify forge session appears
    await expect(page.getByTestId('recent-forges')).toContainText('Design test system');
  });

  test('should track costs across missions', async ({ electronApp }) => {
    const { page } = electronApp;

    // Run multiple missions
    for (let i = 0; i < 3; i++) {
      await page.getByTestId('nav-mission').click();
      await page.getByTestId('new-mission-btn').click();
      await page.getByTestId('prompt-input').fill(`Mission ${i + 1}`);
      await page.getByTestId('backend-select').selectOption('mock');
      await page.getByTestId('start-mission-btn').click();
      await page.waitForSelector('[data-testid="mission-complete"]', {
        timeout: 30000,
      });
    }

    // Check dashboard costs
    await page.getByTestId('nav-dashboard').click();
    const costDisplay = page.getByTestId('total-cost');
    const costText = await costDisplay.textContent();

    expect(parseFloat(costText?.replace('$', '') || '0')).toBeGreaterThan(0);
  });
});
```

### 4. Performance Boundary Tests

Create `e2e/tests/scenarios/performance.spec.ts`:

```typescript
import { test, expect } from '../../fixtures';

test.describe('Performance Boundary Tests', () => {
  test('should handle large log output', async ({ missionPage }) => {
    await missionPage.goto();
    await missionPage.createNewMission();
    await missionPage.setPrompt('Generate verbose output');
    await missionPage.selectBackend('mock-verbose');
    await missionPage.startMission();

    // Wait for substantial logs
    await missionPage.page.waitForTimeout(5000);

    // Verify UI remains responsive
    const startTime = Date.now();
    await missionPage.getByTestId('log-scroll-bottom').click();
    const scrollTime = Date.now() - startTime;

    expect(scrollTime).toBeLessThan(1000); // Should scroll within 1s
  });

  test('should render large spec efficiently', async ({ electronApp }) => {
    const { page } = electronApp;

    await page.getByTestId('nav-specs').click();

    // Open a large spec (pre-loaded in test fixtures)
    await page.getByText('large-spec-fixture').click();

    // Measure render time
    const startTime = Date.now();
    await expect(page.getByTestId('spec-content')).toBeVisible();
    const renderTime = Date.now() - startTime;

    expect(renderTime).toBeLessThan(2000); // Should render within 2s
  });

  test('should maintain responsive UI during background operations', async ({
    missionPage,
  }) => {
    await missionPage.goto();
    await missionPage.createNewMission();
    await missionPage.setPrompt('Background task');
    await missionPage.selectBackend('mock');
    await missionPage.startMission();

    // Try interacting with other UI elements while mission runs
    const navButton = missionPage.page.getByTestId('nav-settings');

    const startTime = Date.now();
    await navButton.click();
    const clickTime = Date.now() - startTime;

    expect(clickTime).toBeLessThan(500); // Clicks should respond within 500ms
  });
});
```

### 5. Test Data Seeds

Create `e2e/tests/fixtures/test-data.ts`:

```typescript
/**
 * Test data seeds for E2E scenarios
 */

export const testUsers = {
  standard: {
    settings: {
      theme: 'dark',
      apiKeys: {
        claude: 'sk-ant-test-key',
      },
    },
  },
  admin: {
    settings: {
      theme: 'light',
      apiKeys: {
        claude: 'sk-ant-admin-key',
        openai: 'sk-openai-key',
      },
    },
  },
};

export const testSpecs = {
  simple: {
    id: '001',
    title: 'Simple Test',
    content: '# Simple Test\n\n## Objective\n\nBasic test spec.',
  },
  complex: {
    id: '002',
    title: 'Complex Test',
    content: `# Complex Test

## Objective

Multi-section test spec.

## Acceptance Criteria

- [ ] Criteria 1
- [ ] Criteria 2
- [ ] Criteria 3

## Implementation Details

Detailed implementation notes...
`,
  },
};

export async function seedTestData(page: import('@playwright/test').Page) {
  await page.evaluate((data) => {
    localStorage.setItem('test_users', JSON.stringify(data.users));
    localStorage.setItem('test_specs', JSON.stringify(data.specs));
  }, { users: testUsers, specs: testSpecs });
}
```

---

## Testing Requirements

1. All critical user journeys have test coverage
2. Error scenarios produce meaningful feedback
3. Integration tests verify cross-feature behavior
4. Performance tests catch regressions
5. Tests run reliably across platforms

---

## Related Specs

- Depends on: [483-e2e-framework.md](483-e2e-framework.md)
- Next: [485-visual-regression.md](485-visual-regression.md)
- Related: [489-flaky-tests.md](489-flaky-tests.md)
