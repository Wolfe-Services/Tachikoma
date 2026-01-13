# 233 - Mission Comparison Component

**Phase:** 10 - Mission Panel UI
**Spec ID:** 233
**Status:** Planned
**Dependencies:** 216-mission-layout, 217-mission-state, 232-history-view
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Create a mission comparison component that allows users to compare two missions side-by-side, highlighting differences in configuration, results, performance metrics, and file changes.

---

## Acceptance Criteria

- [x] Side-by-side mission display
- [x] Configuration diff highlighting
- [x] Performance metrics comparison
- [x] File changes comparison
- [x] Export comparison report
- [x] Share comparison link

---

## Implementation Details

### 1. Types (src/lib/types/comparison.ts)

```typescript
export interface MissionComparison {
  missionA: MissionSummary;
  missionB: MissionSummary;
  configDiff: ConfigDiff[];
  metricsDiff: MetricsDiff;
  fileDiff: FileDiff[];
}

export interface MissionSummary {
  id: string;
  title: string;
  createdAt: string;
  state: string;
  duration: number;
  cost: number;
  tokensUsed: number;
}

export interface ConfigDiff {
  key: string;
  valueA: string;
  valueB: string;
  changed: boolean;
}

export interface MetricsDiff {
  duration: { a: number; b: number; diff: number; percentDiff: number };
  cost: { a: number; b: number; diff: number; percentDiff: number };
  tokens: { a: number; b: number; diff: number; percentDiff: number };
  filesChanged: { a: number; b: number; diff: number };
}

export interface FileDiff {
  path: string;
  inA: boolean;
  inB: boolean;
  status: 'same' | 'different' | 'only_a' | 'only_b';
}
```

### 2. Mission Comparison Component (src/lib/components/mission/MissionComparison.svelte)

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { MissionComparison, MetricsDiff } from '$lib/types/comparison';

  export let comparison: MissionComparison;

  const dispatch = createEventDispatcher<{
    close: void;
    export: void;
  }>();

  function formatDuration(ms: number): string {
    return `${(ms / 1000).toFixed(1)}s`;
  }

  function formatCost(cost: number): string {
    return `$${cost.toFixed(4)}`;
  }

  function formatPercent(diff: number): string {
    const sign = diff > 0 ? '+' : '';
    return `${sign}${diff.toFixed(1)}%`;
  }

  function getDiffClass(percentDiff: number): string {
    if (Math.abs(percentDiff) < 5) return '';
    return percentDiff > 0 ? 'diff--worse' : 'diff--better';
  }
</script>

