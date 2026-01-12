# 285 - Settings Component Tests

**Phase:** 13 - Settings UI
**Spec ID:** 285
**Status:** Planned
**Dependencies:** 271-284 (All Settings specs)
**Estimated Context:** ~10% of model context window

---

## Objective

Create comprehensive test suites for all Settings UI components including unit tests, integration tests, accessibility tests, and visual regression tests to ensure reliability and maintainability.

---

## Acceptance Criteria

- [ ] Unit tests for all settings components
- [ ] Unit tests for settings store
- [ ] Unit tests for validation utilities
- [ ] Integration tests for settings flow
- [ ] Accessibility tests (ARIA, keyboard navigation)
- [ ] Snapshot tests for UI consistency
- [ ] Mock utilities for IPC calls
- [ ] Test utilities for common patterns

---

## Implementation Details

### 1. Test Setup (src/lib/test/setup.ts)

```typescript
import '@testing-library/jest-dom';
import { vi } from 'vitest';

// Mock IPC
vi.mock('$lib/ipc', () => ({
  invoke: vi.fn().mockImplementation((command: string, args?: any) => {
    console.log(`IPC call: ${command}`, args);
    return Promise.resolve(null);
  }),
}));

// Mock browser APIs
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: vi.fn().mockImplementation((query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
});

// Mock localStorage
const localStorageMock = {
  getItem: vi.fn(),
  setItem: vi.fn(),
  removeItem: vi.fn(),
  clear: vi.fn(),
};
Object.defineProperty(window, 'localStorage', { value: localStorageMock });

// Mock clipboard
Object.defineProperty(navigator, 'clipboard', {
  value: {
    writeText: vi.fn().mockResolvedValue(undefined),
    readText: vi.fn().mockResolvedValue(''),
  },
});

// Mock ResizeObserver
class ResizeObserverMock {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
}
window.ResizeObserver = ResizeObserverMock as any;

// Mock IntersectionObserver
class IntersectionObserverMock {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
}
window.IntersectionObserver = IntersectionObserverMock as any;
```

### 2. Test Utilities (src/lib/test/utils.ts)

```typescript
import { render, type RenderResult } from '@testing-library/svelte';
import { get, type Writable } from 'svelte/store';
import type { AllSettings } from '$lib/types/settings';
import { DEFAULT_SETTINGS } from '$lib/stores/settings-defaults';

/**
 * Get current value from a Svelte store
 */
export function getStoreValue<T>(store: { subscribe: (fn: (value: T) => void) => void }): T {
  let value: T;
  store.subscribe(v => value = v)();
  return value!;
}

/**
 * Wait for store to reach expected state
 */
export async function waitForStore<T>(
  store: { subscribe: (fn: (value: T) => void) => void },
  predicate: (value: T) => boolean,
  timeout: number = 1000
): Promise<T> {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      reject(new Error('Timeout waiting for store state'));
    }, timeout);

    const unsubscribe = store.subscribe(value => {
      if (predicate(value)) {
        clearTimeout(timer);
        unsubscribe();
        resolve(value);
      }
    });
  });
}

/**
 * Create mock settings with overrides
 */
export function createMockSettings(overrides: Partial<AllSettings> = {}): AllSettings {
  return {
    ...DEFAULT_SETTINGS,
    ...overrides,
    general: { ...DEFAULT_SETTINGS.general, ...overrides.general },
    appearance: { ...DEFAULT_SETTINGS.appearance, ...overrides.appearance },
    editor: { ...DEFAULT_SETTINGS.editor, ...overrides.editor },
    backends: { ...DEFAULT_SETTINGS.backends, ...overrides.backends },
    keybindings: { ...DEFAULT_SETTINGS.keybindings, ...overrides.keybindings },
    git: { ...DEFAULT_SETTINGS.git, ...overrides.git },
    sync: { ...DEFAULT_SETTINGS.sync, ...overrides.sync },
  };
}

/**
 * Create mock IPC responses
 */
export function createMockIPC(responses: Record<string, any> = {}) {
  return vi.fn().mockImplementation((command: string, args?: any) => {
    if (responses[command]) {
      const response = responses[command];
      return Promise.resolve(typeof response === 'function' ? response(args) : response);
    }
    return Promise.resolve(null);
  });
}

/**
 * Render component with providers
 */
export function renderWithProviders(
  component: any,
  props: Record<string, any> = {}
): RenderResult<any> {
  return render(component, { props });
}

/**
 * Simulate keyboard event
 */
export function createKeyboardEvent(
  key: string,
  options: Partial<KeyboardEventInit> = {}
): KeyboardEvent {
  return new KeyboardEvent('keydown', {
    key,
    bubbles: true,
    cancelable: true,
    ...options,
  });
}

/**
 * Simulate drag and drop
 */
export function createDragEvent(
  type: string,
  files: File[] = []
): DragEvent {
  const dataTransfer = {
    files,
    items: files.map(f => ({ kind: 'file', getAsFile: () => f })),
    types: ['Files'],
    setData: vi.fn(),
    getData: vi.fn(),
  };

  return new DragEvent(type, {
    bubbles: true,
    cancelable: true,
    dataTransfer: dataTransfer as any,
  });
}

/**
 * Create mock file
 */
export function createMockFile(
  content: string,
  name: string,
  type: string = 'application/json'
): File {
  const blob = new Blob([content], { type });
  return new File([blob], name, { type });
}
```

