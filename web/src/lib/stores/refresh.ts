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