# 220 - Spec Selector Component

**Phase:** 10 - Mission Panel UI
**Spec ID:** 220
**Status:** Planned
**Dependencies:** 216-mission-layout, 217-mission-state
**Estimated Context:** ~12% of Sonnet window

---

## Objective

Create a spec selector component that allows users to browse, search, and select specification files to include in a mission, with support for multi-select, dependency visualization, and quick preview.

---

## Acceptance Criteria

- [ ] Tree view of specs by phase/category
- [ ] Multi-select with checkboxes
- [ ] Search and filter functionality
- [ ] Dependency chain visualization
- [ ] Quick preview on hover/focus
- [ ] Keyboard navigation support
- [ ] Selected specs summary panel

---

## Implementation Details

### 1. Types (src/lib/types/spec-selector.ts)

```typescript
/**
 * Types for spec selection functionality.
 */

export interface SpecInfo {
  id: string;
  number: number;
  title: string;
  phase: number;
  phaseName: string;
  status: SpecStatus;
  path: string;
  dependencies: string[];
  dependents: string[];
  estimatedContext: number;
  tags: string[];
}

export type SpecStatus = 'planned' | 'in_progress' | 'complete' | 'blocked';

export interface SpecPhase {
  number: number;
  name: string;
  description: string;
  specs: SpecInfo[];
  isExpanded: boolean;
}

export interface SpecSelectorState {
  specs: Map<string, SpecInfo>;
  phases: SpecPhase[];
  selectedIds: Set<string>;
  expandedPhases: Set<number>;
  searchQuery: string;
  statusFilter: SpecStatus[];
  loading: boolean;
  error: string | null;
  previewSpecId: string | null;
}

export interface SpecSearchResult {
  spec: SpecInfo;
  matchType: 'title' | 'id' | 'tag' | 'content';
  matchedText: string;
  score: number;
}

export interface DependencyChain {
  specId: string;
  depth: number;
  isCircular: boolean;
  chain: string[];
}
```

### 2. Spec Selector Store (src/lib/stores/spec-selector-store.ts)

