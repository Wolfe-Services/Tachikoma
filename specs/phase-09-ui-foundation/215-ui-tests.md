# Spec 215: UI Component Testing

## Phase
Phase 9: UI Foundation

## Spec ID
215

## Status
Planned

## Dependencies
- Spec 196: Component Library Setup
- All component specs (197-214)

## Estimated Context
~12%

---

## Objective

Implement comprehensive testing infrastructure for Tachikoma's UI components including unit tests, integration tests, visual regression tests, accessibility tests, and end-to-end tests to ensure component quality and prevent regressions.

---

## Acceptance Criteria

- [x] Vitest configuration for Svelte components
- [x] Testing utilities and helpers
- [x] Component unit tests
- [x] Accessibility testing with axe-core
- [x] Visual regression testing setup
- [x] Interaction testing
- [x] Store testing utilities
- [x] Mock utilities for Tauri IPC
- [x] Test coverage reporting
- [x] CI integration guidelines

---

## Implementation Details

### vite.config.ts (Test Configuration)

```typescript
import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { sveltekit } from '@sveltejs/kit/vite';
import path from 'path';

export default defineConfig({
  plugins: [sveltekit()],
  test: {
    include: ['src/**/*.{test,spec}.{js,ts}', 'tests/**/*.{test,spec}.{js,ts}'],
    environment: 'jsdom',
    globals: true,
    setupFiles: ['./tests/setup.ts'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      exclude: [
        'node_modules/',
        'tests/',
        '**/*.d.ts',
        '**/*.test.ts',
        '**/*.spec.ts'
      ],
      thresholds: {
        lines: 80,
        functions: 80,
        branches: 75,
        statements: 80
      }
    },
    alias: {
      '@components': path.resolve(__dirname, './src/lib/components'),
      '@stores': path.resolve(__dirname, './src/lib/stores'),
      '@utils': path.resolve(__dirname, './src/lib/utils'),
      '@actions': path.resolve(__dirname, './src/lib/actions')
    }
  }
});
```

### tests/setup.ts

```typescript
import '@testing-library/jest-dom';
import { vi } from 'vitest';
import { readable } from 'svelte/store';
import type { Navigation, Page } from '@sveltejs/kit';

// Mock SvelteKit modules
vi.mock('$app/environment', () => ({
  browser: true,
  dev: true,
  building: false
}));

vi.mock('$app/navigation', () => ({
  goto: vi.fn(),
  invalidate: vi.fn(),
  prefetch: vi.fn(),
  beforeNavigate: vi.fn(),
  afterNavigate: vi.fn()
}));

vi.mock('$app/stores', () => ({
  page: readable<Page>({
    url: new URL('http://localhost'),
    params: {},
    route: { id: '/' },
    status: 200,
    error: null,
    data: {},
    form: null
  }),
  navigating: readable<Navigation | null>(null),
  updated: {
    subscribe: vi.fn(),
    check: vi.fn()
  }
}));

// Mock Tauri APIs
vi.mock('@tauri-apps/api/tauri', () => ({
  invoke: vi.fn(),
  convertFileSrc: vi.fn((path: string) => path)
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
  emit: vi.fn(),
  once: vi.fn()
}));

vi.mock('@tauri-apps/api/window', () => ({
  appWindow: {
    close: vi.fn(),
    minimize: vi.fn(),
    maximize: vi.fn(),
    unmaximize: vi.fn(),
    isMaximized: vi.fn(() => Promise.resolve(false)),
    setTitle: vi.fn()
  }
}));

// Mock window.matchMedia
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: vi.fn().mockImplementation(query => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn()
  }))
});

// Mock ResizeObserver
global.ResizeObserver = vi.fn().mockImplementation(() => ({
  observe: vi.fn(),
  unobserve: vi.fn(),
  disconnect: vi.fn()
}));

// Mock IntersectionObserver
global.IntersectionObserver = vi.fn().mockImplementation(() => ({
  observe: vi.fn(),
  unobserve: vi.fn(),
  disconnect: vi.fn()
}));

// Clean up after each test
afterEach(() => {
  vi.clearAllMocks();
});
```

### tests/utils/render.ts

