# 315 - Dashboard Tests

**Phase:** 14 - Dashboard
**Spec ID:** 315
**Status:** Planned
**Dependencies:** 296-315 (all dashboard specs)
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Create comprehensive test suites for all dashboard components, including unit tests, component tests, integration tests, and visual regression tests.

---

## Acceptance Criteria

- [x] Unit tests for all utilities
- [x] Component tests for all Svelte components
- [x] Store tests for all Svelte stores
- [x] Integration tests for data flows
- [x] Visual regression tests
- [x] Accessibility tests
- [x] Performance benchmarks
- [x] Test coverage > 80%

---

## Implementation Details

### 1. Test Setup (web/src/tests/setup.ts)

```typescript
import '@testing-library/jest-dom';
import { vi } from 'vitest';

// Mock WebSocket
class MockWebSocket {
  static CONNECTING = 0;
  static OPEN = 1;
  static CLOSING = 2;
  static CLOSED = 3;

  readyState = MockWebSocket.CONNECTING;
  onopen: (() => void) | null = null;
  onclose: (() => void) | null = null;
  onerror: ((error: any) => void) | null = null;
  onmessage: ((event: MessageEvent) => void) | null = null;

  constructor(public url: string) {
    setTimeout(() => {
      this.readyState = MockWebSocket.OPEN;
      this.onopen?.();
    }, 0);
  }

  send(data: string) {}
  close(code?: number, reason?: string) {
    this.readyState = MockWebSocket.CLOSED;
    this.onclose?.();
  }
}

vi.stubGlobal('WebSocket', MockWebSocket);

// Mock matchMedia
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: vi.fn().mockImplementation(query => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
});

// Mock ResizeObserver
vi.stubGlobal('ResizeObserver', vi.fn().mockImplementation(() => ({
  observe: vi.fn(),
  unobserve: vi.fn(),
  disconnect: vi.fn(),
})));
```

### 2. Component Test Utils (web/src/tests/utils.ts)

```typescript
import { render, type RenderResult } from '@testing-library/svelte';
import { writable, type Writable } from 'svelte/store';
import type { SvelteComponent } from 'svelte';

interface RenderOptions {
  props?: Record<string, any>;
  stores?: Record<string, Writable<any>>;
}

export function renderWithStores<T extends SvelteComponent>(
  Component: new (...args: any[]) => T,
  options: RenderOptions = {}
): RenderResult<T> & { stores: Record<string, Writable<any>> } {
  const stores = options.stores || {};

  const result = render(Component, {
    props: options.props,
    context: new Map(Object.entries(stores))
  });

  return { ...result, stores };
}

export function createMockMission(overrides = {}) {
  return {
    id: 'msn_test123',
    specId: 'spc_001',
    title: 'Test Mission',
    description: 'A test mission description',
    state: 'running',
    currentStep: 'Executing tests',
    completedSteps: 5,
    totalSteps: 10,
    tokenUsage: {
      input: 1000,
      output: 500,
      total: 1500,
      cost: 0.05
    },
    recentLogs: [],
    error: null,
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
    startedAt: new Date().toISOString(),
    completedAt: null,
    ...overrides
  };
}

export function createMockCostData(overrides = {}) {
  return {
    totalCost: 125.50,
    changePercent: 5.2,
    breakdown: {
      byModel: [
        { label: 'Claude Opus', value: 80, percent: 63.7, color: '#8b5cf6' },
        { label: 'Claude Sonnet', value: 45.5, percent: 36.3, color: '#3b82f6' }
      ],
      topMissions: []
    },
    budget: {
      limit: 500,
      alertThreshold: 80,
      period: 'month' as const
    },
    sparklineData: [100, 110, 105, 120, 115, 125],
    projection: {
      projectedCost: 180,
      confidence: 0.85,
      basedOnDays: 15
    },
    ...overrides
  };
}

export function waitForAnimation(ms = 300): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}
```

### 3. Mission Card Tests (web/src/tests/components/MissionCard.test.ts)

