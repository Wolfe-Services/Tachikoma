<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { SearchFilters } from '$lib/types/spec-search';

  export let filters: SearchFilters = {};

  const dispatch = createEventDispatcher<{
    change: SearchFilters;
  }>();

  function updateFilters() {
    dispatch('change', filters);
  }
</script>

<div class="search-filters">
  <div class="filter-row">
    <label>
      <input type="checkbox" bind:checked={filters.includeDrafts} on:change={updateFilters} />
      Include drafts
    </label>
  </div>
  
  <div class="filter-row">
    <label>
      Phase:
      <select bind:value={filters.phase} on:change={updateFilters}>
        <option value="">All phases</option>
        <option value="0">Phase 0</option>
        <option value="1">Phase 1</option>
        <option value="2">Phase 2</option>
      </select>
    </label>
  </div>

  <div class="filter-row">
    <label>
      Status:
      <select bind:value={filters.status} on:change={updateFilters}>
        <option value="">All statuses</option>
        <option value="draft">Draft</option>
        <option value="in-progress">In Progress</option>
        <option value="complete">Complete</option>
      </select>
    </label>
  </div>
</div>

<style>
  .search-filters {
    padding: 12px;
    border-bottom: 1px solid var(--color-border-default);
    background: var(--color-bg-surface);
  }

  .filter-row {
    margin-bottom: 8px;
  }

  .filter-row:last-child {
    margin-bottom: 0;
  }

  label {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    color: var(--color-fg-default);
  }

  input, select {
    padding: 4px 8px;
    border: 1px solid var(--color-border-default);
    border-radius: 4px;
    background: var(--color-bg-base);
    color: var(--color-fg-default);
  }

  input:focus, select:focus {
    outline: none;
    border-color: var(--color-accent-fg);
  }
</style>