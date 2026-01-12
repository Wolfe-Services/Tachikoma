# Spec 196: Component Library Setup

## Phase
Phase 9: UI Foundation

## Spec ID
196

## Status
Planned

## Dependencies
- Spec 186: SvelteKit Setup
- Spec 191-195: Design System (tokens, typography, colors, spacing, shadows)

## Estimated Context
~10%

---

## Objective

Establish the foundational structure for Tachikoma's component library, including component organization, shared utilities, prop patterns, accessibility standards, and documentation setup for a consistent and maintainable UI component system.

---

## Acceptance Criteria

- [x] Component directory structure established
- [x] Base component utilities created
- [x] TypeScript prop interfaces defined
- [x] Accessibility utilities (a11y)
- [x] Component composition patterns
- [x] Event handling patterns
- [x] Slot patterns for customization
- [x] Component documentation system

---

## Implementation Details

### Component Directory Structure

```
src/lib/components/
├── ui/                      # Core UI components
│   ├── index.ts             # Barrel exports
│   ├── Button/
│   │   ├── Button.svelte
│   │   ├── Button.test.ts
│   │   └── index.ts
│   ├── Input/
│   ├── Select/
│   ├── Checkbox/
│   ├── Card/
│   ├── Modal/
│   ├── Toast/
│   ├── Tooltip/
│   ├── Tabs/
│   └── ...
├── layout/                  # Layout components
│   ├── AppShell.svelte
│   ├── Sidebar.svelte
│   ├── SplitPane.svelte
│   └── ...
├── features/                # Feature-specific components
│   ├── terminal/
│   ├── code-editor/
│   ├── ai-chat/
│   └── ...
└── icons/                   # Icon components
    ├── Icon.svelte
    ├── icons.ts
    └── ...
```

### src/lib/components/ui/index.ts

```typescript
/**
 * Tachikoma UI Component Library
 *
 * Central export for all UI components
 */

// Core components
export { default as Button } from './Button/Button.svelte';
export { default as Input } from './Input/Input.svelte';
export { default as Select } from './Select/Select.svelte';
export { default as Checkbox } from './Checkbox/Checkbox.svelte';
export { default as Toggle } from './Toggle/Toggle.svelte';

// Layout components
export { default as Card } from './Card/Card.svelte';
export { default as Stack } from './Stack/Stack.svelte';
export { default as Inline } from './Inline/Inline.svelte';

// Overlay components
export { default as Modal } from './Modal/Modal.svelte';
export { default as Toast } from './Toast/Toast.svelte';
export { default as Tooltip } from './Tooltip/Tooltip.svelte';
export { default as Dropdown } from './Dropdown/Dropdown.svelte';

// Navigation components
export { default as Tabs } from './Tabs/Tabs.svelte';
export { default as TabPanel } from './Tabs/TabPanel.svelte';
export { default as Accordion } from './Accordion/Accordion.svelte';
export { default as TreeView } from './TreeView/TreeView.svelte';

// Display components
export { default as Badge } from './Badge/Badge.svelte';
export { default as Avatar } from './Avatar/Avatar.svelte';
export { default as Progress } from './Progress/Progress.svelte';
export { default as Spinner } from './Spinner/Spinner.svelte';
export { default as Skeleton } from './Skeleton/Skeleton.svelte';

// Typography components
export { default as Text } from './Text/Text.svelte';
export { default as Heading } from './Heading/Heading.svelte';
export { default as Code } from './Code/Code.svelte';

// Type exports
export type * from './types';
```

### src/lib/components/ui/types.ts

