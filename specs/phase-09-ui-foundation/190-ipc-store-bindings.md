# Spec 190: IPC Store Bindings

## Phase
Phase 9: UI Foundation

## Spec ID
190

## Status
Planned

## Dependencies
- Spec 186: SvelteKit Setup
- Spec 189: Store Architecture
- Phase 1-8 Tauri IPC infrastructure

## Estimated Context
~12%

---

## Objective

Create a robust binding layer between Svelte stores and Tauri IPC commands, enabling seamless communication between the frontend UI and Rust backend with proper error handling, type safety, and reactive updates.

---

## Acceptance Criteria

- [x] Type-safe IPC invoke wrapper
- [x] Event subscription management
- [x] Automatic retry logic for failed calls
- [x] Request/response caching
- [x] Optimistic updates support
- [x] Error transformation and handling
- [x] Loading state management
- [x] Real-time event-to-store bindings

---

## Implementation Details

### src/lib/ipc/invoke.ts

```typescript
import { isTauri } from '@utils/environment';

export interface InvokeOptions {
  timeout?: number;
  retries?: number;
  retryDelay?: number;
  cache?: boolean;
  cacheTime?: number;
}

export interface InvokeError {
  code: string;
  message: string;
  details?: Record<string, unknown>;
}

const DEFAULT_OPTIONS: InvokeOptions = {
  timeout: 30000,
  retries: 0,
  retryDelay: 1000,
  cache: false,
  cacheTime: 60000
};

// Simple in-memory cache
const cache = new Map<string, { data: unknown; timestamp: number }>();

function getCacheKey(command: string, args?: Record<string, unknown>): string {
  return `${command}:${JSON.stringify(args || {})}`;
}

export async function invoke<T>(
  command: string,
  args?: Record<string, unknown>,
  options: InvokeOptions = {}
): Promise<T> {
  const opts = { ...DEFAULT_OPTIONS, ...options };

  // Check cache first
  if (opts.cache) {
    const cacheKey = getCacheKey(command, args);
    const cached = cache.get(cacheKey);

    if (cached && Date.now() - cached.timestamp < opts.cacheTime!) {
      return cached.data as T;
    }
  }

  // Execute with retries
  let lastError: Error | null = null;

  for (let attempt = 0; attempt <= opts.retries!; attempt++) {
    try {
      const result = await executeInvoke<T>(command, args, opts.timeout!);

      // Cache successful result
      if (opts.cache) {
        cache.set(getCacheKey(command, args), {
          data: result,
          timestamp: Date.now()
        });
      }

      return result;
    } catch (error) {
      lastError = error as Error;

      // Don't retry on certain errors
      if (isNonRetryableError(error)) {
        throw transformError(error);
      }

      // Wait before retry
      if (attempt < opts.retries!) {
        await new Promise(resolve => setTimeout(resolve, opts.retryDelay!));
      }
    }
  }

  throw transformError(lastError);
}

async function executeInvoke<T>(
  command: string,
  args: Record<string, unknown> | undefined,
  timeout: number
): Promise<T> {
  if (!isTauri()) {
    throw new Error('Tauri is not available');
  }

  const { invoke: tauriInvoke } = await import('@tauri-apps/api');

  // Create timeout promise
  const timeoutPromise = new Promise<never>((_, reject) => {
    setTimeout(() => {
      reject(new Error(`IPC timeout after ${timeout}ms`));
    }, timeout);
  });

  // Race between invoke and timeout
  return Promise.race([
    tauriInvoke<T>(command, args),
    timeoutPromise
  ]);
}

function isNonRetryableError(error: unknown): boolean {
  if (error instanceof Error) {
    const nonRetryable = [
      'NotFound',
      'Unauthorized',
      'Forbidden',
      'ValidationError'
    ];
    return nonRetryable.some(code => error.message.includes(code));
  }
  return false;
}

function transformError(error: unknown): InvokeError {
  if (error instanceof Error) {
    // Parse Tauri error format
    const match = error.message.match(/^(\w+):\s*(.+)$/);
    if (match) {
      return {
        code: match[1],
        message: match[2]
      };
    }
    return {
      code: 'UnknownError',
      message: error.message
    };
  }
  return {
    code: 'UnknownError',
    message: String(error)
  };
}

// Invalidate cache for specific command
export function invalidateCache(command?: string): void {
  if (command) {
    for (const key of cache.keys()) {
      if (key.startsWith(`${command}:`)) {
        cache.delete(key);
      }
    }
  } else {
    cache.clear();
  }
}
```

