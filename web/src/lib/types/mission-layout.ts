/**
 * Mission panel layout configuration and state types.
 */

export interface PanelConfig {
  id: string;
  minWidth: number;
  maxWidth: number;
  defaultWidth: number;
  collapsible: boolean;
  resizable: boolean;
}

export interface MissionLayoutState {
  sidebarCollapsed: boolean;
  detailsCollapsed: boolean;
  sidebarWidth: number;
  detailsWidth: number;
  activePanel: 'sidebar' | 'main' | 'details';
  focusedElement: string | null;
}

export interface LayoutBreakpoint {
  name: 'mobile' | 'tablet' | 'desktop' | 'wide';
  minWidth: number;
  columns: number;
  sidebarVisible: boolean;
  detailsVisible: boolean;
}

export const LAYOUT_BREAKPOINTS: LayoutBreakpoint[] = [
  { name: 'mobile', minWidth: 0, columns: 1, sidebarVisible: false, detailsVisible: false },
  { name: 'tablet', minWidth: 768, columns: 2, sidebarVisible: true, detailsVisible: false },
  { name: 'desktop', minWidth: 1024, columns: 3, sidebarVisible: true, detailsVisible: true },
  { name: 'wide', minWidth: 1440, columns: 3, sidebarVisible: true, detailsVisible: true },
];

export const DEFAULT_PANEL_CONFIG: Record<string, PanelConfig> = {
  sidebar: {
    id: 'sidebar',
    minWidth: 200,
    maxWidth: 400,
    defaultWidth: 280,
    collapsible: true,
    resizable: true,
  },
  main: {
    id: 'main',
    minWidth: 400,
    maxWidth: Infinity,
    defaultWidth: 600,
    collapsible: false,
    resizable: false,
  },
  details: {
    id: 'details',
    minWidth: 250,
    maxWidth: 500,
    defaultWidth: 320,
    collapsible: true,
    resizable: true,
  },
};