# Spec 206: Accordion Component

## Phase
Phase 9: UI Foundation

## Spec ID
206

## Status
Planned

## Dependencies
- Spec 196: Component Library Setup
- Spec 191-195: Design System

## Estimated Context
~8%

---

## Objective

Implement an Accordion component for Tachikoma with support for single and multiple expansion modes, icons, animations, and full keyboard accessibility following WAI-ARIA disclosure pattern.

---

## Acceptance Criteria

- [ ] Single and multiple expansion modes
- [ ] Animated expand/collapse
- [ ] Custom trigger content
- [ ] Icon customization
- [ ] Disabled items
- [ ] Controlled and uncontrolled modes
- [ ] Keyboard navigation
- [ ] Nested accordions support
- [ ] WAI-ARIA compliant

---

## Implementation Details

### src/lib/components/ui/Accordion/Accordion.svelte

```svelte
<script lang="ts" context="module">
  export interface AccordionItem {
    id: string;
    title: string;
    content?: string;
    disabled?: boolean;
    icon?: string;
  }
</script>

<script lang="ts">
  import { createEventDispatcher, setContext } from 'svelte';
  import { writable } from 'svelte/store';
  import { cn } from '@utils/component';

  export let items: AccordionItem[] = [];
  export let value: string | string[] = [];
  export let multiple: boolean = false;
  export let collapsible: boolean = true;
  export let variant: 'default' | 'bordered' | 'separated' = 'default';
  let className: string = '';
  export { className as class };

  const dispatch = createEventDispatcher<{
    change: string | string[];
  }>();

  // Normalize value to array
  $: expandedItems = Array.isArray(value) ? value : value ? [value] : [];

  const expandedStore = writable<string[]>(expandedItems);
  $: expandedStore.set(expandedItems);

  setContext('accordion', {
    expanded: expandedStore,
    multiple,
    collapsible,
    toggle: (itemId: string) => toggleItem(itemId)
  });

  function toggleItem(itemId: string) {
    const item = items.find(i => i.id === itemId);
    if (item?.disabled) return;

    const isExpanded = expandedItems.includes(itemId);

    let newValue: string | string[];

    if (multiple) {
      if (isExpanded) {
        newValue = expandedItems.filter(id => id !== itemId);
      } else {
        newValue = [...expandedItems, itemId];
      }
    } else {
      if (isExpanded && collapsible) {
        newValue = '';
      } else {
        newValue = itemId;
      }
    }

    value = newValue;
    dispatch('change', newValue);
  }

  function handleKeyDown(event: KeyboardEvent, index: number) {
    const enabledItems = items.filter(i => !i.disabled);
    const currentItem = items[index];
    const currentEnabledIndex = enabledItems.findIndex(i => i.id === currentItem.id);

    let newIndex = currentEnabledIndex;

    switch (event.key) {
      case 'ArrowDown':
        event.preventDefault();
        newIndex = (currentEnabledIndex + 1) % enabledItems.length;
        break;
      case 'ArrowUp':
        event.preventDefault();
        newIndex = (currentEnabledIndex - 1 + enabledItems.length) % enabledItems.length;
        break;
      case 'Home':
        event.preventDefault();
        newIndex = 0;
        break;
      case 'End':
        event.preventDefault();
        newIndex = enabledItems.length - 1;
        break;
      default:
        return;
    }

    const newItem = enabledItems[newIndex];
    const button = document.querySelector(`[data-accordion-trigger="${newItem.id}"]`) as HTMLElement;
    button?.focus();
  }

  $: classes = cn(
    'accordion',
    `accordion-${variant}`,
    className
  );
</script>

<div class={classes} {...$$restProps}>
  {#each items as item, index (item.id)}
    <AccordionItemComponent
      {item}
      expanded={expandedItems.includes(item.id)}
      on:toggle={() => toggleItem(item.id)}
      on:keydown={(e) => handleKeyDown(e, index)}
    >
      <slot name="item" {item} expanded={expandedItems.includes(item.id)} />
    </AccordionItemComponent>
  {/each}

  <slot />
</div>

<script lang="ts">
  import AccordionItemComponent from './AccordionItem.svelte';
</script>

<style>
  .accordion {
    display: flex;
    flex-direction: column;
    width: 100%;
  }

  .accordion-default {
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-lg);
    overflow: hidden;
  }

  .accordion-bordered :global(.accordion-item) {
    border-bottom: 1px solid var(--color-border-default);
  }

  .accordion-bordered :global(.accordion-item:last-child) {
    border-bottom: none;
  }

  .accordion-separated {
    gap: var(--spacing-2);
  }

  .accordion-separated :global(.accordion-item) {
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-lg);
  }
</style>
```

