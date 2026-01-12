# 228 - Test Results Component

**Phase:** 10 - Mission Panel UI
**Spec ID:** 228
**Status:** Planned
**Dependencies:** 216-mission-layout, 217-mission-state
**Estimated Context:** ~11% of Sonnet window

---

## Objective

Create a test results component that displays test execution output in a structured format, showing pass/fail status, coverage metrics, and detailed error information for failed tests.

---

## Acceptance Criteria

- [x] Test suite tree with pass/fail indicators
- [x] Individual test case results
- [x] Error messages with stack traces
- [x] Test coverage summary
- [x] Duration metrics
- [x] Filter by status (passed/failed/skipped)
- [x] Re-run individual tests

---

## Implementation Details

### 1. Types (src/lib/types/test-results.ts)

```typescript
export interface TestRun {
  id: string;
  missionId: string;
  startedAt: string;
  completedAt?: string;
  status: TestRunStatus;
  suites: TestSuite[];
  summary: TestSummary;
  coverage?: CoverageReport;
}

export type TestRunStatus = 'running' | 'passed' | 'failed' | 'error';

export interface TestSuite {
  id: string;
  name: string;
  path: string;
  status: TestStatus;
  tests: TestCase[];
  duration: number;
  startedAt: string;
}

export interface TestCase {
  id: string;
  name: string;
  status: TestStatus;
  duration: number;
  error?: TestError;
  logs: string[];
}

export type TestStatus = 'pending' | 'running' | 'passed' | 'failed' | 'skipped';

export interface TestError {
  message: string;
  stack?: string;
  expected?: string;
  actual?: string;
  diff?: string;
}

export interface TestSummary {
  total: number;
  passed: number;
  failed: number;
  skipped: number;
  duration: number;
}

export interface CoverageReport {
  lines: CoverageMetric;
  branches: CoverageMetric;
  functions: CoverageMetric;
  statements: CoverageMetric;
}

export interface CoverageMetric {
  covered: number;
  total: number;
  percentage: number;
}
```

### 2. Test Results Component (src/lib/components/mission/TestResults.svelte)

```svelte
<script lang="ts">
  import type { TestRun, TestSuite, TestCase, TestStatus } from '$lib/types/test-results';
  import TestSuiteView from './TestSuiteView.svelte';
  import CoverageSummary from './CoverageSummary.svelte';

  export let testRun: TestRun;

  let filter: TestStatus | 'all' = 'all';
  let expandedSuites = new Set<string>();

  const statusIcons: Record<TestStatus, string> = {
    pending: '○',
    running: '◉',
    passed: '✓',
    failed: '✕',
    skipped: '⊘',
  };

  const statusColors: Record<TestStatus, string> = {
    pending: 'var(--color-text-muted)',
    running: 'var(--color-primary)',
    passed: 'var(--color-success)',
    failed: 'var(--color-error)',
    skipped: 'var(--color-warning)',
  };

  function toggleSuite(suiteId: string) {
    if (expandedSuites.has(suiteId)) {
      expandedSuites.delete(suiteId);
    } else {
      expandedSuites.add(suiteId);
    }
    expandedSuites = expandedSuites;
  }

  function expandAll() {
    expandedSuites = new Set(testRun.suites.map(s => s.id));
  }

  function collapseAll() {
    expandedSuites = new Set();
  }

  $: filteredSuites = filter === 'all'
    ? testRun.suites
    : testRun.suites.filter(s => s.tests.some(t => t.status === filter));
</script>

<div class="test-results">
  <header class="test-results__header">
    <div class="test-results__summary">
      <span class="summary-stat summary-stat--passed">
        {statusIcons.passed} {testRun.summary.passed} passed
      </span>
      <span class="summary-stat summary-stat--failed">
        {statusIcons.failed} {testRun.summary.failed} failed
      </span>
      <span class="summary-stat summary-stat--skipped">
        {statusIcons.skipped} {testRun.summary.skipped} skipped
      </span>
      <span class="summary-stat summary-stat--duration">
        {(testRun.summary.duration / 1000).toFixed(2)}s
      </span>
    </div>

    <div class="test-results__controls">
      <select bind:value={filter} class="filter-select">
        <option value="all">All tests</option>
        <option value="passed">Passed</option>
        <option value="failed">Failed</option>
        <option value="skipped">Skipped</option>
      </select>

      <button class="control-btn" on:click={expandAll}>Expand All</button>
      <button class="control-btn" on:click={collapseAll}>Collapse All</button>
    </div>
  </header>

  {#if testRun.coverage}
    <CoverageSummary coverage={testRun.coverage} />
  {/if}

  <div class="test-results__suites">
    {#each filteredSuites as suite}
      <TestSuiteView
        {suite}
        {filter}
        expanded={expandedSuites.has(suite.id)}
        on:toggle={() => toggleSuite(suite.id)}
      />
    {/each}
  </div>
</div>

<style>
  .test-results {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .test-results__header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    border-bottom: 1px solid var(--color-border);
    background: var(--color-bg-secondary);
  }

  .test-results__summary {
    display: flex;
    gap: 16px;
  }

  .summary-stat {
    font-size: 13px;
    font-weight: 500;
  }

  .summary-stat--passed { color: var(--color-success); }
  .summary-stat--failed { color: var(--color-error); }
  .summary-stat--skipped { color: var(--color-warning); }
  .summary-stat--duration { color: var(--color-text-muted); }

  .test-results__controls {
    display: flex;
    gap: 8px;
  }

  .filter-select {
    padding: 6px 10px;
    border: 1px solid var(--color-border);
    border-radius: 4px;
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
    font-size: 13px;
  }

  .control-btn {
    padding: 6px 12px;
    border: 1px solid var(--color-border);
    background: var(--color-bg-primary);
    color: var(--color-text-secondary);
    font-size: 12px;
    border-radius: 4px;
    cursor: pointer;
  }

  .test-results__suites {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
  }
</style>
```

