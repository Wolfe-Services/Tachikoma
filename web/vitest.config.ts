import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { resolve } from 'path';

export default defineConfig({
  plugins: [svelte({ hot: !process.env.VITEST })],
  
  resolve: {
    alias: {
      $lib: resolve(__dirname, './src/lib'),
      $app: resolve(__dirname, './src/app')
    }
  },

  test: {
    include: ['src/**/*.{test,spec}.{js,ts}'],
    globals: true,
    environment: 'jsdom',
    setupFiles: ['./src/test/setup.ts'],

    // Reporter configuration for comprehensive test reporting
    reporters: [
      'default',
      'json',
      'junit',
      'html',
    ],

    outputFile: {
      json: './test-results/results.json',
      junit: './test-results/junit.xml',
      html: './test-results/index.html',
    },

    coverage: {
      // Use v8 provider for accurate coverage
      provider: 'v8',

      // Enable coverage collection
      enabled: true,

      // Output formats
      reporter: ['text', 'json', 'html', 'lcov'],

      // Output directory
      reportsDirectory: './coverage',

      // Files to include
      include: ['src/**/*.{ts,svelte}'],

      // Files to exclude
      exclude: [
        'node_modules/',
        'src/test/',
        '**/*.d.ts',
        '**/*.test.ts',
        '**/*.spec.ts',
        '**/index.ts',
      ],

      // Coverage thresholds
      thresholds: {
        lines: 70,
        functions: 70,
        branches: 60,
        statements: 70,
      },

      // Fail if thresholds not met
      thresholdAutoUpdate: false,

      // Show uncovered lines in output
      all: true,
    },

    // Include timing information
    benchmark: {
      include: ['**/*.bench.{js,ts}'],
      reporters: ['default', 'json'],
      outputFile: './test-results/benchmark.json',
    },
  },
});