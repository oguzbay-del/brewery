import AxeBuilder from '@axe-core/playwright';
import { expect, test, type Page } from '@playwright/test';

const workspaceId = '12345678-1234-4234-8234-123456789abc';

async function mockApi(page: Page, core: { enabled: boolean; read_only: boolean }) {
  await page.addInitScript(() => localStorage.setItem('demleme.locale', 'en'));
  await page.route('**/v1/me/session', async (route) => {
    await route.fulfill({ json: { workspaces: [{ id: workspaceId }] } });
  });
  await page.route('**/v1/meta?*', async (route) => {
    await route.fulfill({ json: { modules: [{ id: 'core', ...core }] } });
  });
}

test('module on: navigation shows the entries without a mark', async ({ page }) => {
  await mockApi(page, { enabled: true, read_only: false });
  await page.goto('/#/recipes');
  const nav = page.getByRole('navigation');
  await expect(nav.getByRole('link', { name: 'Recipes', exact: true })).toBeVisible();
  await expect(nav.getByRole('link', { name: 'Batches', exact: true })).toBeVisible();
  const results = await new AxeBuilder({ page }).analyze();
  expect(results.violations).toEqual([]);
});

test('module read only: entries stay with text and the screen shows a banner', async ({ page }) => {
  await mockApi(page, { enabled: true, read_only: true });
  await page.goto('/#/recipes');
  const nav = page.getByRole('navigation');
  await expect(nav.getByRole('link', { name: /Recipes\s*\(Read only\)/ })).toBeVisible();
  await expect(page.getByRole('status')).toContainText(/read only/i);
  const results = await new AxeBuilder({ page }).analyze();
  expect(results.violations).toEqual([]);
});

test('module off: entries are hidden and the direct URL shows the upgrade message', async ({
  page,
}) => {
  await mockApi(page, { enabled: false, read_only: false });
  await page.goto('/#/recipes');
  const nav = page.getByRole('navigation');
  await expect(nav.getByRole('link', { name: 'Settings' })).toBeVisible();
  await expect(nav.getByRole('link', { name: /Recipes/ })).toHaveCount(0);
  await expect(nav.getByRole('link', { name: /Batches/ })).toHaveCount(0);
  await expect(page.getByRole('heading', { name: /module is off/i })).toBeVisible();
  await expect(page.getByRole('link', { name: /Settings . Modules/ })).toHaveAttribute(
    'href',
    '#/settings',
  );
  const results = await new AxeBuilder({ page }).analyze();
  expect(results.violations).toEqual([]);
});
