# Spec 214: Keyboard Shortcuts and Hotkeys

## Phase
Phase 9: UI Foundation

## Spec ID
214

## Status
Planned

## Dependencies
- Spec 196: Component Library Setup
- Spec 190: IPC Store Bindings (for system integration)

## Estimated Context
~10%

---

## Objective

Implement a comprehensive keyboard shortcuts system for Tachikoma with global and local hotkey support, customizable keybindings, shortcut discovery, conflict detection, and accessibility considerations.

---

## Acceptance Criteria

- [x] Global keyboard shortcut registration
- [x] Component-scoped shortcuts
- [x] Modifier key support (Cmd/Ctrl, Alt, Shift)
- [x] Shortcut conflict detection
- [x] Customizable keybindings
- [x] Shortcut help dialog
- [x] Platform-aware key display (Cmd vs Ctrl)
- [x] Disabled state support
- [x] Focus-aware shortcuts
- [x] Svelte action for easy binding

---

## Implementation Details

### src/lib/utils/keyboard/types.ts

```typescript
export type ModifierKey = 'ctrl' | 'alt' | 'shift' | 'meta';

export interface KeyCombo {
  key: string;
  modifiers: ModifierKey[];
}

export interface Shortcut {
  id: string;
  keys: KeyCombo | KeyCombo[]; // Support multiple key combos
  description: string;
  category?: string;
  handler: (event: KeyboardEvent) => void | boolean;
  enabled?: boolean;
  global?: boolean; // Works regardless of focus
  preventDefault?: boolean;
  allowInInput?: boolean; // Allow in text inputs
}

export interface ShortcutGroup {
  id: string;
  name: string;
  shortcuts: Shortcut[];
}

export interface KeyboardConfig {
  platformOverrides?: boolean; // Convert Ctrl to Cmd on Mac
  globalEnabled?: boolean;
}
```

### src/lib/utils/keyboard/parser.ts

