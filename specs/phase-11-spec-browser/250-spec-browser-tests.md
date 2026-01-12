# Spec 250: Spec Browser Component Tests

## Phase
11 - Spec Browser UI

## Spec ID
250

## Status
Planned

## Dependencies
- Specs 231-249 (All Phase 11 Components)
- Phase 10 (Core UI Components)

## Estimated Context
~12%

---

## Objective

Implement comprehensive test coverage for all Spec Browser components including unit tests, integration tests, accessibility tests, and visual regression tests. Establish testing patterns and utilities for the entire spec browser feature.

---

## Acceptance Criteria

- [ ] Unit tests for all components (>90% coverage)
- [ ] Integration tests for component interactions
- [ ] Accessibility tests (WCAG 2.1 AA compliance)
- [ ] Visual regression tests for key components
- [ ] Performance tests for large datasets
- [ ] Test utilities and mock data factories
- [ ] CI/CD integration for automated testing
- [ ] Test documentation and examples

---

## Implementation Details

### Test Setup and Configuration

```typescript
// vitest.config.ts
import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
  plugins: [svelte({ hot: !process.env.VITEST })],
  test: {
    include: ['src/**/*.{test,spec}.{js,ts}'],
    globals: true,
    environment: 'jsdom',
    setupFiles: ['./src/test/setup.ts'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      exclude: [
        'node_modules/',
        'src/test/',
        '**/*.d.ts',
        '**/*.config.*'
      ],
      thresholds: {
        statements: 90,
        branches: 85,
        functions: 90,
        lines: 90
      }
    },
    reporters: ['default', 'html'],
    outputFile: './test-results/index.html'
  },
  resolve: {
    alias: {
      $lib: '/src/lib',
      $app: '/src/test/mocks/app'
    }
  }
});
```

### Test Setup File

```typescript
// src/test/setup.ts
import '@testing-library/jest-dom/vitest';
import { vi } from 'vitest';
import { cleanup } from '@testing-library/svelte';

// Cleanup after each test
afterEach(() => {
  cleanup();
});

// Mock IntersectionObserver
class MockIntersectionObserver {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
}

window.IntersectionObserver = MockIntersectionObserver as any;

// Mock ResizeObserver
class MockResizeObserver {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
}

window.ResizeObserver = MockResizeObserver as any;

// Mock matchMedia
window.matchMedia = vi.fn().mockImplementation(query => ({
  matches: false,
  media: query,
  onchange: null,
  addListener: vi.fn(),
  removeListener: vi.fn(),
  addEventListener: vi.fn(),
  removeEventListener: vi.fn(),
  dispatchEvent: vi.fn()
}));

// Mock clipboard
Object.assign(navigator, {
  clipboard: {
    writeText: vi.fn().mockResolvedValue(undefined),
    readText: vi.fn().mockResolvedValue('')
  }
});

// Mock localStorage
const localStorageMock = {
  getItem: vi.fn(),
  setItem: vi.fn(),
  removeItem: vi.fn(),
  clear: vi.fn()
};
Object.defineProperty(window, 'localStorage', { value: localStorageMock });

// Mock scrollIntoView
Element.prototype.scrollIntoView = vi.fn();
```

### Test Utilities

```typescript
// src/test/utils/index.ts
import { render, type RenderResult } from '@testing-library/svelte';
import { readable, writable } from 'svelte/store';
import type { Spec, SpecStatus, SpecFilter } from '$lib/types/spec';

// Mock data factory
export function createMockSpec(overrides: Partial<Spec> = {}): Spec {
  const id = overrides.id || String(Math.floor(Math.random() * 1000));

  return {
    id,
    title: `Test Spec ${id}`,
    description: `Description for spec ${id}`,
    status: 'planned' as SpecStatus,
    phase: 1,
    dependencies: [],
    estimatedContext: '~10%',
    tags: [],
    content: `## Objective\n\nTest objective for spec ${id}.`,
    createdAt: new Date(),
    updatedAt: new Date(),
    author: 'Test Author',
    ...overrides
  };
}

export function createMockSpecs(count: number, overrides: Partial<Spec> = {}): Spec[] {
  return Array.from({ length: count }, (_, i) =>
    createMockSpec({ id: String(100 + i), ...overrides })
  );
}