### src/lib/ipc/events.ts

```typescript
import { writable, type Readable, type Writable } from 'svelte/store';
import { onDestroy, onMount } from 'svelte';
import { isTauri } from '@utils/environment';

export type EventCallback<T> = (payload: T) => void;
export type UnsubscribeFn = () => void;

interface EventSubscription {
  event: string;
  unsubscribe: UnsubscribeFn;
}

// Active subscriptions for cleanup
const activeSubscriptions = new Map<string, EventSubscription[]>();

/**
 * Subscribe to a Tauri event
 */
export async function subscribe<T>(
  event: string,
  callback: EventCallback<T>
): Promise<UnsubscribeFn> {
  if (!isTauri()) {
    console.warn('Event subscription not available outside Tauri');
    return () => {};
  }

  const { listen } = await import('@tauri-apps/api/event');

  const unlisten = await listen<T>(event, (e) => {
    callback(e.payload);
  });

  return unlisten;
}

/**
 * Emit a Tauri event
 */
export async function emit<T>(event: string, payload?: T): Promise<void> {
  if (!isTauri()) {
    console.warn('Event emission not available outside Tauri');
    return;
  }

  const { emit: tauriEmit } = await import('@tauri-apps/api/event');
  await tauriEmit(event, payload);
}

/**
 * Create a store that syncs with Tauri events
 */
export function createEventStore<T>(
  eventName: string,
  initialValue: T
): Readable<T> {
  const store = writable<T>(initialValue);

  if (isTauri()) {
    // Setup subscription when in Tauri context
    (async () => {
      const { listen } = await import('@tauri-apps/api/event');

      await listen<T>(eventName, (event) => {
        store.set(event.payload);
      });
    })();
  }

  return {
    subscribe: store.subscribe
  };
}

/**
 * Create a bidirectional sync between a store and Tauri events
 */
export function createSyncedStore<T>(options: {
  eventName: string;
  commandName: string;
  initialValue: T;
  transform?: (payload: unknown) => T;
}): Writable<T> & { refresh: () => Promise<void> } {
  const { eventName, commandName, initialValue, transform = (p) => p as T } = options;
  const store = writable<T>(initialValue);

  async function refresh() {
    if (!isTauri()) return;

    try {
      const { invoke } = await import('@tauri-apps/api');
      const data = await invoke<unknown>(commandName);
      store.set(transform(data));
    } catch (error) {
      console.error(`Failed to refresh ${commandName}:`, error);
    }
  }

  if (isTauri()) {
    // Initial fetch
    refresh();

    // Listen for updates
    (async () => {
      const { listen } = await import('@tauri-apps/api/event');

      await listen<unknown>(eventName, (event) => {
        store.set(transform(event.payload));
      });
    })();
  }

  return {
    subscribe: store.subscribe,
    set: (value: T) => {
      store.set(value);
      // Optionally emit event to backend
    },
    update: (fn: (value: T) => T) => {
      store.update(fn);
    },
    refresh
  };
}

/**
 * Hook for managing event subscriptions in Svelte components
 */
export function useEventSubscription<T>(
  event: string,
  callback: EventCallback<T>
): void {
  let unsubscribe: UnsubscribeFn | null = null;

  onMount(async () => {
    unsubscribe = await subscribe(event, callback);
  });

  onDestroy(() => {
    if (unsubscribe) {
      unsubscribe();
    }
  });
}
```

