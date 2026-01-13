/**
 * Mission comparison types.
 */

export interface MissionComparison {
  missionA: MissionSummary;
  missionB: MissionSummary;
  configDiff: ConfigDiff[];
  metricsDiff: MetricsDiff;
  fileDiff: FileDiff[];
}

export interface MissionSummary {
  id: string;
  title: string;
  createdAt: string;
  state: string;
  duration: number;
  cost: number;
  tokensUsed: number;
  filesChanged: number;
}

export interface ConfigDiff {
  key: string;
  valueA: string;
  valueB: string;
  changed: boolean;
}

export interface MetricsDiff {
  duration: MetricComparison;
  cost: MetricComparison;
  tokens: MetricComparison;
  filesChanged: MetricComparison;
}

export interface MetricComparison {
  a: number;
  b: number;
  diff: number;
  percentDiff: number;
}

export interface FileDiff {
  path: string;
  inA: boolean;
  inB: boolean;
  status: 'same' | 'different' | 'only_a' | 'only_b';
}

export interface ComparisonExportOptions {
  format: 'json' | 'csv' | 'html';
  includeConfig: boolean;
  includeMetrics: boolean;
  includeFiles: boolean;
}