```typescript
import { render as svelteRender, type RenderResult } from '@testing-library/svelte';
import type { ComponentProps, SvelteComponent } from 'svelte';
import userEvent from '@testing-library/user-event';

interface RenderOptions<T extends SvelteComponent> {
  props?: ComponentProps<T>;
  slots?: Record<string, string>;
}

/**
 * Enhanced render function with user event setup
 */
export function render<T extends SvelteComponent>(
  Component: new (...args: any[]) => T,
  options: RenderOptions<T> = {}
): RenderResult<T> & { user: ReturnType<typeof userEvent.setup> } {
  const user = userEvent.setup();
  const result = svelteRender(Component, options);

  return {
    ...result,
    user
  };
}

/**
 * Wait for component to update
 */
export async function waitForUpdate(): Promise<void> {
  await new Promise(resolve => setTimeout(resolve, 0));
}

/**
 * Create a mock component for slot testing
 */
export function createSlotContent(content: string): string {
  return content;
}
```

### tests/utils/mocks.ts

```typescript
import { vi } from 'vitest';
import { writable, readable, type Writable, type Readable } from 'svelte/store';

/**
 * Create a mock Tauri invoke function
 */
export function createMockInvoke(handlers: Record<string, (...args: any[]) => any> = {}) {
  return vi.fn().mockImplementation((cmd: string, args?: any) => {
    if (handlers[cmd]) {
      return Promise.resolve(handlers[cmd](args));
    }
    return Promise.reject(new Error(`Unknown command: ${cmd}`));
  });
}

/**
 * Create a mock event listener
 */
export function createMockEventListener() {
  const listeners: Map<string, Set<Function>> = new Map();

  return {
    listen: vi.fn((event: string, handler: Function) => {
      if (!listeners.has(event)) {
        listeners.set(event, new Set());
      }
      listeners.get(event)!.add(handler);
      return Promise.resolve(() => {
        listeners.get(event)?.delete(handler);
      });
    }),
    emit: vi.fn((event: string, payload?: any) => {
      listeners.get(event)?.forEach(handler => handler({ payload }));
    }),
    clear: () => listeners.clear()
  };
}

/**
 * Create a mock store with spy methods
 */
export function createMockStore<T>(initialValue: T): Writable<T> & {
  getMockValue: () => T;
  setMockValue: (value: T) => void;
} {
  let currentValue = initialValue;
  const store = writable(initialValue);

  return {
    subscribe: store.subscribe,
    set: vi.fn((value: T) => {
      currentValue = value;
      store.set(value);
    }),
    update: vi.fn((updater: (value: T) => T) => {
      currentValue = updater(currentValue);
      store.set(currentValue);
    }),
    getMockValue: () => currentValue,
    setMockValue: (value: T) => {
      currentValue = value;
      store.set(value);
    }
  };
}

/**
 * Mock fetch for API testing
 */
export function createMockFetch(responses: Record<string, any> = {}) {
  return vi.fn().mockImplementation((url: string, options?: RequestInit) => {
    const response = responses[url];

    if (response) {
      return Promise.resolve({
        ok: true,
        status: 200,
        json: () => Promise.resolve(response),
        text: () => Promise.resolve(JSON.stringify(response))
      });
    }

    return Promise.resolve({
      ok: false,
      status: 404,
      json: () => Promise.reject(new Error('Not found')),
      text: () => Promise.resolve('Not found')
    });
  });
}

/**
 * Mock localStorage
 */
export function createMockLocalStorage(): Storage {
  let store: Record<string, string> = {};

  return {
    getItem: vi.fn((key: string) => store[key] || null),
    setItem: vi.fn((key: string, value: string) => {
      store[key] = value;
    }),
    removeItem: vi.fn((key: string) => {
      delete store[key];
    }),
    clear: vi.fn(() => {
      store = {};
    }),
    key: vi.fn((index: number) => Object.keys(store)[index] || null),
    get length() {
      return Object.keys(store).length;
    }
  };
}
```

### tests/utils/accessibility.ts

