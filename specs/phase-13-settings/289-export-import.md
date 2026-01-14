# Spec 289: Export/Import

## Header
- **Spec ID**: 289
- **Phase**: 13 - Settings UI
- **Component**: Export/Import
- **Dependencies**: Spec 288 (Data Cache)
- **Status**: Draft

## Objective
Create an export and import interface that allows users to backup, restore, and transfer their settings, sessions, templates, and other application data between instances or for archival purposes.

## Acceptance Criteria
- [x] Export all settings to a file
- [x] Export selected data categories
- [x] Import settings with validation
- [x] Preview import changes before applying
- [x] Handle version compatibility
- [x] Support multiple export formats (JSON, YAML)
- [x] Create automatic backups
- [x] Restore from backup

## Implementation

### ExportImport.svelte
```svelte
<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { writable, derived } from 'svelte/store';
  import { slide, fade } from 'svelte/transition';
  import ImportPreview from './ImportPreview.svelte';
  import BackupList from './BackupList.svelte';
  import ExportProgress from './ExportProgress.svelte';
  import { exportImportStore } from '$lib/stores/exportImport';
  import type {
    ExportConfig,
    ImportResult,
    BackupEntry,
    ExportCategory
  } from '$lib/types/settings';

  const dispatch = createEventDispatcher<{
    export: ExportConfig;
    import: ImportResult;
    backup: BackupEntry;
    restore: BackupEntry;
  }>();

  const exportCategories: ExportCategory[] = [
    {
      id: 'settings',
      name: 'Application Settings',
      description: 'All application preferences and configurations',
      size: 0,
      selected: true
    },
    {
      id: 'sessions',
      name: 'Session Data',
      description: 'Forge session history and results',
      size: 0,
      selected: true
    },
    {
      id: 'templates',
      name: 'Templates',
      description: 'Custom session templates and presets',
      size: 0,
      selected: true
    },
    {
      id: 'participants',
      name: 'Participant Configs',
      description: 'Custom participant configurations',
      size: 0,
      selected: true
    },
    {
      id: 'oracles',
      name: 'Oracle Configs',
      description: 'Custom oracle configurations',
      size: 0,
      selected: true
    },
    {
      id: 'policies',
      name: 'Policies',
      description: 'Content and behavior policies',
      size: 0,
      selected: true
    },
    {
      id: 'shortcuts',
      name: 'Keyboard Shortcuts',
      description: 'Custom keyboard shortcut bindings',
      size: 0,
      selected: false
    },
    {
      id: 'themes',
      name: 'Custom Themes',
      description: 'Custom color themes',
      size: 0,
      selected: false
    }
  ];

  let categories = writable<ExportCategory[]>(exportCategories);
  let exportFormat = writable<'json' | 'yaml'>('json');
  let includeMetadata = writable<boolean>(true);
  let compressExport = writable<boolean>(false);

  let showImportPreview = writable<boolean>(false);
  let importData = writable<ImportResult | null>(null);
  let importFile = writable<File | null>(null);
  let isExporting = writable<boolean>(false);
  let isImporting = writable<boolean>(false);
  let exportProgress = writable<number>(0);

  const backups = derived(exportImportStore, ($store) => $store.backups);
  const autoBackupEnabled = derived(exportImportStore, ($store) => $store.autoBackupEnabled);

  const selectedCategories = derived(categories, ($cats) =>
    $cats.filter(c => c.selected)
  );

  const estimatedSize = derived(selectedCategories, ($selected) =>
    $selected.reduce((sum, cat) => sum + cat.size, 0)
  );

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }

  function formatDate(date: Date): string {
    return new Date(date).toLocaleDateString(undefined, {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  }

  function toggleCategory(categoryId: string) {
    categories.update(cats =>
      cats.map(c =>
        c.id === categoryId ? { ...c, selected: !c.selected } : c
      )
    );
  }

  function selectAll() {
    categories.update(cats => cats.map(c => ({ ...c, selected: true })));
  }

  function selectNone() {
    categories.update(cats => cats.map(c => ({ ...c, selected: false })));
  }

  async function startExport() {
    isExporting.set(true);
    exportProgress.set(0);

    const config: ExportConfig = {
      categories: $selectedCategories.map(c => c.id),
      format: $exportFormat,
      includeMetadata: $includeMetadata,
      compress: $compressExport
    };

    try {
      const result = await exportImportStore.export(config, (progress) => {
        exportProgress.set(progress);
      });

      const filename = `tachikoma-export-${Date.now()}.${$exportFormat}${$compressExport ? '.gz' : ''}`;
      downloadFile(result, filename);

      dispatch('export', config);
    } catch (err) {
      alert('Export failed: ' + (err as Error).message);
    } finally {
      isExporting.set(false);
    }
  }

  function downloadFile(data: Blob, filename: string) {
    const url = URL.createObjectURL(data);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    a.click();
    URL.revokeObjectURL(url);
  }

  function handleFileSelect(event: Event) {
    const input = event.target as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;

    importFile.set(file);
    parseImportFile(file);
  }

  async function parseImportFile(file: File) {
    try {
      const text = await file.text();
      const data = file.name.endsWith('.yaml') || file.name.endsWith('.yml')
        ? await parseYaml(text)
        : JSON.parse(text);

      const result = await exportImportStore.validateImport(data);
      importData.set(result);
      showImportPreview.set(true);
    } catch (err) {
      alert('Failed to parse import file: ' + (err as Error).message);
    }
  }

  async function parseYaml(text: string): Promise<unknown> {
    // Dynamically import yaml parser
    const yaml = await import('yaml');
    return yaml.parse(text);
  }

  async function confirmImport(options: { merge: boolean; categories: string[] }) {
    if (!$importData) return;

    isImporting.set(true);

    try {
      await exportImportStore.import($importData.data, options);
      dispatch('import', $importData);
      showImportPreview.set(false);
      importData.set(null);
      importFile.set(null);
      alert('Import completed successfully!');
    } catch (err) {
      alert('Import failed: ' + (err as Error).message);
    } finally {
      isImporting.set(false);
    }
  }

  async function createBackup() {
    try {
      const backup = await exportImportStore.createBackup();
      dispatch('backup', backup);
    } catch (err) {
      alert('Backup failed: ' + (err as Error).message);
    }
  }

  async function restoreBackup(backup: BackupEntry) {
    if (!confirm(`Restore from backup "${backup.name}"? Current settings will be overwritten.`)) {
      return;
    }

    try {
      await exportImportStore.restoreBackup(backup.id);
      dispatch('restore', backup);
      alert('Restore completed successfully!');
    } catch (err) {
      alert('Restore failed: ' + (err as Error).message);
    }
  }

  async function deleteBackup(backupId: string) {
    if (confirm('Delete this backup? This cannot be undone.')) {
      await exportImportStore.deleteBackup(backupId);
    }
  }

  function toggleAutoBackup() {
    exportImportStore.setAutoBackup(!$autoBackupEnabled);
  }

  onMount(() => {
    exportImportStore.load();
    exportImportStore.calculateSizes().then((sizes) => {
      categories.update(cats =>
        cats.map(c => ({ ...c, size: sizes[c.id] || 0 }))
      );
    });
  });
</script>

<div class="export-import" data-testid="export-import">
  <header class="config-header">
    <div class="header-title">
      <h2>Export & Import</h2>
      <p class="description">Backup, restore, and transfer your data</p>
    </div>
  </header>

  <div class="export-import-grid">
    <section class="export-section">
      <h3>Export Data</h3>

      <div class="category-select">
        <div class="select-actions">
          <button class="link-btn" on:click={selectAll}>Select All</button>
          <button class="link-btn" on:click={selectNone}>Select None</button>
        </div>

        <div class="categories-list">
          {#each $categories as category (category.id)}
            <label class="category-option" class:selected={category.selected}>
              <input
                type="checkbox"
                checked={category.selected}
                on:change={() => toggleCategory(category.id)}
              />
              <div class="category-info">
                <span class="category-name">{category.name}</span>
                <span class="category-desc">{category.description}</span>
              </div>
              <span class="category-size">{formatBytes(category.size)}</span>
            </label>
          {/each}
        </div>

        <div class="export-summary">
          <span>Selected: {$selectedCategories.length} categories</span>
          <span>Estimated size: {formatBytes($estimatedSize)}</span>
        </div>
      </div>

      <div class="export-options">
        <div class="option-group">
          <label>Format</label>
          <div class="format-options">
            <label class:active={$exportFormat === 'json'}>
              <input
                type="radio"
                name="format"
                value="json"
                bind:group={$exportFormat}
              />
              JSON
            </label>
            <label class:active={$exportFormat === 'yaml'}>
              <input
                type="radio"
                name="format"
                value="yaml"
                bind:group={$exportFormat}
              />
              YAML
            </label>
          </div>
        </div>

        <label class="checkbox-option">
          <input type="checkbox" bind:checked={$includeMetadata} />
          Include metadata (version, timestamp)
        </label>

        <label class="checkbox-option">
          <input type="checkbox" bind:checked={$compressExport} />
          Compress export file
        </label>
      </div>

      <button
        class="btn primary full-width"
        on:click={startExport}
        disabled={$isExporting || $selectedCategories.length === 0}
      >
        {$isExporting ? 'Exporting...' : 'Export Selected Data'}
      </button>

      {#if $isExporting}
        <ExportProgress progress={$exportProgress} />
      {/if}
    </section>

    <section class="import-section">
      <h3>Import Data</h3>

      <div class="import-drop-zone" class:has-file={$importFile}>
        {#if $importFile}
          <div class="file-info">
            <span class="file-name">{$importFile.name}</span>
            <span class="file-size">{formatBytes($importFile.size)}</span>
            <button class="clear-btn" on:click={() => importFile.set(null)}>
              Clear
            </button>
          </div>
        {:else}
          <div class="drop-content">
            <span class="drop-icon">📁</span>
            <p>Drag and drop a file here or</p>
            <label class="btn secondary">
              Browse Files
              <input
                type="file"
                accept=".json,.yaml,.yml,.gz"
                on:change={handleFileSelect}
                hidden
              />
            </label>
          </div>
        {/if}
      </div>

      <div class="import-notes">
        <p>Supported formats: JSON, YAML</p>
        <p>Files exported from this application will be automatically validated</p>
      </div>
    </section>
  </div>

  <section class="backup-section">
    <div class="section-header">
      <h3>Automatic Backups</h3>
      <label class="auto-backup-toggle">
        <input
          type="checkbox"
          checked={$autoBackupEnabled}
          on:change={toggleAutoBackup}
        />
        Enable automatic backups
      </label>
    </div>

    <div class="backup-actions">
      <button class="btn secondary" on:click={createBackup}>
        Create Backup Now
      </button>
    </div>

    {#if $backups.length > 0}
      <div class="backups-list">
        {#each $backups as backup (backup.id)}
          <div class="backup-item">
            <div class="backup-info">
              <span class="backup-name">{backup.name}</span>
              <span class="backup-meta">
                {formatDate(backup.createdAt)} - {formatBytes(backup.size)}
              </span>
            </div>
            <div class="backup-actions">
              <button
                class="action-btn"
                on:click={() => restoreBackup(backup)}
              >
                Restore
              </button>
              <button
                class="action-btn"
                on:click={() => exportImportStore.downloadBackup(backup.id)}
              >
                Download
              </button>
              <button
                class="action-btn danger"
                on:click={() => deleteBackup(backup.id)}
              >
                Delete
              </button>
            </div>
          </div>
        {/each}
      </div>
    {:else}
      <div class="no-backups">
        <p>No backups yet</p>
        <p class="hint">Create a backup to protect your data</p>
      </div>
    {/if}
  </section>

  {#if $showImportPreview && $importData}
    <div class="modal-overlay" transition:fade on:click={() => showImportPreview.set(false)}>
      <div class="modal-content large" on:click|stopPropagation>
        <ImportPreview
          data={$importData}
          on:confirm={(e) => confirmImport(e.detail)}
          on:cancel={() => showImportPreview.set(false)}
          isImporting={$isImporting}
        />
      </div>
    </div>
  {/if}
</div>

<style>
  .export-import {
    max-width: 1000px;
  }

  .config-header {
    margin-bottom: 1.5rem;
  }

  .header-title h2 {
    font-size: 1.5rem;
    font-weight: 600;
    margin-bottom: 0.5rem;
  }

  .description {
    color: var(--text-secondary);
    font-size: 0.875rem;
  }

  .export-import-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1.5rem;
    margin-bottom: 1.5rem;
  }

  section {
    background: var(--card-bg);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    padding: 1.5rem;
  }

  section h3 {
    font-size: 1rem;
    font-weight: 600;
    margin-bottom: 1.25rem;
  }

  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }

  .section-header h3 {
    margin-bottom: 0;
  }

  .select-actions {
    display: flex;
    gap: 1rem;
    margin-bottom: 0.75rem;
  }

  .link-btn {
    background: none;
    border: none;
    color: var(--primary-color);
    font-size: 0.8125rem;
    cursor: pointer;
    padding: 0;
  }

  .categories-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    margin-bottom: 1rem;
  }

  .category-option {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem;
    background: var(--secondary-bg);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .category-option:hover {
    border-color: var(--primary-color);
  }

  .category-option.selected {
    border-color: var(--primary-color);
    background: var(--primary-alpha);
  }

  .category-option input {
    flex-shrink: 0;
  }

  .category-info {
    flex: 1;
    display: flex;
    flex-direction: column;
  }

  .category-name {
    font-weight: 500;
    font-size: 0.875rem;
  }

  .category-desc {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .category-size {
    font-size: 0.8125rem;
    color: var(--text-secondary);
  }

  .export-summary {
    display: flex;
    justify-content: space-between;
    padding: 0.75rem;
    background: var(--secondary-bg);
    border-radius: 6px;
    font-size: 0.8125rem;
    color: var(--text-secondary);
    margin-bottom: 1rem;
  }

  .export-options {
    margin-bottom: 1rem;
  }

  .option-group {
    margin-bottom: 0.75rem;
  }

  .option-group label {
    display: block;
    font-size: 0.8125rem;
    color: var(--text-secondary);
    margin-bottom: 0.375rem;
  }

  .format-options {
    display: flex;
    gap: 0.5rem;
  }

  .format-options label {
    padding: 0.5rem 1rem;
    background: var(--secondary-bg);
    border: 1px solid var(--border-color);
    border-radius: 4px;
    font-size: 0.875rem;
    cursor: pointer;
  }

  .format-options label.active {
    border-color: var(--primary-color);
    background: var(--primary-alpha);
  }

  .format-options input {
    display: none;
  }

  .checkbox-option {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.875rem;
    cursor: pointer;
    margin-bottom: 0.5rem;
  }

  .import-drop-zone {
    border: 2px dashed var(--border-color);
    border-radius: 8px;
    padding: 2rem;
    text-align: center;
    transition: all 0.15s ease;
    margin-bottom: 1rem;
  }

  .import-drop-zone:hover {
    border-color: var(--primary-color);
  }

  .import-drop-zone.has-file {
    border-style: solid;
    background: var(--secondary-bg);
  }

  .drop-content {
    color: var(--text-secondary);
  }

  .drop-icon {
    font-size: 2rem;
    display: block;
    margin-bottom: 0.75rem;
  }

  .file-info {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 1rem;
  }

  .file-name {
    font-weight: 500;
  }

  .file-size {
    color: var(--text-muted);
    font-size: 0.875rem;
  }

  .clear-btn {
    background: none;
    border: none;
    color: var(--error-color);
    cursor: pointer;
    font-size: 0.875rem;
  }

  .import-notes {
    font-size: 0.8125rem;
    color: var(--text-muted);
  }

  .import-notes p {
    margin-bottom: 0.25rem;
  }

  .auto-backup-toggle {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.875rem;
    cursor: pointer;
  }

  .backup-actions {
    margin-bottom: 1rem;
  }

  .backups-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .backup-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 1rem;
    background: var(--secondary-bg);
    border-radius: 6px;
  }

  .backup-info {
    display: flex;
    flex-direction: column;
  }

  .backup-name {
    font-weight: 500;
    font-size: 0.875rem;
  }

  .backup-meta {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .backup-actions,
  .backup-item .backup-actions {
    display: flex;
    gap: 0.5rem;
  }

  .action-btn {
    padding: 0.375rem 0.625rem;
    background: transparent;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    font-size: 0.75rem;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .action-btn:hover {
    border-color: var(--primary-color);
  }

  .action-btn.danger:hover {
    border-color: var(--error-color);
    color: var(--error-color);
  }

  .no-backups {
    text-align: center;
    padding: 2rem;
    color: var(--text-muted);
  }

  .hint {
    font-size: 0.875rem;
    margin-top: 0.5rem;
  }

  .btn {
    padding: 0.625rem 1.25rem;
    border: none;
    border-radius: 6px;
    font-weight: 500;
    cursor: pointer;
  }

  .btn.full-width {
    width: 100%;
  }

  .btn.primary {
    background: var(--primary-color);
    color: white;
  }

  .btn.secondary {
    background: var(--secondary-bg);
    border: 1px solid var(--border-color);
    color: var(--text-primary);
  }

  .modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal-content {
    background: var(--card-bg);
    border-radius: 8px;
    max-width: 600px;
    width: 90%;
    max-height: 80vh;
    overflow-y: auto;
  }

  .modal-content.large {
    max-width: 800px;
  }

  @media (max-width: 768px) {
    .export-import-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
```

## Testing Requirements
1. **Unit Tests**: Test export/import serialization
2. **Validation Tests**: Test import validation
3. **Format Tests**: Test JSON and YAML formats
4. **Backup Tests**: Test backup creation/restore
5. **Compatibility Tests**: Test version compatibility

## Related Specs
- Spec 288: Data Cache
- Spec 290: Profile Management
- Spec 295: Settings Tests
