<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { ExportProgress } from '$lib/types/export';
  import { ipc } from '$lib/ipc';

  export let visible: boolean = false;

  const dispatch = createEventDispatcher<{
    close: void;
    imported: { missionId: string };
  }>();

  let dragActive = false;
  let files: FileList | null = null;
  let progress: ExportProgress | null = null;
  let error: string | null = null;

  function handleDragEnter(e: DragEvent) {
    e.preventDefault();
    e.stopPropagation();
    dragActive = true;
  }

  function handleDragLeave(e: DragEvent) {
    e.preventDefault();
    e.stopPropagation();
    if (!e.relatedTarget || !e.currentTarget.contains(e.relatedTarget as Node)) {
      dragActive = false;
    }
  }

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    e.stopPropagation();
  }

  function handleDrop(e: DragEvent) {
    e.preventDefault();
    e.stopPropagation();
    dragActive = false;
    
    const droppedFiles = e.dataTransfer?.files;
    if (droppedFiles && droppedFiles.length > 0) {
      files = droppedFiles;
    }
  }

  function handleFileSelect(e: Event) {
    const target = e.target as HTMLInputElement;
    if (target.files && target.files.length > 0) {
      files = target.files;
    }
  }

  async function importMission() {
    if (!files || files.length === 0) return;

    const file = files[0];
    progress = { status: 'preparing', progress: 0, currentItem: 'Reading file...', totalItems: 0 };
    error = null;

    try {
      // Convert file to base64 for IPC
      const buffer = await file.arrayBuffer();
      const base64 = btoa(String.fromCharCode(...new Uint8Array(buffer)));

      progress = { status: 'exporting', progress: 25, currentItem: 'Validating format...', totalItems: 4 };
      await new Promise(resolve => setTimeout(resolve, 200));

      progress = { status: 'exporting', progress: 50, currentItem: 'Restoring configuration...', totalItems: 4 };
      await new Promise(resolve => setTimeout(resolve, 300));

      progress = { status: 'exporting', progress: 75, currentItem: 'Importing data...', totalItems: 4 };
      await new Promise(resolve => setTimeout(resolve, 300));

      const result = await ipc.invoke('mission:import', { 
        filename: file.name,
        data: base64,
        size: file.size
      });

      progress = { status: 'complete', progress: 100, currentItem: 'Import complete', totalItems: 4 };
      
      setTimeout(() => {
        dispatch('imported', { missionId: result.missionId });
      }, 500);

    } catch (err) {
      error = err instanceof Error ? err.message : 'Import failed';
      progress = { status: 'error', progress: 0, currentItem: 'Import failed', totalItems: 0 };
    }
  }

  function clearSelection() {
    files = null;
    progress = null;
    error = null;
  }

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  function isValidFile(file: File): boolean {
    const validTypes = ['application/json', 'application/zip', 'text/markdown'];
    const validExtensions = ['.json', '.zip', '.md', '.html'];
    
    return validTypes.includes(file.type) || 
           validExtensions.some(ext => file.name.toLowerCase().endsWith(ext));
  }

  $: selectedFile = files?.[0];
  $: isFileValid = selectedFile ? isValidFile(selectedFile) : false;
</script>

