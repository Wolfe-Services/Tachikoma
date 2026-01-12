# 276 - Keyboard Shortcuts Settings

**Phase:** 13 - Settings UI
**Spec ID:** 276
**Status:** Planned
**Dependencies:** 271-settings-layout, 272-settings-store
**Estimated Context:** ~10% of model context window

---

## Objective

Create the Keyboard Shortcuts Settings panel that allows users to view, search, and customize keyboard shortcuts with support for keybinding presets (default, vim, emacs), conflict detection, and per-command customization.

---

## Acceptance Criteria

- [ ] `KeybindingsSettings.svelte` component with shortcut list
- [ ] Preset selector (default, vim, emacs, custom)
- [ ] Searchable/filterable shortcut list
- [ ] Visual keybinding recorder for custom shortcuts
- [ ] Conflict detection and resolution
- [ ] Reset individual or all keybindings
- [ ] Keyboard shortcut categories
- [ ] Export/import keybindings

---

## Implementation Details

### 1. Keybinding Types (src/lib/types/keybindings.ts)

```typescript
/**
 * Keyboard shortcut types and definitions.
 */

export interface KeyBinding {
  id: string;
  command: string;
  label: string;
  description?: string;
  category: KeyBindingCategory;
  key: string;
  when?: string;
  isDefault: boolean;
  isCustom: boolean;
}

export type KeyBindingCategory =
  | 'general'
  | 'navigation'
  | 'editor'
  | 'mission'
  | 'git'
  | 'terminal';

export interface KeyBindingPreset {
  id: string;
  name: string;
  description: string;
  bindings: Record<string, string>;
}

export interface KeyCombination {
  key: string;
  ctrl: boolean;
  alt: boolean;
  shift: boolean;
  meta: boolean;
}

export interface KeyBindingConflict {
  key: string;
  bindings: KeyBinding[];
}

export const KEYBINDING_CATEGORIES: { id: KeyBindingCategory; label: string; icon: string }[] = [
  { id: 'general', label: 'General', icon: 'settings' },
  { id: 'navigation', label: 'Navigation', icon: 'compass' },
  { id: 'editor', label: 'Editor', icon: 'edit-3' },
  { id: 'mission', label: 'Mission', icon: 'target' },
  { id: 'git', label: 'Git', icon: 'git-branch' },
  { id: 'terminal', label: 'Terminal', icon: 'terminal' },
];

export const DEFAULT_KEYBINDINGS: KeyBinding[] = [
  // General
  { id: 'settings', command: 'app.openSettings', label: 'Open Settings', category: 'general', key: 'Cmd+,', isDefault: true, isCustom: false },
  { id: 'commandPalette', command: 'app.commandPalette', label: 'Command Palette', category: 'general', key: 'Cmd+Shift+P', isDefault: true, isCustom: false },
  { id: 'search', command: 'app.search', label: 'Search', category: 'general', key: 'Cmd+F', isDefault: true, isCustom: false },
  { id: 'quit', command: 'app.quit', label: 'Quit Application', category: 'general', key: 'Cmd+Q', isDefault: true, isCustom: false },

  // Navigation
  { id: 'goBack', command: 'nav.back', label: 'Go Back', category: 'navigation', key: 'Cmd+[', isDefault: true, isCustom: false },
  { id: 'goForward', command: 'nav.forward', label: 'Go Forward', category: 'navigation', key: 'Cmd+]', isDefault: true, isCustom: false },
  { id: 'focusSidebar', command: 'nav.focusSidebar', label: 'Focus Sidebar', category: 'navigation', key: 'Cmd+1', isDefault: true, isCustom: false },
  { id: 'focusMain', command: 'nav.focusMain', label: 'Focus Main Panel', category: 'navigation', key: 'Cmd+2', isDefault: true, isCustom: false },
  { id: 'toggleSidebar', command: 'nav.toggleSidebar', label: 'Toggle Sidebar', category: 'navigation', key: 'Cmd+B', isDefault: true, isCustom: false },

  // Editor
  { id: 'save', command: 'editor.save', label: 'Save', category: 'editor', key: 'Cmd+S', isDefault: true, isCustom: false },
  { id: 'saveAll', command: 'editor.saveAll', label: 'Save All', category: 'editor', key: 'Cmd+Shift+S', isDefault: true, isCustom: false },
  { id: 'undo', command: 'editor.undo', label: 'Undo', category: 'editor', key: 'Cmd+Z', isDefault: true, isCustom: false },
  { id: 'redo', command: 'editor.redo', label: 'Redo', category: 'editor', key: 'Cmd+Shift+Z', isDefault: true, isCustom: false },
  { id: 'copy', command: 'editor.copy', label: 'Copy', category: 'editor', key: 'Cmd+C', isDefault: true, isCustom: false },
  { id: 'cut', command: 'editor.cut', label: 'Cut', category: 'editor', key: 'Cmd+X', isDefault: true, isCustom: false },
  { id: 'paste', command: 'editor.paste', label: 'Paste', category: 'editor', key: 'Cmd+V', isDefault: true, isCustom: false },
  { id: 'selectAll', command: 'editor.selectAll', label: 'Select All', category: 'editor', key: 'Cmd+A', isDefault: true, isCustom: false },
  { id: 'find', command: 'editor.find', label: 'Find', category: 'editor', key: 'Cmd+F', isDefault: true, isCustom: false },
  { id: 'replace', command: 'editor.replace', label: 'Find and Replace', category: 'editor', key: 'Cmd+H', isDefault: true, isCustom: false },

  // Mission
  { id: 'newMission', command: 'mission.new', label: 'New Mission', category: 'mission', key: 'Cmd+N', isDefault: true, isCustom: false },
  { id: 'runMission', command: 'mission.run', label: 'Run Mission', category: 'mission', key: 'Cmd+Enter', isDefault: true, isCustom: false },
  { id: 'stopMission', command: 'mission.stop', label: 'Stop Mission', category: 'mission', key: 'Cmd+.', isDefault: true, isCustom: false },
  { id: 'restartMission', command: 'mission.restart', label: 'Restart Mission', category: 'mission', key: 'Cmd+Shift+R', isDefault: true, isCustom: false },

  // Git
  { id: 'gitCommit', command: 'git.commit', label: 'Commit', category: 'git', key: 'Cmd+Shift+C', isDefault: true, isCustom: false },
  { id: 'gitPush', command: 'git.push', label: 'Push', category: 'git', key: 'Cmd+Shift+U', isDefault: true, isCustom: false },
  { id: 'gitPull', command: 'git.pull', label: 'Pull', category: 'git', key: 'Cmd+Shift+D', isDefault: true, isCustom: false },

  // Terminal
  { id: 'newTerminal', command: 'terminal.new', label: 'New Terminal', category: 'terminal', key: 'Cmd+`', isDefault: true, isCustom: false },
  { id: 'toggleTerminal', command: 'terminal.toggle', label: 'Toggle Terminal', category: 'terminal', key: 'Cmd+J', isDefault: true, isCustom: false },
  { id: 'clearTerminal', command: 'terminal.clear', label: 'Clear Terminal', category: 'terminal', key: 'Cmd+K', isDefault: true, isCustom: false },
];

