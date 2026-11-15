import { test, expect } from "@playwright/test";

const routes = [
  "/",
  "/homebrewers/",
  "/breweries/",
  "/how-it-works/",
  "/technical-principles/",
  "/faq/",
  "/features/",
  "/features/recipe-design/",
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
      const brandMark = page.locator(".site-header .brand-symbol");
      await expect(brandMark).toBeVisible();
      await expect(brandMark).toHaveAttribute("aria-hidden", "true");
      const brandMarkBox = await brandMark.boundingBox();
      expect(brandMarkBox?.width ?? Infinity, "brand emblem width stays compact").toBeLessThanOrEqual(34);
      expect(brandMarkBox?.height ?? Infinity, "brand emblem height stays compact").toBeLessThanOrEqual(44);

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

for (const route of routes.filter((item) => item !== "/")) {
  test(`visible breadcrumb navigation is complete on ${route}`, async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 900 });
    await page.goto(route, { waitUntil: "domcontentloaded" });
    const breadcrumb = page.getByRole("navigation", { name: "Breadcrumb" });
    await expect(breadcrumb).toBeVisible();
    await expect(breadcrumb.getByRole("link").first()).toHaveAttribute("href", /\/$/);
    await expect(breadcrumb.locator('[aria-current="page"]')).toBeVisible();
    await expect(breadcrumb.locator("li").last()).toContainText(/.+/);
  });
}

test("breadcrumb links are keyboard reachable on mobile", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 900 });
  await page.goto("/features/recipe-design/", { waitUntil: "domcontentloaded" });
  const breadcrumb = page.getByRole("navigation", { name: "Breadcrumb" });
  const firstLink = breadcrumb.getByRole("link").first();
  await firstLink.focus();
  await expect(firstLink).toBeFocused();
  await expect(firstLink).toHaveAttribute("href", /\/$/);
});

const activeNavigation = [
  { route: "/homebrewers/", href: "/homebrewers/" },
  { route: "/breweries/", href: "/breweries/" },
  { route: "/how-it-works/", href: "/how-it-works/" },
  { route: "/technical-principles/", href: "/technical-principles/" },
  { route: "/faq/", href: "/faq/" },
  { route: "/features/", href: "/features/" },
  { route: "/features/recipe-design/", href: "/features/" },
];

for (const item of activeNavigation) {
  test(`desktop and mobile navigation identify the current page on ${item.route}`, async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 900 });
    await page.goto(item.route, { waitUntil: "domcontentloaded" });

    const desktopCurrent = page.locator('.desktop-nav a[aria-current="page"]');
    await expect(desktopCurrent).toHaveCount(1);
    await expect(desktopCurrent).toHaveAttribute("href", item.href);

    const mobileCurrent = page.locator('.mobile-menu nav a[aria-current="page"]');
    await expect(mobileCurrent).toHaveCount(1);
    await expect(mobileCurrent).toHaveAttribute("href", item.href);
  });
}
