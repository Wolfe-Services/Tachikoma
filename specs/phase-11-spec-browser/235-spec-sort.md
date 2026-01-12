# Spec 235: Spec Sorting

## Phase
11 - Spec Browser UI

## Spec ID
235

## Status
Planned

## Dependencies
- Spec 231 (Spec List Layout)
- Spec 233 (Spec Filter System)
- Spec 234 (Spec Search)

## Estimated Context
~8%

---

## Objective

Implement a flexible sorting system for the Spec Browser that allows users to sort specs by multiple criteria with configurable sort direction. Support saved sort preferences, multi-column sorting, and drag-and-drop custom ordering.

---

## Acceptance Criteria

- [ ] Sort by ID, title, status, phase, dates
- [ ] Toggle ascending/descending direction
- [ ] Multi-column sort with priority
- [ ] Visual indicator of active sort
- [ ] Persist sort preference
- [ ] Custom drag-and-drop ordering
- [ ] Reset to default sort
- [ ] Keyboard accessible controls

---

## Implementation Details

### SpecSortControls.svelte

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { flip } from 'svelte/animate';
  import { dndzone } from 'svelte-dnd-action';
  import type { SortConfig, SortField, SortDirection } from '$lib/types/spec';
  import Button from '$lib/components/Button.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import Popover from '$lib/components/Popover.svelte';

  export let sortConfig: SortConfig = {
    fields: [{ field: 'id', direction: 'asc' }]
  };
  export let allowMultiSort = true;
  export let allowCustomOrder = false;

  const dispatch = createEventDispatcher<{
    change: SortConfig;
    customOrder: string[];
  }>();

  const sortOptions: { field: SortField; label: string; icon: string }[] = [
    { field: 'id', label: 'ID', icon: 'hash' },
    { field: 'title', label: 'Title', icon: 'type' },
    { field: 'status', label: 'Status', icon: 'activity' },
    { field: 'phase', label: 'Phase', icon: 'layers' },
    { field: 'createdAt', label: 'Created', icon: 'calendar' },
    { field: 'updatedAt', label: 'Updated', icon: 'clock' },
    { field: 'dependencies', label: 'Dependencies', icon: 'git-branch' },
  ];

  let showSortMenu = false;
  let multiSortItems = [...sortConfig.fields];

  $: primarySort = sortConfig.fields[0];
  $: activeSortFields = new Set(sortConfig.fields.map(f => f.field));

  function toggleSort(field: SortField) {
    const existingIndex = sortConfig.fields.findIndex(f => f.field === field);

    if (existingIndex === -1) {
      // Add new sort field
      if (allowMultiSort) {
        sortConfig = {
          ...sortConfig,
          fields: [...sortConfig.fields, { field, direction: 'asc' }]
        };
      } else {
        sortConfig = {
          ...sortConfig,
          fields: [{ field, direction: 'asc' }]
        };
      }
    } else if (sortConfig.fields[existingIndex].direction === 'asc') {
      // Toggle to descending
      const newFields = [...sortConfig.fields];
      newFields[existingIndex] = { field, direction: 'desc' };
      sortConfig = { ...sortConfig, fields: newFields };
    } else {
      // Remove sort field
      if (sortConfig.fields.length > 1) {
        sortConfig = {
          ...sortConfig,
          fields: sortConfig.fields.filter(f => f.field !== field)
        };
      } else {
        // If it's the only sort, toggle back to asc
        sortConfig = {
          ...sortConfig,
          fields: [{ field, direction: 'asc' }]
        };
      }
    }

    dispatch('change', sortConfig);
    saveSortPreference();
  }

  function getSortDirection(field: SortField): SortDirection | null {
    const sortField = sortConfig.fields.find(f => f.field === field);
    return sortField?.direction ?? null;
  }

  function getSortPriority(field: SortField): number | null {
    if (!allowMultiSort || sortConfig.fields.length <= 1) return null;
    const index = sortConfig.fields.findIndex(f => f.field === field);
    return index >= 0 ? index + 1 : null;
  }

  function handleDndConsider(e: CustomEvent) {
    multiSortItems = e.detail.items;
  }

  function handleDndFinalize(e: CustomEvent) {
    multiSortItems = e.detail.items;
    sortConfig = { ...sortConfig, fields: multiSortItems };
    dispatch('change', sortConfig);
    saveSortPreference();
  }

  function removeSort(field: SortField) {
    if (sortConfig.fields.length <= 1) return;

    sortConfig = {
      ...sortConfig,
      fields: sortConfig.fields.filter(f => f.field !== field)
    };
    multiSortItems = [...sortConfig.fields];
    dispatch('change', sortConfig);
    saveSortPreference();
  }

  function resetSort() {
    sortConfig = { fields: [{ field: 'id', direction: 'asc' }] };
    multiSortItems = [...sortConfig.fields];
    dispatch('change', sortConfig);
    saveSortPreference();
  }

  function saveSortPreference() {
    localStorage.setItem('spec-sort-config', JSON.stringify(sortConfig));
  }

  function getFieldLabel(field: SortField): string {
    return sortOptions.find(o => o.field === field)?.label ?? field;
  }

  function getFieldIcon(field: SortField): string {
    return sortOptions.find(o => o.field === field)?.icon ?? 'arrow-up-down';
  }