```typescript
/**
 * Shared component type definitions
 */

// Size variants
export type Size = 'xs' | 'sm' | 'md' | 'lg' | 'xl';
export type ButtonSize = 'sm' | 'md' | 'lg';
export type InputSize = 'sm' | 'md' | 'lg';

// Color/variant types
export type ColorVariant = 'primary' | 'secondary' | 'success' | 'warning' | 'error' | 'info';
export type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'outline' | 'danger';
export type StatusVariant = 'success' | 'warning' | 'error' | 'info' | 'neutral';

// Common prop interfaces
export interface BaseProps {
  class?: string;
  id?: string;
  'data-testid'?: string;
}

export interface DisableableProps {
  disabled?: boolean;
}

export interface LoadingProps {
  loading?: boolean;
}

export interface SizeProps<T extends string = Size> {
  size?: T;
}

export interface VariantProps<T extends string = ColorVariant> {
  variant?: T;
}

// Form-related types
export interface FormFieldProps {
  name?: string;
  value?: string;
  required?: boolean;
  disabled?: boolean;
  readonly?: boolean;
}

export interface ValidationProps {
  error?: string | boolean;
  success?: boolean;
  helperText?: string;
}

// Event types
export interface ClickEvent extends MouseEvent {
  currentTarget: EventTarget & HTMLElement;
}

export interface ChangeEvent extends Event {
  currentTarget: EventTarget & HTMLInputElement;
}

export interface FocusEvent extends globalThis.FocusEvent {
  currentTarget: EventTarget & HTMLElement;
}

export interface KeyboardEvent extends globalThis.KeyboardEvent {
  currentTarget: EventTarget & HTMLElement;
}
```

### src/lib/utils/component.ts

```typescript
/**
 * Component utility functions
 */

import { tick } from 'svelte';

/**
 * Generate unique IDs for components
 */
let idCounter = 0;
export function generateId(prefix: string = 'tachi'): string {
  return `${prefix}-${++idCounter}`;
}

/**
 * Merge class names, filtering out falsy values
 */
export function cn(...classes: (string | false | null | undefined)[]): string {
  return classes.filter(Boolean).join(' ');
}

/**
 * Create a debounced function
 */
export function debounce<T extends (...args: any[]) => any>(
  fn: T,
  delay: number
): (...args: Parameters<T>) => void {
  let timeoutId: ReturnType<typeof setTimeout>;

  return (...args: Parameters<T>) => {
    clearTimeout(timeoutId);
    timeoutId = setTimeout(() => fn(...args), delay);
  };
}

/**
 * Create a throttled function
 */
export function throttle<T extends (...args: any[]) => any>(
  fn: T,
  limit: number
): (...args: Parameters<T>) => void {
  let inThrottle = false;

  return (...args: Parameters<T>) => {
    if (!inThrottle) {
      fn(...args);
      inThrottle = true;
      setTimeout(() => (inThrottle = false), limit);
    }
  };
}

/**
 * Wait for next tick and focus element
 */
export async function focusElement(element: HTMLElement | null): Promise<void> {
  await tick();
  element?.focus();
}

/**
 * Check if element is focusable
 */
export function isFocusable(element: Element): boolean {
  if (!(element instanceof HTMLElement)) return false;

  const focusableSelectors = [
    'a[href]',
    'button:not([disabled])',
    'input:not([disabled])',
    'select:not([disabled])',
    'textarea:not([disabled])',
    '[tabindex]:not([tabindex="-1"])'
  ];

  return focusableSelectors.some(selector => element.matches(selector));
}

/**
 * Get all focusable children of an element
 */
export function getFocusableChildren(container: HTMLElement): HTMLElement[] {
  const elements = container.querySelectorAll<HTMLElement>(
    'a[href], button:not([disabled]), input:not([disabled]), ' +
    'select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])'
  );
  return Array.from(elements);
}

/**
 * Trap focus within a container (for modals, dropdowns)
 */
export function trapFocus(container: HTMLElement): () => void {
  const focusableElements = getFocusableChildren(container);
  const firstElement = focusableElements[0];
  const lastElement = focusableElements[focusableElements.length - 1];

  function handleKeyDown(event: KeyboardEvent) {
    if (event.key !== 'Tab') return;

    if (event.shiftKey) {
      if (document.activeElement === firstElement) {
        event.preventDefault();
        lastElement?.focus();
      }
    } else {
      if (document.activeElement === lastElement) {
        event.preventDefault();
        firstElement?.focus();
      }
    }
  }

  container.addEventListener('keydown', handleKeyDown);

  return () => {
    container.removeEventListener('keydown', handleKeyDown);
  };
}

/**
 * Click outside handler
 */
export function clickOutside(
  node: HTMLElement,
  callback: () => void
): { destroy: () => void } {
  function handleClick(event: MouseEvent) {
    if (node && !node.contains(event.target as Node)) {
      callback();
    }
  }

  document.addEventListener('click', handleClick, true);

  return {
    destroy() {
      document.removeEventListener('click', handleClick, true);
    }
  };
}

/**
 * Portal action - moves element to specified target
 */
export function portal(
  node: HTMLElement,
  target: string | HTMLElement = 'body'
): { destroy: () => void } {
  const targetEl = typeof target === 'string'
    ? document.querySelector(target)
    : target;

  if (targetEl) {
    targetEl.appendChild(node);
  }

  return {
    destroy() {
      node.remove();
    }
  };
}
```

