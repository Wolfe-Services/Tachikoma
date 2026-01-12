# Spec 234: Spec Search

## Phase
11 - Spec Browser UI

## Spec ID
234

## Status
Planned

## Dependencies
- Spec 231 (Spec List Layout)
- Spec 233 (Spec Filter System)

## Estimated Context
~9%

---

## Objective

Implement a powerful search system for the Spec Browser with full-text search, advanced query syntax, search highlighting, recent searches, and search suggestions. The search should integrate seamlessly with the filter system.

---

## Acceptance Criteria

- [ ] Full-text search across spec content
- [ ] Support for field-specific queries (id:, title:, status:)
- [ ] Search highlighting in results
- [ ] Debounced search input
- [ ] Recent search history
- [ ] Search suggestions/autocomplete
- [ ] Keyboard shortcuts for search focus
- [ ] Clear search results count
- [ ] Combine search with active filters

---

## Implementation Details

### SpecSearch.svelte

```svelte
<script lang="ts">
  import { createEventDispatcher, onMount, onDestroy } from 'svelte';
  import { writable, derived } from 'svelte/store';
  import { fade, slide } from 'svelte/transition';
  import type { Spec, SearchResult, SearchSuggestion } from '$lib/types/spec';
  import Icon from '$lib/components/Icon.svelte';
  import Kbd from '$lib/components/Kbd.svelte';
  import { debounce } from '$lib/utils/timing';
  import { parseSearchQuery, executeSearch, highlightMatches } from '$lib/utils/search';
  import { specStore } from '$lib/stores/spec-store';

  export let value = '';
  export let placeholder = 'Search specs...';
  export let maxRecentSearches = 10;
  export let showSuggestions = true;

  const dispatch = createEventDispatcher<{
    search: { query: string; results: SearchResult[] };
    clear: void;
    select: SearchResult;
  }>();

  let inputRef: HTMLInputElement;
  let isOpen = false;
  let selectedIndex = -1;
  let recentSearches = writable<string[]>([]);
  let suggestions = writable<SearchSuggestion[]>([]);

  // Load recent searches from localStorage
  onMount(() => {
    const stored = localStorage.getItem('spec-search-history');
    if (stored) {
      recentSearches.set(JSON.parse(stored));
    }

    // Global keyboard shortcut
    const handleKeydown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault();
        inputRef?.focus();
      }
    };

    document.addEventListener('keydown', handleKeydown);
    return () => document.removeEventListener('keydown', handleKeydown);
  });

  // Save recent searches
  function saveRecentSearch(query: string) {
    if (!query.trim()) return;

    recentSearches.update(searches => {
      const filtered = searches.filter(s => s !== query);
      const updated = [query, ...filtered].slice(0, maxRecentSearches);
      localStorage.setItem('spec-search-history', JSON.stringify(updated));
      return updated;
    });
  }

  // Debounced search execution
  const debouncedSearch = debounce((query: string) => {
    if (!query.trim()) {
      dispatch('clear');
      suggestions.set([]);
      return;
    }

    const parsedQuery = parseSearchQuery(query);
    const results = executeSearch($specStore, parsedQuery);

    dispatch('search', { query, results });

    // Generate suggestions based on partial matches
    if (showSuggestions && query.length >= 2) {
      const suggestionList = generateSuggestions(query, $specStore);
      suggestions.set(suggestionList);
    }
  }, 200);

  function generateSuggestions(query: string, specs: Spec[]): SearchSuggestion[] {
    const suggestions: SearchSuggestion[] = [];
    const lowerQuery = query.toLowerCase();

    // Suggest field-specific searches
    if (!query.includes(':')) {
      suggestions.push(
        { type: 'field', text: `id:${query}`, description: 'Search by spec ID' },
        { type: 'field', text: `title:${query}`, description: 'Search in titles' },
        { type: 'field', text: `status:${query}`, description: 'Filter by status' },
        { type: 'field', text: `phase:${query}`, description: 'Filter by phase' },
      );
    }

    // Suggest matching spec titles
    const matchingSpecs = specs
      .filter(s => s.title.toLowerCase().includes(lowerQuery))
      .slice(0, 5);

    matchingSpecs.forEach(spec => {
      suggestions.push({
        type: 'spec',
        text: spec.title,
        specId: spec.id,
        description: `Spec ${spec.id}`
      });
    });

    // Suggest matching tags
    const allTags = new Set<string>();
    specs.forEach(s => s.tags?.forEach(t => allTags.add(t)));

    const matchingTags = Array.from(allTags)
      .filter(t => t.toLowerCase().includes(lowerQuery))
      .slice(0, 3);

    matchingTags.forEach(tag => {
      suggestions.push({
        type: 'tag',
        text: `tag:${tag}`,
        description: `Filter by tag "${tag}"`
      });
    });

    return suggestions.slice(0, 8);
  }

  function handleInput(e: Event) {
    const target = e.target as HTMLInputElement;
    value = target.value;
    selectedIndex = -1;
    debouncedSearch(value);
  }

  function handleFocus() {
    isOpen = true;
  }

  function handleBlur(e: FocusEvent) {
    // Delay to allow click events on suggestions
    setTimeout(() => {
      if (!e.relatedTarget?.closest('.spec-search__dropdown')) {
        isOpen = false;
      }
    }, 150);
  }

  function handleKeydown(e: KeyboardEvent) {
    const items = [...$suggestions, ...$recentSearches.map(s => ({ type: 'recent', text: s }))];

    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        selectedIndex = Math.min(selectedIndex + 1, items.length - 1);
        break;
      case 'ArrowUp':
        e.preventDefault();
        selectedIndex = Math.max(selectedIndex - 1, -1);
        break;
      case 'Enter':
        e.preventDefault();
        if (selectedIndex >= 0 && items[selectedIndex]) {
          selectSuggestion(items[selectedIndex]);
        } else {
          executeFullSearch();
        }
        break;
      case 'Escape':
        isOpen = false;
        inputRef?.blur();
        break;
    }
  }

  function selectSuggestion(suggestion: SearchSuggestion | { type: string; text: string }) {
    if (suggestion.type === 'spec' && 'specId' in suggestion) {
      const spec = $specStore.find(s => s.id === suggestion.specId);
      if (spec) {
        dispatch('select', { spec, matches: [], score: 1 });
      }
    } else {
      value = suggestion.text;
      executeFullSearch();
    }
    isOpen = false;
  }

  function executeFullSearch() {
    if (!value.trim()) return;

    saveRecentSearch(value);
    const parsedQuery = parseSearchQuery(value);
    const results = executeSearch($specStore, parsedQuery);
    dispatch('search', { query: value, results });
    isOpen = false;
  }

  function clearSearch() {
    value = '';
    dispatch('clear');
    suggestions.set([]);
    inputRef?.focus();
  }

  function clearHistory() {
    recentSearches.set([]);
    localStorage.removeItem('spec-search-history');
  }
</script>

<div class="spec-search" class:spec-search--open={isOpen}>
  <div class="spec-search__input-wrapper">
    <Icon name="search" size={16} class="spec-search__icon" />
    <input
      bind:this={inputRef}
      type="text"
      class="spec-search__input"
      {placeholder}
      {value}
      on:input={handleInput}
      on:focus={handleFocus}
      on:blur={handleBlur}
      on:keydown={handleKeydown}
      role="combobox"
      aria-expanded={isOpen}
      aria-haspopup="listbox"
      aria-autocomplete="list"
    />
    {#if value}
      <button
        class="spec-search__clear"
        on:click={clearSearch}
        aria-label="Clear search"
      >
        <Icon name="x" size={14} />
      </button>
    {:else}
      <Kbd keys={['⌘', 'K']} class="spec-search__shortcut" />
    {/if}
  </div>

  {#if isOpen && (value || $recentSearches.length > 0)}
    <div
      class="spec-search__dropdown"
      role="listbox"
      transition:slide={{ duration: 150 }}
    >
      {#if $suggestions.length > 0}
        <div class="spec-search__section">
          <h4 class="spec-search__section-title">Suggestions</h4>
          <ul class="spec-search__list">
            {#each $suggestions as suggestion, i}
              <li>
                <button
                  class="spec-search__item"
                  class:spec-search__item--selected={selectedIndex === i}
                  on:click={() => selectSuggestion(suggestion)}
                  role="option"
                  aria-selected={selectedIndex === i}
                >
                  <Icon
                    name={suggestion.type === 'spec' ? 'file-text' :
                          suggestion.type === 'tag' ? 'tag' : 'search'}
                    size={14}
                  />
                  <span class="spec-search__item-text">
                    {@html highlightMatches(suggestion.text, value)}
                  </span>
                  {#if suggestion.description}
                    <span class="spec-search__item-desc">{suggestion.description}</span>
                  {/if}
                </button>
              </li>
            {/each}
          </ul>
        </div>
      {/if}

      {#if !value && $recentSearches.length > 0}
        <div class="spec-search__section">
          <div class="spec-search__section-header">
            <h4 class="spec-search__section-title">Recent Searches</h4>
            <button
              class="spec-search__clear-history"
              on:click={clearHistory}
            >
              Clear
            </button>
          </div>
          <ul class="spec-search__list">
            {#each $recentSearches as search, i}
              <li>
                <button
                  class="spec-search__item"
                  class:spec-search__item--selected={selectedIndex === $suggestions.length + i}
                  on:click={() => selectSuggestion({ type: 'recent', text: search })}
                  role="option"
                >
                  <Icon name="clock" size={14} />
                  <span class="spec-search__item-text">{search}</span>
                </button>
              </li>
            {/each}
          </ul>
        </div>
      {/if}

      <div class="spec-search__help">
        <span>Use <code>field:</code> for specific searches</span>
        <span>Press <Kbd keys={['Enter']} /> to search</span>
      </div>
    </div>
  {/if}
</div>

<style>
  .spec-search {
    position: relative;
    width: 100%;
    max-width: 480px;
  }

  .spec-search__input-wrapper {
    position: relative;
    display: flex;
    align-items: center;
  }

  .spec-search :global(.spec-search__icon) {
    position: absolute;
    left: 12px;
    color: var(--color-text-tertiary);
    pointer-events: none;
  }

  .spec-search__input {
    width: 100%;
    padding: 8px 36px 8px 36px;
    font-size: 0.875rem;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    background: var(--color-surface);
    color: var(--color-text-primary);
    transition:
      border-color 0.15s ease,
      box-shadow 0.15s ease;
  }

  .spec-search__input:focus {
    outline: none;
    border-color: var(--color-primary);
    box-shadow: 0 0 0 3px var(--color-primary-alpha);
  }

  .spec-search__input::placeholder {
    color: var(--color-text-tertiary);
  }

  .spec-search__clear {
    position: absolute;
    right: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    padding: 0;
    background: var(--color-surface-elevated);
    border: none;
    border-radius: 4px;
    cursor: pointer;
    color: var(--color-text-tertiary);
  }

  .spec-search__clear:hover {
    background: var(--color-hover);
    color: var(--color-text-primary);
  }

  .spec-search :global(.spec-search__shortcut) {
    position: absolute;
    right: 8px;
    pointer-events: none;
  }

  .spec-search__dropdown {
    position: absolute;
    top: calc(100% + 4px);
    left: 0;
    right: 0;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 8px;
    box-shadow: var(--shadow-lg);
    overflow: hidden;
    z-index: 100;
  }

  .spec-search__section {
    padding: 8px 0;
    border-bottom: 1px solid var(--color-border);
  }

  .spec-search__section:last-of-type {
    border-bottom: none;
  }

  .spec-search__section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0 12px 4px;
  }

  .spec-search__section-title {
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--color-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin: 0;
    padding: 0 12px 4px;
  }

  .spec-search__section-header .spec-search__section-title {
    padding: 0;
  }

  .spec-search__clear-history {
    font-size: 0.75rem;
    color: var(--color-text-tertiary);
    background: none;
    border: none;
    cursor: pointer;
  }

  .spec-search__clear-history:hover {
    color: var(--color-primary);
  }

  .spec-search__list {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .spec-search__item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 8px 12px;
    text-align: left;
    background: none;
    border: none;
    cursor: pointer;
    color: var(--color-text-primary);
    font-size: 0.875rem;
  }

  .spec-search__item:hover,
  .spec-search__item--selected {
    background: var(--color-hover);
  }

  .spec-search__item-text {
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .spec-search__item-text :global(mark) {
    background: var(--color-highlight);
    color: inherit;
    padding: 0 2px;
    border-radius: 2px;
  }

  .spec-search__item-desc {
    font-size: 0.75rem;
    color: var(--color-text-tertiary);
  }

  .spec-search__help {
    display: flex;
    justify-content: space-between;
    padding: 8px 12px;
    font-size: 0.75rem;
    color: var(--color-text-tertiary);
    background: var(--color-surface-subtle);
  }

  .spec-search__help code {
    padding: 1px 4px;
    background: var(--color-surface);
    border-radius: 3px;
    font-family: var(--font-mono);
  }
</style>
```

