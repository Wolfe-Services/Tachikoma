<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { SearchResult } from '$lib/types/spec-search';

  export let result: SearchResult;
  export let selected = false;

  const dispatch = createEventDispatcher<{
    click: void;
  }>();

  function highlightText(text: string, query: string): string {
    if (!query) return text;
    
    const regex = new RegExp(`(${query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')})`, 'gi');
    return text.replace(regex, '<mark>$1</mark>');
  }

  function truncateText(text: string, maxLength = 100): string {
    if (text.length <= maxLength) return text;
    return text.substring(0, maxLength) + '...';
  }

  function getMatchTypeIcon(type: 'title' | 'content' | 'tag' | 'id'): string {
    switch (type) {
      case 'title': return '📄';
      case 'content': return '📝';
      case 'tag': return '🏷️';
      case 'id': return '🔢';
      default: return '📄';
    }
  }
</script>

<button
  class="search-result"
  class:selected
  on:click={() => dispatch('click')}
>
  <div class="search-result__header">
    <h3 class="search-result__title">
      {@html highlightText(result.title, '')}
    </h3>
    <div class="search-result__meta">
      <span class="phase">Phase {result.phase}</span>
      <span class="status status--{result.status}">{result.status}</span>
    </div>
  </div>

  <div class="search-result__path">
    {result.path}
  </div>

  {#if result.matches.length > 0}
    <div class="search-result__matches">
      {#each result.matches.slice(0, 3) as match}
        <div class="match">
          <span class="match__icon" title="{match.field} match">
            {getMatchTypeIcon(match.field)}
          </span>
          <span class="match__text">
            {@html highlightText(truncateText(match.context), match.text)}
          </span>
          {#if match.lineNumber}
            <span class="match__line">:{match.lineNumber}</span>
          {/if}
        </div>
      {/each}
      {#if result.matches.length > 3}
        <div class="match-overflow">
          +{result.matches.length - 3} more matches
        </div>
      {/if}
    </div>
  {/if}

  <div class="search-result__score">
    Score: {result.score.toFixed(2)}
  </div>
</button>

<style>
  .search-result {
    display: block;
    width: 100%;
    padding: 12px;
    border: none;
    background: transparent;
    cursor: pointer;
    border-radius: 8px;
    text-align: left;
    transition: background-color 0.15s ease;
  }

  .search-result:hover,
  .search-result.selected {
    background: var(--color-bg-hover);
  }

  .search-result.selected {
    outline: 2px solid var(--color-accent-fg);
    outline-offset: 2px;
  }

  .search-result__header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 6px;
  }

  .search-result__title {
    font-size: 16px;
    font-weight: 600;
    color: var(--color-fg-default);
    margin: 0;
    flex: 1;
    line-height: 1.3;
  }

  .search-result__title :global(mark) {
    background: var(--color-accent-subtle);
    color: var(--color-accent-fg);
    padding: 1px 2px;
    border-radius: 2px;
  }

  .search-result__meta {
    display: flex;
    gap: 8px;
    flex-shrink: 0;
    margin-left: 12px;
  }

  .phase {
    font-size: 11px;
    padding: 2px 6px;
    background: var(--color-bg-elevated);
    color: var(--color-fg-muted);
    border-radius: 4px;
    text-transform: uppercase;
    font-weight: 500;
  }

  .status {
    font-size: 11px;
    padding: 2px 6px;
    border-radius: 4px;
    text-transform: uppercase;
    font-weight: 500;
  }

  .status--planned {
    background: var(--color-neutral-subtle);
    color: var(--color-neutral-fg);
  }

  .status--in_progress {
    background: var(--color-attention-subtle);
    color: var(--color-attention-fg);
  }

  .status--complete {
    background: var(--color-success-subtle);
    color: var(--color-success-fg);
  }

  .search-result__path {
    font-size: 12px;
    color: var(--color-fg-muted);
    font-family: var(--font-mono);
    margin-bottom: 8px;
  }

  .search-result__matches {
    margin-bottom: 8px;
  }

  .match {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
    margin-bottom: 4px;
    color: var(--color-fg-subtle);
  }

  .match__icon {
    font-size: 12px;
    flex-shrink: 0;
  }

  .match__text {
    flex: 1;
    min-width: 0;
    overflow: hidden;
  }

  .match__text :global(mark) {
    background: var(--color-accent-subtle);
    color: var(--color-accent-fg);
    padding: 1px 2px;
    border-radius: 2px;
  }

  .match__line {
    font-size: 11px;
    color: var(--color-fg-muted);
    font-family: var(--font-mono);
  }

  .match-overflow {
    font-size: 11px;
    color: var(--color-fg-muted);
    font-style: italic;
    margin-top: 4px;
  }

  .search-result__score {
    font-size: 11px;
    color: var(--color-fg-muted);
    text-align: right;
  }

  /* Mobile responsive */
  @media (max-width: 768px) {
    .search-result__header {
      flex-direction: column;
      align-items: flex-start;
      gap: 6px;
    }

    .search-result__meta {
      margin-left: 0;
    }

    .match {
      flex-wrap: wrap;
    }

    .match__line {
      order: -1;
      margin-right: auto;
    }
  }

  /* High contrast mode */
  @media (prefers-contrast: high) {
    .search-result.selected {
      background: var(--color-accent-emphasis);
      color: var(--color-accent-fg);
    }

    .status {
      border: 1px solid currentColor;
    }
  }
</style>