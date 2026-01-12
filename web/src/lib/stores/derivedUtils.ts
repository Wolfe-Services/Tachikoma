import { derived, type Readable } from 'svelte/store';

/**
 * Create a derived store that debounces updates
 */
export function createDebouncedStore<T>(
  store: Readable<T>,
  delay: number = 300
): Readable<T> {
  return derived(
    store,
    (value, set) => {
      const timeout = setTimeout(() => set(value), delay);
      return () => clearTimeout(timeout);
    }
  );
}

/**
 * Create a derived store that throttles updates
 */
export function createThrottledStore<T>(
  store: Readable<T>,
  delay: number = 300
): Readable<T> {
  let lastUpdate = 0;
  
  return derived(
    store,
    (value, set) => {
      const now = Date.now();
      if (now - lastUpdate >= delay) {
        lastUpdate = now;
        set(value);
      } else {
        const timeout = setTimeout(() => {
          lastUpdate = Date.now();
          set(value);
        }, delay - (now - lastUpdate));
        return () => clearTimeout(timeout);
      }
    }
  );
}

/**
 * Create a derived store with async computation
 */
export function createAsyncDerived<T, U>(
  store: Readable<T>,
  fn: (value: T) => Promise<U>,
  initialValue: U
): Readable<{ value: U; loading: boolean; error: Error | null }> {
  return derived(
    store,
    (value, set) => {
      set({ value: initialValue, loading: true, error: null });
      
      fn(value)
        .then(result => {
          set({ value: result, loading: false, error: null });
        })
        .catch(error => {
          set({ value: initialValue, loading: false, error });
        });
    },
    { value: initialValue, loading: false, error: null }
  );
}

/**
 * Combine multiple stores into one derived store
 */
export function combineStores<T extends Record<string, Readable<any>>>(
  stores: T
): Readable<{ [K in keyof T]: T[K] extends Readable<infer U> ? U : never }> {
  const storeEntries = Object.entries(stores);
  const storeValues = storeEntries.map(([, store]) => store);
  
  return derived(storeValues, (values) => {
    const result = {} as any;
    storeEntries.forEach(([key], index) => {
      result[key] = values[index];
    });
    return result;
  });
}

/**
 * Create a derived store that filters array values
 */
export function createFilteredStore<T>(
  store: Readable<T[]>,
  predicate: (item: T) => boolean
): Readable<T[]> {
  return derived(store, items => items.filter(predicate));
}

/**
 * Create a derived store that sorts array values
 */
export function createSortedStore<T>(
  store: Readable<T[]>,
  compareFn: (a: T, b: T) => number
): Readable<T[]> {
  return derived(store, items => [...items].sort(compareFn));
}

/**
 * Create a derived store that groups array values
 */
export function createGroupedStore<T, K extends string | number | symbol>(
  store: Readable<T[]>,
  keyFn: (item: T) => K
): Readable<Record<K, T[]>> {
  return derived(store, items => {
    const groups = {} as Record<K, T[]>;
    for (const item of items) {
      const key = keyFn(item);
      if (!groups[key]) {
        groups[key] = [];
      }
      groups[key].push(item);
    }
    return groups;
  });
}

/**
 * Create a derived store that maps array values
 */
export function createMappedStore<T, U>(
  store: Readable<T[]>,
  mapFn: (item: T, index: number) => U
): Readable<U[]> {
  return derived(store, items => items.map(mapFn));
}

/**
 * Create a derived store with memoization
 */
export function createMemoizedStore<T, U>(
  store: Readable<T>,
  fn: (value: T) => U,
  isEqual: (a: T, b: T) => boolean = (a, b) => a === b
): Readable<U> {
  let lastValue: T;
  let lastResult: U;
  let initialized = false;

  return derived(store, value => {
    if (!initialized || !isEqual(value, lastValue)) {
      lastValue = value;
      lastResult = fn(value);
      initialized = true;
    }
    return lastResult;
  });
}