### src/lib/utils/a11y.ts

```typescript
/**
 * Accessibility utilities
 */

/**
 * ARIA live region announcer
 */
class LiveAnnouncer {
  private container: HTMLElement | null = null;

  private ensureContainer(): HTMLElement {
    if (!this.container) {
      this.container = document.createElement('div');
      this.container.setAttribute('aria-live', 'polite');
      this.container.setAttribute('aria-atomic', 'true');
      this.container.className = 'sr-only';
      document.body.appendChild(this.container);
    }
    return this.container;
  }

  announce(message: string, priority: 'polite' | 'assertive' = 'polite'): void {
    const container = this.ensureContainer();
    container.setAttribute('aria-live', priority);
    container.textContent = '';

    // Force DOM update
    requestAnimationFrame(() => {
      container.textContent = message;
    });
  }
}

export const announcer = new LiveAnnouncer();

/**
 * Get appropriate aria-describedby for form fields
 */
export function getAriaDescribedBy(
  id: string,
  hasError: boolean,
  hasHelperText: boolean
): string | undefined {
  const ids: string[] = [];
  if (hasError) ids.push(`${id}-error`);
  if (hasHelperText) ids.push(`${id}-helper`);
  return ids.length > 0 ? ids.join(' ') : undefined;
}

/**
 * Keyboard navigation helpers
 */
export const Keys = {
  Enter: 'Enter',
  Space: ' ',
  Escape: 'Escape',
  ArrowUp: 'ArrowUp',
  ArrowDown: 'ArrowDown',
  ArrowLeft: 'ArrowLeft',
  ArrowRight: 'ArrowRight',
  Home: 'Home',
  End: 'End',
  Tab: 'Tab'
} as const;

/**
 * Handle roving tabindex for lists/grids
 */
export function useRovingFocus(items: HTMLElement[]): {
  handleKeyDown: (event: KeyboardEvent, currentIndex: number) => void;
  setFocus: (index: number) => void;
} {
  function setFocus(index: number) {
    items.forEach((item, i) => {
      item.setAttribute('tabindex', i === index ? '0' : '-1');
    });
    items[index]?.focus();
  }

  function handleKeyDown(event: KeyboardEvent, currentIndex: number) {
    let nextIndex = currentIndex;

    switch (event.key) {
      case Keys.ArrowDown:
      case Keys.ArrowRight:
        nextIndex = (currentIndex + 1) % items.length;
        event.preventDefault();
        break;
      case Keys.ArrowUp:
      case Keys.ArrowLeft:
        nextIndex = (currentIndex - 1 + items.length) % items.length;
        event.preventDefault();
        break;
      case Keys.Home:
        nextIndex = 0;
        event.preventDefault();
        break;
      case Keys.End:
        nextIndex = items.length - 1;
        event.preventDefault();
        break;
    }

    if (nextIndex !== currentIndex) {
      setFocus(nextIndex);
    }
  }

  return { handleKeyDown, setFocus };
}

/**
 * Screen reader only styles
 */
export const srOnly = `
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border: 0;
`;
```

### src/lib/components/ui/Icon/Icon.svelte