```typescript
import type { KeyCombo, ModifierKey } from './types';

const MODIFIER_MAP: Record<string, ModifierKey> = {
  'ctrl': 'ctrl',
  'control': 'ctrl',
  'alt': 'alt',
  'option': 'alt',
  'shift': 'shift',
  'meta': 'meta',
  'cmd': 'meta',
  'command': 'meta',
  'win': 'meta',
  'super': 'meta'
};

const KEY_ALIASES: Record<string, string> = {
  'esc': 'Escape',
  'escape': 'Escape',
  'enter': 'Enter',
  'return': 'Enter',
  'space': ' ',
  'spacebar': ' ',
  'up': 'ArrowUp',
  'down': 'ArrowDown',
  'left': 'ArrowLeft',
  'right': 'ArrowRight',
  'backspace': 'Backspace',
  'delete': 'Delete',
  'del': 'Delete',
  'tab': 'Tab',
  'home': 'Home',
  'end': 'End',
  'pageup': 'PageUp',
  'pagedown': 'PageDown',
  'plus': '+',
  'minus': '-',
  'equals': '='
};

/**
 * Parse a shortcut string like "Ctrl+Shift+K" into KeyCombo
 */
export function parseShortcut(shortcut: string): KeyCombo {
  const parts = shortcut.toLowerCase().split('+').map(p => p.trim());
  const modifiers: ModifierKey[] = [];
  let key = '';

  for (const part of parts) {
    if (MODIFIER_MAP[part]) {
      modifiers.push(MODIFIER_MAP[part]);
    } else {
      key = KEY_ALIASES[part] || part;
    }
  }

  // Normalize single character keys to uppercase
  if (key.length === 1) {
    key = key.toUpperCase();
  }

  return { key, modifiers };
}

/**
 * Parse multiple shortcuts separated by comma
 */
export function parseShortcuts(shortcuts: string): KeyCombo[] {
  return shortcuts.split(',').map(s => parseShortcut(s.trim()));
}

/**
 * Check if a keyboard event matches a KeyCombo
 */
export function matchesKeyCombo(event: KeyboardEvent, combo: KeyCombo): boolean {
  // Check key
  const eventKey = event.key.length === 1 ? event.key.toUpperCase() : event.key;
  if (eventKey !== combo.key && event.code !== combo.key) {
    return false;
  }

  // Check modifiers
  const hasCtrl = combo.modifiers.includes('ctrl');
  const hasAlt = combo.modifiers.includes('alt');
  const hasShift = combo.modifiers.includes('shift');
  const hasMeta = combo.modifiers.includes('meta');

  return (
    event.ctrlKey === hasCtrl &&
    event.altKey === hasAlt &&
    event.shiftKey === hasShift &&
    event.metaKey === hasMeta
  );
}

/**
 * Format KeyCombo for display
 */
export function formatKeyCombo(combo: KeyCombo, platform: 'mac' | 'windows' | 'linux' = 'mac'): string {
  const symbols: Record<string, Record<ModifierKey, string>> = {
    mac: {
      ctrl: '⌃',
      alt: '⌥',
      shift: '⇧',
      meta: '⌘'
    },
    windows: {
      ctrl: 'Ctrl',
      alt: 'Alt',
      shift: 'Shift',
      meta: 'Win'
    },
    linux: {
      ctrl: 'Ctrl',
      alt: 'Alt',
      shift: 'Shift',
      meta: 'Super'
    }
  };

  const keySymbols: Record<string, string> = {
    'ArrowUp': '↑',
    'ArrowDown': '↓',
    'ArrowLeft': '←',
    'ArrowRight': '→',
    'Enter': '↵',
    'Backspace': '⌫',
    'Delete': '⌦',
    'Escape': 'Esc',
    'Tab': '⇥',
    ' ': 'Space'
  };

  const parts: string[] = [];
  const platformSymbols = symbols[platform];

  // Add modifiers in consistent order
  if (combo.modifiers.includes('ctrl')) parts.push(platformSymbols.ctrl);
  if (combo.modifiers.includes('alt')) parts.push(platformSymbols.alt);
  if (combo.modifiers.includes('shift')) parts.push(platformSymbols.shift);
  if (combo.modifiers.includes('meta')) parts.push(platformSymbols.meta);

  // Add key
  const displayKey = keySymbols[combo.key] || combo.key.toUpperCase();
  parts.push(displayKey);

  return platform === 'mac' ? parts.join('') : parts.join('+');
}
```

### src/lib/stores/keyboard.ts

