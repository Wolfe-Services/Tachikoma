<script lang="ts">
  import { page } from '$app/stores';
  import { missionStore, isRunning } from '$lib/stores/mission';
  
  export let currentRoute: string;
  export let collapsed = false;

  interface NavItem {
    path: string;
    icon: string;
    label: string;
    badge?: string;
  }

  const navItems: NavItem[] = [
    { path: '/', icon: 'dashboard', label: 'Dashboard' },
    { path: '/mission', icon: 'mission', label: 'Mission', badge: $isRunning ? '●' : undefined },
    { path: '/specs', icon: 'specs', label: 'Specs' },
    { path: '/forge', icon: 'forge', label: 'Forge' },
    { path: '/history', icon: 'history', label: 'History' },
    { path: '/settings', icon: 'settings', label: 'Settings' }
  ];

  function toggleSidebar() {
    collapsed = !collapsed;
  }

  function isActiveRoute(itemPath: string): boolean {
    if (itemPath === '/') {
      return $page.route.id === '/' || $page.route.id === null;
    }
    return $page.route.id?.startsWith(itemPath) ?? false;
  }

  function getIcon(iconName: string): string {
    const icons: Record<string, string> = {
      dashboard: `<path d="M3 13h8V3H3v10zm0 8h8v-6H3v6zm10 0h8V11h-8v10zm0-18v6h8V3h-8z"/>`,
      mission: `<path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"/>`,
      specs: `<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8l-6-6z"/><path d="M14 2v6h6"/><path d="M16 13H8"/><path d="M16 17H8"/><path d="M10 9H8"/>`,
      forge: `<path d="M18.56 5.44l.94 2.06.94-2.06 2.06-.94-2.06-.94-.94-2.06-.94 2.06-2.06.94z"/><path d="M11.24 11.24l-.94 2.06-.94-2.06-2.06-.94 2.06-.94.94-2.06.94 2.06 2.06.94z"/><path d="M16.76 6.76l-1.94-1.94L6.76 12.88a3 3 0 0 0 0 4.24l.94.94c1.16 1.16 3.08 1.16 4.24 0l8.06-8.06z"/>`,
      history: `<circle cx="12" cy="12" r="10"/><path d="M12 6v6l4 2"/>`,
      settings: `<circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1 1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/>`
    };
    return icons[iconName] || icons.dashboard;
  }
</script>

<aside class="sidebar" class:collapsed>
  <div class="sidebar-header">
    <button class="sidebar-toggle" on:click={toggleSidebar} aria-label="Toggle sidebar">
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M3 12h18M3 6h18M3 18h18"/>
      </svg>
    </button>
  </div>

  <nav class="sidebar-nav">
    {#each navItems as item}
      <a 
        href={item.path}
        class="nav-item"
        class:active={isActiveRoute(item.path)}
        title={collapsed ? item.label : undefined}
      >
        <div class="nav-icon">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
            {@html getIcon(item.icon)}
          </svg>
        </div>
        {#if !collapsed}
          <span class="nav-label">{item.label}</span>
          {#if item.badge}
            <span class="nav-badge">{item.badge}</span>
          {/if}
        {/if}
      </a>
    {/each}
  </nav>

  <div class="sidebar-footer">
    {#if !collapsed}
      <div class="version-info">
        <div class="version-label">Version</div>
        <div class="version-number">1.0.0-beta</div>
      </div>
    {/if}
  </div>
</aside>

<style>
  .sidebar {
    width: 240px;
    background: var(--bg-secondary);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    transition: width 0.2s ease;
    position: relative;
    z-index: 10;
  }

  .sidebar.collapsed {
    width: 64px;
  }

  .sidebar-header {
    height: 60px;
    display: flex;
    align-items: center;
    justify-content: flex-end;
    padding: 0 1rem;
    border-bottom: 1px solid var(--border);
  }

  .sidebar-toggle {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border: none;
    background: transparent;
    color: var(--text-muted);
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .sidebar-toggle:hover {
    background: var(--bg);
    color: var(--text);
  }

  .sidebar-nav {
    flex: 1;
    padding: 1rem 0;
    overflow-y: auto;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem 1rem;
    margin: 0 0.5rem;
    border-radius: 6px;
    text-decoration: none;
    color: var(--text-muted);
    transition: all 0.2s ease;
    position: relative;
  }

  .nav-item:hover {
    background: var(--bg);
    color: var(--text);
  }

  .nav-item.active {
    background: var(--accent);
    color: white;
  }

  .collapsed .nav-item {
    justify-content: center;
    padding: 0.75rem;
    margin: 0 0.5rem;
  }

  .nav-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .nav-label {
    font-size: 0.875rem;
    font-weight: 500;
    white-space: nowrap;
  }

  .nav-badge {
    margin-left: auto;
    background: #ef4444;
    color: white;
    font-size: 0.625rem;
    padding: 0.125rem 0.375rem;
    border-radius: 10px;
    font-weight: 600;
    min-width: 16px;
    height: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .sidebar-footer {
    padding: 1rem;
    border-top: 1px solid var(--border);
  }

  .version-info {
    text-align: center;
  }

  .version-label {
    font-size: 0.75rem;
    color: var(--text-muted);
    margin-bottom: 0.25rem;
  }

  .version-number {
    font-size: 0.75rem;
    color: var(--accent);
    font-weight: 600;
  }

  @media (max-width: 768px) {
    .sidebar {
      position: fixed;
      top: 60px;
      left: 0;
      bottom: 0;
      transform: translateX(-100%);
      transition: transform 0.3s ease;
      z-index: 50;
    }

    .sidebar:not(.collapsed) {
      transform: translateX(0);
      width: 280px;
    }

    .sidebar.collapsed {
      transform: translateX(-100%);
      width: 240px;
    }
  }
</style>