```svelte
<script lang="ts">
  import { icons, type IconName } from './icons';

  export let name: IconName;
  export let size: number = 20;
  export let strokeWidth: number = 2;
  export let ariaLabel: string | undefined = undefined;
  export let ariaHidden: boolean = !ariaLabel;

  $: icon = icons[name];
  $: viewBox = icon?.viewBox || '0 0 24 24';
  $: paths = icon?.paths || [];
</script>

<svg
  class="icon"
  width={size}
  height={size}
  {viewBox}
  fill="none"
  stroke="currentColor"
  stroke-width={strokeWidth}
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden={ariaHidden}
  aria-label={ariaLabel}
  role={ariaLabel ? 'img' : undefined}
  {...$$restProps}
>
  {#each paths as path}
    {#if path.type === 'path'}
      <path d={path.d} />
    {:else if path.type === 'circle'}
      <circle cx={path.cx} cy={path.cy} r={path.r} />
    {:else if path.type === 'rect'}
      <rect x={path.x} y={path.y} width={path.width} height={path.height} rx={path.rx} />
    {:else if path.type === 'line'}
      <line x1={path.x1} y1={path.y1} x2={path.x2} y2={path.y2} />
    {:else if path.type === 'polyline'}
      <polyline points={path.points} />
    {/if}
  {/each}
</svg>

<style>
  .icon {
    flex-shrink: 0;
    display: inline-block;
    vertical-align: middle;
  }
</style>
```

### src/lib/components/ui/Icon/icons.ts

```typescript
/**
 * Icon definitions
 * Using a subset of Lucide-style icons
 */

type PathElement =
  | { type: 'path'; d: string }
  | { type: 'circle'; cx: number; cy: number; r: number }
  | { type: 'rect'; x: number; y: number; width: number; height: number; rx?: number }
  | { type: 'line'; x1: number; y1: number; x2: number; y2: number }
  | { type: 'polyline'; points: string };

interface IconDefinition {
  viewBox: string;
  paths: PathElement[];
}

export const icons: Record<string, IconDefinition> = {
  // Navigation
  'chevron-left': {
    viewBox: '0 0 24 24',
    paths: [{ type: 'path', d: 'M15 18l-6-6 6-6' }]
  },
  'chevron-right': {
    viewBox: '0 0 24 24',
    paths: [{ type: 'path', d: 'M9 18l6-6-6-6' }]
  },
  'chevron-down': {
    viewBox: '0 0 24 24',
    paths: [{ type: 'path', d: 'M6 9l6 6 6-6' }]
  },
  'chevron-up': {
    viewBox: '0 0 24 24',
    paths: [{ type: 'path', d: 'M18 15l-6-6-6 6' }]
  },
  'x': {
    viewBox: '0 0 24 24',
    paths: [
      { type: 'line', x1: 18, y1: 6, x2: 6, y2: 18 },
      { type: 'line', x1: 6, y1: 6, x2: 18, y2: 18 }
    ]
  },
  'menu': {
    viewBox: '0 0 24 24',
    paths: [
      { type: 'line', x1: 3, y1: 12, x2: 21, y2: 12 },
      { type: 'line', x1: 3, y1: 6, x2: 21, y2: 6 },
      { type: 'line', x1: 3, y1: 18, x2: 21, y2: 18 }
    ]
  },

  // Status
  'check': {
    viewBox: '0 0 24 24',
    paths: [{ type: 'polyline', points: '20 6 9 17 4 12' }]
  },
  'alert-circle': {
    viewBox: '0 0 24 24',
    paths: [
      { type: 'circle', cx: 12, cy: 12, r: 10 },
      { type: 'line', x1: 12, y1: 8, x2: 12, y2: 12 },
      { type: 'line', x1: 12, y1: 16, x2: 12.01, y2: 16 }
    ]
  },
  'info': {
    viewBox: '0 0 24 24',
    paths: [
      { type: 'circle', cx: 12, cy: 12, r: 10 },
      { type: 'line', x1: 12, y1: 16, x2: 12, y2: 12 },
      { type: 'line', x1: 12, y1: 8, x2: 12.01, y2: 8 }
    ]
  },

  // Actions
  'search': {
    viewBox: '0 0 24 24',
    paths: [
      { type: 'circle', cx: 11, cy: 11, r: 8 },
      { type: 'line', x1: 21, y1: 21, x2: 16.65, y2: 16.65 }
    ]
  },
  'plus': {
    viewBox: '0 0 24 24',
    paths: [
      { type: 'line', x1: 12, y1: 5, x2: 12, y2: 19 },
      { type: 'line', x1: 5, y1: 12, x2: 19, y2: 12 }
    ]
  },
  'minus': {
    viewBox: '0 0 24 24',
    paths: [{ type: 'line', x1: 5, y1: 12, x2: 19, y2: 12 }]
  },
  'edit': {
    viewBox: '0 0 24 24',
    paths: [
      { type: 'path', d: 'M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7' },
      { type: 'path', d: 'M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z' }
    ]
  },
  'trash': {
    viewBox: '0 0 24 24',
    paths: [
      { type: 'polyline', points: '3 6 5 6 21 6' },
      { type: 'path', d: 'M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2' }
    ]
  },

  // App specific
  'terminal': {
    viewBox: '0 0 24 24',
    paths: [
      { type: 'polyline', points: '4 17 10 11 4 5' },
      { type: 'line', x1: 12, y1: 19, x2: 20, y2: 19 }
    ]
  },
  'folder': {
    viewBox: '0 0 24 24',
    paths: [
      { type: 'path', d: 'M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z' }
    ]
  },
  'home': {
    viewBox: '0 0 24 24',
    paths: [
      { type: 'path', d: 'M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z' },
      { type: 'polyline', points: '9 22 9 12 15 12 15 22' }
    ]
  },
  'settings': {
    viewBox: '0 0 24 24',
    paths: [
      { type: 'circle', cx: 12, cy: 12, r: 3 },
      { type: 'path', d: 'M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z' }
    ]
  },
  'bot': {
    viewBox: '0 0 24 24',
    paths: [
      { type: 'rect', x: 3, y: 11, width: 18, height: 10, rx: 2 },
      { type: 'circle', cx: 12, cy: 5, r: 2 },
      { type: 'path', d: 'M12 7v4' },
      { type: 'line', x1: 8, y1: 16, x2: 8, y2: 16 },
      { type: 'line', x1: 16, y1: 16, x2: 16, y2: 16 }
    ]
  },
  'loader': {
    viewBox: '0 0 24 24',
    paths: [
      { type: 'line', x1: 12, y1: 2, x2: 12, y2: 6 },
      { type: 'line', x1: 12, y1: 18, x2: 12, y2: 22 },
      { type: 'line', x1: 4.93, y1: 4.93, x2: 7.76, y2: 7.76 },
      { type: 'line', x1: 16.24, y1: 16.24, x2: 19.07, y2: 19.07 },
      { type: 'line', x1: 2, y1: 12, x2: 6, y2: 12 },
      { type: 'line', x1: 18, y1: 12, x2: 22, y2: 12 },
      { type: 'line', x1: 4.93, y1: 19.07, x2: 7.76, y2: 16.24 },
      { type: 'line', x1: 16.24, y1: 7.76, x2: 19.07, y2: 4.93 }
    ]
  }
};

export type IconName = keyof typeof icons;
```

