/**
 * Test types for test charts visualization (Spec 305).
 */

export interface TestResult {
  id: string;
  name: string;
  status: 'passed' | 'failed' | 'skipped' | 'pending';
  duration: number;
  error?: string;
  flaky?: boolean;
}

export interface TestSuite {
  id: string;
  name: string;
  tests: TestResult[];
  duration: number;
  coverage?: number;
}

export interface TestRunSummary {
  totalTests: number;
  passed: number;
  failed: number;
  skipped: number;
  duration: number;
  timestamp: string;
}