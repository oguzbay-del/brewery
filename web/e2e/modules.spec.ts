import AxeBuilder from '@axe-core/playwright';
import { expect, test } from '@playwright/test';

test('settings displays three module states from the API', async ({ page }) => {
  const workspaceId = '12345678-1234-4234-8234-123456789abc';
  await page.addInitScript(() => localStorage.setItem('demleme.locale', 'en'));
  await page.route('**/v1/me/session', async (route) => {
    await route.fulfill({ json: { workspaces: [{ id: workspaceId }] } });
  });
  await page.route('**/v1/meta?*', async (route) => {
    expect(new URL(route.request().url()).searchParams.get('workspace_id')).toBe(workspaceId);
    await route.fulfill({
      json: {
        modules: [
          { id: 'core', enabled: true, read_only: false },
          { id: 'devices', enabled: false, read_only: false },
          { id: 'stock', enabled: true, read_only: true },
        ],
      },
    });
  });
  await page.goto('/#/settings');
  const section = page.locator('section[aria-labelledby="modules-heading"]');
  await expect(section.getByRole('listitem')).toHaveCount(3);
  await expect(section.getByRole('listitem').nth(0)).toContainText(/on/i);
  await expect(section.getByRole('listitem').nth(1)).toContainText(/off/i);
  await expect(section.getByRole('listitem').nth(2)).toContainText(/read.only/i);
  const results = await new AxeBuilder({ page }).analyze();
  expect(results.violations).toEqual([]);
});
