# 232 - History View Component

**Phase:** 10 - Mission Panel UI
**Spec ID:** 232
**Status:** Planned
**Dependencies:** 216-mission-layout, 217-mission-state
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Create a mission history view that displays past missions with filtering, sorting, and quick access to mission details, logs, and artifacts.

---

## Acceptance Criteria

- [ ] Chronological list of past missions
- [ ] Filter by status, date range, tags
- [ ] Sort by date, duration, cost
- [ ] Quick preview on hover
- [ ] Batch operations (delete, export)
- [ ] Search functionality

---

## Implementation Details

### 1. Types (src/lib/types/history.ts)

```typescript
export interface MissionHistoryEntry {
  id: string;
  title: string;
  prompt: string;
  state: string;
  createdAt: string;
  completedAt: string;
  duration: number;
  cost: number;
  tokensUsed: number;
  filesChanged: number;
  tags: string[];
}

export interface HistoryFilter {
  status?: string[];
  dateFrom?: string;
  dateTo?: string;
  tags?: string[];
  search?: string;
}

export interface HistorySort {
  field: 'createdAt' | 'duration' | 'cost' | 'title';
  direction: 'asc' | 'desc';
}
```

### 2. History View Component (src/lib/components/mission/HistoryView.svelte)

```svelte
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
      <button on:click={() => {}}>Export</button>
      <button class="delete-btn" on:click={() => {}}>Delete</button>
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
      {#each filteredEntries as entry}
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
  }

  .history-view__header {
    display: flex;
    gap: 12px;
    padding: 12px;
    border-bottom: 1px solid var(--color-border);
  }

  .history-view__search {
    flex: 1;
    padding: 8px 12px;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
    font-size: 14px;
  }

  .history-view__selection-bar {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 12px;
    background: var(--color-bg-active);
    border-bottom: 1px solid var(--color-border);
  }

  .history-view__selection-bar button {
    padding: 4px 12px;
    border: 1px solid var(--color-border);
    background: var(--color-bg-primary);
    border-radius: 4px;
    font-size: 12px;
    cursor: pointer;
  }

  .delete-btn {
    color: var(--color-error);
    border-color: var(--color-error) !important;
  }

  .history-view__list {
    flex: 1;
    overflow-y: auto;
    padding: 12px;
  }

  .history-view__empty {
    padding: 48px;
    text-align: center;
    color: var(--color-text-muted);
  }

  .history-view__footer {
    padding: 8px 12px;
    border-top: 1px solid var(--color-border);
    font-size: 12px;
    color: var(--color-text-muted);
  }
</style>
```

---

## Testing Requirements

1. Filtering works correctly
2. Sorting changes order
3. Search filters by text
4. Selection works with batch ops
5. Empty states display properly

---

## Related Specs

- Depends on: [217-mission-state.md](217-mission-state.md)
- Next: [233-mission-comparison.md](233-mission-comparison.md)
