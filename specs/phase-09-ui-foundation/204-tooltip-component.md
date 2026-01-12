# Spec 204: Tooltip Component

## Phase
Phase 9: UI Foundation

## Spec ID
204

## Status
Planned

## Dependencies
- Spec 196: Component Library Setup
- Spec 191-195: Design System

## Estimated Context
~8%

---

## Objective

Implement a Tooltip component for Tachikoma with smart positioning, delay controls, keyboard accessibility, and support for rich content while maintaining the Tachikoma visual style.

---

## Acceptance Criteria

- [x] Multiple placement options (top, bottom, left, right + variations)
- [x] Smart auto-positioning to stay in viewport
- [x] Configurable show/hide delays
- [x] Support for text and rich content
- [x] Arrow pointer
- [x] Keyboard accessible (show on focus)
- [x] ARIA tooltip pattern compliance
- [x] Optional Tachikoma glow effect
- [x] Portal-based rendering

---

## Implementation Details

### src/lib/components/ui/Tooltip/Tooltip.svelte

```svelte
<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { fade, scale } from 'svelte/transition';
  import { generateId, cn, portal } from '@utils/component';

  type TooltipPlacement =
    | 'top' | 'top-start' | 'top-end'
    | 'bottom' | 'bottom-start' | 'bottom-end'
    | 'left' | 'left-start' | 'left-end'
    | 'right' | 'right-start' | 'right-end';

  export let content: string = '';
  export let placement: TooltipPlacement = 'top';
  export let showDelay: number = 200;
  export let hideDelay: number = 0;
  export let disabled: boolean = false;
  export let glow: boolean = false;
  export let maxWidth: number = 300;
  export let offset: number = 8;
  let className: string = '';
  export { className as class };

  const id = generateId('tooltip');
  let triggerElement: HTMLElement;
  let tooltipElement: HTMLElement;
  let isVisible = false;
  let showTimeoutId: ReturnType<typeof setTimeout>;
  let hideTimeoutId: ReturnType<typeof setTimeout>;
  let position = { x: 0, y: 0 };
  let actualPlacement = placement;

  function show() {
    if (disabled) return;

    clearTimeout(hideTimeoutId);
    showTimeoutId = setTimeout(async () => {
      isVisible = true;
      await tick();
      updatePosition();
    }, showDelay);
  }

  function hide() {
    clearTimeout(showTimeoutId);
    hideTimeoutId = setTimeout(() => {
      isVisible = false;
    }, hideDelay);
  }

  function updatePosition() {
    if (!triggerElement || !tooltipElement) return;

    const triggerRect = triggerElement.getBoundingClientRect();
    const tooltipRect = tooltipElement.getBoundingClientRect();
    const viewportWidth = window.innerWidth;
    const viewportHeight = window.innerHeight;

    let x = 0;
    let y = 0;

    // Calculate base position
    const positions = {
      top: {
        x: triggerRect.left + triggerRect.width / 2 - tooltipRect.width / 2,
        y: triggerRect.top - tooltipRect.height - offset
      },
      'top-start': {
        x: triggerRect.left,
        y: triggerRect.top - tooltipRect.height - offset
      },
      'top-end': {
        x: triggerRect.right - tooltipRect.width,
        y: triggerRect.top - tooltipRect.height - offset
      },
      bottom: {
        x: triggerRect.left + triggerRect.width / 2 - tooltipRect.width / 2,
        y: triggerRect.bottom + offset
      },
      'bottom-start': {
        x: triggerRect.left,
        y: triggerRect.bottom + offset
      },
      'bottom-end': {
        x: triggerRect.right - tooltipRect.width,
        y: triggerRect.bottom + offset
      },
      left: {
        x: triggerRect.left - tooltipRect.width - offset,
        y: triggerRect.top + triggerRect.height / 2 - tooltipRect.height / 2
      },
      'left-start': {
        x: triggerRect.left - tooltipRect.width - offset,
        y: triggerRect.top
      },
      'left-end': {
        x: triggerRect.left - tooltipRect.width - offset,
        y: triggerRect.bottom - tooltipRect.height
      },
      right: {
        x: triggerRect.right + offset,
        y: triggerRect.top + triggerRect.height / 2 - tooltipRect.height / 2
      },
      'right-start': {
        x: triggerRect.right + offset,
        y: triggerRect.top
      },
      'right-end': {
        x: triggerRect.right + offset,
        y: triggerRect.bottom - tooltipRect.height
      }
    };

    const pos = positions[placement];
    x = pos.x;
    y = pos.y;

    // Adjust for viewport boundaries
    actualPlacement = placement;

    if (x < 0) {
      x = offset;
    } else if (x + tooltipRect.width > viewportWidth) {
      x = viewportWidth - tooltipRect.width - offset;
    }

    if (y < 0) {
      // Flip to bottom
      if (placement.startsWith('top')) {
        actualPlacement = placement.replace('top', 'bottom') as TooltipPlacement;
        y = triggerRect.bottom + offset;
      } else {
        y = offset;
      }
    } else if (y + tooltipRect.height > viewportHeight) {
      // Flip to top
      if (placement.startsWith('bottom')) {
        actualPlacement = placement.replace('bottom', 'top') as TooltipPlacement;
        y = triggerRect.top - tooltipRect.height - offset;
      } else {
        y = viewportHeight - tooltipRect.height - offset;
      }
    }

    position = { x, y };
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === 'Escape' && isVisible) {
      hide();
    }
  }

  onMount(() => {
    window.addEventListener('scroll', hide, true);
    window.addEventListener('resize', hide);

    return () => {
      window.removeEventListener('scroll', hide, true);
      window.removeEventListener('resize', hide);
      clearTimeout(showTimeoutId);
      clearTimeout(hideTimeoutId);
    };
  });

  $: arrowClasses = {
    'top': 'bottom-[-4px] left-1/2 -translate-x-1/2 border-t-current border-l-transparent border-r-transparent border-b-transparent',
    'top-start': 'bottom-[-4px] left-4 border-t-current border-l-transparent border-r-transparent border-b-transparent',
    'top-end': 'bottom-[-4px] right-4 border-t-current border-l-transparent border-r-transparent border-b-transparent',
    'bottom': 'top-[-4px] left-1/2 -translate-x-1/2 border-b-current border-l-transparent border-r-transparent border-t-transparent',
    'bottom-start': 'top-[-4px] left-4 border-b-current border-l-transparent border-r-transparent border-t-transparent',
    'bottom-end': 'top-[-4px] right-4 border-b-current border-l-transparent border-r-transparent border-t-transparent',
    'left': 'right-[-4px] top-1/2 -translate-y-1/2 border-l-current border-t-transparent border-b-transparent border-r-transparent',
    'left-start': 'right-[-4px] top-3 border-l-current border-t-transparent border-b-transparent border-r-transparent',
    'left-end': 'right-[-4px] bottom-3 border-l-current border-t-transparent border-b-transparent border-r-transparent',
    'right': 'left-[-4px] top-1/2 -translate-y-1/2 border-r-current border-t-transparent border-b-transparent border-l-transparent',
    'right-start': 'left-[-4px] top-3 border-r-current border-t-transparent border-b-transparent border-l-transparent',
    'right-end': 'left-[-4px] bottom-3 border-r-current border-t-transparent border-b-transparent border-l-transparent'
  };
</script>

<span
  bind:this={triggerElement}
  class="tooltip-trigger"
  on:mouseenter={show}
  on:mouseleave={hide}
  on:focus={show}
  on:blur={hide}
  on:keydown={handleKeyDown}
  aria-describedby={isVisible ? id : undefined}
>
  <slot />
</span>

{#if isVisible && (content || $$slots.content)}
  <div
    bind:this={tooltipElement}
    {id}
    class={cn('tooltip', glow && 'tooltip-glow', className)}
    role="tooltip"
    style="
      left: {position.x}px;
      top: {position.y}px;
      max-width: {maxWidth}px;
    "
    transition:fade={{ duration: 100 }}
    use:portal={'body'}
  >
    <div class="tooltip-content">
      {#if $$slots.content}
        <slot name="content" />
      {:else}
        {content}
      {/if}
    </div>
    <span class="tooltip-arrow {arrowClasses[actualPlacement]}"></span>
  </div>
{/if}

<style>
  .tooltip-trigger {
    display: inline-flex;
  }

  .tooltip {
    position: fixed;
    z-index: var(--z-tooltip);
    padding: var(--spacing-2) var(--spacing-3);
    background-color: var(--color-bg-overlay);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-lg);
    pointer-events: none;
  }

  .tooltip-glow {
    box-shadow: var(--shadow-lg), var(--glow-sm);
    border-color: var(--tachikoma-500);
  }

  .tooltip-content {
    font-size: var(--text-xs);
    color: var(--color-fg-default);
    line-height: var(--leading-normal);
    word-wrap: break-word;
  }

  .tooltip-arrow {
    position: absolute;
    width: 0;
    height: 0;
    border-width: 4px;
    border-style: solid;
    color: var(--color-bg-overlay);
  }

  .tooltip-glow .tooltip-arrow {
    color: var(--color-bg-overlay);
  }
</style>
```

