# Spec 276: Settings Layout

## Header
- **Spec ID**: 276
- **Phase**: 13 - Settings UI
- **Component**: Settings Layout
- **Dependencies**: Core UI infrastructure
- **Status**: Draft

## Objective
Create the main settings layout with navigation sidebar, content area, and search functionality for managing all application configuration options.

## Acceptance Criteria
1. Responsive settings layout with collapsible sidebar
2. Category-based navigation with icons
3. Global search across all settings
4. Breadcrumb navigation for nested settings
5. Sticky header with save/reset actions
6. Settings change indicator (unsaved changes)
7. Keyboard navigation support
8. Settings persistence and sync status

## Implementation

### SettingsLayout.svelte
```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { writable, derived } from 'svelte/store';
  import { page } from '$app/stores';
  import SettingsSidebar from './SettingsSidebar.svelte';
  import SettingsSearch from './SettingsSearch.svelte';
  import SettingsBreadcrumb from './SettingsBreadcrumb.svelte';
  import UnsavedChangesIndicator from './UnsavedChangesIndicator.svelte';
  import { settingsStore } from '$lib/stores/settings';
  import type { SettingsCategory, SettingsItem } from '$lib/types/settings';

  export let activeCategory: string = 'general';

  const categories: SettingsCategory[] = [
    { id: 'general', label: 'General', icon: 'settings', path: '/settings' },
    { id: 'backend', label: 'Backend', icon: 'server', path: '/settings/backend' },
    { id: 'brains', label: 'Brains', icon: 'brain', path: '/settings/brains' },
    { id: 'forge', label: 'Forge', icon: 'hammer', path: '/settings/forge' },
    { id: 'appearance', label: 'Appearance', icon: 'palette', path: '/settings/appearance' },
    { id: 'accessibility', label: 'Accessibility', icon: 'accessibility', path: '/settings/accessibility' },
    { id: 'keyboard', label: 'Keyboard', icon: 'keyboard', path: '/settings/keyboard' },
    { id: 'notifications', label: 'Notifications', icon: 'bell', path: '/settings/notifications' },
    { id: 'data', label: 'Data & Storage', icon: 'database', path: '/settings/data' },
    { id: 'profiles', label: 'Profiles', icon: 'user', path: '/settings/profiles' },
    { id: 'workspace', label: 'Workspace', icon: 'folder', path: '/settings/workspace' },
    { id: 'advanced', label: 'Advanced', icon: 'code', path: '/settings/advanced' }
  ];

  let sidebarCollapsed = writable<boolean>(false);
  let showSearch = writable<boolean>(false);
  let searchQuery = writable<string>('');
  let searchResults = writable<SettingsItem[]>([]);

  const hasUnsavedChanges = derived(settingsStore, ($store) =>
    $store.pendingChanges.size > 0
  );

  const syncStatus = derived(settingsStore, ($store) => $store.syncStatus);

  const currentCategory = derived(
    [() => activeCategory],
    () => categories.find(c => c.id === activeCategory) || categories[0]
  );

  async function handleSearch(query: string) {
    if (!query.trim()) {
      searchResults.set([]);
      return;
    }

    const results = await settingsStore.search(query);
    searchResults.set(results);
  }

  async function saveAllChanges() {
    await settingsStore.saveAll();
  }

  async function resetChanges() {
    await settingsStore.resetPending();
  }

  function handleKeydown(event: KeyboardEvent) {
    // Ctrl/Cmd + K for search
    if ((event.ctrlKey || event.metaKey) && event.key === 'k') {
      event.preventDefault();
      showSearch.set(true);
    }

    // Ctrl/Cmd + S for save
    if ((event.ctrlKey || event.metaKey) && event.key === 's') {
      event.preventDefault();
      if ($hasUnsavedChanges) {
        saveAllChanges();
      }
    }

    // Escape to close search
    if (event.key === 'Escape' && $showSearch) {
      showSearch.set(false);
      searchQuery.set('');
    }
  }

  onMount(() => {
    window.addEventListener('keydown', handleKeydown);
    return () => window.removeEventListener('keydown', handleKeydown);
  });

  $: handleSearch($searchQuery);
</script>

<div
  class="settings-layout"
  class:sidebar-collapsed={$sidebarCollapsed}
  data-testid="settings-layout"
>
  <SettingsSidebar
    {categories}
    {activeCategory}
    collapsed={$sidebarCollapsed}
    on:toggle={() => sidebarCollapsed.update(v => !v)}
  />

  <main class="settings-main">
    <header class="settings-header">
      <div class="header-left">
        <button
          class="toggle-sidebar-btn"
          on:click={() => sidebarCollapsed.update(v => !v)}
          aria-label={$sidebarCollapsed ? 'Expand sidebar' : 'Collapse sidebar'}
        >
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor">
            <path d="M4 6h16M4 12h16M4 18h16" stroke-width="2" stroke-linecap="round"/>
          </svg>
        </button>

        <SettingsBreadcrumb category={$currentCategory} />
      </div>

      <div class="header-center">
        <button
          class="search-btn"
          on:click={() => showSearch.set(true)}
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor">
            <circle cx="11" cy="11" r="8" stroke-width="2"/>
            <path d="M21 21l-4.35-4.35" stroke-width="2" stroke-linecap="round"/>
          </svg>
          <span>Search settings...</span>
          <kbd>Ctrl K</kbd>
        </button>
      </div>

      <div class="header-right">
        {#if $hasUnsavedChanges}
          <UnsavedChangesIndicator
            count={$settingsStore.pendingChanges.size}
            on:save={saveAllChanges}
            on:reset={resetChanges}
          />
        {/if}

        <div class="sync-status" class:syncing={$syncStatus === 'syncing'}>
          {#if $syncStatus === 'synced'}
            <span class="status-icon synced">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>
              </svg>
            </span>
            <span>Synced</span>
          {:else if $syncStatus === 'syncing'}
            <span class="status-icon syncing">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor">
                <path d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" stroke-width="2"/>
              </svg>
            </span>
            <span>Syncing...</span>
          {:else if $syncStatus === 'error'}
            <span class="status-icon error">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/>
              </svg>
            </span>
            <span>Sync error</span>
          {/if}
        </div>
      </div>
    </header>

    <div class="settings-content">
      <slot />
    </div>
  </main>

  {#if $showSearch}
    <SettingsSearch
      query={$searchQuery}
      results={$searchResults}
      on:close={() => {
        showSearch.set(false);
        searchQuery.set('');
      }}
      on:input={(e) => searchQuery.set(e.detail)}
      on:select={(e) => {
        // Navigate to selected setting
        showSearch.set(false);
        searchQuery.set('');
      }}
    />
  {/if}
</div>

<style>
  .settings-layout {
    display: flex;
    height: 100vh;
    background: var(--settings-bg, #f5f5f5);
  }

  .settings-main {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    transition: margin-left 0.2s ease;
  }

  .sidebar-collapsed .settings-main {
    margin-left: 0;
  }

  .settings-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.5rem;
    background: var(--card-bg);
    border-bottom: 1px solid var(--border-color);
    min-height: 64px;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .toggle-sidebar-btn {
    padding: 0.5rem;
    background: transparent;
    border: none;
    border-radius: 4px;
    color: var(--text-secondary);
    cursor: pointer;
  }

  .toggle-sidebar-btn:hover {
    background: var(--hover-bg);
    color: var(--text-primary);
  }

  .header-center {
    flex: 1;
    display: flex;
    justify-content: center;
    max-width: 500px;
    margin: 0 2rem;
  }

  .search-btn {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    width: 100%;
    padding: 0.5rem 1rem;
    background: var(--secondary-bg);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    color: var(--text-muted);
    font-size: 0.875rem;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .search-btn:hover {
    border-color: var(--primary-color);
    background: var(--hover-bg);
  }

  .search-btn kbd {
    margin-left: auto;
    padding: 0.125rem 0.375rem;
    background: var(--card-bg);
    border: 1px solid var(--border-color);
    border-radius: 4px;
    font-family: inherit;
    font-size: 0.75rem;
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .sync-status {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.8125rem;
    color: var(--text-muted);
  }

  .status-icon {
    display: flex;
    align-items: center;
  }

  .status-icon.synced {
    color: var(--success-color);
  }

  .status-icon.syncing {
    animation: spin 1s linear infinite;
    color: var(--info-color);
  }

  .status-icon.error {
    color: var(--error-color);
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .settings-content {
    flex: 1;
    overflow-y: auto;
    padding: 1.5rem 2rem;
  }

  @media (max-width: 768px) {
    .settings-header {
      padding: 1rem;
    }

    .header-center {
      display: none;
    }

    .settings-content {
      padding: 1rem;
    }
  }
</style>
```

### Types (settings.ts)
```typescript
export interface SettingsCategory {
  id: string;
  label: string;
  icon: string;
  path: string;
  children?: SettingsCategory[];
}

export interface SettingsItem {
  id: string;
  category: string;
  label: string;
  description: string;
  type: SettingType;
  value: unknown;
  defaultValue: unknown;
  path: string;
  keywords: string[];
}

export type SettingType =
  | 'text'
  | 'number'
  | 'boolean'
  | 'select'
  | 'multi-select'
  | 'color'
  | 'file'
  | 'key-binding'
  | 'custom';

export type SyncStatus = 'synced' | 'syncing' | 'error' | 'offline';
```

## Testing Requirements
1. **Unit Tests**: Test navigation, search, and state management
2. **Integration Tests**: Verify settings persistence
3. **Keyboard Tests**: Test all keyboard shortcuts
4. **Responsive Tests**: Validate mobile layouts
5. **Accessibility Tests**: Ensure WCAG compliance

## Related Specs
- Spec 277: Backend Config UI
- Spec 284: Theme Selection
- Spec 295: Settings Tests