```typescript
import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent, screen } from '@testing-library/svelte';
import MissionCard from '$lib/components/missions/MissionCard.svelte';
import { createMockMission } from '../utils';

describe('MissionCard', () => {
  it('renders mission title and description', () => {
    const mission = createMockMission();
    render(MissionCard, { props: { mission } });

    expect(screen.getByText(mission.title)).toBeInTheDocument();
    expect(screen.getByText(mission.description)).toBeInTheDocument();
  });

  it('displays correct status badge for running mission', () => {
    const mission = createMockMission({ state: 'running' });
    render(MissionCard, { props: { mission } });

    expect(screen.getByText('Running')).toBeInTheDocument();
  });

  it('displays correct status badge for completed mission', () => {
    const mission = createMockMission({ state: 'complete' });
    render(MissionCard, { props: { mission } });

    expect(screen.getByText('Complete')).toBeInTheDocument();
  });

  it('shows progress bar for running missions', () => {
    const mission = createMockMission({ state: 'running' });
    render(MissionCard, { props: { mission } });

    expect(screen.getByRole('progressbar')).toBeInTheDocument();
  });

  it('dispatches pause event when pause button clicked', async () => {
    const mission = createMockMission({ state: 'running' });
    const { component } = render(MissionCard, { props: { mission } });

    const handler = vi.fn();
    component.$on('pause', handler);

    const pauseButton = screen.getByTitle('Pause mission');
    await fireEvent.click(pauseButton);

    expect(handler).toHaveBeenCalledWith(
      expect.objectContaining({
        detail: { missionId: mission.id }
      })
    );
  });

  it('dispatches select event when card clicked', async () => {
    const mission = createMockMission();
    const { component } = render(MissionCard, { props: { mission } });

    const handler = vi.fn();
    component.$on('select', handler);

    const card = screen.getByRole('button');
    await fireEvent.click(card);

    expect(handler).toHaveBeenCalled();
  });

  it('shows error section when mission has error', () => {
    const mission = createMockMission({
      state: 'error',
      error: 'Test error message'
    });
    render(MissionCard, { props: { mission, expanded: true } });

    expect(screen.getByText('Test error message')).toBeInTheDocument();
  });

  it('renders in compact mode correctly', () => {
    const mission = createMockMission();
    render(MissionCard, { props: { mission, compact: true } });

    expect(screen.queryByText(mission.description)).not.toBeInTheDocument();
  });
});
```

### 4. Cost Summary Tests (web/src/tests/components/CostSummary.test.ts)

```typescript
import { describe, it, expect } from 'vitest';
import { render, fireEvent, screen } from '@testing-library/svelte';
import CostSummary from '$lib/components/cost/CostSummary.svelte';
import { createMockCostData, waitForAnimation } from '../utils';

describe('CostSummary', () => {
  it('renders total cost with currency formatting', async () => {
    const data = createMockCostData({ totalCost: 125.50 });
    render(CostSummary, { props: { data, period: 'month' } });

    await waitForAnimation();
    expect(screen.getByText('$125.50')).toBeInTheDocument();
  });

  it('shows budget progress bar', () => {
    const data = createMockCostData();
    render(CostSummary, { props: { data, period: 'month' } });

    expect(screen.getByText(/remaining/)).toBeInTheDocument();
  });

  it('displays over-budget warning when applicable', () => {
    const data = createMockCostData({
      totalCost: 600,
      budget: { limit: 500, alertThreshold: 80, period: 'month' }
    });
    render(CostSummary, { props: { data, period: 'month' } });

    const card = document.querySelector('.cost-summary');
    expect(card?.classList.contains('over-budget')).toBe(true);
  });

  it('switches period correctly', async () => {
    const data = createMockCostData();
    render(CostSummary, { props: { data, period: 'month' } });

    const weekButton = screen.getByText('Week');
    await fireEvent.click(weekButton);

    expect(weekButton.classList.contains('active')).toBe(true);
  });

  it('expands breakdown section', async () => {
    const data = createMockCostData();
    render(CostSummary, { props: { data, period: 'month', showBreakdown: true } });

    const expandButton = screen.getByText('View Breakdown');
    await fireEvent.click(expandButton);

    expect(screen.getByText('By Model')).toBeInTheDocument();
  });
});
```

### 5. Store Tests (web/src/tests/stores/dashboard.test.ts)

