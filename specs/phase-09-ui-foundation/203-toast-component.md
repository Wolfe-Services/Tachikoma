# Spec 203: Toast Component

## Phase
Phase 9: UI Foundation

## Spec ID
203

## Status
Planned

## Dependencies
- Spec 196: Component Library Setup
- Spec 191-195: Design System

## Estimated Context
~8%

---

## Objective

Implement a Toast notification system for Tachikoma with support for multiple variants, auto-dismiss, stacking, actions, and accessible announcements for screen readers.

---

## Acceptance Criteria

- [x] Multiple variants (success, error, warning, info)
- [x] Auto-dismiss with configurable duration
- [x] Manual dismiss with close button
- [x] Toast stacking and positioning
- [x] Action buttons within toasts
- [x] Progress bar for auto-dismiss
- [x] Pause on hover
- [x] ARIA live region announcements
- [x] Programmatic toast creation
- [x] Maximum toast limit

---

## Implementation Details

### src/lib/components/ui/Toast/Toast.svelte

```svelte
<script lang="ts">
  import { createEventDispatcher, onMount, onDestroy } from 'svelte';
  import { fly, fade } from 'svelte/transition';
  import { cn } from '@utils/component';
  import Icon from '../Icon/Icon.svelte';
  import Button from '../Button/Button.svelte';

  export type ToastVariant = 'success' | 'error' | 'warning' | 'info' | 'default';

  export let id: string;
  export let variant: ToastVariant = 'default';
  export let title: string | undefined = undefined;
  export let message: string;
  export let duration: number = 5000; // 0 = no auto-dismiss
  export let dismissible: boolean = true;
  export let showProgress: boolean = true;
  export let action: { label: string; onClick: () => void } | undefined = undefined;
  export let icon: string | undefined = undefined;
  export let pauseOnHover: boolean = true;

  const dispatch = createEventDispatcher<{
    dismiss: string;
  }>();

  let progressWidth = 100;
  let isPaused = false;
  let startTime: number;
  let remainingTime: number;
  let animationFrame: number;

  const variantIcons: Record<ToastVariant, string> = {
    success: 'check-circle',
    error: 'alert-circle',
    warning: 'alert-triangle',
    info: 'info',
    default: 'info'
  };

  $: displayIcon = icon || variantIcons[variant];

  onMount(() => {
    if (duration > 0) {
      startTime = Date.now();
      remainingTime = duration;
      startProgressAnimation();
    }
  });

  onDestroy(() => {
    if (animationFrame) {
      cancelAnimationFrame(animationFrame);
    }
  });

  function startProgressAnimation() {
    if (duration <= 0) return;

    function animate() {
      if (isPaused) {
        animationFrame = requestAnimationFrame(animate);
        return;
      }

      const elapsed = Date.now() - startTime;
      const remaining = Math.max(0, remainingTime - elapsed);
      progressWidth = (remaining / duration) * 100;

      if (remaining <= 0) {
        dismiss();
      } else {
        animationFrame = requestAnimationFrame(animate);
      }
    }

    animationFrame = requestAnimationFrame(animate);
  }

  function handleMouseEnter() {
    if (pauseOnHover && duration > 0) {
      isPaused = true;
      remainingTime = (progressWidth / 100) * duration;
    }
  }

  function handleMouseLeave() {
    if (pauseOnHover && duration > 0) {
      isPaused = false;
      startTime = Date.now();
    }
  }

  function dismiss() {
    dispatch('dismiss', id);
  }

  $: classes = cn(
    'toast',
    `toast-${variant}`
  );
</script>

<div
  class={classes}
  role="alert"
  aria-live={variant === 'error' ? 'assertive' : 'polite'}
  on:mouseenter={handleMouseEnter}
  on:mouseleave={handleMouseLeave}
  in:fly={{ x: 100, duration: 200 }}
  out:fade={{ duration: 150 }}
>
  <div class="toast-icon">
    <Icon name={displayIcon} size={20} />
  </div>

  <div class="toast-content">
    {#if title}
      <div class="toast-title">{title}</div>
    {/if}
    <div class="toast-message">{message}</div>

    {#if action}
      <Button
        variant="ghost"
        size="sm"
        class="toast-action"
        on:click={action.onClick}
      >
        {action.label}
      </Button>
    {/if}
  </div>

  {#if dismissible}
    <button
      type="button"
      class="toast-close"
      on:click={dismiss}
      aria-label="Dismiss notification"
    >
      <Icon name="x" size={16} />
    </button>
  {/if}

  {#if showProgress && duration > 0}
    <div class="toast-progress">
      <div
        class="toast-progress-bar"
        style="width: {progressWidth}%"
      ></div>
    </div>
  {/if}
</div>

<style>
  .toast {
    position: relative;
    display: flex;
    align-items: flex-start;
    gap: var(--spacing-3);
    width: 380px;
    max-width: calc(100vw - var(--spacing-8));
    padding: var(--spacing-4);
    background-color: var(--color-bg-overlay);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-lg);
    box-shadow: var(--toast-shadow);
    overflow: hidden;
  }

  /* Variants */
  .toast-success {
    border-left: 4px solid var(--color-success-fg);
  }

  .toast-success .toast-icon {
    color: var(--color-success-fg);
  }

  .toast-error {
    border-left: 4px solid var(--color-error-fg);
  }

  .toast-error .toast-icon {
    color: var(--color-error-fg);
  }

  .toast-warning {
    border-left: 4px solid var(--color-warning-fg);
  }

  .toast-warning .toast-icon {
    color: var(--color-warning-fg);
  }

  .toast-info {
    border-left: 4px solid var(--tachikoma-500);
  }

  .toast-info .toast-icon {
    color: var(--tachikoma-500);
  }

  .toast-default {
    border-left: 4px solid var(--color-border-strong);
  }

  .toast-default .toast-icon {
    color: var(--color-fg-muted);
  }

  .toast-icon {
    flex-shrink: 0;
    margin-top: 2px;
  }

  .toast-content {
    flex: 1;
    min-width: 0;
  }

  .toast-title {
    font-size: var(--text-sm);
    font-weight: var(--font-semibold);
    color: var(--color-fg-default);
    margin-bottom: var(--spacing-1);
  }

  .toast-message {
    font-size: var(--text-sm);
    color: var(--color-fg-muted);
    line-height: var(--leading-relaxed);
    word-break: break-word;
  }

  .toast :global(.toast-action) {
    margin-top: var(--spacing-2);
    padding-left: 0;
    padding-right: 0;
  }

  .toast-close {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    padding: 0;
    background: transparent;
    border: none;
    border-radius: var(--radius-sm);
    color: var(--color-fg-muted);
    cursor: pointer;
    transition: color var(--duration-150) var(--ease-out),
                background-color var(--duration-150) var(--ease-out);
  }

  .toast-close:hover {
    color: var(--color-fg-default);
    background-color: var(--color-bg-hover);
  }

  .toast-progress {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    height: 3px;
    background-color: var(--color-bg-muted);
  }

  .toast-progress-bar {
    height: 100%;
    background-color: var(--tachikoma-500);
    transition: width 100ms linear;
  }

  .toast-success .toast-progress-bar {
    background-color: var(--color-success-fg);
  }

  .toast-error .toast-progress-bar {
    background-color: var(--color-error-fg);
  }

  .toast-warning .toast-progress-bar {
    background-color: var(--color-warning-fg);
  }
</style>
```

