# Spec 237: Spec Editor

## Phase
11 - Spec Browser UI

## Spec ID
237

## Status
Planned

## Dependencies
- Spec 236 (Spec Detail View)
- Spec 238 (Markdown Preview)
- Spec 239 (Spec Validation)

## Estimated Context
~12%

---

## Objective

Create a comprehensive spec editor with real-time markdown preview, syntax highlighting, auto-save, validation feedback, and template support. The editor should provide a seamless editing experience with keyboard shortcuts and collaborative-friendly features.

---

## Acceptance Criteria

- [ ] Split-pane editor with live preview
- [ ] Syntax highlighting for markdown and code blocks
- [ ] Auto-save with conflict detection
- [ ] Form fields for metadata (status, phase, tags)
- [ ] Dependency selector with search
- [ ] Validation feedback in real-time
- [ ] Keyboard shortcuts for common actions
- [ ] Undo/redo support
- [ ] Draft saving and recovery
- [ ] Template insertion support

---

## Implementation Details

### SpecEditor.svelte

```svelte
<script lang="ts">
  import { createEventDispatcher, onMount, onDestroy } from 'svelte';
  import { writable, derived } from 'svelte/store';
  import { fade } from 'svelte/transition';
  import type { Spec, SpecStatus, ValidationResult } from '$lib/types/spec';
  import CodeMirror from '$lib/components/CodeMirror.svelte';
  import MarkdownPreview from './MarkdownPreview.svelte';
  import Button from '$lib/components/Button.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import Select from '$lib/components/Select.svelte';
  import TagInput from '$lib/components/TagInput.svelte';
  import DependencySelector from './DependencySelector.svelte';
  import ValidationPanel from './ValidationPanel.svelte';
  import { validateSpec } from '$lib/utils/validation';
  import { debounce } from '$lib/utils/timing';
  import { specStore } from '$lib/stores/spec-store';

  export let spec: Spec | null = null;
  export let isNew = false;
  export let autoSave = true;
  export let autoSaveDelay = 2000;

  const dispatch = createEventDispatcher<{
    save: Spec;
    cancel: void;
    delete: Spec;
    dirty: boolean;
  }>();

  // Editor state
  let editorContent = writable(spec?.content ?? '');
  let metadata = writable({
    id: spec?.id ?? '',
    title: spec?.title ?? '',
    status: spec?.status ?? 'planned' as SpecStatus,
    phase: spec?.phase ?? 1,
    dependencies: spec?.dependencies ?? [],
    tags: spec?.tags ?? [],
    estimatedContext: spec?.estimatedContext ?? '~10%',
    description: spec?.description ?? ''
  });

  let isDirty = false;
  let isSaving = false;
  let lastSaved: Date | null = null;
  let showPreview = true;
  let showValidation = true;
  let validationResults = writable<ValidationResult[]>([]);
  let editorRef: CodeMirror;
  let hasUnsavedDraft = false;

  // Status options
  const statusOptions = [
    { value: 'planned', label: 'Planned' },
    { value: 'in-progress', label: 'In Progress' },
    { value: 'implemented', label: 'Implemented' },
    { value: 'tested', label: 'Tested' },
    { value: 'deprecated', label: 'Deprecated' }
  ];

  // Available tags from existing specs
  const availableTags = derived(specStore, $specs => {
    const tagSet = new Set<string>();
    $specs.forEach(s => s.tags?.forEach(t => tagSet.add(t)));
    return Array.from(tagSet).sort();
  });

  // Debounced validation
  const debouncedValidate = debounce(() => {
    const fullSpec = buildSpec();
    const results = validateSpec(fullSpec, $specStore);
    validationResults.set(results);
  }, 500);

  // Debounced auto-save
  const debouncedAutoSave = debounce(() => {
    if (autoSave && isDirty) {
      saveDraft();
    }
  }, autoSaveDelay);

  // Watch for changes
  $: {
    $editorContent;
    $metadata;
    markDirty();
  }

  function markDirty() {
    if (!isDirty) {
      isDirty = true;
      dispatch('dirty', true);
    }
    debouncedValidate();
    debouncedAutoSave();
  }

  function buildSpec(): Spec {
    return {
      id: $metadata.id,
      title: $metadata.title,
      description: $metadata.description,
      status: $metadata.status,
      phase: $metadata.phase,
      dependencies: $metadata.dependencies,
      tags: $metadata.tags,
      estimatedContext: $metadata.estimatedContext,
      content: $editorContent,
      createdAt: spec?.createdAt ?? new Date(),
      updatedAt: new Date(),
      author: spec?.author
    };
  }

  async function handleSave() {
    if ($validationResults.some(r => r.severity === 'error')) {
      // Show validation errors
      return;
    }

    isSaving = true;
    try {
      const updatedSpec = buildSpec();
      dispatch('save', updatedSpec);
      isDirty = false;
      lastSaved = new Date();
      clearDraft();
    } finally {
      isSaving = false;
    }
  }

  function handleCancel() {
    if (isDirty) {
      const confirmed = confirm('You have unsaved changes. Discard them?');
      if (!confirmed) return;
    }
    clearDraft();
    dispatch('cancel');
  }

  function saveDraft() {
    const draft = {
      spec: buildSpec(),
      savedAt: new Date().toISOString()
    };
    localStorage.setItem(`spec-draft-${$metadata.id || 'new'}`, JSON.stringify(draft));
  }

  function loadDraft() {
    const key = `spec-draft-${$metadata.id || 'new'}`;
    const stored = localStorage.getItem(key);
    if (stored) {
      try {
        const draft = JSON.parse(stored);
        return draft;
      } catch {
        return null;
      }
    }
    return null;
  }

  function clearDraft() {
    localStorage.removeItem(`spec-draft-${$metadata.id || 'new'}`);
    hasUnsavedDraft = false;
  }

  function recoverDraft() {
    const draft = loadDraft();
    if (draft) {
      editorContent.set(draft.spec.content);
      metadata.update(m => ({
        ...m,
        ...draft.spec,
        id: m.id // Keep original ID
      }));
      hasUnsavedDraft = false;
    }
  }

  function insertTemplate(template: string) {
    editorRef?.insertText(template);
  }

  function insertSnippet(type: string) {
    const snippets: Record<string, string> = {
      'code-block': '\n```typescript\n// Code here\n```\n',
      'checkbox': '\n- [ ] ',
      'table': '\n| Column 1 | Column 2 |\n|----------|----------|\n| Cell 1   | Cell 2   |\n',
      'link': '[text](url)',
      'section': '\n## Section Title\n\nContent here.\n'
    };

    if (snippets[type]) {
      insertTemplate(snippets[type]);
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if ((event.metaKey || event.ctrlKey) && event.key === 's') {
      event.preventDefault();
      handleSave();
    } else if (event.key === 'Escape') {
      handleCancel();
    } else if ((event.metaKey || event.ctrlKey) && event.key === 'p') {
      event.preventDefault();
      showPreview = !showPreview;
    }
  }

  onMount(() => {
    // Check for unsaved draft
    const draft = loadDraft();
    if (draft && draft.savedAt) {
      const draftDate = new Date(draft.savedAt);
      const specDate = spec?.updatedAt ? new Date(spec.updatedAt) : null;

      if (!specDate || draftDate > specDate) {
        hasUnsavedDraft = true;
      }
    }

    // Set up beforeunload handler
    const handleBeforeUnload = (e: BeforeUnloadEvent) => {
      if (isDirty) {
        e.preventDefault();
        e.returnValue = '';
      }
    };

    window.addEventListener('beforeunload', handleBeforeUnload);
    return () => window.removeEventListener('beforeunload', handleBeforeUnload);
  });