---

## Testing Requirements

### Unit Tests

```typescript
// tests/components/utils.test.ts
import { describe, it, expect, vi } from 'vitest';
import { cn, generateId, debounce, throttle } from '@utils/component';

describe('Component Utilities', () => {
  describe('cn', () => {
    it('should merge class names', () => {
      expect(cn('foo', 'bar')).toBe('foo bar');
    });

    it('should filter falsy values', () => {
      expect(cn('foo', false, null, undefined, 'bar')).toBe('foo bar');
    });
  });

  describe('generateId', () => {
    it('should generate unique IDs', () => {
      const id1 = generateId();
      const id2 = generateId();
      expect(id1).not.toBe(id2);
    });

    it('should use custom prefix', () => {
      const id = generateId('btn');
      expect(id).toMatch(/^btn-\d+$/);
    });
  });

  describe('debounce', () => {
    it('should debounce function calls', async () => {
      vi.useFakeTimers();
      const fn = vi.fn();
      const debounced = debounce(fn, 100);

      debounced();
      debounced();
      debounced();

      expect(fn).not.toHaveBeenCalled();

      vi.advanceTimersByTime(100);
      expect(fn).toHaveBeenCalledTimes(1);

      vi.useRealTimers();
    });
  });
});
```

---

## Related Specs

- [191-design-tokens.md](./191-design-tokens.md) - Design tokens
- [197-button-component.md](./197-button-component.md) - Button component
- [198-input-component.md](./198-input-component.md) - Input component
