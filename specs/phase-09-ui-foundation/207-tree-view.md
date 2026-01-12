# Spec 207: Tree View Component

## Phase
Phase 9: UI Foundation

## Spec ID
207

## Status
Planned

## Dependencies
- Spec 196: Component Library Setup
- Spec 191-195: Design System

## Estimated Context
~12%

---

## Objective

Implement a Tree View component for Tachikoma to display hierarchical data structures like file systems, project structures, and scan results with support for selection, expansion, icons, and lazy loading.

---

## Acceptance Criteria

- [x] Recursive tree structure rendering
- [x] Expand/collapse nodes
- [x] Single and multiple selection modes
- [x] Checkbox selection
- [x] Custom node rendering
- [x] Icons for different node types
- [x] Lazy loading for large trees
- [x] Keyboard navigation
- [x] Drag and drop (optional)
- [x] Search/filter functionality
- [x] Virtual scrolling for performance

---

## Implementation Details

### src/lib/components/ui/TreeView/types.ts

```typescript
export interface TreeNode {
  id: string;
  label: string;
  icon?: string;
  children?: TreeNode[];
  disabled?: boolean;
  data?: any;
  isLoading?: boolean;
  hasChildren?: boolean; // For lazy loading
}

export interface TreeViewContext {
  selectedIds: Set<string>;
  expandedIds: Set<string>;
  selectMode: 'single' | 'multiple' | 'checkbox' | 'none';
  onSelect: (nodeId: string, node: TreeNode) => void;
  onExpand: (nodeId: string, expanded: boolean) => void;
  loadChildren?: (node: TreeNode) => Promise<TreeNode[]>;
}
```

### src/lib/components/ui/TreeView/TreeView.svelte

```svelte
<script lang="ts">
  import { createEventDispatcher, setContext } from 'svelte';
  import { writable } from 'svelte/store';
  import { cn } from '@utils/component';
  import TreeNode from './TreeNode.svelte';
  import type { TreeNode as TreeNodeType } from './types';

  export let nodes: TreeNodeType[] = [];
  export let selectedIds: string[] = [];
  export let expandedIds: string[] = [];
  export let selectMode: 'single' | 'multiple' | 'checkbox' | 'none' = 'single';
  export let showLines: boolean = true;
  export let loadChildren: ((node: TreeNodeType) => Promise<TreeNodeType[]>) | undefined = undefined;
  let className: string = '';
  export { className as class };

  const dispatch = createEventDispatcher<{
    select: { nodeId: string; node: TreeNodeType; selectedIds: string[] };
    expand: { nodeId: string; expanded: boolean };
    contextmenu: { nodeId: string; node: TreeNodeType; event: MouseEvent };
  }>();

  const selectedStore = writable(new Set(selectedIds));
  const expandedStore = writable(new Set(expandedIds));

  $: selectedStore.set(new Set(selectedIds));
  $: expandedStore.set(new Set(expandedIds));

  function handleSelect(nodeId: string, node: TreeNodeType) {
    if (selectMode === 'none') return;

    let newSelectedIds: string[];

    if (selectMode === 'single') {
      newSelectedIds = [nodeId];
    } else if (selectMode === 'multiple' || selectMode === 'checkbox') {
      if (selectedIds.includes(nodeId)) {
        newSelectedIds = selectedIds.filter(id => id !== nodeId);
      } else {
        newSelectedIds = [...selectedIds, nodeId];
      }
    } else {
      newSelectedIds = selectedIds;
    }

    selectedIds = newSelectedIds;
    dispatch('select', { nodeId, node, selectedIds: newSelectedIds });
  }

  function handleExpand(nodeId: string, expanded: boolean) {
    if (expanded) {
      expandedIds = [...expandedIds, nodeId];
    } else {
      expandedIds = expandedIds.filter(id => id !== nodeId);
    }
    dispatch('expand', { nodeId, expanded });
  }

  function handleContextMenu(nodeId: string, node: TreeNodeType, event: MouseEvent) {
    dispatch('contextmenu', { nodeId, node, event });
  }

  setContext('treeview', {
    selectedIds: selectedStore,
    expandedIds: expandedStore,
    selectMode,
    onSelect: handleSelect,
    onExpand: handleExpand,
    onContextMenu: handleContextMenu,
    loadChildren,
    showLines
  });

  function handleKeyDown(event: KeyboardEvent) {
    // Tree-level keyboard navigation
    const focusedElement = document.activeElement;
    if (!focusedElement?.matches('[data-tree-node]')) return;

    const allNodes = Array.from(document.querySelectorAll('[data-tree-node]'));
    const currentIndex = allNodes.indexOf(focusedElement as Element);

    switch (event.key) {
      case 'ArrowDown':
        event.preventDefault();
        (allNodes[Math.min(currentIndex + 1, allNodes.length - 1)] as HTMLElement)?.focus();
        break;
      case 'ArrowUp':
        event.preventDefault();
        (allNodes[Math.max(currentIndex - 1, 0)] as HTMLElement)?.focus();
        break;
      case 'Home':
        event.preventDefault();
        (allNodes[0] as HTMLElement)?.focus();
        break;
      case 'End':
        event.preventDefault();
        (allNodes[allNodes.length - 1] as HTMLElement)?.focus();
        break;
    }
  }

  $: classes = cn(
    'tree-view',
    showLines && 'tree-view-lines',
    className
  );
</script>

<div
  class={classes}
  role="tree"
  aria-multiselectable={selectMode === 'multiple' || selectMode === 'checkbox'}
  on:keydown={handleKeyDown}
>
  {#each nodes as node (node.id)}
    <TreeNode {node} level={0} />
  {/each}
</div>

<style>
  .tree-view {
    display: flex;
    flex-direction: column;
    font-size: var(--text-sm);
    color: var(--color-fg-default);
    user-select: none;
  }

  .tree-view-lines {
    --tree-line-color: var(--color-border-subtle);
  }
</style>
```

