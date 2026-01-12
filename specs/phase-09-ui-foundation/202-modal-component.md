# Spec 202: Modal Component

## Phase
Phase 9: UI Foundation

## Spec ID
202

## Status
Planned

## Dependencies
- Spec 196: Component Library Setup
- Spec 191-195: Design System

## Estimated Context
~10%

---

## Objective

Implement a comprehensive Modal (Dialog) component for Tachikoma with support for multiple sizes, focus trapping, accessible keyboard navigation, animations, and nested modal management.

---

## Acceptance Criteria

- [x] Multiple sizes (sm, md, lg, xl, fullscreen)
- [x] Backdrop click to close (configurable)
- [x] Focus trap within modal
- [x] Escape key to close
- [x] Accessible with ARIA attributes
- [x] Smooth enter/exit animations
- [x] Portal-based rendering
- [x] Prevent body scroll when open
- [x] Nested modal support
- [x] Confirmation dialog variant

---

## Implementation Details

### src/lib/components/ui/Modal/Modal.svelte

```svelte
<script lang="ts">
  import { createEventDispatcher, onMount, onDestroy, tick } from 'svelte';
  import { fade, scale } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import { portal, trapFocus, cn } from '@utils/component';
  import { generateId } from '@utils/component';
  import Icon from '../Icon/Icon.svelte';
  import Button from '../Button/Button.svelte';

  type ModalSize = 'sm' | 'md' | 'lg' | 'xl' | 'fullscreen';

  export let open: boolean = false;
  export let size: ModalSize = 'md';
  export let title: string | undefined = undefined;
  export let description: string | undefined = undefined;
  export let closeOnBackdrop: boolean = true;
  export let closeOnEscape: boolean = true;
  export let showCloseButton: boolean = true;
  export let preventScroll: boolean = true;
  export let initialFocus: HTMLElement | null = null;
  let className: string = '';
  export { className as class };

  const dispatch = createEventDispatcher<{
    close: void;
    open: void;
  }>();

  const id = generateId('modal');
  let modalElement: HTMLElement;
  let previouslyFocusedElement: HTMLElement | null = null;
  let cleanupTrapFocus: (() => void) | null = null;

  $: if (open) {
    handleOpen();
  } else {
    handleClose();
  }

  async function handleOpen() {
    await tick();

    // Store previously focused element
    previouslyFocusedElement = document.activeElement as HTMLElement;

    // Prevent body scroll
    if (preventScroll) {
      document.body.style.overflow = 'hidden';
    }

    // Setup focus trap
    if (modalElement) {
      cleanupTrapFocus = trapFocus(modalElement);

      // Focus initial element or first focusable
      if (initialFocus) {
        initialFocus.focus();
      } else {
        const firstFocusable = modalElement.querySelector<HTMLElement>(
          'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
        );
        firstFocusable?.focus();
      }
    }

    dispatch('open');
  }

  function handleClose() {
    // Restore body scroll
    if (preventScroll) {
      document.body.style.overflow = '';
    }

    // Cleanup focus trap
    if (cleanupTrapFocus) {
      cleanupTrapFocus();
      cleanupTrapFocus = null;
    }

    // Restore focus
    previouslyFocusedElement?.focus();
  }

  function close() {
    open = false;
    dispatch('close');
  }

  function handleBackdropClick(event: MouseEvent) {
    if (closeOnBackdrop && event.target === event.currentTarget) {
      close();
    }
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (closeOnEscape && event.key === 'Escape') {
      event.preventDefault();
      close();
    }
  }

  onMount(() => {
    document.addEventListener('keydown', handleKeyDown);
  });

  onDestroy(() => {
    document.removeEventListener('keydown', handleKeyDown);
    handleClose();
  });

  $: sizeClasses = {
    sm: 'modal-sm',
    md: 'modal-md',
    lg: 'modal-lg',
    xl: 'modal-xl',
    fullscreen: 'modal-fullscreen'
  };
</script>

{#if open}
  <div
    class="modal-backdrop"
    on:click={handleBackdropClick}
    transition:fade={{ duration: 150 }}
    use:portal={'body'}
  >
    <div
      bind:this={modalElement}
      class={cn('modal', sizeClasses[size], className)}
      role="dialog"
      aria-modal="true"
      aria-labelledby={title ? `${id}-title` : undefined}
      aria-describedby={description ? `${id}-description` : undefined}
      transition:scale={{ duration: 200, start: 0.95, easing: cubicOut }}
    >
      {#if title || showCloseButton || $$slots.header}
        <header class="modal-header">
          {#if $$slots.header}
            <slot name="header" />
          {:else}
            <div class="modal-header-content">
              {#if title}
                <h2 id="{id}-title" class="modal-title">{title}</h2>
              {/if}
              {#if description}
                <p id="{id}-description" class="modal-description">{description}</p>
              {/if}
            </div>
          {/if}

          {#if showCloseButton}
            <Button
              variant="ghost"
              size="sm"
              iconOnly
              class="modal-close"
              on:click={close}
              aria-label="Close modal"
            >
              <Icon name="x" size={20} />
            </Button>
          {/if}
        </header>
      {/if}

      <div class="modal-body">
        <slot />
      </div>

      {#if $$slots.footer}
        <footer class="modal-footer">
          <slot name="footer" />
        </footer>
      {/if}
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    z-index: var(--z-modal-backdrop);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--spacing-4);
    background-color: rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(4px);
  }

  .modal {
    position: relative;
    z-index: var(--z-modal);
    display: flex;
    flex-direction: column;
    max-height: calc(100vh - var(--spacing-8));
    background-color: var(--color-bg-overlay);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-xl);
    box-shadow: var(--modal-shadow);
    overflow: hidden;
  }

  /* Sizes */
  .modal-sm {
    width: 100%;
    max-width: 400px;
  }

  .modal-md {
    width: 100%;
    max-width: 560px;
  }

  .modal-lg {
    width: 100%;
    max-width: 720px;
  }

  .modal-xl {
    width: 100%;
    max-width: 960px;
  }

  .modal-fullscreen {
    width: calc(100vw - var(--spacing-8));
    height: calc(100vh - var(--spacing-8));
    max-width: none;
    max-height: none;
    border-radius: var(--radius-lg);
  }

  /* Header */
  .modal-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--spacing-4);
    padding: var(--spacing-6);
    border-bottom: 1px solid var(--color-border-subtle);
    flex-shrink: 0;
  }

  .modal-header-content {
    flex: 1;
    min-width: 0;
  }

  .modal-title {
    margin: 0;
    font-size: var(--text-lg);
    font-weight: var(--font-semibold);
    color: var(--color-fg-default);
    line-height: var(--leading-tight);
  }

  .modal-description {
    margin: var(--spacing-1) 0 0;
    font-size: var(--text-sm);
    color: var(--color-fg-muted);
    line-height: var(--leading-normal);
  }

  .modal :global(.modal-close) {
    flex-shrink: 0;
    margin: calc(var(--spacing-1) * -1);
  }

  /* Body */
  .modal-body {
    flex: 1;
    padding: var(--spacing-6);
    overflow-y: auto;
  }

  /* Footer */
  .modal-footer {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: var(--spacing-3);
    padding: var(--spacing-4) var(--spacing-6);
    border-top: 1px solid var(--color-border-subtle);
    background-color: var(--color-bg-surface);
    flex-shrink: 0;
  }
</style>
```

