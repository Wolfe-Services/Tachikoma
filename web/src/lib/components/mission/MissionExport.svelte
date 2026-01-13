<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { ExportFormat, ExportOptions, ExportProgress, ExportResult, ShareableLink } from '$lib/types/export';
  import { ipc } from '$lib/ipc';

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
  let shareableLink: ShareableLink | null = null;

  const formats: { value: ExportFormat; label: string; description: string }[] = [
    { value: 'json', label: 'JSON', description: 'Machine-readable format' },
    { value: 'markdown', label: 'Markdown', description: 'Human-readable report' },
    { value: 'html', label: 'HTML', description: 'Styled web page' },
    { value: 'zip', label: 'Archive', description: 'Complete bundle with files' },
  ];

  async function startExport() {
    progress = { status: 'preparing', progress: 0, currentItem: 'Preparing export...', totalItems: 0 };
    error = null;
    result = null;
    shareableLink = null;

    try {
      // Simulate progress updates for better UX
      const progressUpdates = [
        { progress: 10, currentItem: 'Collecting configuration...' },
        { progress: 25, currentItem: 'Gathering logs...' },
        { progress: 40, currentItem: 'Processing file changes...' },
        { progress: 60, currentItem: 'Calculating costs...' },
        { progress: 80, currentItem: 'Creating archive...' },
        { progress: 95, currentItem: 'Finalizing export...' },
      ];

      for (const update of progressUpdates) {
        progress = { 
          status: 'exporting', 
          ...update,
          totalItems: progressUpdates.length 
        };
        await new Promise(resolve => setTimeout(resolve, 200));
      }

      result = await ipc.invoke('mission:export', { missionId, options });
      progress = { status: 'complete', progress: 100, currentItem: 'Export complete', totalItems: progressUpdates.length };
      dispatch('complete', result);
    } catch (err) {
      error = err instanceof Error ? err.message : 'Export failed';
      progress = { status: 'error', progress: 0, currentItem: 'Export failed', totalItems: 0 };
    }
  }

  async function generateShareableLink() {
    if (!result) return;
    
    try {
      shareableLink = await ipc.invoke('mission:generateShareableLink', { 
        exportId: result.filename,
        expiresIn: '24h' 
      });
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to generate shareable link';
    }
  }

  function downloadResult() {
    if (result) {
      const a = document.createElement('a');
      a.href = result.url;
      a.download = result.filename;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
    }
  }

  async function copyShareableLink() {
    if (shareableLink) {
      try {
        await navigator.clipboard.writeText(shareableLink.url);
        // Could add a toast notification here
      } catch (err) {
        console.error('Failed to copy link:', err);
      }
    }
  }

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  function getSelectedContentCount(): number {
    return [
      options.includeConfig,
      options.includeLogs,
      options.includeDiffs,
      options.includeCosts,
      options.includeCheckpoints
    ].filter(Boolean).length;
  }

  $: hasContentSelected = getSelectedContentCount() > 0;
</script>

