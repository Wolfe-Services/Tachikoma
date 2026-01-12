# Spec 286: Keyboard Config

## Header
- **Spec ID**: 286
- **Phase**: 13 - Settings UI
- **Component**: Keyboard Config
- **Dependencies**: Spec 285 (Font & Accessibility)
- **Status**: Draft

## Objective
Create a keyboard shortcut configuration interface that allows users to customize, view, and manage keyboard shortcuts throughout the application, including conflict detection and preset management.

## Acceptance Criteria
1. Display all available keyboard shortcuts
2. Allow custom shortcut assignments
3. Detect and resolve shortcut conflicts
4. Support shortcut presets (Default, VS Code, Vim, etc.)
5. Enable/disable individual shortcuts
6. Search and filter shortcuts
7. Reset shortcuts to defaults
8. Export/import shortcut configurations

## Implementation

### KeyboardConfig.svelte
```svelte
<script lang="ts">
  import { createEventDispatcher, onMount, onDestroy } from 'svelte';
  import { writable, derived } from 'svelte/store';
  import { fade, slide } from 'svelte/transition';
  import ShortcutEditor from './ShortcutEditor.svelte';
  import ConflictResolver from './ConflictResolver.svelte';
  import { keyboardStore } from '$lib/stores/keyboard';
  import type {
    KeyboardShortcut,
    ShortcutCategory,
    ShortcutPreset,
    KeyCombination
  } from '$lib/types/settings';

  const dispatch = createEventDispatcher<{
    save: KeyboardShortcut[];
    reset: void;
    export: KeyboardShortcut[];
  }>();

  const categories: ShortcutCategory[] = [
    { id: 'general', name: 'General', description: 'Global application shortcuts' },
    { id: 'navigation', name: 'Navigation', description: 'Navigation and focus shortcuts' },
    { id: 'forge', name: 'Forge', description: 'Forge session shortcuts' },
    { id: 'editor', name: 'Editor', description: 'Text editing shortcuts' },
    { id: 'panels', name: 'Panels', description: 'Panel management shortcuts' },
    { id: 'debug', name: 'Debug', description: 'Debugging shortcuts' }
  ];

  const presets: ShortcutPreset[] = [
    { id: 'default', name: 'Default', description: 'Standard shortcuts' },
    { id: 'vscode', name: 'VS Code', description: 'VS Code-style shortcuts' },
    { id: 'vim', name: 'Vim', description: 'Vim-style shortcuts' },
    { id: 'emacs', name: 'Emacs', description: 'Emacs-style shortcuts' },
    { id: 'custom', name: 'Custom', description: 'Your custom shortcuts' }
  ];

  let searchQuery = writable<string>('');
  let selectedCategory = writable<string>('all');
  let editingShortcut = writable<KeyboardShortcut | null>(null);
  let showConflicts = writable<boolean>(false);
  let recordingShortcut = writable<boolean>(false);

  const shortcuts = derived(keyboardStore, ($store) => $store.shortcuts);
  const currentPreset = derived(keyboardStore, ($store) => $store.preset);
  const conflicts = derived(keyboardStore, ($store) => $store.conflicts);

  const filteredShortcuts = derived(
    [shortcuts, searchQuery, selectedCategory],
    ([$shortcuts, $query, $category]) => {
      let result = $shortcuts;

      if ($category !== 'all') {
        result = result.filter(s => s.category === $category);
      }

      if ($query) {
        const q = $query.toLowerCase();
        result = result.filter(s =>
          s.name.toLowerCase().includes(q) ||
          s.description.toLowerCase().includes(q) ||
          formatKeyCombination(s.keys).toLowerCase().includes(q)
        );
      }

      return result;
    }
  );

  const shortcutsByCategory = derived(filteredShortcuts, ($filtered) => {
    const grouped = new Map<string, KeyboardShortcut[]>();
    for (const category of categories) {
      grouped.set(category.id, $filtered.filter(s => s.category === category.id));
    }
    return grouped;
  });

  function formatKeyCombination(keys: KeyCombination): string {
    const parts: string[] = [];
    if (keys.meta) parts.push('⌘');
    if (keys.ctrl) parts.push('Ctrl');
    if (keys.alt) parts.push('Alt');
    if (keys.shift) parts.push('Shift');
    parts.push(keys.key.toUpperCase());
    return parts.join(' + ');
  }

  function editShortcut(shortcut: KeyboardShortcut) {
    editingShortcut.set(shortcut);
  }

  function saveShortcut(shortcut: KeyboardShortcut, newKeys: KeyCombination) {
    const conflict = keyboardStore.checkConflict(shortcut.id, newKeys);
    if (conflict) {
      showConflicts.set(true);
    } else {
      keyboardStore.updateShortcut(shortcut.id, newKeys);
      editingShortcut.set(null);
    }
  }

  function toggleShortcut(shortcutId: string) {
    keyboardStore.toggleEnabled(shortcutId);
  }

  function resetShortcut(shortcutId: string) {
    keyboardStore.resetToDefault(shortcutId);
  }

  function applyPreset(presetId: string) {
    if (confirm(`Apply "${presets.find(p => p.id === presetId)?.name}" preset? This will override your current shortcuts.`)) {
      keyboardStore.applyPreset(presetId);
    }
  }

  async function saveAllShortcuts() {
    await keyboardStore.save();
    dispatch('save', $shortcuts);
  }

  function resetAllToDefaults() {
    if (confirm('Reset all keyboard shortcuts to defaults?')) {
      keyboardStore.resetAllToDefaults();
      dispatch('reset');
    }
  }

  function exportShortcuts() {
    dispatch('export', $shortcuts);
    const data = JSON.stringify($shortcuts, null, 2);
    const blob = new Blob([data], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'keyboard-shortcuts.json';
    a.click();
    URL.revokeObjectURL(url);
  }

  function importShortcuts(event: Event) {
    const input = event.target as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;

    const reader = new FileReader();
    reader.onload = (e) => {
      try {
        const imported = JSON.parse(e.target?.result as string);
        keyboardStore.importShortcuts(imported);
      } catch (err) {
        alert('Failed to import shortcuts: Invalid file format');
      }
    };
    reader.readAsText(file);
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (!$recordingShortcut || !$editingShortcut) return;

    event.preventDefault();
    const keys: KeyCombination = {
      key: event.key,
      meta: event.metaKey,
      ctrl: event.ctrlKey,
      alt: event.altKey,
      shift: event.shiftKey
    };

    if (['Meta', 'Control', 'Alt', 'Shift'].includes(event.key)) return;

    saveShortcut($editingShortcut, keys);
    recordingShortcut.set(false);
  }

  onMount(() => {
    keyboardStore.load();
    window.addEventListener('keydown', handleKeyDown);
  });

  onDestroy(() => {
    window.removeEventListener('keydown', handleKeyDown);
  });
</script>

<div class="keyboard-config" data-testid="keyboard-config">
  <header class="config-header">
    <div class="header-title">
      <h2>Keyboard Shortcuts</h2>
      <p class="description">Customize keyboard shortcuts</p>
    </div>

    <div class="header-actions">
      <button class="btn secondary" on:click={exportShortcuts}>
        Export
      </button>
      <label class="btn secondary import-btn">
        Import
        <input type="file" accept=".json" on:change={importShortcuts} hidden />
      </label>
      <button class="btn secondary" on:click={resetAllToDefaults}>
        Reset All
      </button>
      <button class="btn primary" on:click={saveAllShortcuts}>
        Save Changes
      </button>
    </div>
  </header>

  <div class="preset-section">
    <label>Preset:</label>
    <div class="preset-options">
      {#each presets as preset}
        <button
          class="preset-btn"
          class:active={$currentPreset === preset.id}
          on:click={() => applyPreset(preset.id)}
        >
          {preset.name}
        </button>
      {/each}
    </div>
  </div>

  <div class="search-filter">
    <div class="search-input">
      <input
        type="text"
        placeholder="Search shortcuts..."
        bind:value={$searchQuery}
      />
    </div>

    <div class="category-filter">
      <select bind:value={$selectedCategory}>
        <option value="all">All Categories</option>
        {#each categories as category}
          <option value={category.id}>{category.name}</option>
        {/each}
      </select>
    </div>
  </div>

  {#if $conflicts.length > 0}
    <div class="conflicts-banner" transition:slide>
      <span class="conflict-icon">⚠️</span>
      <span>{$conflicts.length} shortcut conflict(s) detected</span>
      <button class="link-btn" on:click={() => showConflicts.set(true)}>
        Resolve
      </button>
    </div>
  {/if}

  <div class="shortcuts-list">
    {#each categories as category}
      {@const categoryShortcuts = $shortcutsByCategory.get(category.id) || []}
      {#if categoryShortcuts.length > 0 || $selectedCategory === 'all'}
        <section class="shortcut-category">
          <h3>{category.name}</h3>
          <p class="category-desc">{category.description}</p>

          {#if categoryShortcuts.length === 0}
            <p class="empty-category">No shortcuts in this category</p>
          {:else}
            <div class="shortcut-items">
              {#each categoryShortcuts as shortcut (shortcut.id)}
                <div
                  class="shortcut-item"
                  class:disabled={!shortcut.enabled}
                  class:editing={$editingShortcut?.id === shortcut.id}
                >
                  <div class="shortcut-info">
                    <span class="shortcut-name">{shortcut.name}</span>
                    <span class="shortcut-desc">{shortcut.description}</span>
                  </div>

                  <div class="shortcut-keys">
                    {#if $editingShortcut?.id === shortcut.id && $recordingShortcut}
                      <span class="recording">Press keys...</span>
                    {:else}
                      <kbd class="key-combo">{formatKeyCombination(shortcut.keys)}</kbd>
                    {/if}
                  </div>

                  <div class="shortcut-actions">
                    <button
                      class="action-btn"
                      title="Edit shortcut"
                      on:click={() => {
                        editShortcut(shortcut);
                        recordingShortcut.set(true);
                      }}
                    >
                      Edit
                    </button>
                    <button
                      class="action-btn"
                      title={shortcut.enabled ? 'Disable' : 'Enable'}
                      on:click={() => toggleShortcut(shortcut.id)}
                    >
                      {shortcut.enabled ? 'Disable' : 'Enable'}
                    </button>
                    {#if shortcut.isCustomized}
                      <button
                        class="action-btn"
                        title="Reset to default"
                        on:click={() => resetShortcut(shortcut.id)}
                      >
                        Reset
                      </button>
                    {/if}
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </section>
      {/if}
    {/each}
  </div>

  {#if $editingShortcut && !$recordingShortcut}
    <div class="modal-overlay" transition:fade on:click={() => editingShortcut.set(null)}>
      <div class="modal-content" on:click|stopPropagation>
        <ShortcutEditor
          shortcut={$editingShortcut}
          on:save={(e) => saveShortcut($editingShortcut, e.detail)}
          on:close={() => editingShortcut.set(null)}
        />
      </div>
    </div>
  {/if}

  {#if $showConflicts}
    <div class="modal-overlay" transition:fade on:click={() => showConflicts.set(false)}>
      <div class="modal-content" on:click|stopPropagation>
        <ConflictResolver
          conflicts={$conflicts}
          on:resolve={(e) => keyboardStore.resolveConflict(e.detail)}
          on:close={() => showConflicts.set(false)}
        />
      </div>
    </div>
  {/if}
</div>

<style>
  .keyboard-config {
    max-width: 1000px;
  }

  .config-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 1.5rem;
  }

  .header-title h2 {
    font-size: 1.5rem;
    font-weight: 600;
    margin-bottom: 0.5rem;
  }

  .description {
    color: var(--text-secondary);
    font-size: 0.875rem;
  }

  .header-actions {
    display: flex;
    gap: 0.75rem;
  }

  .preset-section {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 1rem;
    background: var(--card-bg);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    margin-bottom: 1rem;
  }

  .preset-section label {
    font-weight: 500;
    font-size: 0.875rem;
  }

  .preset-options {
    display: flex;
    gap: 0.5rem;
  }

  .preset-btn {
    padding: 0.5rem 1rem;
    background: var(--secondary-bg);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    font-size: 0.875rem;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .preset-btn:hover {
    border-color: var(--primary-color);
  }

  .preset-btn.active {
    background: var(--primary-color);
    border-color: var(--primary-color);
    color: white;
  }

  .search-filter {
    display: flex;
    gap: 1rem;
    margin-bottom: 1rem;
  }

  .search-input {
    flex: 1;
  }

  .search-input input {
    width: 100%;
    padding: 0.625rem 0.875rem;
    border: 1px solid var(--border-color);
    border-radius: 6px;
    background: var(--input-bg);
    color: var(--text-primary);
    font-size: 0.875rem;
  }

  .category-filter select {
    padding: 0.625rem 0.875rem;
    border: 1px solid var(--border-color);
    border-radius: 6px;
    background: var(--input-bg);
    color: var(--text-primary);
    font-size: 0.875rem;
  }

  .conflicts-banner {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem 1rem;
    background: var(--warning-alpha);
    border: 1px solid var(--warning-color);
    border-radius: 6px;
    margin-bottom: 1rem;
    font-size: 0.875rem;
  }

  .link-btn {
    background: none;
    border: none;
    color: var(--primary-color);
    cursor: pointer;
    text-decoration: underline;
  }

  .shortcut-category {
    background: var(--card-bg);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    padding: 1.25rem;
    margin-bottom: 1rem;
  }

  .shortcut-category h3 {
    font-size: 1rem;
    font-weight: 600;
    margin-bottom: 0.25rem;
  }

  .category-desc {
    font-size: 0.8125rem;
    color: var(--text-muted);
    margin-bottom: 1rem;
  }

  .empty-category {
    color: var(--text-muted);
    font-style: italic;
    font-size: 0.875rem;
  }

  .shortcut-items {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .shortcut-item {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.75rem 1rem;
    background: var(--secondary-bg);
    border-radius: 6px;
    transition: all 0.15s ease;
  }

  .shortcut-item.disabled {
    opacity: 0.5;
  }

  .shortcut-item.editing {
    border: 2px solid var(--primary-color);
  }

  .shortcut-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
  }

  .shortcut-name {
    font-weight: 500;
    font-size: 0.875rem;
  }

  .shortcut-desc {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .shortcut-keys {
    min-width: 150px;
  }

  .key-combo {
    display: inline-block;
    padding: 0.375rem 0.625rem;
    background: var(--input-bg);
    border: 1px solid var(--border-color);
    border-radius: 4px;
    font-family: monospace;
    font-size: 0.8125rem;
  }

  .recording {
    color: var(--primary-color);
    font-style: italic;
    animation: pulse 1s infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }

  .shortcut-actions {
    display: flex;
    gap: 0.5rem;
  }

  .action-btn {
    padding: 0.375rem 0.625rem;
    background: transparent;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    font-size: 0.75rem;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .action-btn:hover {
    border-color: var(--primary-color);
    color: var(--primary-color);
  }

  .btn {
    padding: 0.625rem 1.25rem;
    border: none;
    border-radius: 6px;
    font-weight: 500;
    cursor: pointer;
  }

  .btn.primary {
    background: var(--primary-color);
    color: white;
  }

  .btn.secondary {
    background: var(--secondary-bg);
    border: 1px solid var(--border-color);
    color: var(--text-primary);
  }

  .import-btn {
    cursor: pointer;
  }

  .modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal-content {
    background: var(--card-bg);
    border-radius: 8px;
    max-width: 500px;
    width: 90%;
    max-height: 80vh;
    overflow-y: auto;
  }
</style>
```

## Testing Requirements
1. **Unit Tests**: Test shortcut parsing and formatting
2. **Conflict Tests**: Verify conflict detection logic
3. **Preset Tests**: Test preset application
4. **Import/Export Tests**: Test configuration serialization
5. **Recording Tests**: Test keyboard capture

## Related Specs
- Spec 285: Font & Accessibility
- Spec 287: Notification Prefs
- Spec 295: Settings Tests
