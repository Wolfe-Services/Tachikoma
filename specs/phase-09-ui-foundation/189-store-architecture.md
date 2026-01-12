# Spec 189: Svelte Store Architecture

## Phase
Phase 9: UI Foundation

## Spec ID
189

## Status
Planned

## Dependencies
- Spec 186: SvelteKit Setup

## Estimated Context
~10%

---

## Objective

Establish a scalable and type-safe Svelte store architecture for Tachikoma's state management, including patterns for global state, derived state, async state handling, and persistence.

---

## Acceptance Criteria

- [ ] Base store factory with TypeScript generics
- [ ] Async store pattern for API/IPC data
- [ ] Derived store utilities
- [ ] Store persistence layer
- [ ] Store devtools integration
- [ ] Store composition patterns
- [ ] Memory leak prevention
- [ ] Store testing utilities

---

## Implementation Details

### src/lib/stores/createStore.ts

```typescript
import { writable, derived, get, type Writable, type Readable } from 'svelte/store';

/**
 * Enhanced store factory with common utilities
 */
export interface EnhancedStore<T> extends Writable<T> {
  reset: () => void;
  get: () => T;
}

export function createStore<T>(initialValue: T): EnhancedStore<T> {
  const { subscribe, set, update } = writable<T>(initialValue);

  return {
    subscribe,
    set,
    update,
    reset: () => set(initialValue),
    get: () => get({ subscribe })
  };
}

/**
 * Create a store with history for undo/redo
 */
export interface HistoryStore<T> extends EnhancedStore<T> {
  undo: () => void;
  redo: () => void;
  canUndo: Readable<boolean>;
  canRedo: Readable<boolean>;
  clearHistory: () => void;
}

export function createHistoryStore<T>(
  initialValue: T,
  maxHistory: number = 50
): HistoryStore<T> {
  const store = createStore<T>(initialValue);
  const past = writable<T[]>([]);
  const future = writable<T[]>([]);

  const canUndo = derived(past, $past => $past.length > 0);
  const canRedo = derived(future, $future => $future.length > 0);

  function pushHistory(value: T) {
    past.update(p => {
      const newPast = [...p, value];
      if (newPast.length > maxHistory) {
        return newPast.slice(-maxHistory);
      }
      return newPast;
    });
    future.set([]);
  }

  return {
    ...store,
    set: (value: T) => {
      pushHistory(store.get());
      store.set(value);
    },
    update: (fn: (value: T) => T) => {
      pushHistory(store.get());
      store.update(fn);
    },
    undo: () => {
      const $past = get(past);
      if ($past.length > 0) {
        const previous = $past[$past.length - 1];
        past.set($past.slice(0, -1));
        future.update(f => [store.get(), ...f]);
        store.set(previous);
      }
    },
    redo: () => {
      const $future = get(future);
      if ($future.length > 0) {
        const next = $future[0];
        future.set($future.slice(1));
        past.update(p => [...p, store.get()]);
        store.set(next);
      }
    },
    canUndo,
    canRedo,
    clearHistory: () => {
      past.set([]);
      future.set([]);
    }
  };
}
```

### src/lib/stores/asyncStore.ts

```typescript
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
```

### src/lib/stores/persistedStore.ts