### 3. Test Suite View Component (src/lib/components/mission/TestSuiteView.svelte)

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { TestSuite, TestCase, TestStatus } from '$lib/types/test-results';

  export let suite: TestSuite;
  export let filter: TestStatus | 'all' = 'all';
  export let expanded = false;

  const dispatch = createEventDispatcher<{ toggle: void }>();

  const statusColors: Record<TestStatus, string> = {
    pending: 'var(--color-text-muted)',
    running: 'var(--color-primary)',
    passed: 'var(--color-success)',
    failed: 'var(--color-error)',
    skipped: 'var(--color-warning)',
  };

  $: filteredTests = filter === 'all'
    ? suite.tests
    : suite.tests.filter(t => t.status === filter);

  $: passedCount = suite.tests.filter(t => t.status === 'passed').length;
  $: failedCount = suite.tests.filter(t => t.status === 'failed').length;
</script>

<div class="test-suite" class:test-suite--failed={suite.status === 'failed'}>
  <button
    class="test-suite__header"
    on:click={() => dispatch('toggle')}
    aria-expanded={expanded}
  >
    <span class="test-suite__icon" style="color: {statusColors[suite.status]}">
      {suite.status === 'passed' ? '✓' : suite.status === 'failed' ? '✕' : '○'}
    </span>

    <span class="test-suite__name">{suite.name}</span>

    <span class="test-suite__stats">
      <span class="stat stat--passed">{passedCount}</span>
      {#if failedCount > 0}
        <span class="stat stat--failed">{failedCount}</span>
      {/if}
    </span>

    <span class="test-suite__duration">{suite.duration}ms</span>

    <span class="test-suite__chevron" class:rotated={expanded}>▸</span>
  </button>

  {#if expanded}
    <div class="test-suite__tests">
      {#each filteredTests as test}
        <div
          class="test-case"
          class:test-case--failed={test.status === 'failed'}
        >
          <span class="test-case__icon" style="color: {statusColors[test.status]}">
            {test.status === 'passed' ? '✓' : test.status === 'failed' ? '✕' : '○'}
          </span>

          <span class="test-case__name">{test.name}</span>
          <span class="test-case__duration">{test.duration}ms</span>

          {#if test.error}
            <div class="test-case__error">
              <p class="error-message">{test.error.message}</p>
              {#if test.error.diff}
                <pre class="error-diff">{test.error.diff}</pre>
              {/if}
              {#if test.error.stack}
                <pre class="error-stack">{test.error.stack}</pre>
              {/if}
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .test-suite {
    margin-bottom: 4px;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    overflow: hidden;
  }

  .test-suite--failed {
    border-color: var(--color-error);
  }

  .test-suite__header {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 10px 12px;
    border: none;
    background: var(--color-bg-secondary);
    cursor: pointer;
    text-align: left;
  }

  .test-suite__icon {
    font-size: 12px;
  }

  .test-suite__name {
    flex: 1;
    font-size: 13px;
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .test-suite__stats {
    display: flex;
    gap: 8px;
    font-size: 12px;
  }

  .stat--passed { color: var(--color-success); }
  .stat--failed { color: var(--color-error); }

  .test-suite__duration {
    font-size: 11px;
    color: var(--color-text-muted);
  }

  .test-suite__chevron {
    color: var(--color-text-muted);
    transition: transform 0.15s;
  }

  .test-suite__chevron.rotated {
    transform: rotate(90deg);
  }

  .test-suite__tests {
    padding: 8px;
    background: var(--color-bg-primary);
  }

  .test-case {
    padding: 8px;
    border-radius: 4px;
  }

  .test-case--failed {
    background: rgba(248, 113, 113, 0.1);
  }

  .test-case__icon {
    margin-right: 8px;
  }

  .test-case__name {
    font-size: 13px;
    color: var(--color-text-primary);
  }

  .test-case__duration {
    margin-left: auto;
    font-size: 11px;
    color: var(--color-text-muted);
  }

  .test-case__error {
    margin-top: 8px;
    padding: 8px;
    background: var(--color-bg-secondary);
    border-radius: 4px;
  }

  .error-message {
    margin: 0 0 8px 0;
    color: var(--color-error);
    font-size: 13px;
  }

  .error-diff,
  .error-stack {
    margin: 0;
    padding: 8px;
    background: var(--color-bg-primary);
    border-radius: 4px;
    font-size: 11px;
    overflow-x: auto;
  }

  .error-stack {
    color: var(--color-text-muted);
    margin-top: 8px;
  }
</style>
```

---

## Testing Requirements

1. Test summary displays correctly
2. Suite expansion works
3. Filter by status works
4. Error details show properly
5. Coverage displays when available
6. Duration formatting is correct

---

## Related Specs

- Depends on: [216-mission-layout.md](216-mission-layout.md)
- Next: [229-cost-tracking.md](229-cost-tracking.md)
