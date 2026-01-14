<script lang="ts">
  import { fly, fade } from 'svelte/transition';
  import { connectionStatus, wsStore } from '$lib/stores/websocket';
  import Icon from '$lib/components/common/Icon.svelte';

  export let showDetails: boolean = false;
  export let position: 'inline' | 'fixed' = 'inline';

  $: status = $connectionStatus;
  $: statusConfig = getStatusConfig(status);

  function getStatusConfig(s: typeof status) {
    switch (s) {
      case 'connected':
        return { icon: 'wifi', color: 'var(--green-500)', label: 'Connected' };
      case 'connecting':
        return { icon: 'loader', color: 'var(--blue-500)', label: 'Connecting...' };
      case 'reconnecting':
        return { icon: 'refresh-cw', color: 'var(--yellow-500)', label: 'Reconnecting...' };
      case 'disconnected':
        return { icon: 'wifi-off', color: 'var(--red-500)', label: 'Disconnected' };
      default:
        return { icon: 'help-circle', color: 'var(--gray-500)', label: 'Unknown' };
    }
  }

  function handleReconnect() {
    // Trigger reconnection - use localhost by default or get from env
    const wsUrl = import.meta.env.VITE_WS_URL || 'ws://localhost:8080/ws';
    wsStore.connect(wsUrl);
  }
</script>

<div
  class="connection-status {position}"
  class:disconnected={status === 'disconnected'}
  title={statusConfig.label}
>
  <span class="status-indicator" style="color: {statusConfig.color}">
    <Icon
      name={statusConfig.icon}
      size={14}
      class={status === 'connecting' || status === 'reconnecting' ? 'spinning' : ''}
    />
  </span>

  {#if showDetails}
    <span class="status-label">{statusConfig.label}</span>
  {/if}

  {#if status === 'disconnected'}
    <button class="reconnect-btn" on:click={handleReconnect}>
      Reconnect
    </button>
  {/if}
</div>

{#if status === 'disconnected' && position === 'fixed'}
  <div class="offline-banner" transition:fly={{ y: -20, duration: 200 }}>
    <Icon name="wifi-off" size={16} />
    <span>You're offline. Some features may be unavailable.</span>
    <button on:click={handleReconnect}>
      Try Again
    </button>
  </div>
{/if}

<style>
  .connection-status {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .connection-status.fixed {
    position: fixed;
    bottom: 1rem;
    right: 1rem;
    padding: 0.5rem 0.75rem;
    background: var(--bg-card);
    border: 1px solid var(--border-color);
    border-radius: 0.5rem;
    box-shadow: var(--shadow-md);
    z-index: 1000;
  }

  .status-indicator {
    display: flex;
    align-items: center;
  }

  :global(.status-indicator .spinning) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .status-label {
    font-size: 0.75rem;
    color: var(--text-secondary);
  }

  .reconnect-btn {
    padding: 0.25rem 0.5rem;
    border: 1px solid var(--border-color);
    background: transparent;
    border-radius: 0.25rem;
    font-size: 0.6875rem;
    color: var(--text-primary);
    cursor: pointer;
  }

  .reconnect-btn:hover {
    background: var(--bg-hover);
  }

  .offline-banner {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.75rem;
    padding: 0.75rem;
    background: var(--red-500);
    color: white;
    font-size: 0.875rem;
    z-index: 1001;
  }

  .offline-banner button {
    padding: 0.25rem 0.75rem;
    border: 1px solid rgba(255, 255, 255, 0.3);
    background: transparent;
    border-radius: 0.375rem;
    font-size: 0.75rem;
    color: white;
    cursor: pointer;
  }

  .offline-banner button:hover {
    background: rgba(255, 255, 255, 0.1);
  }
</style>