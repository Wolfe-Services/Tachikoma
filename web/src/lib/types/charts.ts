/**
 * Time Series Chart Types
 */

export interface TimeSeriesPoint {
  timestamp: string;
  value: number;
}

export interface TimeSeriesData {
  id: string;
  label: string;
  color: string;
  points: TimeSeriesPoint[];
}

export interface TimeRange {
  start: Date;
  end: Date;
}

export type TimeGranularity = 'hour' | 'day' | 'week' | 'month';

export interface ChartTooltipData {
  timestamp: string;
  value: number;
  seriesId: string;
  seriesLabel: string;
  color: string;
}