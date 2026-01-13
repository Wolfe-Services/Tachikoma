# 244 - Spec Search Results

**Phase:** 11 - Spec Browser UI
**Spec ID:** 244
**Status:** Planned
**Dependencies:** 243-spec-search-ui
**Estimated Context:** ~9% of Sonnet window

---

## Objective

Create a search result display component that shows matching specifications with context highlighting, match type indicators, and quick preview capability.

---

## Acceptance Criteria

- [x] Display matching specs with highlights
- [x] Show match context (surrounding text)
- [x] Indicate match type (title/content/tag)
- [x] Quick preview on hover
- [x] Relevance score indicator
- [x] Keyboard selection support

---

## Implementation Details

### 1. Search Result Item Component (src/lib/components/spec-browser/SearchResultItem.svelte)

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { SearchResult, SearchMatch } from '$lib/types/spec-search';

  export let result: SearchResult;
  export let selected = false;

  const dispatch = createEventDispatcher<{ click: void }>();

  const matchTypeLabels = {
    title: 'Title',
    content: 'Content',
    tag: 'Tag',
    id: 'ID',
  };

  const matchTypeColors = {
    title: 'var(--color-primary)',
    content: 'var(--color-success)',
    tag: 'var(--color-warning)',
    id: 'var(--color-text-muted)',
  };

  function highlightMatch(text: string, query: string): string {
    if (!query) return text;
    const regex = new RegExp(`(${query})`, 'gi');
    return text.replace(regex, '<mark>$1</mark>');
  }
</script>

<button
  class="search-result-item"
  class:search-result-item--selected={selected}
  on:click={() => dispatch('click')}
>
  <div class="search-result-item__header">
    <span class="search-result-item__number">
      {String(result.specId).padStart(3, '0')}
    </span>
    <span class="search-result-item__title">
      {@html highlightMatch(result.title, '')}
    </span>
    <span class="search-result-item__phase">Phase {result.phase}</span>
  </div>

  <div class="search-result-item__matches">
    {#each result.matches.slice(0, 3) as match}
      <div class="match-item">
        <span
          class="match-type"
          style="color: {matchTypeColors[match.field]}"
        >
          {matchTypeLabels[match.field]}
        </span>
        <span class="match-context">
          {@html highlightMatch(match.context, match.text)}
        </span>
        {#if match.lineNumber}
          <span class="match-line">L{match.lineNumber}</span>
        {/if}
      </div>
    {/each}
    {#if result.matches.length > 3}
      <span class="more-matches">
        +{result.matches.length - 3} more matches
      </span>
    {/if}
  </div>

  <div class="search-result-item__meta">
    <span class="status-badge status-badge--{result.status}">
      {result.status}
    </span>
    <span class="relevance-score" title="Relevance score">
      {Math.round(result.score * 100)}%
    </span>
  </div>
</button>

<style>
  .search-result-item {
    display: block;
    width: 100%;
    padding: 12px;
    border: none;
    background: transparent;
    border-radius: 8px;
    cursor: pointer;
    text-align: left;
    transition: background-color 0.1s ease;
  }

  .search-result-item:hover {
    background: var(--color-bg-hover);
  }

  .search-result-item--selected {
    background: var(--color-bg-active);
    outline: 2px solid var(--color-primary);
  }

  .search-result-item__header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 8px;
  }

  .search-result-item__number {
    font-family: monospace;
    font-size: 12px;
    color: var(--color-primary);
    padding: 2px 6px;
    background: var(--color-bg-secondary);
    border-radius: 4px;
  }

  .search-result-item__title {
    flex: 1;
    font-size: 14px;
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .search-result-item__title :global(mark) {
    background: rgba(255, 235, 59, 0.4);
    color: inherit;
    padding: 0 2px;
    border-radius: 2px;
  }

  .search-result-item__phase {
    font-size: 11px;
    color: var(--color-text-muted);
  }

  .search-result-item__matches {
    margin-bottom: 8px;
  }

  .match-item {
    display: flex;
    align-items: baseline;
    gap: 8px;
    padding: 4px 0;
    font-size: 12px;
  }

  .match-type {
    font-weight: 500;
    text-transform: uppercase;
    font-size: 10px;
    min-width: 48px;
  }

  .match-context {
    flex: 1;
    color: var(--color-text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .match-context :global(mark) {
    background: rgba(255, 235, 59, 0.4);
    color: var(--color-text-primary);
    padding: 0 2px;
    border-radius: 2px;
  }

  .match-line {
    font-family: monospace;
    font-size: 10px;
    color: var(--color-text-muted);
  }

  .more-matches {
    font-size: 11px;
    color: var(--color-text-muted);
    font-style: italic;
  }

  .search-result-item__meta {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .status-badge {
    font-size: 10px;
    padding: 2px 6px;
    border-radius: 4px;
    text-transform: uppercase;
    font-weight: 500;
  }

  .status-badge--planned {
    background: var(--color-bg-hover);
    color: var(--color-text-muted);
  }

  .status-badge--in_progress {
    background: rgba(33, 150, 243, 0.1);
    color: var(--color-primary);
  }

  .status-badge--complete {
    background: rgba(76, 175, 80, 0.1);
    color: var(--color-success);
  }

  .relevance-score {
    margin-left: auto;
    font-size: 11px;
    color: var(--color-text-muted);
  }
</style>
```

---

## Testing Requirements

1. Results render correctly
2. Highlighting works
3. Match types display
4. Selection styling works
5. Click triggers navigation

---

## Related Specs

- Depends on: [243-spec-search-ui.md](243-spec-search-ui.md)
- Next: [245-spec-quick-nav.md](245-spec-quick-nav.md)
