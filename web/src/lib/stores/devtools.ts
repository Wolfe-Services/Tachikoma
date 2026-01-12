import { writable, type Writable, type Readable } from 'svelte/store';
import { dev } from '$app/environment';

interface DevtoolsMessage {
  type: string;
  storeName: string;
  data: any;
  timestamp: number;
}

interface StoreDevtools {
  log: (message: DevtoolsMessage) => void;
  getHistory: () => DevtoolsMessage[];
  clear: () => void;
  subscribe: (callback: (history: DevtoolsMessage[]) => void) => () => void;
}

class TachikomaStoreDevtools {
  private history: DevtoolsMessage[] = [];
  private subscribers = new Set<(history: DevtoolsMessage[]) => void>();
  private maxHistory = 1000;

  log(message: DevtoolsMessage) {
    if (!dev) return;

    this.history.push(message);
    
    // Keep history within bounds
    if (this.history.length > this.maxHistory) {
      this.history = this.history.slice(-this.maxHistory);
    }

    // Notify subscribers
    this.subscribers.forEach(callback => callback([...this.history]));

    // Log to console in development
    console.group(`🏪 Store: ${message.storeName}`);
    console.log('Type:', message.type);
    console.log('Data:', message.data);
    console.log('Time:', new Date(message.timestamp).toISOString());
    console.groupEnd();
  }

  getHistory() {
    return [...this.history];
  }

  clear() {
    this.history = [];
    this.subscribers.forEach(callback => callback([]));
  }

  subscribe(callback: (history: DevtoolsMessage[]) => void) {
    this.subscribers.add(callback);
    callback([...this.history]);

    return () => {
      this.subscribers.delete(callback);
    };
  }
}

export const storeDevtools = new TachikomaStoreDevtools();

// Make devtools available globally in development
if (dev && typeof window !== 'undefined') {
  (window as any).__TACHIKOMA_STORE_DEVTOOLS__ = storeDevtools;
}

/**
 * Wrap a store with devtools integration
 */
export function withDevtools<T>(
  store: Writable<T>,
  name: string
): Writable<T> {
  if (!dev) return store;

  const { subscribe, set, update } = store;

  // Log initial state
  storeDevtools.log({
    type: 'INIT',
    storeName: name,
    data: null,
    timestamp: Date.now()
  });

  return {
    subscribe: (run) => {
      return subscribe((value) => {
        storeDevtools.log({
          type: 'SUBSCRIBE',
          storeName: name,
          data: value,
          timestamp: Date.now()
        });
        run(value);
      });
    },

    set: (value) => {
      storeDevtools.log({
        type: 'SET',
        storeName: name,
        data: value,
        timestamp: Date.now()
      });
      set(value);
    },

    update: (updater) => {
      update((currentValue) => {
        const newValue = updater(currentValue);
        storeDevtools.log({
          type: 'UPDATE',
          storeName: name,
          data: { from: currentValue, to: newValue },
          timestamp: Date.now()
        });
        return newValue;
      });
    }
  };
}

/**
 * Create a store with automatic devtools integration
 */
export function createDevStore<T>(initialValue: T, name: string): Writable<T> {
  const store = writable<T>(initialValue);
  return withDevtools(store, name);
}

/**
 * Monitor a readable store with devtools
 */
export function monitorStore<T>(store: Readable<T>, name: string): Readable<T> {
  if (!dev) return store;

  return {
    subscribe: (run) => {
      return store.subscribe((value) => {
        storeDevtools.log({
          type: 'READ',
          storeName: name,
          data: value,
          timestamp: Date.now()
        });
        run(value);
      });
    }
  };
}

/**
 * DevTools component for integration into the UI
 */
export interface DevtoolsState {
  isOpen: boolean;
  history: DevtoolsMessage[];
  filter: string;
  selectedStore: string | null;
}

export const devtoolsStore = writable<DevtoolsState>({
  isOpen: false,
  history: [],
  filter: '',
  selectedStore: null
});

// Subscribe to devtools history
if (dev) {
  storeDevtools.subscribe(history => {
    devtoolsStore.update(state => ({ ...state, history }));
  });
}

export const devtoolsActions = {
  toggle: () => {
    devtoolsStore.update(state => ({ ...state, isOpen: !state.isOpen }));
  },

  setFilter: (filter: string) => {
    devtoolsStore.update(state => ({ ...state, filter }));
  },

  selectStore: (storeName: string | null) => {
    devtoolsStore.update(state => ({ ...state, selectedStore: storeName }));
  },

  clear: () => {
    storeDevtools.clear();
  }
};