```typescript
import { writable, derived } from 'svelte/store';
import type {
  SpecSelectorState,
  SpecInfo,
  SpecPhase,
  SpecStatus,
  DependencyChain,
} from '$lib/types/spec-selector';
import { ipcRenderer } from '$lib/ipc';

function createSpecSelectorStore() {
  const initialState: SpecSelectorState = {
    specs: new Map(),
    phases: [],
    selectedIds: new Set(),
    expandedPhases: new Set([0, 1]), // First two phases expanded by default
    searchQuery: '',
    statusFilter: [],
    loading: false,
    error: null,
    previewSpecId: null,
  };

  const { subscribe, set, update } = writable<SpecSelectorState>(initialState);

  return {
    subscribe,

    async loadSpecs(): Promise<void> {
      update(s => ({ ...s, loading: true, error: null }));

      try {
        const specs: SpecInfo[] = await ipcRenderer.invoke('spec:list');
        const specsMap = new Map(specs.map(s => [s.id, s]));

        // Group by phase
        const phaseMap = new Map<number, SpecInfo[]>();
        specs.forEach(spec => {
          if (!phaseMap.has(spec.phase)) {
            phaseMap.set(spec.phase, []);
          }
          phaseMap.get(spec.phase)!.push(spec);
        });

        const phases: SpecPhase[] = Array.from(phaseMap.entries())
          .sort(([a], [b]) => a - b)
          .map(([number, phaseSpecs]) => ({
            number,
            name: phaseSpecs[0]?.phaseName || `Phase ${number}`,
            description: '',
            specs: phaseSpecs.sort((a, b) => a.number - b.number),
            isExpanded: number <= 1,
          }));

        update(s => ({
          ...s,
          specs: specsMap,
          phases,
          loading: false,
        }));
      } catch (error) {
        update(s => ({
          ...s,
          loading: false,
          error: error instanceof Error ? error.message : 'Failed to load specs',
        }));
      }
    },

    toggleSpec(specId: string) {
      update(state => {
        const selectedIds = new Set(state.selectedIds);
        if (selectedIds.has(specId)) {
          selectedIds.delete(specId);
        } else {
          selectedIds.add(specId);
        }
        return { ...state, selectedIds };
      });
    },

    selectSpec(specId: string) {
      update(state => {
        const selectedIds = new Set(state.selectedIds);
        selectedIds.add(specId);
        return { ...state, selectedIds };
      });
    },

    deselectSpec(specId: string) {
      update(state => {
        const selectedIds = new Set(state.selectedIds);
        selectedIds.delete(specId);
        return { ...state, selectedIds };
      });
    },

    selectAll() {
      update(state => {
        const selectedIds = new Set(state.specs.keys());
        return { ...state, selectedIds };
      });
    },

    deselectAll() {
      update(state => ({ ...state, selectedIds: new Set() }));
    },

    selectWithDependencies(specId: string) {
      update(state => {
        const selectedIds = new Set(state.selectedIds);
        const spec = state.specs.get(specId);

        if (!spec) return state;

        // Add spec and all its dependencies
        const toAdd = [specId];
        const visited = new Set<string>();

        while (toAdd.length > 0) {
          const id = toAdd.pop()!;
          if (visited.has(id)) continue;
          visited.add(id);

          selectedIds.add(id);
          const s = state.specs.get(id);
          if (s) {
            toAdd.push(...s.dependencies);
          }
        }

        return { ...state, selectedIds };
      });
    },

    togglePhase(phaseNumber: number) {
      update(state => {
        const expandedPhases = new Set(state.expandedPhases);
        if (expandedPhases.has(phaseNumber)) {
          expandedPhases.delete(phaseNumber);
        } else {
          expandedPhases.add(phaseNumber);
        }
        return { ...state, expandedPhases };
      });
    },

    setSearchQuery(query: string) {
      update(state => ({ ...state, searchQuery: query }));
    },

    setStatusFilter(statuses: SpecStatus[]) {
      update(state => ({ ...state, statusFilter: statuses }));
    },

    setPreviewSpec(specId: string | null) {
      update(state => ({ ...state, previewSpecId: specId }));
    },

    setSelectedIds(ids: string[]) {
      update(state => ({ ...state, selectedIds: new Set(ids) }));
    },
  };
}

export const specSelectorStore = createSpecSelectorStore();

// Derived store for filtered specs
export const filteredPhases = derived(specSelectorStore, $state => {
  const { phases, searchQuery, statusFilter } = $state;
  const query = searchQuery.toLowerCase();

  return phases.map(phase => {
    let specs = phase.specs;

    // Apply search filter
    if (query) {
      specs = specs.filter(s =>
        s.title.toLowerCase().includes(query) ||
        s.id.toLowerCase().includes(query) ||
        s.tags.some(t => t.toLowerCase().includes(query)) ||
        s.number.toString().includes(query)
      );
    }

    // Apply status filter
    if (statusFilter.length > 0) {
      specs = specs.filter(s => statusFilter.includes(s.status));
    }

    return { ...phase, specs };
  }).filter(phase => phase.specs.length > 0);
});

// Selected specs with full info
export const selectedSpecs = derived(specSelectorStore, $state =>
  Array.from($state.selectedIds)
    .map(id => $state.specs.get(id))
    .filter((s): s is SpecInfo => s !== undefined)
);

// Total estimated context for selected specs
export const totalEstimatedContext = derived(selectedSpecs, $specs =>
  $specs.reduce((sum, s) => sum + s.estimatedContext, 0)
);

// Dependency chain analyzer
export function analyzeDependencyChain(
  specs: Map<string, SpecInfo>,
  specId: string
): DependencyChain {
  const chain: string[] = [];
  const visited = new Set<string>();
  let isCircular = false;
  let depth = 0;

  function traverse(id: string, currentDepth: number) {
    if (visited.has(id)) {
      isCircular = true;
      return;
    }

    visited.add(id);
    chain.push(id);
    depth = Math.max(depth, currentDepth);

    const spec = specs.get(id);
    if (spec) {
      for (const depId of spec.dependencies) {
        traverse(depId, currentDepth + 1);
      }
    }
  }

  traverse(specId, 0);

  return { specId, depth, isCircular, chain };
}
```

### 3. Spec Selector Component (src/lib/components/mission/SpecSelector.svelte)

