<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import type { SearchQuery, SearchResult, SearchFilters, SearchHistory } from '$lib/types/spec-search';
  import { invoke } from '$lib/ipc';
  import SearchResultItem from './SearchResultItem.svelte';
  import SearchFiltersPanel from './SearchFiltersPanel.svelte';

  export let initialQuery = '';

  const dispatch = createEventDispatcher<{
    select: string;
    close: void;
  }>();

  let query = initialQuery;
  let results: SearchResult[] = [];
  let isSearching = false;
  let showFilters = false;
  let filters: SearchFilters = {};
  let history: SearchHistory[] = [];
  let selectedIndex = 0;
  let inputRef: HTMLInputElement;
  let searchTimeout: ReturnType<typeof setTimeout>;

  async function search() {
    if (!query.trim()) {
      results = [];
      return;
    }

    isSearching = true;
    try {
      results = await invoke('spec:search', { text: query, filters });
      selectedIndex = 0;

      // Update history
      addToHistory(query, results.length);
    } finally {
      isSearching = false;
    }
  }

  function handleInput() {
    clearTimeout(searchTimeout);
    searchTimeout = setTimeout(search, 150);
  }

  function handleKeyDown(event: KeyboardEvent) {
    switch (event.key) {
      case 'ArrowDown':
        event.preventDefault();
        selectedIndex = Math.min(selectedIndex + 1, results.length - 1);
        break;
      case 'ArrowUp':
        event.preventDefault();
        selectedIndex = Math.max(selectedIndex - 1, 0);
        break;
      case 'Enter':
        event.preventDefault();
        if (results[selectedIndex]) {
          dispatch('select', results[selectedIndex].specId);
        }
        break;
      case 'Escape':
        dispatch('close');
        break;
    }
  }

  function addToHistory(query: string, resultCount: number) {
    const entry: SearchHistory = {
      query,
      timestamp: new Date().toISOString(),
      resultCount,
    };
    history = [entry, ...history.filter(h => h.query !== query)].slice(0, 10);
    localStorage.setItem('spec-search-history', JSON.stringify(history));
  }

  function loadHistory() {
    const saved = localStorage.getItem('spec-search-history');
    if (saved) {
      history = JSON.parse(saved);
    }
  }

  function selectFromHistory(historyQuery: string) {
    query = historyQuery;
    search();
  }

  onMount(() => {
    loadHistory();
    inputRef?.focus();
    if (query) search();
  });
</script>

<div class="spec-search" on:keydown={handleKeyDown}>
  <div class="spec-search__input-row">
    <input
      bind:this={inputRef}
      type="search"
      class="spec-search__input"
      placeholder="Search specifications..."
      bind:value={query}
      on:input={handleInput}
    />

    <button
      class="filter-toggle"
      class:active={showFilters}
      on:click={() => { showFilters = !showFilters; }}
    >
      <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
        <path d="M1 2h14v2H1V2zm2 5h10v2H3V7zm2 5h6v2H5v-2z"/>
      </svg>
    </button>

    <button class="close-btn" on:click={() => dispatch('close')}>
      <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
        <path d="M4.293 4.293a1 1 0 011.414 0L8 6.586l2.293-2.293a1 1 0 111.414 1.414L9.414 8l2.293 2.293a1 1 0 01-1.414 1.414L8 9.414l-2.293 2.293a1 1 0 01-1.414-1.414L6.586 8 4.293 5.707a1 1 0 010-1.414z"/>
      </svg>
    </button>
  </div>

  {#if showFilters}
    <SearchFiltersPanel
      bind:filters
      on:change={search}
    />
  {/if}

  <div class="spec-search__results">
    {#if isSearching}
      <div class="search-status">Searching...</div>
    {:else if query && results.length === 0}
      <div class="search-status">No results found</div>
    {:else if !query && history.length > 0}
      <div class="search-history">
        <h4>Recent Searches</h4>
        {#each history as item}
          <button
            class="history-item"
            on:click={() => selectFromHistory(item.query)}
          >
            <span class="history-query">{item.query}</span>
            <span class="history-count">{item.resultCount} results</span>
          </button>
        {/each}
      </div>
    {:else}
      {#each results as result, index}
        <SearchResultItem
          {result}
          selected={index === selectedIndex}
          on:click={() => dispatch('select', result.specId)}
        />
      {/each}
    {/if}
  </div>

  <div class="spec-search__footer">
    <span class="search-tip">
      <kbd>↑↓</kbd> navigate <kbd>Enter</kbd> open <kbd>Esc</kbd> close
    </span>
    {#if results.length > 0}
      <span class="result-count">{results.length} results</span>
    {/if}
  </div>
</div>

<style>
  .spec-search {
    display: flex;
    flex-direction: column;
    height: 100%;
    max-height: 500px;
    background: var(--color-bg-base);
    border: 1px solid var(--color-border-default);
    border-radius: 12px;
    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.2);
    overflow: hidden;
  }

  .spec-search__input-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px;
    border-bottom: 1px solid var(--color-border-default);
  }

  .spec-search__input {
    flex: 1;
    padding: 10px 14px;
    border: 1px solid var(--color-border-default);
    border-radius: 8px;
    background: var(--color-bg-surface);
    color: var(--color-fg-default);
    font-size: 15px;
  }

  .spec-search__input:focus {
    outline: none;
    border-color: var(--color-accent-fg);
  }

  .filter-toggle,
  .close-btn {
    padding: 8px;
    border: none;
    background: transparent;
    color: var(--color-fg-muted);
    border-radius: 6px;
    cursor: pointer;
  }

  .filter-toggle:hover,
  .close-btn:hover {
    background: var(--color-bg-hover);
    color: var(--color-fg-default);
  }

  .filter-toggle.active {
    color: var(--color-accent-fg);
  }

  .spec-search__results {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
  }

  .search-status {
    padding: 24px;
    text-align: center;
    color: var(--color-fg-muted);
  }

  .search-history h4 {
    font-size: 11px;
    text-transform: uppercase;
    color: var(--color-fg-muted);
    margin: 8px 12px;
  }

  .history-item {
    display: flex;
    justify-content: space-between;
    width: 100%;
    padding: 10px 12px;
    border: none;
    background: transparent;
    cursor: pointer;
    border-radius: 6px;
    text-align: left;
  }

  .history-item:hover {
    background: var(--color-bg-hover);
  }

  .history-query {
    color: var(--color-fg-default);
  }

  .history-count {
    color: var(--color-fg-muted);
    font-size: 12px;
  }

  .spec-search__footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 12px;
    border-top: 1px solid var(--color-border-default);
    background: var(--color-bg-surface);
  }

  .search-tip {
    font-size: 11px;
    color: var(--color-fg-muted);
  }

  .search-tip kbd {
    padding: 2px 4px;
    background: var(--color-bg-hover);
    border-radius: 3px;
    font-size: 10px;
  }

  .result-count {
    font-size: 12px;
    color: var(--color-fg-subtle);
  }
</style>