### src/lib/ipc/mutations.ts

```typescript
import { writable, get, type Writable } from 'svelte/store';
import { invoke } from './invoke';

export interface MutationState<TData, TError = Error> {
  data: TData | null;
  error: TError | null;
  isLoading: boolean;
  isSuccess: boolean;
  isError: boolean;
}

export interface MutationOptions<TData, TVariables, TError = Error> {
  onSuccess?: (data: TData, variables: TVariables) => void | Promise<void>;
  onError?: (error: TError, variables: TVariables) => void;
  onSettled?: (data: TData | null, error: TError | null, variables: TVariables) => void;
  optimisticUpdate?: <TState>(state: TState, variables: TVariables) => TState;
  rollback?: <TState>(state: TState, variables: TVariables) => TState;
}

export interface Mutation<TData, TVariables, TError = Error> {
  state: Writable<MutationState<TData, TError>>;
  mutate: (variables: TVariables) => Promise<TData>;
  mutateAsync: (variables: TVariables) => Promise<TData>;
  reset: () => void;
}

export function createMutation<TData, TVariables, TError = Error>(
  command: string,
  options: MutationOptions<TData, TVariables, TError> = {}
): Mutation<TData, TVariables, TError> {
  const state = writable<MutationState<TData, TError>>({
    data: null,
    error: null,
    isLoading: false,
    isSuccess: false,
    isError: false
  });

  async function mutateAsync(variables: TVariables): Promise<TData> {
    state.set({
      data: null,
      error: null,
      isLoading: true,
      isSuccess: false,
      isError: false
    });

    try {
      const data = await invoke<TData>(command, variables as Record<string, unknown>);

      state.set({
        data,
        error: null,
        isLoading: false,
        isSuccess: true,
        isError: false
      });

      await options.onSuccess?.(data, variables);
      options.onSettled?.(data, null, variables);

      return data;
    } catch (error) {
      state.set({
        data: null,
        error: error as TError,
        isLoading: false,
        isSuccess: false,
        isError: true
      });

      options.onError?.(error as TError, variables);
      options.onSettled?.(null, error as TError, variables);

      throw error;
    }
  }

  function mutate(variables: TVariables): Promise<TData> {
    return mutateAsync(variables).catch(() => {
      // Swallow error for fire-and-forget usage
      return get(state).data as TData;
    });
  }

  function reset() {
    state.set({
      data: null,
      error: null,
      isLoading: false,
      isSuccess: false,
      isError: false
    });
  }

  return {
    state,
    mutate,
    mutateAsync,
    reset
  };
}

/**
 * Create a mutation with optimistic updates
 */
export function createOptimisticMutation<TData, TVariables, TContext, TError = Error>(
  command: string,
  options: MutationOptions<TData, TVariables, TError> & {
    getOptimisticData: (variables: TVariables) => TData;
    targetStore: Writable<TContext>;
    updateStore: (store: TContext, data: TData) => TContext;
    rollbackStore: (store: TContext, context: TContext) => TContext;
  }
): Mutation<TData, TVariables, TError> {
  const { targetStore, updateStore, rollbackStore, getOptimisticData, ...mutationOptions } = options;

  const mutation = createMutation<TData, TVariables, TError>(command, {
    ...mutationOptions,
    onSuccess: async (data, variables) => {
      // Update with real data
      targetStore.update(store => updateStore(store, data));
      await mutationOptions.onSuccess?.(data, variables);
    },
    onError: (error, variables) => {
      // Rollback handled separately
      mutationOptions.onError?.(error, variables);
    }
  });

  const originalMutateAsync = mutation.mutateAsync;

  mutation.mutateAsync = async (variables: TVariables): Promise<TData> => {
    // Store current state for rollback
    const previousState = get(targetStore);

    // Apply optimistic update
    const optimisticData = getOptimisticData(variables);
    targetStore.update(store => updateStore(store, optimisticData));

    try {
      return await originalMutateAsync(variables);
    } catch (error) {
      // Rollback on error
      targetStore.update(store => rollbackStore(store, previousState));
      throw error;
    }
  };

  return mutation;
}
```

