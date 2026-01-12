# 245 - Spec Quick Navigation

**Phase:** 11 - Spec Browser UI
**Spec ID:** 245
**Status:** Planned
**Dependencies:** 236-spec-browser-layout, 243-spec-search-ui
**Estimated Context:** ~9% of Sonnet window

---

## Objective

Create a quick navigation component (command palette style) that allows rapid navigation between specs using keyboard shortcuts and fuzzy search.

---

## Acceptance Criteria

- [ ] Trigger with Cmd+K or Cmd+P
- [ ] Fuzzy search by title or number
- [ ] Recently opened specs at top
- [ ] Keyboard navigation
- [ ] Preview on hover
- [ ] Direct number input (e.g., "216")

---

## Implementation Details

### 1. Quick Nav Component (src/lib/components/spec-browser/SpecQuickNav.svelte)

```svelte
<script lang="ts">
  import { createEventDispatcher, onMount, onDestroy } from 'svelte';
  import { fade, fly } from 'svelte/transition';
  import type { SearchResult } from '$lib/types/spec-search';
  import { ipcRenderer } from '$lib/ipc';

  export let open = false;

  const dispatch = createEventDispatcher<{
    navigate: string;
    close: void;
  }>();

  let query = '';
  let results: SearchResult[] = [];
  let recentSpecs: { id: string; title: string }[] = [];
  let selectedIndex = 0;
  let inputRef: HTMLInputElement;

  async function search() {
    if (!query.trim()) {
      results = [];
      return;
    }

    // Check for direct number input
    const numberMatch = query.match(/^(\d+)$/);
    if (numberMatch) {
      results = await ipcRenderer.invoke('spec:search', {
        text: numberMatch[1],
        filters: {},
      });
    } else {
      results = await ipcRenderer.invoke('spec:search', {
        text: query,
        filters: {},
      });
    }

    selectedIndex = 0;
  }

  function handleKeyDown(event: KeyboardEvent) {
    switch (event.key) {
      case 'ArrowDown':
        event.preventDefault();
        const maxIndex = query ? results.length - 1 : recentSpecs.length - 1;
        selectedIndex = Math.min(selectedIndex + 1, maxIndex);
        break;
      case 'ArrowUp':
        event.preventDefault();
        selectedIndex = Math.max(selectedIndex - 1, 0);
        break;
      case 'Enter':
        event.preventDefault();
        selectItem();
        break;
      case 'Escape':
        dispatch('close');
        break;
    }
  }

  function selectItem() {
    if (query && results[selectedIndex]) {
      navigateTo(results[selectedIndex].specId);
    } else if (!query && recentSpecs[selectedIndex]) {
      navigateTo(recentSpecs[selectedIndex].id);
    }
  }

  function navigateTo(specId: string) {
    addToRecent(specId);
    dispatch('navigate', specId);
    dispatch('close');
  }

  function addToRecent(specId: string) {
    const spec = results.find(r => r.specId === specId) ||
                 recentSpecs.find(r => r.id === specId);
    if (spec) {
      const entry = { id: specId, title: 'title' in spec ? spec.title : '' };
      recentSpecs = [entry, ...recentSpecs.filter(r => r.id !== specId)].slice(0, 5);
      localStorage.setItem('spec-recent', JSON.stringify(recentSpecs));
    }
  }

  function loadRecent() {
    const saved = localStorage.getItem('spec-recent');
    if (saved) {
      recentSpecs = JSON.parse(saved);
    }
  }

  function handleGlobalKeyDown(event: KeyboardEvent) {
    if ((event.metaKey || event.ctrlKey) && (event.key === 'k' || event.key === 'p')) {
      event.preventDefault();
      open = !open;
      if (open) {
        setTimeout(() => inputRef?.focus(), 0);
      }
    }
  }

  $: if (open) {
    query = '';
    results = [];
    selectedIndex = 0;
    loadRecent();
  }

  onMount(() => {
    window.addEventListener('keydown', handleGlobalKeyDown);
    loadRecent();
  });

  onDestroy(() => {
    window.removeEventListener('keydown', handleGlobalKeyDown);
  });
</script>

{#if open}
  <div
    class="quick-nav-overlay"
    on:click={() => dispatch('close')}
    transition:fade={{ duration: 100 }}
  >
    <div
      class="quick-nav"
      on:click|stopPropagation
      on:keydown={handleKeyDown}
      transition:fly={{ y: -20, duration: 150 }}
    >
      <input
        bind:this={inputRef}
        type="text"
        class="quick-nav__input"
        placeholder="Go to spec... (type number or search)"
        bind:value={query}
        on:input={search}
      />

      <div class="quick-nav__results">
        {#if query}
          {#if results.length === 0}
            <div class="quick-nav__empty">No specs found</div>
          {:else}
            {#each results as result, index}
              <button
                class="quick-nav__item"
                class:quick-nav__item--selected={index === selectedIndex}
                on:click={() => navigateTo(result.specId)}
                on:mouseenter={() => { selectedIndex = index; }}
              >
                <span class="item-number">{result.specId}</span>
                <span class="item-title">{result.title}</span>
                <span class="item-phase">Phase {result.phase}</span>
              </button>
            {/each}
          {/if}
        {:else if recentSpecs.length > 0}
          <div class="quick-nav__section">Recent</div>
          {#each recentSpecs as spec, index}
            <button
              class="quick-nav__item"
              class:quick-nav__item--selected={index === selectedIndex}
              on:click={() => navigateTo(spec.id)}
              on:mouseenter={() => { selectedIndex = index; }}
            >
              <span class="item-number">{spec.id}</span>
              <span class="item-title">{spec.title}</span>
            </button>
          {/each}
        {:else}
          <div class="quick-nav__empty">
            Type a spec number or search term
          </div>
        {/if}
      </div>

      <div class="quick-nav__footer">
        <span><kbd>↑↓</kbd> navigate</span>
        <span><kbd>Enter</kbd> open</span>
        <span><kbd>Esc</kbd> close</span>
      </div>
    </div>
  </div>
{/if}

<style>
  .quick-nav-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    justify-content: center;
    padding-top: 100px;
    z-index: 1000;
  }

  .quick-nav {
    width: 500px;
    max-height: 400px;
    background: var(--color-bg-primary);
    border-radius: 12px;
    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.3);
    overflow: hidden;
  }

  .quick-nav__input {
    width: 100%;
    padding: 16px;
    border: none;
    border-bottom: 1px solid var(--color-border);
    background: transparent;
    color: var(--color-text-primary);
    font-size: 16px;
  }

  .quick-nav__input:focus {
    outline: none;
  }

  .quick-nav__results {
    max-height: 280px;
    overflow-y: auto;
    padding: 8px;
  }

  .quick-nav__section {
    padding: 8px 12px;
    font-size: 11px;
    text-transform: uppercase;
    color: var(--color-text-muted);
  }

  .quick-nav__empty {
    padding: 24px;
    text-align: center;
    color: var(--color-text-muted);
  }

  .quick-nav__item {
    display: flex;
    align-items: center;
    gap: 12px;
    width: 100%;
    padding: 10px 12px;
    border: none;
    background: transparent;
    border-radius: 6px;
    cursor: pointer;
    text-align: left;
  }

  .quick-nav__item:hover,
  .quick-nav__item--selected {
    background: var(--color-bg-hover);
  }

  .item-number {
    font-family: monospace;
    font-size: 13px;
    color: var(--color-primary);
    min-width: 40px;
  }

  .item-title {
    flex: 1;
    font-size: 14px;
    color: var(--color-text-primary);
  }

  .item-phase {
    font-size: 12px;
    color: var(--color-text-muted);
  }

  .quick-nav__footer {
    display: flex;
    gap: 16px;
    padding: 8px 16px;
    border-top: 1px solid var(--color-border);
    background: var(--color-bg-secondary);
    font-size: 11px;
    color: var(--color-text-muted);
  }

  .quick-nav__footer kbd {
    padding: 2px 4px;
    background: var(--color-bg-hover);
    border-radius: 3px;
    font-size: 10px;
  }
</style>
```

---

## Testing Requirements

1. Cmd+K opens panel
2. Fuzzy search works
3. Number input navigates directly
4. Recent specs display
5. Keyboard navigation works

---

## Related Specs

- Depends on: [243-spec-search-ui.md](243-spec-search-ui.md)
- Next: [246-spec-breadcrumbs.md](246-spec-breadcrumbs.md)
