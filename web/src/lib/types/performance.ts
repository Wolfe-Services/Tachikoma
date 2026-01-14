export interface PercentileData {
  p50: number;
  p90: number;
  p99: number;
}

export interface LatencyTrendPoint {
  timestamp: string;
  p50: number;
  p90: number;
  p99: number;
}

export interface ThroughputTrendPoint {
  timestamp: string;
  value: number;
}

export interface PerformanceAnomaly {
  timestamp: string;
  description: string;
  severity: 'low' | 'medium' | 'high';
  impact: string;
}

export interface PerformanceData {
  avgLatency: number;
  throughput: number;
  errorRate: number;
  percentiles: PercentileData;
  latencyTrend: LatencyTrendPoint[];
  throughputTrend: ThroughputTrendPoint[];
  anomalies?: PerformanceAnomaly[];
}