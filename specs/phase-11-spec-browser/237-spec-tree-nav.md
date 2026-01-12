# 237 - Spec Tree Navigation

**Phase:** 11 - Spec Browser UI
**Spec ID:** 237
**Status:** Planned
**Dependencies:** 236-spec-browser-layout
**Estimated Context:** ~12% of Sonnet window

---

## Objective

Create a tree navigation component for browsing specification files organized by phase, with expand/collapse, drag-and-drop reordering, and status indicators.

---

## Acceptance Criteria

- [ ] Hierarchical tree view by phase
- [ ] Expand/collapse folders
- [ ] Single-click to preview, double-click to open
- [ ] Drag-and-drop reordering within phase
- [ ] Status indicators (planned/in-progress/complete)
- [ ] Context menu for actions
- [ ] Keyboard navigation (arrows, enter)

---

## Implementation Details

### 1. Types (src/lib/types/spec-tree.ts)

```typescript
export interface SpecTreeNode {
  id: string;
  type: 'phase' | 'spec';
  label: string;
  number: number;
  status?: SpecStatus;
  children?: SpecTreeNode[];
  isExpanded?: boolean;
  path?: string;
}

export type SpecStatus = 'planned' | 'in_progress' | 'complete' | 'blocked';

export interface TreeState {
  nodes: SpecTreeNode[];
  expandedIds: Set<string>;
  selectedId: string | null;
  focusedId: string | null;
  dragState: DragState | null;
}

export interface DragState {
  nodeId: string;
  targetId: string | null;
  position: 'before' | 'after' | 'inside';
}
```

### 2. Tree Store (src/lib/stores/spec-tree-store.ts)

```typescript
import { writable, derived } from 'svelte/store';
import type { SpecTreeNode, TreeState, DragState, SpecStatus } from '$lib/types/spec-tree';
import { ipcRenderer } from '$lib/ipc';

function createSpecTreeStore() {
  const initialState: TreeState = {
    nodes: [],
    expandedIds: new Set(['phase-0', 'phase-1']),
    selectedId: null,
    focusedId: null,
    dragState: null,
  };

  const { subscribe, set, update } = writable<TreeState>(initialState);

  return {
    subscribe,

    async loadSpecs(): Promise<void> {
      const specs = await ipcRenderer.invoke('spec:list');
      const nodes = buildTree(specs);
      update(s => ({ ...s, nodes }));
    },

    toggleExpand(nodeId: string) {
      update(s => {
        const expandedIds = new Set(s.expandedIds);
        if (expandedIds.has(nodeId)) {
          expandedIds.delete(nodeId);
        } else {
          expandedIds.add(nodeId);
        }
        return { ...s, expandedIds };
      });
    },

    expandAll() {
      update(s => {
        const expandedIds = new Set<string>();
        const collect = (nodes: SpecTreeNode[]) => {
          nodes.forEach(n => {
            if (n.children?.length) {
              expandedIds.add(n.id);
              collect(n.children);
            }
          });
        };
        collect(s.nodes);
        return { ...s, expandedIds };
      });
    },

    collapseAll() {
      update(s => ({ ...s, expandedIds: new Set() }));
    },

    select(nodeId: string | null) {
      update(s => ({ ...s, selectedId: nodeId, focusedId: nodeId }));
    },

    setFocus(nodeId: string | null) {
      update(s => ({ ...s, focusedId: nodeId }));
    },

    startDrag(nodeId: string) {
      update(s => ({
        ...s,
        dragState: { nodeId, targetId: null, position: 'after' },
      }));
    },

    updateDrag(targetId: string, position: 'before' | 'after' | 'inside') {
      update(s => {
        if (!s.dragState) return s;
        return {
          ...s,
          dragState: { ...s.dragState, targetId, position },
        };
      });
    },

    endDrag() {
      update(s => ({ ...s, dragState: null }));
    },

    async reorder(nodeId: string, targetId: string, position: 'before' | 'after') {
      await ipcRenderer.invoke('spec:reorder', { nodeId, targetId, position });
      await this.loadSpecs();
    },
  };
}

function buildTree(specs: any[]): SpecTreeNode[] {
  const phases = new Map<number, SpecTreeNode>();

  specs.forEach(spec => {
    if (!phases.has(spec.phase)) {
      phases.set(spec.phase, {
        id: `phase-${spec.phase}`,
        type: 'phase',
        label: spec.phaseName || `Phase ${spec.phase}`,
        number: spec.phase,
        children: [],
        isExpanded: spec.phase <= 1,
      });
    }

    phases.get(spec.phase)!.children!.push({
      id: spec.id,
      type: 'spec',
      label: spec.title,
      number: spec.number,
      status: spec.status,
      path: spec.path,
    });
  });

  return Array.from(phases.values()).sort((a, b) => a.number - b.number);
}

export const specTreeStore = createSpecTreeStore();

export const flattenedNodes = derived(specTreeStore, $state => {
  const result: SpecTreeNode[] = [];
  const flatten = (nodes: SpecTreeNode[], depth = 0) => {
    nodes.forEach(node => {
      result.push({ ...node, depth } as any);
      if (node.children && $state.expandedIds.has(node.id)) {
        flatten(node.children, depth + 1);
      }
    });
  };
  flatten($state.nodes);
  return result;
});
```