```typescript
import { writable, type Writable } from 'svelte/store';
import { browser } from '$app/environment';

export interface PersistedStoreOptions<T> {
  key: string;
  storage?: 'localStorage' | 'sessionStorage';
  serialize?: (value: T) => string;
  deserialize?: (value: string) => T;
  validate?: (value: unknown) => value is T;
  migrate?: (oldValue: unknown, version: number) => T;
  version?: number;
}

export function createPersistedStore<T>(
  initialValue: T,
  options: PersistedStoreOptions<T>
): Writable<T> {
  const {
    key,
    storage = 'localStorage',
    serialize = JSON.stringify,
    deserialize = JSON.parse,
    validate,
    migrate,
    version = 1
  } = options;

  const storageKey = `tachikoma:${key}`;
  const versionKey = `${storageKey}:version`;

  function getStoredValue(): T {
    if (!browser) return initialValue;

    try {
      const storageApi = storage === 'localStorage' ? localStorage : sessionStorage;
      const storedValue = storageApi.getItem(storageKey);
      const storedVersion = parseInt(storageApi.getItem(versionKey) || '0', 10);

      if (storedValue === null) {
        return initialValue;
      }

      let parsed = deserialize(storedValue);

      // Migration
      if (migrate && storedVersion < version) {
        parsed = migrate(parsed, storedVersion);
        storageApi.setItem(versionKey, version.toString());
        storageApi.setItem(storageKey, serialize(parsed));
      }

      // Validation
      if (validate && !validate(parsed)) {
        console.warn(`Invalid stored value for ${key}, using initial value`);
        return initialValue;
      }

      return parsed;
    } catch (error) {
      console.error(`Error reading persisted store ${key}:`, error);
      return initialValue;
    }
  }

  const store = writable<T>(getStoredValue());

  if (browser) {
    store.subscribe(value => {
      try {
        const storageApi = storage === 'localStorage' ? localStorage : sessionStorage;
        storageApi.setItem(storageKey, serialize(value));
        storageApi.setItem(versionKey, version.toString());
      } catch (error) {
        console.error(`Error persisting store ${key}:`, error);
      }
    });

    // Listen for storage changes from other tabs
    window.addEventListener('storage', (event) => {
      if (event.key === storageKey && event.newValue !== null) {
        try {
          const newValue = deserialize(event.newValue);
          if (!validate || validate(newValue)) {
            store.set(newValue);
          }
        } catch {
          // Ignore invalid values
        }
      }
    });
  }

  return store;
}
```

### src/lib/stores/index.ts

```typescript
// Re-export all store utilities
export * from './createStore';
export * from './asyncStore';
export * from './persistedStore';

// Re-export application stores
export * from './auth';
export * from './layout';
export * from './navigation';
export * from './projects';
export * from './settings';
export * from './theme';
```

### src/lib/stores/auth.ts

```typescript
import { derived } from 'svelte/store';
import { createPersistedStore } from './persistedStore';

export interface User {
  id: string;
  email: string;
  name: string;
  avatar?: string;
  roles: string[];
}

export interface AuthState {
  user: User | null;
  isAuthenticated: boolean;
  token: string | null;
  refreshToken: string | null;
  expiresAt: number | null;
}

const initialAuthState: AuthState = {
  user: null,
  isAuthenticated: false,
  token: null,
  refreshToken: null,
  expiresAt: null
};

export const authStore = createPersistedStore<AuthState>(initialAuthState, {
  key: 'auth',
  storage: 'localStorage',
  version: 1,
  validate: (value): value is AuthState => {
    return value !== null && typeof value === 'object' && 'isAuthenticated' in value;
  }
});

// Derived stores
export const currentUser = derived(authStore, $auth => $auth.user);
export const isAuthenticated = derived(authStore, $auth => $auth.isAuthenticated);
export const userRoles = derived(authStore, $auth => $auth.user?.roles ?? []);

// Auth actions
export const authActions = {
  login: (user: User, token: string, refreshToken: string, expiresIn: number) => {
    authStore.set({
      user,
      isAuthenticated: true,
      token,
      refreshToken,
      expiresAt: Date.now() + expiresIn * 1000
    });
  },

  logout: () => {
    authStore.set(initialAuthState);
  },

  updateUser: (updates: Partial<User>) => {
    authStore.update(state => ({
      ...state,
      user: state.user ? { ...state.user, ...updates } : null
    }));
  },

  refreshTokens: (token: string, refreshToken: string, expiresIn: number) => {
    authStore.update(state => ({
      ...state,
      token,
      refreshToken,
      expiresAt: Date.now() + expiresIn * 1000
    }));
  }
};
```

### src/lib/stores/projects.ts

