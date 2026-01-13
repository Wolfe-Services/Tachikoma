import { createPersistedStore } from './persistedStore';
import type { ForgeLayoutConfig } from '$lib/types/forge';

interface LayoutPreferences {
  forge?: ForgeLayoutConfig;
  [key: string]: any;
}

const defaultPreferences: LayoutPreferences = {};

function createLayoutPreferencesStore() {
  const store = createPersistedStore<LayoutPreferences>(defaultPreferences, {
    key: 'layout-preferences',
    storage: 'localStorage',
    version: 1
  });

  return {
    ...store,
    
    async save(layoutType: string, config: any): Promise<void> {
      store.update(prefs => ({
        ...prefs,
        [layoutType]: config
      }));
    },

    async load(layoutType: string): Promise<any | null> {
      return new Promise((resolve) => {
        const unsubscribe = store.subscribe(prefs => {
          resolve(prefs[layoutType] || null);
          unsubscribe();
        });
      });
    }
  };
}

export const layoutPreferencesStore = createLayoutPreferencesStore();