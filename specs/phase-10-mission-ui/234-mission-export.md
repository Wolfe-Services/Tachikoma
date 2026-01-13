# 234 - Mission Export Component

**Phase:** 10 - Mission Panel UI
**Spec ID:** 234
**Status:** Planned
**Dependencies:** 216-mission-layout, 217-mission-state
**Estimated Context:** ~9% of Sonnet window

---

## Objective

Create a mission export component that allows users to export mission data, logs, artifacts, and reports in various formats for archival and sharing.

---

## Acceptance Criteria

- [x] Export format selection (JSON, Markdown, HTML)
- [x] Content selection (config, logs, diffs, costs)
- [x] Archive generation (ZIP)
- [x] Progress indicator for large exports
- [x] Shareable link generation
- [x] Import/restore functionality

---

## Implementation Details

### 1. Types (src/lib/types/export.ts)

```typescript
export type ExportFormat = 'json' | 'markdown' | 'html' | 'zip';

export interface ExportOptions {
  format: ExportFormat;
  includeConfig: boolean;
  includeLogs: boolean;
  includeDiffs: boolean;
  includeCosts: boolean;
  includeCheckpoints: boolean;
  dateRange?: { from: string; to: string };
}

export interface ExportProgress {
  status: 'preparing' | 'exporting' | 'complete' | 'error';
  progress: number;
  currentItem: string;
  totalItems: number;
}

export interface ExportResult {
  filename: string;
  size: number;
  url: string;
  expiresAt?: string;
}
```

### 2. Mission Export Component (src/lib/components/mission/MissionExport.svelte)

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { ExportFormat, ExportOptions, ExportProgress, ExportResult } from '$lib/types/export';
  import { ipcRenderer } from '$lib/ipc';

  export let missionId: string;

  const dispatch = createEventDispatcher<{
    close: void;
    complete: ExportResult;
  }>();

  let options: ExportOptions = {
    format: 'json',
    includeConfig: true,
    includeLogs: true,
    includeDiffs: true,
    includeCosts: true,
    includeCheckpoints: false,
  };

  let progress: ExportProgress | null = null;
  let result: ExportResult | null = null;
  let error: string | null = null;

  const formats: { value: ExportFormat; label: string; description: string }[] = [
    { value: 'json', label: 'JSON', description: 'Machine-readable format' },
    { value: 'markdown', label: 'Markdown', description: 'Human-readable report' },
    { value: 'html', label: 'HTML', description: 'Styled web page' },
    { value: 'zip', label: 'Archive', description: 'Complete bundle with files' },
  ];

  async function startExport() {
    progress = { status: 'preparing', progress: 0, currentItem: '', totalItems: 0 };
    error = null;

    try {
      result = await ipcRenderer.invoke('mission:export', { missionId, options });
      progress = { status: 'complete', progress: 100, currentItem: '', totalItems: 0 };
      dispatch('complete', result);
    } catch (err) {
      error = err instanceof Error ? err.message : 'Export failed';
      progress = { status: 'error', progress: 0, currentItem: '', totalItems: 0 };
    }
  }

  function downloadResult() {
    if (result) {
      const a = document.createElement('a');
      a.href = result.url;
      a.download = result.filename;
      a.click();
    }
  }

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }
</script>

