# 271 - Settings Page Layout

**Phase:** 13 - Settings UI
**Spec ID:** 271
**Status:** Planned
**Dependencies:** 186-sveltekit-setup, 188-layout-system, 189-store-architecture
**Estimated Context:** ~10% of model context window

---

## Objective

Create the main Settings page layout component that provides a responsive sidebar navigation with category sections and a content area for displaying individual settings panels. The layout should support keyboard navigation, search functionality, and accessibility features.

---

## Acceptance Criteria

- [ ] `SettingsLayout.svelte` component created with two-column layout
- [ ] Sidebar navigation with collapsible category groups
- [ ] Settings search functionality with highlighting
- [ ] Breadcrumb navigation for nested settings
- [ ] Responsive design with mobile drawer navigation
- [ ] Keyboard navigation support (arrow keys, enter, escape)
- [ ] ARIA labels and roles for accessibility
- [ ] Smooth transitions between settings panels

---

## Implementation Details

### 1. Types (src/lib/types/settings-layout.ts)

```typescript
/**
 * Settings layout configuration and navigation types.
 */

export interface SettingsCategory {
  id: string;
  label: string;
  icon: string;
  description?: string;
  items: SettingsNavItem[];
}

export interface SettingsNavItem {
  id: string;
  label: string;
  path: string;
  icon?: string;
  badge?: string | number;
  disabled?: boolean;
  keywords?: string[];
}

export interface SettingsBreadcrumb {
  label: string;
  path: string;
}

export interface SettingsLayoutState {
  activeCategory: string;
  activeItem: string;
  sidebarCollapsed: boolean;
  searchQuery: string;
  searchResults: SettingsNavItem[];
  breadcrumbs: SettingsBreadcrumb[];
}

export const SETTINGS_CATEGORIES: SettingsCategory[] = [
  {
    id: 'general',
    label: 'General',
    icon: 'settings',
    items: [
      { id: 'general', label: 'General', path: '/settings/general', keywords: ['startup', 'language', 'updates'] },
      { id: 'appearance', label: 'Appearance', path: '/settings/appearance', keywords: ['theme', 'colors', 'fonts'] },
    ]
  },
  {
    id: 'editor',
    label: 'Editor',
    icon: 'code',
    items: [
      { id: 'editor', label: 'Code Editor', path: '/settings/editor', keywords: ['syntax', 'indentation', 'autocomplete'] },
      { id: 'keybindings', label: 'Keyboard Shortcuts', path: '/settings/keybindings', keywords: ['hotkeys', 'bindings'] },
    ]
  },
  {
    id: 'integrations',
    label: 'Integrations',
    icon: 'puzzle',
    items: [
      { id: 'backends', label: 'LLM Backends', path: '/settings/backends', keywords: ['api', 'claude', 'openai', 'ollama'] },
      { id: 'git', label: 'Git Integration', path: '/settings/git', keywords: ['github', 'repository', 'version'] },
    ]
  },
  {
    id: 'data',
    label: 'Data & Sync',
    icon: 'cloud',
    items: [
      { id: 'sync', label: 'Sync Settings', path: '/settings/sync', keywords: ['cloud', 'backup'] },
      { id: 'profiles', label: 'Profiles', path: '/settings/profiles', keywords: ['workspace', 'project'] },
      { id: 'export', label: 'Export', path: '/settings/export', keywords: ['backup', 'download'] },
      { id: 'import', label: 'Import', path: '/settings/import', keywords: ['restore', 'upload'] },
      { id: 'reset', label: 'Reset', path: '/settings/reset', keywords: ['defaults', 'clear'] },
    ]
  }
];
```

### 2. Settings Navigation Store (src/lib/stores/settings-nav-store.ts)

