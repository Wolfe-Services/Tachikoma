<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { TestSuite, TestCase, TestStatus } from '$lib/types/test-results';

  export let suite: TestSuite;
  export let filter: TestStatus | 'all' = 'all';
  export let expanded = false;

  const dispatch = createEventDispatcher<{ 
    toggle: void; 
    rerun: { suiteId: string; testId?: string; };
  }>();

  const statusColors: Record<TestStatus, string> = {
    pending: 'var(--color-text-muted)',
    running: 'var(--color-primary)',
    passed: 'var(--color-success)',
    failed: 'var(--color-error)',
    skipped: 'var(--color-warning)',
  };

  const statusIcons: Record<TestStatus, string> = {
    pending: '○',
    running: '◉',
    passed: '✓',
    failed: '✕',
    skipped: '⊘',
  };

  $: filteredTests = filter === 'all'
    ? suite.tests
    : suite.tests.filter(t => t.status === filter);

  $: passedCount = suite.tests.filter(t => t.status === 'passed').length;
  $: failedCount = suite.tests.filter(t => t.status === 'failed').length;

  function handleRerunTest(testId: string) {
    dispatch('rerun', { suiteId: suite.id, testId });
  }

  function handleRerunSuite() {
    dispatch('rerun', { suiteId: suite.id });
  }
</script>

<div class="test-suite" class:test-suite--failed={suite.status === 'failed'}>
  <button
    class="test-suite__header"
    on:click={() => dispatch('toggle')}
    aria-expanded={expanded}
  >
    <span class="test-suite__icon" style="color: {statusColors[suite.status]}">
      {statusIcons[suite.status]}
    </span>

    <span class="test-suite__name">{suite.name}</span>

    <span class="test-suite__stats">
      <span class="stat stat--passed">{passedCount}</span>
      {#if failedCount > 0}
        <span class="stat stat--failed">{failedCount}</span>
      {/if}
    </span>

    <span class="test-suite__duration">{suite.duration}ms</span>

    <button 
      class="test-suite__rerun"
      on:click|stopPropagation={handleRerunSuite}
      title="Re-run suite"
    >
      ↻
    </button>

    <span class="test-suite__chevron" class:rotated={expanded}>▸</span>
  </button>

  {#if expanded}
    <div class="test-suite__tests">
      {#each filteredTests as test}
        <div
          class="test-case"
          class:test-case--failed={test.status === 'failed'}
        >
          <div class="test-case__header">
            <span class="test-case__icon" style="color: {statusColors[test.status]}">
              {statusIcons[test.status]}
            </span>

            <span class="test-case__name">{test.name}</span>
            <span class="test-case__duration">{test.duration}ms</span>

            <button 
              class="test-case__rerun"
              on:click={() => handleRerunTest(test.id)}
              title="Re-run test"
            >
              ↻
            </button>
          </div>

          {#if test.error}
            <div class="test-case__error">
              <p class="error-message">{test.error.message}</p>
              
              {#if test.error.expected && test.error.actual}
                <div class="error-comparison">
                  <div class="error-expected">
                    <strong>Expected:</strong>
                    <pre>{test.error.expected}</pre>
                  </div>
                  <div class="error-actual">
                    <strong>Actual:</strong>
                    <pre>{test.error.actual}</pre>
                  </div>
                </div>
              {/if}

              {#if test.error.diff}
                <div class="error-diff">
                  <strong>Diff:</strong>
                  <pre class="diff-content">{test.error.diff}</pre>
                </div>
              {/if}
              
              {#if test.error.stack}
                <details class="error-stack-details">
                  <summary>Stack Trace</summary>
                  <pre class="error-stack">{test.error.stack}</pre>
                </details>
              {/if}
            </div>
          {/if}

          {#if test.logs && test.logs.length > 0}
            <details class="test-logs">
              <summary>Logs ({test.logs.length})</summary>
              <div class="logs-content">
                {#each test.logs as log}
                  <pre class="log-line">{log}</pre>
                {/each}
              </div>
            </details>
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

  .test-suite__header:hover {
    background: var(--color-bg-tertiary);
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

  .test-suite__rerun {
    padding: 2px 6px;
    border: 1px solid var(--color-border);
    background: var(--color-bg-primary);
    color: var(--color-text-secondary);
    font-size: 12px;
    border-radius: 3px;
    cursor: pointer;
  }

  .test-suite__rerun:hover {
    background: var(--color-bg-tertiary);
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
    margin-bottom: 4px;
  }

  .test-case--failed {
    background: rgba(248, 113, 113, 0.1);
  }

  .test-case__header {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .test-case__icon {
    font-size: 12px;
  }

  .test-case__name {
    flex: 1;
    font-size: 13px;
    color: var(--color-text-primary);
  }

  .test-case__duration {
    font-size: 11px;
    color: var(--color-text-muted);
  }

  .test-case__rerun {
    padding: 2px 6px;
    border: 1px solid var(--color-border);
    background: var(--color-bg-secondary);
    color: var(--color-text-secondary);
    font-size: 12px;
    border-radius: 3px;
    cursor: pointer;
  }

  .test-case__rerun:hover {
    background: var(--color-bg-tertiary);
  }

  .test-case__error {
    margin-top: 8px;
    padding: 12px;
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-error);
    border-radius: 4px;
  }

  .error-message {
    margin: 0 0 8px 0;
    color: var(--color-error);
    font-size: 13px;
    font-weight: 500;
  }

  .error-comparison {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
    margin: 8px 0;
  }

  .error-expected strong {
    color: var(--color-success);
  }

  .error-actual strong {
    color: var(--color-error);
  }

  .error-expected pre,
  .error-actual pre {
    margin: 4px 0 0 0;
    padding: 8px;
    background: var(--color-bg-primary);
    border-radius: 4px;
    font-size: 12px;
    overflow-x: auto;
  }

  .error-diff {
    margin: 8px 0;
  }

  .error-diff strong {
    color: var(--color-text-primary);
  }

  .diff-content {
    margin: 4px 0 0 0;
    padding: 8px;
    background: var(--color-bg-primary);
    border-radius: 4px;
    font-size: 12px;
    overflow-x: auto;
  }

  .error-stack-details {
    margin-top: 8px;
  }

  .error-stack-details summary {
    cursor: pointer;
    color: var(--color-text-secondary);
    font-size: 12px;
  }

  .error-stack {
    margin: 8px 0 0 0;
    padding: 8px;
    background: var(--color-bg-primary);
    border-radius: 4px;
    color: var(--color-text-muted);
    font-size: 11px;
    overflow-x: auto;
  }

  .test-logs {
    margin-top: 8px;
  }

  .test-logs summary {
    cursor: pointer;
    color: var(--color-text-secondary);
    font-size: 12px;
  }

  .logs-content {
    margin-top: 8px;
    max-height: 200px;
    overflow-y: auto;
    background: var(--color-bg-secondary);
    border-radius: 4px;
  }

  .log-line {
    margin: 0;
    padding: 4px 8px;
    font-size: 11px;
    color: var(--color-text-muted);
    border-bottom: 1px solid var(--color-border);
  }

  .log-line:last-child {
    border-bottom: none;
  }
</style>