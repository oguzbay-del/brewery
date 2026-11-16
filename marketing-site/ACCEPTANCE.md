# GRISTORA marketing website — W12 final acceptance

**Status:** In progress  
**Scope:** `marketing-site/` only  
**Baseline:** `main` at `f277d1bff4debb9259aa7855a9613e1b71dac60e` (16 November 2026)  
**Production:** Intentionally deferred; this gate validates the noindex preview only.

This is the current release-readiness record for W12. A previous W12 preview artifact was built from a branch that was seven commits behind `main`, before the final Stitch visual refinement and locally bundled fonts. Do not use that older screenshot set as visual approval for the current site. This branch exists to run the acceptance workflow against the current `main` baseline.

## 1. Automated acceptance

Run on this branch and record the resulting GitHub Actions run:

- [ ] Astro / TypeScript diagnostics: `npm run check`.
- [ ] Static build and generated 1200 × 630 social card: `npm run build`.
- [ ] Generated-route, metadata, structured-data, sitemap, robots, image and internal-link validation: `npm run validate:site`.
- [ ] Responsive Playwright and axe checks: `npm run test:browser`.
- [ ] Review the uploaded responsive screenshots and social-card artifact from this exact run.

The current validator checks required routes, sitemap coverage and uniqueness, canonical origin, preview-vs-production robots policy, page titles/descriptions, Open Graph and Twitter metadata, JSON-LD, breadcrumb structure, approved public brand, product-maturity disclosures, accessible landmarks/names, image alt attributes, ARIA references, duplicate IDs, primary audience ordering, local links and fragment targets, and 404 navigation.

## 2. Visual review — current Stitch direction

Review screenshots from the current run, not a stale branch artifact.

- [ ] Homepage at 1440, 1024, 768, 390 and 320 px.
- [ ] Professional Breweries and Homebrewers at desktop, tablet and mobile widths.
- [ ] Pricing, comparison and plan configurator at desktop and mobile widths.
- [ ] How It Works, Technical Principles, FAQ and feature directory/detail at desktop and mobile widths.
- [ ] Confirm the approved GRISTORA logo remains intact; forest-green / stone-beige / amber palette, Plus Jakarta Sans / Inter typography, light editorial hero, rounded panels and pill controls are consistent with `DESIGN.md`.
- [ ] Confirm Professional Breweries remains the primary path and Homebrewers the distinct secondary path.
- [ ] Confirm illustrations remain clearly illustrative; no fabricated product UI, unlicensed photography or unsupported customer proof.
- [ ] Confirm no clipped content, horizontal page overflow, awkward wrapping or unusable controls.

## 3. Accessibility and interaction

- [ ] Automated axe checks pass on the current build.
- [ ] Keyboard-only review: skip link, desktop/mobile navigation, focus visibility/order, links, FAQ disclosures and configurator controls.
- [ ] 200% zoom review on homepage, audience pages, pricing, comparison and configurator.
- [ ] Check narrow mobile tap targets and reduced-motion behavior.
- [ ] Screen-reader spot-check of page landmarks, heading order, navigation names, form labels, status language and image alternatives.

Automated axe and responsive checks do not replace these manual checks.

## 4. Content, privacy and maturity

- [ ] Product and website remain described as in development; planned capabilities are not presented as live.
- [ ] No live signup, trial activation, account provisioning, payment, checkout or sales-request submission is implied.
- [ ] Prices, currency, tax, contract terms, customer claims and integrations remain unpublished unless separately evidenced and approved.
- [ ] No analytics SDK, cookies, pixels, session replay, tracking beacon or form collection has been introduced.
- [ ] Non-ERP scope exclusions and under-consideration disclosures remain visible where relevant.

## 5. Preview, performance and launch boundary

- [ ] GitHub Pages preview build/deployment succeeds from the merged current-main acceptance commit.
- [ ] Preview serves the `/brewery` base path and remains `noindex, nofollow` with `Disallow: /`.
- [ ] Record a Lighthouse or equivalent performance review for the current preview (Performance, Accessibility, Best Practices and SEO); investigate material regressions rather than treating a successful build as a performance pass.
- [ ] Confirm production domain, DNS, Vercel production deployment and public launch remain untouched.

## 6. Completion record

Do not mark W12 complete until all automated checks pass, current-run screenshots have been reviewed, manual accessibility/zoom checks are recorded, and the preview deployment is verified. Record the run ID, commit SHA, artifact IDs, visual-review notes, performance results and any accepted limitations below.

| Evidence | Result |
|---|---|
| Current-main CI run | Pending |
| Responsive screenshot artifact | Pending |
| Social-card artifact | Pending |
| GitHub Pages preview deployment | Pending |
| Manual visual review | Pending |
| Keyboard / screen-reader / 200% zoom | Pending |
| Performance review | Pending |
| Production launch | Deferred by owner decision |

**Stop condition:** once W12 is complete and no critical/high-severity website defects remain, close this workstream. Do not expand into product engineering, external brand research or production launch.