<div class="mission-export">
  <header class="export-header">
    <h3>Export Mission</h3>
    <button class="close-btn" on:click={() => dispatch('close')}>×</button>
  </header>

  {#if !result}
    <div class="export-options">
      <div class="option-group">
        <label class="option-label">Format</label>
        <div class="format-options">
          {#each formats as format}
            <label class="format-option" class:selected={options.format === format.value}>
              <input
                type="radio"
                bind:group={options.format}
                value={format.value}
              />
              <span class="format-label">{format.label}</span>
              <span class="format-desc">{format.description}</span>
            </label>
          {/each}
        </div>
      </div>

      <div class="option-group">
        <label class="option-label">Include</label>
        <div class="include-options">
          <label class="checkbox-option">
            <input type="checkbox" bind:checked={options.includeConfig} />
            Configuration
          </label>
          <label class="checkbox-option">
            <input type="checkbox" bind:checked={options.includeLogs} />
            Logs
          </label>
          <label class="checkbox-option">
            <input type="checkbox" bind:checked={options.includeDiffs} />
            File Changes
          </label>
          <label class="checkbox-option">
            <input type="checkbox" bind:checked={options.includeCosts} />
            Cost Report
          </label>
          <label class="checkbox-option">
            <input type="checkbox" bind:checked={options.includeCheckpoints} />
            Checkpoints
          </label>
        </div>
      </div>
    </div>

    {#if progress && progress.status !== 'complete'}
      <div class="export-progress">
        <div class="progress-bar">
          <div class="progress-fill" style="width: {progress.progress}%"></div>
        </div>
        <span class="progress-text">{progress.currentItem || progress.status}</span>
      </div>
    {/if}

    {#if error}
      <div class="export-error">{error}</div>
    {/if}

    <footer class="export-footer">
      <button class="cancel-btn" on:click={() => dispatch('close')}>Cancel</button>
      <button
        class="export-btn"
        on:click={startExport}
        disabled={progress?.status === 'preparing' || progress?.status === 'exporting'}
      >
        {progress?.status === 'exporting' ? 'Exporting...' : 'Export'}
      </button>
    </footer>
  {:else}
    <div class="export-result">
      <div class="result-icon">✓</div>
      <h4>Export Complete</h4>
      <p class="result-filename">{result.filename}</p>
      <p class="result-size">{formatSize(result.size)}</p>

      <button class="download-btn" on:click={downloadResult}>
        Download
      </button>
    </div>
  {/if}
</div>

<style>
  .mission-export {
    width: 400px;
    background: var(--color-bg-primary);
    border-radius: 12px;
    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.2);
  }

  .export-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px;
    border-bottom: 1px solid var(--color-border);
  }

  .export-header h3 {
    margin: 0;
    font-size: 16px;
  }

  .close-btn {
    border: none;
    background: transparent;
    font-size: 20px;
    cursor: pointer;
    color: var(--color-text-muted);
  }

  .export-options {
    padding: 16px;
  }

  .option-group {
    margin-bottom: 20px;
  }

  .option-label {
    display: block;
    font-size: 13px;
    font-weight: 500;
    color: var(--color-text-secondary);
    margin-bottom: 8px;
  }

  .format-options {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 8px;
  }

  .format-option {
    display: flex;
    flex-direction: column;
    padding: 12px;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    cursor: pointer;
  }

  .format-option.selected {
    border-color: var(--color-primary);
    background: var(--color-bg-active);
  }

  .format-option input {
    display: none;
  }

  .format-label {
    font-weight: 500;
    font-size: 14px;
  }

  .format-desc {
    font-size: 11px;
    color: var(--color-text-muted);
  }

  .include-options {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .checkbox-option {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
    cursor: pointer;
  }

  .export-progress {
    padding: 0 16px 16px;
  }

  .progress-bar {
    height: 8px;
    background: var(--color-bg-hover);
    border-radius: 4px;
    overflow: hidden;
    margin-bottom: 8px;
  }

  .progress-fill {
    height: 100%;
    background: var(--color-primary);
    transition: width 0.3s ease;
  }

  .progress-text {
    font-size: 12px;
    color: var(--color-text-muted);
  }

  .export-error {
    margin: 0 16px 16px;
    padding: 12px;
    background: rgba(244, 67, 54, 0.1);
    border-radius: 6px;
    color: var(--color-error);
    font-size: 13px;
  }

  .export-footer {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
    padding: 16px;
    border-top: 1px solid var(--color-border);
  }

  .cancel-btn {
    padding: 8px 16px;
    border: 1px solid var(--color-border);
    background: transparent;
    border-radius: 6px;
    cursor: pointer;
  }

  .export-btn {
    padding: 8px 24px;
    border: none;
    background: var(--color-primary);
    color: white;
    border-radius: 6px;
    cursor: pointer;
  }

  .export-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .export-result {
    padding: 32px;
    text-align: center;
  }

  .result-icon {
    width: 48px;
    height: 48px;
    background: var(--color-success);
    color: white;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 24px;
    margin: 0 auto 16px;
  }

  .result-filename {
    font-family: monospace;
    font-size: 14px;
  }

  .result-size {
    color: var(--color-text-muted);
    font-size: 13px;
  }

  .download-btn {
    margin-top: 16px;
    padding: 10px 32px;
    border: none;
    background: var(--color-primary);
    color: white;
    border-radius: 6px;
    font-size: 14px;
    cursor: pointer;
  }
</style>
```

---

## Testing Requirements

1. Format selection works
2. Content options toggle
3. Progress displays during export
4. Error handling works
5. Download triggers correctly

---

## Related Specs

- Depends on: [217-mission-state.md](217-mission-state.md)
- Next: [235-mission-tests.md](235-mission-tests.md)
