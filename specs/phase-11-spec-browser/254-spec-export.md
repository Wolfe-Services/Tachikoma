# 254 - Spec Export

**Phase:** 11 - Spec Browser UI
**Spec ID:** 254
**Status:** Planned
**Dependencies:** 236-spec-browser-layout
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Create an export system for specifications that supports multiple formats (Markdown, HTML, PDF, JSON), batch exports, and customizable templates with options for including dependencies and related assets.

---

## Acceptance Criteria

- [x] Export single spec to multiple formats
- [x] Batch export multiple specs
- [x] Export with dependencies
- [x] Customizable export templates
- [x] Include/exclude code blocks
- [x] Export to ZIP archive
- [x] Progress indicator for large exports
- [x] Export history tracking

---

## Implementation Details

### 1. Types (src/lib/types/spec-export.ts)

```typescript
export type ExportFormat = 'markdown' | 'html' | 'pdf' | 'json' | 'docx';

export interface ExportOptions {
  format: ExportFormat;
  includeMetadata: boolean;
  includeDependencies: boolean;
  includeRelated: boolean;
  includeCodeBlocks: boolean;
  includeToc: boolean;
  templateId?: string;
  customStyles?: string;
  pageSize?: 'A4' | 'Letter';
  orientation?: 'portrait' | 'landscape';
}

export interface ExportJob {
  id: string;
  specIds: string[];
  options: ExportOptions;
  status: ExportStatus;
  progress: number;
  outputPath?: string;
  error?: string;
  startedAt: string;
  completedAt?: string;
}

export type ExportStatus = 'pending' | 'processing' | 'completed' | 'failed';

export interface ExportTemplate {
  id: string;
  name: string;
  description: string;
  format: ExportFormat;
  headerTemplate: string;
  footerTemplate: string;
  styles: string;
  isDefault: boolean;
}

export interface ExportPreview {
  content: string;
  pageCount?: number;
  estimatedSize: string;
}

export interface ExportHistory {
  id: string;
  specIds: string[];
  format: ExportFormat;
  outputPath: string;
  timestamp: string;
  fileSize: number;
}
```

### 2. Spec Export Dialog Component (src/lib/components/spec-browser/SpecExportDialog.svelte)