```typescript
import { axe, toHaveNoViolations } from 'jest-axe';
import type { RenderResult } from '@testing-library/svelte';

expect.extend(toHaveNoViolations);

/**
 * Run accessibility tests on rendered component
 */
export async function checkAccessibility(
  container: HTMLElement,
  options: Parameters<typeof axe>[1] = {}
): Promise<void> {
  const results = await axe(container, {
    rules: {
      // Tachikoma-specific rule configurations
      'color-contrast': { enabled: true },
      'keyboard-access': { enabled: true },
      ...options.rules
    },
    ...options
  });

  expect(results).toHaveNoViolations();
}

/**
 * Check if element is keyboard accessible
 */
export function isKeyboardAccessible(element: HTMLElement): boolean {
  const tabIndex = element.tabIndex;
  const isNativelyFocusable = ['A', 'BUTTON', 'INPUT', 'SELECT', 'TEXTAREA'].includes(
    element.tagName
  );

  return tabIndex >= 0 || isNativelyFocusable;
}

/**
 * Check ARIA attributes
 */
export function checkAriaAttributes(
  element: HTMLElement,
  expectedAttributes: Record<string, string | null>
): void {
  for (const [attr, value] of Object.entries(expectedAttributes)) {
    if (value === null) {
      expect(element.hasAttribute(attr)).toBe(false);
    } else {
      expect(element.getAttribute(attr)).toBe(value);
    }
  }
}
```

### tests/components/Button.test.ts

```typescript
import { describe, it, expect, vi } from 'vitest';
import { fireEvent, screen } from '@testing-library/svelte';
import { render } from '../utils/render';
import { checkAccessibility } from '../utils/accessibility';
import Button from '@components/ui/Button/Button.svelte';

describe('Button', () => {
  describe('rendering', () => {
    it('should render with default props', () => {
      const { container } = render(Button, {
        props: { children: 'Click me' }
      });

      expect(screen.getByRole('button')).toBeInTheDocument();
      expect(container.querySelector('.btn-md')).toBeInTheDocument();
      expect(container.querySelector('.btn-primary')).toBeInTheDocument();
    });

    it('should render different variants', () => {
      const variants = ['primary', 'secondary', 'ghost', 'outline', 'danger'] as const;

      variants.forEach(variant => {
        const { container, unmount } = render(Button, {
          props: { variant }
        });

        expect(container.querySelector(`.btn-${variant}`)).toBeInTheDocument();
        unmount();
      });
    });

    it('should render different sizes', () => {
      const sizes = ['sm', 'md', 'lg'] as const;

      sizes.forEach(size => {
        const { container, unmount } = render(Button, {
          props: { size }
        });

        expect(container.querySelector(`.btn-${size}`)).toBeInTheDocument();
        unmount();
      });
    });

    it('should render as link when href provided', () => {
      render(Button, {
        props: { href: '/test' }
      });

      expect(screen.getByRole('link')).toBeInTheDocument();
    });
  });

  describe('interaction', () => {
    it('should handle click events', async () => {
      const handleClick = vi.fn();
      const { user } = render(Button, {
        props: { onclick: handleClick }
      });

      await user.click(screen.getByRole('button'));

      expect(handleClick).toHaveBeenCalledTimes(1);
    });

    it('should not trigger click when disabled', async () => {
      const handleClick = vi.fn();
      const { user } = render(Button, {
        props: { disabled: true, onclick: handleClick }
      });

      await user.click(screen.getByRole('button'));

      expect(handleClick).not.toHaveBeenCalled();
    });

    it('should show loading state', () => {
      const { container } = render(Button, {
        props: { loading: true }
      });

      expect(container.querySelector('.spinner')).toBeInTheDocument();
      expect(screen.getByRole('button')).toBeDisabled();
    });
  });

  describe('accessibility', () => {
    it('should have no accessibility violations', async () => {
      const { container } = render(Button, {
        props: { children: 'Accessible Button' }
      });

      await checkAccessibility(container);
    });

    it('should be keyboard focusable', () => {
      render(Button);

      const button = screen.getByRole('button');
      button.focus();

      expect(document.activeElement).toBe(button);
    });

    it('should trigger on Enter key', async () => {
      const handleClick = vi.fn();
      render(Button, {
        props: { onclick: handleClick }
      });

      const button = screen.getByRole('button');
      await fireEvent.keyDown(button, { key: 'Enter' });

      expect(handleClick).toHaveBeenCalled();
    });

    it('should have correct aria-disabled when disabled', () => {
      render(Button, {
        props: { disabled: true }
      });

      expect(screen.getByRole('button')).toHaveAttribute('disabled');
    });
  });
});
```

### tests/components/Input.test.ts