```typescript
import { writable, derived, get } from 'svelte/store';
import type { Shortcut, ShortcutGroup, KeyCombo, KeyboardConfig } from '@utils/keyboard/types';
import { parseShortcut, parseShortcuts, matchesKeyCombo } from '@utils/keyboard/parser';

interface KeyboardState {
  shortcuts: Map<string, Shortcut>;
  groups: Map<string, ShortcutGroup>;
  enabled: boolean;
  activeScope: string | null;
}

function createKeyboardStore(config: KeyboardConfig = {}) {
  const { subscribe, update, set } = writable<KeyboardState>({
    shortcuts: new Map(),
    groups: new Map(),
    enabled: config.globalEnabled ?? true,
    activeScope: null
  });

  // Track if we've set up the global listener
  let listenerAttached = false;

  function handleKeyDown(event: KeyboardEvent) {
    const state = get({ subscribe });
    if (!state.enabled) return;

    // Check if we're in an input element
    const isInput = ['INPUT', 'TEXTAREA', 'SELECT'].includes(
      (event.target as HTMLElement).tagName
    ) || (event.target as HTMLElement).isContentEditable;

    for (const shortcut of state.shortcuts.values()) {
      if (shortcut.enabled === false) continue;
      if (isInput && !shortcut.allowInInput) continue;

      const keyCombos = Array.isArray(shortcut.keys)
        ? shortcut.keys
        : [shortcut.keys];

      for (const combo of keyCombos) {
        if (matchesKeyCombo(event, combo)) {
          if (shortcut.preventDefault !== false) {
            event.preventDefault();
          }

          const result = shortcut.handler(event);

          // If handler returns false, stop propagation
          if (result === false) {
            event.stopPropagation();
          }

          return;
        }
      }
    }
  }

  function attachListener() {
    if (listenerAttached || typeof window === 'undefined') return;
    window.addEventListener('keydown', handleKeyDown, true);
    listenerAttached = true;
  }

  function detachListener() {
    if (!listenerAttached || typeof window === 'undefined') return;
    window.removeEventListener('keydown', handleKeyDown, true);
    listenerAttached = false;
  }

  return {
    subscribe,

    /**
     * Register a keyboard shortcut
     */
    register(
      id: string,
      keys: string | KeyCombo | KeyCombo[],
      handler: Shortcut['handler'],
      options: Partial<Omit<Shortcut, 'id' | 'keys' | 'handler'>> = {}
    ) {
      const parsedKeys = typeof keys === 'string'
        ? parseShortcuts(keys)
        : Array.isArray(keys) ? keys : [keys];

      const shortcut: Shortcut = {
        id,
        keys: parsedKeys.length === 1 ? parsedKeys[0] : parsedKeys,
        handler,
        description: options.description || '',
        category: options.category,
        enabled: options.enabled ?? true,
        global: options.global ?? false,
        preventDefault: options.preventDefault ?? true,
        allowInInput: options.allowInInput ?? false
      };

      update(state => {
        const shortcuts = new Map(state.shortcuts);
        shortcuts.set(id, shortcut);
        return { ...state, shortcuts };
      });

      attachListener();

      // Return unregister function
      return () => this.unregister(id);
    },

    /**
     * Unregister a shortcut
     */
    unregister(id: string) {
      update(state => {
        const shortcuts = new Map(state.shortcuts);
        shortcuts.delete(id);
        return { ...state, shortcuts };
      });
    },

    /**
     * Register a group of shortcuts
     */
    registerGroup(group: ShortcutGroup) {
      update(state => {
        const groups = new Map(state.groups);
        const shortcuts = new Map(state.shortcuts);

        groups.set(group.id, group);

        for (const shortcut of group.shortcuts) {
          shortcuts.set(shortcut.id, shortcut);
        }

        return { ...state, groups, shortcuts };
      });

      attachListener();
    },

    /**
     * Unregister a group
     */
    unregisterGroup(groupId: string) {
      update(state => {
        const groups = new Map(state.groups);
        const shortcuts = new Map(state.shortcuts);
        const group = groups.get(groupId);

        if (group) {
          for (const shortcut of group.shortcuts) {
            shortcuts.delete(shortcut.id);
          }
          groups.delete(groupId);
        }

        return { ...state, groups, shortcuts };
      });
    },

    /**
     * Enable/disable a specific shortcut
     */
    setEnabled(id: string, enabled: boolean) {
      update(state => {
        const shortcuts = new Map(state.shortcuts);
        const shortcut = shortcuts.get(id);

        if (shortcut) {
          shortcuts.set(id, { ...shortcut, enabled });
        }

        return { ...state, shortcuts };
      });
    },

    /**
     * Enable/disable all shortcuts
     */
    setGlobalEnabled(enabled: boolean) {
      update(state => ({ ...state, enabled }));
    },

    /**
     * Get all registered shortcuts
     */
    getShortcuts() {
      return derived({ subscribe }, $state =>
        Array.from($state.shortcuts.values())
      );
    },

    /**
     * Get shortcuts by category
     */
    getByCategory(category: string) {
      return derived({ subscribe }, $state =>
        Array.from($state.shortcuts.values()).filter(s => s.category === category)
      );
    },

    /**
     * Check for conflicts
     */
    findConflicts(keys: string | KeyCombo): Shortcut[] {
      const state = get({ subscribe });
      const combo = typeof keys === 'string' ? parseShortcut(keys) : keys;

      return Array.from(state.shortcuts.values()).filter(shortcut => {
        const shortcutCombos = Array.isArray(shortcut.keys)
          ? shortcut.keys
          : [shortcut.keys];

        return shortcutCombos.some(sc =>
          sc.key === combo.key &&
          sc.modifiers.length === combo.modifiers.length &&
          sc.modifiers.every(m => combo.modifiers.includes(m))
        );
      });
    },

    /**
     * Reset all shortcuts
     */
    reset() {
      set({
        shortcuts: new Map(),
        groups: new Map(),
        enabled: true,
        activeScope: null
      });
    },

    /**
     * Cleanup
     */
    destroy() {
      detachListener();
      this.reset();
    }
  };
}

export const keyboardStore = createKeyboardStore();

// Convenience function for registering shortcuts
export function registerShortcut(
  id: string,
  keys: string,
  handler: Shortcut['handler'],
  options?: Partial<Omit<Shortcut, 'id' | 'keys' | 'handler'>>
) {
  return keyboardStore.register(id, keys, handler, options);
}
```

