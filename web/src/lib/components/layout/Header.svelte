<script lang="ts">
  import { onMount } from 'svelte';
  import { ipc } from '$lib/ipc';
  import Icon from '../common/Icon.svelte';
  
  let appStatus: 'online' | 'offline' | 'syncing' = 'online';
  let userMenuOpen = false;
  let notifications = 0;
  
  onMount(async () => {
    try {
      await ipc.invoke('config:get', {});
      appStatus = 'online';
    } catch (e) {
      console.log('Failed to get status:', e);
      appStatus = 'offline';
    }
  });
  
  function toggleUserMenu() {
    userMenuOpen = !userMenuOpen;
  }
  
  function closeUserMenu() {
    userMenuOpen = false;
  }
  
  function handleSettings() {
    closeUserMenu();
    window.location.href = '/settings';
  }
  
  function handleHelp() {
    closeUserMenu();
  }
  
  function handleClickOutside(event: MouseEvent) {
    const target = event.target as Element;
    if (userMenuOpen && !target.closest('.user-menu')) {
      closeUserMenu();
    }
  }
  
  const statusLabels = {
    online: 'CONNECTED',
    offline: 'OFFLINE',
    syncing: 'SYNCING...'
  };
</script>

<svelte:window on:click={handleClickOutside} />