### src/lib/ipc/projects.ts

```typescript
import { invoke } from './invoke';
import { createMutation } from './mutations';
import { createEventStore, subscribe } from './events';
import type { Project } from '@stores/projects';

// Queries
export async function getProjects(params: {
  page: number;
  pageSize: number;
  filters?: Record<string, unknown>;
}): Promise<{ items: Project[]; total: number }> {
  return invoke('get_projects', params, { cache: true, cacheTime: 30000 });
}

export async function getProject(id: string): Promise<Project | null> {
  return invoke('get_project', { id }, { cache: true, cacheTime: 60000 });
}

// Mutations
export const createProjectMutation = createMutation<
  Project,
  { name: string; description: string }
>('create_project', {
  onSuccess: (project) => {
    console.log('Project created:', project.id);
  }
});

export const updateProjectMutation = createMutation<
  Project,
  { id: string; data: Partial<Project> }
>('update_project');

export const deleteProjectMutation = createMutation<
  void,
  { id: string }
>('delete_project');

// Event stores
export const projectUpdates = createEventStore<{ projectId: string; action: string }>(
  'project:updated',
  { projectId: '', action: '' }
);

// Subscribe to project events
export function subscribeToProjectEvents(
  projectId: string,
  callbacks: {
    onUpdate?: (project: Project) => void;
    onDelete?: () => void;
    onScanComplete?: (scanId: string) => void;
  }
): () => void {
  const unsubscribers: (() => void)[] = [];

  subscribe<Project>(`project:${projectId}:updated`, (project) => {
    callbacks.onUpdate?.(project);
  }).then(unsub => unsubscribers.push(unsub));

  subscribe<void>(`project:${projectId}:deleted`, () => {
    callbacks.onDelete?.();
  }).then(unsub => unsubscribers.push(unsub));

  subscribe<{ scanId: string }>(`project:${projectId}:scan_complete`, (data) => {
    callbacks.onScanComplete?.(data.scanId);
  }).then(unsub => unsubscribers.push(unsub));

  return () => {
    unsubscribers.forEach(unsub => unsub());
  };
}
```

### src/lib/ipc/ai.ts

```typescript
import { writable, type Readable } from 'svelte/store';
import { invoke } from './invoke';
import { subscribe, emit } from './events';

export interface ChatMessage {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: number;
  metadata?: Record<string, unknown>;
}

export interface ChatSession {
  id: string;
  messages: ChatMessage[];
  isStreaming: boolean;
  currentResponse: string;
}

export function createChatSession(): {
  session: Readable<ChatSession>;
  sendMessage: (content: string) => Promise<void>;
  cancel: () => void;
  clear: () => void;
} {
  const session = writable<ChatSession>({
    id: crypto.randomUUID(),
    messages: [],
    isStreaming: false,
    currentResponse: ''
  });

  let currentStreamUnsubscribe: (() => void) | null = null;

  async function sendMessage(content: string) {
    const userMessage: ChatMessage = {
      id: crypto.randomUUID(),
      role: 'user',
      content,
      timestamp: Date.now()
    };

    session.update(s => ({
      ...s,
      messages: [...s.messages, userMessage],
      isStreaming: true,
      currentResponse: ''
    }));

    try {
      // Subscribe to streaming response
      currentStreamUnsubscribe = await subscribe<{ chunk: string; done: boolean }>(
        'ai:stream',
        (data) => {
          if (data.done) {
            session.update(s => {
              const assistantMessage: ChatMessage = {
                id: crypto.randomUUID(),
                role: 'assistant',
                content: s.currentResponse,
                timestamp: Date.now()
              };
              return {
                ...s,
                messages: [...s.messages, assistantMessage],
                isStreaming: false,
                currentResponse: ''
              };
            });
          } else {
            session.update(s => ({
              ...s,
              currentResponse: s.currentResponse + data.chunk
            }));
          }
        }
      );

      // Start the chat request
      await invoke('ai_chat', {
        sessionId: crypto.randomUUID(),
        message: content
      });
    } catch (error) {
      session.update(s => ({
        ...s,
        isStreaming: false
      }));
      throw error;
    }
  }

  function cancel() {
    if (currentStreamUnsubscribe) {
      currentStreamUnsubscribe();
      currentStreamUnsubscribe = null;
    }
    emit('ai:cancel');
    session.update(s => ({
      ...s,
      isStreaming: false
    }));
  }

  function clear() {
    cancel();
    session.update(s => ({
      ...s,
      messages: [],
      currentResponse: ''
    }));
  }

  return {
    session: { subscribe: session.subscribe },
    sendMessage,
    cancel,
    clear
  };
}
```

