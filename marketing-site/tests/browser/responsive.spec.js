import { test, expect } from "@playwright/test";

const routes = [
  "/",
  "/tr/",
  "/homebrewers/",
  "/breweries/",
  "/how-it-works/",
  "/technical-principles/",
  "/faq/",
  "/pricing/",
  "/get-started/",
  "/compare/",
  "/features/",
  "/features/recipe-design/",
];
const widths = [1440, 1024, 768, 390, 320];
const screenshotRoutes = new Set(["/", "/tr/", "/homebrewers/", "/breweries/"]);

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
        overflowers: [...document.querySelectorAll("body *")]
          .map((element) => ({ tag: element.tagName, className: typeof element.className === "string" ? element.className : "", id: element.id, left: Math.round(element.getBoundingClientRect().left), right: Math.round(element.getBoundingClientRect().right), width: Math.round(element.getBoundingClientRect().width), scrollWidth: element.scrollWidth }))
          .filter((element) => element.right > window.innerWidth + 1 || element.left < -1)
          .sort((a, b) => b.right - a.right)
          .slice(0, 12),
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

for (const route of routes.filter((item) => item !== "/" && item !== "/tr/")) {
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
  { route: "/pricing/", href: "/pricing/" },
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

// The approved Stitch navigation (set B) has no Compare link, so Compare and Get started have no current link.
for (const route of ["/compare/", "/get-started/"]) {
  test(`navigation has no current link on ${route}`, async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 900 });
    await page.goto(route, { waitUntil: "domcontentloaded" });
    await expect(page.locator('.desktop-nav a[aria-current="page"]')).toHaveCount(0);
    await expect(page.locator('.mobile-menu nav a[aria-current="page"]')).toHaveCount(0);
  });
}

test("plan wizard supports Home and Business journeys, terms and review", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 900 });
  await page.goto("/get-started/?audience=home", { waitUntil: "domcontentloaded" });
  await expect(page.locator('[data-step="1"]')).toBeVisible();
  await page.getByRole("button", { name: /Continue/ }).click();
  await expect(page.locator('[data-step="2"]')).toBeVisible();
  await expect(page.locator("#home-seat")).toBeVisible();
  await page.getByRole("button", { name: /Continue/ }).click();
  await expect(page.locator('[data-step="3"]')).toBeVisible();
  await page.getByLabel(/Annual billing/).check();
  await page.getByRole("button", { name: /Continue/ }).click();
  await expect(page.locator('[data-step="4"]')).toBeVisible();
  await expect(page.locator("#review-audience")).toHaveText("Homebrewer");
  await expect(page.locator("#review-seats")).toHaveText("1 user");
  await expect(page.locator("#review-trial")).toHaveText("3 months");
  await expect(page.locator("#summary-trial")).toHaveText("3 months");

  await page.goto("/get-started/?audience=business", { waitUntil: "domcontentloaded" });
  await page.getByRole("button", { name: /Continue/ }).click();
  await expect(page.locator("#business-seats")).toBeVisible();
  await page.getByLabel(/10/).check();
  await page.getByRole("button", { name: /Continue/ }).click();
  await page.getByRole("button", { name: /Continue/ }).click();
  await expect(page.locator("#review-audience")).toHaveText("Professional Brewery");
  await expect(page.locator("#review-seats")).toHaveText("10 users");
  await expect(page.locator("#review-trial")).toHaveText("1 month");
  await expect(page.locator("#summary-trial")).toHaveText("1 month");
});

test("Business wizard routes teams above 20 seats to custom quote", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 900 });
  await page.goto("/get-started/?audience=business", { waitUntil: "domcontentloaded" });
  await page.getByRole("button", { name: /Continue/ }).click();
  await page.getByLabel(/More than 20 users/).check();
  await expect(page.locator("#contact-tier")).toBeVisible();
  await page.getByRole("button", { name: /Review contact request/ }).click();
  await expect(page.locator('[data-step="4"]')).toBeVisible();
  await expect(page.locator("#review-seats")).toContainText("custom quote");
  await expect(page.locator("#wizard-feedback")).toContainText("tailored quote");
  await expect(page.locator("#summary-seats")).toContainText("custom quote");
});
