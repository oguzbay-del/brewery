import { test, expect } from "@playwright/test";

test("comparison page explains stage labels and provides plan entry points", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 900 });
  await page.goto("/compare/", { waitUntil: "domcontentloaded" });
  await expect(page.getByRole("heading", { name: /Choose the workspace that fits your brew/i })).toBeVisible();
  await expect(page.getByRole("link", { name: /View plans & pricing/i })).toHaveAttribute("href", "/brewery/pricing/");
  await expect(page.getByRole("link", { name: /Configure a plan/i })).toHaveAttribute("href", "/brewery/get-started/");
  await expect(page.getByText(/How to read this table/i)).toBeVisible();
  await expect(page.getByRole("table")).toHaveAccessibleName(/Comparison of planned GRISTORA product areas/i);
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
