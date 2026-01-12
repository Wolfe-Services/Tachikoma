# Spec 248: Spec Import

## Phase
11 - Spec Browser UI

## Spec ID
248

## Status
Planned

## Dependencies
- Spec 239 (Spec Validation)
- Spec 247 (Spec Export)

## Estimated Context
~9%

---

## Objective

Implement import functionality for specs supporting multiple formats (Markdown, JSON), with validation, conflict resolution, preview before import, and batch import capabilities.

---

## Acceptance Criteria

- [ ] Import from Markdown files
- [ ] Import from JSON files
- [ ] Drag-and-drop file upload
- [ ] Parse and validate imported specs
- [ ] Preview specs before import
- [ ] Detect and resolve ID conflicts
- [ ] Map dependencies during import
- [ ] Support bulk import (ZIP files)
- [ ] Progress indicator for large imports

---

## Implementation Details

### ImportDialog.svelte

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { writable, derived } from 'svelte/store';
  import type { Spec, ImportResult, ConflictResolution } from '$lib/types/spec';
  import Modal from '$lib/components/Modal.svelte';
  import Button from '$lib/components/Button.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import FileDropzone from '$lib/components/FileDropzone.svelte';
  import ProgressBar from '$lib/components/ProgressBar.svelte';
  import { parseImportFile, validateImportedSpecs } from '$lib/utils/import';
  import { validateSpec } from '$lib/utils/validation';

  export let open = false;
  export let existingSpecs: Spec[] = [];

  const dispatch = createEventDispatcher<{
    close: void;
    import: Spec[];
  }>();

  type ImportStep = 'upload' | 'preview' | 'conflicts' | 'importing';

  let currentStep = writable<ImportStep>('upload');
  let files: File[] = [];
  let parsedSpecs = writable<Spec[]>([]);
  let validationErrors = writable<Map<string, string[]>>(new Map());
  let conflicts = writable<Map<string, ConflictResolution>>(new Map());
  let importing = false;
  let importProgress = 0;
  let error: string | null = null;

  $: existingIds = new Set(existingSpecs.map(s => s.id));
  $: conflictingSpecs = $parsedSpecs.filter(s => existingIds.has(s.id));
  $: hasConflicts = conflictingSpecs.length > 0;
  $: validSpecCount = $parsedSpecs.filter(s =>
    !$validationErrors.get(s.id)?.length
  ).length;

  async function handleFilesSelect(event: CustomEvent<File[]>) {
    files = event.detail;
    error = null;

    if (files.length === 0) return;

    try {
      const allSpecs: Spec[] = [];

      for (const file of files) {
        const specs = await parseImportFile(file);
        allSpecs.push(...specs);
      }

      parsedSpecs.set(allSpecs);

      // Validate each spec
      const errors = new Map<string, string[]>();
      allSpecs.forEach(spec => {
        const results = validateSpec(spec, [...existingSpecs, ...allSpecs]);
        const specErrors = results
          .filter(r => r.severity === 'error')
          .map(r => r.message);

        if (specErrors.length) {
          errors.set(spec.id, specErrors);
        }
      });

      validationErrors.set(errors);

      // Initialize conflict resolutions
      const resolutions = new Map<string, ConflictResolution>();
      conflictingSpecs.forEach(spec => {
        resolutions.set(spec.id, 'skip');
      });
      conflicts.set(resolutions);

      currentStep.set('preview');
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to parse file';
    }
  }

  function handleConflictResolution(specId: string, resolution: ConflictResolution) {
    conflicts.update(c => {
      c.set(specId, resolution);
      return c;
    });
  }

  function proceedToConflicts() {
    if (hasConflicts) {
      currentStep.set('conflicts');
    } else {
      handleImport();
    }
  }

  async function handleImport() {
    currentStep.set('importing');
    importing = true;
    importProgress = 0;

    try {
      const specsToImport: Spec[] = [];

      for (const spec of $parsedSpecs) {
        // Skip specs with validation errors
        if ($validationErrors.get(spec.id)?.length) {
          continue;
        }

        // Handle conflicts
        const conflict = $conflicts.get(spec.id);
        if (conflict === 'skip') {
          continue;
        } else if (conflict === 'rename') {
          spec.id = generateNewId(spec.id);
        }
        // 'replace' keeps the same ID

        specsToImport.push(spec);
        importProgress = (specsToImport.length / $parsedSpecs.length) * 100;
      }

      dispatch('import', specsToImport);
      handleClose();
    } catch (e) {
      error = e instanceof Error ? e.message : 'Import failed';
      currentStep.set('preview');
    } finally {
      importing = false;
    }
  }

  function generateNewId(existingId: string): string {
    const baseNum = parseInt(existingId.replace(/\D/g, ''), 10) || 0;
    let newNum = baseNum + 1;

    while (existingIds.has(String(newNum))) {
      newNum++;
    }

    return String(newNum);
  }

  function handleClose() {
    currentStep.set('upload');
    files = [];
    parsedSpecs.set([]);
    validationErrors.set(new Map());
    conflicts.set(new Map());
    error = null;
    dispatch('close');
  }

  function removeSpec(specId: string) {
    parsedSpecs.update(specs => specs.filter(s => s.id !== specId));
  }
