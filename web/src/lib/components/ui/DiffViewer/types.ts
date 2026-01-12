export interface DiffLine {
  type: 'add' | 'remove' | 'unchanged' | 'header';
  content: string;
  oldLineNumber?: number;
  newLineNumber?: number;
}

export interface DiffHunk {
  oldStart: number;
  oldLines: number;
  newStart: number;
  newLines: number;
  lines: DiffLine[];
  isCollapsed?: boolean;
}

export interface DiffFile {
  oldPath: string;
  newPath: string;
  hunks: DiffHunk[];
  isBinary?: boolean;
  isNew?: boolean;
  isDeleted?: boolean;
  isRenamed?: boolean;
}

export interface WordDiff {
  type: 'add' | 'remove' | 'unchanged';
  content: string;
}

export type ViewMode = 'split' | 'unified';

export interface DiffViewerProps {
  diff: DiffFile;
  viewMode?: ViewMode;
  language?: string;
  showLineNumbers?: boolean;
  expandedContext?: number;
  maxLines?: number; // For virtualization
  class?: string;
}