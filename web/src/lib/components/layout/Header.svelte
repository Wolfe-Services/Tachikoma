<script lang="ts">
  import { missionStore, isRunning } from '$lib/stores/mission';
  import ConnectionStatus from '$lib/components/common/ConnectionStatus.svelte';
  
  let showUserMenu = false;
  let showSettingsMenu = false;

  $: status = $isRunning ? 'Running' : 'Ready';
  $: statusClass = $isRunning ? 'status--running' : 'status--ready';

  function toggleUserMenu() {
    showUserMenu = !showUserMenu;
    showSettingsMenu = false;
  }

  function toggleSettingsMenu() {
    showSettingsMenu = !showSettingsMenu;
    showUserMenu = false;
  }

  function closeMenus() {
    showUserMenu = false;
    showSettingsMenu = false;
  }
</script>

<!-- svelte-ignore a11y-click-events-have-key-events -->
<!-- svelte-ignore a11y-no-static-element-interactions -->
<header class="app-header" on:click={closeMenus}>
  <div class="header-left">
    <h1 class="app-title">Tachikoma</h1>
    <div class="status-indicator">
      <span class="status-dot {statusClass}"></span>
      <span class="status-text">Status: {status}</span>
    </div>
  </div>

  <div class="header-right">
    <ConnectionStatus />
    
    <div class="header-menu">
      <!-- svelte-ignore a11y-click-events-have-key-events -->
      <!-- svelte-ignore a11y-no-static-element-interactions -->
      <button 
        class="menu-button" 
        on:click|stopPropagation={toggleSettingsMenu}
        aria-label="Settings"
      >
        <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 8c-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4-1.79-4-4-4zm8.94 3c-.46-4.17-3.77-7.48-7.94-7.94V1h-2v2.06C6.83 3.52 3.52 6.83 3.06 11H1v2h2.06c.46 4.17 3.77 7.48 7.94 7.94V23h2v-2.06c4.17-.46 7.48-3.77 7.94-7.94H23v-2h-2.06zM12 19c-3.87 0-7-3.13-7-7s3.13-7 7-7 7 3.13 7 7-3.13 7-7 7z"/>
        </svg>
      </button>

      {#if showSettingsMenu}
        <div class="dropdown-menu settings-menu">
          <a href="/settings" class="menu-item">Preferences</a>
          <a href="/settings/backends" class="menu-item">Backends</a>
          <a href="/settings/security" class="menu-item">Security</a>
          <div class="menu-divider"></div>
          <button class="menu-item menu-item--action">Export Data</button>
          <button class="menu-item menu-item--action">Import Settings</button>
        </div>
      {/if}
    </div>

    <div class="header-menu">
      <!-- svelte-ignore a11y-click-events-have-key-events -->
      <!-- svelte-ignore a11y-no-static-element-interactions -->
      <button 
        class="menu-button user-button" 
        on:click|stopPropagation={toggleUserMenu}
        aria-label="User menu"
      >
        <div class="user-avatar">T</div>
      </button>

      {#if showUserMenu}
        <div class="dropdown-menu user-menu">
          <div class="user-info">
            <div class="user-name">Tachikoma User</div>
            <div class="user-role">Operator</div>
          </div>
          <div class="menu-divider"></div>
          <a href="/profile" class="menu-item">Profile</a>
          <a href="/activity" class="menu-item">Activity Log</a>
          <div class="menu-divider"></div>
          <button class="menu-item menu-item--danger">Sign Out</button>
        </div>
      {/if}
    </div>
  </div>
</header>

<style>
  .app-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    height: 60px;
    padding: 0 1.5rem;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 2rem;
  }

  .app-title {
    font-size: 1.5rem;
    font-weight: 600;
    color: var(--text);
    margin: 0;
  }

  .status-indicator {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--text-muted);
  }

  .status-dot.status--ready {
    background: #22c55e;
  }

  .status-dot.status--running {
    background: var(--accent);
    animation: pulse 2s infinite;
  }

  @keyframes pulse {
    0% { opacity: 1; }
    50% { opacity: 0.5; }
    100% { opacity: 1; }
  }

  .status-text {
    font-size: 0.875rem;
    color: var(--text-muted);
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .header-menu {
    position: relative;
  }

  .menu-button {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 40px;
    height: 40px;
    border: none;
    background: transparent;
    color: var(--text-muted);
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .menu-button:hover {
    background: var(--bg);
    color: var(--text);
  }

  .user-button {
    width: auto;
    padding: 0.25rem;
  }

  .user-avatar {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    background: var(--accent);
    color: white;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: 600;
    font-size: 0.875rem;
  }

  .dropdown-menu {
    position: absolute;
    top: 100%;
    right: 0;
    margin-top: 0.5rem;
    min-width: 200px;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 8px;
    box-shadow: 0 10px 25px rgba(0, 0, 0, 0.5);
    z-index: 50;
    padding: 0.5rem 0;
  }

  .user-info {
    padding: 0.75rem 1rem;
    border-bottom: 1px solid var(--border);
    margin-bottom: 0.5rem;
  }

  .user-name {
    font-weight: 600;
    color: var(--text);
    margin-bottom: 0.25rem;
  }

  .user-role {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .menu-item {
    display: block;
    width: 100%;
    padding: 0.5rem 1rem;
    border: none;
    background: transparent;
    color: var(--text);
    text-decoration: none;
    text-align: left;
    font-size: 0.875rem;
    cursor: pointer;
    transition: background-color 0.2s ease;
  }

  .menu-item:hover {
    background: var(--bg);
  }

  .menu-item--action {
    color: var(--text-muted);
  }

  .menu-item--danger {
    color: #ef4444;
  }

  .menu-item--danger:hover {
    background: rgba(239, 68, 68, 0.1);
  }

  .menu-divider {
    height: 1px;
    background: var(--border);
    margin: 0.5rem 0;
  }

  @media (max-width: 768px) {
    .app-header {
      padding: 0 1rem;
    }

    .header-left {
      gap: 1rem;
    }

    .app-title {
      font-size: 1.25rem;
    }

    .status-indicator {
      display: none;
    }

    .dropdown-menu {
      right: -1rem;
      left: 1rem;
      min-width: auto;
    }
  }
</style>