### 3. Store Tests (src/lib/stores/__tests__/settings-store.test.ts)

```typescript
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';
import {
  settingsStore,
  generalSettings,
  appearanceSettings,
  editorSettings,
  backendSettings,
  settingsErrors,
  settingsIsDirty,
} from '../settings-store';
import { DEFAULT_SETTINGS } from '../settings-defaults';
import { getStoreValue, createMockSettings } from '$lib/test/utils';

describe('settingsStore', () => {
  beforeEach(() => {
    settingsStore.resetAll();
  });

  describe('initialization', () => {
    it('starts with default settings', () => {
      const state = getStoreValue(settingsStore);
      expect(state.settings).toEqual(DEFAULT_SETTINGS);
    });

    it('starts not dirty', () => {
      expect(getStoreValue(settingsIsDirty)).toBe(false);
    });

    it('starts with no errors', () => {
      const errors = getStoreValue(settingsErrors);
      const actualErrors = errors.filter(e => e.severity === 'error');
      expect(actualErrors).toHaveLength(0);
    });
  });

  describe('updateSetting', () => {
    it('updates single setting', () => {
      settingsStore.updateSetting('general', 'language', 'es');

      const general = getStoreValue(generalSettings);
      expect(general.language).toBe('es');
    });

    it('marks store as dirty', () => {
      settingsStore.updateSetting('general', 'language', 'fr');

      expect(getStoreValue(settingsIsDirty)).toBe(true);
    });

    it('validates on update', () => {
      settingsStore.updateSetting('appearance', 'fontSize', 100);

      const errors = getStoreValue(settingsErrors);
      expect(errors.some(e => e.path === 'appearance.fontSize')).toBe(true);
    });
  });

  describe('updateCategory', () => {
    it('updates multiple settings in category', () => {
      settingsStore.updateCategory('appearance', {
        theme: 'dark',
        fontSize: 16,
      });

      const appearance = getStoreValue(appearanceSettings);
      expect(appearance.theme).toBe('dark');
      expect(appearance.fontSize).toBe(16);
    });

    it('preserves other settings in category', () => {
      const originalAccentColor = DEFAULT_SETTINGS.appearance.accentColor;

      settingsStore.updateCategory('appearance', {
        theme: 'dark',
      });

      const appearance = getStoreValue(appearanceSettings);
      expect(appearance.accentColor).toBe(originalAccentColor);
    });
  });

  describe('resetCategory', () => {
    it('resets category to defaults', () => {
      settingsStore.updateSetting('editor', 'tabSize', 8);
      settingsStore.resetCategory('editor');

      const editor = getStoreValue(editorSettings);
      expect(editor.tabSize).toBe(DEFAULT_SETTINGS.editor.tabSize);
    });

    it('only resets specified category', () => {
      settingsStore.updateSetting('general', 'language', 'de');
      settingsStore.updateSetting('editor', 'tabSize', 8);
      settingsStore.resetCategory('editor');

      const general = getStoreValue(generalSettings);
      expect(general.language).toBe('de');
    });
  });

  describe('resetAll', () => {
    it('resets all settings to defaults', () => {
      settingsStore.updateSetting('general', 'language', 'jp');
      settingsStore.updateSetting('appearance', 'theme', 'dark');
      settingsStore.updateSetting('editor', 'tabSize', 4);
      settingsStore.resetAll();

      const state = getStoreValue(settingsStore);
      expect(state.settings).toEqual(DEFAULT_SETTINGS);
    });
  });

  describe('discardChanges', () => {
    it('reverts to last saved state', async () => {
      await settingsStore.init();

      settingsStore.updateSetting('general', 'language', 'ru');
      settingsStore.discardChanges();

      const state = getStoreValue(settingsStore);
      expect(state.isDirty).toBe(false);
      expect(state.settings.general.language).toBe('en');
    });
  });

  describe('export/import', () => {
    it('exports settings as JSON', () => {
      const exported = settingsStore.exportSettings();
      const parsed = JSON.parse(exported);

      expect(parsed.settings).toEqual(DEFAULT_SETTINGS);
      expect(parsed.version).toBeDefined();
      expect(parsed.exportedAt).toBeDefined();
    });

    it('imports settings from JSON', async () => {
      const importData = JSON.stringify({
        settings: createMockSettings({ general: { ...DEFAULT_SETTINGS.general, language: 'ko' } }),
        version: 1,
      });

      await settingsStore.importSettings(importData);

      const general = getStoreValue(generalSettings);
      expect(general.language).toBe('ko');
    });

    it('throws on invalid import', async () => {
      await expect(
        settingsStore.importSettings('invalid json')
      ).rejects.toThrow();
    });
  });
});

describe('derived stores', () => {
  beforeEach(() => {
    settingsStore.resetAll();
  });

  it('generalSettings reflects current state', () => {
    settingsStore.updateSetting('general', 'language', 'pt');

    const general = getStoreValue(generalSettings);
    expect(general.language).toBe('pt');
  });

  it('appearanceSettings reflects current state', () => {
    settingsStore.updateSetting('appearance', 'theme', 'dark');

    const appearance = getStoreValue(appearanceSettings);
    expect(appearance.theme).toBe('dark');
  });

  it('editorSettings reflects current state', () => {
    settingsStore.updateSetting('editor', 'tabSize', 4);

    const editor = getStoreValue(editorSettings);
    expect(editor.tabSize).toBe(4);
  });

  it('backendSettings reflects current state', () => {
    settingsStore.updateSetting('backends', 'timeout', 90000);

    const backends = getStoreValue(backendSettings);
    expect(backends.timeout).toBe(90000);
  });
});
```

