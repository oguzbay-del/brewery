import { test, expect, chromium } from "@playwright/test";
import { createServer } from "node:http";
import { readFile, writeFile, mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { DatabaseSync } from "node:sqlite";

test("Chromium stores the WP-16 SVGs as actual tab favicons", async ({}, testInfo) => {
  const sizes = [16, 32, 48];
  const files = new Map();
  for (const size of sizes) {
    const path = `/gristora-favicon-${size}.svg`;
    const svg = await readFile(new URL(`../../../web-stratejisi/assets/gristora${path}`, import.meta.url));
    files.set(path, svg);
  }
  const loaded = new Set();
  const server = createServer((req, res) => {
    const file = files.get(req.url);
    if (file) {
      loaded.add(req.url);
      res.writeHead(200, { "Content-Type": "image/svg+xml" }).end(file);
      return;
    }
    const match = /^\/tab-(16|32|48)$/.exec(req.url);
    if (match) {
      const size = match[1];
      res.writeHead(200, { "Content-Type": "text/html" }).end(
        `<!doctype html><title>WP-16 ${size}</title><link rel="icon" type="image/svg+xml" href="/gristora-favicon-${size}.svg">`
      );
      return;
    }
    res.writeHead(404).end();
  });
  const profile = await mkdtemp(join(tmpdir(), "wp16-favicon-"));
  try {
    await new Promise(resolve => server.listen(0, "127.0.0.1", resolve));
    const origin = `http://127.0.0.1:${server.address().port}`;
    // Full Chromium maintains the tab favicon service and its profile database.
    const browser = await chromium.launchPersistentContext(profile, { channel: "chromium" });
    try {
      for (const size of sizes) {
        const tab = await browser.newPage();
        await tab.goto(`${origin}/tab-${size}`);
        await expect(tab).toHaveTitle(`WP-16 ${size}`);
        await expect.poll(() => loaded.has(`/gristora-favicon-${size}.svg`)).toBe(true);
      }
    } finally {
      await browser.close();
    }
    const db = new DatabaseSync(join(profile, "Default", "Favicons"), { readOnly: true });
    try {
      const select = db.prepare(`
        SELECT f.url, b.width, b.height, b.image_data
        FROM icon_mapping AS m JOIN favicons AS f ON f.id = m.icon_id
        JOIN favicon_bitmaps AS b ON b.icon_id = f.id
        WHERE m.page_url = ? ORDER BY b.width DESC LIMIT 1
      `);
      for (const size of sizes) {
        const icon = select.get(`${origin}/tab-${size}`);
        expect(icon?.url).toBe(`${origin}/gristora-favicon-${size}.svg`);
        expect(icon.width).toBeGreaterThan(0);
        expect(icon.height).toBe(icon.width);
        const png = Buffer.from(icon.image_data);
        expect(png.subarray(0, 8).toString("hex")).toBe("89504e470d0a1a0a");
        await writeFile(testInfo.outputPath(`tab-favicon-${size}.png`), png);
        console.log(`WP-16 ${size}px SVG: tab bitmap ${icon.width}x${icon.height}`);
      }
    } finally {
      db.close();
    }
  } finally {
    await new Promise(resolve => server.close(resolve));
    await rm(profile, { recursive: true, force: true });
  }
});