export function createMockFilter(overrides: Partial<SpecFilter> = {}): SpecFilter {
  return {
    statuses: [],
    phases: [],
    phaseRange: null,
    tags: [],
    tagMode: 'any',
    hasDependencies: null,
    hasDependents: null,
    createdAfter: null,
    createdBefore: null,
    updatedAfter: null,
    updatedBefore: null,
    author: null,
    ...overrides
  };
}

// Custom render with providers
export function renderWithProviders(
  component: any,
  props: Record<string, any> = {},
  options: { specs?: Spec[] } = {}
): RenderResult<any> {
  // Could wrap with context providers if needed
  return render(component, { props });
}

// Wait for async operations
export function waitForAsync(ms: number = 0): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

// Fire custom event helper
export function fireCustomEvent(
  element: Element,
  eventName: string,
  detail: any = {}
): void {
  const event = new CustomEvent(eventName, { detail, bubbles: true });
  element.dispatchEvent(event);
}

// Accessibility testing helpers
export async function checkAccessibility(
  container: HTMLElement
): Promise<{ violations: any[]; passes: any[] }> {
  const axe = await import('axe-core');
  const results = await axe.default.run(container);

  return {
    violations: results.violations,
    passes: results.passes
  };
}

// Mock store factory
export function createMockStore<T>(initialValue: T) {
  const store = writable(initialValue);

  return {
    ...store,
    reset: () => store.set(initialValue)
  };
}
```

### Integration Test Suite

```typescript
// src/test/integration/spec-browser.test.ts
import { render, fireEvent, screen, waitFor, within } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import SpecBrowser from '$lib/components/SpecBrowser.svelte';
import { specStore } from '$lib/stores/spec-store';
import { createMockSpecs } from '../utils';

describe('SpecBrowser Integration', () => {
  const mockSpecs = createMockSpecs(50);

  beforeEach(() => {
    specStore.set(mockSpecs);
    localStorage.clear();
  });

  describe('List and Filter Integration', () => {
    it('filters specs by status', async () => {
      render(SpecBrowser);

      // Open filter panel
      await fireEvent.click(screen.getByText('Filters'));

      // Select "In Progress" status
      await fireEvent.click(screen.getByText('In Progress'));

      // Wait for filter to apply
      await waitFor(() => {
        const items = screen.getAllByRole('option');
        expect(items.every(item =>
          within(item).queryByText('in-progress')
        )).toBe(true);
      });
    });

    it('combines search and filters', async () => {
      render(SpecBrowser);

      // Enter search query
      const searchInput = screen.getByPlaceholderText('Search specs...');
      await fireEvent.input(searchInput, { target: { value: 'test' } });

      // Apply status filter
      await fireEvent.click(screen.getByText('Filters'));
      await fireEvent.click(screen.getByText('Planned'));

      // Results should match both criteria
      await waitFor(() => {
        const items = screen.getAllByRole('option');
        expect(items.length).toBeLessThan(mockSpecs.length);
      });
    });

    it('persists filter state in URL', async () => {
      render(SpecBrowser);

      await fireEvent.click(screen.getByText('Filters'));
      await fireEvent.click(screen.getByText('Implemented'));

      await waitFor(() => {
        expect(window.location.search).toContain('status=implemented');
      });
    });
  });

  describe('Detail View Integration', () => {
    it('opens detail view on spec click', async () => {
      render(SpecBrowser);

      const firstSpec = screen.getAllByRole('option')[0];
      await fireEvent.dblClick(firstSpec);

      await waitFor(() => {
        expect(screen.getByText('Spec 100 details')).toBeInTheDocument();
      });
    });

    it('navigates between specs with keyboard', async () => {
      render(SpecBrowser);

      // Open first spec
      const firstSpec = screen.getAllByRole('option')[0];
      await fireEvent.dblClick(firstSpec);

      // Navigate to next spec
      await fireEvent.keyDown(document.body, { key: 'ArrowRight', altKey: true });

      await waitFor(() => {
        expect(screen.getByText('Spec 101 details')).toBeInTheDocument();
      });
    });
  });

  describe('Editor Integration', () => {
    it('opens editor from detail view', async () => {
      render(SpecBrowser);

      // Open spec
      const firstSpec = screen.getAllByRole('option')[0];
      await fireEvent.dblClick(firstSpec);

      // Click edit button
      await fireEvent.click(screen.getByLabelText('Edit'));

      await waitFor(() => {
        expect(screen.getByText('Edit Spec 100')).toBeInTheDocument();
      });
    });

    it('saves changes and updates list', async () => {
      render(SpecBrowser);

      // Open spec and edit
      const firstSpec = screen.getAllByRole('option')[0];
      await fireEvent.dblClick(firstSpec);
      await fireEvent.click(screen.getByLabelText('Edit'));

      // Change title
      const titleInput = screen.getByLabelText('Title');
      await fireEvent.input(titleInput, { target: { value: 'Updated Title' } });

      // Save
      await fireEvent.click(screen.getByText('Save'));

      // Check list updated
      await waitFor(() => {
        expect(screen.getByText('Updated Title')).toBeInTheDocument();
      });
    });
  });

  describe('Batch Operations Integration', () => {
    it('selects multiple specs and updates status', async () => {
      render(SpecBrowser);

      // Select specs
      const specs = screen.getAllByRole('option');
      await fireEvent.click(specs[0]);
      await fireEvent.click(specs[1], { ctrlKey: true });
      await fireEvent.click(specs[2], { ctrlKey: true });

      // Batch action bar should appear
      expect(screen.getByText('3 of 50 selected')).toBeInTheDocument();

      // Change status
      await fireEvent.click(screen.getByText('Status'));
      await fireEvent.click(screen.getByText('In Progress'));

      // All selected should have new status
      await waitFor(() => {
        // Verify status changed
      });
    });
  });
});
```

### Accessibility Test Suite

```typescript
// src/test/accessibility/spec-browser-a11y.test.ts
import { render, screen } from '@testing-library/svelte';
import { describe, it, expect } from 'vitest';
import axe from 'axe-core';
import SpecListLayout from '$lib/components/SpecListLayout.svelte';
import SpecCard from '$lib/components/SpecCard.svelte';
import SpecDetailView from '$lib/components/SpecDetailView.svelte';
import { createMockSpecs, createMockSpec } from '../utils';

