# 235 - Mission UI Test Suite

**Phase:** 10 - Mission Panel UI
**Spec ID:** 235
**Status:** Planned
**Dependencies:** 216-235 (all Phase 10 specs)
**Estimated Context:** ~12% of Sonnet window

---

## Objective

Create a comprehensive test suite for all Mission Panel UI components, ensuring proper functionality, accessibility, and integration between components.

---

## Acceptance Criteria

- [ ] Unit tests for all components
- [ ] Integration tests for component interactions
- [ ] Accessibility tests (WCAG 2.1 AA)
- [ ] Visual regression tests
- [ ] Performance benchmarks
- [ ] E2E tests for critical flows

---

## Implementation Details

### 1. Test Configuration (src/lib/components/mission/__tests__/setup.ts)

```typescript
import { vi } from 'vitest';
import '@testing-library/jest-dom';

// Mock IPC
vi.mock('$lib/ipc', () => ({
  ipcRenderer: {
    invoke: vi.fn(),
    on: vi.fn(),
    removeListener: vi.fn(),
  },
}));

// Mock stores
export function createMockMission(overrides = {}) {
  return {
    id: 'test-mission-1',
    title: 'Test Mission',
    prompt: 'Test prompt',
    state: 'idle',
    specIds: [],
    backendId: 'claude-sonnet',
    mode: 'agentic',
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
    progress: {
      currentStep: 0,
      totalSteps: 0,
      currentAction: '',
      percentage: 0,
      contextUsage: {
        inputTokens: 0,
        outputTokens: 0,
        maxTokens: 200000,
        usagePercent: 0,
        isNearLimit: false,
        isRedlined: false,
      },
    },
    cost: { inputCost: 0, outputCost: 0, totalCost: 0, currency: 'USD' },
    checkpoints: [],
    tags: [],
    ...overrides,
  };
}

export function createMockBackend(overrides = {}) {
  return {
    id: 'claude-sonnet',
    name: 'Claude Sonnet',
    provider: 'anthropic',
    model: 'claude-3-sonnet',
    status: 'available',
    isDefault: true,
    capabilities: {
      maxContextTokens: 200000,
      maxOutputTokens: 4096,
      supportsVision: true,
      supportsTools: true,
      supportsStreaming: true,
      supportsJson: true,
    },
    pricing: {
      inputCostPer1k: 0.003,
      outputCostPer1k: 0.015,
      currency: 'USD',
    },
    lastChecked: new Date().toISOString(),
    ...overrides,
  };
}
```

### 2. Integration Tests (src/lib/components/mission/__tests__/integration.test.ts)

