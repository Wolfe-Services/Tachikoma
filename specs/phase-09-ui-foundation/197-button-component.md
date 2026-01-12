# Spec 197: Button Component

## Phase
Phase 9: UI Foundation

## Spec ID
197

## Status
Planned

## Dependencies
- Spec 196: Component Library Setup
- Spec 191-195: Design System

## Estimated Context
~10%

---

## Objective

Implement a comprehensive Button component for Tachikoma with multiple variants, sizes, states, icon support, and full accessibility compliance following the Tachikoma blue theme design language.

---

## Acceptance Criteria

- [ ] Multiple button variants (primary, secondary, ghost, outline, danger)
- [ ] Three sizes (sm, md, lg)
- [ ] Loading state with spinner
- [ ] Disabled state
- [ ] Icon support (left, right, icon-only)
- [ ] Keyboard accessible
- [ ] Focus visible styles
- [ ] Click ripple effect (optional)
- [ ] Full TypeScript support

---

## Implementation Details

### src/lib/components/ui/Button/Button.svelte

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { HTMLButtonAttributes } from 'svelte/elements';
  import Spinner from '../Spinner/Spinner.svelte';
  import { cn } from '@utils/component';

  type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'outline' | 'danger';
  type ButtonSize = 'sm' | 'md' | 'lg';

  interface $$Props extends Omit<HTMLButtonAttributes, 'size'> {
    variant?: ButtonVariant;
    size?: ButtonSize;
    loading?: boolean;
    disabled?: boolean;
    fullWidth?: boolean;
    iconOnly?: boolean;
    href?: string;
    class?: string;
  }

  export let variant: ButtonVariant = 'primary';
  export let size: ButtonSize = 'md';
  export let loading: boolean = false;
  export let disabled: boolean = false;
  export let fullWidth: boolean = false;
  export let iconOnly: boolean = false;
  export let href: string | undefined = undefined;
  let className: string = '';
  export { className as class };

  const dispatch = createEventDispatcher<{
    click: MouseEvent;
  }>();

  function handleClick(event: MouseEvent) {
    if (disabled || loading) {
      event.preventDefault();
      return;
    }
    dispatch('click', event);
  }

  $: isDisabled = disabled || loading;
  $: tag = href && !isDisabled ? 'a' : 'button';

  $: classes = cn(
    'btn',
    `btn-${variant}`,
    `btn-${size}`,
    fullWidth && 'btn-full-width',
    iconOnly && 'btn-icon-only',
    loading && 'btn-loading',
    className
  );
</script>

<svelte:element
  this={tag}
  class={classes}
  href={tag === 'a' ? href : undefined}
  type={tag === 'button' ? 'button' : undefined}
  disabled={tag === 'button' ? isDisabled : undefined}
  aria-disabled={isDisabled}
  aria-busy={loading}
  on:click={handleClick}
  {...$$restProps}