### src/lib/actions/tooltip.ts

```typescript
/**
 * Svelte action for simple tooltips
 */

import type { Action } from 'svelte/action';

interface TooltipOptions {
  content: string;
  placement?: 'top' | 'bottom' | 'left' | 'right';
  delay?: number;
}

export const tooltip: Action<HTMLElement, TooltipOptions> = (node, options) => {
  let tooltipElement: HTMLDivElement | null = null;
  let showTimeout: ReturnType<typeof setTimeout>;
  const { content, placement = 'top', delay = 200 } = options;

  function show() {
    showTimeout = setTimeout(() => {
      tooltipElement = document.createElement('div');
      tooltipElement.className = 'tooltip-simple';
      tooltipElement.textContent = content;
      tooltipElement.setAttribute('role', 'tooltip');

      document.body.appendChild(tooltipElement);
      position();
    }, delay);
  }

  function hide() {
    clearTimeout(showTimeout);
    if (tooltipElement) {
      tooltipElement.remove();
      tooltipElement = null;
    }
  }

  function position() {
    if (!tooltipElement) return;

    const rect = node.getBoundingClientRect();
    const tooltipRect = tooltipElement.getBoundingClientRect();
    const offset = 8;

    let x = 0;
    let y = 0;

    switch (placement) {
      case 'top':
        x = rect.left + rect.width / 2 - tooltipRect.width / 2;
        y = rect.top - tooltipRect.height - offset;
        break;
      case 'bottom':
        x = rect.left + rect.width / 2 - tooltipRect.width / 2;
        y = rect.bottom + offset;
        break;
      case 'left':
        x = rect.left - tooltipRect.width - offset;
        y = rect.top + rect.height / 2 - tooltipRect.height / 2;
        break;
      case 'right':
        x = rect.right + offset;
        y = rect.top + rect.height / 2 - tooltipRect.height / 2;
        break;
    }

    tooltipElement.style.left = `${x}px`;
    tooltipElement.style.top = `${y}px`;
  }

  node.addEventListener('mouseenter', show);
  node.addEventListener('mouseleave', hide);
  node.addEventListener('focus', show);
  node.addEventListener('blur', hide);

  return {
    update(newOptions: TooltipOptions) {
      if (tooltipElement && newOptions.content !== content) {
        tooltipElement.textContent = newOptions.content;
      }
    },
    destroy() {
      hide();
      node.removeEventListener('mouseenter', show);
      node.removeEventListener('mouseleave', hide);
      node.removeEventListener('focus', show);
      node.removeEventListener('blur', hide);
    }
  };
};
```