export const KEYBINDING_PRESETS: KeyBindingPreset[] = [
  {
    id: 'default',
    name: 'Default',
    description: 'Standard keyboard shortcuts',
    bindings: {},
  },
  {
    id: 'vim',
    name: 'Vim',
    description: 'Vim-style navigation and editing',
    bindings: {
      'nav.back': 'Ctrl+O',
      'nav.forward': 'Ctrl+I',
      'editor.save': ':w',
      'editor.saveAll': ':wa',
    },
  },
  {
    id: 'emacs',
    name: 'Emacs',
    description: 'Emacs-style shortcuts',
    bindings: {
      'editor.save': 'Ctrl+X Ctrl+S',
      'editor.find': 'Ctrl+S',
      'editor.undo': 'Ctrl+/',
      'editor.copy': 'Alt+W',
      'editor.cut': 'Ctrl+W',
      'editor.paste': 'Ctrl+Y',
    },
  },
];
```

### 2. Keybinding Store (src/lib/stores/keybinding-store.ts)

```typescript
import { writable, derived, get } from 'svelte/store';
import type { KeyBinding, KeyBindingConflict, KeyCombination } from '$lib/types/keybindings';
import { DEFAULT_KEYBINDINGS, KEYBINDING_PRESETS } from '$lib/types/keybindings';
import { keybindingSettings, settingsStore } from './settings-store';

