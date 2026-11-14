import { readdir, readFile, stat } from "node:fs/promises";
import path from "node:path";

const dist = path.resolve("dist");
const basePath = process.env.GITHUB_PAGES === "true" ? "/brewery" : "";
const failures = [];
const checks = [];

async function walk(dir) {
  const entries = await readdir(dir, { withFileTypes: true });
  const files = [];
  for (const entry of entries) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) files.push(...await walk(full));
    else files.push(full);
  }
  return files;
}

function assert(condition, message) {
  if (!condition) failures.push(message);
  else checks.push(message);
}

function decodeHtml(value) {
  return value.replaceAll("&amp;", "&").replaceAll("&#x27;", "'").replaceAll("&quot;", '"');
}

function stripBase(pathname) {
  if (!basePath) return pathname;
  if (pathname === basePath || pathname === `${basePath}/`) return "/";
  return pathname.startsWith(`${basePath}/`) ? pathname.slice(basePath.length) : pathname;
}

function toDistFile(pathname) {
  const clean = stripBase(decodeURIComponent(pathname.split("?")[0] || "/"));
  const relative = clean.replace(/^\/+/, "");
  if (!relative || clean.endsWith("/")) return path.join(dist, relative, "index.html");
  const exact = path.join(dist, relative);
  return exact;
}

async function exists(file) {
  try { return (await stat(file)).isFile(); } catch { return false; }
}

const allFiles = await walk(dist);
const htmlFiles = allFiles.filter((file) => file.endsWith(".html"));
assert(htmlFiles.length >= 20, `Expected at least 20 generated HTML pages; found ${htmlFiles.length}`);

const sitemapPath = path.join(dist, "sitemap.xml");
const robotsPath = path.join(dist, "robots.txt");
assert(await exists(sitemapPath), "sitemap.xml exists");
assert(await exists(robotsPath), "robots.txt exists");

const sitemap = await readFile(sitemapPath, "utf8");
const firstLoc = sitemap.match(/<loc>(.*?)<\/loc>/)?.[1];
assert(Boolean(firstLoc), "sitemap contains canonical URLs");
const siteOrigin = firstLoc ? new URL(decodeHtml(firstLoc)).origin : "https://example.invalid";

const requiredRoutes = [
  "/", "/homebrewers/", "/breweries/", "/features/",
  "/how-it-works/", "/technical-principles/", "/faq/",
];
for (const route of requiredRoutes) {
  const routeUrl = new URL(`${basePath}${route}`, siteOrigin);
  const file = toDistFile(routeUrl.pathname);
  assert(await exists(file), `required route exists: ${route}`);
  assert(sitemap.includes(`${siteOrigin}${basePath}${route}`), `sitemap includes: ${route}`);
}

const sitemapLocs = [...sitemap.matchAll(/<loc>(.*?)<\/loc>/g)].map((match) => decodeHtml(match[1]));
for (const loc of sitemapLocs) {
  const url = new URL(loc);
  const file = toDistFile(url.pathname);
  assert(await exists(file), `sitemap target exists: ${url.pathname}`);
}