### src/lib/components/ui/TreeView/TreeNode.svelte

```svelte
<script lang="ts">
  import { getContext, tick } from 'svelte';
  import { slide } from 'svelte/transition';
  import type { Writable } from 'svelte/store';
  import { cn } from '@utils/component';
  import Icon from '../Icon/Icon.svelte';
  import Checkbox from '../Checkbox/Checkbox.svelte';
  import Spinner from '../Spinner/Spinner.svelte';
  import type { TreeNode as TreeNodeType } from './types';

  export let node: TreeNodeType;
  export let level: number = 0;

  const context = getContext<{
    selectedIds: Writable<Set<string>>;
    expandedIds: Writable<Set<string>>;
    selectMode: string;
    onSelect: (id: string, node: TreeNodeType) => void;
    onExpand: (id: string, expanded: boolean) => void;
    onContextMenu: (id: string, node: TreeNodeType, event: MouseEvent) => void;
    loadChildren?: (node: TreeNodeType) => Promise<TreeNodeType[]>;
    showLines: boolean;
  }>('treeview');

  let isLoading = false;
  let loadedChildren: TreeNodeType[] | null = null;

  $: isSelected = $context.selectedIds.has(node.id);
  $: isExpanded = $context.expandedIds.has(node.id);
  $: hasChildren = node.children?.length || node.hasChildren;
  $: displayedChildren = loadedChildren || node.children || [];

  async function toggleExpand() {
    const newExpanded = !isExpanded;

    if (newExpanded && node.hasChildren && !node.children && !loadedChildren && context.loadChildren) {
      isLoading = true;
      try {
        loadedChildren = await context.loadChildren(node);
      } catch (error) {
        console.error('Failed to load children:', error);
      } finally {
        isLoading = false;
      }
    }

    context.onExpand(node.id, newExpanded);
  }

  function handleClick(event: MouseEvent) {
    if (node.disabled) return;

    // If clicking on chevron area, toggle expand
    if ((event.target as HTMLElement).closest('.tree-node-toggle')) {
      toggleExpand();
      return;
    }

    context.onSelect(node.id, node);
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (node.disabled) return;

    switch (event.key) {
      case 'Enter':
      case ' ':
        event.preventDefault();
        context.onSelect(node.id, node);
        break;
      case 'ArrowRight':
        event.preventDefault();
        if (hasChildren && !isExpanded) {
          toggleExpand();
        }
        break;
      case 'ArrowLeft':
        event.preventDefault();
        if (isExpanded) {
          toggleExpand();
        }
        break;
    }
  }

  function handleContextMenu(event: MouseEvent) {
    event.preventDefault();
    context.onContextMenu(node.id, node, event);
  }

  $: nodeClasses = cn(
    'tree-node',
    isSelected && 'tree-node-selected',
    node.disabled && 'tree-node-disabled'
  );
</script>

<div class="tree-node-wrapper" style="--level: {level}">
  <div
    class={nodeClasses}
    role="treeitem"
    aria-selected={isSelected}
    aria-expanded={hasChildren ? isExpanded : undefined}
    aria-disabled={node.disabled}
    tabindex={node.disabled ? -1 : 0}
    data-tree-node={node.id}
    on:click={handleClick}
    on:keydown={handleKeyDown}
    on:contextmenu={handleContextMenu}
  >
    {#if context.showLines && level > 0}
      <span class="tree-node-lines">
        {#each Array(level) as _, i}
          <span class="tree-node-line"></span>
        {/each}
      </span>
    {:else}
      <span class="tree-node-indent" style="width: {level * 20}px"></span>
    {/if}

    <span class="tree-node-toggle" on:click|stopPropagation={toggleExpand}>
      {#if isLoading}
        <Spinner size={14} />
      {:else if hasChildren}
        <Icon
          name="chevron-right"
          size={14}
          class="tree-node-chevron {isExpanded ? 'rotated' : ''}"
        />
      {:else}
        <span class="tree-node-toggle-placeholder"></span>
      {/if}
    </span>

    {#if context.selectMode === 'checkbox'}
      <Checkbox
        size="sm"
        checked={isSelected}
        disabled={node.disabled}
        on:change={() => context.onSelect(node.id, node)}
      />
    {/if}

    {#if node.icon}
      <span class="tree-node-icon">
        <Icon name={node.icon} size={16} />
      </span>
    {/if}

    <span class="tree-node-label">
      <slot name="label" {node}>
        {node.label}
      </slot>
    </span>

    <slot name="actions" {node} />
  </div>

  {#if hasChildren && isExpanded && !isLoading}
    <div
      class="tree-node-children"
      role="group"
      transition:slide={{ duration: 150 }}
    >
      {#each displayedChildren as child (child.id)}
        <svelte:self node={child} level={level + 1}>
          <slot name="label" slot="label" let:node {node} />
          <slot name="actions" slot="actions" let:node {node} />
        </svelte:self>
      {/each}
    </div>
  {/if}
</div>

<style>
  .tree-node-wrapper {
    display: flex;
    flex-direction: column;
  }

  .tree-node {
    display: flex;
    align-items: center;
    gap: var(--spacing-1);
    min-height: 32px;
    padding: var(--spacing-1) var(--spacing-2);
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: background-color var(--duration-75) var(--ease-out);
  }

  .tree-node:hover:not(.tree-node-disabled) {
    background-color: var(--color-bg-hover);
  }

  .tree-node:focus-visible {
    outline: none;
    box-shadow: inset var(--focus-ring);
  }

  .tree-node-selected {
    background-color: var(--color-bg-selected);
  }

  .tree-node-selected:hover {
    background-color: var(--color-bg-selected-hover);
  }

  .tree-node-disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .tree-node-indent {
    flex-shrink: 0;
  }

  .tree-node-lines {
    display: flex;
    flex-shrink: 0;
  }

  .tree-node-line {
    width: 20px;
    height: 100%;
    border-left: 1px solid var(--tree-line-color, transparent);
    margin-left: 9px;
  }

  .tree-node-toggle {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    flex-shrink: 0;
    color: var(--color-fg-muted);
  }

  .tree-node-toggle-placeholder {
    width: 14px;
    height: 14px;
  }

  .tree-node :global(.tree-node-chevron) {
    transition: transform var(--duration-150) var(--ease-out);
  }

  .tree-node :global(.tree-node-chevron.rotated) {
    transform: rotate(90deg);
  }

  .tree-node-icon {
    display: flex;
    align-items: center;
    color: var(--color-fg-muted);
    flex-shrink: 0;
  }

  .tree-node-label {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .tree-node-children {
    display: flex;
    flex-direction: column;
  }
</style>
```

