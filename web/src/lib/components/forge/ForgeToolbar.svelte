<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import Icon from '$lib/components/common/Icon.svelte';
  import { activeSession } from '$lib/stores/forgeSession';
  import type { ForgeLayoutConfig } from '$lib/types/forge';

  export let sessionId: string | null = null;
  export let layoutConfig: ForgeLayoutConfig;

  const dispatch = createEventDispatcher<{
    togglePanel: 'left' | 'right' | 'bottom';
    newSession: void;
  }>();

  function togglePanel(panel: 'left' | 'right' | 'bottom') {
    dispatch('togglePanel', panel);
  }

  function requestNewSession() {
    dispatch('newSession');
  }

  function handleKeydown(event: KeyboardEvent) {
    // Keyboard shortcuts are handled by parent component
    // This is just for accessibility on button focus
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      const target = event.target as HTMLButtonElement;
      target.click();
    }
  }
</script>

<header class="forge-toolbar" role="banner">
  <div class="toolbar-section left">
    <h1 class="forge-title">
      Think Tank
      {#if sessionId}
        <span class="session-indicator">{$activeSession?.name ?? 'Session Active'}</span>
      {/if}
    </h1>
  </div>

  <div class="toolbar-section center">
    <nav class="panel-controls" role="navigation" aria-label="Panel controls">
      <button
        type="button"
        class="panel-toggle"
        class:active={layoutConfig.leftSidebarVisible}
        on:click={() => togglePanel('left')}
        on:keydown={handleKeydown}
        title="Toggle left sidebar (Ctrl+B)"
        aria-label="Toggle left sidebar"
        aria-pressed={layoutConfig.leftSidebarVisible}
      >
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
          <path d="M2 2h4v12H2V2zm6 0h6v12H8V2z" />
        </svg>
        Sidebar
      </button>

      <button
        type="button"
        class="panel-toggle"
        class:active={layoutConfig.rightPanelVisible}
        on:click={() => togglePanel('right')}
        on:keydown={handleKeydown}
        title="Toggle right panel (Ctrl+\)"
        aria-label="Toggle right panel"
        aria-pressed={layoutConfig.rightPanelVisible}
      >
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
          <path d="M2 2h6v12H2V2zm8 0h4v12h-4V2z" />
        </svg>
        Results
      </button>

      <button
        type="button"
        class="panel-toggle"
        class:active={layoutConfig.bottomPanelVisible}
        on:click={() => togglePanel('bottom')}
        on:keydown={handleKeydown}
        title="Toggle bottom panel (Ctrl+J)"
        aria-label="Toggle bottom panel"
        aria-pressed={layoutConfig.bottomPanelVisible}
      >
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
          <path d="M2 2h12v6H2V2zm0 8h12v4H2v-4z" />
        </svg>
        Logs
      </button>
    </nav>
  </div>

  <div class="toolbar-section right">
    <div class="session-status">
      {#if sessionId}
        <span class="status-dot active" aria-label="Session active"></span>
        <span class="status-text">Active</span>
      {:else}
        <span class="status-dot inactive" aria-label="No active session"></span>
        <span class="status-text">Idle</span>
      {/if}
    </div>

    <button type="button" class="new-session" on:click={requestNewSession} title="Start a new Think Tank session">
      <Icon name="zap" size={16} />
      <span>New</span>
    </button>
  </div>
</header>

<style>
  .forge-toolbar {
    display: flex;
    align-items: center;
    height: 48px;
    padding: 0 16px;
    background: rgba(13, 17, 23, 0.7);
    border-bottom: 1px solid rgba(78, 205, 196, 0.14);
    color: var(--text-primary, #e6edf3);
    -webkit-backdrop-filter: blur(14px) saturate(1.25);
    backdrop-filter: blur(14px) saturate(1.25);
    user-select: none;
  }

  .toolbar-section {
    display: flex;
    align-items: center;
  }

  .toolbar-section.left {
    flex: 0 0 auto;
  }

  .toolbar-section.center {
    flex: 1 1 auto;
    justify-content: center;
  }

  .toolbar-section.right {
    flex: 0 0 auto;
    gap: 0.75rem;
  }

  .forge-title {
    font-size: 18px;
    font-weight: 600;
    margin: 0;
    color: var(--tachi-cyan, #4ecdc4);
    font-family: var(--font-display, 'Orbitron', sans-serif);
    letter-spacing: 1.5px;
    text-transform: uppercase;
  }

  .session-indicator {
    font-size: 12px;
    font-weight: 500;
    color: rgba(230, 237, 243, 0.65);
    margin-left: 8px;
    font-family: var(--font-body, 'Rajdhani', sans-serif);
    letter-spacing: 0.2px;
    text-transform: none;
  }

  .panel-controls {
    display: flex;
    gap: 4px;
    padding: 0 16px;
  }

  .panel-toggle {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    background: transparent;
    border: 1px solid transparent;
    border-radius: 4px;
    color: var(--forge-text, #eaeaea);
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .panel-toggle:hover {
    background: var(--hover-bg, rgba(78, 205, 196, 0.08));
    border-color: rgba(78, 205, 196, 0.18);
  }

  .panel-toggle:focus {
    outline: 2px solid var(--tachi-cyan, #4ecdc4);
    outline-offset: 2px;
  }

  .panel-toggle.active {
    background: rgba(78, 205, 196, 0.15);
    color: var(--tachi-cyan, #4ecdc4);
    border-color: rgba(78, 205, 196, 0.55);
  }

  .panel-toggle svg {
    flex-shrink: 0;
  }

  .session-status {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    font-weight: 500;
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    display: block;
  }

  .status-dot.active {
    background: var(--success-color, #3fb950);
    box-shadow: 0 0 6px rgba(63, 185, 80, 0.6);
  }

  .status-dot.inactive {
    background: var(--muted-color, #6b7280);
  }

  .status-text {
    color: var(--muted-color, #6b7280);
  }

  .new-session {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.4rem 0.65rem;
    border-radius: 10px;
    background: rgba(78, 205, 196, 0.12);
    border: 1px solid rgba(78, 205, 196, 0.22);
    color: var(--tachi-cyan, #4ecdc4);
    cursor: pointer;
    transition: all 0.2s ease;
    font-family: var(--font-display, 'Orbitron', sans-serif);
    letter-spacing: 0.8px;
    text-transform: uppercase;
    font-size: 0.7rem;
  }

  .new-session:hover {
    border-color: rgba(78, 205, 196, 0.5);
    background: rgba(78, 205, 196, 0.16);
    box-shadow: 0 0 18px rgba(78, 205, 196, 0.22);
  }

  @media (max-width: 768px) {
    .forge-toolbar {
      padding: 0 12px;
    }

    .panel-controls {
      padding: 0 8px;
      gap: 2px;
    }

    .panel-toggle {
      padding: 6px 8px;
      font-size: 11px;
    }

    .panel-toggle span:not(.sr-only) {
      display: none;
    }

    .forge-title {
      font-size: 16px;
    }

    .session-indicator {
      display: none;
    }

    .new-session span {
      display: none;
    }
  }
</style>