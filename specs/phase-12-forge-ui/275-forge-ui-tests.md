# Spec 275: Forge UI Tests

## Header
- **Spec ID**: 275
- **Phase**: 12 - Forge UI
- **Component**: Forge UI Tests
- **Dependencies**: Specs 256-274
- **Status**: Draft

## Objective
Create comprehensive test suites for all Forge UI components, ensuring reliability, accessibility, performance, and visual consistency across the deliberation interface.

## Acceptance Criteria
1. Unit tests achieve 90%+ code coverage for all components
2. Integration tests cover all user workflows
3. E2E tests validate critical paths
4. Visual regression tests prevent UI breakage
5. Accessibility tests ensure WCAG 2.1 AA compliance
6. Performance tests meet defined thresholds
7. Cross-browser tests validate compatibility
8. Mobile responsiveness tests verify layouts

## Implementation

### Test Configuration (vitest.config.ts)
```typescript
import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
  plugins: [svelte({ hot: !process.env.VITEST })],
  test: {
    include: ['src/**/*.{test,spec}.{js,ts}'],
    environment: 'jsdom',
    setupFiles: ['./src/tests/setup.ts'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      exclude: [
        'node_modules/',
        'src/tests/',
        '**/*.d.ts',
        '**/*.config.*'
      ],
      thresholds: {
        lines: 90,
        functions: 90,
        branches: 85,
        statements: 90
      }
    },
    globals: true
  }
});
```

### Test Setup (setup.ts)
```typescript
import '@testing-library/jest-dom';
import { vi } from 'vitest';
import { readable, writable } from 'svelte/store';

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
    dispatchEvent: vi.fn()
  }))
});

// Mock ResizeObserver
global.ResizeObserver = vi.fn().mockImplementation(() => ({
  observe: vi.fn(),
  unobserve: vi.fn(),
  disconnect: vi.fn()
}));

// Mock IntersectionObserver
global.IntersectionObserver = vi.fn().mockImplementation(() => ({
  observe: vi.fn(),
  unobserve: vi.fn(),
  disconnect: vi.fn()
}));

// Mock crypto.randomUUID
Object.defineProperty(global.crypto, 'randomUUID', {
  value: () => 'test-uuid-' + Math.random().toString(36).substr(2, 9)
});
```

### ForgeLayout Tests (ForgeLayout.test.ts)
```typescript
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import ForgeLayout from '$lib/components/forge/ForgeLayout.svelte';
import { forgeSessionStore } from '$lib/stores/forgeSession';
import { layoutPreferencesStore } from '$lib/stores/layoutPreferences';

vi.mock('$lib/stores/forgeSession');
vi.mock('$lib/stores/layoutPreferences');

describe('ForgeLayout', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    layoutPreferencesStore.load = vi.fn().mockResolvedValue(null);
    layoutPreferencesStore.save = vi.fn();
  });

  it('renders with default layout configuration', () => {
    render(ForgeLayout, { props: { sessionId: null } });

    expect(screen.getByTestId('forge-layout')).toBeInTheDocument();
    expect(screen.getByRole('main')).toBeInTheDocument();
  });

  it('toggles left sidebar visibility with keyboard shortcut', async () => {
    render(ForgeLayout, { props: { sessionId: 'test-session' } });

    const leftSidebar = screen.getByLabelText('Session sidebar');
    expect(leftSidebar).toBeVisible();

    await fireEvent.keyDown(window, { key: 'b', ctrlKey: true });

    await waitFor(() => {
      expect(screen.queryByLabelText('Session sidebar')).not.toBeInTheDocument();
    });
  });

  it('persists layout preferences on panel resize', async () => {
    render(ForgeLayout, { props: { sessionId: 'test-session' } });

    const resizeHandle = screen.getByRole('separator', { name: /resize left sidebar/i });

    await fireEvent.mouseDown(resizeHandle);
    await fireEvent.mouseMove(document, { clientX: 350 });
    await fireEvent.mouseUp(document);

    expect(layoutPreferencesStore.save).toHaveBeenCalledWith(
      'forge',
      expect.objectContaining({ leftSidebarWidth: expect.any(Number) })
    );
  });

  it('adapts layout for mobile viewports', async () => {
    window.innerWidth = 600;
    window.dispatchEvent(new Event('resize'));

    render(ForgeLayout, { props: { sessionId: 'test-session' } });

    // Sidebars should be hidden on mobile by default
    await waitFor(() => {
      const leftSidebar = screen.queryByLabelText('Session sidebar');
      expect(leftSidebar).toHaveAttribute('data-visible', 'false');
    });
  });
});
```

