import AxeBuilder from '@axe-core/playwright';
import { expect, test } from '@playwright/test';

for (const width of [400, 1280]) {
  for (const theme of ['light', 'dark']) {
    test(`header brand at ${width}px in ${theme} theme`, async ({ page }, testInfo) => {
      await page.setViewportSize({ width, height: 900 });
      await page.goto('/');
      await page.locator('.app-shell__theme-control select').selectOption(theme);
      const brand = page.locator('.app-shell__brand');
      await expect(brand).toHaveAttribute('href', '#/');
      await expect(brand).toHaveAccessibleName(/GRISTORA/i);
      await expect(brand.locator('svg')).toBeVisible();
      const wordmark = brand.locator('.app-shell__wordmark');
      if (width === 400) {
        await expect(wordmark).toBeHidden();
      } else {
        await expect(wordmark).toBeVisible();
      }
      expect(
        await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth),
      ).toBe(true);
      const results = await new AxeBuilder({ page }).include('.app-shell__brand').analyze();
      expect(results.violations).toEqual([]);
      await testInfo.attach('header', {
        body: await page.locator('.app-shell__header').screenshot(),
        contentType: 'image/png',
      });
    });
  }
}
