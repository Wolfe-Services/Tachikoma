import { defineConfig } from 'vitest/config';
import { resolve } from 'path';

export default defineConfig({
  test: {
    name: 'electron-main',
    root: './main',
    environment: 'node',
    include: ['**/__tests__/**/*.test.ts'],
    exclude: ['**/node_modules/**'],
    globals: true,
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      include: ['**/*.ts'],
      exclude: ['**/__tests__/**', '**/*.d.ts', '**/dist/**'],
    },
    setupFiles: ['./test/setup.ts'],
    mockReset: true,
    restoreMocks: true,
  },
  resolve: {
    alias: {
      '@electron': resolve(__dirname, './'),
      '@shared': resolve(__dirname, './shared'),
    },
  },
});