### SessionCreation Tests (SessionCreation.test.ts)
```typescript
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import SessionCreationWizard from '$lib/components/forge/SessionCreationWizard.svelte';
import { forgeSessionStore } from '$lib/stores/forgeSession';

vi.mock('$lib/stores/forgeSession');

describe('SessionCreationWizard', () => {
  const user = userEvent.setup();

  beforeEach(() => {
    vi.clearAllMocks();
    forgeSessionStore.createSession = vi.fn().mockResolvedValue('new-session-id');
    forgeSessionStore.saveDraft = vi.fn().mockResolvedValue('draft-id');
  });

  it('navigates through wizard steps', async () => {
    render(SessionCreationWizard, { props: { template: null } });

    // Step 1: Goal
    expect(screen.getByText('Define Goal')).toBeInTheDocument();

    const nameInput = screen.getByLabelText('Session Name');
    const goalInput = screen.getByLabelText('Session Goal');

    await user.type(nameInput, 'Test Session');
    await user.type(goalInput, 'This is a test goal for the deliberation session');

    const nextButton = screen.getByRole('button', { name: /next/i });
    await user.click(nextButton);

    // Step 2: Participants
    await waitFor(() => {
      expect(screen.getByText('Select Participants')).toBeInTheDocument();
    });
  });

  it('validates goal input before proceeding', async () => {
    render(SessionCreationWizard, { props: { template: null } });

    const nextButton = screen.getByRole('button', { name: /next/i });

    // Next should be disabled without valid goal
    expect(nextButton).toBeDisabled();

    const goalInput = screen.getByLabelText('Session Goal');
    await user.type(goalInput, 'Short');

    // Still disabled with too short goal
    expect(nextButton).toBeDisabled();

    await user.clear(goalInput);
    await user.type(goalInput, 'This is a sufficiently long goal description');

    // Now should be enabled
    await waitFor(() => {
      expect(nextButton).not.toBeDisabled();
    });
  });

  it('calculates cost estimate based on selections', async () => {
    render(SessionCreationWizard, { props: { template: null } });

    // Navigate to participants step
    const nameInput = screen.getByLabelText('Session Name');
    const goalInput = screen.getByLabelText('Session Goal');
    await user.type(nameInput, 'Test Session');
    await user.type(goalInput, 'This is a test goal for the deliberation session');
    await user.click(screen.getByRole('button', { name: /next/i }));

    await waitFor(() => {
      expect(screen.getByText(/estimated cost/i)).toBeInTheDocument();
    });
  });

  it('saves draft and dispatches event', async () => {
    const { component } = render(SessionCreationWizard, { props: { template: null } });

    const savedHandler = vi.fn();
    component.$on('saved', savedHandler);

    const saveDraftButton = screen.getByRole('button', { name: /save draft/i });
    await user.click(saveDraftButton);

    await waitFor(() => {
      expect(forgeSessionStore.saveDraft).toHaveBeenCalled();
      expect(savedHandler).toHaveBeenCalledWith(
        expect.objectContaining({ detail: { draftId: 'draft-id' } })
      );
    });
  });
});
```

