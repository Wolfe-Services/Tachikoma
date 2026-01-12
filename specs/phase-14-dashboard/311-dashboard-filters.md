# 311 - Dashboard Filters

**Phase:** 14 - Dashboard
**Spec ID:** 311
**Status:** Planned
**Dependencies:** 296-dashboard-layout
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Create reusable filter components for dashboard data filtering, including multi-select dropdowns, search filters, and filter persistence across views.

---

## Acceptance Criteria

- [ ] `FilterBar.svelte` component created
- [ ] `FilterDropdown.svelte` multi-select
- [ ] `FilterChip.svelte` for active filters
- [ ] Search within filter options
- [ ] Filter state persistence
- [ ] Clear all filters functionality
- [ ] Filter presets/saved filters
- [ ] Responsive filter panel

---

## Implementation Details

### 1. Filter Bar Component (web/src/lib/components/filters/FilterBar.svelte)

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { fly, fade } from 'svelte/transition';
  import type { FilterConfig, ActiveFilter } from '$lib/types/filters';
  import Icon from '$lib/components/common/Icon.svelte';
  import FilterDropdown from './FilterDropdown.svelte';
  import FilterChip from './FilterChip.svelte';
  import SearchInput from '$lib/components/common/SearchInput.svelte';

  export let filters: FilterConfig[] = [];
  export let activeFilters: ActiveFilter[] = [];
  export let showSearch: boolean = true;
  export let searchPlaceholder: string = 'Search...';
  export let presets: Array<{ id: string; name: string; filters: ActiveFilter[] }> = [];

  const dispatch = createEventDispatcher<{
    change: ActiveFilter[];
    search: string;
    clear: void;
  }>();

  let searchQuery = '';
  let mobileFiltersOpen = false;

  function handleFilterChange(filterId: string, values: string[]) {
    const existing = activeFilters.find(f => f.id === filterId);

    if (values.length === 0) {
      activeFilters = activeFilters.filter(f => f.id !== filterId);
    } else if (existing) {
      activeFilters = activeFilters.map(f =>
        f.id === filterId ? { ...f, values } : f
      );
    } else {
      const config = filters.find(f => f.id === filterId);
      activeFilters = [...activeFilters, {
        id: filterId,
        label: config?.label || filterId,
        values
      }];
    }

    dispatch('change', activeFilters);
  }

  function removeFilter(filterId: string) {
    activeFilters = activeFilters.filter(f => f.id !== filterId);
    dispatch('change', activeFilters);
  }

  function clearAllFilters() {
    activeFilters = [];
    searchQuery = '';
    dispatch('clear');
    dispatch('change', []);
  }

  function applyPreset(preset: typeof presets[0]) {
    activeFilters = [...preset.filters];
    dispatch('change', activeFilters);
  }

  function handleSearch(event: CustomEvent<string>) {
    searchQuery = event.detail;
    dispatch('search', searchQuery);
  }

  $: hasActiveFilters = activeFilters.length > 0 || searchQuery.length > 0;
</script>