```svelte
<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import {
    specSelectorStore,
    filteredPhases,
    selectedSpecs,
    totalEstimatedContext,
  } from '$lib/stores/spec-selector-store';
  import type { SpecInfo, SpecStatus } from '$lib/types/spec-selector';
  import SpecTreeItem from './SpecTreeItem.svelte';
  import SpecPreview from './SpecPreview.svelte';

  export let selectedIds: string[] = [];
  export let maxContext = 50; // Max percentage of context to use

  const dispatch = createEventDispatcher<{
    change: string[];
  }>();

  let searchInput: HTMLInputElement;

  // Sync external selection
  $: if (selectedIds.length !== $specSelectorStore.selectedIds.size) {
    specSelectorStore.setSelectedIds(selectedIds);
  }

  // Emit changes
  $: dispatch('change', Array.from($specSelectorStore.selectedIds));

  const statusOptions: { value: SpecStatus; label: string }[] = [
    { value: 'planned', label: 'Planned' },
    { value: 'in_progress', label: 'In Progress' },
    { value: 'complete', label: 'Complete' },
    { value: 'blocked', label: 'Blocked' },
  ];

  function handleKeyDown(event: KeyboardEvent) {
    // Focus search: Cmd/Ctrl + F
    if ((event.metaKey || event.ctrlKey) && event.key === 'f') {
      event.preventDefault();
      searchInput?.focus();
    }

    // Clear selection: Escape
    if (event.key === 'Escape') {
      specSelectorStore.deselectAll();
    }
  }

  onMount(() => {
    specSelectorStore.loadSpecs();
  });
</script>

<svelte:window on:keydown={handleKeyDown} />

<div class="spec-selector">
  <!-- Search and Filter Bar -->
  <div class="spec-selector__header">
    <div class="spec-selector__search">
      <svg class="search-icon" width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
        <path d="M11.742 10.344a6.5 6.5 0 1 0-1.397 1.398h-.001c.03.04.062.078.098.115l3.85 3.85a1 1 0 0 0 1.415-1.414l-3.85-3.85a1.007 1.007 0 0 0-.115-.1zM12 6.5a5.5 5.5 0 1 1-11 0 5.5 5.5 0 0 1 11 0z"/>
      </svg>
      <input
        bind:this={searchInput}
        type="text"
        class="spec-selector__search-input"
        placeholder="Search specs..."
        value={$specSelectorStore.searchQuery}
        on:input={(e) => specSelectorStore.setSearchQuery(e.currentTarget.value)}
      />
      {#if $specSelectorStore.searchQuery}
        <button
          class="search-clear"
          on:click={() => specSelectorStore.setSearchQuery('')}
          aria-label="Clear search"
        >
          <svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor">
            <path d="M4.293 4.293a1 1 0 011.414 0L7 5.586l1.293-1.293a1 1 0 111.414 1.414L8.414 7l1.293 1.293a1 1 0 01-1.414 1.414L7 8.414l-1.293 1.293a1 1 0 01-1.414-1.414L5.586 7 4.293 5.707a1 1 0 010-1.414z"/>
          </svg>
        </button>
      {/if}
    </div>

    <div class="spec-selector__filters">
      {#each statusOptions as option}
        <label class="filter-chip">
          <input
            type="checkbox"
            checked={$specSelectorStore.statusFilter.includes(option.value)}
            on:change={(e) => {
              const current = $specSelectorStore.statusFilter;
              if (e.currentTarget.checked) {
                specSelectorStore.setStatusFilter([...current, option.value]);
              } else {
                specSelectorStore.setStatusFilter(current.filter(s => s !== option.value));
              }
            }}
          />
          <span class="filter-chip__label">{option.label}</span>
        </label>
      {/each}
    </div>
  </div>

  <!-- Spec Tree -->
  <div class="spec-selector__tree" role="tree" aria-label="Specifications">
    {#if $specSelectorStore.loading}
      <div class="spec-selector__loading">Loading specs...</div>
    {:else if $specSelectorStore.error}
      <div class="spec-selector__error">{$specSelectorStore.error}</div>
    {:else if $filteredPhases.length === 0}
      <div class="spec-selector__empty">No specs found</div>
    {:else}
      {#each $filteredPhases as phase}
        <div class="phase-group" role="group" aria-label={phase.name}>
          <button
            class="phase-header"
            class:phase-header--expanded={$specSelectorStore.expandedPhases.has(phase.number)}
            on:click={() => specSelectorStore.togglePhase(phase.number)}
            aria-expanded={$specSelectorStore.expandedPhases.has(phase.number)}
          >
            <svg class="phase-header__chevron" width="12" height="12" viewBox="0 0 12 12">
              <path
                fill="currentColor"
                d={$specSelectorStore.expandedPhases.has(phase.number)
                  ? 'M2 4l4 4 4-4'
                  : 'M4 2l4 4-4 4'}
              />
            </svg>
            <span class="phase-header__number">Phase {phase.number}</span>
            <span class="phase-header__name">{phase.name}</span>
            <span class="phase-header__count">{phase.specs.length}</span>
          </button>

          {#if $specSelectorStore.expandedPhases.has(phase.number)}
            <div class="phase-specs">
              {#each phase.specs as spec}
                <SpecTreeItem
                  {spec}
                  selected={$specSelectorStore.selectedIds.has(spec.id)}
                  previewing={$specSelectorStore.previewSpecId === spec.id}
                  on:toggle={() => specSelectorStore.toggleSpec(spec.id)}
                  on:selectWithDeps={() => specSelectorStore.selectWithDependencies(spec.id)}
                  on:preview={() => specSelectorStore.setPreviewSpec(spec.id)}
                  on:previewEnd={() => specSelectorStore.setPreviewSpec(null)}
                />
              {/each}
            </div>
          {/if}
        </div>
      {/each}
    {/if}
  </div>

  <!-- Selection Summary -->
  <div class="spec-selector__summary">
    <div class="summary-stats">
      <span class="summary-stat">
        <strong>{$selectedSpecs.length}</strong> specs selected
      </span>
      <span class="summary-stat">
        <strong>{$totalEstimatedContext.toFixed(0)}%</strong> context
      </span>
    </div>

    <div class="summary-actions">
      <button
        class="summary-action"
        on:click={() => specSelectorStore.selectAll()}
        disabled={$selectedSpecs.length === $specSelectorStore.specs.size}
      >
        Select All
      </button>
      <button
        class="summary-action"
        on:click={() => specSelectorStore.deselectAll()}
        disabled={$selectedSpecs.length === 0}
      >
        Clear
      </button>
    </div>

    {#if $totalEstimatedContext > maxContext}
      <div class="summary-warning">
        Warning: Selected specs may exceed recommended context limit
      </div>
    {/if}
  </div>

  <!-- Preview Panel -->
  {#if $specSelectorStore.previewSpecId}
    <div class="spec-selector__preview">
      <SpecPreview specId={$specSelectorStore.previewSpecId} />
    </div>
  {/if}
</div>

<style>
  .spec-selector {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 300px;
  }

  .spec-selector__header {
    padding: 12px;
    border-bottom: 1px solid var(--color-border);
  }

  .spec-selector__search {
    position: relative;
    display: flex;
    align-items: center;
    margin-bottom: 12px;
  }

  .search-icon {
    position: absolute;
    left: 12px;
    color: var(--color-text-muted);
    pointer-events: none;
  }

  .spec-selector__search-input {
    width: 100%;
    padding: 8px 32px 8px 36px;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
    font-size: 14px;
  }

  .spec-selector__search-input:focus {
    outline: none;
    border-color: var(--color-primary);
  }

  .search-clear {
    position: absolute;
    right: 8px;
    padding: 4px;
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
  }

  .spec-selector__filters {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }

  .filter-chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 4px 8px;
    background: var(--color-bg-hover);
    border-radius: 12px;
    font-size: 12px;
    cursor: pointer;
  }

  .filter-chip input {
    width: 14px;
    height: 14px;
  }

  .spec-selector__tree {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
  }

  .spec-selector__loading,
  .spec-selector__error,
  .spec-selector__empty {
    padding: 32px;
    text-align: center;
    color: var(--color-text-muted);
  }

  .spec-selector__error {
    color: var(--color-error);
  }

  .phase-group {
    margin-bottom: 4px;
  }

  .phase-header {
    display: flex;
    align-items: center;
    width: 100%;
    padding: 8px 12px;
    border: none;
    background: var(--color-bg-secondary);
    cursor: pointer;
    border-radius: 6px;
    transition: background-color 0.15s ease;
  }

  .phase-header:hover {
    background: var(--color-bg-hover);
  }

  .phase-header__chevron {
    margin-right: 8px;
    color: var(--color-text-muted);
    transition: transform 0.15s ease;
  }

  .phase-header--expanded .phase-header__chevron {
    transform: rotate(90deg);
  }

  .phase-header__number {
    font-weight: 600;
    color: var(--color-primary);
    margin-right: 8px;
  }

  .phase-header__name {
    flex: 1;
    text-align: left;
    color: var(--color-text-primary);
  }

  .phase-header__count {
    font-size: 12px;
    color: var(--color-text-muted);
    background: var(--color-bg-primary);
    padding: 2px 8px;
    border-radius: 10px;
  }

  .phase-specs {
    padding-left: 20px;
  }

  .spec-selector__summary {
    padding: 12px;
    border-top: 1px solid var(--color-border);
    background: var(--color-bg-secondary);
  }

  .summary-stats {
    display: flex;
    gap: 16px;
    margin-bottom: 8px;
  }

  .summary-stat {
    font-size: 13px;
    color: var(--color-text-secondary);
  }

  .summary-actions {
    display: flex;
    gap: 8px;
  }

  .summary-action {
    padding: 6px 12px;
    border: none;
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
    font-size: 12px;
    border-radius: 4px;
    cursor: pointer;
  }

  .summary-action:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .summary-warning {
    margin-top: 8px;
    padding: 8px;
    background: rgba(255, 152, 0, 0.1);
    border-radius: 4px;
    font-size: 12px;
    color: var(--color-warning);
  }

  .spec-selector__preview {
    position: absolute;
    right: 0;
    top: 0;
    bottom: 0;
    width: 300px;
    background: var(--color-bg-primary);
    border-left: 1px solid var(--color-border);
    box-shadow: -4px 0 12px rgba(0, 0, 0, 0.1);
    overflow-y: auto;
  }
</style>
```

