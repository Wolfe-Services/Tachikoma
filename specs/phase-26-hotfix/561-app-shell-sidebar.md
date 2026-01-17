# Spec 561: App Shell with Sidebar Navigation

## Header
- **Spec ID**: 561
- **Phase**: 26 - Hotfix Critical UI
- **Priority**: P0 - CRITICAL
- **Dependencies**: None
- **Estimated Time**: 30 minutes

## Objective
Create the main application shell with a collapsible sidebar. This is the foundational layout that ALL other pages will use.

## Acceptance Criteria
- [x] File `web/src/lib/components/layout/AppShell.svelte` exists
- [x] File `web/src/lib/components/layout/Sidebar.svelte` exists
- [x] Sidebar has navigation links: Dashboard, Missions, Specs, Forge, Settings
- [x] Sidebar can collapse to icon-only mode
- [x] Collapse state persists in localStorage
- [x] Active route is highlighted in sidebar
- [x] AppShell uses CSS Grid with sidebar + main content areas

## Implementation

### Create layout folder if missing
```bash
mkdir -p web/src/lib/components/layout
```

### Sidebar.svelte
```svelte
<script lang="ts">
  import { page } from '$app/stores';
  import { writable } from 'svelte/store';
  
  // Icons (use lucide or simple SVG)
  const navItems = [
    { href: '/', label: 'Dashboard', icon: 'home' },
    { href: '/missions', label: 'Missions', icon: 'play' },
    { href: '/specs', label: 'Specs', icon: 'file-text' },
    { href: '/forge', label: 'Forge', icon: 'brain' },
    { href: '/settings', label: 'Settings', icon: 'settings' }
  ];
  
  export let collapsed = false;
  
  function toggleCollapse() {
    collapsed = !collapsed;
    localStorage.setItem('sidebar-collapsed', String(collapsed));
  }
</script>

<aside class="sidebar" class:collapsed>
  <div class="sidebar-header">
    <span class="logo">🤖</span>
    {#if !collapsed}
      <span class="title">Tachikoma</span>
    {/if}
  </div>
  
  <nav class="sidebar-nav">
    {#each navItems as item}
      <a 
        href={item.href}
        class="nav-item"
        class:active={$page.url.pathname === item.href}
      >
        <span class="icon">{item.icon}</span>
        {#if !collapsed}
          <span class="label">{item.label}</span>
        {/if}
      </a>
    {/each}
  </nav>
  
  <button class="collapse-btn" on:click={toggleCollapse}>
    {collapsed ? '→' : '←'}
  </button>
</aside>

<style>
  .sidebar {
    width: 240px;
    height: 100vh;
    background: var(--sidebar-bg, #1a1a2e);
    display: flex;
    flex-direction: column;
    transition: width 0.2s ease;
  }
  
  .sidebar.collapsed {
    width: 60px;
  }
  
  .sidebar-header {
    padding: 1rem;
    display: flex;
    align-items: center;
    gap: 0.75rem;
    border-bottom: 1px solid rgba(255,255,255,0.1);
  }
  
  .logo { font-size: 1.5rem; }
  .title { font-weight: 600; color: white; }
  
  .sidebar-nav {
    flex: 1;
    padding: 1rem 0;
  }
  
  .nav-item {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem 1rem;
    color: rgba(255,255,255,0.7);
    text-decoration: none;
    transition: all 0.15s ease;
  }
  
  .nav-item:hover {
    background: rgba(255,255,255,0.1);
    color: white;
  }
  
  .nav-item.active {
    background: rgba(59, 130, 246, 0.2);
    color: #3b82f6;
    border-left: 3px solid #3b82f6;
  }
  
  .collapse-btn {
    margin: 1rem;
    padding: 0.5rem;
    background: rgba(255,255,255,0.1);
    border: none;
    color: white;
    cursor: pointer;
    border-radius: 4px;
  }
</style>
```

### AppShell.svelte
```svelte
<script lang="ts">
  import Sidebar from './Sidebar.svelte';
  import { onMount } from 'svelte';
  
  let sidebarCollapsed = false;
  
  onMount(() => {
    sidebarCollapsed = localStorage.getItem('sidebar-collapsed') === 'true';
  });
</script>

<div class="app-shell">
  <Sidebar bind:collapsed={sidebarCollapsed} />
  <main class="main-content">
    <slot />
  </main>
</div>

<style>
  .app-shell {
    display: grid;
    grid-template-columns: auto 1fr;
    min-height: 100vh;
    background: var(--bg-primary, #0f0f1a);
  }
  
  .main-content {
    overflow-y: auto;
    padding: 1.5rem;
  }
</style>
```

## Verification
Run: `ls web/src/lib/components/layout/`
Expected: `AppShell.svelte Sidebar.svelte`
