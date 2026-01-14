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