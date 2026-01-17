import { defineConfig } from '@playwright/test';
import baseConfig from '../playwright.config';

export default defineConfig({
  ...baseConfig,
  testDir: './visual-tests',

  // Visual test specific settings
  expect: {
    toHaveScreenshot: {
      // Allow slight pixel differences due to font rendering
      maxDiffPixelRatio: 0.01,
      threshold: 0.2,
      animations: 'disabled',
    },
  },

  // Run visual tests serially for consistency
  fullyParallel: false,
  workers: 1,

  // Generate snapshots in specific directory
  snapshotDir: './visual-tests/snapshots',
  snapshotPathTemplate: '{snapshotDir}/{testFilePath}/{arg}{ext}',

  projects: [
    {
      name: 'visual-desktop',
      use: {
        viewport: { width: 1920, height: 1080 },
      },
    },
    {
      name: 'visual-laptop',
      use: {
        viewport: { width: 1440, height: 900 },
      },
    },
    {
      name: 'visual-tablet',
      use: {
        viewport: { width: 1024, height: 768 },
      },
    },
  ],
});