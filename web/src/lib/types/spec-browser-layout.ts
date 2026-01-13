export interface SpecBrowserLayoutState {
  navPanelCollapsed: boolean;
  metadataPanelCollapsed: boolean;
  navPanelWidth: number;
  metadataPanelWidth: number;
  activePanel: 'nav' | 'content' | 'metadata';
  currentSpecId: string | null;
  viewMode: 'view' | 'edit' | 'split';
}

export interface SpecBrowserConfig {
  navPanel: PanelConfig;
  metadataPanel: PanelConfig;
  defaultViewMode: 'view' | 'edit' | 'split';
  showLineNumbers: boolean;
  syntaxHighlight: boolean;
}

export interface PanelConfig {
  minWidth: number;
  maxWidth: number;
  defaultWidth: number;
  collapsible: boolean;
}

export const DEFAULT_BROWSER_CONFIG: SpecBrowserConfig = {
  navPanel: { minWidth: 200, maxWidth: 400, defaultWidth: 260, collapsible: true },
  metadataPanel: { minWidth: 220, maxWidth: 400, defaultWidth: 280, collapsible: true },
  defaultViewMode: 'view',
  showLineNumbers: true,
  syntaxHighlight: true,
};