<div class="mission-comparison">
  <header class="comparison-header">
    <h2>Mission Comparison</h2>
    <div class="header-actions">
      <button on:click={() => dispatch('export')}>Export</button>
      <button on:click={() => dispatch('close')}>Close</button>
    </div>
  </header>

  <!-- Mission Headers -->
  <div class="comparison-grid">
    <div class="comparison-label"></div>
    <div class="comparison-cell comparison-cell--header">
      <h3>{comparison.missionA.title}</h3>
      <span class="mission-date">{new Date(comparison.missionA.createdAt).toLocaleDateString()}</span>
    </div>
    <div class="comparison-cell comparison-cell--header">
      <h3>{comparison.missionB.title}</h3>
      <span class="mission-date">{new Date(comparison.missionB.createdAt).toLocaleDateString()}</span>
    </div>
  </div>

  <!-- Metrics -->
  <section class="comparison-section">
    <h4>Performance Metrics</h4>

    <div class="comparison-grid">
      <div class="comparison-label">Duration</div>
      <div class="comparison-cell">{formatDuration(comparison.metricsDiff.duration.a)}</div>
      <div class="comparison-cell">
        {formatDuration(comparison.metricsDiff.duration.b)}
        <span class="diff {getDiffClass(comparison.metricsDiff.duration.percentDiff)}">
          {formatPercent(comparison.metricsDiff.duration.percentDiff)}
        </span>
      </div>
    </div>

    <div class="comparison-grid">
      <div class="comparison-label">Cost</div>
      <div class="comparison-cell">{formatCost(comparison.metricsDiff.cost.a)}</div>
      <div class="comparison-cell">
        {formatCost(comparison.metricsDiff.cost.b)}
        <span class="diff {getDiffClass(comparison.metricsDiff.cost.percentDiff)}">
          {formatPercent(comparison.metricsDiff.cost.percentDiff)}
        </span>
      </div>
    </div>

    <div class="comparison-grid">
      <div class="comparison-label">Tokens</div>
      <div class="comparison-cell">{comparison.metricsDiff.tokens.a.toLocaleString()}</div>
      <div class="comparison-cell">
        {comparison.metricsDiff.tokens.b.toLocaleString()}
        <span class="diff {getDiffClass(comparison.metricsDiff.tokens.percentDiff)}">
          {formatPercent(comparison.metricsDiff.tokens.percentDiff)}
        </span>
      </div>
    </div>
  </section>

  <!-- Config Diff -->
  <section class="comparison-section">
    <h4>Configuration</h4>

    {#each comparison.configDiff as config}
      <div class="comparison-grid" class:changed={config.changed}>
        <div class="comparison-label">{config.key}</div>
        <div class="comparison-cell">{config.valueA}</div>
        <div class="comparison-cell">{config.valueB}</div>
      </div>
    {/each}
  </section>

  <!-- File Diff -->
  <section class="comparison-section">
    <h4>File Changes</h4>

    <div class="file-diff-list">
      {#each comparison.fileDiff as file}
        <div class="file-diff-item file-diff-item--{file.status}">
          <span class="file-status">
            {#if file.status === 'same'}=
            {:else if file.status === 'different'}~
            {:else if file.status === 'only_a'}A
            {:else}B{/if}
          </span>
          <span class="file-path">{file.path}</span>
        </div>
      {/each}
    </div>
  </section>
</div>

<style>
  .mission-comparison {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow-y: auto;
  }

  .comparison-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px;
    border-bottom: 1px solid var(--color-border);
  }

  .comparison-header h2 {
    margin: 0;
    font-size: 18px;
  }

  .header-actions {
    display: flex;
    gap: 8px;
  }

  .header-actions button {
    padding: 6px 12px;
    border: 1px solid var(--color-border);
    background: var(--color-bg-primary);
    border-radius: 4px;
    cursor: pointer;
  }

  .comparison-section {
    padding: 16px;
    border-bottom: 1px solid var(--color-border);
  }

  .comparison-section h4 {
    margin: 0 0 12px 0;
    font-size: 14px;
    color: var(--color-text-secondary);
  }

  .comparison-grid {
    display: grid;
    grid-template-columns: 120px 1fr 1fr;
    gap: 8px;
    padding: 8px 0;
    border-bottom: 1px solid var(--color-border);
  }

  .comparison-grid.changed {
    background: rgba(33, 150, 243, 0.1);
  }

  .comparison-label {
    font-size: 13px;
    color: var(--color-text-muted);
  }

  .comparison-cell {
    font-size: 13px;
    color: var(--color-text-primary);
  }

  .comparison-cell--header h3 {
    margin: 0;
    font-size: 14px;
  }

  .mission-date {
    font-size: 11px;
    color: var(--color-text-muted);
  }

  .diff {
    margin-left: 8px;
    font-size: 11px;
    padding: 2px 6px;
    border-radius: 4px;
  }

  .diff--better {
    background: rgba(76, 175, 80, 0.2);
    color: var(--color-success);
  }

  .diff--worse {
    background: rgba(244, 67, 54, 0.2);
    color: var(--color-error);
  }

  .file-diff-list {
    font-family: monospace;
    font-size: 12px;
  }

  .file-diff-item {
    display: flex;
    gap: 8px;
    padding: 4px 0;
  }

  .file-status {
    width: 16px;
    text-align: center;
    font-weight: 600;
  }

  .file-diff-item--different .file-status { color: var(--color-primary); }
  .file-diff-item--only_a .file-status { color: var(--color-error); }
  .file-diff-item--only_b .file-status { color: var(--color-success); }
</style>
```

---

## Testing Requirements

1. Side-by-side display works
2. Metrics diff calculates correctly
3. Config changes highlight
4. File diff shows status
5. Export generates report

---

## Related Specs

- Depends on: [232-history-view.md](232-history-view.md)
- Next: [234-mission-export.md](234-mission-export.md)
