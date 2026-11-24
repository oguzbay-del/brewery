import { expect, test } from '@playwright/test';

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    localStorage.setItem('demleme.theme', 'light');
    const labels = {
      appName: 'Demleme',
      skipToContent: 'Skip to content',
      navigation: 'Primary navigation',
      home: 'Home',
      recipes: 'Recipes',
      batches: 'Batches',
      settings: 'Settings',
      theme: 'Theme',
      light: 'Light',
      dark: 'Dark',
      highContrast: 'High contrast',
    };
    const apply = (): boolean => {
      const root = document.getElementById('root');
      if (!root) return false;
      Object.assign(root.dataset, labels);
      return true;
    };
    if (!apply()) {
      const observer = new MutationObserver(() => {
        if (apply()) observer.disconnect();
      });
      observer.observe(document, { childList: true, subtree: true });
    }
  });
});

test('theme and skip link work', async ({ page }) => {
  await page.goto('/');
  const theme = page.getByRole('combobox', { name: 'Theme' });
  const html = page.locator('html');
  await expect(theme).toBeVisible();

  await theme.selectOption('dark');
  await expect(html).toHaveAttribute('data-theme', 'dark');

  await theme.selectOption('high-contrast');
  await expect(html).toHaveAttribute('data-theme-preference', 'high-contrast');
  await expect(html).toHaveAttribute('data-theme', /contrast/);

  await theme.selectOption('light');
  await expect(html).toHaveAttribute('data-theme', 'light');

  const skip = page.getByRole('link', { name: 'Skip to content' });
  await skip.focus();
  await page.keyboard.press('Enter');
  await expect(page.locator('#main-content')).toBeFocused();
});
