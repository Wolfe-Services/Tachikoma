export interface MarkdownConfig {
  syntaxHighlight: boolean;
  interactiveCheckboxes: boolean;
  generateToc: boolean;
  mermaidDiagrams: boolean;
  customCallouts: boolean;
}

export interface TocItem {
  id: string;
  text: string;
  level: number;
}

export interface CheckboxState {
  lineNumber: number;
  checked: boolean;
}

export const DEFAULT_MARKDOWN_CONFIG: MarkdownConfig = {
  syntaxHighlight: true,
  interactiveCheckboxes: true,
  generateToc: true,
  mermaidDiagrams: true,
  customCallouts: true,
};