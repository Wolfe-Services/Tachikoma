# Spec 233: Spec Filter System

## Phase
11 - Spec Browser UI

## Spec ID
233

## Status
Planned

## Dependencies
- Spec 231 (Spec List Layout)
- Phase 10 (Core UI Components)

## Estimated Context
~10%

---

## Objective

Implement a comprehensive filtering system for the Spec Browser that allows users to filter specs by status, phase, tags, dependencies, date ranges, and custom attributes. The filter UI should support saved filter presets and combine filters with AND/OR logic.

---

## Acceptance Criteria

- [ ] Filter by spec status (multiple selection)
- [ ] Filter by phase (single or range)
- [ ] Filter by tags with autocomplete
- [ ] Filter by dependency relationships
- [ ] Date range filtering for created/updated
- [ ] Save and load filter presets
- [ ] Clear all filters with single action
- [ ] Active filter chips display current filters
- [ ] Filter counts show matching specs
- [ ] URL persistence for filter state

---

## Implementation Details

### SpecFilterPanel.svelte

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { writable, derived } from 'svelte/store';
  import type { SpecFilter, FilterPreset, SpecStatus } from '$lib/types/spec';
  import MultiSelect from '$lib/components/MultiSelect.svelte';
  import RangeSlider from '$lib/components/RangeSlider.svelte';
  import DateRangePicker from '$lib/components/DateRangePicker.svelte';
  import TagInput from '$lib/components/TagInput.svelte';
  import Button from '$lib/components/Button.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import Popover from '$lib/components/Popover.svelte';
  import { specStore } from '$lib/stores/spec-store';

  export let filter: SpecFilter = createDefaultFilter();
  export let presets: FilterPreset[] = [];
  export let collapsed = false;

  const dispatch = createEventDispatcher<{
    change: SpecFilter;
    savePreset: { name: string; filter: SpecFilter };
    loadPreset: FilterPreset;
    deletePreset: FilterPreset;
  }>();

  const statusOptions: { value: SpecStatus; label: string; color: string }[] = [
    { value: 'planned', label: 'Planned', color: 'var(--color-status-planned)' },
    { value: 'in-progress', label: 'In Progress', color: 'var(--color-status-progress)' },
    { value: 'implemented', label: 'Implemented', color: 'var(--color-status-implemented)' },
    { value: 'tested', label: 'Tested', color: 'var(--color-status-tested)' },
    { value: 'deprecated', label: 'Deprecated', color: 'var(--color-status-deprecated)' },
  ];

  // Derive available tags from all specs
  const availableTags = derived(specStore, $specs => {
    const tagSet = new Set<string>();
    $specs.forEach(spec => spec.tags?.forEach(tag => tagSet.add(tag)));
    return Array.from(tagSet).sort();
  });

  // Derive available phases
  const availablePhases = derived(specStore, $specs => {
    const phases = new Set($specs.map(s => s.phase));
    return Array.from(phases).sort((a, b) => a - b);
  });

  let showPresetMenu = false;
  let newPresetName = '';

  $: activeFilterCount = countActiveFilters(filter);

  function createDefaultFilter(): SpecFilter {
    return {
      statuses: [],
      phases: [],
      phaseRange: null,
      tags: [],
      tagMode: 'any',
      hasDependencies: null,
      hasDependents: null,
      createdAfter: null,
      createdBefore: null,
      updatedAfter: null,
      updatedBefore: null,
      author: null,
    };
  }

  function countActiveFilters(f: SpecFilter): number {
    let count = 0;
    if (f.statuses.length) count++;
    if (f.phases.length || f.phaseRange) count++;
    if (f.tags.length) count++;
    if (f.hasDependencies !== null) count++;
    if (f.hasDependents !== null) count++;
    if (f.createdAfter || f.createdBefore) count++;
    if (f.updatedAfter || f.updatedBefore) count++;
    if (f.author) count++;
    return count;
  }

  function updateFilter<K extends keyof SpecFilter>(key: K, value: SpecFilter[K]) {
    filter = { ...filter, [key]: value };
    dispatch('change', filter);
  }

  function clearAllFilters() {
    filter = createDefaultFilter();
    dispatch('change', filter);
  }

  function removeFilter(key: keyof SpecFilter) {
    const defaultFilter = createDefaultFilter();
    filter = { ...filter, [key]: defaultFilter[key] };
    dispatch('change', filter);
  }

  function savePreset() {
    if (newPresetName.trim()) {
      dispatch('savePreset', { name: newPresetName.trim(), filter });
      newPresetName = '';
      showPresetMenu = false;
    }
  }

  function loadPreset(preset: FilterPreset) {
    filter = { ...preset.filter };
    dispatch('loadPreset', preset);
    dispatch('change', filter);
    showPresetMenu = false;
  }