### src/lib/actions/hotkey.ts

```typescript
import type { Action } from 'svelte/action';
import { parseShortcut, matchesKeyCombo } from '@utils/keyboard/parser';
import type { KeyCombo } from '@utils/keyboard/types';

interface HotkeyOptions {
  keys: string | KeyCombo;
  handler: (event: KeyboardEvent) => void;
  enabled?: boolean;
  preventDefault?: boolean;
  stopPropagation?: boolean;
}

/**
 * Svelte action for component-scoped keyboard shortcuts
 *
 * Usage:
 * <div use:hotkey={{ keys: 'Ctrl+K', handler: openSearch }}>
 */
export const hotkey: Action<HTMLElement, HotkeyOptions> = (node, options) => {
  let currentOptions = options;

  function handleKeyDown(event: KeyboardEvent) {
    if (currentOptions.enabled === false) return;

    const combo = typeof currentOptions.keys === 'string'
      ? parseShortcut(currentOptions.keys)
      : currentOptions.keys;

    if (matchesKeyCombo(event, combo)) {
      if (currentOptions.preventDefault !== false) {
        event.preventDefault();
      }
      if (currentOptions.stopPropagation) {
        event.stopPropagation();
      }
      currentOptions.handler(event);
    }
  }

  node.addEventListener('keydown', handleKeyDown);

  return {
    update(newOptions: HotkeyOptions) {
      currentOptions = newOptions;
    },
    destroy() {
      node.removeEventListener('keydown', handleKeyDown);
    }
  };
};

/**
 * Multiple hotkeys action
 */
interface MultiHotkeyOptions {
  shortcuts: Array<{
    keys: string;
    handler: (event: KeyboardEvent) => void;
    enabled?: boolean;
  }>;
}

export const hotkeys: Action<HTMLElement, MultiHotkeyOptions> = (node, options) => {
  let currentOptions = options;

  function handleKeyDown(event: KeyboardEvent) {
    for (const shortcut of currentOptions.shortcuts) {
      if (shortcut.enabled === false) continue;

      const combo = parseShortcut(shortcut.keys);
      if (matchesKeyCombo(event, combo)) {
        event.preventDefault();
        shortcut.handler(event);
        return;
      }
    }
  }

  node.addEventListener('keydown', handleKeyDown);

  return {
    update(newOptions: MultiHotkeyOptions) {
      currentOptions = newOptions;
    },
    destroy() {
      node.removeEventListener('keydown', handleKeyDown);
    }
  };
};
```

### src/lib/components/ui/Keyboard/KeyboardShortcut.svelte

```svelte
<script lang="ts">
  import { cn } from '@utils/component';
  import { parseShortcut, formatKeyCombo } from '@utils/keyboard/parser';
  import { browser } from '$app/environment';

  export let keys: string;
  export let size: 'sm' | 'md' = 'md';
  let className: string = '';
  export { className as class };

  $: platform = browser
    ? navigator.platform.toLowerCase().includes('mac') ? 'mac' : 'windows'
    : 'mac';

  $: combo = parseShortcut(keys);
  $: formatted = formatKeyCombo(combo, platform);

  $: classes = cn(
    'keyboard-shortcut',
    `keyboard-shortcut-${size}`,
    className
  );
</script>

<kbd class={classes}>
  {formatted}
</kbd>

<style>
  .keyboard-shortcut {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    font-family: var(--font-mono);
    background-color: var(--color-bg-muted);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-sm);
    color: var(--color-fg-muted);
    white-space: nowrap;
  }

  .keyboard-shortcut-sm {
    font-size: var(--text-xs);
    padding: 1px 4px;
  }

  .keyboard-shortcut-md {
    font-size: var(--text-sm);
    padding: 2px 6px;
  }
</style>
```