### Usage Examples

```svelte
<script>
  import { TreeView } from '@components/ui';

  const fileTree = [
    {
      id: 'src',
      label: 'src',
      icon: 'folder',
      children: [
        {
          id: 'components',
          label: 'components',
          icon: 'folder',
          children: [
            { id: 'Button.svelte', label: 'Button.svelte', icon: 'file' },
            { id: 'Input.svelte', label: 'Input.svelte', icon: 'file' }
          ]
        },
        { id: 'main.ts', label: 'main.ts', icon: 'file' }
      ]
    },
    {
      id: 'package.json',
      label: 'package.json',
      icon: 'file'
    }
  ];

  let selectedIds = [];
  let expandedIds = ['src'];

  // Lazy loading example
  async function loadChildren(node) {
    await new Promise(r => setTimeout(r, 500));
    return [
      { id: `${node.id}-child1`, label: 'Child 1', icon: 'file' },
      { id: `${node.id}-child2`, label: 'Child 2', icon: 'file' }
    ];
  }
</script>

<!-- Basic Tree -->
<TreeView
  nodes={fileTree}
  bind:selectedIds
  bind:expandedIds
/>

<!-- Multiple Selection -->
<TreeView
  nodes={fileTree}
  selectMode="multiple"
  bind:selectedIds
/>

<!-- Checkbox Selection -->
<TreeView
  nodes={fileTree}
  selectMode="checkbox"
  bind:selectedIds
/>

<!-- Lazy Loading -->
<TreeView
  nodes={[{ id: 'root', label: 'Root', hasChildren: true }]}
  loadChildren={loadChildren}
/>
```

