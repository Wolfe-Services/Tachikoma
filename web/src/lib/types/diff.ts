/**
 * Types for diff display functionality.
 */

export interface DiffFile {
  path: string;
  oldPath?: string;
  status: DiffStatus;
  hunks: DiffHunk[];
  language: string;
  binary: boolean;
  stats: DiffStats;
}

export type DiffStatus = 'added' | 'modified' | 'deleted' | 'renamed' | 'copied';

export interface DiffHunk {
  oldStart: number;
  oldLines: number;
  newStart: number;
  newLines: number;
  header: string;
  lines: DiffLine[];
}

export interface DiffLine {
  type: 'add' | 'remove' | 'context' | 'info';
  content: string;
  oldLineNumber?: number;
  newLineNumber?: number;
}

export interface DiffStats {
  additions: number;
  deletions: number;
  totalChanges: number;
}

export type DiffViewMode = 'unified' | 'split';

export interface DiffViewConfig {
  mode: DiffViewMode;
  showLineNumbers: boolean;
  syntaxHighlight: boolean;
  contextLines: number;
  wrapLines: boolean;
}

export const DEFAULT_DIFF_CONFIG: DiffViewConfig = {
  mode: 'unified',
  showLineNumbers: true,
  syntaxHighlight: true,
  contextLines: 3,
  wrapLines: false,
};

export function getLanguageFromPath(path: string): string {
  const ext = path.split('.').pop()?.toLowerCase() || '';
  const langMap: Record<string, string> = {
    ts: 'typescript',
    tsx: 'typescript',
    js: 'javascript',
    jsx: 'javascript',
    rs: 'rust',
    py: 'python',
    md: 'markdown',
    json: 'json',
    yaml: 'yaml',
    yml: 'yaml',
    html: 'html',
    css: 'css',
    scss: 'scss',
    svelte: 'svelte',
  };
  return langMap[ext] || 'text';
}