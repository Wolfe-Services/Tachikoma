# 296 - Dashboard Layout

**Phase:** 14 - Dashboard
**Spec ID:** 296
**Status:** Planned
**Dependencies:** 004-svelte-integration
**Estimated Context:** ~12% of Sonnet window

---

## Objective

Create the main dashboard layout component that provides the structural foundation for all dashboard views, including navigation, sidebar, header, and content areas.

---

## Acceptance Criteria

- [x] `DashboardLayout.svelte` component created
- [x] Responsive grid/flex layout system
- [x] Collapsible sidebar navigation
- [x] Header with user info and quick actions
- [x] Main content area with slot support
- [x] Footer with status indicators
- [x] Dark/light theme support
- [x] Keyboard navigation shortcuts

---

## Implementation Details

### 1. Dashboard Layout Component (web/src/lib/components/dashboard/DashboardLayout.svelte)

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import Sidebar from './Sidebar.svelte';
  import Header from './Header.svelte';
  import Footer from './Footer.svelte';
  import { dashboardStore } from '$lib/stores/dashboard';
  import { theme } from '$lib/stores/theme';

  export let title: string = 'Dashboard';
  export let showSidebar: boolean = true;

  let sidebarCollapsed = false;
  let isMobile = false;

  $: layoutClass = sidebarCollapsed ? 'sidebar-collapsed' : 'sidebar-expanded';
  $: themeClass = $theme === 'dark' ? 'theme-dark' : 'theme-light';

  function toggleSidebar() {
    sidebarCollapsed = !sidebarCollapsed;
    dashboardStore.setSidebarState(sidebarCollapsed);
  }

  function handleKeydown(event: KeyboardEvent) {
    // Ctrl+B to toggle sidebar
    if (event.ctrlKey && event.key === 'b') {
      event.preventDefault();
      toggleSidebar();
    }
    // Ctrl+/ to focus search
    if (event.ctrlKey && event.key === '/') {
      event.preventDefault();
      document.querySelector<HTMLInputElement>('#global-search')?.focus();
    }
  }

  onMount(() => {
    const mediaQuery = window.matchMedia('(max-width: 768px)');
    isMobile = mediaQuery.matches;

    const handler = (e: MediaQueryListEvent) => {
      isMobile = e.matches;
      if (isMobile) sidebarCollapsed = true;
    };

    mediaQuery.addEventListener('change', handler);
    return () => mediaQuery.removeEventListener('change', handler);
  });
</script>

<svelte:window on:keydown={handleKeydown} />

