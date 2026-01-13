export interface SearchQuery {
  text: string;
  filters: SearchFilters;
}

export interface SearchFilters {
  phases?: number[];
  status?: string[];
  tags?: string[];
  hasCode?: boolean;
  modifiedAfter?: string;
}

export interface SearchResult {
  specId: string;
  title: string;
  path: string;
  phase: number;
  status: string;
  matches: SearchMatch[];
  score: number;
}

export interface SearchMatch {
  field: 'title' | 'content' | 'tag' | 'id';
  text: string;
  context: string;
  lineNumber?: number;
}

export interface SearchHistory {
  query: string;
  timestamp: string;
  resultCount: number;
}