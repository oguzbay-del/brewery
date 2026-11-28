import AxeBuilder from '@axe-core/playwright';
import { expect, test } from '@playwright/test';

const themes = ['light', 'dark', 'contrast', 'contrast-dark'] as const;

for (const theme of themes) {
  test(`Kit gallery: ${theme}`, async ({ page }, testInfo) => {
    await page.goto('/kit.html');
    await expect(page.getByRole('heading', { level: 1, name: 'GRISTORA UI Kit' })).toBeVisible();
    await page.getByRole('combobox', { name: 'Tema / Theme' }).selectOption(theme);
    await expect(page.locator('html')).toHaveAttribute('data-theme', theme);
    await expect(page.locator('[data-case]')).toHaveCount(5);
    await page.screenshot({
      path: testInfo.outputPath(`kit-${theme}.png`),
      animations: 'disabled',
      caret: 'hide',
      fullPage: true,
    });
    const result = await new AxeBuilder({ page })
      .withTags(['wcag2a', 'wcag2aa', 'wcag21aa', 'wcag22aa'])
      .analyze();
    expect(result.violations).toEqual([]);
  });
}