<div class="filter-bar">
  <div class="filter-main">
    {#if showSearch}
      <div class="search-section">
        <SearchInput
          bind:value={searchQuery}
          placeholder={searchPlaceholder}
          on:change={handleSearch}
        />
      </div>
    {/if}

    <div class="filters-section desktop-filters">
      {#each filters as filter (filter.id)}
        <FilterDropdown
          {filter}
          selectedValues={activeFilters.find(f => f.id === filter.id)?.values || []}
          on:change={(e) => handleFilterChange(filter.id, e.detail)}
        />
      {/each}
    </div>

    <button
      class="mobile-filters-toggle"
      on:click={() => mobileFiltersOpen = !mobileFiltersOpen}
      aria-expanded={mobileFiltersOpen}
    >
      <Icon name="sliders" size={18} />
      Filters
      {#if activeFilters.length > 0}
        <span class="filter-count">{activeFilters.length}</span>
      {/if}
    </button>

    {#if presets.length > 0}
      <div class="presets-dropdown">
        <button class="presets-btn">
          <Icon name="bookmark" size={16} />
          Presets
        </button>
        <div class="presets-menu">
          {#each presets as preset (preset.id)}
            <button
              class="preset-item"
              on:click={() => applyPreset(preset)}
            >
              {preset.name}
            </button>
          {/each}
        </div>
      </div>
    {/if}

    {#if hasActiveFilters}
      <button class="clear-btn" on:click={clearAllFilters}>
        <Icon name="x" size={14} />
        Clear All
      </button>
    {/if}
  </div>

  {#if activeFilters.length > 0}
    <div class="active-filters" transition:fly={{ y: -10, duration: 150 }}>
      {#each activeFilters as filter (filter.id)}
        <FilterChip
          label={filter.label}
          values={filter.values}
          on:remove={() => removeFilter(filter.id)}
        />
      {/each}
    </div>
  {/if}

  {#if mobileFiltersOpen}
    <div class="mobile-filters-panel" transition:fly={{ y: -10, duration: 200 }}>
      <div class="panel-header">
        <h3>Filters</h3>
        <button class="close-btn" on:click={() => mobileFiltersOpen = false}>
          <Icon name="x" size={20} />
        </button>
      </div>
      <div class="panel-body">
        {#each filters as filter (filter.id)}
          <div class="mobile-filter-group">
            <label class="filter-label">{filter.label}</label>
            <FilterDropdown
              {filter}
              selectedValues={activeFilters.find(f => f.id === filter.id)?.values || []}
              on:change={(e) => handleFilterChange(filter.id, e.detail)}
              inline
            />
          </div>
        {/each}
      </div>
      <div class="panel-footer">
        <button class="btn btn-secondary" on:click={clearAllFilters}>
          Clear All
        </button>
        <button class="btn btn-primary" on:click={() => mobileFiltersOpen = false}>
          Apply Filters
        </button>
      </div>
    </div>
  {/if}
</div>

<style>
  .filter-bar {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    padding: 1rem 0;
  }

  .filter-main {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    flex-wrap: wrap;
  }

  .search-section {
    flex: 1;
    min-width: 200px;
    max-width: 300px;
  }

  .filters-section {
    display: flex;
    gap: 0.5rem;
  }

  .mobile-filters-toggle {
    display: none;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    border: 1px solid var(--border-color);
    background: var(--bg-primary);
    border-radius: 0.5rem;
    font-size: 0.875rem;
    color: var(--text-primary);
    cursor: pointer;
  }

  .filter-count {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 1.25rem;
    height: 1.25rem;
    padding: 0 0.375rem;
    background: var(--accent-color);
    color: white;
    font-size: 0.6875rem;
    font-weight: 600;
    border-radius: 9999px;
  }

  .presets-dropdown {
    position: relative;
  }

  .presets-btn {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.5rem 0.75rem;
    border: 1px solid var(--border-color);
    background: var(--bg-primary);
    border-radius: 0.5rem;
    font-size: 0.8125rem;
    color: var(--text-secondary);
    cursor: pointer;
  }

  .presets-btn:hover {
    background: var(--bg-hover);
  }

  .presets-menu {
    position: absolute;
    top: 100%;
    left: 0;
    margin-top: 0.25rem;
    min-width: 150px;
    background: var(--bg-card);
    border: 1px solid var(--border-color);
    border-radius: 0.5rem;
    box-shadow: var(--shadow-lg);
    opacity: 0;
    visibility: hidden;
    transform: translateY(-5px);
    transition: all 0.15s ease;
    z-index: 100;
  }

  .presets-dropdown:hover .presets-menu {
    opacity: 1;
    visibility: visible;
    transform: translateY(0);
  }

  .preset-item {
    display: block;
    width: 100%;
    padding: 0.5rem 0.75rem;
    border: none;
    background: transparent;
    text-align: left;
    font-size: 0.8125rem;
    color: var(--text-primary);
    cursor: pointer;
  }

  .preset-item:hover {
    background: var(--bg-hover);
  }

  .clear-btn {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.375rem 0.625rem;
    border: none;
    background: transparent;
    font-size: 0.75rem;
    color: var(--text-secondary);
    cursor: pointer;
    border-radius: 0.375rem;
  }

  .clear-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .active-filters {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .mobile-filters-panel {
    display: none;
    position: fixed;
    inset: 0;
    background: var(--bg-card);
    z-index: 1000;
    flex-direction: column;
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem;
    border-bottom: 1px solid var(--border-color);
  }

  .panel-header h3 {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
  }

  .close-btn {
    padding: 0.375rem;
    border: none;
    background: transparent;
    cursor: pointer;
  }

  .panel-body {
    flex: 1;
    padding: 1rem;
    overflow-y: auto;
  }

  .mobile-filter-group {
    margin-bottom: 1rem;
  }

  .filter-label {
    display: block;
    margin-bottom: 0.5rem;
    font-size: 0.8125rem;
    font-weight: 500;
    color: var(--text-primary);
  }

  .panel-footer {
    display: flex;
    gap: 0.75rem;
    padding: 1rem;
    border-top: 1px solid var(--border-color);
  }

  .btn {
    flex: 1;
    padding: 0.75rem 1rem;
    border: none;
    border-radius: 0.5rem;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
  }

  .btn-secondary {
    background: var(--bg-secondary);
    color: var(--text-primary);
  }

  .btn-primary {
    background: var(--accent-color);
    color: white;
  }

  @media (max-width: 768px) {
    .desktop-filters {
      display: none;
    }

    .mobile-filters-toggle {
      display: flex;
    }

    .mobile-filters-panel {
      display: flex;
    }

    .search-section {
      flex: unset;
      width: 100%;
      max-width: unset;
    }
  }
</style>
```

### 2. Filter Dropdown Component (web/src/lib/components/filters/FilterDropdown.svelte)

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { fly } from 'svelte/transition';
  import type { FilterConfig } from '$lib/types/filters';
  import Icon from '$lib/components/common/Icon.svelte';

  export let filter: FilterConfig;
  export let selectedValues: string[] = [];
  export let inline: boolean = false;

  const dispatch = createEventDispatcher<{
    change: string[];
  }>();

  let open = false;
  let searchQuery = '';

  $: filteredOptions = filter.options.filter(opt =>
    opt.label.toLowerCase().includes(searchQuery.toLowerCase())
  );

  $: selectedCount = selectedValues.length;
  $: displayLabel = selectedCount > 0
    ? `${filter.label} (${selectedCount})`
    : filter.label;

  function toggleOption(value: string) {
    if (selectedValues.includes(value)) {
      selectedValues = selectedValues.filter(v => v !== value);
    } else {
      selectedValues = [...selectedValues, value];
    }
    dispatch('change', selectedValues);
  }

  function selectAll() {
    selectedValues = filter.options.map(o => o.value);
    dispatch('change', selectedValues);
  }

  function clearSelection() {
    selectedValues = [];
    dispatch('change', selectedValues);
  }

  function handleClickOutside(event: MouseEvent) {
    const target = event.target as HTMLElement;
    if (!target.closest('.filter-dropdown')) {
      open = false;
    }
  }
</script>

<svelte:window on:click={handleClickOutside} />

<div class="filter-dropdown" class:inline class:open>
  <button
    class="dropdown-trigger"
    on:click|stopPropagation={() => open = !open}
    aria-expanded={open}
    aria-haspopup="listbox"
  >
    <span class="trigger-label">{displayLabel}</span>
    <Icon name={open ? 'chevron-up' : 'chevron-down'} size={14} />
  </button>

  {#if open}
    <div
      class="dropdown-menu"
      role="listbox"
      aria-multiselectable="true"
      transition:fly={{ y: -5, duration: 150 }}
    >
      {#if filter.searchable}
        <div class="menu-search">
          <Icon name="search" size={14} />
          <input
            type="text"
            placeholder="Search..."
            bind:value={searchQuery}
            on:click|stopPropagation
          />
        </div>
      {/if}

      <div class="menu-actions">
        <button on:click|stopPropagation={selectAll}>Select All</button>
        <button on:click|stopPropagation={clearSelection}>Clear</button>
      </div>

      <ul class="options-list">
        {#each filteredOptions as option (option.value)}
          <li>
            <label class="option-item">
              <input
                type="checkbox"
                checked={selectedValues.includes(option.value)}
                on:change={() => toggleOption(option.value)}
                on:click|stopPropagation
              />
              {#if option.icon}
                <Icon name={option.icon} size={14} />
              {/if}
              {#if option.color}
                <span class="option-color" style="background: {option.color}" />
              {/if}
              <span class="option-label">{option.label}</span>
              {#if option.count !== undefined}
                <span class="option-count">{option.count}</span>
              {/if}
            </label>
          </li>
        {/each}

        {#if filteredOptions.length === 0}
          <li class="no-results">No options found</li>
        {/if}
      </ul>
    </div>
  {/if}
</div>

<style>
  .filter-dropdown {
    position: relative;
  }

  .dropdown-trigger {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    border: 1px solid var(--border-color);
    background: var(--bg-primary);
    border-radius: 0.5rem;
    font-size: 0.8125rem;
    color: var(--text-primary);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .dropdown-trigger:hover {
    border-color: var(--border-hover);
  }

  .filter-dropdown.open .dropdown-trigger {
    border-color: var(--accent-color);
  }

  .inline .dropdown-trigger {
    width: 100%;
    justify-content: space-between;
  }

  .dropdown-menu {
    position: absolute;
    top: 100%;
    left: 0;
    margin-top: 0.25rem;
    min-width: 200px;
    max-height: 300px;
    background: var(--bg-card);
    border: 1px solid var(--border-color);
    border-radius: 0.5rem;
    box-shadow: var(--shadow-lg);
    z-index: 100;
    overflow: hidden;
  }

  .inline .dropdown-menu {
    position: static;
    margin-top: 0.5rem;
    box-shadow: none;
    border: 1px solid var(--border-color);
  }

  .menu-search {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid var(--border-color);
    color: var(--text-tertiary);
  }

  .menu-search input {
    flex: 1;
    border: none;
    background: transparent;
    font-size: 0.8125rem;
    color: var(--text-primary);
    outline: none;
  }

  .menu-actions {
    display: flex;
    justify-content: space-between;
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid var(--border-color);
    background: var(--bg-secondary);
  }

  .menu-actions button {
    padding: 0;
    border: none;
    background: transparent;
    font-size: 0.6875rem;
    color: var(--accent-color);
    cursor: pointer;
  }

  .menu-actions button:hover {
    text-decoration: underline;
  }

  .options-list {
    list-style: none;
    padding: 0.25rem 0;
    margin: 0;
    max-height: 200px;
    overflow-y: auto;
  }

  .option-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    font-size: 0.8125rem;
    color: var(--text-primary);
    cursor: pointer;
  }

  .option-item:hover {
    background: var(--bg-hover);
  }

  .option-color {
    width: 0.75rem;
    height: 0.75rem;
    border-radius: 0.125rem;
  }

  .option-label {
    flex: 1;
  }

  .option-count {
    font-size: 0.6875rem;
    color: var(--text-tertiary);
  }

  .no-results {
    padding: 1rem;
    text-align: center;
    font-size: 0.8125rem;
    color: var(--text-tertiary);
  }
</style>
```

### 3. Filter Chip Component (web/src/lib/components/filters/FilterChip.svelte)

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import Icon from '$lib/components/common/Icon.svelte';

  export let label: string;
  export let values: string[];

  const dispatch = createEventDispatcher<{ remove: void }>();

  $: displayValue = values.length > 2
    ? `${values.slice(0, 2).join(', ')} +${values.length - 2}`
    : values.join(', ');
</script>

<div class="filter-chip">
  <span class="chip-label">{label}:</span>
  <span class="chip-value">{displayValue}</span>
  <button
    class="chip-remove"
    on:click={() => dispatch('remove')}
    aria-label="Remove filter"
  >
    <Icon name="x" size={12} />
  </button>
</div>

<style>
  .filter-chip {
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.25rem 0.375rem 0.25rem 0.625rem;
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: 9999px;
    font-size: 0.75rem;
  }

  .chip-label {
    color: var(--text-tertiary);
  }

  .chip-value {
    color: var(--text-primary);
    font-weight: 500;
  }

  .chip-remove {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 1.125rem;
    height: 1.125rem;
    padding: 0;
    border: none;
    background: var(--bg-hover);
    border-radius: 50%;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .chip-remove:hover {
    background: var(--text-tertiary);
    color: white;
  }
</style>
```

### 4. Filter Types (web/src/lib/types/filters.ts)

```typescript
export interface FilterOption {
  value: string;
  label: string;
  icon?: string;
  color?: string;
  count?: number;
}

export interface FilterConfig {
  id: string;
  label: string;
  options: FilterOption[];
  searchable?: boolean;
  multi?: boolean;
}

export interface ActiveFilter {
  id: string;
  label: string;
  values: string[];
}
```

---

## Testing Requirements

1. Filter dropdown opens/closes correctly
2. Multi-select works with checkboxes
3. Search filters options list
4. Active filters display as chips
5. Clear all removes all filters
6. Presets apply correct filters
7. Mobile filter panel works

---

## Related Specs

- Depends on: [296-dashboard-layout.md](296-dashboard-layout.md)
- Next: [312-date-ranges.md](312-date-ranges.md)
- Used by: All dashboard data views
