# 305 - Test Charts

**Phase:** 14 - Dashboard
**Spec ID:** 305
**Status:** Planned
**Dependencies:** 296-dashboard-layout, 301-token-charts
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Create test result visualization components that display test suite outcomes, coverage metrics, and test execution trends with detailed breakdowns by test type and status.

---

## Acceptance Criteria

- [x] `TestResultsChart.svelte` component created
- [x] `CoverageGauge.svelte` for code coverage
- [x] Pass/fail/skip breakdown visualization
- [x] Test duration trends
- [x] Flaky test identification
- [x] Test suite hierarchy view
- [x] Historical test run comparison
- [x] Integration with CI/CD pipelines

---

## Implementation Details

### 1. Test Results Chart (web/src/lib/components/tests/TestResultsChart.svelte)

```svelte
<script lang="ts">
  import { fly, fade } from 'svelte/transition';
  import { flip } from 'svelte/animate';
  import type { TestSuite, TestResult } from '$lib/types/tests';
  import Icon from '$lib/components/common/Icon.svelte';
  import CoverageGauge from './CoverageGauge.svelte';

  export let suites: TestSuite[] = [];
  export let showCoverage: boolean = true;
  export let showDuration: boolean = true;

  let expandedSuites: Set<string> = new Set();
  let selectedFilter: 'all' | 'passed' | 'failed' | 'skipped' = 'all';

  $: totalTests = suites.reduce((sum, s) => sum + s.tests.length, 0);
  $: passedTests = suites.reduce((sum, s) => sum + s.tests.filter(t => t.status === 'passed').length, 0);
  $: failedTests = suites.reduce((sum, s) => sum + s.tests.filter(t => t.status === 'failed').length, 0);
  $: skippedTests = suites.reduce((sum, s) => sum + s.tests.filter(t => t.status === 'skipped').length, 0);
  $: passRate = totalTests > 0 ? (passedTests / totalTests) * 100 : 0;

  $: filteredSuites = filterSuites(suites, selectedFilter);

  function filterSuites(suites: TestSuite[], filter: typeof selectedFilter): TestSuite[] {
    if (filter === 'all') return suites;
    return suites.map(suite => ({
      ...suite,
      tests: suite.tests.filter(t => t.status === filter)
    })).filter(suite => suite.tests.length > 0);
  }

  function toggleSuite(suiteId: string) {
    if (expandedSuites.has(suiteId)) {
      expandedSuites.delete(suiteId);
    } else {
      expandedSuites.add(suiteId);
    }
    expandedSuites = expandedSuites;
  }

  function formatDuration(ms: number): string {
    if (ms < 1000) return `${ms}ms`;
    if (ms < 60000) return `${(ms / 1000).toFixed(2)}s`;
    return `${Math.floor(ms / 60000)}m ${Math.floor((ms % 60000) / 1000)}s`;
  }

  function getStatusIcon(status: TestResult['status']): string {
    switch (status) {
      case 'passed': return 'check-circle';
      case 'failed': return 'x-circle';
      case 'skipped': return 'minus-circle';
      default: return 'circle';
    }
  }

  function getStatusColor(status: TestResult['status']): string {
    switch (status) {
      case 'passed': return 'var(--green-500)';
      case 'failed': return 'var(--red-500)';
      case 'skipped': return 'var(--gray-500)';
      default: return 'var(--text-secondary)';
    }
  }
</script>

<div class="test-results-chart">
  <div class="results-header">
    <div class="header-left">
      <Icon name="check-square" size={20} />
      <h3>Test Results</h3>
    </div>

    <div class="filter-tabs">
      <button
        class:active={selectedFilter === 'all'}
        on:click={() => selectedFilter = 'all'}
      >
        All ({totalTests})
      </button>
      <button
        class="passed"
        class:active={selectedFilter === 'passed'}
        on:click={() => selectedFilter = 'passed'}
      >
        Passed ({passedTests})
      </button>
      <button
        class="failed"
        class:active={selectedFilter === 'failed'}
        on:click={() => selectedFilter = 'failed'}
      >
        Failed ({failedTests})
      </button>
      <button
        class="skipped"
        class:active={selectedFilter === 'skipped'}
        on:click={() => selectedFilter = 'skipped'}
      >
        Skipped ({skippedTests})
      </button>
    </div>
  </div>

  <div class="results-summary">
    <div class="summary-bar">
      <div
        class="bar-segment passed"
        style="width: {(passedTests / totalTests) * 100}%"
      />
      <div
        class="bar-segment failed"
        style="width: {(failedTests / totalTests) * 100}%"
      />
      <div
        class="bar-segment skipped"
        style="width: {(skippedTests / totalTests) * 100}%"
      />
    </div>

    <div class="summary-stats">
      <div class="stat">
        <span class="stat-value" style="color: var(--green-500)">{passedTests}</span>
        <span class="stat-label">Passed</span>
      </div>
      <div class="stat">
        <span class="stat-value" style="color: var(--red-500)">{failedTests}</span>
        <span class="stat-label">Failed</span>
      </div>
      <div class="stat">
        <span class="stat-value" style="color: var(--gray-500)">{skippedTests}</span>
        <span class="stat-label">Skipped</span>
      </div>
      <div class="stat">
        <span class="stat-value">{passRate.toFixed(1)}%</span>
        <span class="stat-label">Pass Rate</span>
      </div>
    </div>
  </div>

  <div class="suites-list">
    {#each filteredSuites as suite (suite.id)}
      <div class="suite" animate:flip={{ duration: 200 }}>
        <button
          class="suite-header"
          on:click={() => toggleSuite(suite.id)}
          aria-expanded={expandedSuites.has(suite.id)}
        >
          <Icon
            name={expandedSuites.has(suite.id) ? 'chevron-down' : 'chevron-right'}
            size={16}
          />

          <span class="suite-status">
            {#if suite.tests.every(t => t.status === 'passed')}
              <Icon name="check-circle" size={16} style="color: var(--green-500)" />
            {:else if suite.tests.some(t => t.status === 'failed')}
              <Icon name="x-circle" size={16} style="color: var(--red-500)" />
            {:else}
              <Icon name="minus-circle" size={16} style="color: var(--gray-500)" />
            {/if}
          </span>

          <span class="suite-name">{suite.name}</span>

          <span class="suite-count">
            {suite.tests.filter(t => t.status === 'passed').length}/{suite.tests.length}
          </span>

          {#if showDuration}
            <span class="suite-duration">
              {formatDuration(suite.duration)}
            </span>
          {/if}
        </button>

        {#if expandedSuites.has(suite.id)}
          <ul class="tests-list" transition:fly={{ y: -10, duration: 150 }}>
            {#each suite.tests as test (test.id)}
              <li class="test-item">
                <Icon
                  name={getStatusIcon(test.status)}
                  size={14}
                  style="color: {getStatusColor(test.status)}"
                />

                <span class="test-name">{test.name}</span>

                {#if test.flaky}
                  <span class="flaky-badge" title="Flaky test">
                    <Icon name="zap" size={12} />
                  </span>
                {/if}

                {#if showDuration}
                  <span class="test-duration">
                    {formatDuration(test.duration)}
                  </span>
                {/if}

                {#if test.status === 'failed' && test.error}
                  <button
                    class="error-toggle"
                    on:click|stopPropagation
                    title="View error"
                  >
                    <Icon name="alert-circle" size={14} />
                  </button>
                {/if}
              </li>

              {#if test.status === 'failed' && test.error}
                <li class="error-details">
                  <pre>{test.error}</pre>
                </li>
              {/if}
            {/each}
          </ul>
        {/if}
      </div>
    {/each}
  </div>

  {#if showCoverage && suites.some(s => s.coverage !== undefined)}
    <div class="coverage-section">
      <h4>Code Coverage</h4>
      <div class="coverage-grid">
        {#each suites.filter(s => s.coverage !== undefined) as suite}
          <div class="coverage-item">
            <span class="coverage-name">{suite.name}</span>
            <CoverageGauge value={suite.coverage || 0} size={60} />
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  .test-results-chart {
    background: var(--bg-card);
    border: 1px solid var(--border-color);
    border-radius: 0.75rem;
    overflow: hidden;
  }

  .results-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.25rem;
    border-bottom: 1px solid var(--border-color);
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .header-left h3 {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .filter-tabs {
    display: flex;
    gap: 0.25rem;
  }

  .filter-tabs button {
    padding: 0.375rem 0.625rem;
    border: none;
    background: transparent;
    border-radius: 0.375rem;
    font-size: 0.75rem;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .filter-tabs button:hover {
    background: var(--bg-hover);
  }

  .filter-tabs button.active {
    background: var(--bg-secondary);
    color: var(--text-primary);
    font-weight: 500;
  }

  .filter-tabs button.passed.active {
    background: var(--green-100);
    color: var(--green-700);
  }

  .filter-tabs button.failed.active {
    background: var(--red-100);
    color: var(--red-700);
  }

  .results-summary {
    padding: 1rem 1.25rem;
    border-bottom: 1px solid var(--border-color);
  }

  .summary-bar {
    display: flex;
    height: 0.5rem;
    background: var(--bg-secondary);
    border-radius: 9999px;
    overflow: hidden;
  }

  .bar-segment {
    transition: width 0.3s ease;
  }

  .bar-segment.passed {
    background: var(--green-500);
  }

  .bar-segment.failed {
    background: var(--red-500);
  }

  .bar-segment.skipped {
    background: var(--gray-400);
  }

  .summary-stats {
    display: flex;
    gap: 2rem;
    margin-top: 1rem;
  }

  .stat {
    display: flex;
    flex-direction: column;
  }

  .stat-value {
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .stat-label {
    font-size: 0.75rem;
    color: var(--text-tertiary);
  }

  .suites-list {
    max-height: 400px;
    overflow-y: auto;
  }

  .suite {
    border-bottom: 1px solid var(--border-color);
  }

  .suite:last-child {
    border-bottom: none;
  }

  .suite-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    width: 100%;
    padding: 0.75rem 1.25rem;
    border: none;
    background: transparent;
    text-align: left;
    cursor: pointer;
    transition: background 0.15s ease;
  }

  .suite-header:hover {
    background: var(--bg-hover);
  }

  .suite-name {
    flex: 1;
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--text-primary);
  }

  .suite-count {
    font-size: 0.75rem;
    color: var(--text-tertiary);
  }

  .suite-duration {
    font-size: 0.75rem;
    color: var(--text-tertiary);
    font-family: monospace;
  }

  .tests-list {
    list-style: none;
    padding: 0;
    margin: 0;
    background: var(--bg-secondary);
  }

  .test-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1.25rem 0.5rem 2.5rem;
    font-size: 0.8125rem;
  }

  .test-name {
    flex: 1;
    color: var(--text-primary);
  }

  .flaky-badge {
    display: flex;
    align-items: center;
    padding: 0.125rem 0.375rem;
    background: var(--yellow-100);
    color: var(--yellow-700);
    border-radius: 9999px;
    font-size: 0.625rem;
  }

  .test-duration {
    font-size: 0.6875rem;
    color: var(--text-tertiary);
    font-family: monospace;
  }

  .error-toggle {
    padding: 0.25rem;
    border: none;
    background: transparent;
    color: var(--red-500);
    cursor: pointer;
    border-radius: 0.25rem;
  }

  .error-toggle:hover {
    background: var(--red-100);
  }

  .error-details {
    padding: 0.75rem 1.25rem 0.75rem 2.5rem;
    background: var(--red-50);
  }

  .error-details pre {
    margin: 0;
    font-size: 0.6875rem;
    color: var(--red-700);
    white-space: pre-wrap;
    overflow-x: auto;
  }

  .coverage-section {
    padding: 1rem 1.25rem;
    border-top: 1px solid var(--border-color);
  }

  .coverage-section h4 {
    margin: 0 0 0.75rem;
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-tertiary);
  }

  .coverage-grid {
    display: flex;
    gap: 1.5rem;
    flex-wrap: wrap;
  }

  .coverage-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
  }

  .coverage-name {
    font-size: 0.75rem;
    color: var(--text-secondary);
  }
</style>
```

