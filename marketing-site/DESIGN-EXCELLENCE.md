# GRISTORA Marketing Design Excellence Program (MQ)

Owner direction — 27 November 2026. Linear and Aesop are the two primary inspiration references. The owner approved an independent Marketing UI Kit and Marketing Visual Quality Gate (VQG). This program applies only to the public marketing website, not the product web app or iPad app.

## 1. Scope and authority

Read [DESIGN.md](DESIGN.md), [BRAND-ASSETS.md](BRAND-ASSETS.md), [PHOTO-ASSETS.md](PHOTO-ASSETS.md), [ACCEPTANCE.md](ACCEPTANCE.md), [CHATGPT-SCOPE.md](CHATGPT-SCOPE.md), and [claims guardrails](../web-stratejisi/41-gristora-public-marketing-claims-guardrails.md). This plan extends the marketing design rules; it does not replace approved brand assets, release approvals, or claims guardrails. Follow the latest owner decisions. If any maturity wording is stale, reconcile it with the verified G8 report (#436) and the owner decisions before changing it. Do not claim live functionality on the basis of a screenshot, an API, or a roadmap entry alone.

Use Astro, TypeScript, native CSS and existing scripts. No new framework, external dependency, paid service, analytics, or new deployment path. Do not change web/, ios/, tasarim-sistemi/, product contracts, engine, auth, DNS or production settings. Class C workflow and snapshot-baseline changes require Claude review under the loop rules. No automatic baseline updates or merge. Do not publish a production release without owner approval.

## 2. Art direction

- [Linear](https://linear.app/): study its product-led hierarchy, short technical explanations, real product context, and focused progression from one capability to the next. Do not copy its logo, code, product visuals, dark palette or distinctive graphical assets.
- [Aesop](https://www.aesop.com/): study quiet editorial rhythm, typography, photographic composition, restraint and material detail. Do not copy its imagery, typography assets, branding, copy, or storefront structure.
- GRISTORA: premium, confident, editorial, technical and human. Keep Forest Green #0B2F26, Amber Gold #F4A261, Stone Beige #F7F6F3 and Slate Gray #475569. Keep approved logo geometry, Plus Jakarta Sans, Inter and restrained editorial serif treatment. Keep professional breweries primary and homebrewers secondary. No generic dark dashboard look, decorative glow, excessive motion or unnecessary redesign.
- Start with a homepage pilot after audit and shared foundations. Use one focal point per section, purposeful negative space, concise heading/copy pairs, clear primary/secondary CTA hierarchy, authentic brewery photography and actual product screenshots. Alternate editorial and technical sections. Preserve existing useful stories, routes, SEO and translated meaning. Do not invent commercial outcomes or functionality. References are inspiration, not templates or permission to reuse assets.

## 3. Marketing foundations

Inventory existing Astro components, global and Stitch CSS, page classes, tokens, image utilities, tests and route templates before adding code. Extend existing CSS custom properties rather than introduce another token source. Preserve approved radii (10–12, 18, 24, pill) and colors. Audit existing spacing/width values first; proposed 4/8/16/24/32/48/64/96 spacing scale and 12/8/4 conceptual desktop/tablet/mobile grid are candidates, not a mandatory rewrite. Define shared container, gutters, type scale, line length, layout density, focus, layering and reduced-motion behavior. Use CSS Grid, Flexbox, minmax(0,1fr), min-inline-size:0, logical properties, clamp and container queries only when useful. No clipping to conceal errors.

Create the marketing-only Astro UI Kit using existing components first. Proposed groups: Header/Footer/Breadcrumb/MobileMenu; Hero/SectionHeading/EditorialQuote; PhotoFrame/ImageGrid/ProductShowcase; FeatureCard/FeatureGrid/FeatureSpotlight; WorkflowSteps/Timeline/SplitSection; PricingCard/PlanSelector/ComparisonTable; CTA/Accordion/Callout/Disclaimer. Define variants, states, accessible semantics, short/long EN/TR text, image ratios and narrow layouts. Prefer native HTML for interactive controls. Keep gallery as a separate development build or dev-only fixture, absent from production dist, sitemap and navigation. No Storybook, React Aria or product UI Kit imports.

## 4. Screenshot and photography register

Audit ProductScreens and its data source and reuse current repository screenshots only. Keep English product screenshots as the initial shared images for EN/TR marketing, unless the owner later approves localized assets. Label language where needed. Do not alter product screenshots to depict unsupported features. Keep a marketing-only register with columns: asset key, source path, source commit/date, platform (web/iPad), screen and verified feature, marketing routes and sections, language, crop/aspect, alt EN/TR, last review, next refresh trigger, approval/evidence. Use existing image metadata as source where possible; avoid duplicate inventories. On request to refresh product screenshots, use this register to identify every placement. No changes to product screens to make screenshots.

Retain raw photo originals; use the existing optimization pipeline and AVIF/WebP/JPEG fallbacks. Record rights/source, crop, focal point, dimensions, alt text and usage. Use authentic editorial imagery, not customer proof. Keep large heroes readable; confirm desktop and mobile crops and reserve image space. No downloads of third-party reference site assets.

## 5. Delivery packages

The table is the design plan, not the live task board. GitHub issue labels determine state. Create a separate issue for each package (or a documented chain of dependent steps), with assigned-chatgpt, area:marketing and one status label. Check for overlap with existing W12 work, G8 #436, and UIQ product tasks. Reuse existing checks and evidence; do not create duplicate work. Dispatch follow-ups when prerequisites are met.

| ID | Priority | Package and scope | Depends on | Done when |
|---|---|---|---|---|
| MQ-01 | P0 | Current-site audit, route/component/style inventory and defect severity | MQ-00 | Current-main EN/TR desktop/mobile screenshots and existing test evidence; prioritized defects with source locations, no invented visual observations |
| MQ-02 | P0 | Linear + Aesop reference study and GRISTORA art direction | MQ-00 | Reference notes with design rules, original layout sketches and pilot acceptance criteria; no copied assets |
| MQ-03 | P0 | Marketing token inventory and standards | MQ-01, MQ-02 | Shared CSS variable map; consistent approved colors/radii/type; responsive spacing, container and layer rules; no product token edits |
| MQ-04 | P0 | Marketing Astro UI Kit | MQ-03 | Reuse existing components, document variants and EN/TR states; initial core components pass accessible and responsive checks |
| MQ-05 | P1 | Development-only component gallery | MQ-04 | Representative component fixtures at target widths; gallery excluded from production output |
| MQ-06 | P1 | Grid, typography and adaptive layout | MQ-03, MQ-04 | Shared layout utilities and section patterns, no page overflow; long TR text and 200% zoom remain usable |
| MQ-07 | P1 | Homepage pilot EN/TR | MQ-02, MQ-04, MQ-06, MQ-08, MQ-09 | Current story retained, technical screenshots and editorial imagery integrated; before/after preview, owner visual approval before sitewide rollout |
| MQ-08 | P1 | Product screenshot presentation and register | MQ-01 | Verified source and placements for every screenshot, actual image references, accessible captions and refresh instructions |
| MQ-09 | P1 | Photo art direction and composition | MQ-01, MQ-02 | Existing approved image inventory and responsive crop rules; no new unlicensed image or performance regression |
| MQ-10 | P1 | Rollout across audience pages, feature/module index/detail, workflow, technical, FAQ, pricing, compare, configurator, 404 | MQ-07 approval | EN/TR page families use shared patterns, retain route/metadata and claim boundaries; batches of small reviewable PRs |
| MQ-11 | P1 | Marketing VQG | MQ-01, MQ-04 | Existing Playwright/axe/Lighthouse extended with targeted geometry and visual report; no weaker checks; baseline/workflow changes separately reviewed |
| MQ-12 | P1 | Designer Review Gate | MQ-02, MQ-07 | Evidence-based scorecard and review of pilot; no critical defect accepted by score |
| MQ-13 | P2 | Performance review and optimization | MQ-10, MQ-11 | Compare current and previous builds under same conditions; investigate regressions, image/font/JS and layout stability |
| MQ-14 | P2 | Final acceptance and handoff | MQ-10–13 | Current-main route and language evidence, all required CI and manual checks, no open critical/high visual defects; update ACCEPTANCE.md with limitations |

MQ-01 and MQ-02 may run independently. MQ-08 and MQ-09 may run in parallel after prerequisites if they touch different files. Keep shared tokens, components and CSS changes serial. At most two parallel subagents if available; otherwise sequential. Use the lowest sufficient model. Do not change product UIQ task ownership or schedule. A pilot approval does not authorize a production release.

## 6. Marketing Visual Quality Gate

Use existing commands documented in README: npm run check, npm run build, npm run validate:site, npm run test:browser and current Lighthouse CI. Validate 320, 390, 768, 1024, 1440 CSS px and 200% zoom. Include EN/TR on representative routes and all changed page families. Keep mobile navigation, keyboard/focus, screen-reader manual checks and reduced motion. The iPad-only product support matrix does not apply to marketing.

Add targeted geometry checks for page overflow, nonintentional overlap, clipped meaningful text/controls, image ratios, CTA visibility and grid alignment. Exclude documented intended overlays, decorative layers and labelled scroll containers. Use deterministic screenshots (fixed browser, fonts, data, viewport and reduced motion). First report mode, then enforce only stable assertions. Baselines are protected; no automatic update or weakened thresholds. Visual evidence includes current SHA, preview URL, viewport, language and artifact location. Use native Playwright, axe WCAG 2.2 AA tags, current Lighthouse reports. Target LCP <=2.5s, INP <=200ms and CLS <=0.1 at field p75 when field data exists; lab Lighthouse is not field INP. No tracking or new paid service. CI results and preview review are separate from production readiness.

## 7. Designer Review Gate

Review each changed page against the approved direction with evidence, not a self-assigned score. Proposed rubric: brand 15, hierarchy 15, typography 10, grid/alignment 10, photography 10, product presentation 10, responsive 10, story 10, interaction/accessibility 10. Rate each 0–5, weighted sum out of 100; target 90 after a real review. Any critical accessibility, claims or layout error blocks acceptance independently of score. Record reviewer, date, commit, before/after screenshots, and required changes. Owner approval of homepage pilot is needed before broad rollout. Existing automated checks remain required.

## 8. Work and handoff

Use GitHub Issues and one branch/PR per task, following CHATGPT-SCOPE and loop v2. No direct push to main. Merge class A only after current required CI passes, no conflict or fix request; protected baseline/workflow edits require Claude approval. All changes remain marketing-only. Use Cloudflare branch previews for comparison; do not change production settings. Document what changed, evidence and remaining risks in every PR. Link this plan from README; do not use this document as a replacement for live issue labels.

First work after plan: MQ-01 current-site audit and MQ-02 art direction. Finish the G8 report or explicitly isolate its claims findings before changing copy. The owner reviews the homepage pilot, then proceed with phased rollout. Keep W12 acceptance work integrated rather than duplicating it.
