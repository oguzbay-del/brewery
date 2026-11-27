# MQ-01 — Marketing site quality audit

Date: 27 November 2026. Scope: marketing-site only. Issue #450. Baseline read: main at `96dd9919f6aa0fea705ddc01ae445d0126d3fa7d` (MQ-00 plan merge). This is an initial source and CI evidence audit, not a completed visual acceptance review.

## 1. Evidence status

- [MQ-00 PR #449](https://github.com/oguzbay-del/brewery/pull/449): documentation-only change, no marketing runtime changes.
- [Marketing CI run 46061385625](https://github.com/oguzbay-del/brewery/actions/runs/46061385625) completed successfully at PR head `402cb04361247f199c830cb9b9a768298df9adf1`. Protected-path guard run 46061385657 passed. This is evidence for that PR head, not an assertion of a new current-main test. The current-main workflow lookup returned no runs.
- [ACCEPTANCE.md](../ACCEPTANCE.md): W12 and W27 contain historical test and screenshot evidence. W27 inspected the Home preview at 320, 390 and 1440 CSS pixels, reported a soft mobile photo crop and recorded mobile Lighthouse performance 86 versus 85 in W12. Do not treat that as a current measurement.
- Current EN/TR screenshots, their artifact contents, manual 200% zoom, keyboard and screen-reader results, and current Lighthouse reports were not inspected in this session. Visual scores and claims of observed clipping are therefore not assigned.

## 2. Route inventory

Source: `src/pages/` and `src/pages/tr/`.

| Page family | EN / TR routes | Current general responsive and axe matrix |
|---|---|---|
| Home | / and /tr/ | Both included |
| Audience | breweries, homebrewers | EN only |
| Workflow | how-it-works | EN only |
| Technical | technical-principles | EN only in main matrix; separate EN/TR calibration diagram keyboard test |
| Feature catalogue | features index and detail | EN index and recipe-design detail; separate navigation tests for both languages |
| Modules | modules index and detail directories | Neither in main matrix |
| Commercial preview | pricing, compare, get-started | EN only in main matrix |
| Help | faq | EN only in main matrix |
| Not found | 404.astro | Not in main matrix |

The main matrix in `tests/browser/responsive.spec.js` has 12 routes x 5 viewport widths = 60 combinations; `accessibility.spec.js` has the same 12 routes x 2 viewports = 24 automated axe combinations. Separate comparison/accessibility tests exist and need detailed review before asserting their additional coverage. Dynamic route generation and full route count remain to be checked from feature and module data and sitemap.

## 3. Reuse inventory

| Existing item | Proposed Marketing UI Kit role |
|---|---|
| src/components/SiteHeader.astro | Header and native mobile navigation |
| src/components/SiteFooter.astro | Footer |
| src/components/Breadcrumbs.astro | Breadcrumb |
| src/components/FontPreload.astro | Font loading, preserve optimization |
| src/components/PagePhoto.astro | Responsive editorial image |
| src/components/ProductScreens.astro | Product showcase; uses placements and screens from data/productScreens.ts; image intrinsic dimensions and lazy loading present |
| src/components/WorkflowLinks.astro | Workflow navigation; inspect props before changing |
| src/styles/global.css | Global palette, type, shell, controls, shared tokens |
| src/styles/stitch-layout.css | Existing editorial hero, section headings, cards, quotes, audience panels and CTA patterns |
| src/styles/home-hero-refinement.css | Homepage photo and responsive treatment |
| src/styles/audience-pages.css, learning-pages.css, faq-evidence.css | Page-family patterns |
| src/styles/pricing.css, comparison.css, modules.css | Commercial and catalogue patterns |
| src/data/features.ts, features.tr.ts, modules.ts, modules.tr.ts | Existing bilingual content and scope data; do not move into UI primitives |
| src/data/photos.ts, productScreens.ts | Image and screenshot metadata; use in MQ-08 and MQ-09 |

Start with these seven Astro components and nine CSS files. Do not replace the shared components with a competing library. Inspect their actual usages before extracting variants.

## 4. Token inventory (initial)

Source: `src/styles/global.css` and `src/styles/stitch-layout.css`.

| Role | Existing values |
|---|---|
| Brand | --forest #0B2F26; --page #F7F6F3; --muted #475569; --amber-fill #F4A261 |
| Accessible accent | --amber #8B4A1F for text; --amber-ink #0B2F26; --amber-tint #FBE9DA |
| Surface and border | --surface #FFFFFF; --soft #E8E7E3; --line rgba(11,47,38,.18) |
| Shape | --radius-card 18px; --radius-panel 24px; --radius-chip and --radius-control 999px |
| Typography | --sans defined as system fallback and later Inter Variable/Inter; --display Plus Jakarta Sans Variable/Plus Jakarta Sans; --serif Georgia/Times; --mono system mono |
| Layout | --content 1240px; .shell width min(var(--content), calc(100% - 64px)); responsive rules at multiple breakpoints |
| Editorial surfaces | --sx-band #F0EFEB; --sx-card #FFFFFF; --sx-soft #F4F3F0; --sx-forest-2 #123E33; --sx-on-dark #F7F6F3; --sx-on-dark-muted #C9D3CF |
| Existing heading | .sx-heading h2 clamp(36px, 4.2vw, 54px); display font and serif emphasis |
| Existing grid | .sx-hero-grid minmax(0,1.15fr) minmax(0,.85fr), gap 56px; .sx-card-grid gap 20px; .sx-audience-grid gap 24px |
| Existing section/CTA | .sx-hero 72/96px block padding; .sx-cta 96px; .sx-cta-inner 56px 64px padding, 48px gap |

MQ-03 must preserve semantic contrast distinctions, especially accessible amber text vs decorative fill. Inventory all declarations, cascade order and media overrides before choosing canonical spacing and breakpoints. These values are observed source declarations, not computed-style measurements.

## 5. Prioritized findings

| ID | Priority | Finding and evidence | Proposed action |
|---|---|---|---|
| A01 | High coverage gap | Main responsive and axe matrices cover only TR Home among TR routes; module routes and 404 are absent. Source: both browser specs. This is a test coverage defect, not proof of a rendering defect. | MQ-11: add representative EN/TR route families, modules and 404; keep existing assertions. |
| A02 | High coverage gap | Responsive suite checks document/body overflow, image loading, compact logo, browser errors and navigation. It captures screenshots but does not compare approved references or assert unintended overlap, clipped important text or section alignment. | MQ-11: targeted geometry report and reviewed screenshot baseline process. Do not modify protected baselines. |
| A03 | High verification gap | W12 manual keyboard, screen reader and 200% zoom review remains open in ACCEPTANCE.md. Current screenshots were not inspected. Existing 720px proxy is not actual browser zoom. | MQ-14: capture current evidence; do not report visual or accessibility acceptance from CI alone. |
| A04 | Medium maintainability | Global and Stitch styles already contain reusable tokens and patterns, but literal spacing values vary across heading, hero, card and CTA rules. Multiple breakpoints and two --sans definitions require cascade inventory. | MQ-03/06: consolidate only where beneficial; avoid broad CSS reset. |
| A05 | Medium design opportunity | ProductScreens uses shared placements and metadata; all images are lazy loaded in the observed component. Its current crops, captions and visual hierarchy have not been reviewed. | MQ-08: verify source assets, placement and content; pilot one readable product image with caption, no fabricated UI. |
| A06 | Medium performance/visual risk | Historical W27 evidence reports soft tall mobile hero crop and mobile Lighthouse 86, desktop 100. | MQ-09/13: current measurement and mobile focal point review; retain responsive AVIF/WebP/JPEG and avoid needless upscale. |
| A07 | High content boundary | #436 G8 remains the source for brand/claim audit. Existing marketing maturity copy must not be turned into unverified live claims during a visual redesign. | Isolate visual work from copy changes; coordinate G8 findings before release. |

No verified critical visual defect is claimed in this initial source audit.

## 6. Pilot requirements and next evidence

- MQ-02 defines the Linear + Aesop editorial/technical pattern. Use existing GRISTORA colors, logo, hierarchy and actual assets. Keep one primary brewery path and a distinct secondary homebrewer path.
- MQ-03 and MQ-04 reuse and extend existing CSS and Astro components. Preserve EN/TR route behavior, metadata, accessible controls and page layout.
- MQ-01 remains open until the latest applicable screenshot artifacts and current page evidence have been examined. Record exact commit, run, screenshot file, viewport and language; inspect both Home versions and representative route families. Check current Lighthouse and comparison test scope.
- Keep manual review findings distinct from automated results. If artifact retrieval is not possible, record the exact limitation and request a cloud preview review through the existing workflow. Do not invent measurements.

No product files, runtime marketing code, dependencies, production configuration or baseline images were changed by this report.

## 7. Follow-up CI and comparison evidence (27 November)

The audit branch at `b747c7c127d38088fcbff66a139ac9440e133fd3` changes only this report. Its [Marketing CI run 46062452542](https://github.com/oguzbay-del/brewery/actions/runs/46062452542) and protected-path-guard run 46062452417 passed. Cloudflare reported a successful branch preview at https://e5dc8415.gristora.pages.dev. The preview has not been visually inspected in this session.

Artifacts for that exact run (not current-main measurements):

| Evidence | Artifact |
|---|---|
| Responsive screenshots | [15229316834](https://github.com/oguzbay-del/brewery/actions/runs/46062452542/artifacts/15229316834), 31,203,924 bytes |
| Lighthouse reports | [15230140125](https://github.com/oguzbay-del/brewery/actions/runs/46062452542/artifacts/15230140125), 5,057,842 bytes |
| Social card | [15229775994](https://github.com/oguzbay-del/brewery/actions/runs/46062452542/artifacts/15229775994), 59,560 bytes |

The artifact listing confirms existence and provenance, not visual or numerical results. Review the contents before closing MQ-01.

Additional browser source checks: `tests/browser/compare-table-accessibility.spec.js` tests the EN comparison scroll region at 320 and 390px with visible keyboard focus, ArrowRight scrolling and no page overflow. `comparison.spec.js` tests EN plan entry links at 390px and intentional table scrolling at 320px; it also tests row-group headings on both EN and TR comparison pages. Preserve these intentional scroll regions in VQG, and extend TR keyboard and responsive coverage rather than replacing the existing tests. No additional rendering defect is inferred from this coverage inventory.

## 8. Actual artifact inspection (27 November 2026)

Evidence: [marketing CI 46063650916](https://github.com/oguzbay-del/brewery/actions/runs/46063650916), head `1187435b55dd9163d87f208c1cead92a11fd77d0`. Guard passed. Cloudflare reported successful preview. Downloaded screenshot artifact [15230321111](https://github.com/oguzbay-del/brewery/actions/runs/46063650916/artifacts/15230321111) and Lighthouse artifact [15229926431](https://github.com/oguzbay-del/brewery/actions/runs/46063650916/artifacts/15229926431). The branch changes documentation only; evidence is for that build, not an independently measured production page.

### Screenshot review

The archive contains 36 PNG files. Inspected contact sheets of the top sections of all 12 captured routes at 390 and 1440px, and the EN/TR homepage at 320, 768, 1024px. Contact sheets are scaled crops of actual CI PNGs, not complete page review. The 320px crop does not include the complete hero actions; no visibility assertion for them from that crop. No Turkish subpage screenshots beyond Home exist in this archive.

| Finding | Evidence | Severity / next |
|---|---|---|
| Soft image detail in tall mobile homepage crop compared with desktop | home-320.png, tr-320.png, home-390.png, tr-390.png, home-1440.png, tr-1440.png | P2 visual improvement; MQ-09 review focal point and image source sizes without degrading transfer |
| Long TR hero headline occupies an additional line at 390 and 1440px versus EN; both top sections remain readable in the inspected crops | home-390.png, tr-390.png, home-1440.png, tr-1440.png | Layout test case, not a defect by itself; preserve natural wrapping |
| Audience heroes have solid Forest Green intro sections, while other reviewed subpage intros use paper surfaces | breweries-390.png, homebrewers-390.png, compare-390.png, pricing-390.png and 1440 counterparts | P2 consistency review for Editorial Light; brand hero is permitted, no recurring dark technical panels |
| Existing feature detail, FAQ, pricing and workflow intros already express the approved warm paper + forest ink + serif hierarchy | features-recipe-design-1440.png, faq-1440.png, pricing-1440.png, how-it-works-1440.png | Reuse rather than redesign from scratch |
| Development/roadmap wording appears in hero and feature introductions | home-390.png, tr-390.png, features-390.png, features-recipe-design-390.png | Content evidence for G8; do not change claims in visual task |
| Missing representative TR subpage, module and 404 screenshots | Archive file list | Coverage gap, not an observed layout defect; MQ-11a |

No obvious major overlap or clipping in inspected top sections at 390/1440px. This is not proof of full-page geometry, interactive state, keyboard, zoom or accessibility acceptance. Full-page lower sections and audience intermediate widths still need review; do not close the task solely on this subset.

### Lighthouse JSON measurements

All ten reports scored 100 in Accessibility, Best Practices and SEO. Lab measurements at localhost (not production field metrics):

| Page | Device | Performance | LCP (s) | TBT (ms) | CLS |
|---|---|---:|---:|---:|---:|
| Home EN | Desktop | 100 | 0.51 | 0 | 0 |
| Home EN | Mobile | 90 | 2.03 | 384 | 0 |
| Home TR | Desktop | 100 | 0.61 | 0 | 0 |
| Home TR | Mobile | 95 | 2.78 | 0 | 0 |
| Compare EN | Desktop | 100 | 0.45 | 0 | 0 |
| Compare EN | Mobile | 100 | 1.88 | 0 | 0 |
| How it works EN | Desktop | 100 | 0.45 | 0 | 0 |
| How it works EN | Mobile | 100 | 1.88 | 0 | 0 |
| Pricing EN | Desktop | 100 | 0.41 | 0 | 0 |
| Pricing EN | Mobile | 100 | 1.80 | 0 | 0 |

Source: corresponding *.report.json entries. EN mobile TBT and TR mobile LCP require repeat measurement and root cause analysis in MQ-13; the lab LCP goal is 2.5s. TBT is not INP. The single-run performance difference does not establish a regression or cause. Keep the existing Lighthouse CI.

Next: review lower sections and missing matrix coverage using MQ-11a report evidence, capture current baseline where possible and finish MQ-01. Do not claim screen-reader or 200% actual zoom review. No runtime or production changes.