<header class="app-header">
  <!-- Left: Logo + Title -->
  <div class="header-brand">
    <span class="brand-text">TACHIKOMA</span>
    <span class="brand-divider">//</span>
    <span class="brand-sub">公安9課</span>
  </div>
  
  <!-- Center: Minimal Status -->
  <div class="header-center">
    <div class="status-pill" class:online={appStatus === 'online'} class:offline={appStatus === 'offline'}>
      <span class="status-dot"></span>
      <span class="status-text">{appStatus === 'online' ? 'ONLINE' : 'OFFLINE'}</span>
    </div>
  </div>
  
  <!-- Right: Actions -->
  <div class="header-actions">
    {#if notifications > 0}
      <button class="action-btn notification-btn" title="Notifications">
        <Icon name="bell" size={18} />
        <span class="notification-badge">{notifications}</span>
      </button>
    {/if}
    
    <button class="action-btn" title="Terminal">
      <Icon name="terminal" size={18} />
    </button>
    
    <div class="user-menu">
      <button class="action-btn user-btn" on:click={toggleUserMenu} title="User menu">
        <Icon name="user" size={18} />
        <Icon name="chevron-down" size={12} />
      </button>
      
      {#if userMenuOpen}
        <div class="user-menu__dropdown">
          <div class="dropdown-header">
            <span class="dropdown-title">OPERATOR</span>
            <span class="dropdown-subtitle">Section 9</span>
          </div>
          <div class="dropdown-divider"></div>
          <button class="menu-item" on:click={handleSettings}>
            <Icon name="settings" size={16} />
            <span>Settings</span>
          </button>
          <button class="menu-item" on:click={handleHelp}>
            <Icon name="help-circle" size={16} />
            <span>Documentation</span>
          </button>
          <div class="dropdown-divider"></div>
          <div class="menu-info">
            <div class="info-row">
              <span class="info-label">VERSION</span>
              <span class="info-value">1.0.0</span>
            </div>
            <div class="info-row">
              <span class="info-label">BUILD</span>
              <span class="info-value">2024.1</span>
            </div>
          </div>
        </div>
      {/if}
    </div>
  </div>
  
</header>

<style>
  .app-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 48px;
    padding: 0 1.5rem;
    background: var(--bg-secondary, #161b22);
    border-bottom: 1px solid var(--border-color, rgba(78, 205, 196, 0.15));
    position: relative;
    z-index: 100;
  }
  
  .header-brand {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  
  .brand-text {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.85rem;
    font-weight: 700;
    color: var(--tachi-cyan, #4ecdc4);
    letter-spacing: 2px;
  }
  
  .brand-divider {
    color: var(--text-muted, rgba(230, 237, 243, 0.3));
    font-size: 0.75rem;
  }
  
  .brand-sub {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.65rem;
    font-weight: 500;
    color: var(--text-muted, rgba(230, 237, 243, 0.4));
    letter-spacing: 1.5px;
  }
  
  .header-center {
    position: absolute;
    left: 50%;
    transform: translateX(-50%);
  }
  
  .status-pill {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.25rem 0.75rem;
    background: rgba(63, 185, 80, 0.1);
    border: 1px solid rgba(63, 185, 80, 0.3);
    border-radius: 12px;
  }
  
  .status-pill.offline {
    background: rgba(255, 107, 107, 0.1);
    border-color: rgba(255, 107, 107, 0.3);
  }
  
  .status-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--success-color, #3fb950);
    box-shadow: 0 0 6px var(--success-color, #3fb950);
    animation: pulse 2s ease-in-out infinite;
  }
  
  .status-pill.offline .status-dot {
    background: var(--error-color, #ff6b6b);
    box-shadow: 0 0 6px var(--error-color, #ff6b6b);
  }
  
  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }
  
  .status-text {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.6rem;
    font-weight: 600;
    color: var(--success-color, #3fb950);
    letter-spacing: 1px;
  }
  
  .status-pill.offline .status-text {
    color: var(--error-color, #ff6b6b);
  }
  
  .header-actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  
  .action-btn {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.5rem 0.75rem;
    border: 1px solid transparent;
    background: transparent;
    color: var(--text-secondary, rgba(230, 237, 243, 0.7));
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.2s ease;
  }
  
  .action-btn:hover {
    background: var(--hover-bg, rgba(78, 205, 196, 0.08));
    color: var(--tachi-cyan, #4ecdc4);
    border-color: var(--border-color, rgba(78, 205, 196, 0.2));
  }
  
  .notification-btn {
    position: relative;
  }
  
  .notification-badge {
    position: absolute;
    top: 0;
    right: 0;
    background: var(--tachi-red, #ff6b6b);
    color: white;
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.6rem;
    font-weight: 600;
    padding: 0.125rem 0.375rem;
    border-radius: 8px;
    min-width: 16px;
    text-align: center;
  }
  
  .user-menu {
    position: relative;
  }
  
  .user-btn {
    gap: 0.25rem;
  }
  
  .user-menu__dropdown {
    position: absolute;
    top: calc(100% + 8px);
    right: 0;
    width: 220px;
    background: var(--bg-secondary, #161b22);
    border: 1px solid var(--border-color, rgba(78, 205, 196, 0.2));
    border-radius: 8px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4), 0 0 20px var(--tachi-cyan-glow, rgba(78, 205, 196, 0.1));
    overflow: hidden;
    z-index: 1000;
  }
  
  .dropdown-header {
    padding: 1rem;
    background: linear-gradient(135deg, rgba(78, 205, 196, 0.1), transparent);
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
  
  .dropdown-title {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--tachi-cyan, #4ecdc4);
    letter-spacing: 1px;
  }
  
  .dropdown-subtitle {
    font-size: 0.7rem;
    color: var(--text-muted, rgba(230, 237, 243, 0.4));
  }
  
  .dropdown-divider {
    height: 1px;
    background: var(--border-color, rgba(78, 205, 196, 0.15));
  }
  
  .menu-item {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    width: 100%;
    padding: 0.75rem 1rem;
    border: none;
    background: transparent;
    color: var(--text-primary, #e6edf3);
    font-family: var(--font-body, 'Rajdhani', sans-serif);
    font-size: 0.9rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
    text-align: left;
  }
  
  .menu-item:hover {
    background: var(--hover-bg, rgba(78, 205, 196, 0.08));
    color: var(--tachi-cyan, #4ecdc4);
  }
  
  .menu-info {
    padding: 0.75rem 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  
  .info-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  
  .info-label {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.6rem;
    color: var(--text-muted, rgba(230, 237, 243, 0.4));
    letter-spacing: 1px;
  }
  
  .info-value {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.7rem;
    color: var(--text-secondary, rgba(230, 237, 243, 0.7));
  }
  
  /* Mobile responsive */
  @media (max-width: 768px) {
    .app-header {
      height: 48px;
      padding: 0 0.75rem;
    }
    
    .header-decor {
      display: none;
    }
    
    .status-label {
      display: none;
    }
  }
</style>