### Convergence Indicator Tests (ConvergenceIndicator.test.ts)
```typescript
import { render, screen, waitFor } from '@testing-library/svelte';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import ConvergenceIndicator from '$lib/components/forge/ConvergenceIndicator.svelte';
import { convergenceService } from '$lib/services/convergenceService';

vi.mock('$lib/services/convergenceService');

describe('ConvergenceIndicator', () => {
  const mockMetrics = {
    overall: 0.75,
    topicScores: [
      { topic: 'Architecture', score: 0.8 },
      { topic: 'Security', score: 0.7 }
    ],
    participantAlignments: [],
    agreementCount: 5,
    disagreementCount: 2
  };

  beforeEach(() => {
    vi.clearAllMocks();
    convergenceService.getMetrics = vi.fn().mockResolvedValue({
      metrics: mockMetrics,
      history: []
    });
    convergenceService.subscribe = vi.fn().mockReturnValue(() => {});
  });

  it('displays convergence score', async () => {
    render(ConvergenceIndicator, {
      props: { sessionId: 'test-session', threshold: 0.8 }
    });

    await waitFor(() => {
      expect(screen.getByText('75%')).toBeInTheDocument();
    });
  });

  it('shows correct status label based on threshold proximity', async () => {
    render(ConvergenceIndicator, {
      props: { sessionId: 'test-session', threshold: 0.8 }
    });

    await waitFor(() => {
      expect(screen.getByText(/near convergence/i)).toBeInTheDocument();
    });
  });

  it('renders compact view when specified', async () => {
    render(ConvergenceIndicator, {
      props: { sessionId: 'test-session', threshold: 0.8, compact: true }
    });

    await waitFor(() => {
      const indicator = screen.getByTestId('convergence-indicator');
      expect(indicator).toHaveClass('compact');
    });
  });

  it('animates score changes smoothly', async () => {
    const { component } = render(ConvergenceIndicator, {
      props: { sessionId: 'test-session', threshold: 0.8 }
    });

    // Simulate update
    const updateCallback = convergenceService.subscribe.mock.calls[0][1];
    updateCallback({
      roundNumber: 2,
      metrics: { ...mockMetrics, overall: 0.85 }
    });

    await waitFor(() => {
      expect(screen.getByText('85%')).toBeInTheDocument();
    }, { timeout: 1000 });
  });
});
```

### E2E Tests (forge.e2e.ts)
```typescript
import { test, expect } from '@playwright/test';

test.describe('Forge UI E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/forge');
  });

  test('complete session creation workflow', async ({ page }) => {
    // Click new session button
    await page.click('[data-testid="new-session-btn"]');

    // Fill in goal
    await page.fill('[data-testid="session-name-input"]', 'E2E Test Session');
    await page.fill('[data-testid="goal-textarea"]', 'Test goal for E2E testing workflow');
    await page.click('button:has-text("Next")');

    // Select participants
    await page.waitForSelector('[data-testid="participant-select"]');
    await page.click('[data-testid="brain-card"]:first-child');
    await page.click('[data-testid="brain-card"]:nth-child(2)');
    await page.click('button:has-text("Next")');

    // Select oracle
    await page.waitForSelector('[data-testid="oracle-select"]');
    await page.click('[data-testid="oracle-card"]:first-child');
    await page.click('button:has-text("Next")');

    // Configure session
    await page.waitForSelector('[data-testid="session-config"]');
    await page.click('button:has-text("Next")');

    // Review and create
    await page.waitForSelector('[data-testid="session-review"]');
    await page.click('button:has-text("Start Session")');

    // Verify session created
    await expect(page.locator('[data-testid="session-active"]')).toBeVisible();
  });

  test('session controls work correctly', async ({ page }) => {
    // Navigate to existing session
    await page.goto('/forge/session/test-session');

    // Verify initial state
    await expect(page.locator('[data-testid="session-controls"]')).toBeVisible();

    // Pause session
    await page.click('button:has-text("Pause")');
    await expect(page.locator('.state-label')).toContainText('Paused');

    // Resume session
    await page.click('button:has-text("Resume")');
    await expect(page.locator('.state-label')).toContainText('Running');
  });

  test('human intervention workflow', async ({ page }) => {
    await page.goto('/forge/session/test-session');

    // Open intervention panel
    await page.click('[data-testid="intervention-panel-toggle"]');

    // Select guidance type
    await page.click('[data-testid="intervention-type-guidance"]');

    // Fill in intervention
    await page.fill('[data-testid="intervention-content"]', 'Test intervention content');

    // Submit
    await page.click('button:has-text("Submit")');

    // Verify submission
    await expect(page.locator('[data-testid="intervention-success"]')).toBeVisible();
  });
});
```

