import { readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { Resvg } from "@resvg/resvg-js";

const source = path.resolve("public/social-card-source.svg");
const destination = path.resolve("public/social-card.png");
const svg = await readFile(source, "utf8");
if (!/<svg\b[^>]*\bwidth=["']1200["'][^>]*\bheight=["']630["']/.test(svg)) {
  throw new Error("Social card source must declare a 1200×630 SVG canvas");
}
const renderer = new Resvg(svg, {
  fitTo: { mode: "original" },
  font: { loadSystemFonts: true },
});
const png = renderer.render().asPng();
if (png.length === 0) throw new Error("Social card renderer returned an empty PNG");
await writeFile(destination, png);
process.stdout.write(`Rendered ${destination} (${png.length} bytes, 1200×630)\n`);