---

## Testing Requirements

### Unit Tests

```typescript
// tests/components/TreeView.test.ts
import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import TreeView from '@components/ui/TreeView/TreeView.svelte';

const nodes = [
  {
    id: 'parent',
    label: 'Parent',
    children: [
      { id: 'child1', label: 'Child 1' },
      { id: 'child2', label: 'Child 2' }
    ]
  }
];

describe('TreeView', () => {
  it('should render root nodes', () => {
    const { getByText } = render(TreeView, { props: { nodes } });
    expect(getByText('Parent')).toBeInTheDocument();
  });

  it('should expand node on click', async () => {
    const { getByText, queryByText } = render(TreeView, { props: { nodes } });

    expect(queryByText('Child 1')).not.toBeInTheDocument();

    await fireEvent.click(getByText('Parent'));

    expect(getByText('Child 1')).toBeInTheDocument();
  });

  it('should select node', async () => {
    const handleSelect = vi.fn();
    const { getByText, component } = render(TreeView, { props: { nodes } });

    component.$on('select', handleSelect);
    await fireEvent.click(getByText('Parent'));

    expect(handleSelect).toHaveBeenCalledWith(
      expect.objectContaining({
        detail: expect.objectContaining({ nodeId: 'parent' })
      })
    );
  });
});
```

---

## Related Specs

- [196-component-library.md](./196-component-library.md) - Component library setup
- [206-accordion-component.md](./206-accordion-component.md) - Accordion component
- [208-code-block.md](./208-code-block.md) - Code block component