function createKeybindingStore() {
  const customBindings = writable<Map<string, string>>(new Map());

  function getEffectiveBindings(): KeyBinding[] {
    const preset = get(keybindingSettings).preset;
    const custom = get(customBindings);
    const presetBindings = KEYBINDING_PRESETS.find(p => p.id === preset)?.bindings || {};

    return DEFAULT_KEYBINDINGS.map(binding => {
      const customKey = custom.get(binding.command);
      const presetKey = presetBindings[binding.command];
      const effectiveKey = customKey || presetKey || binding.key;

      return {
        ...binding,
        key: effectiveKey,
        isDefault: effectiveKey === binding.key,
        isCustom: !!customKey,
      };
    });
  }

  function findConflicts(): KeyBindingConflict[] {
    const bindings = getEffectiveBindings();
    const keyMap = new Map<string, KeyBinding[]>();

    bindings.forEach(binding => {
      const existing = keyMap.get(binding.key) || [];
      existing.push(binding);
      keyMap.set(binding.key, existing);
    });

    const conflicts: KeyBindingConflict[] = [];
    keyMap.forEach((bindings, key) => {
      if (bindings.length > 1) {
        conflicts.push({ key, bindings });
      }
    });

    return conflicts;
  }

  return {
    subscribe: derived([keybindingSettings, customBindings], () => ({
      bindings: getEffectiveBindings(),
      conflicts: findConflicts(),
    })).subscribe,

    setPreset: (presetId: string) => {
      settingsStore.updateSetting('keybindings', 'preset', presetId as any);
      customBindings.set(new Map());
    },

    setBinding: (command: string, key: string) => {
      customBindings.update(m => {
        const newMap = new Map(m);
        newMap.set(command, key);
        return newMap;
      });

      const custom = get(customBindings);
      settingsStore.updateSetting('keybindings', 'customBindings',
        Array.from(custom.entries()).map(([command, key]) => ({
          id: command,
          command,
          key,
        }))
      );
    },

    resetBinding: (command: string) => {
      customBindings.update(m => {
        const newMap = new Map(m);
        newMap.delete(command);
        return newMap;
      });
    },

    resetAll: () => {
      customBindings.set(new Map());
      settingsStore.updateSetting('keybindings', 'customBindings', []);
    },

    parseKeyCombination: (event: KeyboardEvent): KeyCombination => ({
      key: event.key,
      ctrl: event.ctrlKey,
      alt: event.altKey,
      shift: event.shiftKey,
      meta: event.metaKey,
    }),

    formatKeyCombination: (combo: KeyCombination): string => {
      const parts: string[] = [];
      if (combo.meta) parts.push('Cmd');
      if (combo.ctrl) parts.push('Ctrl');
      if (combo.alt) parts.push('Alt');
      if (combo.shift) parts.push('Shift');

      let key = combo.key;
      if (key === ' ') key = 'Space';
      if (key.length === 1) key = key.toUpperCase();
      parts.push(key);

      return parts.join('+');
    },
  };
}