### src/lib/components/ui/Keyboard/ShortcutsDialog.svelte

```svelte
<script lang="ts">
  import { cn } from '@utils/component';
  import { keyboardStore } from '@stores/keyboard';
  import Modal from '../Modal/Modal.svelte';
  import KeyboardShortcut from './KeyboardShortcut.svelte';
  import { formatKeyCombo } from '@utils/keyboard/parser';
  import { browser } from '$app/environment';

  export let open: boolean = false;
  let className: string = '';
  export { className as class };

  $: shortcuts = keyboardStore.getShortcuts();

  // Group shortcuts by category
  $: groupedShortcuts = $shortcuts.reduce((acc, shortcut) => {
    const category = shortcut.category || 'General';
    if (!acc[category]) {
      acc[category] = [];
    }
    acc[category].push(shortcut);
    return acc;
  }, {} as Record<string, typeof $shortcuts>);

  $: categories = Object.keys(groupedShortcuts).sort();

  $: platform = browser
    ? navigator.platform.toLowerCase().includes('mac') ? 'mac' : 'windows'
    : 'mac';

  function formatKeys(shortcut: typeof $shortcuts[0]): string {
    const combos = Array.isArray(shortcut.keys) ? shortcut.keys : [shortcut.keys];
    return combos.map(c => formatKeyCombo(c, platform)).join(' or ');
  }

  $: classes = cn('shortcuts-dialog', className);
</script>

<Modal bind:open title="Keyboard Shortcuts" size="md">
  <div class={classes}>
    {#each categories as category}
      <div class="shortcuts-category">
        <h3 class="shortcuts-category-title">{category}</h3>
        <div class="shortcuts-list">
          {#each groupedShortcuts[category] as shortcut}
            <div class="shortcut-item">
              <span class="shortcut-description">{shortcut.description}</span>
              <span class="shortcut-keys">
                {#each (Array.isArray(shortcut.keys) ? shortcut.keys : [shortcut.keys]) as combo, i}
                  {#if i > 0}
                    <span class="shortcut-or">or</span>
                  {/if}
                  <kbd class="shortcut-key">{formatKeyCombo(combo, platform)}</kbd>
                {/each}
              </span>
            </div>
          {/each}
        </div>
      </div>
    {/each}

    {#if categories.length === 0}
      <p class="shortcuts-empty">No keyboard shortcuts registered.</p>
    {/if}
  </div>
</Modal>

<style>
  .shortcuts-dialog {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-6);
    max-height: 60vh;
    overflow-y: auto;
  }

  .shortcuts-category {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-3);
  }

  .shortcuts-category-title {
    font-size: var(--text-sm);
    font-weight: var(--font-semibold);
    color: var(--color-fg-default);
    margin: 0;
    padding-bottom: var(--spacing-2);
    border-bottom: 1px solid var(--color-border-subtle);
  }

  .shortcuts-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-2);
  }

  .shortcut-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--spacing-2) 0;
  }

  .shortcut-description {
    font-size: var(--text-sm);
    color: var(--color-fg-muted);
  }

  .shortcut-keys {
    display: flex;
    align-items: center;
    gap: var(--spacing-2);
  }

  .shortcut-key {
    display: inline-flex;
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    padding: 2px 6px;
    background-color: var(--color-bg-muted);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-sm);
    color: var(--color-fg-default);
  }

  .shortcut-or {
    font-size: var(--text-xs);
    color: var(--color-fg-muted);
  }

  .shortcuts-empty {
    text-align: center;
    color: var(--color-fg-muted);
    padding: var(--spacing-8);
  }
</style>
```