### 3. Spec Tree Nav Component (src/lib/components/spec-browser/SpecTreeNav.svelte)

```svelte
<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { specTreeStore, flattenedNodes } from '$lib/stores/spec-tree-store';
  import type { SpecTreeNode, SpecStatus } from '$lib/types/spec-tree';
  import SpecTreeItem from './SpecTreeItem.svelte';

  export let selectedId: string | null = null;

  const dispatch = createEventDispatcher<{
    select: string;
    open: string;
  }>();

  let searchQuery = '';
  let treeRef: HTMLElement;

  const statusColors: Record<SpecStatus, string> = {
    planned: 'var(--color-text-muted)',
    in_progress: 'var(--color-primary)',
    complete: 'var(--color-success)',
    blocked: 'var(--color-error)',
  };

  function handleKeyDown(event: KeyboardEvent) {
    const focused = $specTreeStore.focusedId;
    const nodes = $flattenedNodes;
    const currentIndex = nodes.findIndex(n => n.id === focused);

    switch (event.key) {
      case 'ArrowDown':
        event.preventDefault();
        if (currentIndex < nodes.length - 1) {
          specTreeStore.setFocus(nodes[currentIndex + 1].id);
        }
        break;
      case 'ArrowUp':
        event.preventDefault();
        if (currentIndex > 0) {
          specTreeStore.setFocus(nodes[currentIndex - 1].id);
        }
        break;
      case 'ArrowRight':
        event.preventDefault();
        if (focused) {
          const node = nodes.find(n => n.id === focused);
          if (node?.children && !$specTreeStore.expandedIds.has(focused)) {
            specTreeStore.toggleExpand(focused);
          }
        }
        break;
      case 'ArrowLeft':
        event.preventDefault();
        if (focused && $specTreeStore.expandedIds.has(focused)) {
          specTreeStore.toggleExpand(focused);
        }
        break;
      case 'Enter':
        event.preventDefault();
        if (focused) {
          const node = nodes.find(n => n.id === focused);
          if (node?.type === 'spec') {
            dispatch('open', focused);
          }
        }
        break;
    }
  }

  function handleSelect(node: SpecTreeNode) {
    specTreeStore.select(node.id);
    if (node.type === 'spec') {
      dispatch('select', node.id);
    }
  }

  function handleDoubleClick(node: SpecTreeNode) {
    if (node.type === 'spec') {
      dispatch('open', node.id);
    } else {
      specTreeStore.toggleExpand(node.id);
    }
  }

  $: filteredNodes = searchQuery
    ? $flattenedNodes.filter(n =>
        n.label.toLowerCase().includes(searchQuery.toLowerCase()) ||
        n.number.toString().includes(searchQuery)
      )
    : $flattenedNodes;

  onMount(() => {
    specTreeStore.loadSpecs();
  });
</script>

<div class="spec-tree-nav" on:keydown={handleKeyDown}>
  <div class="spec-tree-nav__header">
    <input
      type="search"
      class="spec-tree-nav__search"
      placeholder="Search specs..."
      bind:value={searchQuery}
    />

    <div class="spec-tree-nav__actions">
      <button
        class="action-btn"
        title="Expand all"
        on:click={() => specTreeStore.expandAll()}
      >
        +
      </button>
      <button
        class="action-btn"
        title="Collapse all"
        on:click={() => specTreeStore.collapseAll()}
      >
        -
      </button>
    </div>
  </div>

  <div
    bind:this={treeRef}
    class="spec-tree-nav__tree"
    role="tree"
    aria-label="Specifications"
    tabindex="0"
  >
    {#each filteredNodes as node (node.id)}
      <SpecTreeItem
        {node}
        depth={node.depth || 0}
        expanded={$specTreeStore.expandedIds.has(node.id)}
        selected={selectedId === node.id}
        focused={$specTreeStore.focusedId === node.id}
        statusColor={node.status ? statusColors[node.status] : undefined}
        on:click={() => handleSelect(node)}
        on:dblclick={() => handleDoubleClick(node)}
        on:toggle={() => specTreeStore.toggleExpand(node.id)}
      />
    {/each}
  </div>
</div>

<style>
  .spec-tree-nav {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .spec-tree-nav__header {
    display: flex;
    gap: 8px;
    padding: 12px;
    border-bottom: 1px solid var(--color-border);
  }

  .spec-tree-nav__search {
    flex: 1;
    padding: 6px 10px;
    border: 1px solid var(--color-border);
    border-radius: 4px;
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
    font-size: 13px;
  }

  .spec-tree-nav__actions {
    display: flex;
    gap: 4px;
  }

  .action-btn {
    width: 28px;
    height: 28px;
    border: 1px solid var(--color-border);
    background: var(--color-bg-primary);
    border-radius: 4px;
    cursor: pointer;
    font-size: 14px;
    color: var(--color-text-secondary);
  }

  .action-btn:hover {
    background: var(--color-bg-hover);
  }

  .spec-tree-nav__tree {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
  }

  .spec-tree-nav__tree:focus {
    outline: none;
  }
</style>
```