</script>

<aside class="spec-filter" class:spec-filter--collapsed={collapsed}>
  <header class="spec-filter__header">
    <h2 class="spec-filter__title">
      <Icon name="filter" size={16} />
      Filters
      {#if activeFilterCount > 0}
        <span class="spec-filter__count">{activeFilterCount}</span>
      {/if}
    </h2>
    <div class="spec-filter__actions">
      {#if activeFilterCount > 0}
        <Button variant="ghost" size="sm" on:click={clearAllFilters}>
          Clear all
        </Button>
      {/if}
      <Popover bind:open={showPresetMenu} placement="bottom-end">
        <Button slot="trigger" variant="ghost" size="sm" aria-label="Filter presets">
          <Icon name="bookmark" size={16} />
        </Button>
        <div class="spec-filter__preset-menu">
          <div class="spec-filter__preset-header">
            <span>Filter Presets</span>
          </div>
          {#if presets.length > 0}
            <ul class="spec-filter__preset-list">
              {#each presets as preset}
                <li class="spec-filter__preset-item">
                  <button
                    class="spec-filter__preset-btn"
                    on:click={() => loadPreset(preset)}
                  >
                    {preset.name}
                  </button>
                  <button
                    class="spec-filter__preset-delete"
                    on:click={() => dispatch('deletePreset', preset)}
                    aria-label="Delete preset"
                  >
                    <Icon name="x" size={14} />
                  </button>
                </li>
              {/each}
            </ul>
          {:else}
            <p class="spec-filter__preset-empty">No saved presets</p>
          {/if}
          <div class="spec-filter__preset-save">
            <input
              type="text"
              placeholder="New preset name..."
              bind:value={newPresetName}
              on:keydown={(e) => e.key === 'Enter' && savePreset()}
            />
            <Button size="sm" disabled={!newPresetName.trim()} on:click={savePreset}>
              Save
            </Button>
          </div>
        </div>
      </Popover>
    </div>
  </header>

  <div class="spec-filter__body">
    <!-- Status Filter -->
    <section class="spec-filter__section">
      <h3 class="spec-filter__section-title">Status</h3>
      <div class="spec-filter__status-grid">
        {#each statusOptions as option}
          <label class="spec-filter__status-option">
            <input
              type="checkbox"
              checked={filter.statuses.includes(option.value)}
              on:change={(e) => {
                const checked = e.currentTarget.checked;
                const newStatuses = checked
                  ? [...filter.statuses, option.value]
                  : filter.statuses.filter(s => s !== option.value);
                updateFilter('statuses', newStatuses);
              }}
            />
            <span class="spec-filter__status-indicator" style:background={option.color} />
            <span>{option.label}</span>
          </label>
        {/each}
      </div>
    </section>

    <!-- Phase Filter -->
    <section class="spec-filter__section">
      <h3 class="spec-filter__section-title">Phase</h3>
      <div class="spec-filter__phase-options">
        <label class="spec-filter__radio">
          <input
            type="radio"
            name="phaseMode"
            checked={!filter.phaseRange}
            on:change={() => updateFilter('phaseRange', null)}
          />
          <span>Specific phases</span>
        </label>
        <label class="spec-filter__radio">
          <input
            type="radio"
            name="phaseMode"
            checked={!!filter.phaseRange}
            on:change={() => updateFilter('phaseRange', [1, Math.max(...$availablePhases)])}
          />
          <span>Phase range</span>
        </label>
      </div>

      {#if filter.phaseRange}
        <RangeSlider
          min={1}
          max={Math.max(...$availablePhases, 15)}
          bind:values={filter.phaseRange}
          on:change={(e) => updateFilter('phaseRange', e.detail)}
        />
      {:else}
        <MultiSelect
          options={$availablePhases.map(p => ({ value: p, label: `Phase ${p}` }))}
          selected={filter.phases}
          on:change={(e) => updateFilter('phases', e.detail)}
          placeholder="Select phases..."
        />
      {/if}
    </section>

    <!-- Tags Filter -->
    <section class="spec-filter__section">
      <h3 class="spec-filter__section-title">Tags</h3>
      <TagInput
        tags={filter.tags}
        suggestions={$availableTags}
        on:change={(e) => updateFilter('tags', e.detail)}
        placeholder="Add tags..."
      />
      {#if filter.tags.length > 0}
        <div class="spec-filter__tag-mode">
          <label class="spec-filter__radio">
            <input
              type="radio"
              name="tagMode"
              value="any"
              checked={filter.tagMode === 'any'}
              on:change={() => updateFilter('tagMode', 'any')}
            />
            <span>Match any</span>
          </label>
          <label class="spec-filter__radio">
            <input
              type="radio"
              name="tagMode"
              value="all"
              checked={filter.tagMode === 'all'}
              on:change={() => updateFilter('tagMode', 'all')}
            />
            <span>Match all</span>
          </label>
        </div>
      {/if}
    </section>

    <!-- Dependencies Filter -->
    <section class="spec-filter__section">
      <h3 class="spec-filter__section-title">Dependencies</h3>
      <div class="spec-filter__dep-options">
        <label class="spec-filter__checkbox">
          <input
            type="checkbox"
            checked={filter.hasDependencies === true}
            indeterminate={filter.hasDependencies === null}
            on:change={(e) => {
              const states = [null, true, false];
              const current = states.indexOf(filter.hasDependencies);
              updateFilter('hasDependencies', states[(current + 1) % 3]);
            }}
          />
          <span>Has dependencies</span>
        </label>
        <label class="spec-filter__checkbox">
          <input
            type="checkbox"
            checked={filter.hasDependents === true}
            indeterminate={filter.hasDependents === null}
            on:change={(e) => {
              const states = [null, true, false];
              const current = states.indexOf(filter.hasDependents);
              updateFilter('hasDependents', states[(current + 1) % 3]);
            }}
          />
          <span>Has dependents</span>
        </label>
      </div>
    </section>

    <!-- Date Filters -->
    <section class="spec-filter__section">
      <h3 class="spec-filter__section-title">Created Date</h3>
      <DateRangePicker
        startDate={filter.createdAfter}
        endDate={filter.createdBefore}
        on:change={(e) => {
          updateFilter('createdAfter', e.detail.start);
          updateFilter('createdBefore', e.detail.end);
        }}
      />
    </section>

    <section class="spec-filter__section">
      <h3 class="spec-filter__section-title">Updated Date</h3>
      <DateRangePicker
        startDate={filter.updatedAfter}
        endDate={filter.updatedBefore}
        on:change={(e) => {
          updateFilter('updatedAfter', e.detail.start);
          updateFilter('updatedBefore', e.detail.end);
        }}
      />
    </section>
  </div>
</aside>

<!-- Active Filter Chips -->
{#if activeFilterCount > 0}
  <div class="spec-filter__chips">
    {#if filter.statuses.length > 0}
      <span class="spec-filter__chip">
        Status: {filter.statuses.join(', ')}
        <button on:click={() => removeFilter('statuses')} aria-label="Remove status filter">
          <Icon name="x" size={12} />
        </button>
      </span>
    {/if}
    {#if filter.phases.length > 0}
      <span class="spec-filter__chip">
        Phases: {filter.phases.join(', ')}
        <button on:click={() => removeFilter('phases')} aria-label="Remove phase filter">
          <Icon name="x" size={12} />
        </button>
      </span>
    {/if}
    {#if filter.tags.length > 0}
      <span class="spec-filter__chip">
        Tags ({filter.tagMode}): {filter.tags.join(', ')}
        <button on:click={() => removeFilter('tags')} aria-label="Remove tag filter">
          <Icon name="x" size={12} />
        </button>
      </span>
    {/if}
  </div>
{/if}

<style>
  .spec-filter {
    width: 280px;
    border-right: 1px solid var(--color-border);
    background: var(--color-surface);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    transition: width 0.2s ease;
  }

  .spec-filter--collapsed {
    width: 48px;
  }

  .spec-filter__header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px;
    border-bottom: 1px solid var(--color-border);
  }

  .spec-filter__title {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0;
  }

  .spec-filter__count {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 18px;
    height: 18px;
    padding: 0 5px;
    font-size: 0.75rem;
    font-weight: 600;
    background: var(--color-primary);
    color: white;
    border-radius: 9px;
  }

  .spec-filter__actions {
    display: flex;
    gap: 4px;
  }

  .spec-filter__body {
    flex: 1;
    overflow-y: auto;
    padding: 8px 0;
  }

  .spec-filter__section {
    padding: 12px 16px;
    border-bottom: 1px solid var(--color-border-subtle);
  }

  .spec-filter__section:last-child {
    border-bottom: none;
  }

  .spec-filter__section-title {
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--color-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin: 0 0 12px;
  }

  .spec-filter__status-grid {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .spec-filter__status-option {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 0.875rem;
    cursor: pointer;
  }

  .spec-filter__status-indicator {
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }

  .spec-filter__radio,
  .spec-filter__checkbox {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 0.875rem;
    cursor: pointer;
  }

  .spec-filter__phase-options,
  .spec-filter__tag-mode,
  .spec-filter__dep-options {
    display: flex;
    gap: 16px;
    margin-bottom: 12px;
  }

  .spec-filter__chips {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    padding: 8px 16px;
    background: var(--color-surface-subtle);
    border-bottom: 1px solid var(--color-border);
  }

  .spec-filter__chip {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 4px 8px;
    font-size: 0.75rem;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 4px;
  }

  .spec-filter__chip button {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 2px;
    background: none;
    border: none;
    cursor: pointer;
    color: var(--color-text-tertiary);
    border-radius: 2px;
  }

  .spec-filter__chip button:hover {
    background: var(--color-hover);
    color: var(--color-text-primary);
  }

  .spec-filter__preset-menu {
    width: 240px;
    padding: 8px 0;
  }

  .spec-filter__preset-header {
    padding: 8px 12px;
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--color-text-secondary);
    text-transform: uppercase;
  }

  .spec-filter__preset-list {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .spec-filter__preset-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 4px;
  }

  .spec-filter__preset-btn {
    flex: 1;
    padding: 8px;
    text-align: left;
    background: none;
    border: none;
    cursor: pointer;
    font-size: 0.875rem;
    border-radius: 4px;
  }

  .spec-filter__preset-btn:hover {
    background: var(--color-hover);
  }

  .spec-filter__preset-delete {
    padding: 4px;
    background: none;
    border: none;
    cursor: pointer;
    color: var(--color-text-tertiary);
    border-radius: 4px;
  }

  .spec-filter__preset-delete:hover {
    background: var(--color-danger-subtle);
    color: var(--color-danger);
  }

  .spec-filter__preset-empty {
    padding: 16px;
    text-align: center;
    font-size: 0.875rem;
    color: var(--color-text-tertiary);
  }

  .spec-filter__preset-save {
    display: flex;
    gap: 8px;
    padding: 12px;
    border-top: 1px solid var(--color-border);
  }

  .spec-filter__preset-save input {
    flex: 1;
    padding: 6px 8px;
    font-size: 0.875rem;
    border: 1px solid var(--color-border);
    border-radius: 4px;
  }
</style>
```

### Filter Types

```typescript
// types/spec.ts additions
export interface SpecFilter {
  statuses: SpecStatus[];
  phases: number[];
  phaseRange: [number, number] | null;
  tags: string[];
  tagMode: 'any' | 'all';
  hasDependencies: boolean | null;
  hasDependents: boolean | null;
  createdAfter: Date | null;
  createdBefore: Date | null;
  updatedAfter: Date | null;
  updatedBefore: Date | null;
  author: string | null;
}

export interface FilterPreset {
  id: string;
  name: string;
  filter: SpecFilter;
  createdAt: Date;
}
```

### Filter Utilities

```typescript
// utils/filter.ts
import type { Spec, SpecFilter } from '$lib/types/spec';

export function applyFilter(specs: Spec[], filter: SpecFilter): Spec[] {
  return specs.filter(spec => {
    // Status filter
    if (filter.statuses.length > 0 && !filter.statuses.includes(spec.status)) {
      return false;
    }

    // Phase filter
    if (filter.phaseRange) {
      const [min, max] = filter.phaseRange;
      if (spec.phase < min || spec.phase > max) {
        return false;
      }
    } else if (filter.phases.length > 0 && !filter.phases.includes(spec.phase)) {
      return false;
    }

    // Tags filter
    if (filter.tags.length > 0) {
      const specTags = spec.tags || [];
      if (filter.tagMode === 'all') {
        if (!filter.tags.every(tag => specTags.includes(tag))) {
          return false;
        }
      } else {
        if (!filter.tags.some(tag => specTags.includes(tag))) {
          return false;
        }
      }
    }

    // Dependencies filter
    if (filter.hasDependencies === true && (!spec.dependencies || spec.dependencies.length === 0)) {
      return false;
    }
    if (filter.hasDependencies === false && spec.dependencies && spec.dependencies.length > 0) {
      return false;
    }

    // Date filters
    if (filter.createdAfter && new Date(spec.createdAt) < filter.createdAfter) {
      return false;
    }
    if (filter.createdBefore && new Date(spec.createdAt) > filter.createdBefore) {
      return false;
    }
    if (filter.updatedAfter && new Date(spec.updatedAt) < filter.updatedAfter) {
      return false;
    }
    if (filter.updatedBefore && new Date(spec.updatedAt) > filter.updatedBefore) {
      return false;
    }

    return true;
  });
}

export function filterToQueryString(filter: SpecFilter): string {
  const params = new URLSearchParams();

  if (filter.statuses.length) {
    params.set('status', filter.statuses.join(','));
  }
  if (filter.phases.length) {
    params.set('phase', filter.phases.join(','));
  }
  if (filter.phaseRange) {
    params.set('phaseRange', filter.phaseRange.join('-'));
  }
  if (filter.tags.length) {
    params.set('tags', filter.tags.join(','));
    params.set('tagMode', filter.tagMode);
  }

  return params.toString();
}

export function queryStringToFilter(query: string): Partial<SpecFilter> {
  const params = new URLSearchParams(query);
  const filter: Partial<SpecFilter> = {};

  const status = params.get('status');
  if (status) {
    filter.statuses = status.split(',') as SpecStatus[];
  }

  const phase = params.get('phase');
  if (phase) {
    filter.phases = phase.split(',').map(Number);
  }

  const phaseRange = params.get('phaseRange');
  if (phaseRange) {
    const [min, max] = phaseRange.split('-').map(Number);
    filter.phaseRange = [min, max];
  }

  const tags = params.get('tags');
  if (tags) {
    filter.tags = tags.split(',');
    filter.tagMode = (params.get('tagMode') as 'any' | 'all') || 'any';
  }

  return filter;
}
```

---

## Testing Requirements

### Unit Tests

```typescript
import { render, fireEvent, screen } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import SpecFilterPanel from './SpecFilterPanel.svelte';
import { applyFilter, filterToQueryString, queryStringToFilter } from '$lib/utils/filter';
import { createMockSpecs } from '$lib/test-utils/mock-data';

describe('SpecFilterPanel', () => {
  it('renders all filter sections', () => {
    render(SpecFilterPanel);

    expect(screen.getByText('Status')).toBeInTheDocument();
    expect(screen.getByText('Phase')).toBeInTheDocument();
    expect(screen.getByText('Tags')).toBeInTheDocument();
    expect(screen.getByText('Dependencies')).toBeInTheDocument();
  });

  it('emits filter change on status selection', async () => {
    const { component } = render(SpecFilterPanel);

    const changeHandler = vi.fn();
    component.$on('change', changeHandler);

    await fireEvent.click(screen.getByText('In Progress'));

    expect(changeHandler).toHaveBeenCalledWith(
      expect.objectContaining({
        detail: expect.objectContaining({
          statuses: ['in-progress']
        })
      })
    );
  });

  it('displays active filter count', async () => {
    render(SpecFilterPanel, {
      props: {
        filter: {
          statuses: ['planned'],
          phases: [1, 2],
          tags: [],
          tagMode: 'any',
          hasDependencies: null,
          hasDependents: null,
          createdAfter: null,
          createdBefore: null,
          updatedAfter: null,
          updatedBefore: null,
          author: null
        }
      }
    });

    expect(screen.getByText('2')).toBeInTheDocument();
  });

  it('clears all filters', async () => {
    const { component } = render(SpecFilterPanel, {
      props: {
        filter: { statuses: ['planned'], phases: [1] }
      }
    });

    const changeHandler = vi.fn();
    component.$on('change', changeHandler);

    await fireEvent.click(screen.getByText('Clear all'));

    expect(changeHandler).toHaveBeenCalledWith(
      expect.objectContaining({
        detail: expect.objectContaining({
          statuses: [],
          phases: []
        })
      })
    );
  });
});

describe('applyFilter', () => {
  const specs = createMockSpecs(20);

  it('filters by status', () => {
    const result = applyFilter(specs, {
      ...createDefaultFilter(),
      statuses: ['planned']
    });

    expect(result.every(s => s.status === 'planned')).toBe(true);
  });

  it('filters by phase range', () => {
    const result = applyFilter(specs, {
      ...createDefaultFilter(),
      phaseRange: [1, 5]
    });

    expect(result.every(s => s.phase >= 1 && s.phase <= 5)).toBe(true);
  });

  it('filters by tags with any mode', () => {
    const result = applyFilter(specs, {
      ...createDefaultFilter(),
      tags: ['ui', 'api'],
      tagMode: 'any'
    });

    expect(result.every(s =>
      s.tags?.includes('ui') || s.tags?.includes('api')
    )).toBe(true);
  });

  it('filters by tags with all mode', () => {
    const result = applyFilter(specs, {
      ...createDefaultFilter(),
      tags: ['ui', 'component'],
      tagMode: 'all'
    });

    expect(result.every(s =>
      s.tags?.includes('ui') && s.tags?.includes('component')
    )).toBe(true);
  });
});

describe('filterToQueryString / queryStringToFilter', () => {
  it('serializes filter to query string', () => {
    const filter = {
      statuses: ['planned', 'in-progress'],
      phases: [1, 2, 3],
      tags: ['ui'],
      tagMode: 'any'
    };

    const query = filterToQueryString(filter);
    expect(query).toContain('status=planned,in-progress');
    expect(query).toContain('phase=1,2,3');
  });

  it('parses query string to filter', () => {
    const query = 'status=planned,in-progress&phase=1,2&tags=ui&tagMode=all';
    const filter = queryStringToFilter(query);

    expect(filter.statuses).toEqual(['planned', 'in-progress']);
    expect(filter.phases).toEqual([1, 2]);
    expect(filter.tags).toEqual(['ui']);
    expect(filter.tagMode).toBe('all');
  });
});
```

---

## Related Specs

- Spec 231: Spec List Layout
- Spec 234: Spec Search
- Spec 235: Spec Sort
- Spec 247: Spec Export (filtered export)