```typescript
import { derived } from 'svelte/store';
import { createAsyncStore, createPaginatedStore } from './asyncStore';
import { createStore } from './createStore';
import { invoke } from '@ipc/invoke';

export interface Project {
  id: string;
  name: string;
  description: string;
  status: 'active' | 'completed' | 'archived';
  createdAt: string;
  updatedAt: string;
  targetCount: number;
  scanCount: number;
}

export interface ProjectFilters {
  status: Project['status'] | 'all';
  search: string;
  sortBy: 'name' | 'createdAt' | 'updatedAt';
  sortOrder: 'asc' | 'desc';
}

// Filters store
export const projectFilters = createStore<ProjectFilters>({
  status: 'all',
  search: '',
  sortBy: 'updatedAt',
  sortOrder: 'desc'
});

// Selected project ID
export const selectedProjectId = createStore<string | null>(null);

// Projects list (paginated)
export const projectsStore = createPaginatedStore<Project>(
  async (page, pageSize) => {
    const filters = projectFilters.get();
    const response = await invoke<{ items: Project[]; total: number }>('get_projects', {
      page,
      pageSize,
      filters
    });
    return response;
  },
  20
);

// Current project (async)
export const currentProject = createAsyncStore<Project | null>(
  async () => {
    const id = selectedProjectId.get();
    if (!id) return null;
    return invoke<Project>('get_project', { id });
  },
  { initialData: null }
);

// Derived stores
export const filteredProjects = derived(
  projectsStore,
  $projects => $projects.items
);

export const projectsLoading = derived(
  projectsStore,
  $projects => $projects.loading
);

// Project actions
export const projectActions = {
  create: async (data: Omit<Project, 'id' | 'createdAt' | 'updatedAt' | 'targetCount' | 'scanCount'>) => {
    const project = await invoke<Project>('create_project', { data });
    await projectsStore.refresh();
    return project;
  },

  update: async (id: string, data: Partial<Project>) => {
    const project = await invoke<Project>('update_project', { id, data });
    await projectsStore.refresh();
    if (selectedProjectId.get() === id) {
      currentProject.mutate(project);
    }
    return project;
  },

  delete: async (id: string) => {
    await invoke('delete_project', { id });
    await projectsStore.refresh();
    if (selectedProjectId.get() === id) {
      selectedProjectId.set(null);
      currentProject.reset();
    }
  },

  select: (id: string) => {
    selectedProjectId.set(id);
    currentProject.refetch();
  }
};
```

---

## Testing Requirements

### Unit Tests

```typescript
// tests/stores/createStore.test.ts
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import { createStore, createHistoryStore } from '@stores/createStore';

describe('createStore', () => {
  it('should create a store with initial value', () => {
    const store = createStore({ count: 0 });
    expect(get(store)).toEqual({ count: 0 });
  });

  it('should reset to initial value', () => {
    const store = createStore({ count: 0 });
    store.set({ count: 5 });
    store.reset();
    expect(get(store)).toEqual({ count: 0 });
  });

  it('should get current value', () => {
    const store = createStore({ count: 10 });
    expect(store.get()).toEqual({ count: 10 });
  });
});

describe('createHistoryStore', () => {
  it('should track history on set', () => {
    const store = createHistoryStore(0);
    store.set(1);
    store.set(2);

    expect(get(store)).toBe(2);
    expect(get(store.canUndo)).toBe(true);
  });

  it('should undo changes', () => {
    const store = createHistoryStore(0);
    store.set(1);
    store.set(2);
    store.undo();

    expect(get(store)).toBe(1);
    expect(get(store.canRedo)).toBe(true);
  });

  it('should redo changes', () => {
    const store = createHistoryStore(0);
    store.set(1);
    store.undo();
    store.redo();

    expect(get(store)).toBe(1);
  });
});
```

### Integration Tests

```typescript
// tests/stores/asyncStore.test.ts
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import { createAsyncStore } from '@stores/asyncStore';

describe('createAsyncStore', () => {
  it('should fetch data', async () => {
    const fetcher = vi.fn().mockResolvedValue({ id: 1, name: 'Test' });
    const store = createAsyncStore(fetcher);

    await store.fetch();

    expect(get(store).data).toEqual({ id: 1, name: 'Test' });
    expect(get(store).loading).toBe(false);
  });

  it('should handle errors', async () => {
    const error = new Error('Fetch failed');
    const fetcher = vi.fn().mockRejectedValue(error);
    const store = createAsyncStore(fetcher);

    await expect(store.fetch()).rejects.toThrow('Fetch failed');
    expect(get(store).error).toBe(error);
  });

  it('should use cached data within stale time', async () => {
    const fetcher = vi.fn().mockResolvedValue({ id: 1 });
    const store = createAsyncStore(fetcher, { staleTime: 60000 });

    await store.fetch();
    await store.fetch();

    expect(fetcher).toHaveBeenCalledTimes(1);
  });
});
```

---

## Related Specs

- [186-sveltekit-setup.md](./186-sveltekit-setup.md) - SvelteKit setup
- [190-ipc-store-bindings.md](./190-ipc-store-bindings.md) - IPC bindings
- [188-layout-system.md](./188-layout-system.md) - Layout system