```typescript
import { writable, derived } from 'svelte/store';
import type { SettingsLayoutState, SettingsNavItem, SettingsBreadcrumb } from '$lib/types/settings-layout';
import { SETTINGS_CATEGORIES } from '$lib/types/settings-layout';

function createSettingsNavStore() {
  const initialState: SettingsLayoutState = {
    activeCategory: 'general',
    activeItem: 'general',
    sidebarCollapsed: false,
    searchQuery: '',
    searchResults: [],
    breadcrumbs: [{ label: 'Settings', path: '/settings' }]
  };

  const { subscribe, set, update } = writable<SettingsLayoutState>(initialState);

  function getAllItems(): SettingsNavItem[] {
    return SETTINGS_CATEGORIES.flatMap(cat => cat.items);
  }

  function searchSettings(query: string): SettingsNavItem[] {
    if (!query.trim()) return [];

    const lowerQuery = query.toLowerCase();
    return getAllItems().filter(item => {
      const matchLabel = item.label.toLowerCase().includes(lowerQuery);
      const matchKeywords = item.keywords?.some(kw => kw.toLowerCase().includes(lowerQuery));
      return matchLabel || matchKeywords;
    });
  }

  return {
    subscribe,
    set,
    update,

    setActiveItem: (categoryId: string, itemId: string) => {
      update(s => ({
        ...s,
        activeCategory: categoryId,
        activeItem: itemId,
        breadcrumbs: [
          { label: 'Settings', path: '/settings' },
          { label: SETTINGS_CATEGORIES.find(c => c.id === categoryId)?.label || '', path: `/settings/${categoryId}` },
          { label: getAllItems().find(i => i.id === itemId)?.label || '', path: `/settings/${itemId}` }
        ].filter(b => b.label)
      }));
    },

    toggleSidebar: () => update(s => ({ ...s, sidebarCollapsed: !s.sidebarCollapsed })),

    setSearchQuery: (query: string) => {
      update(s => ({
        ...s,
        searchQuery: query,
        searchResults: searchSettings(query)
      }));
    },

    clearSearch: () => update(s => ({ ...s, searchQuery: '', searchResults: [] })),

    reset: () => set(initialState)
  };
}

export const settingsNavStore = createSettingsNavStore();

export const settingsSearchResults = derived(
  settingsNavStore,
  $nav => $nav.searchResults
);

export const settingsBreadcrumbs = derived(
  settingsNavStore,
  $nav => $nav.breadcrumbs
);
```

### 3. Settings Sidebar Component (src/lib/components/settings/SettingsSidebar.svelte)

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { settingsNavStore } from '$lib/stores/settings-nav-store';
  import { SETTINGS_CATEGORIES } from '$lib/types/settings-layout';
  import type { SettingsNavItem } from '$lib/types/settings-layout';
  import Icon from '$lib/components/ui/Icon.svelte';

  export let collapsed = false;

  const dispatch = createEventDispatcher<{
    navigate: { categoryId: string; itemId: string; path: string };
  }>();

  let expandedCategories: Set<string> = new Set(['general', 'editor', 'integrations', 'data']);
  let focusedIndex = -1;
  let searchInputRef: HTMLInputElement;

  function toggleCategory(categoryId: string) {
    if (expandedCategories.has(categoryId)) {
      expandedCategories.delete(categoryId);
    } else {
      expandedCategories.add(categoryId);
    }
    expandedCategories = expandedCategories;
  }

  function handleItemClick(categoryId: string, item: SettingsNavItem) {
    if (item.disabled) return;
    settingsNavStore.setActiveItem(categoryId, item.id);
    dispatch('navigate', { categoryId, itemId: item.id, path: item.path });
  }

  function handleSearch(event: Event) {
    const target = event.target as HTMLInputElement;
    settingsNavStore.setSearchQuery(target.value);
  }

  function handleKeyDown(event: KeyboardEvent) {
    const allItems = SETTINGS_CATEGORIES.flatMap(cat =>
      cat.items.filter(item => !item.disabled)
    );

    switch (event.key) {
      case 'ArrowDown':
        event.preventDefault();
        focusedIndex = Math.min(focusedIndex + 1, allItems.length - 1);
        break;
      case 'ArrowUp':
        event.preventDefault();
        focusedIndex = Math.max(focusedIndex - 1, 0);
        break;
      case 'Enter':
        if (focusedIndex >= 0 && focusedIndex < allItems.length) {
          const item = allItems[focusedIndex];
          const category = SETTINGS_CATEGORIES.find(c => c.items.includes(item));
          if (category) {
            handleItemClick(category.id, item);
          }
        }
        break;
      case 'Escape':
        settingsNavStore.clearSearch();
        searchInputRef?.blur();
        break;
    }
  }

  function highlightMatch(text: string, query: string): string {
    if (!query) return text;
    const regex = new RegExp(`(${query})`, 'gi');
    return text.replace(regex, '<mark>$1</mark>');
  }

  $: isSearching = $settingsNavStore.searchQuery.length > 0;