async function runAxe(container: HTMLElement) {
  const results = await axe.run(container);
  return results.violations;
}

describe('Accessibility Tests', () => {
  describe('SpecListLayout', () => {
    it('has no accessibility violations', async () => {
      const { container } = render(SpecListLayout, {
        props: { specs: createMockSpecs(5) }
      });

      const violations = await runAxe(container);
      expect(violations).toHaveLength(0);
    });

    it('has proper ARIA roles', () => {
      render(SpecListLayout, {
        props: { specs: createMockSpecs(5) }
      });

      expect(screen.getByRole('listbox')).toBeInTheDocument();
      expect(screen.getAllByRole('option').length).toBe(5);
    });

    it('supports keyboard navigation', async () => {
      render(SpecListLayout, {
        props: { specs: createMockSpecs(5) }
      });

      const list = screen.getByRole('listbox');
      expect(list).toHaveAttribute('tabindex', '0');
    });
  });

  describe('SpecCard', () => {
    it('has no accessibility violations', async () => {
      const { container } = render(SpecCard, {
        props: { spec: createMockSpec() }
      });

      const violations = await runAxe(container);
      expect(violations).toHaveLength(0);
    });

    it('has accessible action buttons', async () => {
      render(SpecCard, {
        props: { spec: createMockSpec() }
      });

      const card = screen.getByRole('option');
      await fireEvent.mouseEnter(card);

      expect(screen.getByLabelText('Edit spec')).toBeInTheDocument();
      expect(screen.getByLabelText('Duplicate spec')).toBeInTheDocument();
      expect(screen.getByLabelText('Delete spec')).toBeInTheDocument();
    });
  });

  describe('SpecDetailView', () => {
    it('has no accessibility violations', async () => {
      const { container } = render(SpecDetailView, {
        props: { spec: createMockSpec() }
      });

      const violations = await runAxe(container);
      expect(violations).toHaveLength(0);
    });

    it('has proper heading hierarchy', () => {
      render(SpecDetailView, {
        props: { spec: createMockSpec() }
      });

      const headings = screen.getAllByRole('heading');
      // Verify heading levels are in order
      const levels = headings.map(h =>
        parseInt(h.tagName.replace('H', ''), 10)
      );

      for (let i = 1; i < levels.length; i++) {
        expect(levels[i]).toBeLessThanOrEqual(levels[i - 1] + 1);
      }
    });

    it('has accessible collapsible sections', () => {
      render(SpecDetailView, {
        props: {
          spec: createMockSpec({
            content: '## Implementation Details\n\nContent here'
          })
        }
      });

      const buttons = screen.getAllByRole('button');
      const expandable = buttons.filter(b =>
        b.hasAttribute('aria-expanded')
      );

      expandable.forEach(button => {
        expect(button).toHaveAttribute('aria-expanded');
      });
    });
  });

  describe('Color Contrast', () => {
    it('status badges have sufficient contrast', async () => {
      const { container } = render(SpecCard, {
        props: {
          spec: createMockSpec({ status: 'planned' })
        }
      });

      const violations = await runAxe(container, {
        rules: ['color-contrast']
      });

      expect(violations).toHaveLength(0);
    });
  });

  describe('Focus Management', () => {
    it('traps focus in modal dialogs', async () => {
      // Test modal focus trap
    });

    it('restores focus after modal close', async () => {
      // Test focus restoration
    });
  });
});
```

### Performance Test Suite

```typescript
// src/test/performance/spec-browser-perf.test.ts
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';
import { describe, it, expect, beforeEach } from 'vitest';
import SpecListLayout from '$lib/components/SpecListLayout.svelte';
import DependencyGraph from '$lib/components/DependencyGraph.svelte';
import { createMockSpecs } from '../utils';