</script>

<div class="spec-sort">
  <!-- Quick sort buttons -->
  <div class="spec-sort__quick">
    {#each sortOptions.slice(0, 4) as option}
      {@const direction = getSortDirection(option.field)}
      {@const priority = getSortPriority(option.field)}
      <button
        class="spec-sort__btn"
        class:spec-sort__btn--active={direction !== null}
        on:click={() => toggleSort(option.field)}
        aria-label="Sort by {option.label}"
        aria-pressed={direction !== null}
      >
        <Icon name={option.icon} size={14} />
        <span>{option.label}</span>
        {#if direction}
          <Icon
            name={direction === 'asc' ? 'arrow-up' : 'arrow-down'}
            size={12}
          />
        {/if}
        {#if priority}
          <span class="spec-sort__priority">{priority}</span>
        {/if}
      </button>
    {/each}
  </div>

  <!-- Sort menu for all options -->
  <Popover bind:open={showSortMenu} placement="bottom-end">
    <Button slot="trigger" variant="ghost" size="sm">
      <Icon name="arrow-up-down" size={14} />
      Sort
      {#if sortConfig.fields.length > 1}
        <span class="spec-sort__count">{sortConfig.fields.length}</span>
      {/if}
    </Button>

    <div class="spec-sort__menu">
      <div class="spec-sort__menu-header">
        <span>Sort Options</span>
        {#if sortConfig.fields.length > 1 || primarySort.field !== 'id'}
          <button class="spec-sort__reset" on:click={resetSort}>
            Reset
          </button>
        {/if}
      </div>

      {#if allowMultiSort && sortConfig.fields.length > 0}
        <div class="spec-sort__active">
          <span class="spec-sort__active-label">Active sorts (drag to reorder)</span>
          <div
            class="spec-sort__active-list"
            use:dndzone={{ items: multiSortItems, flipDurationMs: 150 }}
            on:consider={handleDndConsider}
            on:finalize={handleDndFinalize}
          >
            {#each multiSortItems as sortItem (sortItem.field)}
              <div class="spec-sort__active-item" animate:flip={{ duration: 150 }}>
                <Icon name="grip-vertical" size={12} class="spec-sort__grip" />
                <Icon name={getFieldIcon(sortItem.field)} size={14} />
                <span>{getFieldLabel(sortItem.field)}</span>
                <button
                  class="spec-sort__direction"
                  on:click={() => toggleSort(sortItem.field)}
                  aria-label="Toggle direction"
                >
                  <Icon
                    name={sortItem.direction === 'asc' ? 'arrow-up' : 'arrow-down'}
                    size={12}
                  />
                </button>
                {#if sortConfig.fields.length > 1}
                  <button
                    class="spec-sort__remove"
                    on:click={() => removeSort(sortItem.field)}
                    aria-label="Remove sort"
                  >
                    <Icon name="x" size={12} />
                  </button>
                {/if}
              </div>
            {/each}
          </div>
        </div>
      {/if}

      <div class="spec-sort__options">
        <span class="spec-sort__options-label">Add sort by</span>
        {#each sortOptions as option}
          {#if !activeSortFields.has(option.field)}
            <button
              class="spec-sort__option"
              on:click={() => toggleSort(option.field)}
            >
              <Icon name={option.icon} size={14} />
              <span>{option.label}</span>
            </button>
          {/if}
        {/each}
      </div>

      {#if allowCustomOrder}
        <div class="spec-sort__custom">
          <Button
            variant="ghost"
            size="sm"
            on:click={() => dispatch('customOrder', [])}
          >
            <Icon name="hand" size={14} />
            Enable custom drag order
          </Button>
        </div>
      {/if}
    </div>
  </Popover>
</div>

<style>
  .spec-sort {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .spec-sort__quick {
    display: flex;
    gap: 4px;
  }

  .spec-sort__btn {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 6px 10px;
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--color-text-secondary);
    background: none;
    border: 1px solid transparent;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .spec-sort__btn:hover {
    background: var(--color-hover);
    color: var(--color-text-primary);
  }

  .spec-sort__btn--active {
    background: var(--color-primary-subtle);
    color: var(--color-primary);
    border-color: var(--color-primary-alpha);
  }

  .spec-sort__priority {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 14px;
    height: 14px;
    font-size: 0.625rem;
    font-weight: 700;
    background: var(--color-primary);
    color: white;
    border-radius: 50%;
  }

  .spec-sort__count {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 16px;
    height: 16px;
    padding: 0 4px;
    font-size: 0.625rem;
    font-weight: 600;
    background: var(--color-primary);
    color: white;
    border-radius: 8px;
  }

  .spec-sort__menu {
    width: 280px;
    padding: 8px 0;
  }

  .spec-sort__menu-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 12px;
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--color-text-secondary);
    text-transform: uppercase;
  }

  .spec-sort__reset {
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--color-text-tertiary);
    background: none;
    border: none;
    cursor: pointer;
    text-transform: none;
  }

  .spec-sort__reset:hover {
    color: var(--color-primary);
  }

  .spec-sort__active {
    padding: 8px 12px;
    border-bottom: 1px solid var(--color-border);
  }

  .spec-sort__active-label,
  .spec-sort__options-label {
    display: block;
    font-size: 0.625rem;
    font-weight: 600;
    color: var(--color-text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: 8px;
  }

  .spec-sort__active-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .spec-sort__active-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 8px;
    font-size: 0.875rem;
    background: var(--color-surface-elevated);
    border-radius: 4px;
    cursor: grab;
  }

  .spec-sort__active-item:active {
    cursor: grabbing;
  }

  .spec-sort :global(.spec-sort__grip) {
    color: var(--color-text-tertiary);
  }

  .spec-sort__direction,
  .spec-sort__remove {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    padding: 0;
    background: none;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    color: var(--color-text-tertiary);
    margin-left: auto;
  }

  .spec-sort__direction:hover,
  .spec-sort__remove:hover {
    background: var(--color-hover);
    color: var(--color-text-primary);
  }

  .spec-sort__options {
    padding: 8px 12px;
  }

  .spec-sort__option {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 8px;
    font-size: 0.875rem;
    text-align: left;
    background: none;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    color: var(--color-text-primary);
  }

  .spec-sort__option:hover {
    background: var(--color-hover);
  }

  .spec-sort__custom {
    padding: 8px 12px;
    border-top: 1px solid var(--color-border);
  }

  /* Responsive */
  @media (max-width: 768px) {
    .spec-sort__quick {
      display: none;
    }
  }
</style>
```

### Sort Utilities

```typescript
// utils/sort.ts
import type { Spec, SortConfig, SortField, SortDirection } from '$lib/types/spec';

export function sortSpecs(specs: Spec[], config: SortConfig): Spec[] {
  return [...specs].sort((a, b) => {
    for (const { field, direction } of config.fields) {
      const comparison = compareByField(a, b, field);
      if (comparison !== 0) {
        return direction === 'asc' ? comparison : -comparison;
      }
    }
    return 0;
  });
}

function compareByField(a: Spec, b: Spec, field: SortField): number {
  switch (field) {
    case 'id':
      return compareNumericId(a.id, b.id);

    case 'title':
      return a.title.localeCompare(b.title);

    case 'status':
      return compareStatus(a.status, b.status);

    case 'phase':
      return a.phase - b.phase;

    case 'createdAt':
      return new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime();

    case 'updatedAt':
      return new Date(a.updatedAt).getTime() - new Date(b.updatedAt).getTime();

    case 'dependencies':
      return (a.dependencies?.length ?? 0) - (b.dependencies?.length ?? 0);

    default:
      return 0;
  }
}

function compareNumericId(a: string, b: string): number {
  // Extract numeric part from ID (e.g., "231" from "spec-231")
  const numA = parseInt(a.replace(/\D/g, ''), 10) || 0;
  const numB = parseInt(b.replace(/\D/g, ''), 10) || 0;
  return numA - numB;
}

const STATUS_ORDER: Record<string, number> = {
  'planned': 0,
  'in-progress': 1,
  'implemented': 2,
  'tested': 3,
  'deprecated': 4,
};

function compareStatus(a: string, b: string): number {
  return (STATUS_ORDER[a] ?? 99) - (STATUS_ORDER[b] ?? 99);
}

export function loadSortConfig(): SortConfig {
  try {
    const stored = localStorage.getItem('spec-sort-config');
    if (stored) {
      return JSON.parse(stored);
    }
  } catch {
    // Ignore parse errors
  }
  return { fields: [{ field: 'id', direction: 'asc' }] };
}

export function saveSortConfig(config: SortConfig): void {
  localStorage.setItem('spec-sort-config', JSON.stringify(config));
}

// For custom ordering
export function applyCustomOrder(specs: Spec[], order: string[]): Spec[] {
  const orderMap = new Map(order.map((id, index) => [id, index]));

  return [...specs].sort((a, b) => {
    const orderA = orderMap.get(a.id) ?? Infinity;
    const orderB = orderMap.get(b.id) ?? Infinity;
    return orderA - orderB;
  });
}
```

### Sort Types

```typescript
// types/spec.ts additions
export type SortField =
  | 'id'
  | 'title'
  | 'status'
  | 'phase'
  | 'createdAt'
  | 'updatedAt'
  | 'dependencies';

export type SortDirection = 'asc' | 'desc';

export interface SortFieldConfig {
  field: SortField;
  direction: SortDirection;
}

export interface SortConfig {
  fields: SortFieldConfig[];
}
```

---

## Testing Requirements

### Unit Tests

```typescript
import { render, fireEvent, screen } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import SpecSortControls from './SpecSortControls.svelte';
import { sortSpecs, compareByField } from '$lib/utils/sort';
import { createMockSpecs } from '$lib/test-utils/mock-data';

describe('SpecSortControls', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('renders quick sort buttons', () => {
    render(SpecSortControls);

    expect(screen.getByText('ID')).toBeInTheDocument();
    expect(screen.getByText('Title')).toBeInTheDocument();
    expect(screen.getByText('Status')).toBeInTheDocument();
    expect(screen.getByText('Phase')).toBeInTheDocument();
  });

  it('shows active state for current sort', () => {
    render(SpecSortControls, {
      props: {
        sortConfig: { fields: [{ field: 'title', direction: 'asc' }] }
      }
    });

    const titleBtn = screen.getByText('Title').closest('button');
    expect(titleBtn).toHaveClass('spec-sort__btn--active');
  });

  it('toggles sort direction on click', async () => {
    const { component } = render(SpecSortControls, {
      props: {
        sortConfig: { fields: [{ field: 'id', direction: 'asc' }] }
      }
    });

    const changeHandler = vi.fn();
    component.$on('change', changeHandler);

    await fireEvent.click(screen.getByText('ID').closest('button')!);

    expect(changeHandler).toHaveBeenCalledWith(
      expect.objectContaining({
        detail: { fields: [{ field: 'id', direction: 'desc' }] }
      })
    );
  });

  it('adds new sort field', async () => {
    const { component } = render(SpecSortControls, {
      props: {
        sortConfig: { fields: [{ field: 'id', direction: 'asc' }] },
        allowMultiSort: true
      }
    });

    const changeHandler = vi.fn();
    component.$on('change', changeHandler);

    await fireEvent.click(screen.getByText('Title').closest('button')!);

    expect(changeHandler).toHaveBeenCalledWith(
      expect.objectContaining({
        detail: {
          fields: [
            { field: 'id', direction: 'asc' },
            { field: 'title', direction: 'asc' }
          ]
        }
      })
    );
  });

  it('shows sort count badge for multi-sort', () => {
    render(SpecSortControls, {
      props: {
        sortConfig: {
          fields: [
            { field: 'phase', direction: 'asc' },
            { field: 'id', direction: 'asc' }
          ]
        },
        allowMultiSort: true
      }
    });

    expect(screen.getByText('2')).toBeInTheDocument();
  });
});

describe('sortSpecs', () => {
  const specs = createMockSpecs(10);

  it('sorts by ID ascending', () => {
    const sorted = sortSpecs(specs, {
      fields: [{ field: 'id', direction: 'asc' }]
    });

    for (let i = 1; i < sorted.length; i++) {
      const prevId = parseInt(sorted[i - 1].id.replace(/\D/g, ''), 10);
      const currId = parseInt(sorted[i].id.replace(/\D/g, ''), 10);
      expect(prevId).toBeLessThanOrEqual(currId);
    }
  });

  it('sorts by title descending', () => {
    const sorted = sortSpecs(specs, {
      fields: [{ field: 'title', direction: 'desc' }]
    });

    for (let i = 1; i < sorted.length; i++) {
      expect(sorted[i - 1].title.localeCompare(sorted[i].title)).toBeGreaterThanOrEqual(0);
    }
  });

  it('handles multi-field sorting', () => {
    const sorted = sortSpecs(specs, {
      fields: [
        { field: 'phase', direction: 'asc' },
        { field: 'id', direction: 'asc' }
      ]
    });

    // Within same phase, IDs should be ascending
    for (let i = 1; i < sorted.length; i++) {
      if (sorted[i].phase === sorted[i - 1].phase) {
        const prevId = parseInt(sorted[i - 1].id.replace(/\D/g, ''), 10);
        const currId = parseInt(sorted[i].id.replace(/\D/g, ''), 10);
        expect(prevId).toBeLessThanOrEqual(currId);
      }
    }
  });

  it('sorts by status using custom order', () => {
    const sorted = sortSpecs(specs, {
      fields: [{ field: 'status', direction: 'asc' }]
    });

    const statusOrder = ['planned', 'in-progress', 'implemented', 'tested', 'deprecated'];
    for (let i = 1; i < sorted.length; i++) {
      const prevOrder = statusOrder.indexOf(sorted[i - 1].status);
      const currOrder = statusOrder.indexOf(sorted[i].status);
      expect(prevOrder).toBeLessThanOrEqual(currOrder);
    }
  });
});
```

---

## Related Specs

- Spec 231: Spec List Layout
- Spec 233: Spec Filter System
- Spec 234: Spec Search
- Spec 236: Spec Detail View
