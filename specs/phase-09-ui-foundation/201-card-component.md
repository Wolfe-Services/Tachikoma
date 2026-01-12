# Spec 201: Card Component

## Phase
Phase 9: UI Foundation

## Spec ID
201

## Status
Planned

## Dependencies
- Spec 196: Component Library Setup
- Spec 191-195: Design System

## Estimated Context
~8%

---

## Objective

Implement a versatile Card component for Tachikoma to display grouped content with support for headers, footers, actions, hover effects, and the signature Tachikoma glow aesthetic.

---

## Acceptance Criteria

- [x] Basic card with padding and border
- [x] Card header with title and actions
- [x] Card footer for actions
- [x] Clickable/interactive card variant
- [x] Hover effects with optional glow
- [x] Loading state with skeleton
- [x] Multiple padding sizes
- [x] Collapsible card
- [x] Card with image/media

---

## Implementation Details

### src/lib/components/ui/Card/Card.svelte

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { slide } from 'svelte/transition';
  import { cn } from '@utils/component';
  import Icon from '../Icon/Icon.svelte';

  type CardPadding = 'none' | 'sm' | 'md' | 'lg';
  type CardVariant = 'default' | 'outlined' | 'elevated' | 'ghost';

  export let variant: CardVariant = 'default';
  export let padding: CardPadding = 'md';
  export let interactive: boolean = false;
  export let selected: boolean = false;
  export let glow: boolean = false;
  export let collapsible: boolean = false;
  export let collapsed: boolean = false;
  export let disabled: boolean = false;
  export let href: string | undefined = undefined;
  let className: string = '';
  export { className as class };

  const dispatch = createEventDispatcher<{
    click: MouseEvent;
    toggle: boolean;
  }>();

  function handleClick(event: MouseEvent) {
    if (disabled) return;

    if (collapsible) {
      collapsed = !collapsed;
      dispatch('toggle', collapsed);
    }

    dispatch('click', event);
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      handleClick(event as unknown as MouseEvent);
    }
  }

  $: tag = href ? 'a' : interactive || collapsible ? 'button' : 'div';
  $: isInteractive = interactive || collapsible || !!href;

  $: classes = cn(
    'card',
    `card-${variant}`,
    `card-padding-${padding}`,
    isInteractive && 'card-interactive',
    selected && 'card-selected',
    glow && 'card-glow',
    disabled && 'card-disabled',
    className
  );
</script>

<svelte:element
  this={tag}
  class={classes}
  href={tag === 'a' ? href : undefined}
  type={tag === 'button' ? 'button' : undefined}
  disabled={tag === 'button' ? disabled : undefined}
  role={isInteractive && tag === 'div' ? 'button' : undefined}
  tabindex={isInteractive && tag === 'div' ? 0 : undefined}
  aria-expanded={collapsible ? !collapsed : undefined}
  on:click={handleClick}
  on:keydown={handleKeyDown}
  {...$$restProps}
