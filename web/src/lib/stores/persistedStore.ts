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