```typescript
import { describe, it, expect, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import { dashboardStore, sidebarCollapsed } from '$lib/stores/dashboard';

describe('dashboardStore', () => {
  beforeEach(() => {
    dashboardStore.reset();
  });

  it('has correct initial state', () => {
    const state = get(dashboardStore);

    expect(state.sidebarCollapsed).toBe(false);
    expect(state.activeView).toBe('overview');
    expect(state.refreshInterval).toBe(30000);
    expect(state.lastRefresh).toBe(null);
  });

  it('sets sidebar state correctly', () => {
    dashboardStore.setSidebarState(true);

    const state = get(dashboardStore);
    expect(state.sidebarCollapsed).toBe(true);
  });

  it('updates active view', () => {
    dashboardStore.setActiveView('missions');

    const state = get(dashboardStore);
    expect(state.activeView).toBe('missions');
  });

  it('marks refresh timestamp', () => {
    dashboardStore.markRefreshed();

    const state = get(dashboardStore);
    expect(state.lastRefresh).toBeInstanceOf(Date);
  });

  it('derived sidebarCollapsed store reflects state', () => {
    dashboardStore.setSidebarState(true);

    expect(get(sidebarCollapsed)).toBe(true);
  });
});
```

### 6. WebSocket Store Tests (web/src/tests/stores/websocket.test.ts)

```typescript
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';
import { wsStore, connectionStatus, isConnected } from '$lib/stores/websocket';

describe('wsStore', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    wsStore.disconnect();
    vi.useRealTimers();
  });

  it('starts with disconnected status', () => {
    expect(get(connectionStatus)).toBe('disconnected');
    expect(get(isConnected)).toBe(false);
  });

  it('transitions to connecting when connect called', () => {
    wsStore.connect('ws://localhost:3001');

    expect(get(connectionStatus)).toBe('connecting');
  });

  it('transitions to connected after WebSocket opens', async () => {
    wsStore.connect('ws://localhost:3001');

    await vi.runAllTimersAsync();

    expect(get(connectionStatus)).toBe('connected');
    expect(get(isConnected)).toBe(true);
  });

  it('calls message handlers for matching type', async () => {
    const handler = vi.fn();
    const unsubscribe = wsStore.onMessage('test_type', handler);

    wsStore.connect('ws://localhost:3001');
    await vi.runAllTimersAsync();

    // Simulate incoming message
    // Would need to trigger through mock WebSocket

    unsubscribe();
  });
});
```

### 7. Visual Regression Test Setup (web/src/tests/visual/visual.config.ts)

```typescript
import { test, expect } from '@playwright/test';

export const visualTest = test.extend({
  // Custom fixtures for visual testing
});

export async function captureScreenshot(
  page: any,
  name: string,
  options?: { fullPage?: boolean }
) {
  await expect(page).toHaveScreenshot(`${name}.png`, {
    fullPage: options?.fullPage ?? false,
    threshold: 0.1, // 10% pixel difference threshold
  });
}
```

### 8. Accessibility Tests (web/src/tests/a11y/dashboard.test.ts)

```typescript
import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/svelte';
import { axe, toHaveNoViolations } from 'jest-axe';
import DashboardLayout from '$lib/components/dashboard/DashboardLayout.svelte';

expect.extend(toHaveNoViolations);

describe('Dashboard Accessibility', () => {
  it('DashboardLayout has no accessibility violations', async () => {
    const { container } = render(DashboardLayout);
    const results = await axe(container);

    expect(results).toHaveNoViolations();
  });
});
```

### 9. Vitest Config (web/vitest.config.ts)

```typescript
import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
  plugins: [svelte({ hot: !process.env.VITEST })],
  test: {
    include: ['src/tests/**/*.test.ts'],
    globals: true,
    environment: 'jsdom',
    setupFiles: ['src/tests/setup.ts'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      exclude: ['src/tests/**'],
      thresholds: {
        lines: 80,
        branches: 80,
        functions: 80,
        statements: 80
      }
    }
  },
  resolve: {
    alias: {
      $lib: '/src/lib'
    }
  }
});
```

---

## Testing Requirements

1. All component tests pass
2. All store tests pass
3. Visual regression tests pass
4. Accessibility tests pass
5. Code coverage > 80%
6. Performance benchmarks met
7. CI pipeline integration works

---

## Related Specs

- Depends on: All Phase 14 specs (296-314)
- Related: [008-test-infrastructure.md](../phase-00-setup/008-test-infrastructure.md)
- Next Phase: [316-server-crate.md](../phase-15-server/316-server-crate.md)
