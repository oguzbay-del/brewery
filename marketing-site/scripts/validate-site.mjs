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

function getMetaContent(html, attribute, value) {
  const tags = [...html.matchAll(/<meta\b[^>]*>/gi)].map((match) => match[0]);
  const tag = tags.find((candidate) => {
    const match = candidate.match(new RegExp("\\b" + attribute + "\\s*=\\s*(?:\"([^\"]*)\"|'([^']*)')", "i"));
    return (match?.[1] ?? match?.[2]) === value;
  });
  const content = tag?.match(/\bcontent\s*=\s*(?:"([^"]*)"|'([^']*)')/i);
  return content?.[1] ?? content?.[2];
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

const socialCardPath = path.join(dist, "social-card.png");
const socialCardExists = await exists(socialCardPath);
assert(socialCardExists, "generated social-card.png exists");
if (socialCardExists) {
  const png = await readFile(socialCardPath);
  assert(png.subarray(0, 8).equals(Buffer.from([137, 80, 78, 71, 13, 10, 26, 10])), "social card is a valid PNG file");
  if (png.length >= 24) {
    assert(png.readUInt32BE(16) === 1200, "social card width is 1200 pixels");
    assert(png.readUInt32BE(20) === 630, "social card height is 630 pixels");
  } else {
    assert(false, "social card PNG contains a complete IHDR header");
  }
  assert(png.length <= 512 * 1024, "social card PNG is no larger than 512 KiB");
}

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
assert(sitemap.startsWith('<?xml version="1.0"'), "sitemap declares XML encoding");
assert(sitemap.includes('<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">'), "sitemap uses the standard XML namespace");
assert(sitemapLocs.length > 0, "sitemap contains at least one URL");
assert(new Set(sitemapLocs).size === sitemapLocs.length, "sitemap contains no duplicate URLs");
for (const loc of sitemapLocs) {
  try {
    const url = new URL(loc);
    assert(url.origin === siteOrigin, "sitemap URL stays on the canonical site origin: " + loc);
    assert(!url.search && !url.hash, "sitemap URL has no query string or fragment: " + loc);
    assert(url.pathname === "/" || url.pathname.endsWith("/"), "sitemap route uses a trailing slash: " + loc);
    assert(!url.pathname.endsWith("/404.html"), "sitemap excludes the not-found page");
    const file = toDistFile(url.pathname);
    assert(await exists(file), "sitemap target exists: " + url.pathname);
  } catch {
    failures.push("sitemap contains an invalid absolute URL: " + loc);
  }
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
  const metaDescription = getMetaContent(html, "name", "description");
  assert(Boolean(metaDescription?.trim()), rel + ": meta description is not empty");
  if (metaDescription) {
    const length = decodeHtml(metaDescription).trim().length;
    assert(length >= 40 && length <= 220, rel + ": meta description is between 40 and 220 characters (found " + length + ")");
  }
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
  assert(twitterTitleCount === 1, rel + ": exactly one Twitter title");
  assert(twitterDescriptionCount === 1, rel + ": exactly one Twitter description");
  assert(ogImageCount <= 1, rel + ": at most one Open Graph image");
  assert(twitterImageCount <= 1, rel + ": at most one Twitter image");
  const twitterCard = getMetaContent(html, "name", "twitter:card");
  assert(["summary", "summary_large_image"].includes(twitterCard), rel + ": Twitter card uses a supported card type");
  const ogTitle = getMetaContent(html, "property", "og:title");
  const twitterTitle = getMetaContent(html, "name", "twitter:title");
  assert(Boolean(twitterTitle?.trim()), rel + ": Twitter title is not empty");
  if (ogTitle && twitterTitle) assert(decodeHtml(ogTitle) === decodeHtml(twitterTitle), rel + ": Twitter title matches Open Graph title");
  const ogDescription = getMetaContent(html, "property", "og:description");
  const twitterDescription = getMetaContent(html, "name", "twitter:description");
  if (ogDescription) {
    assert(Boolean(ogDescription.trim()), rel + ": Open Graph description is not empty");
    const length = decodeHtml(ogDescription).trim().length;
    assert(length >= 30 && length <= 220, rel + ": Open Graph description is between 30 and 220 characters (found " + length + ")");
  }
  assert(Boolean(twitterDescription?.trim()), rel + ": Twitter description is not empty");
  if (twitterDescription) {
    const length = decodeHtml(twitterDescription).trim().length;
    assert(length >= 30 && length <= 220, rel + ": Twitter description is between 30 and 220 characters (found " + length + ")");
  }
  const ogImage = getMetaContent(html, "property", "og:image");
  const twitterImage = getMetaContent(html, "name", "twitter:image");
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
  const ogUrl = getMetaContent(html, "property", "og:url");
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
        assert(/professional brewery teams/i.test(metaDescription || "") && /professional brewery teams/i.test(ogDescription || ""), "FAQ search and social descriptions lead with professional brewery teams");
        const faqIntro = html.match(/<section class=["']shell explain-page-hero["']>([\\s\\S]*?)<\\/section>/)?.[1] || "";
        assert(/primary audience/i.test(faqIntro) && /secondary path/i.test(faqIntro), "FAQ introduction states the Business-first audience hierarchy");
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
  const desktopNav = html.match(/<nav class=["']desktop-nav["'][^>]*>([\s\S]*?)<\/nav>/)?.[1] || "";
  const breweriesNavIndex = desktopNav.indexOf(">Breweries<");
  const homebrewersNavIndex = desktopNav.indexOf(">Homebrewers<");
  assert(breweriesNavIndex >= 0 && homebrewersNavIndex > breweriesNavIndex, rel + ": Professional Breweries precedes Homebrewers in desktop navigation");

  const headerCta = html.match(/<a class=["']header-cta["'][^>]*href=["']([^"']+)["']/)?.[1];
  if (pagePath !== "/breweries/") {
    const headerCtaPath = headerCta ? new URL(headerCta, pageUrl).pathname : "";
    assert(headerCtaPath.endsWith("/breweries/"), rel + ": primary header CTA leads to Professional Breweries");
  }

  if (pagePath === "/") {
    const primaryCta = html.match(/<a class=["']button button-primary["'][^>]*href=["']([^"']+)["']/)?.[1];
    const primaryCtaPath = primaryCta ? new URL(primaryCta, pageUrl).pathname : "";
    assert(primaryCtaPath.endsWith("/breweries/"), "homepage primary CTA leads to Professional Breweries");

    const featuredSlugs = [...html.matchAll(/href=["'][^"']*\/features\/([^/"']+)\/["']/g)].map((match) => match[1]);
    const expectedFeaturedSlugs = [
      "production-management",
      "quality-laboratory",
      "packaging-compliance",
      "recipe-design",
      "brewing-calculations",
      "water-mash-yeast",
    ];
    assert(
      expectedFeaturedSlugs.every((slug, index) => featuredSlugs[index] === slug),
      "homepage feature overview prioritises Business-only capabilities before shared capabilities",
    );
  }

  if (pagePath === "/how-it-works/") {
    const businessSectionIndex = html.indexOf('id="business-workflow"');
    const homeSectionIndex = html.indexOf('id="home-workflow"');
    const comparison = html.match(/<table class=["']role-table audience-comparison-table["']>([\s\S]*?)<\/table>/)?.[1] || "";
    const businessColumnIndex = comparison.indexOf(">Professional Breweries<");
    const homeColumnIndex = comparison.indexOf(">Homebrewers<");
    assert(businessSectionIndex >= 0 && homeSectionIndex > businessSectionIndex, "How It Works presents Professional Breweries before Homebrewers");
    assert(businessColumnIndex >= 0 && homeColumnIndex > businessColumnIndex, "audience comparison presents Professional Breweries first");
  }

  if (pagePath === "/features/") {
    assert(html.includes("professional brewery workflows") && html.includes("distinct Homebrewers path"), "product directory copy establishes Business-first hierarchy and separate Homebrewers path");
  }

  if (pagePath === "/faq/") {
    assert(html.includes("primary commercial audience") && html.includes("distinct secondary audience"), "FAQ states primary and secondary audience priority");
  }

  if (pagePath.startsWith("/features/")) {
    const featureTags = html.match(/<div class=["']feature-tags["']>([\s\S]*?)<\/div>/)?.[1] || "";
    const breweryTagIndex = featureTags.indexOf(">Breweries<");
    const homeTagIndex = featureTags.indexOf(">Homebrewers<");
    if (breweryTagIndex >= 0 && homeTagIndex >= 0) {
      assert(breweryTagIndex < homeTagIndex, rel + ": shared feature labels list Breweries before Homebrewers");
    }
  }
  const navTags = [...html.matchAll(/<nav\b([^>]*)>/g)].map((match) => match[1]);
  assert(navTags.length > 0 && navTags.every((attrs) => /aria-label=["'][^"']+["']|aria-labelledby=["'][^"']+["']/.test(attrs)), `${rel}: navigation landmarks have accessible names`);
  const imageTags = [...html.matchAll(/<img\b([^>]*)>/g)].map((match) => match[1]);
  assert(imageTags.every((attrs) => /\balt=["'][^"']*["']/.test(attrs)), `${rel}: every image has an alt attribute`);
  const elementTags = [...html.matchAll(/<([a-z][a-z0-9:-]*)\b([^>]*)>/gi)];
  const idsForAria = new Set(idList);
  for (const [, tagName, rawAttrs] of elementTags) {
    const attrMap = Object.fromEntries([...rawAttrs.matchAll(/([\w:-]+)\s*=\s*(?:"([^"]*)"|'([^']*)')/g)].map((m) => [m[1].toLowerCase(), m[2] ?? m[3] ?? ""]));
    for (const refAttr of ["aria-labelledby", "aria-describedby", "aria-controls", "aria-owns", "aria-details", "aria-errormessage"]) {
      if (!attrMap[refAttr]) continue;
      for (const refId of attrMap[refAttr].trim().split(/\s+/)) assert(idsForAria.has(refId), rel + ": " + tagName + " " + refAttr + " reference exists: " + refId);
    }
    if (tagName.toLowerCase() === "iframe") assert(Boolean(attrMap.title?.trim()), rel + ": iframe has a descriptive title");
    if (attrMap.target === "_blank") assert(/\bnoopener\b/.test(attrMap.rel || "") && /\bnoreferrer\b/.test(attrMap.rel || ""), rel + ": new-tab link protects opener and referrer");
  }
  const hasAccessibleName = (attrs, inner) => Boolean(
    /\baria-label(?:ledby)?\s*=|\btitle\s*=/.test(attrs) ||
    inner.replace(/<script[\s\S]*?<\/script>|<style[\s\S]*?<\/style>/gi, "")
      .replace(/<[^>]+>/g, " ").replace(/&nbsp;|&#160;/g, " ")
      .replace(/&[a-z#0-9]+;/gi, " ").replace(/\s+/g, " ").trim()
  );
  for (const tagName of ["a", "button", "summary"]) {
    const pattern = new RegExp("<" + tagName + "\\b([^>]*)>([\\s\\S]*?)<\/" + tagName + ">", "gi");
    for (const [, attrs, inner] of html.matchAll(pattern)) {
      assert(hasAccessibleName(attrs, inner), rel + ": " + tagName + " has a non-empty accessible name");
    }
  }
  for (const [, attrs] of html.matchAll(/<input\b([^>]*)>/gi)) {
    const type = attrs.match(/\btype=["']([^"']+)["']/i)?.[1]?.toLowerCase() || "text";
    if (type === "hidden") continue;
    const id = attrs.match(/\bid=["']([^"']+)["']/i)?.[1];
    const hasLabel = id ? [...html.matchAll(/<label\b([^>]*)>/gi)].some(([, labelAttrs]) => labelAttrs.match(/\bfor=["']([^"']+)["']/i)?.[1] === id) : false;
    const hasAria = /\baria-label(?:ledby)?\s*=/.test(attrs);
    const hasButtonValue = ["button", "submit", "reset"].includes(type) && /\bvalue=["'][^"']+["']/.test(attrs);
    assert(Boolean(hasLabel || hasAria || hasButtonValue), rel + ": input " + type + " has an accessible name");
  }
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
const sitemapDirective = robots.match(/^Sitemap:\s*(\S+)\s*$/im)?.[1];
assert(Boolean(sitemapDirective), "robots.txt references sitemap with a valid directive");
if (sitemapDirective) {
  try {
    const robotsSitemapUrl = new URL(sitemapDirective);
    assert(robotsSitemapUrl.origin === siteOrigin && robotsSitemapUrl.pathname === basePath + "/sitemap.xml", "robots.txt sitemap directive matches the generated sitemap URL");
  } catch { failures.push("robots.txt contains an invalid sitemap URL"); }
}
assert(/^User-agent:\s*\*/im.test(robots), "robots.txt declares the wildcard user agent");
assert(/^Allow:\s*\//im.test(robots), "robots.txt explicitly allows crawling");

console.log(`Static site validation: ${checks.length} checks passed, ${failures.length} failures.`);
if (failures.length) {
  for (const failure of failures) console.error(`FAIL: ${failure}`);
  process.exitCode = 1;
} else {
  console.log("Routes, sitemap targets, metadata, local links, anchors and legacy-claim checks passed.");
}