```svelte
<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import type {
    ExportOptions,
    ExportFormat,
    ExportJob,
    ExportTemplate,
    ExportPreview,
    ExportHistory,
  } from '$lib/types/spec-export';
  import { ipcRenderer } from '$lib/ipc';
  import { fade, slide } from 'svelte/transition';

  export let open = false;
  export let specIds: string[] = [];
  export let initialFormat: ExportFormat = 'markdown';

  const dispatch = createEventDispatcher<{
    export: ExportJob;
    close: void;
  }>();

  let options: ExportOptions = {
    format: initialFormat,
    includeMetadata: true,
    includeDependencies: false,
    includeRelated: false,
    includeCodeBlocks: true,
    includeToc: true,
    pageSize: 'A4',
    orientation: 'portrait',
  };

  let templates: ExportTemplate[] = [];
  let selectedTemplate: ExportTemplate | null = null;
  let preview: ExportPreview | null = null;
  let isExporting = false;
  let exportJob: ExportJob | null = null;
  let history: ExportHistory[] = [];
  let showHistory = false;
  let showAdvanced = false;

  const formatLabels: Record<ExportFormat, string> = {
    markdown: 'Markdown (.md)',
    html: 'HTML (.html)',
    pdf: 'PDF (.pdf)',
    json: 'JSON (.json)',
    docx: 'Word Document (.docx)',
  };

  const formatIcons: Record<ExportFormat, string> = {
    markdown: '📝',
    html: '🌐',
    pdf: '📄',
    json: '{}',
    docx: '📃',
  };

  async function loadTemplates() {
    templates = await ipcRenderer.invoke('export:get-templates');
    const defaultTemplate = templates.find(t => t.isDefault && t.format === options.format);
    if (defaultTemplate) {
      selectedTemplate = defaultTemplate;
      options.templateId = defaultTemplate.id;
    }
  }

  async function loadHistory() {
    history = await ipcRenderer.invoke('export:get-history');
  }

  async function generatePreview() {
    preview = await ipcRenderer.invoke('export:preview', {
      specIds,
      options,
    });
  }

  async function startExport() {
    isExporting = true;
    exportJob = null;

    try {
      // Open save dialog
      const savePath = await ipcRenderer.invoke('dialog:save', {
        title: 'Export Specs',
        defaultPath: getDefaultFilename(),
        filters: getFileFilters(),
      });

      if (!savePath) {
        isExporting = false;
        return;
      }

      // Start export job
      exportJob = await ipcRenderer.invoke('export:start', {
        specIds,
        options,
        outputPath: savePath,
      });

      // Listen for progress updates
      const unsubscribe = ipcRenderer.on('export:progress', (event, data) => {
        if (data.jobId === exportJob?.id) {
          exportJob = { ...exportJob, ...data };
        }
      });

      // Wait for completion
      const result = await ipcRenderer.invoke('export:wait', exportJob.id);
      exportJob = result;

      if (result.status === 'completed') {
        dispatch('export', result);
      }

      unsubscribe();
    } catch (error) {
      console.error('Export failed:', error);
      if (exportJob) {
        exportJob = { ...exportJob, status: 'failed', error: String(error) };
      }
    } finally {
      isExporting = false;
    }
  }

  function getDefaultFilename(): string {
    const timestamp = new Date().toISOString().split('T')[0];
    if (specIds.length === 1) {
      return `spec-${specIds[0]}-${timestamp}`;
    }
    return `specs-export-${timestamp}`;
  }

  function getFileFilters() {
    const filters: { name: string; extensions: string[] }[] = [];
    switch (options.format) {
      case 'markdown':
        filters.push({ name: 'Markdown', extensions: ['md'] });
        break;
      case 'html':
        filters.push({ name: 'HTML', extensions: ['html'] });
        break;
      case 'pdf':
        filters.push({ name: 'PDF', extensions: ['pdf'] });
        break;
      case 'json':
        filters.push({ name: 'JSON', extensions: ['json'] });
        break;
      case 'docx':
        filters.push({ name: 'Word Document', extensions: ['docx'] });
        break;
    }
    if (specIds.length > 1 || options.includeDependencies) {
      filters.push({ name: 'ZIP Archive', extensions: ['zip'] });
    }
    return filters;
  }

  async function openExportedFile() {
    if (exportJob?.outputPath) {
      await ipcRenderer.invoke('shell:open-path', exportJob.outputPath);
    }
  }

  async function openInFolder() {
    if (exportJob?.outputPath) {
      await ipcRenderer.invoke('shell:show-item-in-folder', exportJob.outputPath);
    }
  }

  function close() {
    open = false;
    dispatch('close');
    resetState();
  }

  function resetState() {
    preview = null;
    exportJob = null;
    isExporting = false;
  }

  onMount(() => {
    loadTemplates();
    loadHistory();
  });

  $: if (open) generatePreview();
  $: if (options.format) {
    const template = templates.find(t => t.isDefault && t.format === options.format);
    if (template) {
      selectedTemplate = template;
      options.templateId = template.id;
    }
  }
</script>

{#if open}
  <div
    class="export-overlay"
    on:click={close}
    transition:fade={{ duration: 150 }}
  >
    <div class="export-dialog" on:click|stopPropagation>
      <header class="export-dialog__header">
        <h2>Export Specifications</h2>
        <button class="close-btn" on:click={close}>
          <svg width="20" height="20" viewBox="0 0 20 20" fill="currentColor">
            <path d="M5.293 5.293a1 1 0 011.414 0L10 8.586l3.293-3.293a1 1 0 111.414 1.414L11.414 10l3.293 3.293a1 1 0 01-1.414 1.414L10 11.414l-3.293 3.293a1 1 0 01-1.414-1.414L8.586 10 5.293 6.707a1 1 0 010-1.414z"/>
          </svg>
        </button>
      </header>

      <div class="export-dialog__content">
        <div class="export-main">
          <section class="export-section">
            <h3>Selected Specs ({specIds.length})</h3>
            <div class="selected-specs">
              {#each specIds.slice(0, 5) as specId}
                <span class="spec-tag">{specId}</span>
              {/each}
              {#if specIds.length > 5}
                <span class="more-specs">+{specIds.length - 5} more</span>
              {/if}
            </div>
          </section>

          <section class="export-section">
            <h3>Export Format</h3>
            <div class="format-grid">
              {#each Object.entries(formatLabels) as [format, label]}
                <button
                  class="format-option"
                  class:selected={options.format === format}
                  on:click={() => { options.format = format; }}
                >
                  <span class="format-icon">{formatIcons[format]}</span>
                  <span class="format-label">{label}</span>
                </button>
              {/each}
            </div>
          </section>

          <section class="export-section">
            <h3>Options</h3>
            <div class="options-list">
              <label class="option-item">
                <input type="checkbox" bind:checked={options.includeMetadata} />
                <span>Include metadata (frontmatter)</span>
              </label>
              <label class="option-item">
                <input type="checkbox" bind:checked={options.includeToc} />
                <span>Include table of contents</span>
              </label>
              <label class="option-item">
                <input type="checkbox" bind:checked={options.includeCodeBlocks} />
                <span>Include code blocks</span>
              </label>
              <label class="option-item">
                <input type="checkbox" bind:checked={options.includeDependencies} />
                <span>Include dependencies</span>
              </label>
              <label class="option-item">
                <input type="checkbox" bind:checked={options.includeRelated} />
                <span>Include related specs</span>
              </label>
            </div>
          </section>

          {#if options.format === 'pdf'}
            <section class="export-section">
              <h3>PDF Options</h3>
              <div class="pdf-options">
                <div class="option-group">
                  <label>Page Size</label>
                  <select bind:value={options.pageSize}>
                    <option value="A4">A4</option>
                    <option value="Letter">Letter</option>
                  </select>
                </div>
                <div class="option-group">
                  <label>Orientation</label>
                  <select bind:value={options.orientation}>
                    <option value="portrait">Portrait</option>
                    <option value="landscape">Landscape</option>
                  </select>
                </div>
              </div>
            </section>
          {/if}

          <button
            class="advanced-toggle"
            on:click={() => { showAdvanced = !showAdvanced; }}
          >
            {showAdvanced ? 'Hide' : 'Show'} Advanced Options
            <svg
              width="12"
              height="12"
              viewBox="0 0 12 12"
              class:rotated={showAdvanced}
            >
              <path d="M3 4.5l3 3 3-3" stroke="currentColor" stroke-width="1.5" fill="none"/>
            </svg>
          </button>

          {#if showAdvanced}
            <section class="export-section" transition:slide={{ duration: 150 }}>
              <h3>Template</h3>
              <select
                bind:value={options.templateId}
                on:change={() => {
                  selectedTemplate = templates.find(t => t.id === options.templateId) || null;
                }}
              >
                {#each templates.filter(t => t.format === options.format) as template}
                  <option value={template.id}>
                    {template.name} {template.isDefault ? '(Default)' : ''}
                  </option>
                {/each}
              </select>
              {#if selectedTemplate}
                <p class="template-description">{selectedTemplate.description}</p>
              {/if}
            </section>

            <section class="export-section" transition:slide={{ duration: 150 }}>
              <h3>Custom Styles</h3>
              <textarea
                bind:value={options.customStyles}
                placeholder="Add custom CSS styles..."
                rows="4"
              />
            </section>
          {/if}
        </div>

        <aside class="export-preview">
          <header class="preview-header">
            <h3>Preview</h3>
            <button class="refresh-btn" on:click={generatePreview}>
              Refresh
            </button>
          </header>

          {#if preview}
            <div class="preview-stats">
              {#if preview.pageCount}
                <span>{preview.pageCount} pages</span>
              {/if}
              <span>{preview.estimatedSize}</span>
            </div>
            <div class="preview-content">
              <pre>{preview.content.slice(0, 2000)}{preview.content.length > 2000 ? '\n...' : ''}</pre>
            </div>
          {:else}
            <div class="preview-loading">
              Generating preview...
            </div>
          {/if}
        </aside>
      </div>

      {#if exportJob}
        <div class="export-progress" transition:slide={{ duration: 150 }}>
          {#if exportJob.status === 'processing'}
            <div class="progress-bar">
              <div class="progress-fill" style="width: {exportJob.progress}%" />
            </div>
            <span class="progress-text">Exporting... {exportJob.progress}%</span>
          {:else if exportJob.status === 'completed'}
            <div class="export-success">
              <span class="success-icon">✅</span>
              <span>Export completed!</span>
              <div class="success-actions">
                <button on:click={openExportedFile}>Open File</button>
                <button on:click={openInFolder}>Show in Folder</button>
              </div>
            </div>
          {:else if exportJob.status === 'failed'}
            <div class="export-error">
              <span class="error-icon">❌</span>
              <span>Export failed: {exportJob.error}</span>
            </div>
          {/if}
        </div>
      {/if}

      <footer class="export-dialog__footer">
        <button
          class="history-btn"
          on:click={() => { showHistory = !showHistory; }}
        >
          History
        </button>
        <div class="footer-spacer" />
        <button class="cancel-btn" on:click={close}>
          Cancel
        </button>
        <button
          class="export-btn"
          on:click={startExport}
          disabled={isExporting || specIds.length === 0}
        >
          {isExporting ? 'Exporting...' : 'Export'}
        </button>
      </footer>
    </div>
  </div>
{/if}

{#if showHistory}
  <div
    class="history-overlay"
    on:click={() => { showHistory = false; }}
    transition:fade={{ duration: 150 }}
  >
    <div class="history-panel" on:click|stopPropagation>
      <header class="history-header">
        <h3>Export History</h3>
        <button on:click={() => { showHistory = false; }}>×</button>
      </header>
      <div class="history-list">
        {#if history.length === 0}
          <p class="empty-history">No export history</p>
        {:else}
          {#each history as item}
            <div class="history-item">
              <div class="history-item__info">
                <span class="history-format">{formatIcons[item.format]}</span>
                <span class="history-specs">{item.specIds.length} specs</span>
                <span class="history-date">
                  {new Date(item.timestamp).toLocaleDateString()}
                </span>
                <span class="history-size">{formatFileSize(item.fileSize)}</span>
              </div>
              <button
                class="history-open"
                on:click={() => ipcRenderer.invoke('shell:open-path', item.outputPath)}
              >
                Open
              </button>
            </div>
          {/each}
        {/if}
      </div>
    </div>
  </div>
{/if}

<script context="module">
  function formatFileSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }
</script>

<style>
  .export-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .export-dialog {
    width: 90%;
    max-width: 900px;
    max-height: 90vh;
    background: var(--color-bg-primary);
    border-radius: 12px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .export-dialog__header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 20px 24px;
    border-bottom: 1px solid var(--color-border);
  }

  .export-dialog__header h2 {
    margin: 0;
    font-size: 20px;
  }

  .close-btn {
    padding: 8px;
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
    border-radius: 6px;
  }

  .close-btn:hover {
    background: var(--color-bg-hover);
  }

  .export-dialog__content {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .export-main {
    flex: 1;
    padding: 24px;
    overflow-y: auto;
  }

  .export-section {
    margin-bottom: 24px;
  }

  .export-section h3 {
    margin: 0 0 12px 0;
    font-size: 14px;
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .selected-specs {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .spec-tag {
    padding: 4px 10px;
    background: var(--color-bg-secondary);
    border-radius: 4px;
    font-family: monospace;
    font-size: 12px;
  }

  .more-specs {
    padding: 4px 10px;
    color: var(--color-text-muted);
    font-size: 12px;
  }

  .format-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
    gap: 12px;
  }

  .format-option {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 16px;
    border: 1px solid var(--color-border);
    background: transparent;
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .format-option:hover {
    border-color: var(--color-primary);
  }

  .format-option.selected {
    border-color: var(--color-primary);
    background: rgba(33, 150, 243, 0.1);
  }

  .format-icon {
    font-size: 24px;
  }

  .format-label {
    font-size: 12px;
    color: var(--color-text-secondary);
  }

  .options-list {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .option-item {
    display: flex;
    align-items: center;
    gap: 10px;
    cursor: pointer;
    font-size: 13px;
  }

  .option-item input {
    cursor: pointer;
  }

  .pdf-options {
    display: flex;
    gap: 16px;
  }

  .option-group {
    flex: 1;
  }

  .option-group label {
    display: block;
    margin-bottom: 6px;
    font-size: 12px;
    color: var(--color-text-muted);
  }

  .option-group select {
    width: 100%;
    padding: 8px 12px;
    border: 1px solid var(--color-border);
    border-radius: 4px;
    background: var(--color-bg-secondary);
    color: var(--color-text-primary);
  }

  .advanced-toggle {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 0;
    border: none;
    background: transparent;
    color: var(--color-text-secondary);
    font-size: 13px;
    cursor: pointer;
  }

  .advanced-toggle:hover {
    color: var(--color-primary);
  }

  .advanced-toggle svg {
    transition: transform 0.15s ease;
  }

  .advanced-toggle svg.rotated {
    transform: rotate(180deg);
  }

  .export-section select {
    width: 100%;
    padding: 8px 12px;
    border: 1px solid var(--color-border);
    border-radius: 4px;
    background: var(--color-bg-secondary);
    color: var(--color-text-primary);
  }

  .template-description {
    margin: 8px 0 0 0;
    font-size: 12px;
    color: var(--color-text-muted);
  }

  .export-section textarea {
    width: 100%;
    padding: 10px 12px;
    border: 1px solid var(--color-border);
    border-radius: 4px;
    background: var(--color-bg-secondary);
    color: var(--color-text-primary);
    font-family: monospace;
    font-size: 12px;
    resize: vertical;
  }

  .export-preview {
    width: 300px;
    border-left: 1px solid var(--color-border);
    display: flex;
    flex-direction: column;
    background: var(--color-bg-secondary);
  }

  .preview-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    border-bottom: 1px solid var(--color-border);
  }

  .preview-header h3 {
    margin: 0;
    font-size: 13px;
  }

  .refresh-btn {
    padding: 4px 10px;
    border: 1px solid var(--color-border);
    background: transparent;
    border-radius: 4px;
    font-size: 11px;
    cursor: pointer;
  }

  .preview-stats {
    display: flex;
    gap: 12px;
    padding: 8px 16px;
    font-size: 11px;
    color: var(--color-text-muted);
    border-bottom: 1px solid var(--color-border);
  }

  .preview-content {
    flex: 1;
    overflow: auto;
    padding: 16px;
  }

  .preview-content pre {
    margin: 0;
    font-size: 10px;
    line-height: 1.4;
    white-space: pre-wrap;
    word-break: break-word;
  }

  .preview-loading {
    padding: 24px;
    text-align: center;
    color: var(--color-text-muted);
  }

  .export-progress {
    padding: 16px 24px;
    border-top: 1px solid var(--color-border);
    background: var(--color-bg-secondary);
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
    color: var(--color-text-secondary);
  }

  .export-success,
  .export-error {
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 14px;
  }

  .success-actions {
    display: flex;
    gap: 8px;
    margin-left: auto;
  }

  .success-actions button {
    padding: 6px 12px;
    border: 1px solid var(--color-border);
    background: transparent;
    border-radius: 4px;
    font-size: 12px;
    cursor: pointer;
  }

  .export-error {
    color: var(--color-error);
  }

  .export-dialog__footer {
    display: flex;
    align-items: center;
    padding: 16px 24px;
    border-top: 1px solid var(--color-border);
    background: var(--color-bg-secondary);
  }

  .history-btn {
    padding: 8px 14px;
    border: 1px solid var(--color-border);
    background: transparent;
    border-radius: 6px;
    font-size: 13px;
    cursor: pointer;
  }

  .footer-spacer {
    flex: 1;
  }

  .cancel-btn {
    padding: 10px 20px;
    border: 1px solid var(--color-border);
    background: transparent;
    border-radius: 6px;
    font-size: 14px;
    cursor: pointer;
    margin-right: 12px;
  }

  .export-btn {
    padding: 10px 24px;
    border: none;
    background: var(--color-primary);
    color: white;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
  }

  .export-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* History Panel */
  .history-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.3);
    z-index: 1001;
  }

  .history-panel {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    max-height: 50vh;
    background: var(--color-bg-primary);
    border-radius: 12px 12px 0 0;
    display: flex;
    flex-direction: column;
  }

  .history-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px;
    border-bottom: 1px solid var(--color-border);
  }

  .history-header h3 {
    margin: 0;
    font-size: 16px;
  }

  .history-header button {
    padding: 6px 12px;
    border: none;
    background: transparent;
    font-size: 20px;
    cursor: pointer;
  }

  .history-list {
    flex: 1;
    overflow-y: auto;
    padding: 16px;
  }

  .empty-history {
    text-align: center;
    color: var(--color-text-muted);
  }

  .history-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    margin-bottom: 8px;
  }

  .history-item__info {
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 13px;
  }

  .history-format {
    font-size: 18px;
  }

  .history-date,
  .history-size {
    color: var(--color-text-muted);
    font-size: 12px;
  }

  .history-open {
    padding: 6px 12px;
    border: 1px solid var(--color-border);
    background: transparent;
    border-radius: 4px;
    font-size: 12px;
    cursor: pointer;
  }
</style>
```

---

## Testing Requirements

1. All export formats work
2. Options apply correctly
3. Preview generates accurately
4. Progress updates in real-time
5. Batch export works
6. History records exports
7. File opens successfully

---

## Related Specs

- Depends on: [236-spec-browser-layout.md](236-spec-browser-layout.md)
- Next: [255-spec-browser-tests.md](255-spec-browser-tests.md)
