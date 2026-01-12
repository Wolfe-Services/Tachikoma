# 282 - Import Settings

**Phase:** 13 - Settings UI
**Spec ID:** 282
**Status:** Planned
**Dependencies:** 271-settings-layout, 272-settings-store, 284-settings-validation
**Estimated Context:** ~8% of model context window

---

## Objective

Create the Settings Import functionality that allows users to import settings from a file with validation, preview, selective import, and merge options.

---

## Acceptance Criteria

- [ ] `ImportSettings.svelte` component with import options
- [ ] File upload via drag-and-drop or file picker
- [ ] Support for JSON and YAML formats
- [ ] Validation of imported settings
- [ ] Preview of changes before import
- [ ] Selective import by category
- [ ] Merge or replace options
- [ ] Error handling with detailed messages

---

## Implementation Details

### 1. Import Settings Component (src/lib/components/settings/ImportSettings.svelte)

```svelte
<script lang="ts">
  import { settingsStore } from '$lib/stores/settings-store';
  import { validateSettings } from '$lib/utils/settings-validation';
  import type { AllSettings, SettingsValidationError } from '$lib/types/settings';
  import { DEFAULT_SETTINGS } from '$lib/stores/settings-defaults';
  import SettingsSection from './SettingsSection.svelte';
  import SettingsRow from './SettingsRow.svelte';
  import Toggle from '$lib/components/ui/Toggle.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Icon from '$lib/components/ui/Icon.svelte';
  import Modal from '$lib/components/ui/Modal.svelte';

  interface ImportSection {
    id: keyof AllSettings;
    label: string;
    hasChanges: boolean;
    changesCount: number;
    selected: boolean;
    errors: SettingsValidationError[];
  }

  type ImportMode = 'merge' | 'replace';

  let fileInput: HTMLInputElement;
  let isDragOver = false;
  let importedData: Partial<AllSettings> | null = null;
  let importSections: ImportSection[] = [];
  let parseError: string | null = null;
  let importMode: ImportMode = 'merge';
  let showConfirmModal = false;
  let isImporting = false;

  function handleDragOver(event: DragEvent) {
    event.preventDefault();
    isDragOver = true;
  }

  function handleDragLeave() {
    isDragOver = false;
  }

  function handleDrop(event: DragEvent) {
    event.preventDefault();
    isDragOver = false;

    const files = event.dataTransfer?.files;
    if (files && files.length > 0) {
      processFile(files[0]);
    }
  }

  function handleFileSelect(event: Event) {
    const input = event.target as HTMLInputElement;
    const files = input.files;
    if (files && files.length > 0) {
      processFile(files[0]);
    }
  }

  async function processFile(file: File) {
    parseError = null;
    importedData = null;
    importSections = [];

    // Check file type
    const isJson = file.name.endsWith('.json');
    const isYaml = file.name.endsWith('.yaml') || file.name.endsWith('.yml');

    if (!isJson && !isYaml) {
      parseError = 'Unsupported file type. Please use JSON or YAML files.';
      return;
    }

    try {
      const content = await file.text();
      let parsed: any;

      if (isJson) {
        parsed = JSON.parse(content);
      } else {
        // Simple YAML parsing
        parsed = parseYaml(content);
      }

      // Validate structure
      if (!parsed.settings && !parsed.general) {
        // Check if it's a raw settings object
        if (parsed.general || parsed.appearance || parsed.editor) {
          parsed = { settings: parsed };
        } else {
          throw new Error('Invalid settings file structure');
        }
      }

      const settings = parsed.settings || parsed;
      importedData = settings;

      // Analyze changes
      analyzeChanges(settings);
    } catch (error) {
      parseError = `Failed to parse file: ${(error as Error).message}`;
    }
  }

  function parseYaml(content: string): any {
    // Basic YAML to JSON conversion
    const lines = content.split('\n');
    const result: any = {};
    const stack: { indent: number; obj: any; key: string }[] = [{ indent: -1, obj: result, key: '' }];

    for (const line of lines) {
      if (line.trim() === '' || line.trim().startsWith('#')) continue;

      const indent = line.search(/\S/);
      const trimmed = line.trim();

      // Handle key-value pairs
      const colonIndex = trimmed.indexOf(':');
      if (colonIndex > 0) {
        const key = trimmed.slice(0, colonIndex).trim();
        let value = trimmed.slice(colonIndex + 1).trim();

        // Pop stack to find parent
        while (stack.length > 1 && stack[stack.length - 1].indent >= indent) {
          stack.pop();
        }

        const parent = stack[stack.length - 1].obj;

        if (value === '' || value === '|' || value === '>') {
          // Nested object or multiline string
          parent[key] = {};
          stack.push({ indent, obj: parent[key], key });
        } else {
          // Parse value
          if (value === 'true') value = true as any;
          else if (value === 'false') value = false as any;
          else if (value === 'null') value = null as any;
          else if (/^-?\d+$/.test(value)) value = parseInt(value) as any;
          else if (/^-?\d+\.\d+$/.test(value)) value = parseFloat(value) as any;
          else if (value.startsWith('"') && value.endsWith('"')) value = value.slice(1, -1);

          parent[key] = value;
        }
      }
    }

    return result;
  }

  function analyzeChanges(settings: Partial<AllSettings>) {
    const currentState = get(settingsStore);
    const sections: ImportSection[] = [];

    const categoryLabels: Record<keyof AllSettings, string> = {
      general: 'General Settings',
      appearance: 'Appearance',
      editor: 'Editor Preferences',
      keybindings: 'Keyboard Shortcuts',
      backends: 'LLM Backends',
      git: 'Git Settings',
      sync: 'Sync Settings',
    };

    for (const key of Object.keys(categoryLabels) as (keyof AllSettings)[]) {
      if (settings[key]) {
        const currentSettings = currentState.settings[key];
        const newSettings = settings[key];
        const changes = countChanges(currentSettings, newSettings);
        const errors = validateCategorySettings(key, newSettings);

        sections.push({
          id: key,
          label: categoryLabels[key],
          hasChanges: changes > 0,
          changesCount: changes,
          selected: changes > 0 && errors.length === 0,
          errors,
        });
      }
    }

    importSections = sections;
  }

  function countChanges(current: any, imported: any): number {
    let changes = 0;

    for (const key of Object.keys(imported)) {
      if (JSON.stringify(current[key]) !== JSON.stringify(imported[key])) {
        changes++;
      }
    }

    return changes;
  }

  function validateCategorySettings(category: keyof AllSettings, settings: any): SettingsValidationError[] {
    const fullSettings = {
      ...DEFAULT_SETTINGS,
      [category]: settings,
    };
    const errors = validateSettings(fullSettings);
    return errors.filter(e => e.path.startsWith(category));
  }

  function toggleSection(sectionId: keyof AllSettings) {
    importSections = importSections.map(s =>
      s.id === sectionId ? { ...s, selected: !s.selected } : s
    );
  }

  function startImport() {
    showConfirmModal = true;
  }

  async function executeImport() {
    if (!importedData) return;

    isImporting = true;

    try {
      const currentState = get(settingsStore);
      let newSettings: AllSettings;

      if (importMode === 'replace') {
        newSettings = { ...DEFAULT_SETTINGS };
      } else {
        newSettings = structuredClone(currentState.settings);
      }

      // Apply selected sections
      for (const section of importSections) {
        if (section.selected && importedData[section.id]) {
          if (importMode === 'merge') {
            newSettings[section.id] = {
              ...newSettings[section.id],
              ...importedData[section.id],
            };
          } else {
            newSettings[section.id] = importedData[section.id] as any;
          }
        }
      }

      settingsStore.setSettings(newSettings);
      await settingsStore.save();

      // Reset state
      importedData = null;
      importSections = [];
      showConfirmModal = false;
    } catch (error) {
      console.error('Import failed:', error);
    }

    isImporting = false;
  }

  function clearImport() {
    importedData = null;
    importSections = [];
    parseError = null;
    if (fileInput) {
      fileInput.value = '';
    }
  }

  function get<T>(store: { subscribe: (fn: (value: T) => void) => void }): T {
    let value: T;
    store.subscribe(v => value = v)();
    return value!;
  }

  $: selectedCount = importSections.filter(s => s.selected).length;
  $: totalChanges = importSections
    .filter(s => s.selected)
    .reduce((sum, s) => sum + s.changesCount, 0);
  $: hasErrors = importSections.some(s => s.selected && s.errors.length > 0);
</script>

<div class="import-settings">
  <h2 class="settings-title">Import Settings</h2>
  <p class="settings-description">
    Import settings from a previously exported file.
  </p>

  <!-- File Upload -->
  <SettingsSection title="Select File">
    <div
      class="drop-zone"
      class:drop-zone--active={isDragOver}
      class:drop-zone--has-file={importedData !== null}
      on:dragover={handleDragOver}
      on:dragleave={handleDragLeave}
      on:drop={handleDrop}
      role="button"
      tabindex="0"
      on:click={() => fileInput.click()}
      on:keydown={(e) => e.key === 'Enter' && fileInput.click()}
    >
      <input
        bind:this={fileInput}
        type="file"
        accept=".json,.yaml,.yml"
        on:change={handleFileSelect}
        hidden
      />

      {#if importedData}
        <div class="drop-zone__loaded">
          <Icon name="file-check" size={48} />
          <span class="drop-zone__filename">File loaded successfully</span>
          <span class="drop-zone__info">
            {importSections.length} sections found, {importSections.filter(s => s.hasChanges).length} with changes
          </span>
        </div>
      {:else}
        <Icon name="upload" size={48} />
        <span class="drop-zone__label">
          Drag and drop a settings file here
        </span>
        <span class="drop-zone__hint">
          or click to browse (JSON or YAML)
        </span>
      {/if}
    </div>

    {#if parseError}
      <div class="parse-error">
        <Icon name="alert-circle" size={16} />
        <span>{parseError}</span>
      </div>
    {/if}

    {#if importedData}
      <Button variant="ghost" size="small" on:click={clearImport}>
        <Icon name="x" size={14} />
        Clear
      </Button>
    {/if}
  </SettingsSection>

  {#if importedData}
    <!-- Import Mode -->
    <SettingsSection title="Import Mode">
      <div class="import-modes">
        <button
          class="import-mode"
          class:import-mode--selected={importMode === 'merge'}
          on:click={() => importMode = 'merge'}
        >
          <Icon name="git-merge" size={24} />
          <span class="import-mode__label">Merge</span>
          <span class="import-mode__desc">Combine with existing settings</span>
        </button>
        <button
          class="import-mode"
          class:import-mode--selected={importMode === 'replace'}
          on:click={() => importMode = 'replace'}
        >
          <Icon name="refresh-cw" size={24} />
          <span class="import-mode__label">Replace</span>
          <span class="import-mode__desc">Overwrite all selected settings</span>
        </button>
      </div>
    </SettingsSection>

    <!-- Select Sections -->
    <SettingsSection title="Settings to Import">
      <div class="import-sections">
        {#each importSections as section}
          <div
            class="import-section"
            class:import-section--selected={section.selected}
            class:import-section--error={section.errors.length > 0}
            class:import-section--no-changes={!section.hasChanges}
          >
            <div
              class="import-section__checkbox"
              on:click={() => toggleSection(section.id)}
              on:keydown={(e) => e.key === 'Enter' && toggleSection(section.id)}
              role="checkbox"
              aria-checked={section.selected}
              tabindex="0"
            >
              {#if section.selected}
                <Icon name="check-square" size={20} />
              {:else}
                <Icon name="square" size={20} />
              {/if}
            </div>

            <div class="import-section__info">
              <span class="import-section__label">{section.label}</span>
              {#if section.hasChanges}
                <span class="import-section__changes">
                  {section.changesCount} change{section.changesCount !== 1 ? 's' : ''}
                </span>
              {:else}
                <span class="import-section__no-changes">No changes</span>
              {/if}
            </div>

            {#if section.errors.length > 0}
              <div class="import-section__errors">
                <Icon name="alert-triangle" size={14} />
                <span>{section.errors.length} error{section.errors.length !== 1 ? 's' : ''}</span>
              </div>
            {/if}
          </div>
        {/each}
      </div>

      {#if hasErrors}
        <div class="validation-errors">
          <h4>Validation Errors</h4>
          <ul>
            {#each importSections.flatMap(s => s.errors) as error}
              <li>
                <strong>{error.path}:</strong> {error.message}
              </li>
            {/each}
          </ul>
        </div>
      {/if}
    </SettingsSection>

    <!-- Import Summary -->
    <div class="import-summary">
      <div class="import-summary__info">
        <span class="import-summary__count">
          {selectedCount} section{selectedCount !== 1 ? 's' : ''} selected
        </span>
        <span class="import-summary__changes">
          {totalChanges} total change{totalChanges !== 1 ? 's' : ''}
        </span>
      </div>

      <Button
        variant="primary"
        disabled={selectedCount === 0 || hasErrors}
        on:click={startImport}
      >
        <Icon name="download" size={16} />
        Import Selected
      </Button>
    </div>
  {/if}
</div>

<!-- Confirm Modal -->
{#if showConfirmModal}
  <Modal title="Confirm Import" on:close={() => showConfirmModal = false}>
    <div class="confirm-modal">
      <p>
        You are about to {importMode === 'merge' ? 'merge' : 'replace'}
        {selectedCount} setting section{selectedCount !== 1 ? 's' : ''}
        with {totalChanges} change{totalChanges !== 1 ? 's' : ''}.
      </p>

      {#if importMode === 'replace'}
        <div class="confirm-modal__warning">
          <Icon name="alert-triangle" size={16} />
          <span>Replace mode will overwrite all settings in the selected sections.</span>
        </div>
      {/if}

      <div class="confirm-modal__actions">
        <Button variant="secondary" on:click={() => showConfirmModal = false} disabled={isImporting}>
          Cancel
        </Button>
        <Button variant="primary" on:click={executeImport} disabled={isImporting}>
          {#if isImporting}
            <Icon name="loader" size={16} class="spinning" />
          {/if}
          Confirm Import
        </Button>
      </div>
    </div>
  </Modal>
{/if}

<style>
  .import-settings {
    max-width: 720px;
  }

  .settings-title {
    font-size: 24px;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0 0 8px 0;
  }

  .settings-description {
    color: var(--color-text-secondary);
    font-size: 14px;
    margin: 0 0 24px 0;
  }

  /* Drop Zone */
  .drop-zone {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 48px 24px;
    border: 2px dashed var(--color-border);
    border-radius: 12px;
    background: var(--color-bg-secondary);
    cursor: pointer;
    transition: all 0.2s ease;
    text-align: center;
  }

  .drop-zone:hover {
    border-color: var(--color-primary);
    background: rgba(33, 150, 243, 0.05);
  }

  .drop-zone--active {
    border-color: var(--color-primary);
    background: rgba(33, 150, 243, 0.1);
  }

  .drop-zone--has-file {
    border-style: solid;
    border-color: var(--color-success);
  }

  .drop-zone__label {
    font-size: 16px;
    color: var(--color-text-primary);
    margin-top: 16px;
  }

  .drop-zone__hint {
    font-size: 13px;
    color: var(--color-text-muted);
    margin-top: 4px;
  }

  .drop-zone__loaded {
    display: flex;
    flex-direction: column;
    align-items: center;
    color: var(--color-success);
  }

  .drop-zone__filename {
    font-size: 16px;
    font-weight: 500;
    margin-top: 12px;
    color: var(--color-text-primary);
  }

  .drop-zone__info {
    font-size: 13px;
    color: var(--color-text-secondary);
    margin-top: 4px;
  }

  .parse-error {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px;
    background: rgba(244, 67, 54, 0.1);
    color: var(--color-error);
    border-radius: 6px;
    font-size: 13px;
    margin-top: 12px;
  }

  /* Import Modes */
  .import-modes {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }

  .import-mode {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 20px;
    border: 2px solid var(--color-border);
    border-radius: 8px;
    background: transparent;
    cursor: pointer;
    transition: all 0.15s ease;
    text-align: center;
  }

  .import-mode:hover {
    border-color: var(--color-text-muted);
  }

  .import-mode--selected {
    border-color: var(--color-primary);
    background: rgba(33, 150, 243, 0.05);
  }

  .import-mode__label {
    font-size: 16px;
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .import-mode__desc {
    font-size: 12px;
    color: var(--color-text-muted);
  }

  /* Import Sections */
  .import-sections {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .import-section {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    background: var(--color-bg-secondary);
    border: 2px solid transparent;
    border-radius: 8px;
    transition: all 0.15s ease;
  }

  .import-section--selected {
    border-color: var(--color-primary);
    background: rgba(33, 150, 243, 0.05);
  }

  .import-section--error {
    border-color: var(--color-error);
    background: rgba(244, 67, 54, 0.05);
  }

  .import-section--no-changes {
    opacity: 0.6;
  }

  .import-section__checkbox {
    color: var(--color-text-muted);
    cursor: pointer;
  }

  .import-section--selected .import-section__checkbox {
    color: var(--color-primary);
  }

  .import-section__info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .import-section__label {
    font-size: 14px;
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .import-section__changes {
    font-size: 12px;
    color: var(--color-success);
  }

  .import-section__no-changes {
    font-size: 12px;
    color: var(--color-text-muted);
  }

  .import-section__errors {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 12px;
    color: var(--color-error);
  }

  .validation-errors {
    margin-top: 16px;
    padding: 16px;
    background: rgba(244, 67, 54, 0.05);
    border: 1px solid var(--color-error);
    border-radius: 8px;
  }

  .validation-errors h4 {
    margin: 0 0 8px 0;
    color: var(--color-error);
    font-size: 14px;
  }

  .validation-errors ul {
    margin: 0;
    padding-left: 20px;
    font-size: 13px;
    color: var(--color-text-secondary);
  }

  /* Import Summary */
  .import-summary {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-top: 24px;
    padding-top: 24px;
    border-top: 1px solid var(--color-border);
  }

  .import-summary__info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .import-summary__count {
    font-size: 14px;
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .import-summary__changes {
    font-size: 12px;
    color: var(--color-text-secondary);
  }

  /* Confirm Modal */
  .confirm-modal {
    padding: 8px;
  }

  .confirm-modal p {
    font-size: 14px;
    color: var(--color-text-secondary);
    margin: 0 0 16px 0;
  }

  .confirm-modal__warning {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px;
    background: rgba(255, 152, 0, 0.1);
    color: var(--color-warning);
    border-radius: 6px;
    font-size: 13px;
    margin-bottom: 20px;
  }

  .confirm-modal__actions {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
  }

  :global(.spinning) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
</style>
```

