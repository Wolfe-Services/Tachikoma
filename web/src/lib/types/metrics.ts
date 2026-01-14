export interface FailureReason {
  reason: string;
  count: number;
  percent: number;
}

export interface SuccessRateData {
  rate: number;
  change: number;
  successful: number;
  failed: number;
  total: number;
  trendData: number[];
  failureReasons: FailureReason[];
}