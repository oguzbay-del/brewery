import { test, expect } from "@playwright/test";
import { readFile, writeFile } from "node:fs/promises";
import AxeBuilder from "@axe-core/playwright";

// MQ-11a: report mode. Existing responsive and accessibility gates stay hard.
const widths = [320, 390, 768, 1024, 1440];
const families = [
  "", "breweries/", "homebrewers/", "features/", "features/recipe-design/",
  "modules/", "modules/#core", "how-it-works/", "technical-principles/", "faq/",
  "pricing/", "compare/", "get-started/",
];
const routes = ["en", "tr"].flatMap((locale) =>
  families.map((family) => ({
    locale,
    path: (locale === "tr" ? "/tr/" : "/") + family,
  }))
);
routes.push({ locale: "en", path: "/mq-visual-missing/", missing: true });

// Scope overlap to sibling cards in known layout groups. Do not compare arbitrary
// elements, since a parent, its content and intended overlays can intersect.
async function inspectGeometry(page) {
  return page.evaluate(() => {
    const findings = [];
    const tolerance = 1;
    const viewport = document.documentElement.clientWidth;
    const identify = (el) => {
      const parts = [];
      for (let node = el; node && node !== document.body; node = node.parentElement) {
        if (node.id) {
          parts.unshift("#" + CSS.escape(node.id));
          break;
        }
        const index = Array.from(node.parentElement?.children || []).indexOf(node) + 1;
        parts.unshift(node.localName + ":nth-child(" + index + ")");
      }
      return parts.join(" > ");
    };
    const visible = (el) => {
      const style = getComputedStyle(el);
      const box = el.getBoundingClientRect();
      return style.visibility === "visible" && style.display !== "none" &&
        box.width > tolerance && box.height > tolerance && !el.closest("[hidden], [inert]");
    };
    const report = (rule, el, expected, actual, severity = "P1") => {
      findings.push({ rule, selector: identify(el), expected, actual, severity });
    };
    // Existing labelled scroll regions intentionally allow wider content.
    const intentionalScroll = (el) => {
      for (let node = el.parentElement; node && node !== document.body; node = node.parentElement) {
        if (node.matches('.comparison-table-wrap, .role-table-wrap, .visual-diagram-scroll') &&
            ["auto", "scroll"].includes(getComputedStyle(node).overflowX)) return true;
      }
      return false;
    };
    for (const el of [document.documentElement, document.body]) {
      if (el.scrollWidth > viewport + tolerance) {
        report("page-overflow", el, "scrollWidth <= viewport + 1", el.scrollWidth);
      }
    }
    for (const el of document.querySelectorAll("main h1, main h2, main h3, main a, main button, main input, main select, main textarea, main summary, main img")) {
      if (!visible(el) || intentionalScroll(el)) continue;
      const box = el.getBoundingClientRect();
      const style = getComputedStyle(el);
      if (box.left < -tolerance || box.right > viewport + tolerance) {
        report("outside-viewport", el, "horizontal bounds within viewport", {
          left: Math.round(box.left), right: Math.round(box.right), viewport,
        });
      }
      if (["hidden", "clip"].includes(style.overflowX) &&
          el.clientWidth > 0 && el.scrollWidth > el.clientWidth + tolerance &&
          el.tagName !== "IMG" && el.tagName !== "INPUT") {
        report("horizontal-clipping", el, "scrollWidth <= clientWidth + 1", {
          scrollWidth: el.scrollWidth, clientWidth: el.clientWidth,
        });
      }
      if (["hidden", "clip"].includes(style.overflowY) &&
          el.clientHeight > 0 && el.scrollHeight > el.clientHeight + tolerance &&
          el.tagName !== "IMG" && el.tagName !== "INPUT") {
        report("vertical-clipping", el, "scrollHeight <= clientHeight + 1", {
          scrollHeight: el.scrollHeight, clientHeight: el.clientHeight,
        });
      }
      if (el.tagName === "IMG" && (!el.complete || el.naturalWidth === 0)) {
        report("image-load", el, "loaded image", { src: el.currentSrc, complete: el.complete });
      }
      // A fill image can be stretched. Cover/contain intentionally preserve ratio.
      if (el.tagName === "IMG" && el.complete && el.naturalWidth > 0 &&
          el.naturalHeight > 0 && style.objectFit === "fill" &&
          el.clientWidth > 0 && el.clientHeight > 0) {
        const intrinsic = el.naturalWidth / el.naturalHeight;
        const displayed = el.clientWidth / el.clientHeight;
        if (Math.abs(displayed / intrinsic - 1) > 0.05) {
          report("image-aspect-review", el, "Intrinsic aspect ratio within 5% when object-fit is fill", {
            intrinsic, displayed, src: el.currentSrc,
          }, "P2");
        }
      }
      // Check meaningful text and controls against clipping ancestors. Do not
      // classify intended image cropping or labelled horizontal scroll as defects.
      if (el.tagName !== "IMG") {
        for (let parent = el.parentElement; parent && parent !== document.body; parent = parent.parentElement) {
          const parentStyle = getComputedStyle(parent);
          const parentBox = parent.getBoundingClientRect();
          const left = parentBox.left + parent.clientLeft;
          const top = parentBox.top + parent.clientTop;
          const clipX = ["hidden", "clip"].includes(parentStyle.overflowX);
          const clipY = ["hidden", "clip"].includes(parentStyle.overflowY);
          if ((clipX && (box.left < left - tolerance || box.right > left + parent.clientWidth + tolerance)) ||
              (clipY && (box.top < top - tolerance || box.bottom > top + parent.clientHeight + tolerance))) {
            report("ancestor-clipping-review", el, "Meaningful element fits clipping ancestor", {
              ancestor: identify(parent), clipX, clipY,
            }, "P2");
            break;
          }
        }
      }
    }
    for (const group of document.querySelectorAll(".sx-card-grid, .sx-audience-grid")) {
      const children = Array.from(group.children).filter(visible);
      for (let i = 0; i < children.length; i++) {
        const a = children[i].getBoundingClientRect();
        for (let j = i + 1; j < children.length; j++) {
          const b = children[j].getBoundingClientRect();
          const x = Math.min(a.right, b.right) - Math.max(a.left, b.left);
          const y = Math.min(a.bottom, b.bottom) - Math.max(a.top, b.top);
          if (x > tolerance && y > tolerance) {
            report("sibling-overlap", children[i], "no intersecting card interiors", {
              other: identify(children[j]), overlapWidth: Math.round(x), overlapHeight: Math.round(y),
            });
          }
        }
      }
    }
    // Compare only direct, visible children of known CTA wrappers. Mixed
    // typography, button widths and intentional centered text are not defects.
    for (const group of document.querySelectorAll(".sx-cta-inner")) {
      const children = Array.from(group.children).filter(visible);
      const bounds = group.getBoundingClientRect();
      const style = getComputedStyle(group);
      const left = bounds.left + group.clientLeft + parseFloat(style.paddingLeft);
      const right = bounds.left + group.clientLeft + group.clientWidth - parseFloat(style.paddingRight);
      for (const child of children) {
        const box = child.getBoundingClientRect();
        if (box.left < left - tolerance || box.right > right + tolerance) {
          report("cta-alignment-review", child, "CTA child fits its content column", {
            group: identify(group), left: Math.round(box.left), right: Math.round(box.right),
            expectedLeft: Math.round(left), expectedRight: Math.round(right),
          }, "P2");
        }
      }
    }
    // Section heading bounds, not arbitrary identical text alignment: allow
    // center-aligned headings and editorial columns within the section.
    for (const heading of document.querySelectorAll(".section-heading, .sx-heading")) {
      const parent = heading.parentElement;
      if (!parent || !visible(heading) || !visible(parent)) continue;
      const a = heading.getBoundingClientRect();
      const b = parent.getBoundingClientRect();
      if (a.left < b.left - tolerance || a.right > b.right + tolerance) {
        report("section-alignment-review", heading, "Heading fits its section", {
          section: identify(parent), left: Math.round(a.left), right: Math.round(a.right),
          sectionLeft: Math.round(b.left), sectionRight: Math.round(b.right),
        }, "P2");
      }
    }
    return findings;
  });
}

