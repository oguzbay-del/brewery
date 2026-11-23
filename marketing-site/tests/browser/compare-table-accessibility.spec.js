import { test, expect } from "@playwright/test";

for (const width of [320, 390]) {
  test(`comparison table region is keyboard-scrollable and visibly focused at ${width}px`, async ({ page }) => {
    await page.setViewportSize({ width, height: 900 });
    const response = await page.goto("/compare/", { waitUntil: "domcontentloaded" });
    expect(response?.ok(), "comparison page responds successfully").toBeTruthy();

    const region = page.getByRole("region", {
      name: "Scrollable Homebrewer and Professional Brewery function comparison",
    });
    await expect(region).toBeVisible();

    const beforeLayout = await region.evaluate((element) => ({
      clientWidth: element.clientWidth,
      scrollWidth: element.scrollWidth,
    }));
    expect(beforeLayout.scrollWidth).toBeGreaterThan(beforeLayout.clientWidth);

    await region.focus();
    await expect(region).toBeFocused();
    const focusStyle = await region.evaluate((element) => {
      const style = getComputedStyle(element);
      return { outlineStyle: style.outlineStyle, outlineWidth: style.outlineWidth };
    });
    expect(focusStyle.outlineStyle).toBe("solid");
    expect(Number.parseFloat(focusStyle.outlineWidth)).toBeGreaterThanOrEqual(3);

    const beforeScroll = await region.evaluate((element) => element.scrollLeft);
    await page.keyboard.press("ArrowRight");
    const afterScroll = await region.evaluate((element) => element.scrollLeft);
    expect(afterScroll).toBeGreaterThan(beforeScroll);
    const pageWidth = await page.evaluate(() => ({
      viewport: window.innerWidth,
      document: document.documentElement.scrollWidth,
      body: document.body.scrollWidth,
    }));
    expect(pageWidth.document).toBeLessThanOrEqual(width);
    expect(pageWidth.body).toBeLessThanOrEqual(width);
  });
}
