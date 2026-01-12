/**
 * Test results types for mission test execution display.
 */

export interface TestRun {
  id: string;
  missionId: string;
  startedAt: string;
  completedAt?: string;
  status: TestRunStatus;
  suites: TestSuite[];
  summary: TestSummary;
  coverage?: CoverageReport;
}

export type TestRunStatus = 'running' | 'passed' | 'failed' | 'error';

export interface TestSuite {
  id: string;
  name: string;
  path: string;
  status: TestStatus;
  tests: TestCase[];
  duration: number;
  startedAt: string;
}

export interface TestCase {
  id: string;
  name: string;
  status: TestStatus;
  duration: number;
  error?: TestError;
  logs: string[];
}

export type TestStatus = 'pending' | 'running' | 'passed' | 'failed' | 'skipped';

export interface TestError {
  message: string;
  stack?: string;
  expected?: string;
  actual?: string;
  diff?: string;
}

export interface TestSummary {
  total: number;
  passed: number;
  failed: number;
  skipped: number;
  duration: number;
}

export interface CoverageReport {
  lines: CoverageMetric;
  branches: CoverageMetric;
  functions: CoverageMetric;
  statements: CoverageMetric;
}

export interface CoverageMetric {
  covered: number;
  total: number;
  percentage: number;
}