async function provenance() {
  let headSha = null;
  if (process.env.GITHUB_EVENT_PATH) {
    const event = JSON.parse(await readFile(process.env.GITHUB_EVENT_PATH, "utf8"));
    headSha = event.pull_request?.head?.sha || event.after || null;
  }
  return {
    headSha: headSha || process.env.GITHUB_SHA || null,
    testedSha: process.env.GITHUB_SHA || null,
    runId: process.env.GITHUB_RUN_ID || null,
  };
}

test("MQ-11a detects an intentional geometry defect in an isolated fixture", async ({ page }) => {
  await page.setViewportSize({ width: 320, height: 900 });
  await page.setContent('<main><h1 style="width:500px">Wide heading</h1><div class="sx-card-grid" style="display:grid"><article style="grid-area:1/1;width:100px;height:100px">One</article><article style="grid-area:1/1;width:100px;height:100px">Two</article></div><h2 style="width:50px;overflow:hidden;white-space:nowrap">Unintentionally clipped text</h2></main>');
  const findings = await inspectGeometry(page);
  expect(findings.some((f) => f.rule === "page-overflow")).toBe(true);
  expect(findings.some((f) => f.rule === "sibling-overlap")).toBe(true);
  expect(findings.some((f) => f.rule === "horizontal-clipping")).toBe(true);
  await page.setContent('<main><div style="width:50px;height:30px;overflow:hidden"><button style="width:120px;height:50px">Clipped control</button></div></main>');
  const clipped = await inspectGeometry(page);
  expect(clipped.some((f) => f.rule === "ancestor-clipping-review")).toBe(true);
  await page.setContent('<main><div class="sx-cta-inner" style="width:200px;padding:20px"><div style="width:300px">CTA</div></div><section style="width:100px"><div class="section-heading" style="width:150px">Heading</div></section></main>');
  const alignment = await inspectGeometry(page);
  expect(alignment.some((f) => f.rule === "cta-alignment-review")).toBe(true);
  expect(alignment.some((f) => f.rule === "section-alignment-review")).toBe(true);
});

