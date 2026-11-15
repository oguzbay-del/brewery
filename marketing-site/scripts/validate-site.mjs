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
  const ogTitleCount = (html.match(/property=["']og:title["']/g) || []).length;
  const ogDescriptionCount = (html.match(/property=["']og:description["']/g) || []).length;
  const ogUrlCount = (html.match(/property=["']og:url["']/g) || []).length;
  const twitterCardCount = (html.match(/name=["']twitter:card["']/g) || []).length;
  const twitterTitleCount = (html.match(/name=["']twitter:title["']/g) || []).length;
  const twitterDescriptionCount = (html.match(/name=["']twitter:description["']/g) || []).length;
  const ogImageCount = (html.match(/property=["']og:image["']/g) || []).length;
  const twitterImageCount = (html.match(/name=["']twitter:image["']/g) || []).length;
  assert(ogTitleCount === 1, rel + ": exactly one Open Graph title");
  assert(ogDescriptionCount === 1, rel + ": exactly one Open Graph description");
  assert(ogUrlCount === 1, rel + ": exactly one Open Graph URL");
  assert(twitterCardCount === 1, rel + ": exactly one Twitter card type");
  assert(twitterTitleCount <= 1, rel + ": at most one Twitter title");
  assert(twitterDescriptionCount <= 1, rel + ": at most one Twitter description");
  assert(ogImageCount <= 1, rel + ": at most one Open Graph image");
  assert(twitterImageCount <= 1, rel + ": at most one Twitter image");
  const twitterCard = html.match(/<meta\s+name=["']twitter:card["']\s+content=["']([^"']+)["']/)?.[1];
  assert(["summary", "summary_large_image"].includes(twitterCard), rel + ": Twitter card uses a supported card type");
  const ogTitle = html.match(/<meta\s+property=["']og:title["']\s+content=["']([^"']+)["']/)?.[1];
  const twitterTitle = html.match(/<meta\s+name=["']twitter:title["']\s+content=["']([^"']+)["']/)?.[1];
  if (twitterTitle) assert(Boolean(twitterTitle.trim()), rel + ": Twitter title is not empty");
  if (ogTitle && twitterTitle) assert(decodeHtml(ogTitle) === decodeHtml(twitterTitle), rel + ": Twitter title matches Open Graph title");
  const ogDescription = html.match(/<meta\s+property=["']og:description["']\s+content=["']([^"']+)["']/)?.[1];
  const twitterDescription = html.match(/<meta\s+name=["']twitter:description["']\s+content=["']([^"']+)["']/)?.[1];
  if (ogDescription) assert(Boolean(ogDescription.trim()), rel + ": Open Graph description is not empty");
  if (twitterDescription) assert(Boolean(twitterDescription.trim()), rel + ": Twitter description is not empty");
  const ogImage = html.match(/<meta\s+property=["']og:image["']\s+content=["']([^"']+)["']/)?.[1];
  const twitterImage = html.match(/<meta\s+name=["']twitter:image["']\s+content=["']([^"']+)["']/)?.[1];
  if (ogImage && twitterImage) assert(new URL(decodeHtml(ogImage), pageUrl).href === new URL(decodeHtml(twitterImage), pageUrl).href, rel + ": Twitter image matches Open Graph image");
  for (const [label, imageUrl] of [["Open Graph", ogImage], ["Twitter", twitterImage]]) {
    if (!imageUrl) continue;
    try {
      const parsedImage = new URL(decodeHtml(imageUrl), pageUrl);
      if (parsedImage.origin === siteOrigin) assert(await exists(toDistFile(parsedImage.pathname)), rel + ": " + label + " image target exists");
      else assert(parsedImage.protocol === "https:", rel + ": " + label + " image uses HTTPS");
    } catch { failures.push(rel + ": invalid " + label + " image URL"); }
  }
  assert(html.includes('<html lang="en">'), rel + ": language is declared");
  const canonicalTag = html.match(/<link\b[^>]*rel=["']canonical["'][^>]*>/)?.[0];
  const canonicalHref = canonicalTag?.match(/\bhref=["']([^"']+)["']/)?.[1];
  const ogUrl = html.match(/<meta\s+property=["']og:url["']\s+content=["']([^"']+)["']/)?.[1];
  assert(Boolean(canonicalHref), rel + ": canonical URL is present");
  if (canonicalHref) {
    const canonical = new URL(decodeHtml(canonicalHref), siteOrigin);
    assert(canonical.origin === siteOrigin && canonical.pathname === pageUrl.pathname, rel + ": canonical URL matches page path");
    if (ogUrl) assert(new URL(decodeHtml(ogUrl), siteOrigin).href === canonical.href, rel + ": Open Graph URL matches canonical");
  }
  if (pagePath !== "/") {
    const jsonLdBlocks = [...html.matchAll(/<script\s+type=["']application\/ld\+json["'][^>]*>([\s\S]*?)<\/script>/g)].map((match) => match[1]);
    const breadcrumb = jsonLdBlocks.map((block) => { try { return JSON.parse(block); } catch { return null; } }).find((item) => item?.["@type"] === "BreadcrumbList");
    assert(Boolean(breadcrumb), rel + ": BreadcrumbList structured data exists");
    if (breadcrumb) {
      const items = breadcrumb.itemListElement || [];
      assert(items.length >= 2 && items.every((item, index) => item["@type"] === "ListItem" && item.position === index + 1 && typeof item.name === "string" && typeof item.item === "string"), rel + ": breadcrumb items have ordered names and URLs");
      const breadcrumbUrls = items.map((item) => { try { return new URL(item.item); } catch { return null; } });
      assert(breadcrumbUrls.every((url) => url?.origin === siteOrigin), rel + ": breadcrumb URLs are absolute and stay on the site origin");
      assert(breadcrumbUrls[0]?.href === new URL(`${basePath}/`, siteOrigin).href, rel + ": first breadcrumb points to the site home");
      assert(items.at(-1)?.item === pageUrl.href, rel + ": final breadcrumb points to the canonical page URL");
    }
  }
  const jsonLd = html.match(/<script\s+type=["']application\/ld\+json["'][^>]*>([\s\S]*?)<\/script>/)?.[1];
  assert(Boolean(jsonLd), rel + ": JSON-LD structured data exists");
  if (jsonLd) {
    try {
      const structured = JSON.parse(jsonLd);
      assert(true, rel + ": JSON-LD is valid JSON");
      if (pagePath === "/faq/") {
        const questions = Array.isArray(structured.mainEntity) ? structured.mainEntity : [];
        const visibleFaqCount = (html.match(/<details class=["']audience-faq-item["']/g) || []).length;
        assert(structured["@type"] === "FAQPage", "FAQ route uses FAQPage structured data");
        assert(questions.length === visibleFaqCount && visibleFaqCount > 0, "FAQ schema question count matches visible FAQ items");
        assert(questions.every((item) =>
          item["@type"] === "Question" &&
          typeof item.name === "string" &&
          item.acceptedAnswer?.["@type"] === "Answer" &&
          typeof item.acceptedAnswer.text === "string"
        ), "FAQ schema questions and accepted answers are complete");
      }
    } catch { failures.push(rel + ": JSON-LD is invalid JSON"); }
  }

  for (const phrase of forbidden) {
    assert(!html.toLowerCase().includes(phrase.toLowerCase()), `${rel}: no unsafe legacy claim “${phrase}”`);
  }

  const idList = [...html.matchAll(/\bid=["']([^"']+)["']/g)].map((match) => match[1]);
  const ids = new Set(idList);
  assert(idList.length === ids.size, `${rel}: no duplicate HTML IDs`);
  assert((html.match(/<h1(?:\s|>)/g) || []).length === 1, `${rel}: exactly one primary heading`);
  assert((html.match(/<main(?:\s|>)/g) || []).length === 1, `${rel}: exactly one main landmark`);
  assert(/<a[^>]*class=["']skip-link["'][^>]*href=["']#main-content["']/.test(html), `${rel}: skip link targets main content`);
  const navTags = [...html.matchAll(/<nav\b([^>]*)>/g)].map((match) => match[1]);
  assert(navTags.length > 0 && navTags.every((attrs) => /aria-label=["'][^"']+["']|aria-labelledby=["'][^"']+["']/.test(attrs)), `${rel}: navigation landmarks have accessible names`);
  const imageTags = [...html.matchAll(/<img\b([^>]*)>/g)].map((match) => match[1]);
  assert(imageTags.every((attrs) => /\balt=["'][^"']*["']/.test(attrs)), `${rel}: every image has an alt attribute`);
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