</script>

<aside
  class="settings-sidebar"
  class:settings-sidebar--collapsed={collapsed}
  role="navigation"
  aria-label="Settings navigation"
>
  <div class="settings-sidebar__search">
    <Icon name="search" size={16} />
    <input
      bind:this={searchInputRef}
      type="text"
      placeholder="Search settings..."
      value={$settingsNavStore.searchQuery}
      on:input={handleSearch}
      on:keydown={handleKeyDown}
      aria-label="Search settings"
    />
    {#if isSearching}
      <button
        class="settings-sidebar__search-clear"
        on:click={() => settingsNavStore.clearSearch()}
        aria-label="Clear search"
      >
        <Icon name="x" size={14} />
      </button>
    {/if}
  </div>

  {#if isSearching}
    <div class="settings-sidebar__results" role="listbox">
      {#each $settingsNavStore.searchResults as item, index}
        {@const category = SETTINGS_CATEGORIES.find(c => c.items.some(i => i.id === item.id))}
        <button
          class="settings-sidebar__result-item"
          class:focused={focusedIndex === index}
          role="option"
          aria-selected={$settingsNavStore.activeItem === item.id}
          on:click={() => category && handleItemClick(category.id, item)}
        >
          <span class="settings-sidebar__result-label">
            {@html highlightMatch(item.label, $settingsNavStore.searchQuery)}
          </span>
          <span class="settings-sidebar__result-category">{category?.label}</span>
        </button>
      {:else}
        <div class="settings-sidebar__no-results">No settings found</div>
      {/each}
    </div>
  {:else}
    <nav class="settings-sidebar__nav">
      {#each SETTINGS_CATEGORIES as category}
        <div class="settings-sidebar__category">
          <button
            class="settings-sidebar__category-header"
            on:click={() => toggleCategory(category.id)}
            aria-expanded={expandedCategories.has(category.id)}
          >
            <Icon name={category.icon} size={18} />
            <span>{category.label}</span>
            <Icon
              name="chevron-down"
              size={14}
              class="settings-sidebar__chevron {expandedCategories.has(category.id) ? 'expanded' : ''}"
            />
          </button>

          {#if expandedCategories.has(category.id)}
            <ul class="settings-sidebar__items" role="list">
              {#each category.items as item}
                <li>
                  <button
                    class="settings-sidebar__item"
                    class:settings-sidebar__item--active={$settingsNavStore.activeItem === item.id}
                    class:settings-sidebar__item--disabled={item.disabled}
                    disabled={item.disabled}
                    on:click={() => handleItemClick(category.id, item)}
                    aria-current={$settingsNavStore.activeItem === item.id ? 'page' : undefined}
                  >
                    {#if item.icon}
                      <Icon name={item.icon} size={16} />
                    {/if}
                    <span>{item.label}</span>
                    {#if item.badge}
                      <span class="settings-sidebar__badge">{item.badge}</span>
                    {/if}
                  </button>
                </li>
              {/each}
            </ul>
          {/if}
        </div>
      {/each}
    </nav>
  {/if}
</aside>

<style>
  .settings-sidebar {
    width: 260px;
    height: 100%;
    background: var(--color-bg-secondary);
    border-right: 1px solid var(--color-border);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    transition: width 0.2s ease;
  }

  .settings-sidebar--collapsed {
    width: 0;
  }

  .settings-sidebar__search {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 16px;
    border-bottom: 1px solid var(--color-border);
  }

  .settings-sidebar__search input {
    flex: 1;
    border: none;
    background: transparent;
    color: var(--color-text-primary);
    font-size: 14px;
    outline: none;
  }

  .settings-sidebar__search input::placeholder {
    color: var(--color-text-muted);
  }

  .settings-sidebar__search-clear {
    padding: 4px;
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
    border-radius: 4px;
  }

  .settings-sidebar__search-clear:hover {
    color: var(--color-text-primary);
    background: var(--color-bg-hover);
  }

  .settings-sidebar__nav {
    flex: 1;
    overflow-y: auto;
    padding: 8px 0;
  }

  .settings-sidebar__category {
    margin-bottom: 4px;
  }

  .settings-sidebar__category-header {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 8px 16px;
    border: none;
    background: transparent;
    color: var(--color-text-secondary);
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    cursor: pointer;
    transition: color 0.15s ease;
  }

  .settings-sidebar__category-header:hover {
    color: var(--color-text-primary);
  }

  .settings-sidebar__chevron {
    margin-left: auto;
    transition: transform 0.2s ease;
  }

  .settings-sidebar__chevron.expanded {
    transform: rotate(180deg);
  }

  .settings-sidebar__items {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .settings-sidebar__item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 8px 16px 8px 40px;
    border: none;
    background: transparent;
    color: var(--color-text-secondary);
    font-size: 14px;
    text-align: left;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .settings-sidebar__item:hover:not(:disabled) {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .settings-sidebar__item--active {
    background: var(--color-bg-active);
    color: var(--color-primary);
  }

  .settings-sidebar__item--disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .settings-sidebar__badge {
    margin-left: auto;
    padding: 2px 6px;
    background: var(--color-primary);
    color: white;
    font-size: 11px;
    font-weight: 600;
    border-radius: 10px;
  }

  .settings-sidebar__results {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
  }

  .settings-sidebar__result-item {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    width: 100%;
    padding: 10px 12px;
    border: none;
    background: transparent;
    border-radius: 6px;
    cursor: pointer;
    transition: background 0.15s ease;
  }

  .settings-sidebar__result-item:hover,
  .settings-sidebar__result-item.focused {
    background: var(--color-bg-hover);
  }

  .settings-sidebar__result-label {
    color: var(--color-text-primary);
    font-size: 14px;
  }

  .settings-sidebar__result-label :global(mark) {
    background: var(--color-warning);
    color: var(--color-text-primary);
    border-radius: 2px;
  }

  .settings-sidebar__result-category {
    color: var(--color-text-muted);
    font-size: 12px;
    margin-top: 2px;
  }

  .settings-sidebar__no-results {
    padding: 20px;
    text-align: center;
    color: var(--color-text-muted);
    font-size: 14px;
  }
</style>
```

### 4. Main Settings Layout Component (src/lib/components/settings/SettingsLayout.svelte)

```svelte
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { settingsNavStore, settingsBreadcrumbs } from '$lib/stores/settings-nav-store';
  import SettingsSidebar from './SettingsSidebar.svelte';
  import Icon from '$lib/components/ui/Icon.svelte';

  export let title = 'Settings';

  let isMobile = false;
  let mobileMenuOpen = false;

  function handleResize() {
    isMobile = window.innerWidth < 768;
    if (!isMobile) {
      mobileMenuOpen = false;
    }
  }

  function handleNavigate(event: CustomEvent<{ categoryId: string; itemId: string; path: string }>) {
    goto(event.detail.path);
    if (isMobile) {
      mobileMenuOpen = false;
    }
  }

  function handleKeyDown(event: KeyboardEvent) {
    // Cmd/Ctrl + , to open settings (if not already there)
    if ((event.metaKey || event.ctrlKey) && event.key === ',') {
      event.preventDefault();
    }

    // Escape to close mobile menu
    if (event.key === 'Escape' && mobileMenuOpen) {
      mobileMenuOpen = false;
    }
  }

  onMount(() => {
    handleResize();
    window.addEventListener('resize', handleResize);
    window.addEventListener('keydown', handleKeyDown);
  });

  onDestroy(() => {
    window.removeEventListener('resize', handleResize);
    window.removeEventListener('keydown', handleKeyDown);
  });
</script>

<div class="settings-layout" role="main" aria-label="Settings">
  <!-- Mobile Header -->
  {#if isMobile}
    <header class="settings-layout__mobile-header">
      <button
        class="settings-layout__menu-btn"
        on:click={() => mobileMenuOpen = !mobileMenuOpen}
        aria-label={mobileMenuOpen ? 'Close menu' : 'Open menu'}
        aria-expanded={mobileMenuOpen}
      >
        <Icon name={mobileMenuOpen ? 'x' : 'menu'} size={24} />
      </button>
      <h1 class="settings-layout__title">{title}</h1>
    </header>
  {/if}

  <!-- Sidebar (drawer on mobile) -->
  <div
    class="settings-layout__sidebar-container"
    class:settings-layout__sidebar-container--mobile={isMobile}
    class:settings-layout__sidebar-container--open={mobileMenuOpen}
  >
    {#if isMobile && mobileMenuOpen}
      <div
        class="settings-layout__overlay"
        on:click={() => mobileMenuOpen = false}
        on:keydown={(e) => e.key === 'Enter' && (mobileMenuOpen = false)}
        role="button"
        tabindex="0"
        aria-label="Close menu"
      />
    {/if}
    <SettingsSidebar
      collapsed={isMobile && !mobileMenuOpen}
      on:navigate={handleNavigate}
    />
  </div>

  <!-- Main Content -->
  <div class="settings-layout__content">
    <!-- Breadcrumbs -->
    {#if !isMobile}
      <nav class="settings-layout__breadcrumbs" aria-label="Breadcrumb">
        <ol>
          {#each $settingsBreadcrumbs as crumb, index}
            <li>
              {#if index < $settingsBreadcrumbs.length - 1}
                <a href={crumb.path}>{crumb.label}</a>
                <Icon name="chevron-right" size={14} />
              {:else}
                <span aria-current="page">{crumb.label}</span>
              {/if}
            </li>
          {/each}
        </ol>
      </nav>
    {/if}

    <!-- Settings Panel Content -->
    <div class="settings-layout__panel">
      <slot />
    </div>
  </div>
</div>

<style>
  .settings-layout {
    display: flex;
    height: 100%;
    width: 100%;
    background: var(--color-bg-primary);
    overflow: hidden;
  }

  .settings-layout__mobile-header {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    height: 56px;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 0 16px;
    background: var(--color-bg-secondary);
    border-bottom: 1px solid var(--color-border);
    z-index: 100;
  }

  .settings-layout__menu-btn {
    padding: 8px;
    border: none;
    background: transparent;
    color: var(--color-text-primary);
    cursor: pointer;
    border-radius: 6px;
  }

  .settings-layout__menu-btn:hover {
    background: var(--color-bg-hover);
  }

  .settings-layout__title {
    font-size: 18px;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0;
  }

  .settings-layout__sidebar-container {
    flex-shrink: 0;
  }

  .settings-layout__sidebar-container--mobile {
    position: fixed;
    top: 56px;
    left: 0;
    bottom: 0;
    z-index: 99;
    transform: translateX(-100%);
    transition: transform 0.3s ease;
  }

  .settings-layout__sidebar-container--open {
    transform: translateX(0);
  }

  .settings-layout__overlay {
    position: fixed;
    top: 56px;
    left: 260px;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    z-index: 98;
  }

  .settings-layout__content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    padding-top: 0;
  }

  @media (max-width: 767px) {
    .settings-layout__content {
      padding-top: 56px;
    }
  }

  .settings-layout__breadcrumbs {
    padding: 16px 24px;
    border-bottom: 1px solid var(--color-border);
  }

  .settings-layout__breadcrumbs ol {
    display: flex;
    align-items: center;
    gap: 8px;
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .settings-layout__breadcrumbs li {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .settings-layout__breadcrumbs a {
    color: var(--color-text-muted);
    text-decoration: none;
    font-size: 14px;
  }

  .settings-layout__breadcrumbs a:hover {
    color: var(--color-primary);
  }

  .settings-layout__breadcrumbs span {
    color: var(--color-text-primary);
    font-size: 14px;
    font-weight: 500;
  }

  .settings-layout__panel {
    flex: 1;
    overflow-y: auto;
    padding: 24px;
  }
</style>
```

---

## Testing Requirements

1. Layout renders correctly at all breakpoints
2. Sidebar navigation expands/collapses categories
3. Search filters settings correctly with highlighting
4. Mobile drawer opens/closes properly
5. Keyboard navigation works as expected
6. Breadcrumbs update correctly on navigation
7. ARIA labels are set for accessibility
8. Transitions animate smoothly

### Test File (src/lib/components/settings/__tests__/SettingsLayout.test.ts)

```typescript
import { render, fireEvent, screen } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import SettingsLayout from '../SettingsLayout.svelte';
import { settingsNavStore } from '$lib/stores/settings-nav-store';

describe('SettingsLayout', () => {
  beforeEach(() => {
    settingsNavStore.reset();
  });

  it('renders with default layout', () => {
    render(SettingsLayout);

    expect(screen.getByRole('main', { name: 'Settings' })).toBeInTheDocument();
    expect(screen.getByRole('navigation', { name: 'Settings navigation' })).toBeInTheDocument();
  });

  it('expands and collapses categories', async () => {
    render(SettingsLayout);

    const generalHeader = screen.getByText('General');
    await fireEvent.click(generalHeader);

    // Check if items are hidden after collapse
    const generalItem = screen.queryByText('General', { selector: '.settings-sidebar__item span' });
    expect(generalItem).not.toBeVisible();
  });

  it('searches settings correctly', async () => {
    render(SettingsLayout);

    const searchInput = screen.getByPlaceholderText('Search settings...');
    await fireEvent.input(searchInput, { target: { value: 'theme' } });

    expect(screen.getByText('Appearance')).toBeInTheDocument();
  });

  it('clears search on escape', async () => {
    render(SettingsLayout);

    const searchInput = screen.getByPlaceholderText('Search settings...');
    await fireEvent.input(searchInput, { target: { value: 'test' } });
    await fireEvent.keyDown(searchInput, { key: 'Escape' });

    expect(searchInput).toHaveValue('');
  });

  it('navigates with keyboard', async () => {
    render(SettingsLayout);

    const searchInput = screen.getByPlaceholderText('Search settings...');
    await fireEvent.input(searchInput, { target: { value: 'editor' } });
    await fireEvent.keyDown(searchInput, { key: 'ArrowDown' });
    await fireEvent.keyDown(searchInput, { key: 'Enter' });

    // Verify navigation occurred
    const state = get(settingsNavStore);
    expect(state.activeItem).toBe('editor');
  });

  it('shows mobile menu on small screens', async () => {
    Object.defineProperty(window, 'innerWidth', { value: 500, writable: true });
    window.dispatchEvent(new Event('resize'));

    render(SettingsLayout);

    expect(screen.getByRole('button', { name: 'Open menu' })).toBeInTheDocument();
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

- Depends on: [186-sveltekit-setup.md](../phase-09-ui-foundation/186-sveltekit-setup.md)
- Depends on: [188-layout-system.md](../phase-09-ui-foundation/188-layout-system.md)
- Depends on: [189-store-architecture.md](../phase-09-ui-foundation/189-store-architecture.md)
- Next: [272-settings-store.md](272-settings-store.md)
- Used by: All Settings UI specs (272-285)
