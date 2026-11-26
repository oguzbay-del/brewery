import { test, expect } from "@playwright/test";

test("comparison page provides plan entry points", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 900 });
  await page.goto("/compare/", { waitUntil: "domcontentloaded" });
  await expect(page.getByRole("heading", { name: /Choose the workspace that fits your brew/i })).toBeVisible();
  await expect(page.getByRole("link", { name: /View plans & pricing/i })).toHaveAttribute("href", "/pricing/");
  await expect(page.getByRole("link", { name: /Configure a plan/i })).toHaveAttribute("href", "/get-started/");
  await expect(page.getByRole("table")).toHaveAccessibleName(/Comparison of GRISTORA product areas/i);
});

test("comparison table remains horizontally scrollable without page overflow on mobile", async ({ page }) => {
  await page.setViewportSize({ width: 320, height: 800 });
  await page.goto("/compare/", { waitUntil: "domcontentloaded" });
  const region = page.getByRole("region", { name: /Scrollable Homebrewer and Professional Brewery function comparison/i });
  await expect(region).toBeVisible();
  const sizes = await page.evaluate(() => ({ viewport: innerWidth, page: document.documentElement.scrollWidth, table: document.querySelector(".comparison-table").scrollWidth, region: document.querySelector(".comparison-table-wrap").clientWidth }));
  expect(sizes.page).toBeLessThanOrEqual(sizes.viewport);
  expect(sizes.table).toBeGreaterThan(sizes.region);
});

for (const route of ["/compare/", "/tr/compare/"]) {
  test(`each feature group has a row-group header on ${route}`, async ({ page }) => {
    await page.goto(route);
    const table = page.getByRole("table");
    await expect(table.locator('thead th[scope="col"]')).toHaveCount(3);
    const groups = table.locator("tbody");
    expect(await groups.count()).toBeGreaterThan(1);
    for (const group of await groups.all()) {
      const rows = group.locator("tr");
      await expect(rows.first().locator('th[scope="rowgroup"]')).toHaveAttribute("colspan", "3");
      const featureCount = await rows.count() - 1;
      expect(featureCount).toBeGreaterThan(0);
      await expect(group.locator('th[scope="row"]')).toHaveCount(featureCount);
    }
  });
}