### 4. Spec Tree Item Component (src/lib/components/spec-browser/SpecTreeItem.svelte)

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { SpecTreeNode } from '$lib/types/spec-tree';

  export let node: SpecTreeNode;
  export let depth = 0;
  export let expanded = false;
  export let selected = false;
  export let focused = false;
  export let statusColor: string | undefined;

  const dispatch = createEventDispatcher<{
    click: void;
    dblclick: void;
    toggle: void;
  }>();
</script>

<div
  class="tree-item"
  class:tree-item--selected={selected}
  class:tree-item--focused={focused}
  class:tree-item--phase={node.type === 'phase'}
  style="padding-left: {12 + depth * 16}px"
  role="treeitem"
  aria-selected={selected}
  aria-expanded={node.children ? expanded : undefined}
  on:click={() => dispatch('click')}
  on:dblclick={() => dispatch('dblclick')}
>
  {#if node.children}
    <button
      class="tree-item__toggle"
      on:click|stopPropagation={() => dispatch('toggle')}
      aria-label={expanded ? 'Collapse' : 'Expand'}
    >
      <span class:rotated={expanded}>▸</span>
    </button>
  {:else}
    <span class="tree-item__spacer"></span>
  {/if}

  {#if node.type === 'phase'}
    <span class="tree-item__icon tree-item__icon--folder">📁</span>
  {:else}
    <span class="tree-item__icon">📄</span>
  {/if}

  <span class="tree-item__number">{String(node.number).padStart(3, '0')}</span>
  <span class="tree-item__label">{node.label}</span>

  {#if statusColor}
    <span class="tree-item__status" style="background: {statusColor}"></span>
  {/if}
</div>

<style>
  .tree-item {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 8px;
    border-radius: 4px;
    cursor: pointer;
    user-select: none;
  }

  .tree-item:hover {
    background: var(--color-bg-hover);
  }

  .tree-item--selected {
    background: var(--color-bg-active);
  }

  .tree-item--focused {
    outline: 2px solid var(--color-primary);
    outline-offset: -2px;
  }

  .tree-item--phase {
    font-weight: 600;
  }

  .tree-item__toggle {
    padding: 2px;
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
    font-size: 10px;
  }

  .tree-item__toggle span {
    display: inline-block;
    transition: transform 0.15s;
  }

  .tree-item__toggle span.rotated {
    transform: rotate(90deg);
  }

  .tree-item__spacer {
    width: 14px;
  }

  .tree-item__icon {
    font-size: 12px;
  }

  .tree-item__number {
    font-family: monospace;
    font-size: 11px;
    color: var(--color-primary);
    min-width: 28px;
  }

  .tree-item__label {
    flex: 1;
    font-size: 13px;
    color: var(--color-text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .tree-item__status {
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }
</style>
```

---

## Testing Requirements

1. Tree loads and displays specs
2. Expand/collapse works
3. Selection and focus work
4. Keyboard navigation works
5. Search filters correctly
6. Double-click opens spec

---

## Related Specs

- Depends on: [236-spec-browser-layout.md](236-spec-browser-layout.md)
- Next: [238-spec-file-viewer.md](238-spec-file-viewer.md)
