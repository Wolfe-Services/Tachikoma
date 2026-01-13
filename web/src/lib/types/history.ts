/**
 * Mission history view types.
 */

export interface MissionHistoryEntry {
  id: string;
  title: string;
  prompt: string;
  state: string;
  createdAt: string;
  completedAt: string;
  duration: number;
  cost: number;
  tokensUsed: number;
  filesChanged: number;
  tags: string[];
}

export interface HistoryFilter {
  status?: string[];
  dateFrom?: string;
  dateTo?: string;
  tags?: string[];
  search?: string;
}

export interface HistorySort {
  field: 'createdAt' | 'duration' | 'cost' | 'title';
  direction: 'asc' | 'desc';
}