### Usage Examples

```svelte
<script>
  import { Tooltip, Button, Icon } from '@components/ui';
  import { tooltip } from '@lib/actions/tooltip';
</script>

<!-- Component-based Tooltip -->
<Tooltip content="This is a tooltip">
  <Button>Hover me</Button>
</Tooltip>

<!-- Different placements -->
<Tooltip content="Top tooltip" placement="top">
  <span>Top</span>
</Tooltip>

<Tooltip content="Bottom tooltip" placement="bottom">
  <span>Bottom</span>
</Tooltip>

<!-- With glow effect -->
<Tooltip content="Tachikoma glow" glow>
  <Button variant="primary">Glowing</Button>
</Tooltip>

<!-- Rich content -->
<Tooltip>
  <Button iconOnly>
    <Icon name="info" />
  </Button>

  <svelte:fragment slot="content">
    <strong>Keyboard Shortcut</strong>
    <p>Press Ctrl+K to open command palette</p>
  </svelte:fragment>
</Tooltip>

<!-- Action-based (simpler) -->
<button use:tooltip={{ content: 'Simple tooltip', placement: 'right' }}>
  Action tooltip
</button>
```

---

## Testing Requirements

### Unit Tests

```typescript
// tests/components/Tooltip.test.ts
import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent, waitFor } from '@testing-library/svelte';
import Tooltip from '@components/ui/Tooltip/Tooltip.svelte';

describe('Tooltip', () => {
  it('should not show tooltip by default', () => {
    const { queryByRole } = render(Tooltip, {
      props: { content: 'Test tooltip' },
      slots: { default: '<button>Trigger</button>' }
    });

    expect(queryByRole('tooltip')).not.toBeInTheDocument();
  });

  it('should show tooltip on hover', async () => {
    const { getByText, getByRole } = render(Tooltip, {
      props: { content: 'Test tooltip', showDelay: 0 },
      slots: { default: '<button>Trigger</button>' }
    });

    await fireEvent.mouseEnter(getByText('Trigger').parentElement!);

    await waitFor(() => {
      expect(getByRole('tooltip')).toBeInTheDocument();
      expect(getByRole('tooltip')).toHaveTextContent('Test tooltip');
    });
  });

  it('should hide tooltip on mouse leave', async () => {
    const { getByText, queryByRole } = render(Tooltip, {
      props: { content: 'Test tooltip', showDelay: 0, hideDelay: 0 },
      slots: { default: '<button>Trigger</button>' }
    });

    const trigger = getByText('Trigger').parentElement!;

    await fireEvent.mouseEnter(trigger);
    await waitFor(() => expect(queryByRole('tooltip')).toBeInTheDocument());

    await fireEvent.mouseLeave(trigger);
    await waitFor(() => expect(queryByRole('tooltip')).not.toBeInTheDocument());
  });

  it('should show on focus for accessibility', async () => {
    const { getByText, getByRole } = render(Tooltip, {
      props: { content: 'Test tooltip', showDelay: 0 },
      slots: { default: '<button>Trigger</button>' }
    });

    await fireEvent.focus(getByText('Trigger').parentElement!);

    await waitFor(() => {
      expect(getByRole('tooltip')).toBeInTheDocument();
    });
  });

  it('should not show when disabled', async () => {
    const { getByText, queryByRole } = render(Tooltip, {
      props: { content: 'Test tooltip', disabled: true, showDelay: 0 },
      slots: { default: '<button>Trigger</button>' }
    });

    await fireEvent.mouseEnter(getByText('Trigger').parentElement!);

    // Small delay to ensure timeout would have fired
    await new Promise(r => setTimeout(r, 50));

    expect(queryByRole('tooltip')).not.toBeInTheDocument();
  });
});
```

---

## Related Specs

- [196-component-library.md](./196-component-library.md) - Component library setup
- [203-toast-component.md](./203-toast-component.md) - Toast notifications
- [205-tabs-component.md](./205-tabs-component.md) - Tabs component
