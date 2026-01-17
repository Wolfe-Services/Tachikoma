import { defineConfig } from 'vitest/config';
import baseConfig from './vitest.config';

export default defineConfig({
  ...baseConfig,
  test: {
    ...baseConfig.test,
    retry: 3, // Retry failed tests

    // Hook to track flaky tests
    onConsoleLog(log) {
      // Could send to tracking service
      return false;
    },

    reporters: [
      'default',
      ['json', { outputFile: 'test-results/flaky-report.json' }],
    ],
  },
});