### src/lib/components/ui/Accordion/AccordionItem.svelte

```svelte
<script lang="ts">
  import { createEventDispatcher, getContext } from 'svelte';
  import { slide } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import type { Writable } from 'svelte/store';
  import { cn } from '@utils/component';
  import Icon from '../Icon/Icon.svelte';
  import type { AccordionItem } from './Accordion.svelte';

  export let item: AccordionItem;
  export let expanded: boolean = false;

  const dispatch = createEventDispatcher<{
    toggle: void;
    keydown: KeyboardEvent;
  }>();

  const context = getContext<{
    expanded: Writable<string[]>;
    multiple: boolean;
    collapsible: boolean;
    toggle: (id: string) => void;
  } | undefined>('accordion');

  function handleClick() {
    if (item.disabled) return;
    dispatch('toggle');
    context?.toggle(item.id);
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      handleClick();
    } else {
      dispatch('keydown', event);
    }
  }

  $: isExpanded = context
    ? ($context.expanded).includes(item.id)
    : expanded;
</script>

<div
  class="accordion-item"
  class:expanded={isExpanded}
  class:disabled={item.disabled}
>
  <button
    type="button"
    class="accordion-trigger"
    data-accordion-trigger={item.id}
    id="accordion-header-{item.id}"
    aria-expanded={isExpanded}
    aria-controls="accordion-panel-{item.id}"
    disabled={item.disabled}
    on:click={handleClick}
    on:keydown={handleKeyDown}
  >
    {#if item.icon}
      <span class="accordion-icon">
        <Icon name={item.icon} size={18} />
      </span>
    {/if}

    <span class="accordion-title">
      <slot name="trigger">
        {item.title}
      </slot>
    </span>

    <span class="accordion-chevron" class:rotated={isExpanded}>
      <Icon name="chevron-down" size={18} />
    </span>
  </button>

  {#if isExpanded}
    <div
      id="accordion-panel-{item.id}"
      class="accordion-panel"
      role="region"
      aria-labelledby="accordion-header-{item.id}"
      transition:slide={{ duration: 200, easing: cubicOut }}
    >
      <div class="accordion-content">
        <slot>
          {item.content || ''}
        </slot>
      </div>
    </div>
  {/if}
</div>

<style>
  .accordion-item {
    background-color: var(--color-bg-surface);
  }

  .accordion-item.disabled {
    opacity: 0.5;
  }

  .accordion-trigger {
    display: flex;
    align-items: center;
    width: 100%;
    padding: var(--spacing-4);
    background: transparent;
    border: none;
    color: var(--color-fg-default);
    font-family: inherit;
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    text-align: left;
    cursor: pointer;
    transition: background-color var(--duration-150) var(--ease-out);
  }

  .accordion-trigger:hover:not(:disabled) {
    background-color: var(--color-bg-hover);
  }

  .accordion-trigger:focus-visible {
    outline: none;
    box-shadow: inset var(--focus-ring);
  }

  .accordion-trigger:disabled {
    cursor: not-allowed;
  }

  .accordion-icon {
    display: flex;
    align-items: center;
    margin-right: var(--spacing-3);
    color: var(--color-fg-muted);
  }

  .accordion-title {
    flex: 1;
    min-width: 0;
  }

  .accordion-chevron {
    display: flex;
    align-items: center;
    color: var(--color-fg-muted);
    transition: transform var(--duration-200) var(--ease-out);
  }

  .accordion-chevron.rotated {
    transform: rotate(180deg);
  }

  .accordion-panel {
    overflow: hidden;
  }

  .accordion-content {
    padding: 0 var(--spacing-4) var(--spacing-4);
    font-size: var(--text-sm);
    color: var(--color-fg-muted);
    line-height: var(--leading-relaxed);
  }
</style>
```

