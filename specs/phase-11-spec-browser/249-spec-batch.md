# Spec 249: Batch Operations

## Phase
11 - Spec Browser UI

## Spec ID
249

## Status
Planned

## Dependencies
- Spec 231 (Spec List Layout)
- Spec 242 (Spec Status Tracking)
- Spec 247 (Spec Export)

## Estimated Context
~9%

---

## Objective

Implement batch operation capabilities for specs including multi-select, bulk status updates, bulk deletion, bulk tagging, and batch export. Provide undo functionality and progress feedback for long operations.

---

## Acceptance Criteria

- [ ] Multi-select specs with Shift+Click
- [ ] Select all / deselect all
- [ ] Batch status change
- [ ] Batch delete with confirmation
- [ ] Batch tag add/remove
- [ ] Batch phase update
- [ ] Batch export selected
- [ ] Progress indicator for operations
- [ ] Undo last batch operation
- [ ] Keyboard shortcuts for batch actions

---

## Implementation Details

### BatchActionBar.svelte

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { fade, fly } from 'svelte/transition';
  import type { Spec, SpecStatus } from '$lib/types/spec';
  import Button from '$lib/components/Button.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import Dropdown from '$lib/components/Dropdown.svelte';
  import TagInput from '$lib/components/TagInput.svelte';
  import ProgressBar from '$lib/components/ProgressBar.svelte';

  export let selectedSpecs: Spec[] = [];
  export let totalSpecs: number = 0;
  export let availableTags: string[] = [];

  const dispatch = createEventDispatcher<{
    clear: void;
    selectAll: void;
    statusChange: { specs: Spec[]; status: SpecStatus };
    delete: { specs: Spec[] };
    tagAdd: { specs: Spec[]; tags: string[] };
    tagRemove: { specs: Spec[]; tags: string[] };
    phaseChange: { specs: Spec[]; phase: number };
    export: { specs: Spec[] };
    duplicate: { specs: Spec[] };
  }>();

  let showStatusMenu = false;
  let showTagMenu = false;
  let showPhaseMenu = false;
  let tagAction: 'add' | 'remove' = 'add';
  let selectedTags: string[] = [];
  let selectedPhase: number | null = null;
  let isProcessing = false;
  let progress = 0;

  const statusOptions: { value: SpecStatus; label: string; color: string }[] = [
    { value: 'planned', label: 'Planned', color: 'var(--color-status-planned)' },
    { value: 'in-progress', label: 'In Progress', color: 'var(--color-status-progress)' },
    { value: 'implemented', label: 'Implemented', color: 'var(--color-status-implemented)' },
    { value: 'tested', label: 'Tested', color: 'var(--color-status-tested)' },
    { value: 'deprecated', label: 'Deprecated', color: 'var(--color-status-deprecated)' }
  ];

  $: selectedCount = selectedSpecs.length;
  $: allSelected = selectedCount === totalSpecs && totalSpecs > 0;

  function handleStatusChange(status: SpecStatus) {
    dispatch('statusChange', { specs: selectedSpecs, status });
    showStatusMenu = false;
  }

  function handleTagAction() {
    if (selectedTags.length === 0) return;

    if (tagAction === 'add') {
      dispatch('tagAdd', { specs: selectedSpecs, tags: selectedTags });
    } else {
      dispatch('tagRemove', { specs: selectedSpecs, tags: selectedTags });
    }

    selectedTags = [];
    showTagMenu = false;
  }

  function handlePhaseChange() {
    if (selectedPhase === null) return;

    dispatch('phaseChange', { specs: selectedSpecs, phase: selectedPhase });
    selectedPhase = null;
    showPhaseMenu = false;
  }

  function handleDelete() {
    const confirmed = confirm(
      `Are you sure you want to delete ${selectedCount} spec${selectedCount !== 1 ? 's' : ''}? This action cannot be undone.`
    );

    if (confirmed) {
      dispatch('delete', { specs: selectedSpecs });
    }
  }

  function handleExport() {
    dispatch('export', { specs: selectedSpecs });
  }

  function handleDuplicate() {
    dispatch('duplicate', { specs: selectedSpecs });
  }

  function handleSelectAll() {
    if (allSelected) {
      dispatch('clear');
    } else {
      dispatch('selectAll');
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (selectedCount === 0) return;

    if (event.key === 'Delete' || event.key === 'Backspace') {
      if (event.metaKey || event.ctrlKey) {
        handleDelete();
      }
    } else if (event.key === 'Escape') {
      dispatch('clear');
    } else if (event.key === 'e' && (event.metaKey || event.ctrlKey)) {
      event.preventDefault();
      handleExport();
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if selectedCount > 0}
  <div
    class="batch-action-bar"
    transition:fly={{ y: 50, duration: 200 }}
  >
    <div class="batch-action-bar__selection">
      <button
        class="batch-action-bar__checkbox"
        on:click={handleSelectAll}
        aria-label={allSelected ? 'Deselect all' : 'Select all'}
      >
        {#if allSelected}
          <Icon name="check-square" size={18} />
        {:else}
          <Icon name="minus-square" size={18} />
        {/if}
      </button>
      <span class="batch-action-bar__count">
        {selectedCount} of {totalSpecs} selected
      </span>
      <button
        class="batch-action-bar__clear"
        on:click={() => dispatch('clear')}
      >
        Clear selection
      </button>
    </div>

    <div class="batch-action-bar__actions">
      <!-- Status Change -->
      <Dropdown bind:open={showStatusMenu} placement="top">
        <Button slot="trigger" variant="ghost" size="sm">
          <Icon name="activity" size={14} />
          Status
          <Icon name="chevron-up" size={12} />
        </Button>
        <div class="batch-action-bar__menu">
          {#each statusOptions as option}
            <button
              class="batch-action-bar__menu-item"
              on:click={() => handleStatusChange(option.value)}
            >
              <span
                class="batch-action-bar__status-dot"
                style:background={option.color}
              />
              {option.label}
            </button>
          {/each}
        </div>
      </Dropdown>

      <!-- Tags -->
      <Dropdown bind:open={showTagMenu} placement="top">
        <Button slot="trigger" variant="ghost" size="sm">
          <Icon name="tag" size={14} />
          Tags
          <Icon name="chevron-up" size={12} />
        </Button>
        <div class="batch-action-bar__menu batch-action-bar__menu--wide">
          <div class="batch-action-bar__tag-action">
            <label>
              <input
                type="radio"
                bind:group={tagAction}
                value="add"
              />
              Add tags
            </label>
            <label>
              <input
                type="radio"
                bind:group={tagAction}
                value="remove"
              />
              Remove tags
            </label>
          </div>
          <TagInput
            bind:tags={selectedTags}
            suggestions={availableTags}
            placeholder="Select tags..."
          />
          <Button
            variant="primary"
            size="sm"
            disabled={selectedTags.length === 0}
            on:click={handleTagAction}
          >
            Apply to {selectedCount} spec{selectedCount !== 1 ? 's' : ''}
          </Button>
        </div>
      </Dropdown>

      <!-- Phase -->
      <Dropdown bind:open={showPhaseMenu} placement="top">
        <Button slot="trigger" variant="ghost" size="sm">
          <Icon name="layers" size={14} />
          Phase
          <Icon name="chevron-up" size={12} />
        </Button>
        <div class="batch-action-bar__menu">
          <div class="batch-action-bar__phase-input">
            <input
              type="number"
              min="1"
              max="99"
              bind:value={selectedPhase}
              placeholder="Phase #"
            />
            <Button
              variant="primary"
              size="sm"
              disabled={selectedPhase === null}
              on:click={handlePhaseChange}
            >
              Apply
            </Button>
          </div>
        </div>
      </Dropdown>

      <div class="batch-action-bar__divider" />

      <!-- Export -->
      <Button variant="ghost" size="sm" on:click={handleExport}>
        <Icon name="download" size={14} />
        Export
      </Button>

      <!-- Duplicate -->
      <Button variant="ghost" size="sm" on:click={handleDuplicate}>
        <Icon name="copy" size={14} />
        Duplicate
      </Button>

      <div class="batch-action-bar__divider" />

      <!-- Delete -->
      <Button variant="ghost" size="sm" on:click={handleDelete}>
        <Icon name="trash" size={14} />
        Delete
      </Button>
    </div>

    {#if isProcessing}
      <div class="batch-action-bar__progress">
        <ProgressBar value={progress} />
      </div>
    {/if}
  </div>
{/if}

<style>
  .batch-action-bar {
    position: fixed;
    bottom: 24px;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 12px 20px;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 12px;
    box-shadow: var(--shadow-xl);
    z-index: 100;
  }

  .batch-action-bar__selection {
    display: flex;
    align-items: center;
    gap: 12px;
    padding-right: 16px;
    border-right: 1px solid var(--color-border);
  }

  .batch-action-bar__checkbox {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 4px;
    background: none;
    border: none;
    cursor: pointer;
    color: var(--color-primary);
  }

  .batch-action-bar__count {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .batch-action-bar__clear {
    font-size: 0.75rem;
    color: var(--color-text-tertiary);
    background: none;
    border: none;
    cursor: pointer;
    text-decoration: underline;
  }

  .batch-action-bar__clear:hover {
    color: var(--color-primary);
  }

  .batch-action-bar__actions {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .batch-action-bar__divider {
    width: 1px;
    height: 24px;
    background: var(--color-border);
    margin: 0 8px;
  }

  .batch-action-bar__menu {
    padding: 8px 0;
    min-width: 160px;
  }

  .batch-action-bar__menu--wide {
    padding: 16px;
    min-width: 280px;
  }

  .batch-action-bar__menu-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 8px 16px;
    font-size: 0.875rem;
    text-align: left;
    background: none;
    border: none;
    cursor: pointer;
  }

  .batch-action-bar__menu-item:hover {
    background: var(--color-hover);
  }

  .batch-action-bar__status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }

  .batch-action-bar__tag-action {
    display: flex;
    gap: 16px;
    margin-bottom: 12px;
    font-size: 0.875rem;
  }

  .batch-action-bar__tag-action label {
    display: flex;
    align-items: center;
    gap: 6px;
    cursor: pointer;
  }

  .batch-action-bar__phase-input {
    display: flex;
    gap: 8px;
    padding: 8px 12px;
  }

  .batch-action-bar__phase-input input {
    width: 80px;
    padding: 6px 10px;
    font-size: 0.875rem;
    border: 1px solid var(--color-border);
    border-radius: 4px;
  }

  .batch-action-bar__progress {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    height: 3px;
    border-radius: 0 0 12px 12px;
    overflow: hidden;
  }
</style>
```

### Batch Operations Store

```typescript
// stores/batch-operations.ts
import { writable, derived } from 'svelte/store';
import type { Spec, SpecStatus } from '$lib/types/spec';

interface BatchOperation {
  id: string;
  type: 'status' | 'tag' | 'phase' | 'delete' | 'duplicate';
  specs: Spec[];
  previousState: Map<string, Partial<Spec>>;
  timestamp: Date;
}

const operationHistory = writable<BatchOperation[]>([]);
const maxHistorySize = 10;

export function recordOperation(operation: Omit<BatchOperation, 'id' | 'timestamp'>) {
  const newOp: BatchOperation = {
    ...operation,
    id: crypto.randomUUID(),
    timestamp: new Date()
  };

  operationHistory.update(history => {
    const updated = [newOp, ...history].slice(0, maxHistorySize);
    return updated;
  });
}

export function undoLastOperation(): BatchOperation | null {
  let undoneOp: BatchOperation | null = null;

  operationHistory.update(history => {
    if (history.length === 0) return history;

    [undoneOp, ...history] = history;
    return history;
  });

  return undoneOp;
}

export const canUndo = derived(operationHistory, $history => $history.length > 0);

export const lastOperation = derived(
  operationHistory,
  $history => $history[0] ?? null
);

// Batch operation executors
export async function batchUpdateStatus(
  specs: Spec[],
  status: SpecStatus,
  onProgress?: (progress: number) => void
): Promise<void> {
  const previousState = new Map<string, Partial<Spec>>();

  for (let i = 0; i < specs.length; i++) {
    const spec = specs[i];
    previousState.set(spec.id, { status: spec.status });

    // Update spec status
    spec.status = status;

    onProgress?.((i + 1) / specs.length * 100);
  }

  recordOperation({
    type: 'status',
    specs,
    previousState
  });
}

export async function batchAddTags(
  specs: Spec[],
  tags: string[],
  onProgress?: (progress: number) => void
): Promise<void> {
  const previousState = new Map<string, Partial<Spec>>();

  for (let i = 0; i < specs.length; i++) {
    const spec = specs[i];
    previousState.set(spec.id, { tags: [...(spec.tags || [])] });

    // Add tags
    const currentTags = new Set(spec.tags || []);
    tags.forEach(tag => currentTags.add(tag));
    spec.tags = Array.from(currentTags);

    onProgress?.((i + 1) / specs.length * 100);
  }

  recordOperation({
    type: 'tag',
    specs,
    previousState
  });
}

export async function batchRemoveTags(
  specs: Spec[],
  tags: string[],
  onProgress?: (progress: number) => void
): Promise<void> {
  const previousState = new Map<string, Partial<Spec>>();
  const tagsToRemove = new Set(tags);

  for (let i = 0; i < specs.length; i++) {
    const spec = specs[i];
    previousState.set(spec.id, { tags: [...(spec.tags || [])] });

    // Remove tags
    spec.tags = (spec.tags || []).filter(t => !tagsToRemove.has(t));

    onProgress?.((i + 1) / specs.length * 100);
  }

  recordOperation({
    type: 'tag',
    specs,
    previousState
  });
}

export async function batchUpdatePhase(
  specs: Spec[],
  phase: number,
  onProgress?: (progress: number) => void
): Promise<void> {
  const previousState = new Map<string, Partial<Spec>>();

  for (let i = 0; i < specs.length; i++) {
    const spec = specs[i];
    previousState.set(spec.id, { phase: spec.phase });

    spec.phase = phase;

    onProgress?.((i + 1) / specs.length * 100);
  }

  recordOperation({
    type: 'phase',
    specs,
    previousState
  });
}

export async function batchDelete(
  specs: Spec[],
  onProgress?: (progress: number) => void
): Promise<void> {
  const previousState = new Map<string, Partial<Spec>>();

  for (let i = 0; i < specs.length; i++) {
    const spec = specs[i];
    previousState.set(spec.id, { ...spec });

    // Mark for deletion or actually delete
    onProgress?.((i + 1) / specs.length * 100);
  }

  recordOperation({
    type: 'delete',
    specs,
    previousState
  });
}

export async function batchDuplicate(
  specs: Spec[],
  generateNewId: (id: string) => string,
  onProgress?: (progress: number) => void
): Promise<Spec[]> {
  const duplicates: Spec[] = [];

  for (let i = 0; i < specs.length; i++) {
    const spec = specs[i];
    const newId = generateNewId(spec.id);

    const duplicate: Spec = {
      ...spec,
      id: newId,
      title: `${spec.title} (Copy)`,
      createdAt: new Date(),
      updatedAt: new Date()
    };

    duplicates.push(duplicate);
    onProgress?.((i + 1) / specs.length * 100);
  }

  recordOperation({
    type: 'duplicate',
    specs: duplicates,
    previousState: new Map()
  });

  return duplicates;
}
```

---

## Testing Requirements

### Unit Tests

```typescript
import { render, fireEvent, screen } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import BatchActionBar from './BatchActionBar.svelte';
import {
  batchUpdateStatus,
  batchAddTags,
  batchRemoveTags,
  undoLastOperation
} from '$lib/stores/batch-operations';
import { createMockSpecs } from '$lib/test-utils/mock-data';

describe('BatchActionBar', () => {
  const mockSpecs = createMockSpecs(5);

  it('shows selection count', () => {
    render(BatchActionBar, {
      props: {
        selectedSpecs: mockSpecs.slice(0, 3),
        totalSpecs: mockSpecs.length,
        availableTags: []
      }
    });

    expect(screen.getByText('3 of 5 selected')).toBeInTheDocument();
  });

  it('hides when no selection', () => {
    render(BatchActionBar, {
      props: {
        selectedSpecs: [],
        totalSpecs: mockSpecs.length,
        availableTags: []
      }
    });

    expect(screen.queryByText(/selected/)).not.toBeInTheDocument();
  });

  it('shows status dropdown', async () => {
    render(BatchActionBar, {
      props: {
        selectedSpecs: mockSpecs,
        totalSpecs: mockSpecs.length,
        availableTags: []
      }
    });

    await fireEvent.click(screen.getByText('Status'));

    expect(screen.getByText('Planned')).toBeInTheDocument();
    expect(screen.getByText('In Progress')).toBeInTheDocument();
  });

  it('dispatches statusChange event', async () => {
    const { component } = render(BatchActionBar, {
      props: {
        selectedSpecs: mockSpecs,
        totalSpecs: mockSpecs.length,
        availableTags: []
      }
    });

    const handler = vi.fn();
    component.$on('statusChange', handler);

    await fireEvent.click(screen.getByText('Status'));
    await fireEvent.click(screen.getByText('In Progress'));

    expect(handler).toHaveBeenCalledWith(
      expect.objectContaining({
        detail: { specs: mockSpecs, status: 'in-progress' }
      })
    );
  });

  it('confirms before delete', async () => {
    const confirmSpy = vi.spyOn(window, 'confirm').mockReturnValue(false);

    const { component } = render(BatchActionBar, {
      props: {
        selectedSpecs: mockSpecs,
        totalSpecs: mockSpecs.length,
        availableTags: []
      }
    });

    const handler = vi.fn();
    component.$on('delete', handler);

    await fireEvent.click(screen.getByText('Delete'));

    expect(confirmSpy).toHaveBeenCalled();
    expect(handler).not.toHaveBeenCalled();

    confirmSpy.mockRestore();
  });

  it('handles select all toggle', async () => {
    const { component } = render(BatchActionBar, {
      props: {
        selectedSpecs: mockSpecs.slice(0, 3),
        totalSpecs: mockSpecs.length,
        availableTags: []
      }
    });

    const selectAllHandler = vi.fn();
    component.$on('selectAll', selectAllHandler);

    const checkbox = screen.getByLabelText('Select all');
    await fireEvent.click(checkbox);

    expect(selectAllHandler).toHaveBeenCalled();
  });

  it('clears selection on Escape', async () => {
    const { component } = render(BatchActionBar, {
      props: {
        selectedSpecs: mockSpecs,
        totalSpecs: mockSpecs.length,
        availableTags: []
      }
    });

    const clearHandler = vi.fn();
    component.$on('clear', clearHandler);

    await fireEvent.keyDown(window, { key: 'Escape' });

    expect(clearHandler).toHaveBeenCalled();
  });
});

describe('Batch Operations Store', () => {
  const specs = createMockSpecs(3).map(s => ({ ...s }));

  beforeEach(() => {
    // Reset specs to original state
    specs.forEach(s => {
      s.status = 'planned';
      s.tags = [];
      s.phase = 1;
    });
  });

  it('updates status for all specs', async () => {
    await batchUpdateStatus(specs, 'in-progress');

    expect(specs.every(s => s.status === 'in-progress')).toBe(true);
  });

  it('adds tags to all specs', async () => {
    await batchAddTags(specs, ['tag1', 'tag2']);

    expect(specs.every(s => s.tags?.includes('tag1'))).toBe(true);
    expect(specs.every(s => s.tags?.includes('tag2'))).toBe(true);
  });

  it('removes tags from all specs', async () => {
    specs.forEach(s => s.tags = ['tag1', 'tag2', 'tag3']);

    await batchRemoveTags(specs, ['tag2']);

    expect(specs.every(s => !s.tags?.includes('tag2'))).toBe(true);
    expect(specs.every(s => s.tags?.includes('tag1'))).toBe(true);
  });

  it('calls progress callback', async () => {
    const onProgress = vi.fn();

    await batchUpdateStatus(specs, 'tested', onProgress);

    expect(onProgress).toHaveBeenCalledTimes(specs.length);
    expect(onProgress).toHaveBeenLastCalledWith(100);
  });

  it('records operation for undo', async () => {
    await batchUpdateStatus(specs, 'implemented');

    const undoneOp = undoLastOperation();

    expect(undoneOp).not.toBeNull();
    expect(undoneOp?.type).toBe('status');
  });
});
```

---

## Related Specs

- Spec 231: Spec List Layout
- Spec 242: Spec Status Tracking
- Spec 247: Spec Export
- Spec 250: Component Tests
