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

  function handleRerun(event: CustomEvent<{ suiteId: string; testId?: string }>) {
    // Emit event to parent component for handling
    const { suiteId, testId } = event.detail;
    if (testId) {
      console.log(`Re-running test ${testId} in suite ${suiteId}`);
      // TODO: Integrate with test runner
    } else {
      console.log(`Re-running suite ${suiteId}`);
      // TODO: Integrate with test runner
    }
  }

  $: filteredSuites = filter === 'all'
    ? testRun.suites
    : testRun.suites.filter(s => s.tests.some(t => t.status === filter));

  $: overallStatus = testRun.status;
  $: duration = (testRun.summary.duration / 1000).toFixed(2);
</script>

<div class="test-results">
  <header class="test-results__header">
    <div class="test-results__summary">
      <div class="summary-overview">
        <span class="summary-overview__status" class:status--passed={overallStatus === 'passed'} class:status--failed={overallStatus === 'failed'} class:status--running={overallStatus === 'running'}>
          {#if overallStatus === 'passed'}
            {statusIcons.passed} All Tests Passed
          {:else if overallStatus === 'failed'}
            {statusIcons.failed} Tests Failed
          {:else if overallStatus === 'running'}
            {statusIcons.running} Tests Running
          {:else}
            {statusIcons.pending} Test Error
          {/if}
        </span>
        <span class="summary-overview__duration">{duration}s</span>
      </div>

      <div class="summary-stats">
        <span class="summary-stat summary-stat--passed">
          {statusIcons.passed} {testRun.summary.passed} passed
        </span>
        {#if testRun.summary.failed > 0}
          <span class="summary-stat summary-stat--failed">
            {statusIcons.failed} {testRun.summary.failed} failed
          </span>
        {/if}
        {#if testRun.summary.skipped > 0}
          <span class="summary-stat summary-stat--skipped">
            {statusIcons.skipped} {testRun.summary.skipped} skipped
          </span>
        {/if}
        <span class="summary-stat summary-stat--total">
          {testRun.summary.total} total
        </span>
      </div>
    </div>

    <div class="test-results__controls">
      <select bind:value={filter} class="filter-select">
        <option value="all">All tests</option>
        <option value="passed">Passed</option>
        <option value="failed">Failed</option>
        <option value="skipped">Skipped</option>
      </select>

      <button class="control-btn" on:click={expandAll} title="Expand all test suites">
        Expand All
      </button>
      <button class="control-btn" on:click={collapseAll} title="Collapse all test suites">
        Collapse All
      </button>
    </div>
  </header>

  {#if testRun.coverage}
    <CoverageSummary coverage={testRun.coverage} />
  {/if}

  <div class="test-results__content">
    {#if filteredSuites.length === 0}
      <div class="test-results__empty">
        <p class="empty-message">
          {#if filter === 'all'}
            No test suites found
          {:else}
            No tests with status "{filter}" found
          {/if}
        </p>
      </div>
    {:else}
      <div class="test-results__suites">
        {#each filteredSuites as suite}
          <TestSuiteView
            {suite}
            {filter}
            expanded={expandedSuites.has(suite.id)}
            on:toggle={() => toggleSuite(suite.id)}
            on:rerun={handleRerun}
          />
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  .test-results {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--color-bg-surface);
    border-radius: 8px;
    overflow: hidden;
  }

  .test-results__header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    padding: 16px;
    border-bottom: 1px solid var(--color-border-default);
    background: var(--color-bg-elevated);
    gap: 16px;
  }

  .test-results__summary {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .summary-overview {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .summary-overview__status {
    font-size: 14px;
    font-weight: 600;
  }

  .summary-overview__status.status--passed {
    color: var(--color-success-fg);
  }

  .summary-overview__status.status--failed {
    color: var(--color-error-fg);
  }

  .summary-overview__status.status--running {
    color: var(--color-accent-fg);
  }

  .summary-overview__duration {
    font-size: 12px;
    color: var(--color-fg-muted);
    font-weight: 500;
  }

  .summary-stats {
    display: flex;
    gap: 16px;
    flex-wrap: wrap;
  }

  .summary-stat {
    font-size: 13px;
    font-weight: 500;
  }

  .summary-stat--passed { color: var(--color-success-fg); }
  .summary-stat--failed { color: var(--color-error-fg); }
  .summary-stat--skipped { color: var(--color-warning-fg); }
  .summary-stat--total { color: var(--color-fg-muted); }

  .test-results__controls {
    display: flex;
    gap: 8px;
    align-items: flex-start;
  }

  .filter-select {
    padding: 6px 10px;
    border: 1px solid var(--color-border-default);
    border-radius: 6px;
    background: var(--color-bg-surface);
    color: var(--color-fg-default);
    font-size: 13px;
    cursor: pointer;
  }

  .filter-select:hover {
    background: var(--color-bg-hover);
  }

  .filter-select:focus {
    outline: 2px solid var(--color-accent-fg);
    outline-offset: -2px;
  }

  .control-btn {
    padding: 6px 12px;
    border: 1px solid var(--color-border-default);
    background: var(--color-bg-surface);
    color: var(--color-fg-muted);
    font-size: 12px;
    border-radius: 6px;
    cursor: pointer;
    white-space: nowrap;
  }

  .control-btn:hover {
    background: var(--color-bg-hover);
    color: var(--color-fg-default);
  }

  .test-results__content {
    flex: 1;
    overflow-y: auto;
    min-height: 0;
  }

  .test-results__empty {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 200px;
  }

  .empty-message {
    color: var(--color-fg-muted);
    font-size: 14px;
    text-align: center;
    margin: 0;
  }

  .test-results__suites {
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  /* Scrollbar styling for webkit browsers */
  .test-results__content::-webkit-scrollbar {
    width: 8px;
  }

  .test-results__content::-webkit-scrollbar-track {
    background: var(--color-bg-muted);
  }

  .test-results__content::-webkit-scrollbar-thumb {
    background: var(--color-border-default);
    border-radius: 4px;
  }

  .test-results__content::-webkit-scrollbar-thumb:hover {
    background: var(--color-fg-subtle);
  }
</style>