export const keybindingStore = createKeybindingStore();
```

### 3. Keybindings Settings Component (src/lib/components/settings/KeybindingsSettings.svelte)

```svelte
<script lang="ts">
  import { keybindingStore } from '$lib/stores/keybinding-store';
  import { keybindingSettings } from '$lib/stores/settings-store';
  import { KEYBINDING_CATEGORIES, KEYBINDING_PRESETS, DEFAULT_KEYBINDINGS } from '$lib/types/keybindings';
  import type { KeyBinding, KeyBindingCategory, KeyCombination } from '$lib/types/keybindings';
  import SettingsSection from './SettingsSection.svelte';
  import Select from '$lib/components/ui/Select.svelte';
  import Input from '$lib/components/ui/Input.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Icon from '$lib/components/ui/Icon.svelte';
  import Modal from '$lib/components/ui/Modal.svelte';

  let searchQuery = '';
  let selectedCategory: KeyBindingCategory | 'all' = 'all';
  let recordingBinding: KeyBinding | null = null;
  let recordedKey: string = '';
  let conflictWarning: string = '';

  function handlePresetChange(event: CustomEvent<string>) {
    keybindingStore.setPreset(event.detail);
  }

  function filterBindings(bindings: KeyBinding[]): KeyBinding[] {
    return bindings.filter(binding => {
      const matchesSearch = !searchQuery ||
        binding.label.toLowerCase().includes(searchQuery.toLowerCase()) ||
        binding.command.toLowerCase().includes(searchQuery.toLowerCase()) ||
        binding.key.toLowerCase().includes(searchQuery.toLowerCase());

      const matchesCategory = selectedCategory === 'all' || binding.category === selectedCategory;

      return matchesSearch && matchesCategory;
    });
  }

  function groupByCategory(bindings: KeyBinding[]): Record<KeyBindingCategory, KeyBinding[]> {
    const groups: Record<string, KeyBinding[]> = {};

    KEYBINDING_CATEGORIES.forEach(cat => {
      groups[cat.id] = [];
    });

    bindings.forEach(binding => {
      if (groups[binding.category]) {
        groups[binding.category].push(binding);
      }
    });

    return groups as Record<KeyBindingCategory, KeyBinding[]>;
  }

  function startRecording(binding: KeyBinding) {
    recordingBinding = binding;
    recordedKey = '';
    conflictWarning = '';
  }

  function stopRecording() {
    recordingBinding = null;
    recordedKey = '';
    conflictWarning = '';
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (!recordingBinding) return;

    event.preventDefault();
    event.stopPropagation();

    // Ignore standalone modifier keys
    if (['Control', 'Alt', 'Shift', 'Meta'].includes(event.key)) {
      return;
    }

    // Escape cancels
    if (event.key === 'Escape') {
      stopRecording();
      return;
    }

    const combo = keybindingStore.parseKeyCombination(event);
    const formatted = keybindingStore.formatKeyCombination(combo);
    recordedKey = formatted;

    // Check for conflicts
    const existing = $keybindingStore.bindings.find(
      b => b.key === formatted && b.command !== recordingBinding?.command
    );

    if (existing) {
      conflictWarning = `Conflicts with "${existing.label}"`;
    } else {
      conflictWarning = '';
    }
  }

  function saveBinding() {
    if (!recordingBinding || !recordedKey) return;

    keybindingStore.setBinding(recordingBinding.command, recordedKey);
    stopRecording();
  }

  function resetBinding(binding: KeyBinding) {
    keybindingStore.resetBinding(binding.command);
  }

  function getDefaultKey(command: string): string {
    return DEFAULT_KEYBINDINGS.find(b => b.command === command)?.key || '';
  }

  $: filteredBindings = filterBindings($keybindingStore.bindings);
  $: groupedBindings = groupByCategory(filteredBindings);
  $: hasConflicts = $keybindingStore.conflicts.length > 0;
</script>

<svelte:window on:keydown={recordingBinding ? handleKeyDown : undefined} />