### 4. Component Tests (src/lib/components/settings/__tests__/integration.test.ts)

```typescript
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import SettingsLayout from '../SettingsLayout.svelte';
import GeneralSettings from '../GeneralSettings.svelte';
import AppearanceSettings from '../AppearanceSettings.svelte';
import { settingsStore } from '$lib/stores/settings-store';
import { getStoreValue } from '$lib/test/utils';

describe('Settings Integration', () => {
  beforeEach(() => {
    settingsStore.resetAll();
  });

  describe('GeneralSettings', () => {
    it('renders and interacts correctly', async () => {
      const user = userEvent.setup();
      render(GeneralSettings);

      // Find and change language
      const select = screen.getByRole('combobox');
      await user.selectOptions(select, 'es');

      const state = getStoreValue(settingsStore);
      expect(state.settings.general.language).toBe('es');
    });

    it('toggles work correctly', async () => {
      const user = userEvent.setup();
      render(GeneralSettings);

      const toggle = screen.getByRole('switch', { name: /automatic updates/i });
      await user.click(toggle);

      const state = getStoreValue(settingsStore);
      expect(state.settings.general.autoUpdate).toBe(false);
    });
  });

  describe('AppearanceSettings', () => {
    it('changes theme correctly', async () => {
      const user = userEvent.setup();
      render(AppearanceSettings);

      const darkButton = screen.getByRole('button', { name: /dark/i });
      await user.click(darkButton);

      const state = getStoreValue(settingsStore);
      expect(state.settings.appearance.theme).toBe('dark');
    });

    it('changes accent color correctly', async () => {
      const user = userEvent.setup();
      render(AppearanceSettings);

      const purpleButton = screen.getByRole('button', { name: /purple/i });
      await user.click(purpleButton);

      const state = getStoreValue(settingsStore);
      expect(state.settings.appearance.accentColor).toBe('#9c27b0');
    });
  });

  describe('Settings Flow', () => {
    it('persists changes across components', async () => {
      const user = userEvent.setup();

      // Change in general settings
      const { unmount: unmount1 } = render(GeneralSettings);
      const select = screen.getByRole('combobox');
      await user.selectOptions(select, 'de');
      unmount1();

      // Verify in store
      let state = getStoreValue(settingsStore);
      expect(state.settings.general.language).toBe('de');
      expect(state.isDirty).toBe(true);

      // Change in appearance settings
      const { unmount: unmount2 } = render(AppearanceSettings);
      const darkButton = screen.getByRole('button', { name: /dark/i });
      await user.click(darkButton);
      unmount2();

      // Verify both changes persisted
      state = getStoreValue(settingsStore);
      expect(state.settings.general.language).toBe('de');
      expect(state.settings.appearance.theme).toBe('dark');
    });

    it('shows validation errors', async () => {
      const user = userEvent.setup();
      render(AppearanceSettings);

      // Force an invalid value through store
      settingsStore.updateSetting('appearance', 'fontSize', 100);

      await waitFor(() => {
        const state = getStoreValue(settingsStore);
        expect(state.errors.some(e => e.path === 'appearance.fontSize')).toBe(true);
      });
    });
  });
});
```

