import { Page, expect } from '@playwright/test';

export interface VisualTestOptions {
  /** Mask dynamic elements (timestamps, counters, etc.) */
  mask?: string[];
  /** Wait for animations to complete */
  waitForAnimations?: boolean;
  /** Additional wait time in ms */
  additionalWait?: number;
  /** Clip to specific element */
  clip?: string;
}

/**
 * Take a visual snapshot with common options
 */
export async function visualSnapshot(
  page: Page,
  name: string,
  options: VisualTestOptions = {}
): Promise<void> {
  // Wait for page to stabilize
  await page.waitForLoadState('networkidle');

  if (options.waitForAnimations !== false) {
    // Disable CSS animations
    await page.addStyleTag({
      content: `
        *, *::before, *::after {
          animation-duration: 0s !important;
          animation-delay: 0s !important;
          transition-duration: 0s !important;
          transition-delay: 0s !important;
        }
      `,
    });
  }

  if (options.additionalWait) {
    await page.waitForTimeout(options.additionalWait);
  }

  // Build mask locators
  const maskLocators = options.mask?.map(selector => page.locator(selector)) ?? [];

  // Add default masks for dynamic content
  const defaultMasks = [
    '[data-testid="timestamp"]',
    '[data-testid="duration"]',
    '[data-testid="session-id"]',
  ];

  for (const selector of defaultMasks) {
    if (await page.locator(selector).count() > 0) {
      maskLocators.push(page.locator(selector));
    }
  }

  const screenshotOptions: Parameters<typeof expect>[1] = {
    mask: maskLocators,
  };

  if (options.clip) {
    await expect(page.locator(options.clip)).toHaveScreenshot(`${name}.png`, screenshotOptions);
  } else {
    await expect(page).toHaveScreenshot(`${name}.png`, screenshotOptions);
  }
}

/**
 * Test all theme variations
 */
export async function testAllThemes(
  page: Page,
  testFn: (theme: string) => Promise<void>
): Promise<void> {
  const themes = ['light', 'dark', 'tachikoma'];

  for (const theme of themes) {
    await page.evaluate((t) => {
      document.documentElement.setAttribute('data-theme', t);
    }, theme);

    await testFn(theme);
  }
}

/**
 * Component visual test helper
 */
export async function testComponent(
  page: Page,
  componentSelector: string,
  name: string,
  options: VisualTestOptions = {}
): Promise<void> {
  await expect(page.locator(componentSelector)).toHaveScreenshot(
    `${name}.png`,
    {
      mask: options.mask?.map(s => page.locator(s)),
    }
  );
}