</script>

<Modal {open} on:close={handleClose} size="lg" title="Import Specs">
  <div class="import-dialog">
    {#if $currentStep === 'upload'}
      <div class="import-dialog__upload">
        <FileDropzone
          accept=".md,.json,.zip"
          multiple
          on:select={handleFilesSelect}
        >
          <div class="import-dialog__dropzone-content">
            <Icon name="upload-cloud" size={48} />
            <h3>Drop files here to import</h3>
            <p>Supports Markdown (.md), JSON (.json), and ZIP archives</p>
            <Button variant="outline">
              <Icon name="folder" size={14} />
              Browse files
            </Button>
          </div>
        </FileDropzone>

        {#if error}
          <div class="import-dialog__error">
            <Icon name="alert-circle" size={16} />
            {error}
          </div>
        {/if}
      </div>
    {:else if $currentStep === 'preview'}
      <div class="import-dialog__preview">
        <div class="import-dialog__summary">
          <div class="import-dialog__stat">
            <span class="import-dialog__stat-value">{$parsedSpecs.length}</span>
            <span class="import-dialog__stat-label">Specs found</span>
          </div>
          <div class="import-dialog__stat">
            <span class="import-dialog__stat-value import-dialog__stat-value--success">
              {validSpecCount}
            </span>
            <span class="import-dialog__stat-label">Valid</span>
          </div>
          <div class="import-dialog__stat">
            <span class="import-dialog__stat-value import-dialog__stat-value--warning">
              {conflictingSpecs.length}
            </span>
            <span class="import-dialog__stat-label">Conflicts</span>
          </div>
          <div class="import-dialog__stat">
            <span class="import-dialog__stat-value import-dialog__stat-value--error">
              {$parsedSpecs.length - validSpecCount}
            </span>
            <span class="import-dialog__stat-label">Errors</span>
          </div>
        </div>

        <div class="import-dialog__spec-list">
          <h4>Specs to Import</h4>
          <table class="import-dialog__table">
            <thead>
              <tr>
                <th>ID</th>
                <th>Title</th>
                <th>Phase</th>
                <th>Status</th>
                <th></th>
              </tr>
            </thead>
            <tbody>
              {#each $parsedSpecs as spec}
                {@const errors = $validationErrors.get(spec.id) || []}
                {@const isConflict = existingIds.has(spec.id)}
                <tr
                  class:import-dialog__row--error={errors.length > 0}
                  class:import-dialog__row--warning={isConflict && errors.length === 0}
                >
                  <td>
                    <span class="import-dialog__spec-id">{spec.id}</span>
                    {#if isConflict}
                      <span class="import-dialog__conflict-badge">Conflict</span>
                    {/if}
                  </td>
                  <td>{spec.title}</td>
                  <td>{spec.phase}</td>
                  <td>{spec.status}</td>
                  <td>
                    {#if errors.length > 0}
                      <span class="import-dialog__error-icon" title={errors.join(', ')}>
                        <Icon name="alert-circle" size={16} />
                      </span>
                    {/if}
                    <button
                      class="import-dialog__remove-btn"
                      on:click={() => removeSpec(spec.id)}
                      aria-label="Remove from import"
                    >
                      <Icon name="x" size={14} />
                    </button>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>

        {#if $parsedSpecs.length - validSpecCount > 0}
          <div class="import-dialog__notice">
            <Icon name="info" size={16} />
            Specs with validation errors will be skipped during import.
          </div>
        {/if}
      </div>
    {:else if $currentStep === 'conflicts'}
      <div class="import-dialog__conflicts">
        <h4>Resolve Conflicts</h4>
        <p>The following specs have IDs that already exist. Choose how to handle each:</p>

        <div class="import-dialog__conflict-list">
          {#each conflictingSpecs as spec}
            {@const resolution = $conflicts.get(spec.id) || 'skip'}
            <div class="import-dialog__conflict-item">
              <div class="import-dialog__conflict-info">
                <span class="import-dialog__spec-id">{spec.id}</span>
                <span class="import-dialog__spec-title">{spec.title}</span>
              </div>

              <div class="import-dialog__conflict-options">
                <label class="import-dialog__radio">
                  <input
                    type="radio"
                    name="conflict-{spec.id}"
                    value="skip"
                    checked={resolution === 'skip'}
                    on:change={() => handleConflictResolution(spec.id, 'skip')}
                  />
                  <span>Skip</span>
                </label>
                <label class="import-dialog__radio">
                  <input
                    type="radio"
                    name="conflict-{spec.id}"
                    value="replace"
                    checked={resolution === 'replace'}
                    on:change={() => handleConflictResolution(spec.id, 'replace')}
                  />
                  <span>Replace existing</span>
                </label>
                <label class="import-dialog__radio">
                  <input
                    type="radio"
                    name="conflict-{spec.id}"
                    value="rename"
                    checked={resolution === 'rename'}
                    on:change={() => handleConflictResolution(spec.id, 'rename')}
                  />
                  <span>Import as new ({generateNewId(spec.id)})</span>
                </label>
              </div>
            </div>
          {/each}
        </div>
      </div>
    {:else if $currentStep === 'importing'}
      <div class="import-dialog__importing">
        <Icon name="loader" size={32} class="spinning" />
        <h3>Importing specs...</h3>
        <ProgressBar value={importProgress} />
        <span>{Math.round(importProgress)}%</span>
      </div>
    {/if}
  </div>

  <svelte:fragment slot="footer">
    {#if $currentStep === 'upload'}
      <Button variant="outline" on:click={handleClose}>
        Cancel
      </Button>
    {:else if $currentStep === 'preview'}
      <Button variant="outline" on:click={() => currentStep.set('upload')}>
        Back
      </Button>
      <Button
        variant="primary"
        disabled={validSpecCount === 0}
        on:click={proceedToConflicts}
      >
        {#if hasConflicts}
          Resolve Conflicts
        {:else}
          Import {validSpecCount} Spec{validSpecCount !== 1 ? 's' : ''}
        {/if}
      </Button>
    {:else if $currentStep === 'conflicts'}
      <Button variant="outline" on:click={() => currentStep.set('preview')}>
        Back
      </Button>
      <Button variant="primary" on:click={handleImport}>
        <Icon name="download" size={14} />
        Import Specs
      </Button>
    {/if}
  </svelte:fragment>
</Modal>

<style>
  .import-dialog {
    min-height: 400px;
  }

  .import-dialog__upload {
    height: 100%;
  }

  .import-dialog__dropzone-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding: 48px;
    text-align: center;
    color: var(--color-text-secondary);
  }

  .import-dialog__dropzone-content h3 {
    font-size: 1.125rem;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0;
  }

  .import-dialog__dropzone-content p {
    font-size: 0.875rem;
    margin: 0;
  }

  .import-dialog__error {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 16px;
    padding: 12px 16px;
    background: var(--color-danger-subtle);
    color: var(--color-danger);
    border-radius: 6px;
    font-size: 0.875rem;
  }

  .import-dialog__summary {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 16px;
    margin-bottom: 24px;
  }

  .import-dialog__stat {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 16px;
    background: var(--color-surface-subtle);
    border-radius: 8px;
  }

  .import-dialog__stat-value {
    font-size: 2rem;
    font-weight: 700;
    color: var(--color-text-primary);
  }

  .import-dialog__stat-value--success {
    color: var(--color-success);
  }

  .import-dialog__stat-value--warning {
    color: var(--color-warning);
  }

  .import-dialog__stat-value--error {
    color: var(--color-danger);
  }

  .import-dialog__stat-label {
    font-size: 0.75rem;
    color: var(--color-text-tertiary);
    text-transform: uppercase;
  }

  .import-dialog__spec-list h4 {
    font-size: 0.875rem;
    font-weight: 600;
    margin: 0 0 12px;
  }

  .import-dialog__table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.875rem;
  }

  .import-dialog__table th,
  .import-dialog__table td {
    padding: 10px 12px;
    text-align: left;
    border-bottom: 1px solid var(--color-border);
  }

  .import-dialog__table th {
    font-weight: 600;
    color: var(--color-text-secondary);
    background: var(--color-surface-subtle);
  }

  .import-dialog__row--error {
    background: var(--color-danger-subtle);
  }

  .import-dialog__row--warning {
    background: var(--color-warning-subtle);
  }

  .import-dialog__spec-id {
    font-family: var(--font-mono);
    font-weight: 600;
    color: var(--color-primary);
  }

  .import-dialog__conflict-badge {
    margin-left: 8px;
    padding: 2px 6px;
    font-size: 0.625rem;
    font-weight: 600;
    background: var(--color-warning);
    color: white;
    border-radius: 3px;
  }

  .import-dialog__error-icon {
    color: var(--color-danger);
  }

  .import-dialog__remove-btn {
    padding: 4px;
    background: none;
    border: none;
    cursor: pointer;
    color: var(--color-text-tertiary);
    border-radius: 4px;
  }

  .import-dialog__remove-btn:hover {
    background: var(--color-danger-subtle);
    color: var(--color-danger);
  }

  .import-dialog__notice {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 16px;
    padding: 12px 16px;
    background: var(--color-info-subtle);
    color: var(--color-info);
    border-radius: 6px;
    font-size: 0.875rem;
  }

  .import-dialog__conflicts h4 {
    font-size: 1rem;
    font-weight: 600;
    margin: 0 0 8px;
  }

  .import-dialog__conflicts > p {
    font-size: 0.875rem;
    color: var(--color-text-secondary);
    margin: 0 0 20px;
  }

  .import-dialog__conflict-list {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .import-dialog__conflict-item {
    padding: 16px;
    background: var(--color-surface-subtle);
    border-radius: 8px;
  }

  .import-dialog__conflict-info {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 12px;
  }

  .import-dialog__spec-title {
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .import-dialog__conflict-options {
    display: flex;
    gap: 24px;
  }

  .import-dialog__radio {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 0.875rem;
    cursor: pointer;
  }

  .import-dialog__importing {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 16px;
    padding: 60px;
    text-align: center;
  }

  .import-dialog__importing h3 {
    font-size: 1.125rem;
    margin: 0;
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

### Import Utilities

```typescript
// utils/import.ts
import type { Spec } from '$lib/types/spec';
import JSZip from 'jszip';

export type ConflictResolution = 'skip' | 'replace' | 'rename';

export async function parseImportFile(file: File): Promise<Spec[]> {
  const extension = file.name.split('.').pop()?.toLowerCase();

  switch (extension) {
    case 'md':
      return parseMarkdownFile(file);
    case 'json':
      return parseJsonFile(file);
    case 'zip':
      return parseZipFile(file);
    default:
      throw new Error(`Unsupported file type: ${extension}`);
  }
}

async function parseMarkdownFile(file: File): Promise<Spec[]> {
  const content = await file.text();
  const specs: Spec[] = [];

  // Split by spec delimiter (---) or by # Spec headers
  const specBlocks = content.split(/(?=^# Spec \d+)/m).filter(block => block.trim());

  for (const block of specBlocks) {
    const spec = parseMarkdownSpec(block);
    if (spec) {
      specs.push(spec);
    }
  }

  return specs;
}

function parseMarkdownSpec(content: string): Spec | null {
  // Extract metadata from markdown
  const idMatch = content.match(/^# Spec (\d+)/m);
  const titleMatch = content.match(/^# Spec \d+:\s*(.+)/m);
  const phaseMatch = content.match(/^## Phase\s*\n(\d+)/m);
  const statusMatch = content.match(/^## Status\s*\n(\w+(?:-\w+)?)/m);
  const depsMatch = content.match(/^## Dependencies\s*\n([^\n#]+)/m);
  const contextMatch = content.match(/^## Estimated Context\s*\n([^\n#]+)/m);

  if (!idMatch) return null;

  const id = idMatch[1];
  const title = titleMatch?.[1]?.trim() || `Spec ${id}`;

  // Extract content (everything after the metadata sections)
  const contentMatch = content.match(/---\n([\s\S]+)$/);
  const specContent = contentMatch?.[1] || content;

  return {
    id,
    title,
    description: '',
    status: (statusMatch?.[1]?.toLowerCase() || 'planned') as any,
    phase: parseInt(phaseMatch?.[1] || '1', 10),
    dependencies: parseDependencies(depsMatch?.[1] || ''),
    estimatedContext: contextMatch?.[1]?.trim() || '~10%',
    tags: [],
    content: specContent.trim(),
    createdAt: new Date(),
    updatedAt: new Date()
  };
}

function parseDependencies(deps: string): string[] {
  if (!deps || deps.toLowerCase().includes('none')) return [];
  return deps
    .split(/[,\n]/)
    .map(d => d.trim())
    .filter(d => /^\d+$/.test(d) || /^Spec \d+$/.test(d))
    .map(d => d.replace(/^Spec /, ''));
}

async function parseJsonFile(file: File): Promise<Spec[]> {
  const content = await file.text();
  const data = JSON.parse(content);

  if (Array.isArray(data)) {
    return data.map(normalizeSpec);
  }

  if (data.specs && Array.isArray(data.specs)) {
    return data.specs.map(normalizeSpec);
  }

  if (data.id) {
    return [normalizeSpec(data)];
  }

  throw new Error('Invalid JSON format');
}

function normalizeSpec(data: any): Spec {
  return {
    id: String(data.id || ''),
    title: data.title || '',
    description: data.description || '',
    status: data.status || 'planned',
    phase: parseInt(data.phase, 10) || 1,
    dependencies: Array.isArray(data.dependencies) ? data.dependencies.map(String) : [],
    estimatedContext: data.estimatedContext || '~10%',
    tags: Array.isArray(data.tags) ? data.tags : [],
    content: data.content || '',
    createdAt: data.createdAt ? new Date(data.createdAt) : new Date(),
    updatedAt: data.updatedAt ? new Date(data.updatedAt) : new Date(),
    author: data.author
  };
}

async function parseZipFile(file: File): Promise<Spec[]> {
  const zip = await JSZip.loadAsync(file);
  const specs: Spec[] = [];

  for (const [path, zipEntry] of Object.entries(zip.files)) {
    if (zipEntry.dir) continue;

    const ext = path.split('.').pop()?.toLowerCase();
    if (ext !== 'md' && ext !== 'json') continue;

    const content = await zipEntry.async('text');
    const mockFile = new File([content], path, {
      type: ext === 'json' ? 'application/json' : 'text/markdown'
    });

    const parsed = ext === 'json'
      ? await parseJsonFile(mockFile)
      : await parseMarkdownFile(mockFile);

    specs.push(...parsed);
  }

  return specs;
}

export function validateImportedSpecs(
  specs: Spec[],
  existingSpecs: Spec[]
): Map<string, string[]> {
  const errors = new Map<string, string[]>();
  const seenIds = new Set<string>();

  for (const spec of specs) {
    const specErrors: string[] = [];

    // Check for duplicate IDs within import
    if (seenIds.has(spec.id)) {
      specErrors.push('Duplicate ID in import file');
    }
    seenIds.add(spec.id);

    // Required fields
    if (!spec.id) {
      specErrors.push('Missing spec ID');
    }
    if (!spec.title) {
      specErrors.push('Missing title');
    }

    // Validate status
    const validStatuses = ['planned', 'in-progress', 'implemented', 'tested', 'deprecated'];
    if (!validStatuses.includes(spec.status)) {
      specErrors.push(`Invalid status: ${spec.status}`);
    }

    // Validate phase
    if (spec.phase < 1 || spec.phase > 99) {
      specErrors.push('Phase must be between 1 and 99');
    }

    if (specErrors.length) {
      errors.set(spec.id || 'unknown', specErrors);
    }
  }

  return errors;
}
```

---

## Testing Requirements

### Unit Tests

```typescript
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import ImportDialog from './ImportDialog.svelte';
import { parseImportFile, validateImportedSpecs } from '$lib/utils/import';
import { createMockSpecs } from '$lib/test-utils/mock-data';

describe('ImportDialog', () => {
  const existingSpecs = createMockSpecs(3);

  it('shows dropzone initially', () => {
    render(ImportDialog, { props: { open: true, existingSpecs } });

    expect(screen.getByText('Drop files here to import')).toBeInTheDocument();
  });

  it('parses uploaded file and shows preview', async () => {
    render(ImportDialog, { props: { open: true, existingSpecs } });

    const jsonContent = JSON.stringify({
      specs: [{ id: '100', title: 'Test Spec', status: 'planned', phase: 1, content: 'Test' }]
    });

    const file = new File([jsonContent], 'specs.json', { type: 'application/json' });

    // Simulate file drop
    // Note: Testing file drops requires more setup
  });

  it('shows conflict count', async () => {
    // Test conflict detection
  });

  it('allows removing specs from import', async () => {
    // Test spec removal
  });
});

describe('parseImportFile', () => {
  it('parses JSON file with specs array', async () => {
    const content = JSON.stringify({
      specs: [
        { id: '100', title: 'Spec 1', status: 'planned', phase: 1, content: 'Content 1' },
        { id: '101', title: 'Spec 2', status: 'in-progress', phase: 2, content: 'Content 2' }
      ]
    });

    const file = new File([content], 'specs.json', { type: 'application/json' });
    const specs = await parseImportFile(file);

    expect(specs.length).toBe(2);
    expect(specs[0].id).toBe('100');
    expect(specs[1].id).toBe('101');
  });

  it('parses markdown file', async () => {
    const content = `# Spec 100: Test Spec

## Phase
1

## Status
planned

## Dependencies
None

---

## Objective

Test content here.
`;

    const file = new File([content], 'spec.md', { type: 'text/markdown' });
    const specs = await parseImportFile(file);

    expect(specs.length).toBe(1);
    expect(specs[0].id).toBe('100');
    expect(specs[0].title).toBe('Test Spec');
  });

  it('handles multiple specs in one markdown file', async () => {
    const content = `# Spec 100: First Spec

## Phase
1

## Status
planned

---

Content 1

# Spec 101: Second Spec

## Phase
2

## Status
in-progress

---

Content 2
`;

    const file = new File([content], 'specs.md', { type: 'text/markdown' });
    const specs = await parseImportFile(file);

    expect(specs.length).toBe(2);
  });

  it('throws error for unsupported file type', async () => {
    const file = new File(['test'], 'file.txt', { type: 'text/plain' });

    await expect(parseImportFile(file)).rejects.toThrow('Unsupported file type');
  });
});

describe('validateImportedSpecs', () => {
  it('detects missing required fields', () => {
    const specs = [
      { id: '', title: '', status: 'planned', phase: 1, content: '' }
    ] as any[];

    const errors = validateImportedSpecs(specs, []);

    expect(errors.get('')).toContain('Missing spec ID');
  });

  it('detects duplicate IDs within import', () => {
    const specs = [
      { id: '100', title: 'Spec 1', status: 'planned', phase: 1 },
      { id: '100', title: 'Spec 2', status: 'planned', phase: 1 }
    ] as any[];

    const errors = validateImportedSpecs(specs, []);

    expect(errors.get('100')).toContain('Duplicate ID in import file');
  });

  it('validates status values', () => {
    const specs = [
      { id: '100', title: 'Spec', status: 'invalid', phase: 1 }
    ] as any[];

    const errors = validateImportedSpecs(specs, []);

    expect(errors.get('100')?.[0]).toContain('Invalid status');
  });
});
```

---

## Related Specs

- Spec 239: Spec Validation
- Spec 247: Spec Export
- Spec 249: Batch Operations