### 5. Accessibility Tests (src/lib/components/settings/__tests__/accessibility.test.ts)

```typescript
import { describe, it, expect, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import { axe, toHaveNoViolations } from 'jest-axe';
import SettingsLayout from '../SettingsLayout.svelte';
import GeneralSettings from '../GeneralSettings.svelte';
import AppearanceSettings from '../AppearanceSettings.svelte';
import KeybindingsSettings from '../KeybindingsSettings.svelte';
import { settingsStore } from '$lib/stores/settings-store';

expect.extend(toHaveNoViolations);

describe('Settings Accessibility', () => {
  beforeEach(() => {
    settingsStore.resetAll();
  });

  describe('SettingsLayout', () => {
    it('has no accessibility violations', async () => {
      const { container } = render(SettingsLayout);
      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('has proper ARIA landmarks', () => {
      render(SettingsLayout);

      expect(screen.getByRole('main', { name: /settings/i })).toBeInTheDocument();
      expect(screen.getByRole('navigation', { name: /settings navigation/i })).toBeInTheDocument();
    });

    it('supports keyboard navigation', async () => {
      render(SettingsLayout);

      const searchInput = screen.getByPlaceholderText(/search settings/i);
      searchInput.focus();
      expect(document.activeElement).toBe(searchInput);
    });
  });

  describe('GeneralSettings', () => {
    it('has no accessibility violations', async () => {
      const { container } = render(GeneralSettings);
      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('has proper labels', () => {
      render(GeneralSettings);

      expect(screen.getByLabelText(/display language/i)).toBeInTheDocument();
      expect(screen.getByRole('switch', { name: /start minimized/i })).toBeInTheDocument();
    });
  });

  describe('AppearanceSettings', () => {
    it('has no accessibility violations', async () => {
      const { container } = render(AppearanceSettings);
      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('theme buttons have proper aria-pressed', () => {
      render(AppearanceSettings);

      const systemButton = screen.getByRole('button', { name: /system/i });
      expect(systemButton).toHaveAttribute('aria-pressed', 'true');

      const darkButton = screen.getByRole('button', { name: /dark/i });
      expect(darkButton).toHaveAttribute('aria-pressed', 'false');
    });
  });

  describe('KeybindingsSettings', () => {
    it('has no accessibility violations', async () => {
      const { container } = render(KeybindingsSettings);
      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('has proper role attributes', () => {
      render(KeybindingsSettings);

      // Categories should be expandable
      const categoryHeaders = screen.getAllByRole('button', { expanded: true });
      expect(categoryHeaders.length).toBeGreaterThan(0);
    });
  });
});
```

