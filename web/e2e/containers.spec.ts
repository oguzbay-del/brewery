import AxeBuilder from '@axe-core/playwright';
import { expect, test, type Page } from '@playwright/test';

const workspaceId = '12345678-1234-4234-8234-123456789abc';
const kegId = '22222222-2222-4222-8222-222222222222';

const container = {
  id: kegId,
  serial_number: 'KEG-0001',
  package_type_id: '33333333-3333-4333-8333-333333333333',
  package_type_name: 'Keg 30 L',
  package_kind: 'keg',
  volume_l: 30,
  status: 'filled',
  version: 1,
  created_at: '2026-11-01T10:00:00Z',
  updated_at: '2026-11-02T10:00:00Z',
};

async function mockApi(page: Page, production: { enabled: boolean; read_only: boolean }) {
  await page.addInitScript(() => localStorage.setItem('demleme.locale', 'en'));
  await page.route('**/v1/me/session', (route) =>
    route.fulfill({ json: { workspaces: [{ id: workspaceId }] } }),
  );
  await page.route('**/v1/meta?*', (route) =>
    route.fulfill({ json: { modules: [{ id: 'production', ...production }] } }),
  );
  await page.route('**/package-types', (route) =>
    route.fulfill({
      json: {
        items: [
          {
            id: container.package_type_id,
            kind: 'keg',
            name: 'Keg 30 L',
            volume_l: 30,
            single_use: false,
            system: true,
          },
        ],
      },
    }),
  );
  await page.route(`**/containers/${kegId}`, (route) =>
    route.fulfill({
      json: {
        container,
        events: [
          {
            id: 'e1',
            event_type: 'fill',
            old_status: 'clean',
            new_status: 'filled',
            note: 'BBT1',
            occurred_at: '2026-11-02T10:00:00Z',
          },
        ],
      },
    }),
  );
  await page.route(/\/containers(\?.*)?$/, (route) =>
    route.request().method() === 'GET'
      ? route.fulfill({
          json: {
            items: [container],
            next_cursor: null,
            status_counts: ['clean', 'filled', 'out', 'dirty', 'maintenance', 'retired'].map(
              (status, index) => ({ status, count: index }),
            ),
          },
        })
      : route.fallback(),
  );
}

test('kegs screen: list, counts, detail with events, no axe violations', async ({ page }) => {
  await mockApi(page, { enabled: true, read_only: false });
  await page.goto('/#/containers');
  await expect(page.getByRole('heading', { level: 1, name: 'Kegs' })).toBeVisible();
  await expect(page.getByRole('button', { name: /Dirty\s*3/ })).toBeVisible();
  await page.getByRole('button', { name: 'KEG-0001' }).click();
  const detail = page.getByRole('region', { name: 'Keg detail' });
  await expect(detail.getByText('BBT1')).toBeVisible();
  await expect(detail.getByText(/Filled \(Clean → Filled\)/)).toBeVisible();
  const results = await new AxeBuilder({ page }).analyze();
  expect(results.violations).toEqual([]);
});

test('kegs screen: Production off hides the entry and shows the upgrade message', async ({
  page,
}) => {
  await mockApi(page, { enabled: false, read_only: false });
  await page.goto('/#/containers');
  await expect(page.getByRole('heading', { name: /module is off/i })).toBeVisible();
  await expect(page.getByRole('navigation').getByRole('link', { name: /Kegs/ })).toHaveCount(0);
  const results = await new AxeBuilder({ page }).analyze();
  expect(results.violations).toEqual([]);
});

test('kegs screen: read only disables writes with a text reason', async ({ page }) => {
  await mockApi(page, { enabled: true, read_only: true });
  await page.goto('/#/containers');
  await expect(page.getByRole('button', { name: 'Print labels' })).toHaveAttribute(
    'aria-disabled',
    'true',
  );
  await expect(page.getByText(/Read only: the Production module/).first()).toBeVisible();
  const results = await new AxeBuilder({ page }).analyze();
  expect(results.violations).toEqual([]);
});

for (const width of [400, 1280]) {
  test(`kegs screen has no horizontal overflow at ${width}px`, async ({ page }) => {
    await mockApi(page, { enabled: true, read_only: false });
    await page.setViewportSize({ width, height: 800 });
    await page.goto('/#/containers');
    await expect(page.getByRole('button', { name: 'KEG-0001' })).toBeVisible();
    const { scrollWidth, clientWidth } = await page.evaluate(() => ({
      scrollWidth: document.documentElement.scrollWidth,
      clientWidth: document.documentElement.clientWidth,
    }));
    expect(scrollWidth).toBeLessThanOrEqual(clientWidth);
  });
}