### Usage Examples

```svelte
<script>
  import { onMount, onDestroy } from 'svelte';
  import {
    KeyboardShortcut,
    ShortcutsDialog
  } from '@components/ui';
  import { keyboardStore, registerShortcut } from '@stores/keyboard';
  import { hotkey, hotkeys } from '@actions/hotkey';

  let showShortcuts = false;
  let searchOpen = false;

  // Register global shortcuts
  onMount(() => {
    // Register individual shortcuts
    const unregisterSearch = registerShortcut(
      'global.search',
      'Ctrl+K, Cmd+K',
      () => { searchOpen = true; },
      {
        description: 'Open search',
        category: 'Navigation',
        global: true
      }
    );

    const unregisterHelp = registerShortcut(
      'global.help',
      'Shift+?',
      () => { showShortcuts = true; },
      {
        description: 'Show keyboard shortcuts',
        category: 'Help',
        global: true
      }
    );

    // Register a group of shortcuts
    keyboardStore.registerGroup({
      id: 'editor',
      name: 'Editor',
      shortcuts: [
        {
          id: 'editor.save',
          keys: parseShortcut('Ctrl+S'),
          description: 'Save file',
          category: 'Editor',
          handler: () => saveFile()
        },
        {
          id: 'editor.undo',
          keys: parseShortcut('Ctrl+Z'),
          description: 'Undo',
          category: 'Editor',
          handler: () => undo()
        },
        {
          id: 'editor.redo',
          keys: [parseShortcut('Ctrl+Y'), parseShortcut('Ctrl+Shift+Z')],
          description: 'Redo',
          category: 'Editor',
          handler: () => redo()
        }
      ]
    });

    return () => {
      unregisterSearch();
      unregisterHelp();
      keyboardStore.unregisterGroup('editor');
    };
  });

  // Component functions
  function saveFile() {
    console.log('Saving file...');
  }

  function undo() {
    console.log('Undo');
  }

  function redo() {
    console.log('Redo');
  }
</script>

<!-- Display shortcut hints -->
<button>
  Search
  <KeyboardShortcut keys="Ctrl+K" size="sm" />
</button>

<!-- Component-scoped shortcut using action -->
<div
  class="panel"
  use:hotkey={{
    keys: 'Escape',
    handler: () => closePanel()
  }}
>
  Panel content
</div>

<!-- Multiple shortcuts on one element -->
<div
  class="list"
  use:hotkeys={{
    shortcuts: [
      { keys: 'j', handler: () => selectNext() },
      { keys: 'k', handler: () => selectPrev() },
      { keys: 'Enter', handler: () => openSelected() }
    ]
  }}
>
  List content
</div>

<!-- Shortcuts help dialog -->
<ShortcutsDialog bind:open={showShortcuts} />

<!-- Conditional shortcuts -->
{#if isEditing}
  <div use:hotkey={{
    keys: 'Escape',
    handler: () => cancelEdit(),
    enabled: isEditing
  }}>
    Edit mode content
  </div>
{/if}
```

---

## Testing Requirements

### Unit Tests