---

## Testing Requirements

1. File drag-and-drop works
2. File picker works
3. JSON files parse correctly
4. YAML files parse correctly
5. Invalid files show error
6. Change detection works
7. Section selection toggles
8. Merge mode preserves existing
9. Replace mode overwrites
10. Validation errors display

### Test File (src/lib/components/settings/__tests__/ImportSettings.test.ts)

```typescript
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import ImportSettings from '../ImportSettings.svelte';
import { settingsStore } from '$lib/stores/settings-store';

describe('ImportSettings', () => {
  beforeEach(() => {
    settingsStore.resetAll();
  });

  it('renders import drop zone', () => {
    render(ImportSettings);

    expect(screen.getByText('Import Settings')).toBeInTheDocument();
    expect(screen.getByText(/Drag and drop/)).toBeInTheDocument();
  });

  it('shows file info after loading', async () => {
    render(ImportSettings);

    const file = new File(
      [JSON.stringify({ settings: { general: { language: 'fr' } } })],
      'settings.json',
      { type: 'application/json' }
    );

    const input = document.querySelector('input[type="file"]') as HTMLInputElement;
    Object.defineProperty(input, 'files', { value: [file] });
    await fireEvent.change(input);

    await waitFor(() => {
      expect(screen.getByText('File loaded successfully')).toBeInTheDocument();
    });
  });

  it('shows parse error for invalid file', async () => {
    render(ImportSettings);

    const file = new File(['not valid json'], 'settings.json', { type: 'application/json' });

    const input = document.querySelector('input[type="file"]') as HTMLInputElement;
    Object.defineProperty(input, 'files', { value: [file] });
    await fireEvent.change(input);

    await waitFor(() => {
      expect(screen.getByText(/Failed to parse/)).toBeInTheDocument();
    });
  });

  it('toggles import mode', async () => {
    render(ImportSettings);

    // Load a file first
    const file = new File(
      [JSON.stringify({ settings: { general: { language: 'fr' } } })],
      'settings.json',
      { type: 'application/json' }
    );

    const input = document.querySelector('input[type="file"]') as HTMLInputElement;
    Object.defineProperty(input, 'files', { value: [file] });
    await fireEvent.change(input);

    await waitFor(() => {
      expect(screen.getByText('Merge')).toBeInTheDocument();
    });

    const replaceButton = screen.getByText('Replace');
    await fireEvent.click(replaceButton);

    expect(replaceButton.closest('.import-mode')).toHaveClass('import-mode--selected');
  });

  it('shows confirmation modal before import', async () => {
    render(ImportSettings);

    const file = new File(
      [JSON.stringify({ settings: { general: { language: 'fr' } } })],
      'settings.json',
      { type: 'application/json' }
    );

    const input = document.querySelector('input[type="file"]') as HTMLInputElement;
    Object.defineProperty(input, 'files', { value: [file] });
    await fireEvent.change(input);

    await waitFor(() => {
      expect(screen.getByText('Import Selected')).toBeInTheDocument();
    });

    await fireEvent.click(screen.getByText('Import Selected'));

    expect(screen.getByText('Confirm Import')).toBeInTheDocument();
  });
});
```

---

## Related Specs

- Depends on: [271-settings-layout.md](271-settings-layout.md)
- Depends on: [272-settings-store.md](272-settings-store.md)
- Depends on: [284-settings-validation.md](284-settings-validation.md)
- Previous: [281-settings-export.md](281-settings-export.md)
- Next: [283-settings-profiles.md](283-settings-profiles.md)
