<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { MissionComparison, ComparisonExportOptions } from '$lib/types/comparison';

  export let comparison: MissionComparison;

  const dispatch = createEventDispatcher<{
    close: void;
    export: ComparisonExportOptions;
    share: string;
  }>();

  let shareUrl = '';
  let showExportDialog = false;
  let exportOptions: ComparisonExportOptions = {
    format: 'json',
    includeConfig: true,
    includeMetrics: true,
    includeFiles: true,
  };

  function formatDuration(ms: number): string {
    if (ms < 1000) return `${ms}ms`;
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

  function handleExport() {
    showExportDialog = true;
  }

  function confirmExport() {
    dispatch('export', exportOptions);
    showExportDialog = false;
  }

  function cancelExport() {
    showExportDialog = false;
  }

  function handleShare() {
    // Generate share URL with comparison ID
    const comparisonId = `${comparison.missionA.id}-${comparison.missionB.id}`;
    shareUrl = `${window.location.origin}/comparison/${comparisonId}`;
    dispatch('share', shareUrl);
  }

  function copyShareUrl() {
    navigator.clipboard.writeText(shareUrl);
  }
</script>

<div class="mission-comparison">
  <header class="comparison-header">
    <h2>Mission Comparison</h2>
    <div class="header-actions">
      <button class="btn btn--secondary" on:click={handleShare}>Share</button>
      <button class="btn btn--secondary" on:click={handleExport}>Export</button>
      <button class="btn btn--ghost" on:click={() => dispatch('close')}>Close</button>
    </div>
  </header>

  <!-- Mission Headers -->
  <div class="comparison-grid comparison-grid--header">
    <div class="comparison-label"></div>
    <div class="comparison-cell comparison-cell--header">
      <h3>{comparison.missionA.title}</h3>
      <div class="mission-meta">
        <span class="mission-date">{new Date(comparison.missionA.createdAt).toLocaleDateString()}</span>
        <span class="mission-state mission-state--{comparison.missionA.state}">{comparison.missionA.state}</span>
      </div>
    </div>
    <div class="comparison-cell comparison-cell--header">
      <h3>{comparison.missionB.title}</h3>
      <div class="mission-meta">
        <span class="mission-date">{new Date(comparison.missionB.createdAt).toLocaleDateString()}</span>
        <span class="mission-state mission-state--{comparison.missionB.state}">{comparison.missionB.state}</span>
      </div>
    </div>
  </div>

  <!-- Performance Metrics -->
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
      <div class="comparison-label">Tokens Used</div>
      <div class="comparison-cell">{comparison.metricsDiff.tokens.a.toLocaleString()}</div>
      <div class="comparison-cell">
        {comparison.metricsDiff.tokens.b.toLocaleString()}
        <span class="diff {getDiffClass(comparison.metricsDiff.tokens.percentDiff)}">
          {formatPercent(comparison.metricsDiff.tokens.percentDiff)}
        </span>
      </div>
    </div>

    <div class="comparison-grid">
      <div class="comparison-label">Files Changed</div>
      <div class="comparison-cell">{comparison.metricsDiff.filesChanged.a}</div>
      <div class="comparison-cell">
        {comparison.metricsDiff.filesChanged.b}
        {#if comparison.metricsDiff.filesChanged.diff !== 0}
          <span class="diff {comparison.metricsDiff.filesChanged.diff > 0 ? 'diff--more' : 'diff--less'}">
            {comparison.metricsDiff.filesChanged.diff > 0 ? '+' : ''}{comparison.metricsDiff.filesChanged.diff}
          </span>
        {/if}
      </div>
    </div>
  </section>

  <!-- Configuration Diff -->
  <section class="comparison-section">
    <h4>Configuration</h4>

    <div class="config-diff">
      {#each comparison.configDiff as config}
        <div class="comparison-grid" class:changed={config.changed}>
          <div class="comparison-label">{config.key}</div>
          <div class="comparison-cell">{config.valueA || '—'}</div>
          <div class="comparison-cell">{config.valueB || '—'}</div>
        </div>
      {/each}

      {#if comparison.configDiff.length === 0}
        <div class="empty-state">No configuration differences found.</div>
      {/if}
    </div>
  </section>

  <!-- File Changes -->
  <section class="comparison-section">
    <h4>File Changes</h4>

    <div class="file-diff-list">
      {#each comparison.fileDiff as file}
        <div class="file-diff-item file-diff-item--{file.status}">
          <span class="file-status" title={file.status}>
            {#if file.status === 'same'}
              <span class="icon-equal">=</span>
            {:else if file.status === 'different'}
              <span class="icon-modified">~</span>
            {:else if file.status === 'only_a'}
              <span class="icon-removed">−</span>
            {:else}
              <span class="icon-added">+</span>
            {/if}
          </span>
          <span class="file-path">{file.path}</span>
        </div>
      {/each}

      {#if comparison.fileDiff.length === 0}
        <div class="empty-state">No file differences to display.</div>
      {/if}
    </div>
  </section>
</div>

<!-- Export Dialog -->
{#if showExportDialog}
  <div class="export-dialog-overlay" on:click={cancelExport}>
    <div class="export-dialog" on:click|stopPropagation>
      <header class="export-dialog__header">
        <h3>Export Comparison</h3>
        <button class="btn btn--ghost" on:click={cancelExport}>×</button>
      </header>

      <div class="export-dialog__body">
        <div class="form-group">
          <label for="export-format">Format:</label>
          <select id="export-format" bind:value={exportOptions.format}>
            <option value="json">JSON</option>
            <option value="csv">CSV</option>
            <option value="html">HTML Report</option>
          </select>
        </div>

        <div class="form-group">
          <label>Include Sections:</label>
          <div class="checkbox-group">
            <label class="checkbox-label">
              <input type="checkbox" bind:checked={exportOptions.includeConfig} />
              Configuration Diff
            </label>
            <label class="checkbox-label">
              <input type="checkbox" bind:checked={exportOptions.includeMetrics} />
              Performance Metrics
            </label>
            <label class="checkbox-label">
              <input type="checkbox" bind:checked={exportOptions.includeFiles} />
              File Changes
            </label>
          </div>
        </div>
      </div>

      <footer class="export-dialog__footer">
        <button class="btn btn--secondary" on:click={cancelExport}>Cancel</button>
        <button class="btn btn--primary" on:click={confirmExport}>Export</button>
      </footer>
    </div>
  </div>
{/if}

<!-- Share URL Display -->
{#if shareUrl}
  <div class="share-url-display">
    <label for="share-url">Share URL:</label>
    <div class="share-url-input">
      <input id="share-url" type="text" readonly value={shareUrl} />
      <button class="btn btn--secondary btn--sm" on:click={copyShareUrl}>Copy</button>
    </div>
  </div>
{/if}

<style>
  .mission-comparison {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--color-bg-surface);
    border-radius: var(--card-radius);
    overflow: hidden;
  }

  .comparison-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-4);
    border-bottom: 1px solid var(--color-border-default);
    background: var(--color-bg-elevated);
  }

  .comparison-header h2 {
    margin: 0;
    font-size: var(--text-lg);
    color: var(--color-fg-default);
  }

  .header-actions {
    display: flex;
    gap: var(--space-2);
  }

  .btn {
    height: var(--btn-height-md);
    padding: 0 var(--btn-padding-x);
    border: 1px solid transparent;
    border-radius: var(--btn-radius);
    font-size: var(--btn-font-size);
    font-weight: var(--font-medium);
    cursor: pointer;
    transition: all var(--duration-150) var(--ease-out);
  }

  .btn--primary {
    background: var(--color-primary);
    color: white;
  }

  .btn--primary:hover {
    background: var(--color-primary-hover);
  }

  .btn--secondary {
    border-color: var(--color-border-default);
    background: var(--color-bg-surface);
    color: var(--color-fg-default);
  }

  .btn--secondary:hover {
    background: var(--color-bg-hover);
  }

  .btn--ghost {
    background: transparent;
    color: var(--color-fg-muted);
  }

  .btn--ghost:hover {
    background: var(--color-bg-hover);
    color: var(--color-fg-default);
  }

  .btn--sm {
    height: var(--btn-height-sm);
    padding: 0 var(--btn-padding-x-sm);
    font-size: var(--btn-font-size-sm);
  }

  .comparison-section {
    padding: var(--space-4);
    border-bottom: 1px solid var(--color-border-muted);
  }

  .comparison-section:last-child {
    border-bottom: none;
  }

  .comparison-section h4 {
    margin: 0 0 var(--space-3) 0;
    font-size: var(--text-base);
    font-weight: var(--font-medium);
    color: var(--color-fg-default);
  }

  .comparison-grid {
    display: grid;
    grid-template-columns: 140px 1fr 1fr;
    gap: var(--space-2);
    padding: var(--space-2) 0;
    border-bottom: 1px solid var(--color-border-subtle);
  }

  .comparison-grid:last-child {
    border-bottom: none;
  }

  .comparison-grid.changed {
    background: var(--color-primary-subtle);
    border-radius: var(--radius-sm);
    padding: var(--space-2);
    margin: var(--space-1) 0;
  }

  .comparison-grid--header {
    background: var(--color-bg-elevated);
    border-radius: var(--radius-sm);
    padding: var(--space-3);
    margin-bottom: var(--space-3);
    border: none;
  }

  .comparison-label {
    font-size: var(--text-sm);
    color: var(--color-fg-muted);
    font-weight: var(--font-medium);
  }

  .comparison-cell {
    font-size: var(--text-sm);
    color: var(--color-fg-default);
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .comparison-cell--header h3 {
    margin: 0 0 var(--space-1) 0;
    font-size: var(--text-base);
    font-weight: var(--font-semibold);
    color: var(--color-fg-default);
  }

  .mission-meta {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .mission-date {
    font-size: var(--text-xs);
    color: var(--color-fg-muted);
  }

  .mission-state {
    font-size: var(--text-xs);
    padding: 2px var(--space-2);
    border-radius: var(--radius-sm);
    font-weight: var(--font-medium);
    text-transform: uppercase;
  }

  .mission-state--completed {
    background: var(--color-success-subtle);
    color: var(--color-success);
  }

  .mission-state--failed {
    background: var(--color-error-subtle);
    color: var(--color-error);
  }

  .mission-state--running {
    background: var(--color-primary-subtle);
    color: var(--color-primary);
  }

  .diff {
    font-size: var(--text-xs);
    padding: 2px var(--space-1);
    border-radius: var(--radius-sm);
    font-weight: var(--font-medium);
  }

  .diff--better {
    background: var(--color-success-subtle);
    color: var(--color-success);
  }

  .diff--worse {
    background: var(--color-error-subtle);
    color: var(--color-error);
  }

  .diff--more {
    background: var(--color-primary-subtle);
    color: var(--color-primary);
  }

  .diff--less {
    background: var(--color-warning-subtle);
    color: var(--color-warning);
  }

  .config-diff {
    font-family: var(--font-mono);
  }

  .file-diff-list {
    font-family: var(--font-mono);
    font-size: var(--text-sm);
  }

  .file-diff-item {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-1) 0;
    border-bottom: 1px solid var(--color-border-subtle);
  }

  .file-diff-item:last-child {
    border-bottom: none;
  }

  .file-status {
    width: 20px;
    text-align: center;
    font-weight: var(--font-bold);
    font-size: var(--text-base);
  }

  .icon-equal { color: var(--color-fg-muted); }
  .icon-modified { color: var(--color-primary); }
  .icon-removed { color: var(--color-error); }
  .icon-added { color: var(--color-success); }

  .file-path {
    color: var(--color-fg-default);
  }

  .empty-state {
    text-align: center;
    padding: var(--space-4);
    color: var(--color-fg-muted);
    font-style: italic;
  }

  /* Export Dialog */
  .export-dialog-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .export-dialog {
    background: var(--color-bg-surface);
    border: 1px solid var(--color-border-default);
    border-radius: var(--card-radius);
    width: 400px;
    max-width: 90vw;
    box-shadow: var(--shadow-lg);
  }

  .export-dialog__header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-4);
    border-bottom: 1px solid var(--color-border-muted);
  }

  .export-dialog__header h3 {
    margin: 0;
    font-size: var(--text-base);
    color: var(--color-fg-default);
  }

  .export-dialog__body {
    padding: var(--space-4);
  }

  .export-dialog__footer {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-2);
    padding: var(--space-4);
    border-top: 1px solid var(--color-border-muted);
  }

  .form-group {
    margin-bottom: var(--space-4);
  }

  .form-group label {
    display: block;
    margin-bottom: var(--space-2);
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    color: var(--color-fg-default);
  }

  .form-group select {
    width: 100%;
    height: var(--input-height-md);
    padding: 0 var(--input-padding-x);
    border: 1px solid var(--color-border-default);
    border-radius: var(--input-radius);
    background: var(--color-bg-input);
    color: var(--color-fg-default);
  }

  .checkbox-group {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-weight: normal !important;
    margin-bottom: 0 !important;
    cursor: pointer;
  }

  .checkbox-label input[type="checkbox"] {
    margin: 0;
  }

  /* Share URL */
  .share-url-display {
    padding: var(--space-4);
    background: var(--color-bg-elevated);
    border-top: 1px solid var(--color-border-muted);
  }

  .share-url-display label {
    display: block;
    margin-bottom: var(--space-2);
    font-size: var(--text-sm);
    color: var(--color-fg-default);
  }

  .share-url-input {
    display: flex;
    gap: var(--space-2);
  }

  .share-url-input input {
    flex: 1;
    height: var(--input-height-md);
    padding: 0 var(--input-padding-x);
    border: 1px solid var(--color-border-default);
    border-radius: var(--input-radius);
    background: var(--color-bg-input);
    color: var(--color-fg-default);
    font-family: var(--font-mono);
    font-size: var(--text-sm);
  }

  @media (max-width: 768px) {
    .comparison-grid {
      grid-template-columns: 1fr;
      gap: var(--space-1);
    }

    .comparison-label {
      font-weight: var(--font-semibold);
      color: var(--color-fg-default);
    }

    .header-actions {
      flex-wrap: wrap;
    }

    .export-dialog {
      width: 95vw;
    }
  }
</style>