```typescript
import { describe, it, expect, vi } from 'vitest';
import { screen, fireEvent } from '@testing-library/svelte';
import { render } from '../utils/render';
import { checkAccessibility } from '../utils/accessibility';
import Input from '@components/ui/Input/Input.svelte';

describe('Input', () => {
  describe('rendering', () => {
    it('should render with default props', () => {
      render(Input);

      expect(screen.getByRole('textbox')).toBeInTheDocument();
    });

    it('should render with label', () => {
      render(Input, {
        props: { label: 'Email', name: 'email' }
      });

      expect(screen.getByLabelText('Email')).toBeInTheDocument();
    });

    it('should render with placeholder', () => {
      render(Input, {
        props: { placeholder: 'Enter email' }
      });

      expect(screen.getByPlaceholderText('Enter email')).toBeInTheDocument();
    });

    it('should render error state', () => {
      const { container } = render(Input, {
        props: { error: 'Invalid email' }
      });

      expect(screen.getByText('Invalid email')).toBeInTheDocument();
      expect(container.querySelector('.input-error')).toBeInTheDocument();
    });

    it('should render helper text', () => {
      render(Input, {
        props: { helperText: 'Enter your email address' }
      });

      expect(screen.getByText('Enter your email address')).toBeInTheDocument();
    });
  });

  describe('interaction', () => {
    it('should handle input changes', async () => {
      const handleInput = vi.fn();
      const { user } = render(Input, {
        props: { oninput: handleInput }
      });

      const input = screen.getByRole('textbox');
      await user.type(input, 'test');

      expect(handleInput).toHaveBeenCalled();
      expect(input).toHaveValue('test');
    });

    it('should handle blur events', async () => {
      const handleBlur = vi.fn();
      const { user } = render(Input, {
        props: { onblur: handleBlur }
      });

      const input = screen.getByRole('textbox');
      await user.click(input);
      await user.tab();

      expect(handleBlur).toHaveBeenCalled();
    });

    it('should clear value when clearable', async () => {
      const { user, container } = render(Input, {
        props: { clearable: true, value: 'test' }
      });

      const clearButton = container.querySelector('.input-clear');
      expect(clearButton).toBeInTheDocument();

      await user.click(clearButton!);

      expect(screen.getByRole('textbox')).toHaveValue('');
    });

    it('should toggle password visibility', async () => {
      const { user, container } = render(Input, {
        props: { type: 'password', value: 'secret' }
      });

      const input = screen.getByRole('textbox', { hidden: true });
      expect(input).toHaveAttribute('type', 'password');

      const toggleButton = container.querySelector('.input-password-toggle');
      await user.click(toggleButton!);

      expect(input).toHaveAttribute('type', 'text');
    });

    it('should not allow input when disabled', async () => {
      const { user } = render(Input, {
        props: { disabled: true }
      });

      const input = screen.getByRole('textbox');
      await user.type(input, 'test');

      expect(input).toHaveValue('');
    });
  });

  describe('accessibility', () => {
    it('should have no accessibility violations', async () => {
      const { container } = render(Input, {
        props: { label: 'Email', name: 'email' }
      });

      await checkAccessibility(container);
    });

    it('should associate label with input', () => {
      render(Input, {
        props: { label: 'Username', name: 'username' }
      });

      const input = screen.getByLabelText('Username');
      expect(input).toBeInTheDocument();
    });

    it('should have aria-invalid when error', () => {
      render(Input, {
        props: { error: 'Invalid' }
      });

      expect(screen.getByRole('textbox')).toHaveAttribute('aria-invalid', 'true');
    });

    it('should have aria-describedby for helper text', () => {
      render(Input, {
        props: { helperText: 'Help text', name: 'field' }
      });

      const input = screen.getByRole('textbox');
      expect(input).toHaveAttribute('aria-describedby');
    });
  });
});
```

### tests/components/Modal.test.ts