### src/lib/components/ui/Modal/ConfirmDialog.svelte

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import Modal from './Modal.svelte';
  import Button from '../Button/Button.svelte';
  import Icon from '../Icon/Icon.svelte';

  type ConfirmVariant = 'default' | 'danger' | 'warning';

  export let open: boolean = false;
  export let title: string = 'Confirm';
  export let message: string = 'Are you sure?';
  export let confirmText: string = 'Confirm';
  export let cancelText: string = 'Cancel';
  export let variant: ConfirmVariant = 'default';
  export let loading: boolean = false;

  const dispatch = createEventDispatcher<{
    confirm: void;
    cancel: void;
  }>();

  function handleConfirm() {
    dispatch('confirm');
  }

  function handleCancel() {
    open = false;
    dispatch('cancel');
  }

  $: iconName = variant === 'danger' ? 'alert-circle' : variant === 'warning' ? 'alert-triangle' : 'info';
  $: iconColor = variant === 'danger' ? 'var(--color-error-fg)' : variant === 'warning' ? 'var(--color-warning-fg)' : 'var(--color-info-fg)';
</script>

<Modal
  bind:open
  size="sm"
  closeOnBackdrop={!loading}
  closeOnEscape={!loading}
  showCloseButton={false}
>
  <div class="confirm-dialog">
    <div class="confirm-icon" style="color: {iconColor}">
      <Icon name={iconName} size={48} />
    </div>

    <h3 class="confirm-title">{title}</h3>
    <p class="confirm-message">{message}</p>

    <slot />
  </div>

  <svelte:fragment slot="footer">
    <Button
      variant="ghost"
      on:click={handleCancel}
      disabled={loading}
    >
      {cancelText}
    </Button>
    <Button
      variant={variant === 'danger' ? 'danger' : 'primary'}
      on:click={handleConfirm}
      {loading}
    >
      {confirmText}
    </Button>
  </svelte:fragment>