```typescript
// tests/keyboard/parser.test.ts
import { describe, it, expect } from 'vitest';
import {
  parseShortcut,
  parseShortcuts,
  matchesKeyCombo,
  formatKeyCombo
} from '@utils/keyboard/parser';

describe('parseShortcut', () => {
  it('should parse simple key', () => {
    const combo = parseShortcut('K');
    expect(combo.key).toBe('K');
    expect(combo.modifiers).toEqual([]);
  });

  it('should parse with modifiers', () => {
    const combo = parseShortcut('Ctrl+Shift+K');
    expect(combo.key).toBe('K');
    expect(combo.modifiers).toContain('ctrl');
    expect(combo.modifiers).toContain('shift');
  });

  it('should handle aliases', () => {
    const combo = parseShortcut('Cmd+Enter');
    expect(combo.key).toBe('Enter');
    expect(combo.modifiers).toContain('meta');
  });

  it('should handle special keys', () => {
    const combo = parseShortcut('Ctrl+Escape');
    expect(combo.key).toBe('Escape');
  });
});

describe('parseShortcuts', () => {
  it('should parse multiple shortcuts', () => {
    const combos = parseShortcuts('Ctrl+K, Cmd+K');
    expect(combos).toHaveLength(2);
    expect(combos[0].modifiers).toContain('ctrl');
    expect(combos[1].modifiers).toContain('meta');
  });
});

describe('matchesKeyCombo', () => {
  it('should match simple key', () => {
    const combo = parseShortcut('K');
    const event = new KeyboardEvent('keydown', { key: 'k' });

    expect(matchesKeyCombo(event, combo)).toBe(true);
  });

  it('should match with modifiers', () => {
    const combo = parseShortcut('Ctrl+K');
    const matchingEvent = new KeyboardEvent('keydown', {
      key: 'k',
      ctrlKey: true
    });
    const nonMatchingEvent = new KeyboardEvent('keydown', {
      key: 'k',
      ctrlKey: false
    });

    expect(matchesKeyCombo(matchingEvent, combo)).toBe(true);
    expect(matchesKeyCombo(nonMatchingEvent, combo)).toBe(false);
  });

  it('should not match with extra modifiers', () => {
    const combo = parseShortcut('Ctrl+K');
    const event = new KeyboardEvent('keydown', {
      key: 'k',
      ctrlKey: true,
      shiftKey: true
    });

    expect(matchesKeyCombo(event, combo)).toBe(false);
  });
});

describe('formatKeyCombo', () => {
  it('should format for Mac', () => {
    const combo = parseShortcut('Ctrl+Shift+K');
    expect(formatKeyCombo(combo, 'mac')).toBe('⌃⇧K');
  });

  it('should format for Windows', () => {
    const combo = parseShortcut('Ctrl+Shift+K');
    expect(formatKeyCombo(combo, 'windows')).toBe('Ctrl+Shift+K');
  });

  it('should format meta key correctly', () => {
    const combo = parseShortcut('Cmd+K');
    expect(formatKeyCombo(combo, 'mac')).toBe('⌘K');
    expect(formatKeyCombo(combo, 'windows')).toBe('Win+K');
  });
});

// tests/keyboard/store.test.ts
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import { keyboardStore, registerShortcut } from '@stores/keyboard';

describe('keyboardStore', () => {
  beforeEach(() => {
    keyboardStore.reset();
  });

  it('should register shortcuts', () => {
    registerShortcut('test', 'Ctrl+K', vi.fn(), {
      description: 'Test shortcut'
    });

    const shortcuts = get(keyboardStore.getShortcuts());
    expect(shortcuts).toHaveLength(1);
    expect(shortcuts[0].id).toBe('test');
  });

  it('should unregister shortcuts', () => {
    const unregister = registerShortcut('test', 'Ctrl+K', vi.fn());
    unregister();

    const shortcuts = get(keyboardStore.getShortcuts());
    expect(shortcuts).toHaveLength(0);
  });

  it('should find conflicts', () => {
    registerShortcut('test1', 'Ctrl+K', vi.fn());
    registerShortcut('test2', 'Ctrl+J', vi.fn());

    const conflicts = keyboardStore.findConflicts('Ctrl+K');
    expect(conflicts).toHaveLength(1);
    expect(conflicts[0].id).toBe('test1');
  });

  it('should enable/disable shortcuts', () => {
    registerShortcut('test', 'Ctrl+K', vi.fn());
    keyboardStore.setEnabled('test', false);

    const shortcuts = get(keyboardStore.getShortcuts());
    expect(shortcuts[0].enabled).toBe(false);
  });
});
```

---

## Related Specs

- [196-component-library.md](./196-component-library.md) - Component library setup
- [202-modal-component.md](./202-modal-component.md) - Modal component
- [190-ipc-store-bindings.md](./190-ipc-store-bindings.md) - IPC integration