>
  {#if $$slots.header || collapsible}
    <div class="card-header">
      <slot name="header" />
      {#if collapsible}
        <span class="card-collapse-icon" class:collapsed>
          <Icon name="chevron-down" size={20} />
        </span>
      {/if}
    </div>
  {/if}

  {#if !collapsed}
    <div
      class="card-body"
      transition:slide={{ duration: collapsible ? 200 : 0 }}
    >
      <slot />
    </div>
  {/if}

  {#if $$slots.footer && !collapsed}
    <div class="card-footer">
      <slot name="footer" />
    </div>
  {/if}
</svelte:element>

<style>
  .card {
    display: flex;
    flex-direction: column;
    background-color: var(--color-bg-surface);
    border-radius: var(--radius-lg);
    transition:
      background-color var(--duration-150) var(--ease-out),
      border-color var(--duration-150) var(--ease-out),
      box-shadow var(--duration-200) var(--ease-out),
      transform var(--duration-150) var(--ease-out);
  }

  /* Reset for interactive elements */
  button.card,
  a.card {
    text-align: left;
    text-decoration: none;
    color: inherit;
    font: inherit;
    border: none;
    cursor: pointer;
  }

  /* Variants */
  .card-default {
    border: 1px solid var(--color-border-default);
  }

  .card-outlined {
    border: 1px solid var(--color-border-default);
    background-color: transparent;
  }

  .card-elevated {
    border: none;
    box-shadow: var(--elevation-2);
  }

  .card-ghost {
    border: 1px solid transparent;
    background-color: transparent;
  }

  /* Padding */
  .card-padding-none .card-body {
    padding: 0;
  }

  .card-padding-sm .card-body {
    padding: var(--spacing-3);
  }

  .card-padding-md .card-body {
    padding: var(--spacing-4);
  }

  .card-padding-lg .card-body {
    padding: var(--spacing-6);
  }

  /* Interactive states */
  .card-interactive:hover:not(.card-disabled) {
    border-color: var(--color-border-strong);
    background-color: var(--color-bg-elevated);
  }

  .card-interactive:active:not(.card-disabled) {
    transform: scale(0.99);
  }

  .card-interactive:focus-visible {
    outline: none;
    box-shadow: var(--focus-ring);
  }

  /* Selected state */
  .card-selected {
    border-color: var(--tachikoma-500);
    background-color: var(--color-accent-subtle);
  }

  .card-selected.card-interactive:hover {
    border-color: var(--tachikoma-400);
  }

  /* Glow effect */
  .card-glow {
    box-shadow: var(--glow-sm);
  }

  .card-glow:hover {
    box-shadow: var(--glow-md);
  }

  .card-glow.card-selected {
    box-shadow: var(--glow-md);
  }

  /* Disabled state */
  .card-disabled {
    opacity: 0.5;
    cursor: not-allowed;
    pointer-events: none;
  }

  /* Header */
  .card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-4);
    border-bottom: 1px solid var(--color-border-subtle);
  }

  .card-padding-sm .card-header {
    padding: var(--spacing-3);
  }

  .card-padding-lg .card-header {
    padding: var(--spacing-6);
  }

  .card-collapse-icon {
    display: flex;
    color: var(--color-fg-muted);
    transition: transform var(--duration-200) var(--ease-out);
  }

  .card-collapse-icon.collapsed {
    transform: rotate(-90deg);
  }

  /* Body */
  .card-body {
    flex: 1;
  }

  /* Footer */
  .card-footer {
    display: flex;
    align-items: center;
    gap: var(--spacing-2);
    padding: var(--spacing-4);
    border-top: 1px solid var(--color-border-subtle);
  }

  .card-padding-sm .card-footer {
    padding: var(--spacing-3);
  }

  .card-padding-lg .card-footer {
    padding: var(--spacing-6);
  }
</style>
```

### src/lib/components/ui/Card/CardHeader.svelte

```svelte
<script lang="ts">
  import { cn } from '@utils/component';

  export let title: string | undefined = undefined;
  export let subtitle: string | undefined = undefined;
  let className: string = '';
  export { className as class };
</script>

<div class={cn('card-header-content', className)}>
  <div class="card-header-text">
    {#if title}
      <h3 class="card-title">{title}</h3>
    {/if}
    {#if subtitle}
      <p class="card-subtitle">{subtitle}</p>
    {/if}
    <slot name="title" />
  </div>

  {#if $$slots.actions}
    <div class="card-header-actions">
      <slot name="actions" />
    </div>
  {/if}
</div>

<style>
  .card-header-content {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--spacing-4);
    width: 100%;
  }

  .card-header-text {
    flex: 1;
    min-width: 0;
  }

  .card-title {
    margin: 0;
    font-size: var(--text-lg);
    font-weight: var(--font-semibold);
    color: var(--color-fg-default);
    line-height: var(--leading-tight);
  }

  .card-subtitle {
    margin: var(--spacing-1) 0 0;
    font-size: var(--text-sm);
    color: var(--color-fg-muted);
    line-height: var(--leading-normal);
  }

  .card-header-actions {
    display: flex;
    align-items: center;
    gap: var(--spacing-2);
    flex-shrink: 0;
  }
</style>
```

### Usage Examples

```svelte
<script>
  import { Card, CardHeader, Button, Icon } from '@components/ui';

  let collapsed = false;
</script>

<!-- Basic Card -->
<Card>
  <p>Card content goes here</p>
</Card>

<!-- Card with Header -->
<Card>
  <svelte:fragment slot="header">
    <CardHeader
      title="Project Overview"
      subtitle="Last updated 2 hours ago"
    >
      <svelte:fragment slot="actions">
        <Button size="sm" variant="ghost" iconOnly>
          <Icon name="edit" />
        </Button>
      </svelte:fragment>
    </CardHeader>
  </svelte:fragment>

  <p>Card body content</p>

  <svelte:fragment slot="footer">
    <Button variant="ghost" size="sm">Cancel</Button>
    <Button variant="primary" size="sm">Save</Button>
  </svelte:fragment>
</Card>

<!-- Interactive Card -->
<Card interactive on:click={() => console.log('clicked')}>
  <h4>Click me</h4>
  <p>This card is clickable</p>
</Card>

<!-- Selected Card with Glow -->
<Card selected glow>
  <h4>Selected</h4>
  <p>This card shows selected state with glow</p>
</Card>

<!-- Collapsible Card -->
<Card collapsible bind:collapsed>
  <svelte:fragment slot="header">
    <CardHeader title="Collapsible Section" />
  </svelte:fragment>

  <p>This content can be collapsed</p>
</Card>

<!-- Link Card -->
<Card href="/projects/123" interactive>
  <h4>Project Alpha</h4>
  <p>Click to view project details</p>
</Card>
```

---

## Testing Requirements

### Unit Tests

```typescript
// tests/components/Card.test.ts
import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import Card from '@components/ui/Card/Card.svelte';

describe('Card', () => {
  it('should render basic card', () => {
    const { container } = render(Card, {
      slots: { default: '<p>Content</p>' }
    });

    expect(container.querySelector('.card')).toBeInTheDocument();
    expect(container.querySelector('.card-body')).toBeInTheDocument();
  });

  it('should render with different variants', () => {
    const { container, rerender } = render(Card, {
      props: { variant: 'elevated' }
    });

    expect(container.querySelector('.card-elevated')).toBeInTheDocument();

    rerender({ variant: 'outlined' });
    expect(container.querySelector('.card-outlined')).toBeInTheDocument();
  });

  it('should handle click when interactive', async () => {
    const handleClick = vi.fn();
    const { getByRole, component } = render(Card, {
      props: { interactive: true }
    });

    component.$on('click', handleClick);
    await fireEvent.click(getByRole('button'));

    expect(handleClick).toHaveBeenCalled();
  });

  it('should render as link when href provided', () => {
    const { container } = render(Card, {
      props: { href: '/test' }
    });

    const link = container.querySelector('a.card');
    expect(link).toBeInTheDocument();
    expect(link).toHaveAttribute('href', '/test');
  });

  it('should toggle collapsed state', async () => {
    const { container, component } = render(Card, {
      props: { collapsible: true, collapsed: false }
    });

    expect(container.querySelector('.card-body')).toBeInTheDocument();

    await fireEvent.click(container.querySelector('.card')!);

    // After toggle, body should be hidden
    expect(component.collapsed).toBe(true);
  });

  it('should show glow effect', () => {
    const { container } = render(Card, {
      props: { glow: true }
    });

    expect(container.querySelector('.card-glow')).toBeInTheDocument();
  });

  it('should show selected state', () => {
    const { container } = render(Card, {
      props: { selected: true }
    });

    expect(container.querySelector('.card-selected')).toBeInTheDocument();
  });
});
```

---

## Related Specs

- [196-component-library.md](./196-component-library.md) - Component library setup
- [202-modal-component.md](./202-modal-component.md) - Modal component
- [195-shadows-elevation.md](./195-shadows-elevation.md) - Shadows and elevation
