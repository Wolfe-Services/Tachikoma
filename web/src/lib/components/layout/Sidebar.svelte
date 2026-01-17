<script lang="ts">
  import { page } from '$app/stores';
  import { onMount } from 'svelte';
  import Icon from '../common/Icon.svelte';
  import TachikomaLogo from '../common/TachikomaLogo.svelte';
  
  // Navigation items - reflects actual system functionality
  const navItems = [
    { href: '/', label: 'Command', icon: 'home', description: 'System Dashboard', jp: '司令部' },
    { href: '/missions', label: 'Deploy', icon: 'play', description: 'Loop Runner', jp: '展開' },
    { href: '/specs', label: 'Registry', icon: 'file-text', description: 'Spec Browser', jp: '仕様書' },
    { href: '/forge', label: 'Think Tank', icon: 'brain', description: 'Multi-Model AI', jp: '思考戦車' },
    { href: '/settings', label: 'Config', icon: 'settings', description: 'API & Settings', jp: '設定' }
  ];
  
  export let collapsed = false;
  
  onMount(() => {
    const stored = localStorage.getItem('sidebar-collapsed');
    if (stored !== null) {
      collapsed = stored === 'true';
    }
  });
  
  function toggleCollapse() {
    collapsed = !collapsed;
    localStorage.setItem('sidebar-collapsed', String(collapsed));
  }
  
  function isActiveRoute(href: string, pathname: string): boolean {
    if (href === '/') {
      return pathname === '/';
    }
    return pathname.startsWith(href);
  }
</script>

