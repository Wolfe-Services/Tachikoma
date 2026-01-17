import { defineConfig, devices } from '@playwright/test';
import path from 'path';

// Path to the built Electron app
const electronPath = process.platform === 'darwin'
  ? path.join(__dirname, '../electron/out/Tachikoma-darwin-arm64/Tachikoma.app/Contents/MacOS/Tachikoma')
  : process.platform === 'win32'
    ? path.join(__dirname, '../electron/out/Tachikoma-win32-x64/Tachikoma.exe')
    : path.join(__dirname, '../electron/out/Tachikoma-linux-x64/tachikoma');

export default defineConfig({
  testDir: './tests',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: [
    ['html', { outputFolder: 'playwright-report' }],
    ['json', { outputFile: 'test-results/results.json' }],
    ['junit', { outputFile: 'test-results/junit.xml' }],
  ],
  timeout: 60_000,

  use: {
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    video: 'on-first-retry',
  },

  projects: [
    {
      name: 'electron',
      use: {
        // Custom launch for Electron
      },
    },
  ],

  // Web server for development testing
  webServer: process.env.E2E_DEV ? {
    command: 'npm run dev',
    url: 'http://localhost:5173',
    reuseExistingServer: !process.env.CI,
  } : undefined,
});