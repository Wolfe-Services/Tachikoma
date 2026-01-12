import { writable, derived } from 'svelte/store';
import type { MissionLayoutState, LayoutBreakpoint } from '$lib/types/mission-layout';
import { LAYOUT_BREAKPOINTS, DEFAULT_PANEL_CONFIG } from '$lib/types/mission-layout';

function createMissionLayoutStore() {
  const initialState: MissionLayoutState = {
    sidebarCollapsed: false,
    detailsCollapsed: false,
    sidebarWidth: DEFAULT_PANEL_CONFIG.sidebar.defaultWidth,
    detailsWidth: DEFAULT_PANEL_CONFIG.details.defaultWidth,
    activePanel: 'main',
    focusedElement: null,
  };

  const { subscribe, set, update } = writable<MissionLayoutState>(initialState);

  return {
    subscribe,
    set,
    update,

    toggleSidebar: () => update(s => ({ ...s, sidebarCollapsed: !s.sidebarCollapsed })),
    toggleDetails: () => update(s => ({ ...s, detailsCollapsed: !s.detailsCollapsed })),

    setSidebarWidth: (width: number) => update(s => ({
      ...s,
      sidebarWidth: Math.max(
        DEFAULT_PANEL_CONFIG.sidebar.minWidth,
        Math.min(DEFAULT_PANEL_CONFIG.sidebar.maxWidth, width)
      ),
    })),

    setDetailsWidth: (width: number) => update(s => ({
      ...s,
      detailsWidth: Math.max(
        DEFAULT_PANEL_CONFIG.details.minWidth,
        Math.min(DEFAULT_PANEL_CONFIG.details.maxWidth, width)
      ),
    })),

    setActivePanel: (panel: 'sidebar' | 'main' | 'details') =>
      update(s => ({ ...s, activePanel: panel })),

    setFocusedElement: (elementId: string | null) =>
      update(s => ({ ...s, focusedElement: elementId })),

    reset: () => set(initialState),
  };
}

export const missionLayoutStore = createMissionLayoutStore();

// Derived store for current breakpoint
export const currentBreakpoint = derived<typeof missionLayoutStore, LayoutBreakpoint>(
  missionLayoutStore,
  ($layout, set) => {
    if (typeof window === 'undefined') {
      set(LAYOUT_BREAKPOINTS[2]); // Default to desktop
      return;
    }

    const updateBreakpoint = () => {
      const width = window.innerWidth;
      const breakpoint = [...LAYOUT_BREAKPOINTS]
        .reverse()
        .find(bp => width >= bp.minWidth) || LAYOUT_BREAKPOINTS[0];
      set(breakpoint);
    };

    updateBreakpoint();
    window.addEventListener('resize', updateBreakpoint);

    return () => window.removeEventListener('resize', updateBreakpoint);
  }
);