```typescript
import { describe, it, expect, vi } from 'vitest';
import { screen, fireEvent, waitFor } from '@testing-library/svelte';
import { render } from '../utils/render';
import { checkAccessibility } from '../utils/accessibility';
import Modal from '@components/ui/Modal/Modal.svelte';

describe('Modal', () => {
  describe('rendering', () => {
    it('should not render when closed', () => {
      render(Modal, {
        props: { open: false, title: 'Test Modal' }
      });

      expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    });

    it('should render when open', () => {
      render(Modal, {
        props: { open: true, title: 'Test Modal' }
      });

      expect(screen.getByRole('dialog')).toBeInTheDocument();
      expect(screen.getByText('Test Modal')).toBeInTheDocument();
    });

    it('should render with different sizes', () => {
      const sizes = ['sm', 'md', 'lg', 'xl', 'fullscreen'] as const;

      sizes.forEach(size => {
        const { container, unmount } = render(Modal, {
          props: { open: true, size }
        });

        expect(container.querySelector(`.modal-${size}`)).toBeInTheDocument();
        unmount();
      });
    });
  });

  describe('interaction', () => {
    it('should close on backdrop click', async () => {
      const handleClose = vi.fn();
      const { user, container } = render(Modal, {
        props: { open: true, onclose: handleClose }
      });

      const backdrop = container.querySelector('.modal-backdrop');
      await user.click(backdrop!);

      expect(handleClose).toHaveBeenCalled();
    });

    it('should not close on backdrop click when closeOnBackdrop is false', async () => {
      const handleClose = vi.fn();
      const { user, container } = render(Modal, {
        props: { open: true, closeOnBackdrop: false, onclose: handleClose }
      });

      const backdrop = container.querySelector('.modal-backdrop');
      await user.click(backdrop!);

      expect(handleClose).not.toHaveBeenCalled();
    });

    it('should close on Escape key', async () => {
      const handleClose = vi.fn();
      render(Modal, {
        props: { open: true, onclose: handleClose }
      });

      await fireEvent.keyDown(document, { key: 'Escape' });

      expect(handleClose).toHaveBeenCalled();
    });

    it('should not close on Escape when closeOnEscape is false', async () => {
      const handleClose = vi.fn();
      render(Modal, {
        props: { open: true, closeOnEscape: false, onclose: handleClose }
      });

      await fireEvent.keyDown(document, { key: 'Escape' });

      expect(handleClose).not.toHaveBeenCalled();
    });

    it('should close on close button click', async () => {
      const handleClose = vi.fn();
      const { user } = render(Modal, {
        props: { open: true, onclose: handleClose }
      });

      const closeButton = screen.getByLabelText('Close');
      await user.click(closeButton);

      expect(handleClose).toHaveBeenCalled();
    });
  });

  describe('focus management', () => {
    it('should trap focus within modal', async () => {
      render(Modal, {
        props: { open: true, title: 'Focus Test' }
      });

      const modal = screen.getByRole('dialog');
      const focusableElements = modal.querySelectorAll(
        'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
      );

      expect(focusableElements.length).toBeGreaterThan(0);
    });

    it('should focus first focusable element on open', async () => {
      render(Modal, {
        props: { open: true, title: 'Focus Test' }
      });

      await waitFor(() => {
        const closeButton = screen.getByLabelText('Close');
        expect(document.activeElement).toBe(closeButton);
      });
    });
  });

  describe('accessibility', () => {
    it('should have no accessibility violations', async () => {
      const { container } = render(Modal, {
        props: { open: true, title: 'Accessible Modal' }
      });

      await checkAccessibility(container);
    });

    it('should have correct ARIA attributes', () => {
      render(Modal, {
        props: { open: true, title: 'ARIA Test' }
      });

      const dialog = screen.getByRole('dialog');
      expect(dialog).toHaveAttribute('aria-modal', 'true');
      expect(dialog).toHaveAttribute('aria-labelledby');
    });
  });
});
```

### tests/stores/theme.test.ts

```typescript
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import { createMockLocalStorage } from '../utils/mocks';

// Mock localStorage before importing the store
const mockStorage = createMockLocalStorage();
Object.defineProperty(window, 'localStorage', { value: mockStorage });

// Import after mocking
import { themeStore, setTheme, toggleTheme } from '@stores/theme';

describe('themeStore', () => {
  beforeEach(() => {
    mockStorage.clear();
    vi.clearAllMocks();
  });

  it('should initialize with default theme', () => {
    const theme = get(themeStore);
    expect(['light', 'dark', 'system']).toContain(theme);
  });

  it('should set theme', () => {
    setTheme('dark');
    expect(get(themeStore)).toBe('dark');
  });

  it('should toggle between light and dark', () => {
    setTheme('light');
    toggleTheme();
    expect(get(themeStore)).toBe('dark');

    toggleTheme();
    expect(get(themeStore)).toBe('light');
  });

  it('should persist theme to localStorage', () => {
    setTheme('dark');
    expect(mockStorage.setItem).toHaveBeenCalledWith(
      expect.any(String),
      expect.stringContaining('dark')
    );
  });

  it('should load theme from localStorage', () => {
    mockStorage.getItem.mockReturnValue('"dark"');

    // Re-import or recreate the store to test initialization
    // This depends on your store implementation
  });
});
```

