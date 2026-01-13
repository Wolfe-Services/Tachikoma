export interface SpecFile {
  id: string;
  path: string;
  content: string;
  frontmatter: SpecFrontmatter;
  lastModified: string;
  checksum: string;
}

export interface SpecFrontmatter {
  phase: number;
  specId: number;
  title: string;
  status: string;
  dependencies: string[];
  estimatedContext: string;
}

export interface ViewerState {
  specId: string | null;
  content: string;
  originalContent: string;
  isDirty: boolean;
  isSaving: boolean;
  lastSaved: string | null;
  cursorPosition: { line: number; column: number };
  scrollPosition: { top: number; left: number };
}

export interface FindReplaceState {
  isOpen: boolean;
  query: string;
  replacement: string;
  caseSensitive: boolean;
  wholeWord: boolean;
  useRegex: boolean;
  currentMatch: number;
  totalMatches: number;
  matches: Array<{
    line: number;
    column: number;
    length: number;
  }>;
}