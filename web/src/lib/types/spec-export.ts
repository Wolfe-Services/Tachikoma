export type ExportFormat = 'markdown' | 'html' | 'pdf' | 'json' | 'docx';

export interface ExportOptions {
  format: ExportFormat;
  includeMetadata: boolean;
  includeDependencies: boolean;
  includeRelated: boolean;
  includeCodeBlocks: boolean;
  includeToc: boolean;
  templateId?: string;
  customStyles?: string;
  pageSize?: 'A4' | 'Letter';
  orientation?: 'portrait' | 'landscape';
}

export interface ExportJob {
  id: string;
  specIds: string[];
  options: ExportOptions;
  status: ExportStatus;
  progress: number;
  outputPath?: string;
  error?: string;
  startedAt: string;
  completedAt?: string;
}

export type ExportStatus = 'pending' | 'processing' | 'completed' | 'failed';

export interface ExportTemplate {
  id: string;
  name: string;
  description: string;
  format: ExportFormat;
  headerTemplate: string;
  footerTemplate: string;
  styles: string;
  isDefault: boolean;
}

export interface ExportPreview {
  content: string;
  pageCount?: number;
  estimatedSize: string;
}

export interface ExportHistory {
  id: string;
  specIds: string[];
  format: ExportFormat;
  outputPath: string;
  timestamp: string;
  fileSize: number;
}