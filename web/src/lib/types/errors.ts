export interface ErrorItem {
  id: string;
  type: string;
  message: string;
  severity: 'critical' | 'high' | 'medium' | 'low';
  count: number;
  firstSeen: string;
  lastSeen: string;
  affectedMissions: number;
  stackTrace?: string;
}

export interface ErrorTrendPoint {
  timestamp: string;
  count: number;
}

export interface ErrorStats {
  currentRate: number;
  changePercent: number;
  totalErrors: number;
  byType: Record<string, number>;
  trendData: ErrorTrendPoint[];
}