```typescript
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { ipcRenderer } from '$lib/ipc';
import MissionPanel from '../MissionPanel.svelte';
import MissionCreationDialog from '../MissionCreationDialog.svelte';
import { missionStore } from '$lib/stores/mission-store';
import { createMockMission, createMockBackend } from './setup';

describe('Mission Panel Integration', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(ipcRenderer.invoke).mockImplementation(async (channel, ...args) => {
      switch (channel) {
        case 'mission:list':
          return [createMockMission()];
        case 'backend:list':
          return [createMockBackend()];
        case 'spec:list':
          return [];
        default:
          return null;
      }
    });
  });

  it('loads and displays missions on mount', async () => {
    render(MissionPanel);

    await waitFor(() => {
      expect(screen.getByText('Test Mission')).toBeInTheDocument();
    });
  });

  it('creates a new mission through dialog', async () => {
    const mockMission = createMockMission({ id: 'new-1', title: 'New Mission' });
    vi.mocked(ipcRenderer.invoke).mockResolvedValueOnce(mockMission);

    const { component } = render(MissionCreationDialog, { open: true });

    // Skip template
    await fireEvent.click(screen.getByText('Skip and start from scratch'));

    // Fill in mission details
    await fireEvent.input(screen.getByLabelText('Mission Title'), {
      target: { value: 'New Mission' },
    });
    await fireEvent.input(screen.getByLabelText('Mission Prompt'), {
      target: { value: 'Create a new feature' },
    });

    // Navigate and complete
    await fireEvent.click(screen.getByText('Next'));
    // Continue through steps...

    expect(ipcRenderer.invoke).toHaveBeenCalledWith('mission:create', expect.any(Object));
  });

  it('updates mission state when IPC events received', async () => {
    render(MissionPanel);

    // Simulate IPC event
    const mockHandler = vi.mocked(ipcRenderer.on).mock.calls.find(
      call => call[0] === 'mission:event'
    )?.[1];

    if (mockHandler) {
      mockHandler(null, {
        type: 'mission:state-changed',
        payload: {
          missionId: 'test-mission-1',
          previousState: 'idle',
          newState: 'running',
          timestamp: new Date().toISOString(),
        },
      });
    }

    await waitFor(() => {
      expect(screen.getByText('Running')).toBeInTheDocument();
    });
  });
});

describe('Mission Controls Integration', () => {
  it('starts mission and updates UI', async () => {
    vi.mocked(ipcRenderer.invoke).mockResolvedValueOnce([
      createMockMission({ state: 'idle' }),
    ]);

    render(MissionPanel);

    await waitFor(() => {
      expect(screen.getByText('Start Mission')).toBeInTheDocument();
    });

    await fireEvent.click(screen.getByText('Start Mission'));

    expect(ipcRenderer.invoke).toHaveBeenCalledWith('mission:start', 'test-mission-1');
  });

  it('shows confirmation before aborting', async () => {
    vi.mocked(ipcRenderer.invoke).mockResolvedValueOnce([
      createMockMission({ state: 'running' }),
    ]);

    render(MissionPanel);

    await waitFor(() => {
      expect(screen.getByText('Abort')).toBeInTheDocument();
    });

    await fireEvent.click(screen.getByText('Abort'));

    expect(screen.getByText('Abort Mission?')).toBeInTheDocument();
  });
});
```

### 3. Accessibility Tests (src/lib/components/mission/__tests__/a11y.test.ts)

```typescript
import { render } from '@testing-library/svelte';
import { axe, toHaveNoViolations } from 'jest-axe';
import { describe, it, expect } from 'vitest';
import MissionPanel from '../MissionPanel.svelte';
import MissionControls from '../MissionControls.svelte';
import ProgressDisplay from '../ProgressDisplay.svelte';
import ContextMeter from '../ContextMeter.svelte';

expect.extend(toHaveNoViolations);

describe('Accessibility', () => {
  it('MissionPanel has no accessibility violations', async () => {
    const { container } = render(MissionPanel);
    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it('MissionControls has proper ARIA attributes', async () => {
    const { container } = render(MissionControls, {
      missionId: 'test-1',
    });

    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it('ProgressDisplay has accessible progress bar', async () => {
    const { container } = render(ProgressDisplay, {
      progress: {
        percentage: 50,
        currentStep: 2,
        totalSteps: 4,
        currentAction: 'Processing',
        elapsedMs: 30000,
        estimatedRemainingMs: 30000,
        stepsCompleted: [],
        isPaused: false,
        isIndeterminate: false,
      },
    });

    const progressbar = container.querySelector('[role="progressbar"]');
    expect(progressbar).toHaveAttribute('aria-valuenow', '50');
    expect(progressbar).toHaveAttribute('aria-valuemin', '0');
    expect(progressbar).toHaveAttribute('aria-valuemax', '100');

    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it('ContextMeter has proper meter semantics', async () => {
    const { container } = render(ContextMeter, {
      usage: {
        inputTokens: 50000,
        outputTokens: 10000,
        totalTokens: 60000,
        maxTokens: 200000,
        usagePercent: 30,
        zone: 'safe',
      },
    });

    const meter = container.querySelector('[role="meter"]');
    expect(meter).toHaveAttribute('aria-valuenow');
    expect(meter).toHaveAttribute('aria-label', 'Context window usage');
  });
});
```