describe('Performance Tests', () => {
  describe('Large Dataset Handling', () => {
    it('renders 1000 specs without blocking', async () => {
      const largeDataset = createMockSpecs(1000);

      const startTime = performance.now();

      render(SpecListLayout, {
        props: {
          specs: largeDataset,
          virtualScroll: true
        }
      });

      const endTime = performance.now();
      const renderTime = endTime - startTime;

      // Should render in under 100ms with virtual scrolling
      expect(renderTime).toBeLessThan(100);
    });

    it('virtual scroll renders only visible items', () => {
      const largeDataset = createMockSpecs(1000);

      render(SpecListLayout, {
        props: {
          specs: largeDataset,
          virtualScroll: true
        }
      });

      const renderedItems = screen.getAllByRole('option');

      // Should only render visible + buffer items, not all 1000
      expect(renderedItems.length).toBeLessThan(50);
    });

    it('scroll performance remains stable', async () => {
      const largeDataset = createMockSpecs(1000);

      render(SpecListLayout, {
        props: {
          specs: largeDataset,
          virtualScroll: true
        }
      });

      const list = screen.getByRole('listbox');
      const measurements: number[] = [];

      for (let i = 0; i < 10; i++) {
        const startTime = performance.now();

        await fireEvent.scroll(list, {
          target: { scrollTop: i * 500 }
        });

        const endTime = performance.now();
        measurements.push(endTime - startTime);
      }

      const avgTime = measurements.reduce((a, b) => a + b, 0) / measurements.length;

      // Average scroll update should be under 16ms (60fps)
      expect(avgTime).toBeLessThan(16);
    });
  });

  describe('Search Performance', () => {
    it('search debounce prevents excessive updates', async () => {
      const specs = createMockSpecs(500);
      let updateCount = 0;

      render(SpecListLayout, {
        props: {
          specs,
          onFilterChange: () => updateCount++
        }
      });

      const searchInput = screen.getByPlaceholderText('Search specs...');

      // Type quickly
      for (const char of 'test query') {
        await fireEvent.input(searchInput, {
          target: { value: searchInput.value + char }
        });
      }

      // Should have debounced to fewer updates
      await waitFor(() => {
        expect(updateCount).toBeLessThan(5);
      }, { timeout: 500 });
    });
  });

  describe('Dependency Graph Performance', () => {
    it('renders complex graph without timeout', async () => {
      const specs = createMockSpecs(100).map((spec, i) => ({
        ...spec,
        dependencies: i > 0 ? [String(99 + i)] : []
      }));

      const startTime = performance.now();

      render(DependencyGraph, {
        props: { specs, spec: specs[0] }
      });

      const endTime = performance.now();

      // Graph should initialize in under 500ms
      expect(endTime - startTime).toBeLessThan(500);
    });
  });
});
```

### Visual Regression Tests

```typescript
// src/test/visual/spec-browser-visual.test.ts
import { test, expect } from '@playwright/test';

