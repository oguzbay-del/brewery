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
const widths = [1440, 1024, 768, 390, 320];
const screenshotRoutes = new Set(["/", "/homebrewers/", "/breweries/"]);

for (const route of routes) {
  for (const width of widths) {
    test(`${route} renders without horizontal overflow at ${width}px`, async ({ page }, testInfo) => {
      await page.setViewportSize({ width, height: 900 });
      const pageErrors = [];
      page.on("pageerror", (error) => pageErrors.push(error.message));

      const response = await page.goto(route, { waitUntil: "domcontentloaded" });
      expect(response?.ok(), `HTTP response for ${route}`).toBeTruthy();
      await expect(page.locator("main")).toHaveCount(1);
      await expect(page.locator("h1")).toHaveCount(1);
      await expect(page.locator("header")).toBeVisible();

      const imageResults = await page.locator("img").evaluateAll(async (images) => {
        return Promise.all(images.map(async (img) => {
          img.loading = "eager";
          try {
            await img.decode();
            return { hasAlt: img.hasAttribute("alt"), loaded: img.naturalWidth > 0, src: img.currentSrc };
          } catch {
            return { hasAlt: img.hasAttribute("alt"), loaded: false, src: img.currentSrc };
          }
        }));
      });
      for (const image of imageResults) {
        expect(image.hasAlt, `Image has an alt attribute: ${image.src}`).toBeTruthy();
        expect(image.loaded, `Image loaded: ${image.src}`).toBeTruthy();
      }

      const dimensions = await page.evaluate(() => ({
        viewport: window.innerWidth,
        document: document.documentElement.scrollWidth,
        body: document.body.scrollWidth,
      }));
      expect(dimensions.document, `document overflow at ${width}px: ${JSON.stringify(dimensions)}`).toBeLessThanOrEqual(width);
      expect(dimensions.body, `body overflow at ${width}px: ${JSON.stringify(dimensions)}`).toBeLessThanOrEqual(width);
      expect(pageErrors, `browser errors on ${route}`).toEqual([]);

      if (screenshotRoutes.has(route) || width === 1440 || width === 390) {
        const name = `${route === "/" ? "home" : route.replaceAll("/", "-").replace(/^-|-$/g, "")}-${width}.png`;
        await page.screenshot({ path: testInfo.outputPath(name), fullPage: true, animations: "disabled" });
      }
    });
  }
}

for (const width of [320, 390, 768, 1024, 1440]) {
  test(`primary navigation is usable at ${width}px`, async ({ page }) => {
    await page.setViewportSize({ width, height: 900 });
    await page.goto("/", { waitUntil: "domcontentloaded" });
    const mobileMenu = page.locator(".mobile-menu");
    if (await mobileMenu.isVisible()) {
      await mobileMenu.locator("summary").click();
      await expect(mobileMenu.locator("nav")).toBeVisible();
      await expect(mobileMenu.locator("nav").getByRole("link", { name: /Homebrewers/i })).toBeVisible();
    } else {
      await expect(page.locator(".desktop-nav")).toBeVisible();
    }
  });
}
