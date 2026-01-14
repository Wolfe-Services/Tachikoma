<script lang="ts">
  import { onMount, onDestroy, createEventDispatcher } from 'svelte';
  import { fade } from 'svelte/transition';
  import Icon from '$lib/components/common/Icon.svelte';

  export let loading: boolean = false;
  export let lastUpdated: Date | null = null;
  export let autoRefresh: boolean = false;
  export let interval: number = 30000; // 30 seconds default
  export let disabled: boolean = false;
  export let showLastUpdated: boolean = true;
  export let pauseOnHidden: boolean = true;

  const dispatch = createEventDispatcher<{
    refresh: void;
    autoRefreshChange: boolean;
    intervalChange: number;
  }>();

  let intervalId: ReturnType<typeof setInterval> | null = null;
  let showIntervalMenu = false;
  let countdown = 0;
  let isDocumentHidden = false;

  const intervalOptions = [
    { value: 10000, label: '10s' },
    { value: 30000, label: '30s' },
    { value: 60000, label: '1m' },
    { value: 300000, label: '5m' },
    { value: 600000, label: '10m' }
  ];

  $: {
    if (autoRefresh && !loading && !isDocumentHidden) {
      startAutoRefresh();
    } else {
      stopAutoRefresh();
    }
  }

  function startAutoRefresh() {
    stopAutoRefresh();
    countdown = Math.ceil(interval / 1000);

    intervalId = setInterval(() => {
      countdown--;
      if (countdown <= 0) {
        handleRefresh();
        countdown = Math.ceil(interval / 1000);
      }
    }, 1000);
  }

  function stopAutoRefresh() {
    if (intervalId) {
      clearInterval(intervalId);
      intervalId = null;
    }
    countdown = 0;
  }

  function handleRefresh() {
    if (!loading && !disabled) {
      dispatch('refresh');
    }
  }

  function toggleAutoRefresh() {
    autoRefresh = !autoRefresh;
    dispatch('autoRefreshChange', autoRefresh);
  }

  function setIntervalValue(newInterval: number) {
    interval = newInterval;
    dispatch('intervalChange', interval);
    showIntervalMenu = false;
    if (autoRefresh) {
      startAutoRefresh();
    }
  }

  function formatLastUpdated(date: Date): string {
    const now = new Date();
    const diff = now.getTime() - date.getTime();

    if (diff < 60000) return 'Just now';
    if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
    if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`;
    return date.toLocaleDateString();
  }

  function handleVisibilityChange() {
    isDocumentHidden = document.hidden;
    if (pauseOnHidden && document.hidden) {
      stopAutoRefresh();
    } else if (autoRefresh && !document.hidden) {
      startAutoRefresh();
    }
  }

  function handleClickOutside(event: MouseEvent) {
    const target = event.target as HTMLElement;
    if (!target.closest('.interval-dropdown')) {
      showIntervalMenu = false;
    }
  }

  onMount(() => {
    if (typeof document !== 'undefined') {
      document.addEventListener('visibilitychange', handleVisibilityChange);
    }
  });

  onDestroy(() => {
    stopAutoRefresh();
    if (typeof document !== 'undefined') {
      document.removeEventListener('visibilitychange', handleVisibilityChange);
    }
  });
</script>

<svelte:window on:click={handleClickOutside} />

<div class="refresh-control">
  {#if showLastUpdated && lastUpdated}
    <span class="last-updated" title={lastUpdated.toLocaleString()}>
      Updated {formatLastUpdated(lastUpdated)}
    </span>
  {/if}

  <div class="controls">
    <button
      class="refresh-btn"
      on:click={handleRefresh}
      disabled={loading || disabled}
      aria-label="Refresh data"
      title="Refresh data"
    >
      <Icon name="refresh-cw" size={16} class={loading ? 'spinning' : ''} />
    </button>

    <div class="auto-refresh-group">
      <button
        class="auto-refresh-toggle"
        class:active={autoRefresh}
        on:click={toggleAutoRefresh}
        disabled={disabled}
        title={autoRefresh ? 'Disable auto-refresh' : 'Enable auto-refresh'}
      >
        <Icon name="clock" size={14} />
        {#if autoRefresh && countdown > 0}
          <span class="countdown">{countdown}s</span>
        {/if}
      </button>

      <div class="interval-dropdown">
        <button
          class="interval-btn"
          on:click|stopPropagation={() => showIntervalMenu = !showIntervalMenu}
          disabled={disabled}
          title="Set refresh interval"
        >
          <Icon name="chevron-down" size={12} />
        </button>

        {#if showIntervalMenu}
          <div class="interval-menu" transition:fade={{ duration: 100 }}>
            {#each intervalOptions as option}
              <button
                class="interval-option"
                class:selected={interval === option.value}
                on:click={() => setIntervalValue(option.value)}
              >
                {option.label}
                {#if interval === option.value}
                  <Icon name="check" size={12} />
                {/if}
              </button>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  </div>
</div>

<style>
  .refresh-control {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }

  .last-updated {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
  }

  .controls {
    display: flex;
    align-items: center;
  }

  .refresh-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: var(--space-8);
    height: var(--space-8);
    border: 1px solid var(--color-border);
    background: var(--color-bg-surface);
    border-radius: var(--radius-md) 0 0 var(--radius-md);
    color: var(--color-text-secondary);
    cursor: pointer;
    transition: all var(--duration-150) ease;
  }

  .refresh-btn:hover:not(:disabled) {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .refresh-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  :global(.refresh-btn .spinning) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .auto-refresh-group {
    display: flex;
  }

  .auto-refresh-toggle {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    height: var(--space-8);
    padding: 0 var(--space-2);
    border: 1px solid var(--color-border);
    border-left: none;
    background: var(--color-bg-surface);
    color: var(--color-text-muted);
    cursor: pointer;
    transition: all var(--duration-150) ease;
  }

  .auto-refresh-toggle:hover:not(:disabled) {
    background: var(--color-bg-hover);
  }

  .auto-refresh-toggle.active {
    background: var(--color-primary-subtle);
    color: var(--color-primary);
  }

  .countdown {
    font-size: var(--text-xs);
    font-weight: var(--font-semibold);
    font-variant-numeric: tabular-nums;
  }

  .interval-dropdown {
    position: relative;
  }

  .interval-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: var(--space-6);
    height: var(--space-8);
    border: 1px solid var(--color-border);
    border-left: none;
    border-radius: 0 var(--radius-md) var(--radius-md) 0;
    background: var(--color-bg-surface);
    color: var(--color-text-muted);
    cursor: pointer;
    transition: all var(--duration-150) ease;
  }

  .interval-btn:hover:not(:disabled) {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .interval-menu {
    position: absolute;
    top: 100%;
    right: 0;
    margin-top: var(--space-1);
    min-width: 80px;
    background: var(--color-bg-overlay);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-lg);
    z-index: var(--z-dropdown);
    overflow: hidden;
  }

  .interval-option {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: var(--space-2) var(--space-3);
    border: none;
    background: transparent;
    font-size: var(--text-xs);
    color: var(--color-text-primary);
    cursor: pointer;
  }

  .interval-option:hover {
    background: var(--color-bg-hover);
  }

  .interval-option.selected {
    background: var(--color-primary-subtle);
    color: var(--color-primary);
  }
</style>