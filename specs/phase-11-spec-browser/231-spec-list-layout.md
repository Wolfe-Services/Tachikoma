# Spec 231: Spec List Layout Component

## Phase
11 - Spec Browser UI

## Spec ID
231

## Status
Planned

## Dependencies
- Phase 10 (Core UI Components)
- Spec 201 (Base Layout System)

## Estimated Context
~10%

---

## Objective

Create a flexible list layout component for displaying specs in the Spec Browser. The layout should support multiple view modes (list, grid, compact), virtual scrolling for performance, and responsive design for various screen sizes.

---

## Acceptance Criteria

- [ ] List layout renders specs in configurable view modes
- [ ] Virtual scrolling handles 1000+ specs efficiently
- [ ] Responsive breakpoints adapt layout automatically
- [ ] Keyboard navigation works across all view modes
- [ ] Empty state displays when no specs match
- [ ] Loading skeleton shows during data fetch
- [ ] Selection state supports single and multi-select

---

## Implementation Details

### SpecListLayout.svelte

```svelte
<script lang="ts">
  import { createEventDispatcher, onMount, onDestroy } from 'svelte';
  import { writable, derived } from 'svelte/store';
  import type { Spec, ViewMode, SelectionMode } from '$lib/types/spec';
  import SpecCard from './SpecCard.svelte';
  import SpecListItem from './SpecListItem.svelte';
  import LoadingSkeleton from '$lib/components/LoadingSkeleton.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';

  export let specs: Spec[] = [];
  export let viewMode: ViewMode = 'list';
  export let selectionMode: SelectionMode = 'single';
  export let loading = false;
  export let virtualScroll = true;
  export let itemHeight = 72;
  export let gridColumns = 3;

  const dispatch = createEventDispatcher<{
    select: { spec: Spec; selected: Spec[] };
    open: { spec: Spec };
    contextmenu: { spec: Spec; event: MouseEvent };
  }>();

  let containerRef: HTMLElement;
  let scrollTop = 0;
  let containerHeight = 0;
  let selectedIds = writable<Set<string>>(new Set());

  // Virtual scroll calculations
  $: visibleCount = Math.ceil(containerHeight / itemHeight) + 2;
  $: startIndex = Math.max(0, Math.floor(scrollTop / itemHeight) - 1);
  $: endIndex = Math.min(specs.length, startIndex + visibleCount);
  $: visibleSpecs = virtualScroll
    ? specs.slice(startIndex, endIndex)
    : specs;
  $: totalHeight = specs.length * itemHeight;
  $: offsetY = startIndex * itemHeight;

  // Grid layout calculations
  $: gridItemWidth = containerRef
    ? Math.floor(containerRef.clientWidth / gridColumns) - 16
    : 300;

  function handleScroll(event: Event) {
    const target = event.target as HTMLElement;
    scrollTop = target.scrollTop;
  }

  function handleSelect(spec: Spec, event: MouseEvent) {
    if (selectionMode === 'none') return;

    selectedIds.update(ids => {
      const newIds = new Set(ids);

      if (selectionMode === 'multi' && (event.ctrlKey || event.metaKey)) {
        if (newIds.has(spec.id)) {
          newIds.delete(spec.id);
        } else {
          newIds.add(spec.id);
        }
      } else if (selectionMode === 'multi' && event.shiftKey && ids.size > 0) {
        // Range selection
        const lastSelected = Array.from(ids).pop();
        const lastIndex = specs.findIndex(s => s.id === lastSelected);
        const currentIndex = specs.findIndex(s => s.id === spec.id);
        const [start, end] = [Math.min(lastIndex, currentIndex), Math.max(lastIndex, currentIndex)];

        for (let i = start; i <= end; i++) {
          newIds.add(specs[i].id);
        }
      } else {
        newIds.clear();
        newIds.add(spec.id);
      }

      return newIds;
    });

    const selected = specs.filter(s => $selectedIds.has(s.id));
    dispatch('select', { spec, selected });
  }

  function handleOpen(spec: Spec) {
    dispatch('open', { spec });
  }

  function handleKeydown(event: KeyboardEvent) {
    if (!specs.length) return;

    const currentIndex = specs.findIndex(s => $selectedIds.has(s.id));
    let nextIndex = currentIndex;

    switch (event.key) {
      case 'ArrowDown':
        nextIndex = Math.min(specs.length - 1, currentIndex + 1);
        event.preventDefault();
        break;
      case 'ArrowUp':
        nextIndex = Math.max(0, currentIndex - 1);
        event.preventDefault();
        break;
      case 'Enter':
        if (currentIndex >= 0) {
          handleOpen(specs[currentIndex]);
        }
        break;
      case 'Home':
        nextIndex = 0;
        event.preventDefault();
        break;
      case 'End':
        nextIndex = specs.length - 1;
        event.preventDefault();
        break;
    }

    if (nextIndex !== currentIndex && nextIndex >= 0) {
      selectedIds.set(new Set([specs[nextIndex].id]));
      dispatch('select', { spec: specs[nextIndex], selected: [specs[nextIndex]] });

      // Scroll into view
      const itemTop = nextIndex * itemHeight;
      if (itemTop < scrollTop) {
        containerRef.scrollTop = itemTop;
      } else if (itemTop + itemHeight > scrollTop + containerHeight) {
        containerRef.scrollTop = itemTop - containerHeight + itemHeight;
      }
    }
  }

  onMount(() => {
    const resizeObserver = new ResizeObserver(entries => {
      for (const entry of entries) {
        containerHeight = entry.contentRect.height;
      }
    });

    if (containerRef) {
      resizeObserver.observe(containerRef);
    }

    return () => resizeObserver.disconnect();
  });

  function getViewModeClass(mode: ViewMode): string {
    return {
      list: 'spec-list--list',
      grid: 'spec-list--grid',
      compact: 'spec-list--compact'
    }[mode];
  }
</script>

<div
  bind:this={containerRef}
  class="spec-list {getViewModeClass(viewMode)}"
  class:spec-list--loading={loading}
  on:scroll={handleScroll}
  on:keydown={handleKeydown}
  tabindex="0"
  role="listbox"
  aria-label="Spec list"
  aria-multiselectable={selectionMode === 'multi'}
>
  {#if loading}
    <div class="spec-list__loading">
      {#each Array(5) as _, i}
        <LoadingSkeleton height={itemHeight} />
      {/each}
    </div>
  {:else if specs.length === 0}
    <EmptyState
      icon="document"
      title="No specs found"
      description="Try adjusting your filters or create a new spec"
    >
      <slot name="empty-action" />
    </EmptyState>
  {:else}
    {#if virtualScroll}
      <div class="spec-list__scroll-container" style="height: {totalHeight}px">
        <div class="spec-list__visible" style="transform: translateY({offsetY}px)">
          {#each visibleSpecs as spec (spec.id)}
            {#if viewMode === 'grid'}
              <SpecCard
                {spec}
                selected={$selectedIds.has(spec.id)}
                width={gridItemWidth}
                on:click={(e) => handleSelect(spec, e.detail)}
                on:dblclick={() => handleOpen(spec)}
                on:contextmenu={(e) => dispatch('contextmenu', { spec, event: e.detail })}
              />
            {:else}
              <SpecListItem
                {spec}
                selected={$selectedIds.has(spec.id)}
                compact={viewMode === 'compact'}
                on:click={(e) => handleSelect(spec, e.detail)}
                on:dblclick={() => handleOpen(spec)}
                on:contextmenu={(e) => dispatch('contextmenu', { spec, event: e.detail })}
              />
            {/if}
          {/each}
        </div>
      </div>
    {:else}
      {#each specs as spec (spec.id)}
        {#if viewMode === 'grid'}
          <SpecCard
            {spec}
            selected={$selectedIds.has(spec.id)}
            width={gridItemWidth}
            on:click={(e) => handleSelect(spec, e.detail)}
            on:dblclick={() => handleOpen(spec)}
            on:contextmenu={(e) => dispatch('contextmenu', { spec, event: e.detail })}
          />
        {:else}
          <SpecListItem
            {spec}
            selected={$selectedIds.has(spec.id)}
            compact={viewMode === 'compact'}
            on:click={(e) => handleSelect(spec, e.detail)}
            on:dblclick={() => handleOpen(spec)}
            on:contextmenu={(e) => dispatch('contextmenu', { spec, event: e.detail })}
          />
        {/if}
      {/each}
    {/if}
  {/if}
</div>

<style>
  .spec-list {
    height: 100%;
    overflow-y: auto;
    overflow-x: hidden;
    outline: none;
    position: relative;
  }

  .spec-list:focus {
    outline: 2px solid var(--color-primary);
    outline-offset: -2px;
  }

  .spec-list--list,
  .spec-list--compact {
    display: flex;
    flex-direction: column;
  }

  .spec-list--grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 16px;
    padding: 16px;
  }

  .spec-list__scroll-container {
    position: relative;
  }

  .spec-list__visible {
    position: absolute;
    left: 0;
    right: 0;
  }

  .spec-list__loading {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 16px;
  }

  /* Responsive adjustments */
  @media (max-width: 768px) {
    .spec-list--grid {
      grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
      gap: 12px;
      padding: 12px;
    }
  }

  @media (max-width: 480px) {
    .spec-list--grid {
      grid-template-columns: 1fr;
      padding: 8px;
    }
  }
</style>
```

