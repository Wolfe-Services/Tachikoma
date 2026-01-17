<script lang="ts">
  import type { HistoryFilter, HistorySort } from '$lib/types/history';

  export let filter: HistoryFilter = {};
  export let sort: HistorySort = { field: 'createdAt', direction: 'desc' };

  const statusOptions = ['idle', 'running', 'complete', 'error'];
  const sortFields = [
    { value: 'createdAt', label: 'Created' },
    { value: 'duration', label: 'Duration' },
    { value: 'cost', label: 'Cost' },
    { value: 'title', label: 'Title' }
  ];

  function updateFilter(key: keyof HistoryFilter, value: any) {
    filter = { ...filter, [key]: value };
  }

  function toggleStatus(status: string) {
    const currentStatuses = filter.status || [];
    if (currentStatuses.includes(status)) {
      filter.status = currentStatuses.filter(s => s !== status);
    } else {
      filter.status = [...currentStatuses, status];
    }
  }
</script>

<div class="history-filters">
  <!-- Status Filter -->
  <div class="filter-group">
    <label class="filter-label">Status</label>
    <div class="status-pills">
      {#each statusOptions as status}
        <button
          class="status-pill"
          class:active={filter.status?.includes(status)}
          on:click={() => toggleStatus(status)}
        >
          {status}
        </button>
      {/each}
    </div>
  </div>

  <!-- Date Range Filter -->
  <div class="filter-group">
    <label class="filter-label">Date Range</label>
    <div class="date-range">
      <input
        type="date"
        placeholder="From"
        value={filter.dateFrom || ''}
        on:input={e => updateFilter('dateFrom', e.target.value)}
      />
      <input
        type="date"
        placeholder="To"
        value={filter.dateTo || ''}
        on:input={e => updateFilter('dateTo', e.target.value)}
      />
    </div>
  </div>

  <!-- Sort -->
  <div class="filter-group">
    <label class="filter-label">Sort</label>
    <div class="sort-controls">
      <select
        value={sort.field}
        on:change={e => sort = { ...sort, field: e.target.value }}
      >
        {#each sortFields as field}
          <option value={field.value}>{field.label}</option>
        {/each}
      </select>
      <button
        class="sort-direction"
        on:click={() => sort = { ...sort, direction: sort.direction === 'asc' ? 'desc' : 'asc' }}
      >
        {#if sort.direction === 'asc'}↑{:else}↓{/if}
      </button>
    </div>
  </div>
</div>

<style>
  .history-filters {
    display: flex;
    gap: var(--space-6);
    align-items: end;
    flex-wrap: wrap;
  }

  .filter-group {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .filter-label {
    font-size: var(--text-xs);
    color: var(--color-text-secondary);
    font-weight: var(--font-medium);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .status-pills {
    display: flex;
    gap: var(--space-1);
  }

  .status-pill {
    padding: var(--space-1) var(--space-2);
    border: 1px solid var(--color-border);
    background: var(--color-bg-surface);
    color: var(--color-text-secondary);
    border-radius: var(--radius-md);
    font-size: var(--text-xs);
    cursor: pointer;
    transition: all var(--duration-150) var(--ease-out);
  }

  .status-pill:hover {
    background: var(--color-bg-hover);
  }

  .status-pill.active {
    background: var(--color-primary-subtle);
    border-color: var(--color-primary);
    color: var(--color-primary);
  }

  .date-range {
    display: flex;
    gap: var(--space-2);
  }

  .date-range input {
    padding: var(--space-2) var(--space-3);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: var(--color-bg-input);
    color: var(--color-text-primary);
    font-size: var(--text-sm);
    width: 140px;
  }

  .date-range input:focus {
    outline: none;
    border-color: var(--color-border-focus);
    box-shadow: var(--focus-ring);
  }

  .sort-controls {
    display: flex;
    gap: var(--space-1);
  }

  .sort-controls select {
    padding: var(--space-2) var(--space-3);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: var(--color-bg-input);
    color: var(--color-text-primary);
    font-size: var(--text-sm);
    min-width: 120px;
  }

  .sort-controls select:focus {
    outline: none;
    border-color: var(--color-border-focus);
    box-shadow: var(--focus-ring);
  }

  .sort-direction {
    padding: var(--space-2);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: var(--color-bg-surface);
    color: var(--color-text-secondary);
    cursor: pointer;
    font-size: var(--text-base);
    line-height: 1;
    width: var(--space-8);
    height: var(--space-8);
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--duration-150) var(--ease-out);
  }

  .sort-direction:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  @media (max-width: 768px) {
    .history-filters {
      flex-direction: column;
      align-items: stretch;
    }

    .filter-group {
      width: 100%;
    }
  }
</style>