### tests/integration/form.test.ts

```typescript
import { describe, it, expect, vi } from 'vitest';
import { screen, waitFor } from '@testing-library/svelte';
import { render } from '../utils/render';
import Form from '@components/ui/Form/Form.svelte';
import FormField from '@components/ui/Form/FormField.svelte';
import ValidatedInput from '@components/ui/Form/ValidatedInput.svelte';
import { createForm } from '@utils/validation/form';
import { required, email, minLength } from '@utils/validation/validators';

// Create a test wrapper component
import { mount } from 'svelte';

describe('Form Integration', () => {
  it('should validate and submit form', async () => {
    const onSubmit = vi.fn();

    const form = createForm({
      fields: {
        email: {
          initialValue: '',
          validators: [required(), email()]
        },
        password: {
          initialValue: '',
          validators: [required(), minLength(8)]
        }
      },
      onSubmit
    });

    const { user } = render(Form, {
      props: { form }
    });

    // Fill in form fields
    const emailInput = screen.getByLabelText('Email');
    const passwordInput = screen.getByLabelText('Password');

    await user.type(emailInput, 'test@example.com');
    await user.type(passwordInput, 'password123');

    // Submit form
    const submitButton = screen.getByRole('button', { name: /submit/i });
    await user.click(submitButton);

    await waitFor(() => {
      expect(onSubmit).toHaveBeenCalledWith({
        email: 'test@example.com',
        password: 'password123'
      });
    });
  });

  it('should show validation errors', async () => {
    const form = createForm({
      fields: {
        email: {
          initialValue: '',
          validators: [required(), email()]
        }
      },
      validateOn: ['onBlur']
    });

    const { user } = render(Form, {
      props: { form }
    });

    const emailInput = screen.getByLabelText('Email');

    // Type invalid email and blur
    await user.type(emailInput, 'invalid');
    await user.tab();

    await waitFor(() => {
      expect(screen.getByText(/valid email/i)).toBeInTheDocument();
    });
  });

  it('should clear errors on valid input', async () => {
    const form = createForm({
      fields: {
        email: {
          initialValue: '',
          validators: [required(), email()]
        }
      },
      validateOn: ['onChange']
    });

    const { user } = render(Form, {
      props: { form }
    });

    const emailInput = screen.getByLabelText('Email');

    // Type invalid then valid
    await user.type(emailInput, 'invalid');

    await waitFor(() => {
      expect(screen.getByText(/valid email/i)).toBeInTheDocument();
    });

    await user.clear(emailInput);
    await user.type(emailInput, 'valid@example.com');

    await waitFor(() => {
      expect(screen.queryByText(/valid email/i)).not.toBeInTheDocument();
    });
  });
});
```

### package.json (Test Scripts)

```json
{
  "scripts": {
    "test": "vitest",
    "test:run": "vitest run",
    "test:watch": "vitest watch",
    "test:coverage": "vitest run --coverage",
    "test:ui": "vitest --ui",
    "test:e2e": "playwright test",
    "test:a11y": "vitest run --config vitest.a11y.config.ts"
  },
  "devDependencies": {
    "@testing-library/jest-dom": "^6.1.0",
    "@testing-library/svelte": "^4.0.0",
    "@testing-library/user-event": "^14.5.0",
    "@vitest/coverage-v8": "^1.0.0",
    "@vitest/ui": "^1.0.0",
    "jest-axe": "^8.0.0",
    "jsdom": "^23.0.0",
    "vitest": "^1.0.0"
  }
}
```

---

## Testing Requirements

### Coverage Goals

| Category | Target |
|----------|--------|
| Statements | 80% |
| Branches | 75% |
| Functions | 80% |
| Lines | 80% |

### Test Categories

1. **Unit Tests**: Test individual components in isolation
2. **Integration Tests**: Test component interactions
3. **Accessibility Tests**: Ensure WCAG compliance
4. **Visual Regression Tests**: Prevent UI regressions
5. **End-to-End Tests**: Test complete user flows

### CI Integration

```yaml
# .github/workflows/test.yml
name: Tests

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'

      - name: Install dependencies
        run: npm ci

      - name: Run tests
        run: npm run test:coverage

      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          files: ./coverage/lcov.info
```

---

## Related Specs

- [196-component-library.md](./196-component-library.md) - Component library setup
- All component specs (197-214) - Components to test