### 6. Validation Tests (src/lib/utils/__tests__/validators.test.ts)

```typescript
import { describe, it, expect } from 'vitest';
import {
  required,
  minValue,
  maxValue,
  range,
  pattern,
  enumValue,
  url,
  email,
  hexColor,
  compose,
  conditional,
  createResult,
} from '../validators';
import type { ValidationContext } from '$lib/types/validation';
import { DEFAULT_SETTINGS } from '$lib/stores/settings-defaults';

const createContext = (path: string, field: string): ValidationContext => ({
  settings: DEFAULT_SETTINGS,
  path,
  field,
});

describe('validators', () => {
  describe('required', () => {
    it('fails for null', () => {
      const result = required(createContext('test', 'field'), null);
      expect(result.valid).toBe(false);
    });

    it('fails for empty string', () => {
      const result = required(createContext('test', 'field'), '');
      expect(result.valid).toBe(false);
    });

    it('passes for valid value', () => {
      const result = required(createContext('test', 'field'), 'value');
      expect(result.valid).toBe(true);
    });
  });

  describe('minValue', () => {
    const validator = minValue(5);

    it('fails below minimum', () => {
      const result = validator.validate(3, createContext('test', 'field'));
      expect(result.valid).toBe(false);
    });

    it('passes at minimum', () => {
      const result = validator.validate(5, createContext('test', 'field'));
      expect(result.valid).toBe(true);
    });

    it('passes above minimum', () => {
      const result = validator.validate(10, createContext('test', 'field'));
      expect(result.valid).toBe(true);
    });
  });

  describe('maxValue', () => {
    const validator = maxValue(10);

    it('fails above maximum', () => {
      const result = validator.validate(15, createContext('test', 'field'));
      expect(result.valid).toBe(false);
    });

    it('passes at maximum', () => {
      const result = validator.validate(10, createContext('test', 'field'));
      expect(result.valid).toBe(true);
    });
  });

  describe('range', () => {
    const validator = range(5, 10);

    it('fails below range', () => {
      const result = validator.validate(3, createContext('test', 'field'));
      expect(result.valid).toBe(false);
    });

    it('fails above range', () => {
      const result = validator.validate(15, createContext('test', 'field'));
      expect(result.valid).toBe(false);
    });

    it('passes within range', () => {
      const result = validator.validate(7, createContext('test', 'field'));
      expect(result.valid).toBe(true);
    });
  });

  describe('pattern', () => {
    const validator = pattern(/^[A-Z]+$/, 'Must be uppercase');

    it('fails for non-matching', () => {
      const result = validator.validate('abc', createContext('test', 'field'));
      expect(result.valid).toBe(false);
    });

    it('passes for matching', () => {
      const result = validator.validate('ABC', createContext('test', 'field'));
      expect(result.valid).toBe(true);
    });
  });

  describe('enumValue', () => {
    const validator = enumValue(['a', 'b', 'c'] as const);

    it('fails for non-member', () => {
      const result = validator.validate('d' as any, createContext('test', 'field'));
      expect(result.valid).toBe(false);
    });

    it('passes for member', () => {
      const result = validator.validate('a', createContext('test', 'field'));
      expect(result.valid).toBe(true);
    });
  });

  describe('url', () => {
    const validator = url();

    it('fails for invalid URL', () => {
      const result = validator.validate('not-a-url', createContext('test', 'field'));
      expect(result.valid).toBe(false);
    });

    it('passes for valid URL', () => {
      const result = validator.validate('https://example.com', createContext('test', 'field'));
      expect(result.valid).toBe(true);
    });
  });

  describe('email', () => {
    const validator = email();

    it('fails for invalid email', () => {
      const result = validator.validate('not-an-email', createContext('test', 'field'));
      expect(result.valid).toBe(false);
    });

    it('passes for valid email', () => {
      const result = validator.validate('user@example.com', createContext('test', 'field'));
      expect(result.valid).toBe(true);
    });
  });

  describe('hexColor', () => {
    const validator = hexColor();

    it('fails for invalid hex', () => {
      const result = validator.validate('red', createContext('test', 'field'));
      expect(result.valid).toBe(false);
    });

    it('passes for valid hex', () => {
      const result = validator.validate('#FF5500', createContext('test', 'field'));
      expect(result.valid).toBe(true);
    });
  });

  describe('compose', () => {
    const validator = compose(minValue(0), maxValue(100));

    it('fails if any validator fails', () => {
      const result = validator.validate(150, createContext('test', 'field'));
      expect(result.valid).toBe(false);
    });

    it('passes if all validators pass', () => {
      const result = validator.validate(50, createContext('test', 'field'));
      expect(result.valid).toBe(true);
    });
  });

  describe('conditional', () => {
    const validator = conditional(
      (s) => s.general.autoUpdate,
      minValue(1000)
    );

    it('skips validation when condition false', () => {
      const context = {
        ...createContext('test', 'field'),
        settings: { ...DEFAULT_SETTINGS, general: { ...DEFAULT_SETTINGS.general, autoUpdate: false } },
      };
      const result = validator.validate(100, context);
      expect(result.valid).toBe(true);
    });

    it('validates when condition true', () => {
      const context = {
        ...createContext('test', 'field'),
        settings: { ...DEFAULT_SETTINGS, general: { ...DEFAULT_SETTINGS.general, autoUpdate: true } },
      };
      const result = validator.validate(100, context);
      expect(result.valid).toBe(false);
    });
  });
});
```