<div class="mission-export">
  <header class="export-header">
    <h3>Export Mission</h3>
    <button class="close-btn" on:click={() => dispatch('close')} aria-label="Close">×</button>
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
                name="export-format"
              />
              <span class="format-label">{format.label}</span>
              <span class="format-desc">{format.description}</span>
            </label>
          {/each}
        </div>
      </div>

      <div class="option-group">
        <label class="option-label">Include ({getSelectedContentCount()} selected)</label>
        <div class="include-options">
          <label class="checkbox-option">
            <input type="checkbox" bind:checked={options.includeConfig} />
            <span>Configuration</span>
          </label>
          <label class="checkbox-option">
            <input type="checkbox" bind:checked={options.includeLogs} />
            <span>Logs</span>
          </label>
          <label class="checkbox-option">
            <input type="checkbox" bind:checked={options.includeDiffs} />
            <span>File Changes</span>
          </label>
          <label class="checkbox-option">
            <input type="checkbox" bind:checked={options.includeCosts} />
            <span>Cost Report</span>
          </label>
          <label class="checkbox-option">
            <input type="checkbox" bind:checked={options.includeCheckpoints} />
            <span>Checkpoints</span>
          </label>
        </div>
      </div>
    </div>

    {#if progress && progress.status !== 'complete'}
      <div class="export-progress">
        <div class="progress-bar">
          <div class="progress-fill" style="width: {progress.progress}%"></div>
        </div>
        <div class="progress-info">
          <span class="progress-text">{progress.currentItem}</span>
          <span class="progress-percent">{progress.progress}%</span>
        </div>
      </div>
    {/if}

    {#if error}
      <div class="export-error">
        <span class="error-icon">⚠</span>
        {error}
      </div>
    {/if}

    <footer class="export-footer">
      <button class="cancel-btn" on:click={() => dispatch('close')}>Cancel</button>
      <button
        class="export-btn"
        on:click={startExport}
        disabled={progress?.status === 'preparing' || progress?.status === 'exporting' || !hasContentSelected}
      >
        {progress?.status === 'exporting' ? 'Exporting...' : 'Export'}
      </button>
    </footer>
  {:else}
    <div class="export-result">
      <div class="result-icon">✓</div>
      <h4>Export Complete</h4>
      <div class="result-info">
        <p class="result-filename">{result.filename}</p>
        <p class="result-size">{formatSize(result.size)}</p>
      </div>

      <div class="result-actions">
        <button class="download-btn" on:click={downloadResult}>
          <span class="btn-icon">↓</span>
          Download
        </button>
        
        {#if !shareableLink}
          <button class="share-btn" on:click={generateShareableLink}>
            <span class="btn-icon">🔗</span>
            Create Share Link
          </button>
        {:else}
          <div class="shareable-link">
            <input 
              type="text" 
              readonly 
              value={shareableLink.url} 
              class="link-input"
              on:click={(e) => e.currentTarget.select()}
            />
            <button class="copy-btn" on:click={copyShareableLink}>Copy</button>
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  .mission-export {
    width: 460px;
    background: var(--color-bg-primary);
    border-radius: var(--radius-xl);
    box-shadow: var(--shadow-xl);
    border: 1px solid var(--color-border);
  }

  .export-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-4);
    border-bottom: 1px solid var(--color-border);
  }

  .export-header h3 {
    margin: 0;
    font-size: var(--text-base);
    font-weight: var(--font-semibold);
    color: var(--color-text-primary);
  }

  .close-btn {
    border: none;
    background: transparent;
    font-size: var(--text-xl);
    cursor: pointer;
    color: var(--color-text-muted);
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-sm);
    transition: background-color var(--duration-200);
  }

  .close-btn:hover {
    background: var(--color-bg-hover);
  }

  .export-options {
    padding: var(--space-4);
  }

  .option-group {
    margin-bottom: var(--space-6);
  }

  .option-group:last-child {
    margin-bottom: 0;
  }

  .option-label {
    display: block;
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    color: var(--color-text-secondary);
    margin-bottom: var(--space-3);
  }

  .format-options {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: var(--space-2);
  }

  .format-option {
    display: flex;
    flex-direction: column;
    padding: var(--space-3);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: all var(--duration-200);
    background: var(--color-bg-surface);
  }

  .format-option:hover {
    background: var(--color-bg-hover);
  }

  .format-option.selected {
    border-color: var(--color-primary);
    background: var(--color-bg-active);
  }

  .format-option input {
    display: none;
  }

  .format-label {
    font-weight: var(--font-medium);
    font-size: var(--text-sm);
    color: var(--color-text-primary);
  }

  .format-desc {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
    margin-top: var(--space-1);
  }

  .include-options {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-3);
  }

  .checkbox-option {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-sm);
    cursor: pointer;
    color: var(--color-text-primary);
  }

  .checkbox-option input[type="checkbox"] {
    margin: 0;
  }

  .export-progress {
    padding: 0 var(--space-4) var(--space-4);
  }

  .progress-bar {
    height: 8px;
    background: var(--color-bg-hover);
    border-radius: var(--radius-full);
    overflow: hidden;
    margin-bottom: var(--space-3);
  }

  .progress-fill {
    height: 100%;
    background: var(--color-primary);
    transition: width var(--duration-300) var(--ease-out);
    border-radius: var(--radius-full);
  }

  .progress-info {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .progress-text {
    font-size: var(--text-sm);
    color: var(--color-text-secondary);
  }

  .progress-percent {
    font-size: var(--text-sm);
    color: var(--color-text-muted);
    font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
  }

  .export-error {
    margin: 0 var(--space-4) var(--space-4);
    padding: var(--space-3);
    background: rgba(244, 67, 54, 0.1);
    border-radius: var(--radius-md);
    color: var(--color-error);
    font-size: var(--text-sm);
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .error-icon {
    font-size: var(--text-base);
  }

  .export-footer {
    display: flex;
    gap: var(--space-2);
    justify-content: flex-end;
    padding: var(--space-4);
    border-top: 1px solid var(--color-border);
  }

  .cancel-btn {
    padding: var(--space-2) var(--space-4);
    border: 1px solid var(--color-border);
    background: var(--color-bg-surface);
    color: var(--color-text-primary);
    border-radius: var(--radius-md);
    cursor: pointer;
    font-size: var(--text-sm);
    transition: all var(--duration-200);
  }

  .cancel-btn:hover {
    background: var(--color-bg-hover);
  }

  .export-btn {
    padding: var(--space-2) var(--space-6);
    border: none;
    background: var(--color-primary);
    color: white;
    border-radius: var(--radius-md);
    cursor: pointer;
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    transition: all var(--duration-200);
  }

  .export-btn:hover:not(:disabled) {
    background: var(--color-primary-hover);
  }

  .export-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .export-result {
    padding: var(--space-8) var(--space-4);
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
    font-size: var(--text-xl);
    font-weight: var(--font-bold);
    margin: 0 auto var(--space-4);
  }

  .result-info h4 {
    margin: 0 0 var(--space-4) 0;
    font-size: var(--text-lg);
    font-weight: var(--font-semibold);
    color: var(--color-text-primary);
  }

  .result-filename {
    font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
    font-size: var(--text-sm);
    color: var(--color-text-primary);
    margin: 0 0 var(--space-1) 0;
  }

  .result-size {
    color: var(--color-text-muted);
    font-size: var(--text-sm);
    margin: 0 0 var(--space-6) 0;
  }

  .result-actions {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    align-items: center;
  }

  .download-btn, .share-btn {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-6);
    border: none;
    background: var(--color-primary);
    color: white;
    border-radius: var(--radius-md);
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    cursor: pointer;
    transition: all var(--duration-200);
  }

  .download-btn:hover, .share-btn:hover {
    background: var(--color-primary-hover);
  }

  .share-btn {
    background: var(--color-bg-surface);
    color: var(--color-text-primary);
    border: 1px solid var(--color-border);
  }

  .share-btn:hover {
    background: var(--color-bg-hover);
  }

  .btn-icon {
    font-size: var(--text-base);
  }

  .shareable-link {
    display: flex;
    gap: var(--space-2);
    width: 100%;
    max-width: 300px;
  }

  .link-input {
    flex: 1;
    padding: var(--space-2) var(--space-3);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    font-size: var(--text-xs);
    font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
    background: var(--color-bg-surface);
    color: var(--color-text-primary);
  }

  .copy-btn {
    padding: var(--space-2) var(--space-3);
    border: 1px solid var(--color-border);
    background: var(--color-bg-surface);
    color: var(--color-text-primary);
    border-radius: var(--radius-md);
    cursor: pointer;
    font-size: var(--text-xs);
    transition: all var(--duration-200);
  }

  .copy-btn:hover {
    background: var(--color-bg-hover);
  }
</style>