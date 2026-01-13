export type ExportFormat = 'json' | 'markdown' | 'html' | 'zip';

export interface ExportOptions {
  format: ExportFormat;
  includeConfig: boolean;
  includeLogs: boolean;
  includeDiffs: boolean;
  includeCosts: boolean;
  includeCheckpoints: boolean;
  dateRange?: { from: string; to: string };
}

export interface ExportProgress {
  status: 'preparing' | 'exporting' | 'complete' | 'error';
  progress: number;
  currentItem: string;
  totalItems: number;
}

export interface ExportResult {
  filename: string;
  size: number;
  url: string;
  expiresAt?: string;
}

export interface ShareableLink {
  id: string;
  url: string;
  expiresAt: string;
  accessCount: number;
  maxAccess?: number;
}