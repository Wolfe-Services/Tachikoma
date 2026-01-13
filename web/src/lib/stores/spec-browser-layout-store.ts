import { writable, derived } from 'svelte/store';
import type { SpecBrowserLayoutState } from '$lib/types/spec-browser-layout';
import { DEFAULT_BROWSER_CONFIG } from '$lib/types/spec-browser-layout';

function createSpecBrowserLayoutStore() {
  const initialState: SpecBrowserLayoutState = {
    navPanelCollapsed: false,
    metadataPanelCollapsed: false,
    navPanelWidth: DEFAULT_BROWSER_CONFIG.navPanel.defaultWidth,
    metadataPanelWidth: DEFAULT_BROWSER_CONFIG.metadataPanel.defaultWidth,
    activePanel: 'content',
    currentSpecId: null,
    viewMode: 'view',
  };

  const { subscribe, set, update } = writable<SpecBrowserLayoutState>(initialState);

  return {
    subscribe,

    toggleNavPanel: () => update(s => ({ ...s, navPanelCollapsed: !s.navPanelCollapsed })),
    toggleMetadataPanel: () => update(s => ({ ...s, metadataPanelCollapsed: !s.metadataPanelCollapsed })),

    setNavPanelWidth: (width: number) => update(s => ({
      ...s,
      navPanelWidth: Math.max(
        DEFAULT_BROWSER_CONFIG.navPanel.minWidth,
        Math.min(DEFAULT_BROWSER_CONFIG.navPanel.maxWidth, width)
      ),
    })),

    setMetadataPanelWidth: (width: number) => update(s => ({
      ...s,
      metadataPanelWidth: Math.max(
        DEFAULT_BROWSER_CONFIG.metadataPanel.minWidth,
        Math.min(DEFAULT_BROWSER_CONFIG.metadataPanel.maxWidth, width)
      ),
    })),

    setActivePanel: (panel: 'nav' | 'content' | 'metadata') =>
      update(s => ({ ...s, activePanel: panel })),

    setCurrentSpec: (specId: string | null) =>
      update(s => ({ ...s, currentSpecId: specId })),

    setViewMode: (mode: 'view' | 'edit' | 'split') =>
      update(s => ({ ...s, viewMode: mode })),

    reset: () => set(initialState),
  };
}

export const specBrowserLayoutStore = createSpecBrowserLayoutStore();