---

## Testing Requirements

1. All unit tests pass
2. All integration tests pass
3. All accessibility tests pass
4. Test coverage >= 80%
5. No console errors during tests
6. Tests are maintainable and readable
7. Mock utilities work correctly
8. Tests run in reasonable time (<30s)

### Test Configuration (vitest.config.ts)

```typescript
import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import path from 'path';

export default defineConfig({
  plugins: [svelte({ hot: !process.env.VITEST })],
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: ['./src/lib/test/setup.ts'],
    include: ['src/**/*.{test,spec}.{js,ts}'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      include: ['src/lib/**/*.{ts,svelte}'],
      exclude: ['src/lib/test/**', '**/*.d.ts'],
      thresholds: {
        global: {
          branches: 80,
          functions: 80,
          lines: 80,
          statements: 80,
        },
      },
    },
  },
  resolve: {
    alias: {
      $lib: path.resolve('./src/lib'),
      '$app/environment': path.resolve('./src/lib/test/mocks/environment.ts'),
      '$app/navigation': path.resolve('./src/lib/test/mocks/navigation.ts'),
      '$app/stores': path.resolve('./src/lib/test/mocks/stores.ts'),
    },
  },
});
```

---

## Related Specs

- Tests for: All Settings specs (271-284)
- Depends on: All Settings component implementations
- Related: [008-test-infrastructure.md](../phase-00-setup/008-test-infrastructure.md)