### Search Utilities

```typescript
// utils/search.ts
import type { Spec, SearchResult, ParsedQuery, SearchField } from '$lib/types/spec';

export interface ParsedQuery {
  terms: string[];
  fields: Map<SearchField, string>;
  exact: string[];
}

export type SearchField = 'id' | 'title' | 'status' | 'phase' | 'tag' | 'content';

export function parseSearchQuery(query: string): ParsedQuery {
  const result: ParsedQuery = {
    terms: [],
    fields: new Map(),
    exact: []
  };

  // Match exact phrases in quotes
  const exactRegex = /"([^"]+)"/g;
  let match;
  while ((match = exactRegex.exec(query)) !== null) {
    result.exact.push(match[1]);
  }
  query = query.replace(exactRegex, '');

  // Match field:value patterns
  const fieldRegex = /(\w+):(\S+)/g;
  while ((match = fieldRegex.exec(query)) !== null) {
    const field = match[1].toLowerCase() as SearchField;
    if (['id', 'title', 'status', 'phase', 'tag', 'content'].includes(field)) {
      result.fields.set(field, match[2]);
    }
  }
  query = query.replace(fieldRegex, '');

  // Remaining terms
  result.terms = query.split(/\s+/).filter(t => t.length > 0);

  return result;
}

export function executeSearch(specs: Spec[], query: ParsedQuery): SearchResult[] {
  const results: SearchResult[] = [];

  for (const spec of specs) {
    const matches: { field: string; indices: [number, number][] }[] = [];
    let score = 0;

    // Check field-specific queries
    for (const [field, value] of query.fields) {
      const fieldScore = matchField(spec, field, value);
      if (fieldScore === 0) {
        score = -1; // Exclude if field doesn't match
        break;
      }
      score += fieldScore * 2; // Field matches are weighted higher
    }

    if (score === -1) continue;

    // Check exact phrases
    for (const phrase of query.exact) {
      const content = `${spec.title} ${spec.description || ''} ${spec.content}`;
      const index = content.toLowerCase().indexOf(phrase.toLowerCase());
      if (index === -1) {
        score = -1;
        break;
      }
      score += 3;
      matches.push({ field: 'content', indices: [[index, index + phrase.length]] });
    }

    if (score === -1) continue;

    // Check general terms
    for (const term of query.terms) {
      const termLower = term.toLowerCase();

      // Check title (highest weight)
      const titleIndex = spec.title.toLowerCase().indexOf(termLower);
      if (titleIndex !== -1) {
        score += 3;
        matches.push({ field: 'title', indices: [[titleIndex, titleIndex + term.length]] });
      }

      // Check ID
      if (spec.id.toLowerCase().includes(termLower)) {
        score += 2;
      }

      // Check description
      if (spec.description?.toLowerCase().includes(termLower)) {
        score += 1;
      }

      // Check content
      const contentIndex = spec.content.toLowerCase().indexOf(termLower);
      if (contentIndex !== -1) {
        score += 0.5;
        matches.push({ field: 'content', indices: [[contentIndex, contentIndex + term.length]] });
      }

      // Check tags
      if (spec.tags?.some(t => t.toLowerCase().includes(termLower))) {
        score += 1;
      }
    }

    if (score > 0) {
      results.push({ spec, matches, score });
    }
  }

  // Sort by score descending
  return results.sort((a, b) => b.score - a.score);
}

function matchField(spec: Spec, field: SearchField, value: string): number {
  const valueLower = value.toLowerCase();

  switch (field) {
    case 'id':
      return spec.id.toLowerCase().includes(valueLower) ? 1 : 0;
    case 'title':
      return spec.title.toLowerCase().includes(valueLower) ? 1 : 0;
    case 'status':
      return spec.status.toLowerCase() === valueLower ? 1 : 0;
    case 'phase':
      return spec.phase.toString() === value ? 1 : 0;
    case 'tag':
      return spec.tags?.some(t => t.toLowerCase() === valueLower) ? 1 : 0;
    case 'content':
      return spec.content.toLowerCase().includes(valueLower) ? 1 : 0;
    default:
      return 0;
  }
}

export function highlightMatches(text: string, query: string): string {
  if (!query) return text;

  const terms = query.split(/\s+/).filter(t => t.length > 0 && !t.includes(':'));
  let result = text;

  for (const term of terms) {
    const regex = new RegExp(`(${escapeRegExp(term)})`, 'gi');
    result = result.replace(regex, '<mark>$1</mark>');
  }

  return result;
}

function escapeRegExp(string: string): string {
  return string.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

export function getSearchSnippet(
  content: string,
  query: string,
  maxLength = 150
): string {
  const terms = query.split(/\s+/).filter(t => t.length > 0);
  const contentLower = content.toLowerCase();

  // Find the first matching term
  let startIndex = 0;
  for (const term of terms) {
    const index = contentLower.indexOf(term.toLowerCase());
    if (index !== -1) {
      startIndex = Math.max(0, index - 40);
      break;
    }
  }

  let snippet = content.slice(startIndex, startIndex + maxLength);

  // Add ellipsis
  if (startIndex > 0) {
    snippet = '...' + snippet;
  }
  if (startIndex + maxLength < content.length) {
    snippet = snippet + '...';
  }

  return highlightMatches(snippet, query);
}
```