const cases = routes.flatMap((route) => widths.map((width) => ({
  route, width, state: "default",
})));
for (const route of routes.filter((r) => r.path === "/" || r.path === "/tr/" || r.path.endsWith("/faq/"))) {
  for (const width of [390, 1440]) {
    for (const state of ["forced-colors", "reduced-motion", "keyboard-focus"]) {
      cases.push({ route, width, state });
    }
  }
}

for (const route of routes.filter((r) => r.path === "/" || r.path === "/tr/")) {
  for (const width of [320, 390]) {
    cases.push({ route, width, state: "mobile-menu-open" });
  }
}

for (const { route, width, state } of cases) {
    test(`MQ-11a visual report ${route.locale} ${route.path} ${width}px ${state}`, async ({ page, browser }, testInfo) => {
      test.setTimeout(60_000);
      await page.setViewportSize({ width, height: 900 });
      await page.emulateMedia({
        reducedMotion: state === "reduced-motion" ? "reduce" : "no-preference",
        forcedColors: state === "forced-colors" ? "active" : "none",
        colorScheme: "light",
      });
      const response = await page.goto(route.path, { waitUntil: "load" });
      if (route.missing) expect(response?.status()).toBe(404);
      else expect(response?.ok()).toBeTruthy();
      // Module details are anchored cards, not separate pages.
      if (route.path.endsWith("#core")) await expect(page.locator("article#core")).toBeVisible();
      await page.evaluate(async () => {
        await document.fonts.ready;
        await Promise.allSettled(Array.from(document.images, (img) => {
          img.loading = "eager";
          return img.decode();
        }));
      });
      if (state === "mobile-menu-open") {
        const menu = page.locator("details.mobile-menu");
        await menu.locator("summary").click();
        await expect(menu).toHaveAttribute("open", "");
        await expect(menu.locator("nav")).toBeVisible();
      }
      const findings = await inspectGeometry(page);
      if (state === "mobile-menu-open") {
        const menuBounds = await page.locator("details.mobile-menu nav").boundingBox();
        if (!menuBounds || menuBounds.x < -1 || menuBounds.x + menuBounds.width > width + 1) {
          findings.push({
            rule: "mobile-menu-bounds", selector: "details.mobile-menu nav",
            expected: "Expanded menu stays within horizontal viewport bounds",
            actual: menuBounds, severity: "P1",
          });
        }
      }
      if (state === "keyboard-focus") {
        await page.keyboard.press("Tab");
        const focus = await page.evaluate(() => {
          const el = document.activeElement;
          if (!el || el === document.body) return null;
          const box = el.getBoundingClientRect();
          const style = getComputedStyle(el);
          return {
            selector: el.id ? "#" + CSS.escape(el.id) : el.localName,
            focusVisible: el.matches(":focus-visible"),
            outlineStyle: style.outlineStyle,
            outlineWidth: style.outlineWidth,
            boxShadow: style.boxShadow,
            bounds: { left: box.left, right: box.right, top: box.top, bottom: box.bottom },
          };
        });
        if (!focus || !focus.focusVisible || focus.bounds.right <= 0 ||
            focus.bounds.left >= width || focus.bounds.bottom <= 0 || focus.bounds.top >= 900) {
          findings.push({
            rule: "keyboard-focus-visibility", selector: focus?.selector || "document",
            expected: "First keyboard focus is visible inside the viewport",
            actual: focus, severity: "P1",
          });
        }
        // A custom focus treatment may be valid; this is a review hint only.
        if (focus && (focus.outlineStyle === "none" || focus.outlineWidth === "0px") &&
            focus.boxShadow === "none") {
          findings.push({
            rule: "keyboard-focus-indicator-review", selector: focus.selector,
            expected: "Visible focus treatment, including custom indicators",
            actual: focus, severity: "P2",
          });
        }
      }
      // Existing axe assertions stay unchanged. New route coverage is report-only.
      if (width === 390 || width === 1440) {
        const result = await new AxeBuilder({ page })
          .withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa"]).analyze();
        for (const violation of result.violations) {
          for (const node of violation.nodes) {
            findings.push({
              rule: "axe:" + violation.id,
              selector: JSON.stringify(node.target),
              expected: violation.help,
              actual: node.failureSummary || violation.description,
              severity: ["critical", "serious"].includes(violation.impact) ? "P1" : "P2",
            });
          }
        }
      }
      const name = "mq11a-" + route.locale + "-" +
        (route.path.replace("#", "").split("/").filter(Boolean).join("-") || "home") +
        "-" + width + "-" + state;
      const screenshot = name + ".png";
      await page.screenshot({
        path: testInfo.outputPath(screenshot),
        fullPage: width === 390 || width === 1440,
        animations: "disabled", caret: "hide", scale: "css",
      });
      const evidence = {
        mode: "report-only", ...await provenance(), route: route.path,
        locale: route.locale, viewport: { width, height: 900 },
        browser: testInfo.project.name, browserVersion: browser.version(),
        state, screenshot, findings,
        limitations: [
          "Not a visual acceptance pass",
          "Labelled comparison, role and diagram scroll regions excluded from element checks",
          "Sibling overlap limited to card grids; intended overlays are not compared",
          "Image bounds require human review for intentional frame crops",
          "Alignment checks are limited to CTA children and known section headings",
          "Deduplicate across cases by cause; raw observations are retained",
        ],
      };
      await writeFile(testInfo.outputPath(name + ".json"), JSON.stringify(evidence, null, 2));
      // Current workflow archives PNG only. Log each report for traceable evidence
      // until a protected workflow change permits JSON artifact upload.
      console.log("MQ11A_REPORT " + JSON.stringify(evidence));
    });
}
