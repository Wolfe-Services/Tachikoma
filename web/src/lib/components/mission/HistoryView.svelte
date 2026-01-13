<script lang="ts">
  import { onMount } from 'svelte';
  import type { MissionHistoryEntry, HistoryFilter, HistorySort } from '$lib/types/history';
  import HistoryCard from './HistoryCard.svelte';
  import HistoryFilters from './HistoryFilters.svelte';

  export let entries: MissionHistoryEntry[] = [];

  let filter: HistoryFilter = {};
  let sort: HistorySort = { field: 'createdAt', direction: 'desc' };
  let selectedIds = new Set<string>();
  let searchQuery = '';

  function applyFilter(entries: MissionHistoryEntry[]): MissionHistoryEntry[] {
    return entries.filter(entry => {
      if (filter.status?.length && !filter.status.includes(entry.state)) return false;
      if (filter.dateFrom && entry.createdAt < filter.dateFrom) return false;
      if (filter.dateTo && entry.createdAt > filter.dateTo) return false;
      if (filter.tags?.length && !filter.tags.some(t => entry.tags.includes(t))) return false;
      if (searchQuery) {
        const query = searchQuery.toLowerCase();
        return entry.title.toLowerCase().includes(query) ||
               entry.prompt.toLowerCase().includes(query);
      }
      return true;
    });
  }

  function applySort(entries: MissionHistoryEntry[]): MissionHistoryEntry[] {
    return [...entries].sort((a, b) => {
      const aVal = a[sort.field];
      const bVal = b[sort.field];
      const cmp = aVal < bVal ? -1 : aVal > bVal ? 1 : 0;
      return sort.direction === 'asc' ? cmp : -cmp;
    });
  }

  function toggleSelection(id: string) {
    if (selectedIds.has(id)) {
      selectedIds.delete(id);
    } else {
      selectedIds.add(id);
    }
    selectedIds = selectedIds;
  }

  function selectAll() {
    selectedIds = new Set(filteredEntries.map(e => e.id));
  }

  function clearSelection() {
    selectedIds = new Set();
  }

  function handleExport() {
    const selectedEntries = filteredEntries.filter(e => selectedIds.has(e.id));
    // TODO: Implement export functionality
    console.log('Exporting', selectedEntries.length, 'missions');
  }

  function handleDelete() {
    const selectedEntries = filteredEntries.filter(e => selectedIds.has(e.id));
    if (confirm(`Delete ${selectedEntries.length} missions? This action cannot be undone.`)) {
      // TODO: Implement delete functionality
      console.log('Deleting', selectedEntries.length, 'missions');
      clearSelection();
    }
  }

  $: filteredEntries = applySort(applyFilter(entries));
</script>

<div class="history-view">
  <header class="history-view__header">
    <input
      type="search"
      placeholder="Search missions..."
      bind:value={searchQuery}
      class="history-view__search"
    />

    <HistoryFilters bind:filter bind:sort />
  </header>

  {#if selectedIds.size > 0}
    <div class="history-view__selection-bar">
      <span>{selectedIds.size} selected</span>
      <button on:click={clearSelection}>Clear</button>
      <button on:click={selectAll}>Select All</button>
      <button on:click={handleExport}>Export</button>
      <button class="delete-btn" on:click={handleDelete}>Delete</button>
    </div>
  {/if}

  <div class="history-view__list">
    {#if filteredEntries.length === 0}
      <div class="history-view__empty">
        {#if entries.length === 0}
          No mission history yet.
        {:else}
          No missions match your filters.
        {/if}
      </div>
    {:else}
      {#each filteredEntries as entry (entry.id)}
        <HistoryCard
          {entry}
          selected={selectedIds.has(entry.id)}
          on:select={() => toggleSelection(entry.id)}
          on:open={() => {}}
        />
      {/each}
    {/if}
  </div>

  <footer class="history-view__footer">
    <span>{filteredEntries.length} of {entries.length} missions</span>
  </footer>
</div>

<style>
  .history-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--color-bg-surface);
    border-radius: var(--card-radius);
    overflow: hidden;
  }

  .history-view__header {
    display: flex;
    gap: var(--space-3);
    padding: var(--space-4);
    border-bottom: 1px solid var(--color-border);
    background: var(--color-bg-elevated);
  }

  .history-view__search {
    flex: 1;
    height: var(--input-height-md);
    padding: 0 var(--input-padding-x);
    border: 1px solid var(--color-border);
    border-radius: var(--input-radius);
    background: var(--color-bg-input);
    color: var(--color-text-primary);
    font-size: var(--input-font-size);
    transition: border-color var(--duration-150) var(--ease-out);
  }

  .history-view__search:focus {
    outline: none;
    border-color: var(--color-border-focus);
    box-shadow: var(--focus-ring);
  }

  .history-view__search::placeholder {
    color: var(--color-text-muted);
  }

  .history-view__selection-bar {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-3) var(--space-4);
    background: var(--color-primary-subtle);
    border-bottom: 1px solid var(--color-border);
    color: var(--color-text-primary);
    font-size: var(--text-sm);
  }

  .history-view__selection-bar span {
    font-weight: var(--font-medium);
  }

  .history-view__selection-bar button {
    height: var(--btn-height-sm);
    padding: 0 var(--btn-padding-x-sm);
    border: 1px solid var(--color-border);
    background: var(--color-bg-surface);
    color: var(--color-text-primary);
    border-radius: var(--btn-radius);
    font-size: var(--btn-font-size-sm);
    font-weight: var(--font-medium);
    cursor: pointer;
    transition: all var(--duration-150) var(--ease-out);
  }

  .history-view__selection-bar button:hover {
    background: var(--color-bg-hover);
  }

  .delete-btn {
    color: var(--color-error) !important;
    border-color: var(--color-error) !important;
  }

  .delete-btn:hover {
    background: var(--color-error-subtle) !important;
  }

  .history-view__list {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-4);
  }

  .history-view__empty {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 200px;
    color: var(--color-text-muted);
    font-size: var(--text-base);
    text-align: center;
  }

  .history-view__footer {
    padding: var(--space-3) var(--space-4);
    border-top: 1px solid var(--color-border);
    background: var(--color-bg-elevated);
    font-size: var(--text-xs);
    color: var(--color-text-muted);
  }

  @media (max-width: 768px) {
    .history-view__header {
      flex-direction: column;
      gap: var(--space-3);
    }

    .history-view__selection-bar {
      flex-wrap: wrap;
      gap: var(--space-2);
    }
  }
</style>