export interface ContextUsage {
  inputTokens: number;
  outputTokens: number;
  totalTokens: number;
  maxTokens: number;
  usagePercent: number;
  zone: ContextZone;
}

export type ContextZone = 'safe' | 'warning' | 'danger' | 'critical';

export const CONTEXT_THRESHOLDS = {
  warning: 60,
  danger: 80,
  critical: 95,
};

export function getContextZone(percent: number): ContextZone {
  if (percent >= CONTEXT_THRESHOLDS.critical) return 'critical';
  if (percent >= CONTEXT_THRESHOLDS.danger) return 'danger';
  if (percent >= CONTEXT_THRESHOLDS.warning) return 'warning';
  return 'safe';
}