</script>

<div class="spec-editor" on:keydown={handleKeydown}>
  <!-- Header -->
  <header class="spec-editor__header">
    <div class="spec-editor__header-left">
      <h2>{isNew ? 'Create New Spec' : `Edit Spec ${$metadata.id}`}</h2>
      {#if isDirty}
        <span class="spec-editor__dirty-indicator">
          <Icon name="circle" size={8} />
          Unsaved changes
        </span>
      {:else if lastSaved}
        <span class="spec-editor__saved-indicator">
          Saved {lastSaved.toLocaleTimeString()}
        </span>
      {/if}
    </div>

    <div class="spec-editor__header-actions">
      <Button variant="ghost" on:click={() => showPreview = !showPreview}>
        <Icon name={showPreview ? 'eye-off' : 'eye'} size={16} />
        {showPreview ? 'Hide' : 'Show'} Preview
      </Button>
      <Button variant="ghost" on:click={() => showValidation = !showValidation}>
        <Icon name="alert-circle" size={16} />
        Validation
        {#if $validationResults.length > 0}
          <span class="spec-editor__validation-count">
            {$validationResults.length}
          </span>
        {/if}
      </Button>
      <Button variant="outline" on:click={handleCancel}>
        Cancel
      </Button>
      <Button
        variant="primary"
        on:click={handleSave}
        loading={isSaving}
        disabled={$validationResults.some(r => r.severity === 'error')}
      >
        <Icon name="save" size={16} />
        Save
      </Button>
    </div>
  </header>

  <!-- Draft recovery banner -->
  {#if hasUnsavedDraft}
    <div class="spec-editor__draft-banner" transition:fade>
      <Icon name="file-text" size={16} />
      <span>You have an unsaved draft from a previous session.</span>
      <Button variant="ghost" size="sm" on:click={recoverDraft}>
        Recover draft
      </Button>
      <Button variant="ghost" size="sm" on:click={clearDraft}>
        Discard
      </Button>
    </div>
  {/if}

  <!-- Metadata Form -->
  <div class="spec-editor__metadata">
    <div class="spec-editor__field spec-editor__field--id">
      <label for="spec-id">ID</label>
      <input
        id="spec-id"
        type="text"
        bind:value={$metadata.id}
        placeholder="e.g., 237"
        disabled={!isNew}
      />
    </div>

    <div class="spec-editor__field spec-editor__field--title">
      <label for="spec-title">Title</label>
      <input
        id="spec-title"
        type="text"
        bind:value={$metadata.title}
        placeholder="Spec title"
      />
    </div>

    <div class="spec-editor__field">
      <label for="spec-status">Status</label>
      <Select
        id="spec-status"
        options={statusOptions}
        bind:value={$metadata.status}
      />
    </div>

    <div class="spec-editor__field">
      <label for="spec-phase">Phase</label>
      <input
        id="spec-phase"
        type="number"
        min="1"
        max="99"
        bind:value={$metadata.phase}
      />
    </div>

    <div class="spec-editor__field">
      <label for="spec-context">Est. Context</label>
      <input
        id="spec-context"
        type="text"
        bind:value={$metadata.estimatedContext}
        placeholder="~10%"
      />
    </div>

    <div class="spec-editor__field spec-editor__field--tags">
      <label>Tags</label>
      <TagInput
        bind:tags={$metadata.tags}
        suggestions={$availableTags}
        placeholder="Add tags..."
      />
    </div>

    <div class="spec-editor__field spec-editor__field--deps">
      <label>Dependencies</label>
      <DependencySelector
        bind:selected={$metadata.dependencies}
        currentSpecId={$metadata.id}
      />
    </div>

    <div class="spec-editor__field spec-editor__field--desc">
      <label for="spec-desc">Description</label>
      <textarea
        id="spec-desc"
        bind:value={$metadata.description}
        placeholder="Brief description of the spec"
        rows="2"
      />
    </div>
  </div>

  <!-- Toolbar -->
  <div class="spec-editor__toolbar">
    <div class="spec-editor__toolbar-group">
      <Button variant="ghost" size="sm" on:click={() => insertSnippet('section')}>
        <Icon name="heading" size={14} />
        Section
      </Button>
      <Button variant="ghost" size="sm" on:click={() => insertSnippet('code-block')}>
        <Icon name="code" size={14} />
        Code
      </Button>
      <Button variant="ghost" size="sm" on:click={() => insertSnippet('checkbox')}>
        <Icon name="check-square" size={14} />
        Checkbox
      </Button>
      <Button variant="ghost" size="sm" on:click={() => insertSnippet('table')}>
        <Icon name="table" size={14} />
        Table
      </Button>
      <Button variant="ghost" size="sm" on:click={() => insertSnippet('link')}>
        <Icon name="link" size={14} />
        Link
      </Button>
    </div>

    <div class="spec-editor__toolbar-group">
      <span class="spec-editor__word-count">
        {$editorContent.split(/\s+/).filter(Boolean).length} words
      </span>
    </div>
  </div>

  <!-- Editor Body -->
  <div class="spec-editor__body" class:spec-editor__body--split={showPreview}>
    <div class="spec-editor__editor-pane">
      <CodeMirror
        bind:this={editorRef}
        bind:value={$editorContent}
        language="markdown"
        theme="github-dark"
        lineNumbers
        lineWrapping
        placeholder="Write your spec content here..."
      />
    </div>

    {#if showPreview}
      <div class="spec-editor__preview-pane">
        <div class="spec-editor__preview-header">
          <span>Preview</span>
        </div>
        <div class="spec-editor__preview-content">
          <MarkdownPreview content={$editorContent} />
        </div>
      </div>
    {/if}
  </div>

  <!-- Validation Panel -->
  {#if showValidation && $validationResults.length > 0}
    <div class="spec-editor__validation" transition:fade>
      <ValidationPanel results={$validationResults} on:fix />
    </div>
  {/if}
</div>

<style>
  .spec-editor {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--color-surface);
  }

  .spec-editor__header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 20px;
    border-bottom: 1px solid var(--color-border);
    background: var(--color-surface-subtle);
  }

  .spec-editor__header-left {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .spec-editor__header-left h2 {
    font-size: 1rem;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0;
  }

  .spec-editor__dirty-indicator {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 0.75rem;
    color: var(--color-warning);
  }

  .spec-editor__saved-indicator {
    font-size: 0.75rem;
    color: var(--color-text-tertiary);
  }

  .spec-editor__header-actions {
    display: flex;
    gap: 8px;
  }

  .spec-editor__validation-count {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 16px;
    height: 16px;
    padding: 0 4px;
    font-size: 0.625rem;
    font-weight: 600;
    background: var(--color-warning);
    color: white;
    border-radius: 8px;
  }

  .spec-editor__draft-banner {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 20px;
    background: var(--color-info-subtle);
    border-bottom: 1px solid var(--color-info-alpha);
    font-size: 0.875rem;
    color: var(--color-info);
  }

  .spec-editor__metadata {
    display: grid;
    grid-template-columns: auto 1fr auto auto auto;
    gap: 16px;
    padding: 16px 20px;
    border-bottom: 1px solid var(--color-border);
    background: var(--color-surface);
  }

  .spec-editor__field {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .spec-editor__field label {
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--color-text-secondary);
  }

  .spec-editor__field input,
  .spec-editor__field textarea {
    padding: 8px 12px;
    font-size: 0.875rem;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    background: var(--color-surface);
  }

  .spec-editor__field input:focus,
  .spec-editor__field textarea:focus {
    outline: none;
    border-color: var(--color-primary);
    box-shadow: 0 0 0 3px var(--color-primary-alpha);
  }

  .spec-editor__field--id {
    width: 80px;
  }

  .spec-editor__field--title {
    grid-column: span 2;
  }

  .spec-editor__field--tags {
    grid-column: span 3;
  }

  .spec-editor__field--deps {
    grid-column: span 2;
  }

  .spec-editor__field--desc {
    grid-column: 1 / -1;
  }

  .spec-editor__toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 20px;
    border-bottom: 1px solid var(--color-border);
    background: var(--color-surface-subtle);
  }

  .spec-editor__toolbar-group {
    display: flex;
    gap: 4px;
  }

  .spec-editor__word-count {
    font-size: 0.75rem;
    color: var(--color-text-tertiary);
  }

  .spec-editor__body {
    flex: 1;
    display: flex;
    overflow: hidden;
  }

  .spec-editor__body--split {
    /* Split view active */
  }

  .spec-editor__editor-pane {
    flex: 1;
    min-width: 0;
    overflow: hidden;
  }

  .spec-editor__body--split .spec-editor__editor-pane {
    flex: 0 0 50%;
    border-right: 1px solid var(--color-border);
  }

  .spec-editor__preview-pane {
    flex: 0 0 50%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .spec-editor__preview-header {
    padding: 8px 16px;
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--color-text-secondary);
    border-bottom: 1px solid var(--color-border);
    background: var(--color-surface-subtle);
  }

  .spec-editor__preview-content {
    flex: 1;
    overflow-y: auto;
    padding: 20px;
  }

  .spec-editor__validation {
    border-top: 1px solid var(--color-border);
    max-height: 200px;
    overflow-y: auto;
  }
</style>
```

### DependencySelector.svelte

```svelte
<script lang="ts">
  import { derived } from 'svelte/store';
  import type { Spec } from '$lib/types/spec';
  import Icon from '$lib/components/Icon.svelte';
  import { specStore } from '$lib/stores/spec-store';

  export let selected: string[] = [];
  export let currentSpecId: string = '';

  let searchQuery = '';
  let isOpen = false;

  // Filter out current spec and already selected
  $: availableSpecs = $specStore.filter(s =>
    s.id !== currentSpecId &&
    !selected.includes(s.id) &&
    (searchQuery === '' ||
      s.id.toLowerCase().includes(searchQuery.toLowerCase()) ||
      s.title.toLowerCase().includes(searchQuery.toLowerCase()))
  );

  $: selectedSpecs = selected
    .map(id => $specStore.find(s => s.id === id))
    .filter(Boolean) as Spec[];

  function addDependency(specId: string) {
    selected = [...selected, specId];
    searchQuery = '';
  }

  function removeDependency(specId: string) {
    selected = selected.filter(id => id !== specId);
  }
</script>

<div class="dep-selector">
  <div class="dep-selector__selected">
    {#each selectedSpecs as spec}
      <span class="dep-selector__tag">
        <span class="dep-selector__tag-id">{spec.id}</span>
        <span class="dep-selector__tag-title">{spec.title}</span>
        <button
          class="dep-selector__tag-remove"
          on:click={() => removeDependency(spec.id)}
          aria-label="Remove dependency"
        >
          <Icon name="x" size={12} />
        </button>
      </span>
    {/each}

    <input
      type="text"
      class="dep-selector__input"
      bind:value={searchQuery}
      on:focus={() => isOpen = true}
      on:blur={() => setTimeout(() => isOpen = false, 150)}
      placeholder={selected.length === 0 ? 'Search specs...' : ''}
    />
  </div>

  {#if isOpen && availableSpecs.length > 0}
    <ul class="dep-selector__dropdown">
      {#each availableSpecs.slice(0, 10) as spec}
        <li>
          <button
            class="dep-selector__option"
            on:mousedown|preventDefault={() => addDependency(spec.id)}
          >
            <span class="dep-selector__option-id">{spec.id}</span>
            <span class="dep-selector__option-title">{spec.title}</span>
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .dep-selector {
    position: relative;
  }

  .dep-selector__selected {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    padding: 6px;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    background: var(--color-surface);
    min-height: 38px;
  }

  .dep-selector__selected:focus-within {
    border-color: var(--color-primary);
    box-shadow: 0 0 0 3px var(--color-primary-alpha);
  }

  .dep-selector__tag {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 2px 6px;
    background: var(--color-surface-elevated);
    border-radius: 4px;
    font-size: 0.75rem;
  }

  .dep-selector__tag-id {
    font-weight: 600;
    color: var(--color-primary);
  }

  .dep-selector__tag-title {
    color: var(--color-text-secondary);
    max-width: 100px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .dep-selector__tag-remove {
    display: flex;
    padding: 2px;
    background: none;
    border: none;
    cursor: pointer;
    color: var(--color-text-tertiary);
    border-radius: 2px;
  }

  .dep-selector__tag-remove:hover {
    background: var(--color-hover);
    color: var(--color-text-primary);
  }

  .dep-selector__input {
    flex: 1;
    min-width: 100px;
    padding: 4px;
    border: none;
    background: none;
    font-size: 0.875rem;
    outline: none;
  }

  .dep-selector__dropdown {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    margin-top: 4px;
    padding: 4px 0;
    list-style: none;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    box-shadow: var(--shadow-lg);
    z-index: 10;
    max-height: 200px;
    overflow-y: auto;
  }

  .dep-selector__option {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 8px 12px;
    text-align: left;
    background: none;
    border: none;
    cursor: pointer;
  }

  .dep-selector__option:hover {
    background: var(--color-hover);
  }

  .dep-selector__option-id {
    font-family: var(--font-mono);
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--color-primary);
  }

  .dep-selector__option-title {
    font-size: 0.875rem;
    color: var(--color-text-primary);
  }
</style>
```

---

## Testing Requirements

### Unit Tests

```typescript
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import SpecEditor from './SpecEditor.svelte';
import { createMockSpec, createMockSpecs } from '$lib/test-utils/mock-data';
import { specStore } from '$lib/stores/spec-store';

describe('SpecEditor', () => {
  const mockSpec = createMockSpec({
    id: '237',
    title: 'Spec Editor',
    status: 'in-progress',
    content: '## Objective\n\nEditor implementation.'
  });

  beforeEach(() => {
    localStorage.clear();
    specStore.set(createMockSpecs(10));
  });

  it('renders editor with spec data', () => {
    render(SpecEditor, { props: { spec: mockSpec } });

    expect(screen.getByText('Edit Spec 237')).toBeInTheDocument();
    expect(screen.getByDisplayValue('Spec Editor')).toBeInTheDocument();
  });

  it('renders new spec mode', () => {
    render(SpecEditor, { props: { spec: null, isNew: true } });

    expect(screen.getByText('Create New Spec')).toBeInTheDocument();
  });

  it('shows dirty indicator on changes', async () => {
    render(SpecEditor, { props: { spec: mockSpec } });

    const titleInput = screen.getByDisplayValue('Spec Editor');
    await fireEvent.input(titleInput, { target: { value: 'Updated Title' } });

    expect(screen.getByText('Unsaved changes')).toBeInTheDocument();
  });

  it('handles save action', async () => {
    const { component } = render(SpecEditor, { props: { spec: mockSpec } });

    const saveHandler = vi.fn();
    component.$on('save', saveHandler);

    await fireEvent.click(screen.getByText('Save'));

    expect(saveHandler).toHaveBeenCalled();
  });

  it('prompts before cancel with unsaved changes', async () => {
    const confirmSpy = vi.spyOn(window, 'confirm').mockReturnValue(false);

    render(SpecEditor, { props: { spec: mockSpec } });

    // Make a change
    const titleInput = screen.getByDisplayValue('Spec Editor');
    await fireEvent.input(titleInput, { target: { value: 'Changed' } });

    await fireEvent.click(screen.getByText('Cancel'));

    expect(confirmSpy).toHaveBeenCalled();
    confirmSpy.mockRestore();
  });

  it('toggles preview pane', async () => {
    render(SpecEditor, { props: { spec: mockSpec } });

    expect(screen.getByText('Preview')).toBeInTheDocument();

    await fireEvent.click(screen.getByText('Hide Preview'));

    expect(screen.queryByText('Preview')).not.toBeInTheDocument();
  });

  it('inserts code snippet', async () => {
    const { component } = render(SpecEditor, { props: { spec: mockSpec } });

    await fireEvent.click(screen.getByText('Code'));

    // Editor should contain code block snippet
    // (This depends on CodeMirror integration)
  });

  it('saves draft to localStorage', async () => {
    render(SpecEditor, { props: { spec: mockSpec, autoSave: true, autoSaveDelay: 100 } });

    const titleInput = screen.getByDisplayValue('Spec Editor');
    await fireEvent.input(titleInput, { target: { value: 'Draft Title' } });

    await waitFor(() => {
      const draft = localStorage.getItem('spec-draft-237');
      expect(draft).not.toBeNull();
    }, { timeout: 500 });
  });

  it('shows draft recovery banner', () => {
    localStorage.setItem('spec-draft-237', JSON.stringify({
      spec: { ...mockSpec, title: 'Draft Version' },
      savedAt: new Date().toISOString()
    }));

    render(SpecEditor, { props: { spec: mockSpec } });

    expect(screen.getByText(/unsaved draft/)).toBeInTheDocument();
  });

  it('recovers draft on request', async () => {
    localStorage.setItem('spec-draft-237', JSON.stringify({
      spec: { ...mockSpec, title: 'Draft Version' },
      savedAt: new Date().toISOString()
    }));

    render(SpecEditor, { props: { spec: mockSpec } });

    await fireEvent.click(screen.getByText('Recover draft'));

    expect(screen.getByDisplayValue('Draft Version')).toBeInTheDocument();
  });

  it('handles keyboard shortcuts', async () => {
    const { component } = render(SpecEditor, { props: { spec: mockSpec } });

    const saveHandler = vi.fn();
    component.$on('save', saveHandler);

    await fireEvent.keyDown(document.body, { key: 's', metaKey: true });

    expect(saveHandler).toHaveBeenCalled();
  });
});

describe('DependencySelector', () => {
  beforeEach(() => {
    specStore.set(createMockSpecs(10));
  });

  it('displays selected dependencies', () => {
    render(DependencySelector, {
      props: { selected: ['231', '232'], currentSpecId: '237' }
    });

    expect(screen.getByText('231')).toBeInTheDocument();
    expect(screen.getByText('232')).toBeInTheDocument();
  });

  it('searches available specs', async () => {
    render(DependencySelector, {
      props: { selected: [], currentSpecId: '237' }
    });

    const input = screen.getByPlaceholderText('Search specs...');
    await fireEvent.focus(input);
    await fireEvent.input(input, { target: { value: '231' } });

    expect(screen.getByText('231')).toBeInTheDocument();
  });

  it('removes dependency on click', async () => {
    const { component } = render(DependencySelector, {
      props: { selected: ['231'], currentSpecId: '237' }
    });

    await fireEvent.click(screen.getByLabelText('Remove dependency'));

    // Component should update selected
  });
});
```

---

## Related Specs

- Spec 236: Spec Detail View
- Spec 238: Markdown Preview
- Spec 239: Spec Validation
- Spec 240: Spec Templates
- Spec 241: Spec Creation Form