---

## Testing Requirements

### Unit Tests

```typescript
// tests/ipc/invoke.test.ts
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke, invalidateCache } from '@ipc/invoke';

// Mock Tauri
vi.mock('@tauri-apps/api', () => ({
  invoke: vi.fn()
}));

vi.mock('@utils/environment', () => ({
  isTauri: () => true
}));

describe('invoke', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    invalidateCache();
  });

  it('should invoke Tauri command', async () => {
    const { invoke: tauriInvoke } = await import('@tauri-apps/api');
    (tauriInvoke as any).mockResolvedValue({ id: 1 });

    const result = await invoke('test_command', { arg: 'value' });

    expect(tauriInvoke).toHaveBeenCalledWith('test_command', { arg: 'value' });
    expect(result).toEqual({ id: 1 });
  });

  it('should retry on failure', async () => {
    const { invoke: tauriInvoke } = await import('@tauri-apps/api');
    (tauriInvoke as any)
      .mockRejectedValueOnce(new Error('Network error'))
      .mockResolvedValueOnce({ id: 1 });

    const result = await invoke('test_command', {}, { retries: 1, retryDelay: 10 });

    expect(tauriInvoke).toHaveBeenCalledTimes(2);
    expect(result).toEqual({ id: 1 });
  });

  it('should use cache', async () => {
    const { invoke: tauriInvoke } = await import('@tauri-apps/api');
    (tauriInvoke as any).mockResolvedValue({ id: 1 });

    await invoke('test_command', {}, { cache: true });
    await invoke('test_command', {}, { cache: true });

    expect(tauriInvoke).toHaveBeenCalledTimes(1);
  });
});
```

### Integration Tests

```typescript
// tests/ipc/events.test.ts
import { describe, it, expect, vi } from 'vitest';
import { get } from 'svelte/store';
import { createEventStore, createSyncedStore } from '@ipc/events';

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn((event, callback) => {
    // Simulate event
    setTimeout(() => callback({ payload: { value: 'updated' } }), 10);
    return Promise.resolve(() => {});
  }),
  emit: vi.fn()
}));

describe('Event Stores', () => {
  it('should create event store with initial value', () => {
    const store = createEventStore('test:event', { value: 'initial' });
    expect(get(store)).toEqual({ value: 'initial' });
  });

  it('should update on event', async () => {
    const store = createEventStore('test:event', { value: 'initial' });

    // Wait for event simulation
    await new Promise(resolve => setTimeout(resolve, 20));

    expect(get(store)).toEqual({ value: 'updated' });
  });
});
```

---

## Related Specs

- [186-sveltekit-setup.md](./186-sveltekit-setup.md) - SvelteKit setup
- [189-store-architecture.md](./189-store-architecture.md) - Store architecture
- Phase 1-8 Tauri backend specs
