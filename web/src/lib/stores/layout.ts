import { writable, derived } from 'svelte/store';
import { browser } from '$app/environment';

export interface LayoutState {
  sidebarOpen: boolean;
  sidebarWidth: number;
  panelSizes: Record<string, number>;
  activePanel: string | null;
  isFullscreen: boolean;
}

const STORAGE_KEY = 'tachikoma:layout';

function getInitialState(): LayoutState {
  if (browser) {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved) {
      try {
        return JSON.parse(saved);
      } catch {
        // Invalid JSON, use defaults
      }
    }
  }

  return {
    sidebarOpen: true,
    sidebarWidth: 240,
    panelSizes: {},
    activePanel: null,
    isFullscreen: false
  };
}

function createLayoutStore() {
  const { subscribe, set, update } = writable<LayoutState>(getInitialState());

  // Persist to localStorage
  if (browser) {
    subscribe(state => {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
    });
  }

  return {
    subscribe,

    toggleSidebar: () => {
      update(state => ({
        ...state,
        sidebarOpen: !state.sidebarOpen
      }));
    },

    setSidebarOpen: (open: boolean) => {
      update(state => ({
        ...state,
        sidebarOpen: open
      }));
    },

    setSidebarWidth: (width: number) => {
      update(state => ({
        ...state,
        sidebarWidth: Math.max(180, Math.min(400, width))
      }));
    },

    setPanelSize: (panelId: string, size: number) => {
      update(state => ({
        ...state,
        panelSizes: {
          ...state.panelSizes,
          [panelId]: size
        }
      }));
    },

    setActivePanel: (panelId: string | null) => {
      update(state => ({
        ...state,
        activePanel: panelId
      }));
    },

    toggleFullscreen: () => {
      update(state => ({
        ...state,
        isFullscreen: !state.isFullscreen
      }));
    },

    reset: () => {
      set(getInitialState());
    }
  };
}

export const layoutStore = createLayoutStore();

// Derived stores
export const sidebarOpen = derived(layoutStore, $layout => $layout.sidebarOpen);
export const isFullscreen = derived(layoutStore, $layout => $layout.isFullscreen);