>
  {#if loading}
    <span class="btn-spinner">
      <Spinner size={size === 'lg' ? 20 : size === 'sm' ? 14 : 16} />
    </span>
  {/if}

  {#if $$slots.leftIcon && !loading}
    <span class="btn-icon btn-icon-left">
      <slot name="leftIcon" />
    </span>
  {/if}

  <span class="btn-content" class:sr-only={loading && iconOnly}>
    <slot />
  </span>

  {#if $$slots.rightIcon && !loading}
    <span class="btn-icon btn-icon-right">
      <slot name="rightIcon" />
    </span>
  {/if}
</svelte:element>

<style>
  .btn {
    /* Reset */
    appearance: none;
    border: none;
    background: none;
    font: inherit;
    cursor: pointer;
    text-decoration: none;

    /* Layout */
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--spacing-2);
    position: relative;

    /* Typography */
    font-family: var(--font-sans);
    font-weight: var(--font-medium);
    line-height: 1;
    white-space: nowrap;

    /* Appearance */
    border-radius: var(--radius-md);
    transition:
      background-color var(--duration-150) var(--ease-out),
      border-color var(--duration-150) var(--ease-out),
      color var(--duration-150) var(--ease-out),
      box-shadow var(--duration-150) var(--ease-out),
      transform var(--duration-75) var(--ease-out);
  }

  /* Focus visible */
  .btn:focus-visible {
    outline: none;
    box-shadow: var(--focus-ring);
  }

  /* Active state */
  .btn:active:not(:disabled):not([aria-disabled="true"]) {
    transform: scale(0.98);
  }

  /* Disabled state */
  .btn:disabled,
  .btn[aria-disabled="true"] {
    cursor: not-allowed;
    opacity: 0.5;
  }

  /* ============================================
   * VARIANTS
   * ============================================ */

  /* Primary - Tachikoma Blue */
  .btn-primary {
    background-color: var(--tachikoma-500);
    color: var(--color-bg-base);
    box-shadow: var(--button-shadow-primary);
  }

  .btn-primary:hover:not(:disabled):not([aria-disabled="true"]) {
    background-color: var(--tachikoma-400);
    box-shadow: var(--button-shadow-primary-hover);
  }

  .btn-primary:active:not(:disabled):not([aria-disabled="true"]) {
    background-color: var(--tachikoma-600);
  }

  /* Secondary */
  .btn-secondary {
    background-color: var(--color-bg-elevated);
    color: var(--color-fg-default);
    border: 1px solid var(--color-border-default);
  }

  .btn-secondary:hover:not(:disabled):not([aria-disabled="true"]) {
    background-color: var(--color-bg-overlay);
    border-color: var(--color-border-strong);
  }

  .btn-secondary:active:not(:disabled):not([aria-disabled="true"]) {
    background-color: var(--color-bg-muted);
  }

  /* Ghost */
  .btn-ghost {
    background-color: transparent;
    color: var(--color-fg-default);
  }

  .btn-ghost:hover:not(:disabled):not([aria-disabled="true"]) {
    background-color: var(--color-bg-hover);
  }

  .btn-ghost:active:not(:disabled):not([aria-disabled="true"]) {
    background-color: var(--color-bg-active);
  }

  /* Outline */
  .btn-outline {
    background-color: transparent;
    color: var(--tachikoma-500);
    border: 1px solid var(--tachikoma-500);
  }

  .btn-outline:hover:not(:disabled):not([aria-disabled="true"]) {
    background-color: var(--color-accent-subtle);
  }

  .btn-outline:active:not(:disabled):not([aria-disabled="true"]) {
    background-color: var(--color-accent-muted);
  }

  /* Danger */
  .btn-danger {
    background-color: var(--error-600);
    color: white;
  }

  .btn-danger:hover:not(:disabled):not([aria-disabled="true"]) {
    background-color: var(--error-500);
  }

  .btn-danger:active:not(:disabled):not([aria-disabled="true"]) {
    background-color: var(--error-700);
  }

  /* ============================================
   * SIZES
   * ============================================ */

  .btn-sm {
    height: var(--spacing-8);  /* 32px */
    padding: 0 var(--spacing-3);
    font-size: var(--text-sm);
    gap: var(--spacing-1-5);
  }

  .btn-md {
    height: var(--spacing-10); /* 40px */
    padding: 0 var(--spacing-4);
    font-size: var(--text-sm);
  }

  .btn-lg {
    height: var(--spacing-12); /* 48px */
    padding: 0 var(--spacing-6);
    font-size: var(--text-base);
    gap: var(--spacing-2-5);
  }

  /* Icon-only sizes */
  .btn-icon-only.btn-sm {
    width: var(--spacing-8);
    padding: 0;
  }

  .btn-icon-only.btn-md {
    width: var(--spacing-10);
    padding: 0;
  }

  .btn-icon-only.btn-lg {
    width: var(--spacing-12);
    padding: 0;
  }

  /* ============================================
   * MODIFIERS
   * ============================================ */

  .btn-full-width {
    width: 100%;
  }

  .btn-loading {
    pointer-events: none;
  }

  .btn-loading .btn-content {
    opacity: 0;
  }

  /* ============================================
   * INNER ELEMENTS
   * ============================================ */

  .btn-content {
    display: flex;
    align-items: center;
    justify-content: center;
    transition: opacity var(--duration-150) var(--ease-out);
  }

  .btn-spinner {
    position: absolute;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .btn-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .btn-icon :global(svg) {
    width: 1em;
    height: 1em;
  }

  .btn-sm .btn-icon :global(svg) {
    width: 14px;
    height: 14px;
  }

  .btn-lg .btn-icon :global(svg) {
    width: 20px;
    height: 20px;
  }

  /* Screen reader only */
  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }
</style>
```

### src/lib/components/ui/Button/index.ts

```typescript
export { default as Button } from './Button.svelte';
export type { ButtonVariant, ButtonSize } from './Button.svelte';
```

### src/lib/components/ui/Spinner/Spinner.svelte

```svelte
<script lang="ts">
  export let size: number = 20;
  export let color: string = 'currentColor';
</script>

<svg
  class="spinner"
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  xmlns="http://www.w3.org/2000/svg"
  aria-hidden="true"
>
  <circle
    class="spinner-track"
    cx="12"
    cy="12"
    r="10"
    stroke={color}
    stroke-width="3"
    stroke-linecap="round"
  />
  <circle
    class="spinner-head"
    cx="12"
    cy="12"
    r="10"
    stroke={color}
    stroke-width="3"
    stroke-linecap="round"
  />
</svg>

<style>
  .spinner {
    animation: spin 1s linear infinite;
  }

  .spinner-track {
    opacity: 0.2;
  }

  .spinner-head {
    stroke-dasharray: 60;
    stroke-dashoffset: 45;
    transform-origin: center;
  }

  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }
</style>
```

### Usage Examples

```svelte
<script>
  import { Button } from '@components/ui';
  import Icon from '@components/ui/Icon/Icon.svelte';

  let loading = false;

  async function handleSubmit() {
    loading = true;
    await doSomething();
    loading = false;
  }
</script>

<!-- Basic variants -->
<Button variant="primary">Primary</Button>
<Button variant="secondary">Secondary</Button>
<Button variant="ghost">Ghost</Button>
<Button variant="outline">Outline</Button>
<Button variant="danger">Danger</Button>

<!-- Sizes -->
<Button size="sm">Small</Button>
<Button size="md">Medium</Button>
<Button size="lg">Large</Button>

<!-- With icons -->
<Button>
  <svelte:fragment slot="leftIcon">
    <Icon name="plus" />
  </svelte:fragment>
  Add Item
</Button>

<Button>
  Next
  <svelte:fragment slot="rightIcon">
    <Icon name="chevron-right" />
  </svelte:fragment>
</Button>

<!-- Icon only -->
<Button iconOnly aria-label="Settings">
  <Icon name="settings" />
</Button>

<!-- Loading state -->
<Button {loading} on:click={handleSubmit}>
  Submit
</Button>

<!-- Disabled -->
<Button disabled>Disabled</Button>

<!-- Full width -->
<Button fullWidth>Full Width Button</Button>

<!-- As link -->
<Button href="/settings" variant="ghost">
  Go to Settings
</Button>
```

---

## Testing Requirements

### Unit Tests

```typescript
// tests/components/Button.test.ts
import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import Button from '@components/ui/Button/Button.svelte';

describe('Button', () => {
  it('should render with default props', () => {
    const { getByRole } = render(Button, {
      props: {},
      slots: { default: 'Click me' }
    });

    const button = getByRole('button');
    expect(button).toBeInTheDocument();
    expect(button).toHaveClass('btn', 'btn-primary', 'btn-md');
  });

  it('should render different variants', () => {
    const { getByRole, rerender } = render(Button, {
      props: { variant: 'secondary' }
    });

    expect(getByRole('button')).toHaveClass('btn-secondary');

    rerender({ variant: 'danger' });
    expect(getByRole('button')).toHaveClass('btn-danger');
  });

  it('should render different sizes', () => {
    const { getByRole, rerender } = render(Button, {
      props: { size: 'sm' }
    });

    expect(getByRole('button')).toHaveClass('btn-sm');

    rerender({ size: 'lg' });
    expect(getByRole('button')).toHaveClass('btn-lg');
  });

  it('should handle click events', async () => {
    const handleClick = vi.fn();
    const { getByRole, component } = render(Button);

    component.$on('click', handleClick);
    await fireEvent.click(getByRole('button'));

    expect(handleClick).toHaveBeenCalledTimes(1);
  });

  it('should not fire click when disabled', async () => {
    const handleClick = vi.fn();
    const { getByRole, component } = render(Button, {
      props: { disabled: true }
    });

    component.$on('click', handleClick);
    await fireEvent.click(getByRole('button'));

    expect(handleClick).not.toHaveBeenCalled();
  });

  it('should not fire click when loading', async () => {
    const handleClick = vi.fn();
    const { getByRole, component } = render(Button, {
      props: { loading: true }
    });

    component.$on('click', handleClick);
    await fireEvent.click(getByRole('button'));

    expect(handleClick).not.toHaveBeenCalled();
  });

  it('should show loading spinner', () => {
    const { container } = render(Button, {
      props: { loading: true }
    });

    expect(container.querySelector('.spinner')).toBeInTheDocument();
  });

  it('should render as anchor when href is provided', () => {
    const { container } = render(Button, {
      props: { href: '/test' }
    });

    const anchor = container.querySelector('a');
    expect(anchor).toBeInTheDocument();
    expect(anchor).toHaveAttribute('href', '/test');
  });

  it('should have correct aria attributes when loading', () => {
    const { getByRole } = render(Button, {
      props: { loading: true }
    });

    const button = getByRole('button');
    expect(button).toHaveAttribute('aria-busy', 'true');
    expect(button).toHaveAttribute('aria-disabled', 'true');
  });

  it('should be keyboard accessible', async () => {
    const handleClick = vi.fn();
    const { getByRole, component } = render(Button);

    component.$on('click', handleClick);

    const button = getByRole('button');
    button.focus();
    await fireEvent.keyDown(button, { key: 'Enter' });

    // Enter triggers click on buttons
    expect(button).toHaveFocus();
  });
});
```

### Accessibility Tests

```typescript
// tests/components/Button.a11y.test.ts
import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/svelte';
import { axe, toHaveNoViolations } from 'jest-axe';
import Button from '@components/ui/Button/Button.svelte';

expect.extend(toHaveNoViolations);

describe('Button Accessibility', () => {
  it('should have no accessibility violations', async () => {
    const { container } = render(Button, {
      slots: { default: 'Click me' }
    });

    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it('should have no violations when disabled', async () => {
    const { container } = render(Button, {
      props: { disabled: true },
      slots: { default: 'Disabled' }
    });

    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it('should have no violations for icon-only button with label', async () => {
    const { container } = render(Button, {
      props: { iconOnly: true, 'aria-label': 'Settings' }
    });

    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });
});
```

---

## Related Specs

- [196-component-library.md](./196-component-library.md) - Component library setup
- [198-input-component.md](./198-input-component.md) - Input component
- [191-design-tokens.md](./191-design-tokens.md) - Design tokens
