import AxeBuilder from "@axe-core/playwright";
import { test, expect } from "@playwright/test";

const routes = [
  "/",
  "/homebrewers/",
  "/breweries/",
  "/how-it-works/",
  "/technical-principles/",
  "/faq/",
  "/features/",
];
const viewports = [
  { name: "desktop", width: 1440 },
  { name: "mobile", width: 390 },
];

for (const route of routes) {
  for (const viewport of viewports) {
    test(`WCAG 2.2 AA automated audit: ${route} at ${viewport.name}`, async ({ page }) => {
      await page.setViewportSize({ width: viewport.width, height: 900 });
      await page.goto(route, { waitUntil: "domcontentloaded" });

      const results = await new AxeBuilder({ page })
        .withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa"])
        .analyze();

      const violations = results.violations.map((violation) => ({
        id: violation.id,
        impact: violation.impact,
        help: violation.help,
        nodes: violation.nodes.map((node) => ({
          target: node.target,
          summary: node.failureSummary,
        })),
      }));
      expect(violations, JSON.stringify(violations, null, 2)).toEqual([]);
    });
  }
}

test("keyboard users can reach the skip link and main content", async ({ page }) => {
  await page.goto("/", { waitUntil: "domcontentloaded" });
  await page.keyboard.press("Tab");
  await expect(page.locator(".skip-link")).toBeFocused();
  await page.keyboard.press("Enter");
  await expect(page).toHaveURL(/#main-content$/);
});

test("mobile navigation can be opened with the keyboard", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 900 });
  await page.goto("/", { waitUntil: "domcontentloaded" });
  const menu = page.locator(".mobile-menu");
  await menu.locator("summary").focus();
  await page.keyboard.press("Enter");
  await expect(menu).toHaveAttribute("open", "");
  await expect(menu.locator("nav")).toBeVisible();
});

test("FAQ disclosure can be opened with the keyboard", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 900 });
  await page.goto("/faq/", { waitUntil: "domcontentloaded" });
  const firstDetails = page.locator(".audience-faq-item").first();
  const firstSummary = firstDetails.locator("summary");
  await firstSummary.focus();
  await page.keyboard.press("Enter");
  await expect(firstDetails).toHaveAttribute("open", "");
});

for (const route of ["/", "/homebrewers/", "/breweries/"]) {
  test(`narrow desktop layout proxy (720px) remains usable on ${route}`, async ({ page }) => {
    await page.setViewportSize({ width: 720, height: 900 });
    await page.goto(route, { waitUntil: "domcontentloaded" });
    const dimensions = await page.evaluate(() => ({
      viewport: window.innerWidth,
      document: document.documentElement.scrollWidth,
      body: document.body.scrollWidth,
    }));
    expect(dimensions.document).toBeLessThanOrEqual(720);
    expect(dimensions.body).toBeLessThanOrEqual(720);
  });
}