<div class="keybindings-settings">
  <h2 class="settings-title">Keyboard Shortcuts</h2>
  <p class="settings-description">
    Customize keyboard shortcuts to match your workflow.
  </p>

  <!-- Preset Selection -->
  <SettingsSection title="Preset">
    <div class="preset-selector">
      {#each KEYBINDING_PRESETS as preset}
        <button
          class="preset-option"
          class:preset-option--active={$keybindingSettings.preset === preset.id}
          on:click={() => keybindingStore.setPreset(preset.id)}
        >
          <span class="preset-option__name">{preset.name}</span>
          <span class="preset-option__desc">{preset.description}</span>
        </button>
      {/each}
    </div>
  </SettingsSection>

  <!-- Conflicts Warning -->
  {#if hasConflicts}
    <div class="conflicts-warning">
      <Icon name="alert-triangle" size={20} />
      <div class="conflicts-warning__content">
        <strong>{$keybindingStore.conflicts.length} shortcut conflict{$keybindingStore.conflicts.length > 1 ? 's' : ''} detected</strong>
        <ul>
          {#each $keybindingStore.conflicts as conflict}
            <li>
              <kbd>{conflict.key}</kbd>: {conflict.bindings.map(b => b.label).join(', ')}
            </li>
          {/each}
        </ul>
      </div>
    </div>
  {/if}

  <!-- Search and Filter -->
  <SettingsSection title="Shortcuts">
    <div class="shortcuts-toolbar">
      <div class="shortcuts-search">
        <Icon name="search" size={16} />
        <Input
          type="text"
          placeholder="Search shortcuts..."
          bind:value={searchQuery}
        />
      </div>

      <Select
        value={selectedCategory}
        options={[
          { value: 'all', label: 'All Categories' },
          ...KEYBINDING_CATEGORIES.map(c => ({ value: c.id, label: c.label })),
        ]}
        on:change={(e) => selectedCategory = e.detail}
      />

      <Button variant="secondary" on:click={() => keybindingStore.resetAll()}>
        <Icon name="rotate-ccw" size={16} />
        Reset All
      </Button>
    </div>

    <!-- Shortcuts List -->
    <div class="shortcuts-list">
      {#each KEYBINDING_CATEGORIES as category}
        {@const categoryBindings = groupedBindings[category.id]}
        {#if categoryBindings.length > 0 && (selectedCategory === 'all' || selectedCategory === category.id)}
          <div class="shortcuts-category">
            <h4 class="shortcuts-category__title">
              <Icon name={category.icon} size={16} />
              {category.label}
            </h4>

            <div class="shortcuts-table">
              {#each categoryBindings as binding}
                <div
                  class="shortcut-row"
                  class:shortcut-row--custom={binding.isCustom}
                  class:shortcut-row--conflict={$keybindingStore.conflicts.some(c =>
                    c.bindings.some(b => b.command === binding.command)
                  )}
                >
                  <div class="shortcut-row__info">
                    <span class="shortcut-row__label">{binding.label}</span>
                    {#if binding.description}
                      <span class="shortcut-row__desc">{binding.description}</span>
                    {/if}
                  </div>

                  <div class="shortcut-row__key">
                    <button
                      class="key-button"
                      on:click={() => startRecording(binding)}
                      title="Click to change"
                    >
                      <kbd>{binding.key}</kbd>
                      {#if binding.isCustom}
                        <span class="key-button__badge">Custom</span>
                      {/if}
                    </button>
                  </div>

                  <div class="shortcut-row__actions">
                    {#if binding.isCustom}
                      <Button
                        variant="ghost"
                        size="small"
                        on:click={() => resetBinding(binding)}
                        title="Reset to default ({getDefaultKey(binding.command)})"
                      >
                        <Icon name="rotate-ccw" size={14} />
                      </Button>
                    {/if}
                  </div>
                </div>
              {/each}
            </div>
          </div>
        {/if}
      {/each}

      {#if filteredBindings.length === 0}
        <div class="shortcuts-empty">
          <Icon name="search" size={48} />
          <p>No shortcuts found matching "{searchQuery}"</p>
        </div>
      {/if}
    </div>
  </SettingsSection>
</div>

<!-- Recording Modal -->
{#if recordingBinding}
  <Modal title="Set Keyboard Shortcut" on:close={stopRecording}>
    <div class="recording-modal">
      <p class="recording-modal__label">
        Press your desired key combination for:
      </p>
      <p class="recording-modal__command">{recordingBinding.label}</p>

      <div class="recording-modal__input" class:recording-modal__input--conflict={conflictWarning}>
        {#if recordedKey}
          <kbd class="recording-modal__key">{recordedKey}</kbd>
        {:else}
          <span class="recording-modal__placeholder">Press keys...</span>
        {/if}
      </div>

      {#if conflictWarning}
        <p class="recording-modal__warning">
          <Icon name="alert-triangle" size={14} />
          {conflictWarning}
        </p>
      {/if}

      <div class="recording-modal__actions">
        <Button variant="secondary" on:click={stopRecording}>
          Cancel
        </Button>
        <Button
          variant="primary"
          disabled={!recordedKey}
          on:click={saveBinding}
        >
          Save Shortcut
        </Button>
      </div>

      <p class="recording-modal__hint">
        Press <kbd>Escape</kbd> to cancel
      </p>
    </div>
  </Modal>
{/if}

<style>
  .keybindings-settings {
    max-width: 800px;
  }

  .settings-title {
    font-size: 24px;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0 0 8px 0;
  }

  .settings-description {
    color: var(--color-text-secondary);
    font-size: 14px;
    margin: 0 0 24px 0;
  }

  /* Preset Selector */
  .preset-selector {
    display: flex;
    gap: 12px;
  }

  .preset-option {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    padding: 16px;
    border: 2px solid var(--color-border);
    border-radius: 8px;
    background: transparent;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .preset-option:hover {
    border-color: var(--color-text-muted);
  }

  .preset-option--active {
    border-color: var(--color-primary);
    background: var(--color-bg-active);
  }

  .preset-option__name {
    font-size: 14px;
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .preset-option__desc {
    font-size: 12px;
    color: var(--color-text-muted);
    margin-top: 4px;
  }

  /* Conflicts Warning */
  .conflicts-warning {
    display: flex;
    gap: 12px;
    padding: 16px;
    background: rgba(255, 152, 0, 0.1);
    border: 1px solid var(--color-warning);
    border-radius: 8px;
    margin-bottom: 24px;
    color: var(--color-warning);
  }

  .conflicts-warning__content {
    flex: 1;
  }

  .conflicts-warning__content strong {
    display: block;
    margin-bottom: 8px;
    color: var(--color-text-primary);
  }

  .conflicts-warning__content ul {
    margin: 0;
    padding-left: 20px;
    color: var(--color-text-secondary);
    font-size: 13px;
  }

  /* Toolbar */
  .shortcuts-toolbar {
    display: flex;
    gap: 12px;
    margin-bottom: 16px;
  }

  .shortcuts-search {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0 12px;
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: 6px;
  }

  .shortcuts-search :global(input) {
    border: none;
    background: transparent;
  }

  /* Shortcuts List */
  .shortcuts-list {
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .shortcuts-category__title {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0 0 12px 0;
  }

  .shortcuts-table {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .shortcut-row {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 10px 12px;
    background: var(--color-bg-secondary);
    border-radius: 6px;
    transition: background 0.15s ease;
  }

  .shortcut-row:hover {
    background: var(--color-bg-hover);
  }

  .shortcut-row--custom {
    background: rgba(33, 150, 243, 0.05);
  }

  .shortcut-row--conflict {
    background: rgba(255, 152, 0, 0.1);
  }

  .shortcut-row__info {
    flex: 1;
    min-width: 0;
  }

  .shortcut-row__label {
    font-size: 14px;
    color: var(--color-text-primary);
  }

  .shortcut-row__desc {
    display: block;
    font-size: 12px;
    color: var(--color-text-muted);
    margin-top: 2px;
  }

  .shortcut-row__key {
    min-width: 150px;
    text-align: center;
  }

  .key-button {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 4px 8px;
    border: none;
    background: transparent;
    cursor: pointer;
    border-radius: 4px;
    transition: background 0.15s ease;
  }

  .key-button:hover {
    background: var(--color-bg-primary);
  }

  .key-button kbd {
    padding: 4px 8px;
    background: var(--color-bg-primary);
    border: 1px solid var(--color-border);
    border-radius: 4px;
    font-family: inherit;
    font-size: 13px;
    color: var(--color-text-primary);
  }

  .key-button__badge {
    font-size: 10px;
    padding: 2px 6px;
    background: var(--color-primary);
    color: white;
    border-radius: 4px;
  }

  .shortcut-row__actions {
    width: 40px;
  }

  .shortcuts-empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 48px;
    color: var(--color-text-muted);
    text-align: center;
  }

  /* Recording Modal */
  .recording-modal {
    text-align: center;
    padding: 16px;
  }

  .recording-modal__label {
    font-size: 14px;
    color: var(--color-text-secondary);
    margin: 0 0 8px 0;
  }

  .recording-modal__command {
    font-size: 18px;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0 0 24px 0;
  }

  .recording-modal__input {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 80px;
    margin-bottom: 16px;
    padding: 16px;
    background: var(--color-bg-secondary);
    border: 2px solid var(--color-primary);
    border-radius: 12px;
  }

  .recording-modal__input--conflict {
    border-color: var(--color-warning);
  }

  .recording-modal__key {
    font-size: 24px;
    padding: 8px 16px;
    background: var(--color-bg-primary);
    border: 1px solid var(--color-border);
    border-radius: 8px;
  }

  .recording-modal__placeholder {
    font-size: 16px;
    color: var(--color-text-muted);
  }

  .recording-modal__warning {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    color: var(--color-warning);
    font-size: 13px;
    margin-bottom: 16px;
  }

  .recording-modal__actions {
    display: flex;
    justify-content: center;
    gap: 12px;
    margin-bottom: 16px;
  }

  .recording-modal__hint {
    font-size: 12px;
    color: var(--color-text-muted);
    margin: 0;
  }

  .recording-modal__hint kbd {
    padding: 2px 6px;
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: 4px;
    font-size: 11px;
  }
</style>
```

---

## Testing Requirements

1. Preset selection changes keybindings
2. Search filters shortcuts correctly
3. Category filter works
4. Key recording captures combinations
5. Conflict detection shows warnings
6. Custom bindings are saved
7. Reset binding restores default
8. Reset all clears customizations

### Test File (src/lib/components/settings/__tests__/KeybindingsSettings.test.ts)

```typescript
import { render, fireEvent, screen } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import KeybindingsSettings from '../KeybindingsSettings.svelte';
import { keybindingStore } from '$lib/stores/keybinding-store';
import { settingsStore } from '$lib/stores/settings-store';

describe('KeybindingsSettings', () => {
  beforeEach(() => {
    settingsStore.resetAll();
  });

  it('renders keybinding categories', () => {
    render(KeybindingsSettings);

    expect(screen.getByText('General')).toBeInTheDocument();
    expect(screen.getByText('Navigation')).toBeInTheDocument();
    expect(screen.getByText('Editor')).toBeInTheDocument();
  });

  it('filters shortcuts by search', async () => {
    render(KeybindingsSettings);

    const searchInput = screen.getByPlaceholderText('Search shortcuts...');
    await fireEvent.input(searchInput, { target: { value: 'save' } });

    expect(screen.getByText('Save')).toBeInTheDocument();
    expect(screen.queryByText('Quit Application')).not.toBeInTheDocument();
  });

  it('changes preset', async () => {
    render(KeybindingsSettings);

    const vimButton = screen.getByText('Vim').closest('button');
    await fireEvent.click(vimButton!);

    const state = get(settingsStore);
    expect(state.settings.keybindings.preset).toBe('vim');
  });

  it('opens recording modal on key button click', async () => {
    render(KeybindingsSettings);

    const keyButton = screen.getAllByRole('button').find(
      btn => btn.querySelector('kbd')
    );
    await fireEvent.click(keyButton!);

    expect(screen.getByText('Set Keyboard Shortcut')).toBeInTheDocument();
  });

  it('records key combination', async () => {
    render(KeybindingsSettings);

    // Open recording modal
    const keyButton = screen.getAllByRole('button').find(
      btn => btn.querySelector('kbd')
    );
    await fireEvent.click(keyButton!);

    // Simulate key press
    await fireEvent.keyDown(window, {
      key: 'K',
      metaKey: true,
      shiftKey: true,
    });

    expect(screen.getByText('Cmd+Shift+K')).toBeInTheDocument();
  });

  it('detects conflicts', async () => {
    render(KeybindingsSettings);

    // Set a conflicting binding
    keybindingStore.setBinding('app.search', 'Cmd+S');

    expect(screen.getByText(/conflict/i)).toBeInTheDocument();
  });
});

function get<T>(store: { subscribe: (fn: (value: T) => void) => void }): T {
  let value: T;
  store.subscribe(v => value = v)();
  return value!;
}
```

---

## Related Specs

- Depends on: [271-settings-layout.md](271-settings-layout.md)
- Depends on: [272-settings-store.md](272-settings-store.md)
- Previous: [275-settings-backends.md](275-settings-backends.md)
- Next: [277-settings-editor.md](277-settings-editor.md)
