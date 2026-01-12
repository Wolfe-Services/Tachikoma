import { writable, derived, get, type Readable } from 'svelte/store';

export interface AsyncState<T, E = Error> {
  data: T | null;
  loading: boolean;
  error: E | null;
  lastFetched: number | null;
}

export interface AsyncStore<T, E = Error> extends Readable<AsyncState<T, E>> {
  fetch: () => Promise<T>;
  refetch: () => Promise<T>;
  mutate: (data: T | ((prev: T | null) => T)) => void;
  reset: () => void;
  invalidate: () => void;
}

export interface AsyncStoreOptions<T> {
  initialData?: T | null;
  cacheTime?: number; // milliseconds
  staleTime?: number; // milliseconds
  onError?: (error: Error) => void;
  onSuccess?: (data: T) => void;
}

export function createAsyncStore<T, E = Error>(
  fetcher: () => Promise<T>,
  options: AsyncStoreOptions<T> = {}
): AsyncStore<T, E> {
  const {
    initialData = null,
    cacheTime = 5 * 60 * 1000, // 5 minutes
    staleTime = 0,
    onError,
    onSuccess
  } = options;

  const state = writable<AsyncState<T, E>>({
    data: initialData,
    loading: false,
    error: null,
    lastFetched: null
  });

  let abortController: AbortController | null = null;

  async function fetch(): Promise<T> {
    const currentState = get(state);

    // Check if data is still fresh
    if (
      currentState.data !== null &&
      currentState.lastFetched !== null &&
      Date.now() - currentState.lastFetched < staleTime
    ) {
      return currentState.data;
    }

    // Cancel any in-flight request
    if (abortController) {
      abortController.abort();
    }
    abortController = new AbortController();

    state.update(s => ({ ...s, loading: true, error: null }));

    try {
      const data = await fetcher();

      state.set({
        data,
        loading: false,
        error: null,
        lastFetched: Date.now()
      });

      onSuccess?.(data);
      return data;
    } catch (error) {
      if ((error as Error).name === 'AbortError') {
        throw error;
      }

      state.update(s => ({
        ...s,
        loading: false,
        error: error as E
      }));

      onError?.(error as Error);
      throw error;
    }
  }

  async function refetch(): Promise<T> {
    state.update(s => ({ ...s, lastFetched: null }));
    return fetch();
  }

  function mutate(data: T | ((prev: T | null) => T)) {
    state.update(s => ({
      ...s,
      data: typeof data === 'function' ? (data as Function)(s.data) : data,
      lastFetched: Date.now()
    }));
  }

  function reset() {
    if (abortController) {
      abortController.abort();
    }
    state.set({
      data: initialData,
      loading: false,
      error: null,
      lastFetched: null
    });
  }

  function invalidate() {
    state.update(s => ({ ...s, lastFetched: null }));
  }

  return {
    subscribe: state.subscribe,
    fetch,
    refetch,
    mutate,
    reset,
    invalidate
  };
}

/**
 * Create a paginated async store
 */
export interface PaginatedState<T> {
  items: T[];
  page: number;
  pageSize: number;
  totalItems: number;
  totalPages: number;
  loading: boolean;
  error: Error | null;
  hasMore: boolean;
}

export interface PaginatedStore<T> extends Readable<PaginatedState<T>> {
  loadPage: (page: number) => Promise<void>;
  loadMore: () => Promise<void>;
  refresh: () => Promise<void>;
  reset: () => void;
}

export function createPaginatedStore<T>(
  fetcher: (page: number, pageSize: number) => Promise<{ items: T[]; total: number }>,
  pageSize: number = 20
): PaginatedStore<T> {
  const state = writable<PaginatedState<T>>({
    items: [],
    page: 0,
    pageSize,
    totalItems: 0,
    totalPages: 0,
    loading: false,
    error: null,
    hasMore: true
  });

  async function loadPage(page: number) {
    state.update(s => ({ ...s, loading: true, error: null }));

    try {
      const { items, total } = await fetcher(page, pageSize);

      state.update(s => ({
        ...s,
        items: page === 0 ? items : [...s.items, ...items],
        page,
        totalItems: total,
        totalPages: Math.ceil(total / pageSize),
        loading: false,
        hasMore: items.length === pageSize && (page + 1) * pageSize < total
      }));
    } catch (error) {
      state.update(s => ({
        ...s,
        loading: false,
        error: error as Error
      }));
    }
  }

  async function loadMore() {
    const currentState = get(state);
    if (!currentState.loading && currentState.hasMore) {
      await loadPage(currentState.page + 1);
    }
  }

  async function refresh() {
    state.update(s => ({ ...s, items: [], page: 0 }));
    await loadPage(0);
  }

  function reset() {
    state.set({
      items: [],
      page: 0,
      pageSize,
      totalItems: 0,
      totalPages: 0,
      loading: false,
      error: null,
      hasMore: true
    });
  }

  return {
    subscribe: state.subscribe,
    loadPage,
    loadMore,
    refresh,
    reset
  };
}