### 4. Spec Tree Item (src/lib/components/mission/SpecTreeItem.svelte)

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { SpecInfo } from '$lib/types/spec-selector';

  export let spec: SpecInfo;
  export let selected = false;
  export let previewing = false;

  const dispatch = createEventDispatcher<{
    toggle: void;
    selectWithDeps: void;
    preview: void;
    previewEnd: void;
  }>();

  const statusColors: Record<string, string> = {
    planned: 'var(--color-text-muted)',
    in_progress: 'var(--color-primary)',
    complete: 'var(--color-success)',
    blocked: 'var(--color-error)',
  };

  function handleContextMenu(event: MouseEvent) {
    event.preventDefault();
    dispatch('selectWithDeps');
  }
</script>

<div
  class="spec-item"
  class:spec-item--selected={selected}
  class:spec-item--previewing={previewing}
  role="treeitem"
  aria-selected={selected}
  on:mouseenter={() => dispatch('preview')}
  on:mouseleave={() => dispatch('previewEnd')}
  on:contextmenu={handleContextMenu}
>
  <label class="spec-item__checkbox">
    <input
      type="checkbox"
      checked={selected}
      on:change={() => dispatch('toggle')}
    />
  </label>

  <span class="spec-item__number">{spec.number}</span>

  <span class="spec-item__title">{spec.title}</span>

  <span
    class="spec-item__status"
    style="color: {statusColors[spec.status]}"
    title={spec.status}
  >
    {#if spec.status === 'complete'}
      <svg width="12" height="12" viewBox="0 0 12 12" fill="currentColor">
        <path d="M10.28 2.28a.75.75 0 010 1.06l-5.25 5.25a.75.75 0 01-1.06 0L1.72 6.34a.75.75 0 011.06-1.06l1.72 1.72 4.72-4.72a.75.75 0 011.06 0z"/>
      </svg>
    {:else if spec.status === 'in_progress'}
      <svg width="12" height="12" viewBox="0 0 12 12" fill="currentColor">
        <circle cx="6" cy="6" r="3"/>
      </svg>
    {:else if spec.status === 'blocked'}
      <svg width="12" height="12" viewBox="0 0 12 12" fill="currentColor">
        <path d="M6 0a6 6 0 100 12A6 6 0 006 0zm0 10.5a4.5 4.5 0 110-9 4.5 4.5 0 010 9zm0-7.5a.75.75 0 00-.75.75v3a.75.75 0 001.5 0v-3A.75.75 0 006 3zm0 6a.75.75 0 100-1.5.75.75 0 000 1.5z"/>
      </svg>
    {/if}
  </span>

  {#if spec.dependencies.length > 0}
    <span class="spec-item__deps" title="{spec.dependencies.length} dependencies">
      <svg width="10" height="10" viewBox="0 0 10 10" fill="currentColor">
        <path d="M5 0v3M5 7v3M0 5h3M7 5h3"/>
      </svg>
      {spec.dependencies.length}
    </span>
  {/if}

  <span class="spec-item__context" title="Estimated context usage">
    {spec.estimatedContext}%
  </span>
</div>

<style>
  .spec-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 8px;
    border-radius: 4px;
    cursor: pointer;
    transition: background-color 0.1s ease;
  }

  .spec-item:hover {
    background: var(--color-bg-hover);
  }

  .spec-item--selected {
    background: var(--color-bg-active);
  }

  .spec-item--previewing {
    outline: 2px solid var(--color-primary);
    outline-offset: -2px;
  }

  .spec-item__checkbox input {
    width: 14px;
    height: 14px;
    cursor: pointer;
  }

  .spec-item__number {
    min-width: 32px;
    font-family: monospace;
    font-size: 12px;
    color: var(--color-primary);
  }

  .spec-item__title {
    flex: 1;
    font-size: 13px;
    color: var(--color-text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .spec-item__status {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 16px;
  }

  .spec-item__deps {
    display: flex;
    align-items: center;
    gap: 2px;
    font-size: 11px;
    color: var(--color-text-muted);
  }

  .spec-item__context {
    font-size: 11px;
    color: var(--color-text-muted);
    min-width: 28px;
    text-align: right;
  }
</style>
```

---

## Testing Requirements

1. Specs load and display correctly
2. Search filters specs properly
3. Status filters work
4. Multi-select works with checkboxes
5. Select with dependencies works
6. Context estimation updates correctly
7. Keyboard navigation functions

### Test File (src/lib/components/mission/__tests__/SpecSelector.test.ts)

```typescript
import { render, fireEvent, screen } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import SpecSelector from '../SpecSelector.svelte';
import { specSelectorStore } from '$lib/stores/spec-selector-store';

vi.mock('$lib/ipc', () => ({
  ipcRenderer: {
    invoke: vi.fn().mockResolvedValue([
      { id: '001', number: 1, title: 'Project Structure', phase: 0, phaseName: 'Setup', status: 'complete', dependencies: [], estimatedContext: 5, tags: [] },
      { id: '002', number: 2, title: 'Rust Workspace', phase: 0, phaseName: 'Setup', status: 'in_progress', dependencies: ['001'], estimatedContext: 8, tags: [] },
    ]),
  },
}));

describe('SpecSelector', () => {
  beforeEach(async () => {
    await specSelectorStore.loadSpecs();
  });

  it('renders loaded specs', async () => {
    render(SpecSelector);

    expect(screen.getByText('Project Structure')).toBeInTheDocument();
    expect(screen.getByText('Rust Workspace')).toBeInTheDocument();
  });

  it('filters specs by search', async () => {
    render(SpecSelector);

    const searchInput = screen.getByPlaceholderText('Search specs...');
    await fireEvent.input(searchInput, { target: { value: 'Rust' } });

    expect(screen.queryByText('Project Structure')).not.toBeInTheDocument();
    expect(screen.getByText('Rust Workspace')).toBeInTheDocument();
  });

  it('toggles spec selection', async () => {
    render(SpecSelector);

    const checkbox = screen.getAllByRole('checkbox')[0];
    await fireEvent.click(checkbox);

    expect(checkbox).toBeChecked();
  });

  it('emits change event with selected IDs', async () => {
    const { component } = render(SpecSelector);
    const handler = vi.fn();
    component.$on('change', handler);

    const checkbox = screen.getAllByRole('checkbox')[0];
    await fireEvent.click(checkbox);

    expect(handler).toHaveBeenCalled();
  });
});
```

---

## Related Specs

- Depends on: [216-mission-layout.md](216-mission-layout.md)
- Depends on: [217-mission-state.md](217-mission-state.md)
- Next: [221-backend-selector.md](221-backend-selector.md)
- Used by: [218-mission-creation.md](218-mission-creation.md)