test.describe('Visual Regression Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/spec-browser');
    await page.waitForSelector('[data-testid="spec-list"]');
  });

  test('spec list layout matches snapshot', async ({ page }) => {
    await expect(page).toHaveScreenshot('spec-list-layout.png', {
      fullPage: true
    });
  });

  test('spec card matches snapshot', async ({ page }) => {
    // Switch to grid view
    await page.click('[data-testid="view-mode-grid"]');

    const card = page.locator('[data-testid="spec-card"]').first();
    await expect(card).toHaveScreenshot('spec-card.png');
  });

  test('spec card hover state matches snapshot', async ({ page }) => {
    await page.click('[data-testid="view-mode-grid"]');

    const card = page.locator('[data-testid="spec-card"]').first();
    await card.hover();

    await expect(card).toHaveScreenshot('spec-card-hover.png');
  });

  test('filter panel matches snapshot', async ({ page }) => {
    await page.click('[data-testid="filter-toggle"]');

    const panel = page.locator('[data-testid="filter-panel"]');
    await expect(panel).toHaveScreenshot('filter-panel.png');
  });

  test('detail view matches snapshot', async ({ page }) => {
    await page.dblclick('[data-testid="spec-list-item"]');

    await page.waitForSelector('[data-testid="spec-detail"]');
    await expect(page).toHaveScreenshot('spec-detail-view.png');
  });

  test('editor matches snapshot', async ({ page }) => {
    await page.dblclick('[data-testid="spec-list-item"]');
    await page.click('[data-testid="edit-button"]');

    await page.waitForSelector('[data-testid="spec-editor"]');
    await expect(page).toHaveScreenshot('spec-editor.png');
  });

  test('dark mode matches snapshot', async ({ page }) => {
    await page.click('[data-testid="theme-toggle"]');

    await expect(page).toHaveScreenshot('spec-browser-dark.png', {
      fullPage: true
    });
  });
});
```

---

## Testing Requirements

### Meta Tests (Tests for Testing Infrastructure)

```typescript
// src/test/meta/test-utils.test.ts
import { describe, it, expect } from 'vitest';
import {
  createMockSpec,
  createMockSpecs,
  createMockFilter
} from '../utils';

describe('Test Utilities', () => {
  describe('createMockSpec', () => {
    it('creates spec with default values', () => {
      const spec = createMockSpec();

      expect(spec.id).toBeDefined();
      expect(spec.title).toBeDefined();
      expect(spec.status).toBe('planned');
      expect(spec.phase).toBe(1);
    });

    it('allows overriding values', () => {
      const spec = createMockSpec({
        id: '999',
        title: 'Custom Title',
        status: 'tested'
      });

      expect(spec.id).toBe('999');
      expect(spec.title).toBe('Custom Title');
      expect(spec.status).toBe('tested');
    });
  });

  describe('createMockSpecs', () => {
    it('creates specified number of specs', () => {
      const specs = createMockSpecs(10);

      expect(specs.length).toBe(10);
    });

    it('assigns sequential IDs', () => {
      const specs = createMockSpecs(5);
      const ids = specs.map(s => s.id);

      expect(ids).toEqual(['100', '101', '102', '103', '104']);
    });

    it('applies overrides to all specs', () => {
      const specs = createMockSpecs(3, { status: 'implemented' });

      expect(specs.every(s => s.status === 'implemented')).toBe(true);
    });
  });

  describe('createMockFilter', () => {
    it('creates empty filter by default', () => {
      const filter = createMockFilter();

      expect(filter.statuses).toEqual([]);
      expect(filter.phases).toEqual([]);
      expect(filter.tags).toEqual([]);
    });

    it('allows specifying filter values', () => {
      const filter = createMockFilter({
        statuses: ['planned', 'in-progress'],
        phases: [1, 2]
      });

      expect(filter.statuses).toEqual(['planned', 'in-progress']);
      expect(filter.phases).toEqual([1, 2]);
    });
  });
});
```

---

## Related Specs

- Spec 231-249: All Phase 11 Components
- Phase 10: Core UI Components
