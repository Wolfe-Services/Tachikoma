import { test, expect } from '@playwright/test';
import { visualSnapshot, testAllThemes } from '../utils';

test.describe('Dashboard Visual Tests', () => {
  test.beforeEach(async ({ page }) => {
    // Load test data
    await page.evaluate(() => {
      localStorage.setItem('onboarding_complete', 'true');
    });
    await page.goto('/dashboard');
    await page.waitForLoadState('networkidle');
  });

  test('dashboard empty state', async ({ page }) => {
    await visualSnapshot(page, 'dashboard-empty');
  });

  test('dashboard with missions', async ({ page }) => {
    // Seed mission data
    await page.evaluate(() => {
      localStorage.setItem('missions', JSON.stringify([
        { id: '1', name: 'Test Mission', status: 'complete' },
        { id: '2', name: 'Another Mission', status: 'running' },
      ]));
    });
    await page.reload();

    await visualSnapshot(page, 'dashboard-with-missions', {
      mask: ['[data-testid="mission-timestamp"]'],
    });
  });

  test('dashboard themes', async ({ page }) => {
    await testAllThemes(page, async (theme) => {
      await visualSnapshot(page, `dashboard-${theme}`);
    });
  });
});