### 4. Performance Tests (src/lib/components/mission/__tests__/performance.test.ts)

```typescript
import { render } from '@testing-library/svelte';
import { describe, it, expect } from 'vitest';
import LogViewer from '../LogViewer.svelte';
import HistoryView from '../HistoryView.svelte';

describe('Performance', () => {
  it('LogViewer handles 10000 entries efficiently', () => {
    const entries = Array.from({ length: 10000 }, (_, i) => ({
      id: `log-${i}`,
      timestamp: new Date().toISOString(),
      level: 'info' as const,
      message: `Log message ${i}`,
      source: 'test',
    }));

    const start = performance.now();
    render(LogViewer, { missionId: 'test-1' });
    const renderTime = performance.now() - start;

    // Should render in under 100ms
    expect(renderTime).toBeLessThan(100);
  });

  it('HistoryView handles 1000 missions efficiently', () => {
    const entries = Array.from({ length: 1000 }, (_, i) => ({
      id: `mission-${i}`,
      title: `Mission ${i}`,
      prompt: `Prompt ${i}`,
      state: 'complete',
      createdAt: new Date().toISOString(),
      completedAt: new Date().toISOString(),
      duration: 60000,
      cost: 0.10,
      tokensUsed: 10000,
      filesChanged: 5,
      tags: [],
    }));

    const start = performance.now();
    render(HistoryView, { entries });
    const renderTime = performance.now() - start;

    // Should render in under 50ms
    expect(renderTime).toBeLessThan(50);
  });
});
```

### 5. E2E Test Spec (e2e/mission-panel.spec.ts)

```typescript
import { test, expect } from '@playwright/test';

test.describe('Mission Panel E2E', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('complete mission creation flow', async ({ page }) => {
    // Open creation dialog
    await page.click('button:has-text("New Mission")');

    // Skip template
    await page.click('text=Skip and start from scratch');

    // Fill mission details
    await page.fill('[aria-label="Mission Title"]', 'E2E Test Mission');
    await page.fill('[aria-label="Mission Prompt"]', 'Test the E2E flow');

    // Navigate through steps
    await page.click('button:has-text("Next")');
    await page.click('button:has-text("Next")');

    // Select backend
    await page.click('text=Claude Sonnet');
    await page.click('button:has-text("Next")');

    // Review and create
    await page.click('button:has-text("Create Mission")');

    // Verify mission appears
    await expect(page.locator('text=E2E Test Mission')).toBeVisible();
  });

  test('mission execution flow', async ({ page }) => {
    // Assume a mission exists
    await page.click('text=Test Mission');

    // Start mission
    await page.click('button:has-text("Start Mission")');

    // Verify progress appears
    await expect(page.locator('[role="progressbar"]')).toBeVisible();

    // Verify logs start streaming
    await expect(page.locator('.log-entry')).toBeVisible({ timeout: 10000 });
  });

  test('checkpoint restoration', async ({ page }) => {
    await page.click('text=Test Mission');

    // Navigate to checkpoints
    await page.click('text=Checkpoints');

    // Select a checkpoint
    await page.click('.checkpoint-card >> nth=0');

    // Click restore
    await page.click('button:has-text("Restore")');

    // Confirm restoration
    await page.click('button:has-text("Restore")');

    // Verify success message
    await expect(page.locator('text=Checkpoint restored')).toBeVisible();
  });
});
```

---

## Testing Requirements

1. All component unit tests pass
2. Integration tests cover key flows
3. No accessibility violations
4. Performance within benchmarks
5. E2E tests pass in CI

---

## Related Specs

- Depends on: All Phase 10 specs (216-234)
- Next: [236-spec-browser-layout.md](../phase-11-spec-browser/236-spec-browser-layout.md)
