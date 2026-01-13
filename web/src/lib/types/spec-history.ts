export interface SpecVersion {
  id: string;
  specId: string;
  version: number;
  content: string;
  frontmatter: Record<string, unknown>;
  author: string;
  timestamp: string;
  message: string;
  changes: VersionChanges;
}

export interface VersionChanges {
  additions: number;
  deletions: number;
  sections: string[];
}

export interface VersionComparison {
  base: SpecVersion;
  compare: SpecVersion;
  hunks: DiffHunk[];
}

export interface DiffHunk {
  oldStart: number;
  oldLines: number;
  newStart: number;
  newLines: number;
  lines: DiffLine[];
}

export interface DiffLine {
  type: 'context' | 'addition' | 'deletion';
  content: string;
  oldLineNumber?: number;
  newLineNumber?: number;
}