### src/lib/components/ui/Toast/ToastContainer.svelte

```svelte
<script lang="ts">
  import { flip } from 'svelte/animate';
  import { toastStore, type Toast } from '@stores/toast';
  import ToastComponent from './Toast.svelte';

  export let position: 'top-right' | 'top-left' | 'bottom-right' | 'bottom-left' | 'top-center' | 'bottom-center' = 'bottom-right';

  function handleDismiss(event: CustomEvent<string>) {
    toastStore.remove(event.detail);
  }

  $: positionClasses = {
    'top-right': 'top-4 right-4',
    'top-left': 'top-4 left-4',
    'bottom-right': 'bottom-4 right-4',
    'bottom-left': 'bottom-4 left-4',
    'top-center': 'top-4 left-1/2 -translate-x-1/2',
    'bottom-center': 'bottom-4 left-1/2 -translate-x-1/2'
  };

  $: isTop = position.startsWith('top');
</script>

<div
  class="toast-container {positionClasses[position]}"
  class:flex-col-reverse={isTop}
  aria-live="polite"
  aria-label="Notifications"
>
  {#each $toastStore as toast (toast.id)}
    <div animate:flip={{ duration: 200 }}>
      <ToastComponent
        {...toast}
        on:dismiss={handleDismiss}
      />
    </div>
  {/each}
</div>

<style>
  .toast-container {
    position: fixed;
    z-index: var(--z-toast);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-3);
    pointer-events: none;
  }

  .toast-container > :global(*) {
    pointer-events: auto;
  }

  .flex-col-reverse {
    flex-direction: column-reverse;
  }
</style>
```

### src/lib/stores/toast.ts