<div class="dashboard-layout {layoutClass} {themeClass}">
  {#if showSidebar}
    <aside class="dashboard-sidebar" class:collapsed={sidebarCollapsed}>
      <Sidebar
        collapsed={sidebarCollapsed}
        currentPath={$page.url.pathname}
        on:toggle={toggleSidebar}
      />
    </aside>
  {/if}

  <div class="dashboard-main">
    <Header {title} on:toggleSidebar={toggleSidebar} />

    <main class="dashboard-content">
      <slot />
    </main>

    <Footer />
  </div>
</div>

<style>
  .dashboard-layout {
    display: grid;
    grid-template-columns: auto 1fr;
    min-height: 100vh;
    background: var(--bg-primary);
    color: var(--text-primary);
  }

  .dashboard-layout.sidebar-collapsed {
    grid-template-columns: 60px 1fr;
  }

  .dashboard-layout.sidebar-expanded {
    grid-template-columns: 260px 1fr;
  }

  .dashboard-sidebar {
    position: sticky;
    top: 0;
    height: 100vh;
    overflow-y: auto;
    background: var(--bg-secondary);
    border-right: 1px solid var(--border-color);
    transition: width 0.2s ease;
  }

  .dashboard-main {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
  }

  .dashboard-content {
    flex: 1;
    padding: 1.5rem;
    overflow-y: auto;
  }

  @media (max-width: 768px) {
    .dashboard-layout {
      grid-template-columns: 1fr;
    }

    .dashboard-sidebar {
      position: fixed;
      z-index: 100;
      transform: translateX(-100%);
    }

    .dashboard-sidebar:not(.collapsed) {
      transform: translateX(0);
    }
  }
</style>
```

### 2. Sidebar Component (web/src/lib/components/dashboard/Sidebar.svelte)

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { navigationItems, type NavItem } from '$lib/config/navigation';
  import Icon from '$lib/components/common/Icon.svelte';

  export let collapsed: boolean = false;
  export let currentPath: string = '/';

  const dispatch = createEventDispatcher();

  function isActive(item: NavItem): boolean {
    return currentPath.startsWith(item.path);
  }
</script>

<nav class="sidebar" class:collapsed aria-label="Main navigation">
  <div class="sidebar-header">
    <a href="/" class="logo">
      {#if collapsed}
        <span class="logo-icon">T</span>
      {:else}
        <span class="logo-full">Tachikoma</span>
      {/if}
    </a>
  </div>

  <ul class="nav-list">
    {#each navigationItems as item}
      <li class="nav-item">
        <a
          href={item.path}
          class="nav-link"
          class:active={isActive(item)}
          aria-current={isActive(item) ? 'page' : undefined}
          title={collapsed ? item.label : undefined}
        >
          <Icon name={item.icon} size={20} />
          {#if !collapsed}
            <span class="nav-label">{item.label}</span>
          {/if}
          {#if item.badge && !collapsed}
            <span class="nav-badge">{item.badge}</span>
          {/if}
        </a>
      </li>
    {/each}
  </ul>

  <button
    class="collapse-btn"
    on:click={() => dispatch('toggle')}
    aria-label={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
  >
    <Icon name={collapsed ? 'chevron-right' : 'chevron-left'} />
  </button>
</nav>

<style>
  .sidebar {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: 1rem 0;
  }

  .sidebar-header {
    padding: 0 1rem 1rem;
    border-bottom: 1px solid var(--border-color);
  }

  .logo {
    font-size: 1.25rem;
    font-weight: 700;
    color: var(--text-primary);
    text-decoration: none;
  }

  .nav-list {
    flex: 1;
    list-style: none;
    padding: 1rem 0.5rem;
    margin: 0;
  }

  .nav-link {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem 1rem;
    border-radius: 0.5rem;
    color: var(--text-secondary);
    text-decoration: none;
    transition: all 0.15s ease;
  }

  .nav-link:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .nav-link.active {
    background: var(--accent-bg);
    color: var(--accent-color);
  }

  .nav-badge {
    margin-left: auto;
    padding: 0.125rem 0.5rem;
    font-size: 0.75rem;
    background: var(--accent-color);
    color: white;
    border-radius: 9999px;
  }

  .collapse-btn {
    margin: 0.5rem;
    padding: 0.5rem;
    border: none;
    background: var(--bg-hover);
    border-radius: 0.5rem;
    cursor: pointer;
  }

  .collapsed .nav-link {
    justify-content: center;
    padding: 0.75rem;
  }
</style>
```

### 3. Header Component (web/src/lib/components/dashboard/Header.svelte)

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { user } from '$lib/stores/auth';
  import { theme, toggleTheme } from '$lib/stores/theme';
  import Icon from '$lib/components/common/Icon.svelte';
  import SearchBar from '$lib/components/common/SearchBar.svelte';
  import UserMenu from '$lib/components/common/UserMenu.svelte';

  export let title: string = 'Dashboard';

  const dispatch = createEventDispatcher();
</script>

<header class="dashboard-header">
  <div class="header-left">
    <button
      class="menu-toggle"
      on:click={() => dispatch('toggleSidebar')}
      aria-label="Toggle sidebar"
    >
      <Icon name="menu" />
    </button>
    <h1 class="page-title">{title}</h1>
  </div>

  <div class="header-center">
    <SearchBar id="global-search" placeholder="Search missions, specs..." />
  </div>

  <div class="header-right">
    <button
      class="theme-toggle"
      on:click={toggleTheme}
      aria-label="Toggle theme"
    >
      <Icon name={$theme === 'dark' ? 'sun' : 'moon'} />
    </button>

    <button class="notifications" aria-label="Notifications">
      <Icon name="bell" />
      <span class="notification-dot" />
    </button>

    {#if $user}
      <UserMenu user={$user} />
    {/if}
  </div>
</header>

<style>
  .dashboard-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: 1rem 1.5rem;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .menu-toggle {
    display: none;
    padding: 0.5rem;
    border: none;
    background: transparent;
    cursor: pointer;
  }

  .page-title {
    font-size: 1.25rem;
    font-weight: 600;
    margin: 0;
  }

  .header-center {
    flex: 1;
    max-width: 400px;
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .theme-toggle,
  .notifications {
    position: relative;
    padding: 0.5rem;
    border: none;
    background: transparent;
    cursor: pointer;
    border-radius: 0.5rem;
  }

  .theme-toggle:hover,
  .notifications:hover {
    background: var(--bg-hover);
  }

  .notification-dot {
    position: absolute;
    top: 0.25rem;
    right: 0.25rem;
    width: 0.5rem;
    height: 0.5rem;
    background: var(--danger-color);
    border-radius: 50%;
  }

  @media (max-width: 768px) {
    .menu-toggle {
      display: block;
    }

    .header-center {
      display: none;
    }
  }
</style>
```

### 4. Footer Component (web/src/lib/components/dashboard/Footer.svelte)

```svelte
<script lang="ts">
  import { connectionStatus } from '$lib/stores/connection';
  import { version } from '$lib/config/app';

  $: statusColor = {
    connected: 'var(--success-color)',
    connecting: 'var(--warning-color)',
    disconnected: 'var(--danger-color)'
  }[$connectionStatus];
</script>

<footer class="dashboard-footer">
  <div class="footer-left">
    <span class="status-indicator" style="background: {statusColor}" />
    <span class="status-text">{$connectionStatus}</span>
  </div>

  <div class="footer-right">
    <span class="version">v{version}</span>
  </div>
</footer>

<style>
  .dashboard-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.5rem 1.5rem;
    background: var(--bg-secondary);
    border-top: 1px solid var(--border-color);
    font-size: 0.75rem;
    color: var(--text-tertiary);
  }

  .footer-left {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .status-indicator {
    width: 0.5rem;
    height: 0.5rem;
    border-radius: 50%;
  }

  .version {
    font-family: monospace;
  }
</style>
```

### 5. Dashboard Store (web/src/lib/stores/dashboard.ts)

```typescript
import { writable, derived } from 'svelte/store';

interface DashboardState {
  sidebarCollapsed: boolean;
  activeView: string;
  refreshInterval: number;
  lastRefresh: Date | null;
}

function createDashboardStore() {
  const { subscribe, set, update } = writable<DashboardState>({
    sidebarCollapsed: false,
    activeView: 'overview',
    refreshInterval: 30000, // 30 seconds
    lastRefresh: null
  });

  return {
    subscribe,
    setSidebarState: (collapsed: boolean) =>
      update(s => ({ ...s, sidebarCollapsed: collapsed })),
    setActiveView: (view: string) =>
      update(s => ({ ...s, activeView: view })),
    setRefreshInterval: (interval: number) =>
      update(s => ({ ...s, refreshInterval: interval })),
    markRefreshed: () =>
      update(s => ({ ...s, lastRefresh: new Date() })),
    reset: () => set({
      sidebarCollapsed: false,
      activeView: 'overview',
      refreshInterval: 30000,
      lastRefresh: null
    })
  };
}

export const dashboardStore = createDashboardStore();

export const sidebarCollapsed = derived(
  dashboardStore,
  $store => $store.sidebarCollapsed
);
```

---

## Testing Requirements

1. Layout renders correctly at all breakpoints
2. Sidebar toggles and persists state
3. Keyboard shortcuts work correctly
4. Theme switching applies to all components
5. Navigation highlights active route
6. Footer shows correct connection status

---

## Related Specs

- Depends on: [004-svelte-integration.md](../phase-00-setup/004-svelte-integration.md)
- Next: [297-mission-cards.md](297-mission-cards.md)
- Used by: All dashboard views