</Modal>

<style>
  .confirm-dialog {
    text-align: center;
    padding: var(--spacing-4) 0;
  }

  .confirm-icon {
    margin-bottom: var(--spacing-4);
  }

  .confirm-title {
    margin: 0 0 var(--spacing-2);
    font-size: var(--text-lg);
    font-weight: var(--font-semibold);
    color: var(--color-fg-default);
  }

  .confirm-message {
    margin: 0;
    font-size: var(--text-sm);
    color: var(--color-fg-muted);
    line-height: var(--leading-relaxed);
  }
</style>
```

### src/lib/stores/modal.ts

```typescript
import { writable, get } from 'svelte/store';

interface ModalState {
  id: string;
  component: any;
  props: Record<string, any>;
}

function createModalStore() {
  const { subscribe, set, update } = writable<ModalState[]>([]);

  return {
    subscribe,

    open: (component: any, props: Record<string, any> = {}) => {
      const id = crypto.randomUUID();

      update(modals => [
        ...modals,
        { id, component, props }
      ]);

      return id;
    },

    close: (id?: string) => {
      update(modals => {
        if (id) {
          return modals.filter(m => m.id !== id);
        }
        // Close topmost modal
        return modals.slice(0, -1);
      });
    },

    closeAll: () => {
      set([]);
    },

    isOpen: (id: string) => {
      const modals = get({ subscribe });
      return modals.some(m => m.id === id);
    }
  };
}

export const modalStore = createModalStore();

// Convenience functions
export function openModal(component: any, props?: Record<string, any>) {
  return modalStore.open(component, props);
}

export function closeModal(id?: string) {
  modalStore.close(id);
}

export function closeAllModals() {
  modalStore.closeAll();
}
```

---

## Testing Requirements

### Unit Tests

```typescript
// tests/components/Modal.test.ts
import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import Modal from '@components/ui/Modal/Modal.svelte';

describe('Modal', () => {
  it('should not render when closed', () => {
    const { queryByRole } = render(Modal, {
      props: { open: false }
    });

    expect(queryByRole('dialog')).not.toBeInTheDocument();
  });

  it('should render when open', () => {
    const { getByRole } = render(Modal, {
      props: { open: true }
    });

    expect(getByRole('dialog')).toBeInTheDocument();
  });

  it('should render title and description', () => {
    const { getByText } = render(Modal, {
      props: {
        open: true,
        title: 'Test Title',
        description: 'Test description'
      }
    });

    expect(getByText('Test Title')).toBeInTheDocument();
    expect(getByText('Test description')).toBeInTheDocument();
  });

  it('should close on backdrop click', async () => {
    const handleClose = vi.fn();
    const { container, component } = render(Modal, {
      props: { open: true, closeOnBackdrop: true }
    });

    component.$on('close', handleClose);

    const backdrop = container.querySelector('.modal-backdrop');
    await fireEvent.click(backdrop!);

    expect(handleClose).toHaveBeenCalled();
  });

  it('should not close on backdrop click when disabled', async () => {
    const handleClose = vi.fn();
    const { container, component } = render(Modal, {
      props: { open: true, closeOnBackdrop: false }
    });

    component.$on('close', handleClose);

    const backdrop = container.querySelector('.modal-backdrop');
    await fireEvent.click(backdrop!);

    expect(handleClose).not.toHaveBeenCalled();
  });

  it('should close on escape key', async () => {
    const handleClose = vi.fn();
    const { component } = render(Modal, {
      props: { open: true, closeOnEscape: true }
    });

    component.$on('close', handleClose);

    await fireEvent.keyDown(document, { key: 'Escape' });

    expect(handleClose).toHaveBeenCalled();
  });

  it('should render close button when enabled', () => {
    const { getByLabelText } = render(Modal, {
      props: { open: true, showCloseButton: true }
    });

    expect(getByLabelText('Close modal')).toBeInTheDocument();
  });

  it('should apply correct size class', () => {
    const { container } = render(Modal, {
      props: { open: true, size: 'lg' }
    });

    expect(container.querySelector('.modal-lg')).toBeInTheDocument();
  });

  it('should have correct ARIA attributes', () => {
    const { getByRole } = render(Modal, {
      props: {
        open: true,
        title: 'Test'
      }
    });

    const dialog = getByRole('dialog');
    expect(dialog).toHaveAttribute('aria-modal', 'true');
    expect(dialog).toHaveAttribute('aria-labelledby');
  });
});
```

---

## Related Specs

- [196-component-library.md](./196-component-library.md) - Component library setup
- [203-toast-component.md](./203-toast-component.md) - Toast notifications
- [212-error-boundaries.md](./212-error-boundaries.md) - Error handling