### Search Types

```typescript
// types/spec.ts additions
export interface SearchResult {
  spec: Spec;
  matches: {
    field: string;
    indices: [number, number][];
  }[];
  score: number;
}

export interface SearchSuggestion {
  type: 'field' | 'spec' | 'tag' | 'recent';
  text: string;
  specId?: string;
  description?: string;
}
```

---

## Testing Requirements

### Unit Tests

```typescript
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import SpecSearch from './SpecSearch.svelte';
import { parseSearchQuery, executeSearch, highlightMatches } from '$lib/utils/search';
import { createMockSpecs } from '$lib/test-utils/mock-data';

describe('SpecSearch', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('renders search input with placeholder', () => {
    render(SpecSearch);

    expect(screen.getByPlaceholderText('Search specs...')).toBeInTheDocument();
  });

  it('shows keyboard shortcut when empty', () => {
    render(SpecSearch);

    expect(screen.getByText('K')).toBeInTheDocument();
  });

  it('shows clear button when has value', async () => {
    render(SpecSearch, { props: { value: 'test' } });

    expect(screen.getByLabelText('Clear search')).toBeInTheDocument();
  });

  it('debounces search input', async () => {
    const { component } = render(SpecSearch);

    const searchHandler = vi.fn();
    component.$on('search', searchHandler);

    const input = screen.getByRole('combobox');
    await fireEvent.input(input, { target: { value: 'test' } });

    // Should not fire immediately
    expect(searchHandler).not.toHaveBeenCalled();

    // Wait for debounce
    await waitFor(() => {
      expect(searchHandler).toHaveBeenCalled();
    }, { timeout: 300 });
  });

  it('shows recent searches dropdown', async () => {
    localStorage.setItem('spec-search-history', JSON.stringify(['previous search']));

    render(SpecSearch);
    const input = screen.getByRole('combobox');
    await fireEvent.focus(input);

    expect(screen.getByText('Recent Searches')).toBeInTheDocument();
    expect(screen.getByText('previous search')).toBeInTheDocument();
  });

  it('handles keyboard navigation', async () => {
    localStorage.setItem('spec-search-history', JSON.stringify(['search1', 'search2']));

    render(SpecSearch);
    const input = screen.getByRole('combobox');
    await fireEvent.focus(input);
    await fireEvent.keyDown(input, { key: 'ArrowDown' });
    await fireEvent.keyDown(input, { key: 'ArrowDown' });

    const items = screen.getAllByRole('option');
    expect(items[1]).toHaveClass('spec-search__item--selected');
  });
});

describe('parseSearchQuery', () => {
  it('parses simple terms', () => {
    const result = parseSearchQuery('hello world');

    expect(result.terms).toEqual(['hello', 'world']);
    expect(result.fields.size).toBe(0);
    expect(result.exact).toEqual([]);
  });

  it('parses field queries', () => {
    const result = parseSearchQuery('status:planned phase:11');

    expect(result.fields.get('status')).toBe('planned');
    expect(result.fields.get('phase')).toBe('11');
  });

  it('parses exact phrases', () => {
    const result = parseSearchQuery('"exact phrase" other');

    expect(result.exact).toEqual(['exact phrase']);
    expect(result.terms).toEqual(['other']);
  });

  it('handles mixed query', () => {
    const result = parseSearchQuery('status:in-progress "user auth" login');

    expect(result.fields.get('status')).toBe('in-progress');
    expect(result.exact).toEqual(['user auth']);
    expect(result.terms).toEqual(['login']);
  });
});

describe('executeSearch', () => {
  const specs = createMockSpecs(10);

  it('returns results sorted by score', () => {
    const query = parseSearchQuery('component');
    const results = executeSearch(specs, query);

    expect(results.length).toBeGreaterThan(0);
    expect(results[0].score).toBeGreaterThanOrEqual(results[1]?.score ?? 0);
  });

  it('filters by status field', () => {
    const query = parseSearchQuery('status:planned');
    const results = executeSearch(specs, query);

    expect(results.every(r => r.spec.status === 'planned')).toBe(true);
  });

  it('returns empty for non-matching query', () => {
    const query = parseSearchQuery('xyznonexistent123');
    const results = executeSearch(specs, query);

    expect(results).toEqual([]);
  });
});

describe('highlightMatches', () => {
  it('wraps matching terms in mark tags', () => {
    const result = highlightMatches('Hello World', 'world');

    expect(result).toBe('Hello <mark>World</mark>');
  });

  it('handles multiple matches', () => {
    const result = highlightMatches('foo bar foo', 'foo');

    expect(result).toBe('<mark>foo</mark> bar <mark>foo</mark>');
  });

  it('returns original text for empty query', () => {
    const result = highlightMatches('Hello World', '');

    expect(result).toBe('Hello World');
  });
});
```

---

## Related Specs

- Spec 231: Spec List Layout
- Spec 233: Spec Filter System
- Spec 235: Spec Sort
- Spec 236: Spec Detail View