{#if visible}
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div class="import-overlay" on:click={() => dispatch('close')}>
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div class="mission-import" on:click={(e) => e.stopPropagation()}>
      <header class="import-header">
        <h3>Import Mission</h3>
        <button class="close-btn" on:click={() => dispatch('close')} aria-label="Close">×</button>
      </header>

      <div class="import-content">
        {#if !files}
          <div 
            class="drop-zone"
            class:drag-active={dragActive}
            on:dragenter={handleDragEnter}
            on:dragleave={handleDragLeave}
            on:dragover={handleDragOver}
            on:drop={handleDrop}
            role="button"
            tabindex="0"
          >
            <div class="drop-zone-content">
              <div class="upload-icon">📁</div>
              <h4>Drop files here or click to browse</h4>
              <p>Supports JSON, ZIP, Markdown, and HTML exports</p>
              <input 
                type="file" 
                accept=".json,.zip,.md,.html" 
                on:change={handleFileSelect}
                class="file-input"
                id="file-input"
              />
              <label for="file-input" class="browse-btn">Browse Files</label>
            </div>
          </div>
        {:else}
          <div class="file-preview">
            <div class="file-info">
              <div class="file-icon" class:invalid={!isFileValid}>
                {selectedFile.name.endsWith('.zip') ? '📦' : 
                 selectedFile.name.endsWith('.json') ? '📄' : 
                 selectedFile.name.endsWith('.md') ? '📝' : '🌐'}
              </div>
              <div class="file-details">
                <p class="file-name">{selectedFile.name}</p>
                <p class="file-size">{formatSize(selectedFile.size)}</p>
                {#if !isFileValid}
                  <p class="file-error">Unsupported file type</p>
                {/if}
              </div>
              <button class="clear-btn" on:click={clearSelection} aria-label="Remove file">×</button>
            </div>

            {#if progress}
              <div class="import-progress">
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
              <div class="import-error">
                <span class="error-icon">⚠</span>
                {error}
              </div>
            {/if}
          </div>
        {/if}
      </div>

      <footer class="import-footer">
        <button class="cancel-btn" on:click={() => dispatch('close')}>Cancel</button>
        {#if files && isFileValid}
          <button 
            class="import-btn" 
            on:click={importMission}
            disabled={progress?.status === 'preparing' || progress?.status === 'exporting'}
          >
            {progress?.status === 'exporting' ? 'Importing...' : 'Import Mission'}
          </button>
        {/if}
      </footer>
    </div>
  </div>
{/if}

<style>
  .import-overlay {
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

  .mission-import {
    width: 500px;
    background: var(--color-bg-primary);
    border-radius: var(--radius-xl);
    box-shadow: var(--shadow-xl);
    border: 1px solid var(--color-border);
    max-height: 80vh;
    overflow: hidden;
  }

  .import-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-4);
    border-bottom: 1px solid var(--color-border);
  }

  .import-header h3 {
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

  .import-content {
    padding: var(--space-4);
  }

  .drop-zone {
    border: 2px dashed var(--color-border);
    border-radius: var(--radius-lg);
    padding: var(--space-8);
    text-align: center;
    cursor: pointer;
    transition: all var(--duration-200);
    background: var(--color-bg-surface);
    position: relative;
  }

  .drop-zone:hover, .drop-zone.drag-active {
    border-color: var(--color-primary);
    background: var(--color-bg-active);
  }

  .drop-zone-content h4 {
    margin: var(--space-3) 0 var(--space-2) 0;
    font-size: var(--text-base);
    font-weight: var(--font-medium);
    color: var(--color-text-primary);
  }

  .drop-zone-content p {
    margin: 0 0 var(--space-4) 0;
    font-size: var(--text-sm);
    color: var(--color-text-muted);
  }

  .upload-icon {
    font-size: var(--text-4xl);
    margin-bottom: var(--space-2);
  }

  .file-input {
    display: none;
  }

  .browse-btn {
    display: inline-block;
    padding: var(--space-2) var(--space-4);
    background: var(--color-primary);
    color: white;
    border-radius: var(--radius-md);
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    cursor: pointer;
    transition: background-color var(--duration-200);
  }

  .browse-btn:hover {
    background: var(--color-primary-hover);
  }

  .file-preview {
    background: var(--color-bg-surface);
    border-radius: var(--radius-lg);
    padding: var(--space-4);
  }

  .file-info {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    margin-bottom: var(--space-4);
  }

  .file-icon {
    font-size: var(--text-2xl);
    width: 40px;
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--color-bg-hover);
    border-radius: var(--radius-md);
  }

  .file-icon.invalid {
    background: rgba(244, 67, 54, 0.1);
  }

  .file-details {
    flex: 1;
  }

  .file-name {
    margin: 0 0 var(--space-1) 0;
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    color: var(--color-text-primary);
    font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
  }

  .file-size {
    margin: 0;
    font-size: var(--text-xs);
    color: var(--color-text-muted);
  }

  .file-error {
    margin: var(--space-1) 0 0 0;
    font-size: var(--text-xs);
    color: var(--color-error);
  }

  .clear-btn {
    border: none;
    background: var(--color-bg-hover);
    color: var(--color-text-muted);
    width: 24px;
    height: 24px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: var(--text-lg);
    transition: background-color var(--duration-200);
  }

  .clear-btn:hover {
    background: var(--color-error);
    color: white;
  }

  .import-progress {
    margin-top: var(--space-4);
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

  .import-error {
    margin-top: var(--space-4);
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

  .import-footer {
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

  .import-btn {
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

  .import-btn:hover:not(:disabled) {
    background: var(--color-primary-hover);
  }

  .import-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>