### SpecListItem.svelte

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { Spec } from '$lib/types/spec';
  import StatusBadge from '$lib/components/StatusBadge.svelte';
  import { formatRelativeTime } from '$lib/utils/date';

  export let spec: Spec;
  export let selected = false;
  export let compact = false;

  const dispatch = createEventDispatcher<{
    click: MouseEvent;
    dblclick: void;
    contextmenu: MouseEvent;
  }>();
</script>

<div
  class="spec-list-item"
  class:spec-list-item--selected={selected}
  class:spec-list-item--compact={compact}
  role="option"
  aria-selected={selected}
  on:click={(e) => dispatch('click', e)}
  on:dblclick={() => dispatch('dblclick')}
  on:contextmenu|preventDefault={(e) => dispatch('contextmenu', e)}
>
  <div class="spec-list-item__id">{spec.id}</div>
  <div class="spec-list-item__content">
    <div class="spec-list-item__title">{spec.title}</div>
    {#if !compact && spec.description}
      <div class="spec-list-item__description">{spec.description}</div>
    {/if}
  </div>
  <div class="spec-list-item__meta">
    <StatusBadge status={spec.status} size={compact ? 'sm' : 'md'} />
    {#if !compact}
      <span class="spec-list-item__date">
        {formatRelativeTime(spec.updatedAt)}
      </span>
    {/if}
  </div>
</div>

<style>
  .spec-list-item {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 12px 16px;
    border-bottom: 1px solid var(--color-border);
    cursor: pointer;
    transition: background-color 0.15s ease;
  }

  .spec-list-item:hover {
    background-color: var(--color-hover);
  }

  .spec-list-item--selected {
    background-color: var(--color-selected);
  }

  .spec-list-item--compact {
    padding: 8px 12px;
    gap: 12px;
  }

  .spec-list-item__id {
    font-family: var(--font-mono);
    font-size: 0.875rem;
    color: var(--color-text-secondary);
    min-width: 60px;
  }

  .spec-list-item__content {
    flex: 1;
    min-width: 0;
  }

  .spec-list-item__title {
    font-weight: 500;
    color: var(--color-text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .spec-list-item__description {
    font-size: 0.875rem;
    color: var(--color-text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-top: 2px;
  }

  .spec-list-item__meta {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-shrink: 0;
  }

  .spec-list-item__date {
    font-size: 0.75rem;
    color: var(--color-text-tertiary);
  }

  .spec-list-item--compact .spec-list-item__title {
    font-size: 0.875rem;
  }
</style>
```

### Types (types/spec.ts)

```typescript
export type ViewMode = 'list' | 'grid' | 'compact';
export type SelectionMode = 'none' | 'single' | 'multi';

export interface Spec {
  id: string;
  title: string;
  description?: string;
  status: SpecStatus;
  phase: number;
  dependencies: string[];
  estimatedContext: string;
  content: string;
  createdAt: Date;
  updatedAt: Date;
  author?: string;
  tags?: string[];
}

export type SpecStatus =
  | 'planned'
  | 'in-progress'
  | 'implemented'
  | 'tested'
  | 'deprecated';
```

---

## Testing Requirements

### Unit Tests

```typescript
import { render, fireEvent, screen } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import SpecListLayout from './SpecListLayout.svelte';
import { createMockSpecs } from '$lib/test-utils/mock-data';

describe('SpecListLayout', () => {
  const mockSpecs = createMockSpecs(10);

  it('renders specs in list view mode', () => {
    render(SpecListLayout, { props: { specs: mockSpecs, viewMode: 'list' } });

    expect(screen.getByRole('listbox')).toBeInTheDocument();
    expect(screen.getAllByRole('option')).toHaveLength(10);
  });

  it('handles single selection', async () => {
    const { component } = render(SpecListLayout, {
      props: { specs: mockSpecs, selectionMode: 'single' }
    });

    const selectHandler = vi.fn();
    component.$on('select', selectHandler);

    await fireEvent.click(screen.getAllByRole('option')[0]);

    expect(selectHandler).toHaveBeenCalledWith(
      expect.objectContaining({
        detail: expect.objectContaining({
          spec: mockSpecs[0],
          selected: [mockSpecs[0]]
        })
      })
    );
  });

  it('supports keyboard navigation', async () => {
    render(SpecListLayout, { props: { specs: mockSpecs } });

    const list = screen.getByRole('listbox');
    await fireEvent.keyDown(list, { key: 'ArrowDown' });

    expect(screen.getAllByRole('option')[0]).toHaveAttribute('aria-selected', 'true');
  });

  it('displays empty state when no specs', () => {
    render(SpecListLayout, { props: { specs: [] } });

    expect(screen.getByText('No specs found')).toBeInTheDocument();
  });

  it('shows loading skeleton when loading', () => {
    render(SpecListLayout, { props: { specs: [], loading: true } });

    expect(screen.getByRole('listbox')).toHaveClass('spec-list--loading');
  });

  it('handles multi-select with Ctrl+click', async () => {
    const { component } = render(SpecListLayout, {
      props: { specs: mockSpecs, selectionMode: 'multi' }
    });

    const selectHandler = vi.fn();
    component.$on('select', selectHandler);

    const options = screen.getAllByRole('option');
    await fireEvent.click(options[0]);
    await fireEvent.click(options[2], { ctrlKey: true });

    expect(selectHandler).toHaveBeenLastCalledWith(
      expect.objectContaining({
        detail: expect.objectContaining({
          selected: expect.arrayContaining([mockSpecs[0], mockSpecs[2]])
        })
      })
    );
  });
});
```

### Integration Tests

```typescript
import { render, screen, waitFor } from '@testing-library/svelte';
import { describe, it, expect } from 'vitest';
import SpecBrowser from './SpecBrowser.svelte';
import { specStore } from '$lib/stores/spec-store';

describe('SpecListLayout Integration', () => {
  it('integrates with spec store for data loading', async () => {
    render(SpecBrowser);

    await waitFor(() => {
      expect(screen.getAllByRole('option').length).toBeGreaterThan(0);
    });
  });

  it('virtual scrolling renders correct items on scroll', async () => {
    const manySpecs = createMockSpecs(1000);
    specStore.set(manySpecs);

    render(SpecBrowser);

    const list = screen.getByRole('listbox');
    await fireEvent.scroll(list, { target: { scrollTop: 5000 } });

    // Should render items around scroll position, not all 1000
    const renderedItems = screen.getAllByRole('option');
    expect(renderedItems.length).toBeLessThan(50);
  });
});
```

---

## Related Specs

- Spec 232: Spec Card Component
- Spec 233: Spec Filter System
- Spec 234: Spec Search
- Spec 235: Spec Sort
- Spec 201: Base Layout System