for (const file of htmlFiles) {
  const rel = path.relative(dist, file).replaceAll(path.sep, "/");
  if (rel === "404.html") continue;
  const pagePath = rel === "index.html" ? "/" : rel.endsWith("/index.html") ? `/${rel.slice(0, -"index.html".length)}` : `/${rel}`;
  const pageUrl = new URL(`${basePath}${pagePath}`, siteOrigin);
  assert(sitemap.includes(pageUrl.href), `sitemap lists page: ${pagePath}`);
}
const forbidden = [
  "A complete toolkit for your next brew",
  "selected advanced features available to enable",
  "From formulation to production control",
];
for (const file of htmlFiles) {
  const html = await readFile(file, "utf8");
  const rel = path.relative(dist, file).replaceAll(path.sep, "/");
  if (rel === "404.html") continue;
  const pagePath = rel === "index.html" ? "/" : rel.endsWith("/index.html") ? `/${rel.slice(0, -"index.html".length)}` : `/${rel}`;
  const pageUrl = new URL(`${basePath}${pagePath}`, siteOrigin);
  const titleCount = (html.match(/<title>/g) || []).length;
  const canonicalCount = (html.match(/rel=["']canonical["']/g) || []).length;
  const descriptionCount = (html.match(/name=["']description["']/g) || []).length;
  assert(titleCount === 1, `${rel}: exactly one title`);
  assert(canonicalCount === 1, `${rel}: exactly one canonical`);
  assert(descriptionCount === 1, `${rel}: exactly one meta description`);
  const ogTitleCount = (html.match(/property=["\']og:title["\']/g) || []).length;
  const ogDescriptionCount = (html.match(/property=["\']og:description["\']/g) || []).length;
  const ogUrlCount = (html.match(/property=["\']og:url["\']/g) || []).length;
  const twitterCardCount = (html.match(/name=["\']twitter:card["\']/g) || []).length;
  assert(ogTitleCount === 1, `${rel}: exactly one Open Graph title`);
  assert(ogDescriptionCount === 1, `${rel}: exactly one Open Graph description`);
  assert(ogUrlCount === 1, `${rel}: exactly one Open Graph URL`);
  assert(twitterCardCount === 1, `${rel}: exactly one Twitter card type`);
  assert(html.includes('<html lang="en">'), `${rel}: language is declared`);
  const canonicalTag = html.match(/<link\\b[^>]*rel=["\']canonical["\'][^>]*>/)?.[0];
  const canonicalHref = canonicalTag?.match(/\\bhref=["\']([^"\']+)["\']/)?.[1];
  const ogUrl = html.match(/<meta\\s+property=["\']og:url["\']\\s+content=["\']([^"\']+)["\']/)?.[1];
  assert(Boolean(canonicalHref), `${rel}: canonical URL is present`);
  if (canonicalHref) {
    const canonical = new URL(decodeHtml(canonicalHref), siteOrigin);
    assert(canonical.origin === siteOrigin && canonical.pathname === pageUrl.pathname, `${rel}: canonical URL matches page path`);
    if (ogUrl) assert(new URL(decodeHtml(ogUrl), siteOrigin).href === canonical.href, `${rel}: Open Graph URL matches canonical`);
  }
  const jsonLd = html.match(/<script\\s+type=["\']application\\/ld\\+json["\'][^>]*>([\\s\\S]*?)<\\/script>/)?.[1];
  assert(Boolean(jsonLd), `${rel}: JSON-LD structured data exists`);
  if (jsonLd) {
    try { JSON.parse(jsonLd); assert(true, `${rel}: JSON-LD is valid JSON`); }
    catch { failures.push(rel + ": JSON-LD is invalid JSON"); }
  }

  for (const phrase of forbidden) {
    assert(!html.toLowerCase().includes(phrase.toLowerCase()), `${rel}: no unsafe legacy claim “${phrase}”`);
  }

  const ids = new Set([...html.matchAll(/\bid=["']([^"']+)["']/g)].map((match) => match[1]));
  const attrs = [...html.matchAll(/\b(?:href|src)=["']([^"']+)["']/g)].map((match) => decodeHtml(match[1]));
  for (const raw of attrs) {
    if (!raw || raw.startsWith("data:") || raw.startsWith("mailto:") || raw.startsWith("tel:") || raw.startsWith("javascript:")) continue;
    let url;
    try { url = new URL(raw, pageUrl); } catch { failures.push(`${rel}: invalid URL ${raw}`); continue; }
    if (url.origin !== siteOrigin) continue;
    const targetFile = toDistFile(url.pathname);
    if (!await exists(targetFile)) {
      failures.push(`${rel}: broken local reference ${raw} -> ${path.relative(dist, targetFile)}`);
      continue;
    }
    if (url.hash) {
      const targetHtml = targetFile.endsWith(".html") ? await readFile(targetFile, "utf8") : "";
      const targetIds = new Set([...targetHtml.matchAll(/\bid=["']([^"']+)["']/g)].map((match) => match[1]));
      if (!targetIds.has(decodeURIComponent(url.hash.slice(1)))) failures.push(`${rel}: missing anchor ${raw}`);
    }
  }
}

const robots = await readFile(robotsPath, "utf8");
assert(robots.includes("Sitemap:"), "robots.txt references sitemap");

console.log(`Static site validation: ${checks.length} checks passed, ${failures.length} failures.`);
if (failures.length) {
  for (const failure of failures) console.error(`FAIL: ${failure}`);
  process.exitCode = 1;
} else {
  console.log("Routes, sitemap targets, metadata, local links, anchors and legacy-claim checks passed.");
}