### Usage Examples

```svelte
<script>
  import { Accordion } from '@components/ui';

  const items = [
    {
      id: 'item1',
      title: 'What is Tachikoma?',
      content: 'Tachikoma is an AI-powered penetration testing assistant.'
    },
    {
      id: 'item2',
      title: 'How does it work?',
      content: 'It uses advanced AI to help identify vulnerabilities.'
    },
    {
      id: 'item3',
      title: 'Is it secure?',
      content: 'Yes, all operations are performed locally.',
      icon: 'shield'
    }
  ];

  let expanded = 'item1';
</script>

<!-- Single expansion -->
<Accordion {items} bind:value={expanded} />

<!-- Multiple expansion -->
<Accordion {items} multiple bind:value={expanded} />

<!-- Separated style -->
<Accordion {items} variant="separated" />

<!-- With custom content -->
<Accordion {items}>
  <svelte:fragment slot="item" let:item let:expanded>
    <p>Custom content for {item.title}</p>
  </svelte:fragment>
</Accordion>
```

---

## Testing Requirements

### Unit Tests

```typescript
// tests/components/Accordion.test.ts
import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import Accordion from '@components/ui/Accordion/Accordion.svelte';

const items = [
  { id: 'item1', title: 'Item 1', content: 'Content 1' },
  { id: 'item2', title: 'Item 2', content: 'Content 2' },
  { id: 'item3', title: 'Item 3', content: 'Content 3', disabled: true }
];

describe('Accordion', () => {
  it('should render all items', () => {
    const { getAllByRole } = render(Accordion, { props: { items } });
    expect(getAllByRole('button')).toHaveLength(3);
  });

  it('should expand item on click', async () => {
    const { getByText, getByRole } = render(Accordion, { props: { items } });

    await fireEvent.click(getByText('Item 1'));

    expect(getByRole('region')).toBeInTheDocument();
    expect(getByText('Content 1')).toBeInTheDocument();
  });

  it('should collapse when clicking expanded item (collapsible)', async () => {
    const { getByText, queryByRole } = render(Accordion, {
      props: { items, value: 'item1', collapsible: true }
    });

    await fireEvent.click(getByText('Item 1'));

    expect(queryByRole('region')).not.toBeInTheDocument();
  });

  it('should allow multiple expanded in multiple mode', async () => {
    const { getByText, getAllByRole } = render(Accordion, {
      props: { items, multiple: true }
    });

    await fireEvent.click(getByText('Item 1'));
    await fireEvent.click(getByText('Item 2'));

    expect(getAllByRole('region')).toHaveLength(2);
  });

  it('should not expand disabled items', async () => {
    const { getByText, queryByRole } = render(Accordion, { props: { items } });

    await fireEvent.click(getByText('Item 3'));

    expect(queryByRole('region')).not.toBeInTheDocument();
  });

  it('should navigate with keyboard', async () => {
    const { getByText } = render(Accordion, { props: { items } });

    const trigger = getByText('Item 1');
    trigger.focus();

    await fireEvent.keyDown(trigger, { key: 'ArrowDown' });

    expect(document.activeElement).toBe(getByText('Item 2'));
  });
});
```

---

## Related Specs

- [196-component-library.md](./196-component-library.md) - Component library setup
- [205-tabs-component.md](./205-tabs-component.md) - Tabs component
- [207-tree-view.md](./207-tree-view.md) - Tree view component