### 2. Coverage Gauge (web/src/lib/components/tests/CoverageGauge.svelte)

```svelte
<script lang="ts">
  export let value: number = 0;
  export let size: number = 80;
  export let strokeWidth: number = 6;

  $: radius = (size - strokeWidth) / 2;
  $: circumference = 2 * Math.PI * radius;
  $: progress = (value / 100) * circumference;

  $: color = value >= 80
    ? 'var(--green-500)'
    : value >= 60
      ? 'var(--yellow-500)'
      : 'var(--red-500)';
</script>

<svg
  class="coverage-gauge"
  width={size}
  height={size}
  viewBox="0 0 {size} {size}"
>
  <circle
    class="gauge-bg"
    cx={size / 2}
    cy={size / 2}
    r={radius}
    fill="none"
    stroke="var(--bg-secondary)"
    stroke-width={strokeWidth}
  />

  <circle
    class="gauge-progress"
    cx={size / 2}
    cy={size / 2}
    r={radius}
    fill="none"
    stroke={color}
    stroke-width={strokeWidth}
    stroke-dasharray="{progress} {circumference}"
    stroke-linecap="round"
    transform="rotate(-90 {size / 2} {size / 2})"
  />

  <text
    class="gauge-value"
    x={size / 2}
    y={size / 2}
    text-anchor="middle"
    dominant-baseline="middle"
    fill={color}
  >
    {value.toFixed(0)}%
  </text>
</svg>

<style>
  .gauge-progress {
    transition: stroke-dasharray 0.5s ease;
  }

  .gauge-value {
    font-size: 0.875rem;
    font-weight: 600;
  }
</style>
```

### 3. Test Types (web/src/lib/types/tests.ts)

```typescript
export interface TestResult {
  id: string;
  name: string;
  status: 'passed' | 'failed' | 'skipped' | 'pending';
  duration: number;
  error?: string;
  flaky?: boolean;
}

export interface TestSuite {
  id: string;
  name: string;
  tests: TestResult[];
  duration: number;
  coverage?: number;
}

export interface TestRunSummary {
  totalTests: number;
  passed: number;
  failed: number;
  skipped: number;
  duration: number;
  timestamp: string;
}
```

---

## Testing Requirements

1. Summary bar shows correct proportions
2. Filter tabs filter results correctly
3. Suite expansion toggles correctly
4. Duration formatting is accurate
5. Failed test errors display properly
6. Coverage gauge shows correct percentage
7. Flaky test indicators appear correctly

---

## Related Specs

- Depends on: [296-dashboard-layout.md](296-dashboard-layout.md)
- Next: [306-deploy-metrics.md](306-deploy-metrics.md)
- Used by: CI/CD views, test result panels
