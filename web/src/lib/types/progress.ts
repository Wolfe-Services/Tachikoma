/**
 * Types for progress display functionality.
 */

export interface ProgressInfo {
  percentage: number;
  currentStep: number;
  totalSteps: number;
  currentAction: string;
  elapsedMs: number;
  estimatedRemainingMs: number;
  stepsCompleted: StepInfo[];
  isPaused: boolean;
  isIndeterminate: boolean;
}

export interface StepInfo {
  number: number;
  name: string;
  status: StepStatus;
  duration: number;
  startedAt: string;
  completedAt?: string;
}

export type StepStatus = 'pending' | 'running' | 'complete' | 'error' | 'skipped';

export interface ProgressDisplayConfig {
  showSteps: boolean;
  showTime: boolean;
  showCurrentAction: boolean;
  animate: boolean;
  announceChanges: boolean;
}

export function formatDuration(ms: number): string {
  if (ms < 1000) return 'less than a second';

  const seconds = Math.floor(ms / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);

  if (hours > 0) {
    return `${hours}h ${minutes % 60}m`;
  }
  if (minutes > 0) {
    return `${minutes}m ${seconds % 60}s`;
  }
  return `${seconds}s`;
}

export function estimateRemaining(
  elapsed: number,
  percentage: number
): number {
  if (percentage <= 0) return 0;
  const total = elapsed / (percentage / 100);
  return Math.max(0, total - elapsed);
}