```typescript
import { writable, get } from 'svelte/store';
import type { ToastVariant } from '@components/ui/Toast/Toast.svelte';

export interface Toast {
  id: string;
  variant: ToastVariant;
  title?: string;
  message: string;
  duration?: number;
  dismissible?: boolean;
  showProgress?: boolean;
  action?: {
    label: string;
    onClick: () => void;
  };
  icon?: string;
  pauseOnHover?: boolean;
}

export interface ToastOptions extends Omit<Toast, 'id' | 'variant'> {}

const MAX_TOASTS = 5;

function createToastStore() {
  const { subscribe, set, update } = writable<Toast[]>([]);

  function add(toast: Omit<Toast, 'id'>): string {
    const id = crypto.randomUUID();
    const newToast: Toast = {
      ...toast,
      id,
      duration: toast.duration ?? 5000,
      dismissible: toast.dismissible ?? true,
      showProgress: toast.showProgress ?? true,
      pauseOnHover: toast.pauseOnHover ?? true
    };

    update(toasts => {
      const updated = [...toasts, newToast];
      // Remove oldest if exceeding max
      if (updated.length > MAX_TOASTS) {
        return updated.slice(-MAX_TOASTS);
      }
      return updated;
    });

    return id;
  }

  function remove(id: string) {
    update(toasts => toasts.filter(t => t.id !== id));
  }

  function clear() {
    set([]);
  }

  return {
    subscribe,
    add,
    remove,
    clear,

    // Convenience methods
    success: (message: string, options?: ToastOptions) =>
      add({ variant: 'success', message, ...options }),

    error: (message: string, options?: ToastOptions) =>
      add({ variant: 'error', message, duration: 0, ...options }),

    warning: (message: string, options?: ToastOptions) =>
      add({ variant: 'warning', message, ...options }),

    info: (message: string, options?: ToastOptions) =>
      add({ variant: 'info', message, ...options }),

    default: (message: string, options?: ToastOptions) =>
      add({ variant: 'default', message, ...options })
  };
}

export const toastStore = createToastStore();

// Export convenience functions
export const toast = {
  success: toastStore.success,
  error: toastStore.error,
  warning: toastStore.warning,
  info: toastStore.info,
  show: toastStore.default,
  dismiss: toastStore.remove,
  clear: toastStore.clear
};
```

### Usage Examples

```svelte
<script>
  import { toast } from '@stores/toast';
  import { ToastContainer } from '@components/ui';

  function showSuccess() {
    toast.success('Operation completed successfully!');
  }

  function showError() {
    toast.error('Something went wrong. Please try again.');
  }

  function showWithAction() {
    toast.info('File uploaded', {
      action: {
        label: 'View',
        onClick: () => console.log('View clicked')
      }
    });
  }

  function showWithTitle() {
    toast.warning('Connection unstable', {
      title: 'Network Warning',
      message: 'Your connection appears to be unstable. Some features may not work properly.'
    });
  }
</script>

<!-- Add ToastContainer to your root layout -->
<ToastContainer position="bottom-right" />
```

---

## Testing Requirements

### Unit Tests

```typescript
// tests/components/Toast.test.ts
import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import { get } from 'svelte/store';
import Toast from '@components/ui/Toast/Toast.svelte';
import { toastStore, toast } from '@stores/toast';

describe('Toast Component', () => {
  it('should render with message', () => {
    const { getByText } = render(Toast, {
      props: {
        id: '1',
        message: 'Test message',
        variant: 'info'
      }
    });

    expect(getByText('Test message')).toBeInTheDocument();
  });

  it('should render title when provided', () => {
    const { getByText } = render(Toast, {
      props: {
        id: '1',
        title: 'Test Title',
        message: 'Test message',
        variant: 'success'
      }
    });

    expect(getByText('Test Title')).toBeInTheDocument();
  });

  it('should dispatch dismiss event', async () => {
    const handleDismiss = vi.fn();
    const { getByLabelText, component } = render(Toast, {
      props: {
        id: 'test-id',
        message: 'Test',
        variant: 'info',
        dismissible: true
      }
    });

    component.$on('dismiss', handleDismiss);
    await fireEvent.click(getByLabelText('Dismiss notification'));

    expect(handleDismiss).toHaveBeenCalledWith(
      expect.objectContaining({ detail: 'test-id' })
    );
  });
});

describe('Toast Store', () => {
  beforeEach(() => {
    toastStore.clear();
  });

  it('should add toast', () => {
    toast.success('Success!');
    const toasts = get(toastStore);
    expect(toasts).toHaveLength(1);
    expect(toasts[0].variant).toBe('success');
  });

  it('should remove toast', () => {
    const id = toast.info('Test');
    expect(get(toastStore)).toHaveLength(1);

    toast.dismiss(id);
    expect(get(toastStore)).toHaveLength(0);
  });

  it('should limit max toasts', () => {
    for (let i = 0; i < 10; i++) {
      toast.info(`Toast ${i}`);
    }

    expect(get(toastStore)).toHaveLength(5); // MAX_TOASTS
  });

  it('should clear all toasts', () => {
    toast.info('One');
    toast.info('Two');
    toast.clear();

    expect(get(toastStore)).toHaveLength(0);
  });
});
```

---

## Related Specs

- [196-component-library.md](./196-component-library.md) - Component library setup
- [202-modal-component.md](./202-modal-component.md) - Modal component
- [212-error-boundaries.md](./212-error-boundaries.md) - Error handling