### Accessibility Tests (accessibility.test.ts)
```typescript
import { render } from '@testing-library/svelte';
import { axe, toHaveNoViolations } from 'jest-axe';
import { describe, it, expect } from 'vitest';
import ForgeLayout from '$lib/components/forge/ForgeLayout.svelte';
import SessionCreationWizard from '$lib/components/forge/SessionCreationWizard.svelte';
import RoundVisualization from '$lib/components/forge/RoundVisualization.svelte';

expect.extend(toHaveNoViolations);

describe('Accessibility Tests', () => {
  it('ForgeLayout has no accessibility violations', async () => {
    const { container } = render(ForgeLayout, { props: { sessionId: null } });
    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it('SessionCreationWizard has no accessibility violations', async () => {
    const { container } = render(SessionCreationWizard, { props: { template: null } });
    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it('RoundVisualization has no accessibility violations', async () => {
    const { container } = render(RoundVisualization, {
      props: { sessionId: 'test-session' }
    });
    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it('all interactive elements are keyboard accessible', async () => {
    const { container } = render(ForgeLayout, { props: { sessionId: 'test-session' } });

    const focusableElements = container.querySelectorAll(
      'button, a, input, select, textarea, [tabindex]:not([tabindex="-1"])'
    );

    focusableElements.forEach(element => {
      expect(element).toHaveAttribute('tabindex');
      // Or be naturally focusable
      expect(
        ['BUTTON', 'A', 'INPUT', 'SELECT', 'TEXTAREA'].includes(element.tagName) ||
        element.getAttribute('tabindex') !== '-1'
      ).toBe(true);
    });
  });
});
```

### Visual Regression Tests (visual.test.ts)
```typescript
import { test, expect } from '@playwright/test';

test.describe('Visual Regression Tests', () => {
  test('forge layout matches snapshot', async ({ page }) => {
    await page.goto('/forge');
    await page.waitForSelector('[data-testid="forge-layout"]');

    await expect(page).toHaveScreenshot('forge-layout.png', {
      fullPage: true,
      animations: 'disabled'
    });
  });

  test('session creation wizard matches snapshot', async ({ page }) => {
    await page.goto('/forge/new');
    await page.waitForSelector('[data-testid="session-creation-wizard"]');

    await expect(page).toHaveScreenshot('session-wizard.png', {
      animations: 'disabled'
    });
  });

  test('convergence indicator states', async ({ page }) => {
    // Test different convergence states
    for (const score of [0.3, 0.6, 0.8, 1.0]) {
      await page.goto(`/forge/test?convergence=${score}`);
      await page.waitForSelector('[data-testid="convergence-indicator"]');

      await expect(page.locator('[data-testid="convergence-indicator"]'))
        .toHaveScreenshot(`convergence-${score * 100}.png`);
    }
  });
});
```

## Testing Requirements
1. **Unit Tests**: All components must have 90%+ coverage
2. **Integration Tests**: All user workflows must be tested
3. **E2E Tests**: Critical paths must have automated tests
4. **Accessibility Tests**: WCAG 2.1 AA compliance required
5. **Visual Tests**: All major views must have snapshot tests

## Related Specs
- Specs 256-274: All Forge UI Components
- Spec 295: Settings Tests (Phase 13)
