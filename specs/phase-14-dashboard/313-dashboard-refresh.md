# 313 - Dashboard Refresh

**Phase:** 14 - Dashboard
**Spec ID:** 313
**Status:** Planned
**Dependencies:** 296-dashboard-layout
**Estimated Context:** ~6% of Sonnet window

---

## Objective

Create dashboard refresh functionality that allows manual and automatic data refresh, with configurable intervals and visual feedback on refresh status.

---

## Acceptance Criteria

- [ ] `RefreshControl.svelte` component created
- [ ] Manual refresh button
- [ ] Auto-refresh toggle with intervals
- [ ] Last updated timestamp display
- [ ] Loading state indicator
- [ ] Refresh error handling
- [ ] Configurable refresh intervals
- [ ] Pause refresh on tab inactive

---

## Implementation Details

### 1. Refresh Control Component (web/src/lib/components/common/RefreshControl.svelte)

```svelte
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

  function setInterval(newInterval: number) {
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
                on:click={() => setInterval(option.value)}
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
    gap: 0.75rem;
  }

  .last-updated {
    font-size: 0.75rem;
    color: var(--text-tertiary);
  }

  .controls {
    display: flex;
    align-items: center;
  }

  .refresh-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 2rem;
    height: 2rem;
    border: 1px solid var(--border-color);
    background: var(--bg-primary);
    border-radius: 0.375rem 0 0 0.375rem;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .refresh-btn:hover:not(:disabled) {
    background: var(--bg-hover);
    color: var(--text-primary);
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
    gap: 0.25rem;
    height: 2rem;
    padding: 0 0.5rem;
    border: 1px solid var(--border-color);
    border-left: none;
    background: var(--bg-primary);
    color: var(--text-tertiary);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .auto-refresh-toggle:hover:not(:disabled) {
    background: var(--bg-hover);
  }

  .auto-refresh-toggle.active {
    background: var(--accent-color-light, rgba(59, 130, 246, 0.1));
    color: var(--accent-color);
  }

  .countdown {
    font-size: 0.625rem;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }

  .interval-dropdown {
    position: relative;
  }

  .interval-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 1.5rem;
    height: 2rem;
    border: 1px solid var(--border-color);
    border-left: none;
    border-radius: 0 0.375rem 0.375rem 0;
    background: var(--bg-primary);
    color: var(--text-tertiary);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .interval-btn:hover:not(:disabled) {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .interval-menu {
    position: absolute;
    top: 100%;
    right: 0;
    margin-top: 0.25rem;
    min-width: 80px;
    background: var(--bg-card);
    border: 1px solid var(--border-color);
    border-radius: 0.375rem;
    box-shadow: var(--shadow-lg);
    z-index: 100;
    overflow: hidden;
  }

  .interval-option {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 0.5rem 0.75rem;
    border: none;
    background: transparent;
    font-size: 0.75rem;
    color: var(--text-primary);
    cursor: pointer;
  }

  .interval-option:hover {
    background: var(--bg-hover);
  }

  .interval-option.selected {
    background: var(--accent-color-light, rgba(59, 130, 246, 0.1));
    color: var(--accent-color);
  }
</style>
```

### 2. Refresh Store (web/src/lib/stores/refresh.ts)

```typescript
import { writable, derived } from 'svelte/store';

interface RefreshState {
  loading: boolean;
  lastUpdated: Date | null;
  autoRefresh: boolean;
  interval: number;
  error: string | null;
}

function createRefreshStore() {
  const { subscribe, set, update } = writable<RefreshState>({
    loading: false,
    lastUpdated: null,
    autoRefresh: false,
    interval: 30000,
    error: null
  });

  return {
    subscribe,
    setLoading: (loading: boolean) =>
      update(s => ({ ...s, loading })),
    markUpdated: () =>
      update(s => ({ ...s, lastUpdated: new Date(), error: null })),
    setAutoRefresh: (enabled: boolean) =>
      update(s => ({ ...s, autoRefresh: enabled })),
    setInterval: (interval: number) =>
      update(s => ({ ...s, interval })),
    setError: (error: string | null) =>
      update(s => ({ ...s, error, loading: false })),
    reset: () => set({
      loading: false,
      lastUpdated: null,
      autoRefresh: false,
      interval: 30000,
      error: null
    })
  };
}

export const refreshStore = createRefreshStore();

export const isRefreshing = derived(
  refreshStore,
  $store => $store.loading
);
```

---

## Testing Requirements

1. Manual refresh triggers callback
2. Auto-refresh starts/stops correctly
3. Interval selection works
4. Countdown displays accurately
5. Tab visibility pauses refresh
6. Loading state shows spinner
7. Last updated formats correctly

---

## Related Specs

- Depends on: [296-dashboard-layout.md](296-dashboard-layout.md)
- Next: [314-realtime-updates.md](314-realtime-updates.md)
- Used by: All dashboard components
