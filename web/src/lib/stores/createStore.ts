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