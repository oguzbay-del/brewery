# GRISTORA marketing website — W12 final acceptance

**Status:** In progress  
**Scope:** `marketing-site/` only  
**Baseline:** `main` at `c4aff1dfceb9841b01e69b2a515a6ace7a6f4ea0` (17 November 2026)  
**Production:** Intentionally deferred; this gate validates the noindex preview only.

This is the current release-readiness record for W12. The current acceptance evidence is from the latest `main` commit and its dedicated marketing CI and GitHub Pages preview deployment runs. Use only the screenshot and social-card artifacts recorded below for visual review.

## 1. Automated acceptance

Latest successful run: [#44090051170](https://github.com/oguzbay-del/brewery/actions/runs/44090051170), commit `ff8a1f423b1e7ddcf9cb1ed17109345eeeaf5c94`. The current `main` tip `c4aff1dfceb9841b01e69b2a515a6ace7a6f4ea0` is one empty/no-file-change commit ahead, so the tested source tree is identical.

- [x] Astro / TypeScript diagnostics: `npm run check`.
- [x] Static build and generated 1200 × 630 social card: `npm run build`.
- [x] Generated-route, metadata, structured-data, sitemap, robots, image and internal-link validation: `npm run validate:site`.
- [x] Responsive Playwright and axe checks: `npm run test:browser`.
- [ ] Review the uploaded responsive screenshots and social-card artifact from this run (artifact IDs recorded below).

The current validator checks required routes, sitemap coverage and uniqueness, canonical origin, preview-vs-production robots policy, page titles/descriptions, Open Graph and Twitter metadata, JSON-LD, breadcrumb structure, approved public brand, product-maturity disclosures, accessible landmarks/names, image alt attributes, ARIA references, duplicate IDs, primary audience ordering, local links and fragment targets, and 404 navigation.

## 2. Visual review — current Stitch direction

Review screenshots from the current run, not a stale branch artifact.

- [ ] Homepage at 1440, 1024, 768, 390 and 320 px.
- [ ] Professional Breweries and Homebrewers at desktop, tablet and mobile widths.
- [ ] Pricing, comparison and plan configurator at desktop and mobile widths.
- [ ] How It Works, Technical Principles, FAQ and feature directory/detail at desktop and mobile widths.
- [ ] Confirm the approved GRISTORA logo remains intact; forest-green / stone-beige / amber palette, Plus Jakarta Sans / Inter typography, light editorial hero, rounded panels and pill controls are consistent with `DESIGN.md`.
- [ ] Confirm the social card uses the supplied approved full logo lockup and emblem; do not recreate the wordmark with ordinary text or substitute a generic brewing-vessel icon.
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
| Current-main CI run | **Run [44090051170](https://github.com/oguzbay-del/brewery/actions/runs/44090051170) passed** at `ff8a1f423b1e7ddcf9cb1ed17109345eeeaf5c94`; current `main` has an empty/no-file-change commit on top, so the tested tree is identical |
| Responsive screenshot artifact | **#14330514342** (`marketing-responsive-qa-screenshots`, 12,185,910 bytes); review pending |
| Social-card artifact | **#14329774380** (`marketing-social-card-png`, 59,560 bytes); review pending |
| GitHub Pages preview deployment | **Run [44090051147](https://github.com/oguzbay-del/brewery/actions/runs/44090051147) succeeded** for tested source tree; live HTTP/content verification still pending |
| Manual visual review | Sample review: home 1440/390/320, breweries 1440/390, pricing 390, compare 390 and social card; no obvious clipping. Full route-by-route review pending |
| Keyboard / screen-reader / 200% zoom | Pending |
| Performance review | Pending |
| Production launch | Deferred by owner decision |

**Stop condition:** once W12 is complete and no critical/high-severity website defects remain, close this workstream. Do not expand into product engineering, external brand research or production launch.


## Post-merge QA evidence — 19 November 2026

| Evidence | Result |
|---|---|
| Main source tested | Marketing source tree from PR #169 merge commit `03bde32bd189f1798cacc32d574b075f3bc7540c`; current main at QA start `fc5be28070686aaa43140b2ca801e01f4697294a` (later commits were documentation-only) |
| Marketing CI | [Run #44397172981](https://github.com/oguzbay-del/brewery/actions/runs/44397172981) — **passed** |
| GitHub Pages deployment | [Run #44397173007](https://github.com/oguzbay-del/brewery/actions/runs/44397173007) — build and deploy jobs **passed**. `actions/deploy-pages` ran with `preview: false`, publishing a publicly accessible GitHub Pages copy at `https://oguzbay-del.github.io/brewery/`; it is configured `noindex, nofollow` and is separate from the custom production domain `gristora.com` |
| Responsive screenshots | Artifact #14471831688 — 31 PNGs; sample visual review started |
| Social card | Artifact #14471001544 — generated 1200 × 630 PNG visually inspected; logo lockup and approved palette treatment appear consistent |
| Responsive coverage | 11 routes × 5 viewport widths (1440, 1024, 768, 390, 320); document/body overflow assertions passed |
| Accessibility automation | WCAG 2.2 AA axe-tag audit on 11 routes at desktop/mobile; suite passed. Keyboard skip link, mobile nav, FAQ disclosure, reduced-motion and focusable scroll region checks are included |
| Visual review sample | Homepage desktop/mobile, Breweries mobile, Pricing mobile, Compare desktop/mobile, How It Works mobile and social card reviewed; no obvious clipping in sampled layouts |
| Open visual item | `/compare/` audience decision cards show two large empty dashed media frames at desktop and mobile. Needs owner-approved imagery or a deliberate non-image treatment; no asset invented/added |
| Manual accessibility | Keyboard-only full-site, screen-reader and 200% zoom review still pending |
| Stitch parity | Not verified: no active Stitch connector/export available in this environment |
| Production launch | Deferred; no production deployment initiated |

The mobile comparison table is intentionally horizontally scrollable within its labelled, keyboard-focusable region; this is not page-level horizontal overflow. The pricing preview keeps commercial pricing unannounced and explicitly states that proposed trials are not active. Do not treat either as a live offer. The GitHub Pages workflow uses `preview: false`; therefore the GitHub Pages URL is public (although noindex), not a private/ephemeral preview. The custom production domain remains untouched.

**Acceptance remains in progress.** Do not close W12 until the empty media-frame decision, remaining route-by-route visual review, manual accessibility/zoom review, preview HTTP/content check and performance review are resolved or explicitly accepted by the owner.


## Compare audience imagery — 19 November 2026

- Owner approved filling the two empty audience media frames on `/compare/`.
- Added two original, locally hosted SVG illustrations: `public/audience/homebrewer.svg` and `public/audience/professional-brewery.svg`; no external image requests, stock-photo licensing dependency, customer logos or product UI claims.
- The illustrations use the existing forest/stone/amber palette and 16:9 editorial composition. Meaningful alt text, explicit dimensions, lazy loading and async decoding are included.
- Automated build, route validation, responsive screenshot review and accessibility checks for this change remain pending. W12 stays **in progress** until evidence is recorded.


## Lighthouse performance audit — 19 November 2026

- Added a repeatable Lighthouse CI audit to the marketing-site pull-request workflow.
- The audit runs against the built Astro static preview, not the production domain, for Home, Compare, Pricing and How it works in mobile and desktop modes.
- Captures Performance, Accessibility, Best Practices and SEO categories; publishes a score table in the GitHub Actions run summary and uploads full HTML/JSON reports as a 14-day artifact.
- Initial measurements are pending the PR workflow run. No performance score is claimed until the runner completes and reports are reviewed.
