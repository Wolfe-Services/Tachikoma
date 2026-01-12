import { defineConfig } from 'vitest/config';
import { resolve } from 'path';

export default defineConfig({
  test: {
    name: 'electron-main',
    root: './',
    environment: 'node',
    include: ['main/**/__tests__/**/*.test.ts', 'preload/**/__tests__/**/*.test.ts'],
    exclude: ['**/node_modules/**', '**/dist/**', '**/build/**'],
    globals: true,
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      include: ['main/**/*.ts', 'preload/**/*.ts'],
      exclude: ['**/__tests__/**', '**/*.d.ts', '**/dist/**'],
      thresholds: {
        global: {
          branches: 80,
          functions: 80,
          lines: 80,
          statements: 80,
        },
      },
    },
    setupFiles: ['./test/setup.ts'],
    mockReset: true,
    restoreMocks: true,
    testTimeout: 10000,
    hookTimeout: 10000,
  },
  resolve: {
    alias: {
      '@electron': resolve(__dirname, './'),
      '@shared': resolve(__dirname, './shared'),
      '@main': resolve(__dirname, './main'),
      '@preload': resolve(__dirname, './preload'),
    },
  },
});