<aside class="sidebar" class:collapsed>
  <!-- Logo Section -->
  <div class="sidebar-header">
    <div class="logo-container">
      <TachikomaLogo size={collapsed ? 32 : 40} animated={true} />
    </div>
    {#if !collapsed}
      <div class="brand">
        <span class="brand-name">TACHIKOMA</span>
        <span class="brand-tagline">SECTION 9 // AI DIVISION</span>
      </div>
    {/if}
  </div>
  
  <!-- Status Indicator -->
  {#if !collapsed}
    <div class="status-section">
      <div class="status-badge online">
        <span class="status-dot"></span>
        <span class="status-text">SYSTEM ONLINE</span>
        <span class="status-jp">稼働中</span>
      </div>
    </div>
  {/if}
  
  <!-- Navigation -->
  <nav class="sidebar-nav">
    {#each navItems as item}
      <a 
        href={item.href}
        class="nav-item"
        class:active={isActiveRoute(item.href, $page.url.pathname)}
        title={collapsed ? item.label : undefined}
      >
        <div class="nav-icon">
          <Icon name={item.icon} size={20} glow={isActiveRoute(item.href, $page.url.pathname)} />
        </div>
        {#if !collapsed}
          <div class="nav-content">
            <span class="nav-label">{item.label} <span class="nav-jp">{item.jp}</span></span>
            <span class="nav-description">{item.description}</span>
          </div>
        {/if}
        {#if isActiveRoute(item.href, $page.url.pathname)}
          <div class="active-indicator"></div>
        {/if}
      </a>
    {/each}
  </nav>
  
  <!-- Footer -->
  <div class="sidebar-footer">
    <button class="collapse-btn" on:click={toggleCollapse} title={collapsed ? 'Expand' : 'Collapse'}>
      <Icon name={collapsed ? 'chevron-right' : 'chevron-left'} size={16} />
    </button>
    {#if !collapsed}
      <div class="version-info">
        <span class="version">v1.0.0</span>
        <span class="build">公安9課</span>
      </div>
    {/if}
  </div>
</aside>

<style>
  .sidebar {
    width: 260px;
    height: 100vh;
    background: var(--sidebar-bg, linear-gradient(180deg, #0d1117 0%, #0a0c10 100%));
    border-right: 1px solid var(--border-color, rgba(78, 205, 196, 0.15));
    display: flex;
    flex-direction: column;
    transition: width 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    position: relative;
    overflow: hidden;
  }
  
  /* Ghost in the Shell manga background */
  .sidebar::after {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-image: url('/gints.jpeg');
    background-size: 180% auto;
    background-position: center center;
    background-repeat: no-repeat;
    opacity: 0.12;
    pointer-events: none;
    z-index: 0;
    filter: grayscale(30%) contrast(1.1);
  }
  
  /* Subtle gradient overlay on top of image */
  .sidebar::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: linear-gradient(
      180deg, 
      rgba(13, 17, 23, 0.85) 0%, 
      rgba(13, 17, 23, 0.6) 30%,
      rgba(13, 17, 23, 0.4) 60%,
      rgba(13, 17, 23, 0.7) 100%
    );
    pointer-events: none;
    z-index: 1;
  }
  
  .sidebar.collapsed {
    width: 72px;
  }
  
  .sidebar-header {
    padding: 1.25rem;
    display: flex;
    align-items: center;
    gap: 1rem;
    border-bottom: 1px solid var(--border-color, rgba(78, 205, 196, 0.15));
    min-height: 80px;
    position: relative;
    z-index: 2;
    background: rgba(13, 17, 23, 0.6);
    backdrop-filter: blur(4px);
  }
  
  .logo-container {
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }
  
  .brand {
    display: flex;
    flex-direction: column;
    gap: 0;
    overflow: hidden;
  }
  
  .brand-name {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 1.1rem;
    font-weight: 700;
    color: var(--tachi-cyan, #4ecdc4);
    letter-spacing: 2px;
    text-shadow: 0 0 10px var(--tachi-cyan-glow, rgba(78, 205, 196, 0.4));
  }
  
  .brand-tagline {
    font-family: var(--font-body, 'Rajdhani', sans-serif);
    font-size: 0.7rem;
    font-weight: 500;
    color: var(--text-muted, rgba(230, 237, 243, 0.4));
    letter-spacing: 3px;
    text-transform: uppercase;
  }
  
  .status-section {
    padding: 0.75rem 1.25rem;
    border-bottom: 1px solid var(--border-color, rgba(78, 205, 196, 0.1));
    position: relative;
    z-index: 2;
    background: rgba(13, 17, 23, 0.4);
  }
  
  .status-badge {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.4rem 0.75rem;
    background: rgba(63, 185, 80, 0.1);
    border: 1px solid rgba(63, 185, 80, 0.2);
    border-radius: 4px;
  }
  
  .status-badge.online .status-dot {
    background: var(--success-color, #3fb950);
    box-shadow: 0 0 8px rgba(63, 185, 80, 0.6);
  }
  
  .status-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    animation: pulse 2s ease-in-out infinite;
  }
  
  .status-text {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.65rem;
    font-weight: 500;
    color: var(--success-color, #3fb950);
    letter-spacing: 1px;
  }

  .status-jp {
    font-family: 'Noto Sans JP', sans-serif;
    font-size: 0.6rem;
    color: var(--success-color, #3fb950);
    opacity: 0.7;
    margin-left: auto;
  }
  
  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }
  
  .sidebar-nav {
    flex: 1;
    padding: 1rem 0.75rem;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    position: relative;
    z-index: 2;
  }
  
  .nav-item {
    display: flex;
    align-items: center;
    gap: 0.875rem;
    padding: 0.875rem 1rem;
    color: var(--text-secondary, rgba(230, 237, 243, 0.7));
    text-decoration: none;
    transition: all 0.2s ease;
    position: relative;
    border-radius: 8px;
    overflow: hidden;
  }
  
  .nav-item::before {
    content: '';
    position: absolute;
    inset: 0;
    background: linear-gradient(90deg, var(--tachi-cyan, #4ecdc4), transparent);
    opacity: 0;
    transition: opacity 0.2s ease;
  }
  
  .nav-item:hover {
    background: var(--hover-bg, rgba(78, 205, 196, 0.15));
    color: var(--text-primary, #e6edf3);
    backdrop-filter: blur(4px);
  }
  
  .nav-item:hover::before {
    opacity: 0.05;
  }
  
  .nav-item.active {
    background: var(--active-bg, rgba(78, 205, 196, 0.2));
    color: var(--tachi-cyan, #4ecdc4);
    backdrop-filter: blur(6px);
  }
  
  .nav-item.active::before {
    opacity: 0.1;
  }
  
  .active-indicator {
    position: absolute;
    left: 0;
    top: 50%;
    transform: translateY(-50%);
    width: 3px;
    height: 60%;
    background: var(--tachi-cyan, #4ecdc4);
    border-radius: 0 2px 2px 0;
    box-shadow: 0 0 10px var(--tachi-cyan-glow, rgba(78, 205, 196, 0.4));
  }
  
  .nav-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    min-width: 24px;
    position: relative;
    z-index: 1;
  }
  
  .nav-content {
    display: flex;
    flex-direction: column;
    gap: 0;
    overflow: hidden;
    position: relative;
    z-index: 1;
  }
  
  .nav-label {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.8rem;
    font-weight: 500;
    letter-spacing: 0.5px;
    white-space: nowrap;
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .nav-jp {
    font-family: 'Noto Sans JP', sans-serif;
    font-size: 0.65rem;
    color: var(--tachi-cyan, #4ecdc4);
    opacity: 0.6;
    letter-spacing: 0;
  }
  
  .nav-description {
    font-size: 0.7rem;
    color: var(--text-muted, rgba(230, 237, 243, 0.4));
    white-space: nowrap;
  }
  
  .sidebar-footer {
    padding: 1rem;
    border-top: 1px solid var(--border-color, rgba(78, 205, 196, 0.15));
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
    position: relative;
    z-index: 2;
    background: rgba(13, 17, 23, 0.6);
    backdrop-filter: blur(4px);
  }
  
  .collapse-btn {
    padding: 0.625rem;
    background: var(--button-bg, rgba(78, 205, 196, 0.1));
    border: 1px solid var(--border-color, rgba(78, 205, 196, 0.2));
    color: var(--text-secondary, rgba(230, 237, 243, 0.7));
    cursor: pointer;
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s ease;
  }
  
  .collapse-btn:hover {
    background: var(--hover-bg, rgba(78, 205, 196, 0.15));
    color: var(--tachi-cyan, #4ecdc4);
    border-color: var(--tachi-cyan, #4ecdc4);
    box-shadow: 0 0 10px var(--tachi-cyan-glow, rgba(78, 205, 196, 0.3));
  }
  
  .version-info {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 0;
  }
  
  .version {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.65rem;
    color: var(--text-muted, rgba(230, 237, 243, 0.4));
  }
  
  .build {
    font-size: 0.6rem;
    color: var(--tachi-cyan, #4ecdc4);
    opacity: 0.6;
    letter-spacing: 1px;
  }
  
  /* Collapsed state adjustments */
  .collapsed .sidebar-header {
    justify-content: center;
    padding: 1rem;
  }
  
  .collapsed .sidebar-nav {
    padding: 1rem 0.5rem;
  }
  
  .collapsed .nav-item {
    justify-content: center;
    padding: 0.875rem;
  }
  
  .collapsed .sidebar-footer {
    justify-content: center;
  }
</style>
