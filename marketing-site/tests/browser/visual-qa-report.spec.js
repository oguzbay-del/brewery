import { test, expect } from "@playwright/test";
import { readFile, writeFile } from "node:fs/promises";
import AxeBuilder from "@axe-core/playwright";

// MQ-11a: report mode. Existing responsive and accessibility gates stay hard.
const widths = [320, 390, 768, 1024, 1440];
const families = [
  "", "breweries/", "homebrewers/", "features/", "features/recipe-design/",
  "modules/", "how-it-works/", "technical-principles/", "faq/",
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
      await page.evaluate(async () => {
        await document.fonts.ready;
        await Promise.allSettled(Array.from(document.images, (img) => {
          img.loading = "eager";
          return img.decode();
        }));
      });
      const findings = await inspectGeometry(page);
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
        (route.path.split("/").filter(Boolean).join("-") || "home") +
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
        limitations: ["Not a visual acceptance pass", "Intentional scroll regions excluded from element checks"],
      };
      await writeFile(testInfo.outputPath(name + ".json"), JSON.stringify(evidence, null, 2));
      // Current workflow archives PNG only. Log each report for traceable evidence
      // until a protected workflow change permits JSON artifact upload.
      console.log("MQ11A_REPORT " + JSON.stringify(evidence));
    });
}
