# GRISTORA marketing website — W12 final acceptance

**Status:** In progress  
**Scope:** `marketing-site/` only  
**Baseline:** `main` at `c4aff1dfceb9841b01e69b2a515a6ace7a6f4ea0` (17 November 2026)  
**Production:** Intentionally deferred; this gate validates the noindex preview only.

This is the current release-readiness record for W12. The CI and GitHub Pages evidence below is retained as historical traceability; current branch previews are served by Cloudflare Pages. Use only current-run screenshot and social-card artifacts for visual review.

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

- [ ] Cloudflare Pages branch preview succeeds for the acceptance PR; verify that the preview remains noindex and does not change production domains or DNS.
- [ ] Preview serves the `/brewery` base path and remains `noindex, nofollow` with `Disallow: /`.
- [ ] Record a Lighthouse or equivalent performance review for the current preview (Performance, Accessibility, Best Practices and SEO); investigate material regressions rather than treating a successful build as a performance pass.
- [ ] Confirm production domain, DNS, production deployment and public launch remain untouched.

## 6. Completion record

Do not mark W12 complete until all automated checks pass, current-run screenshots have been reviewed, manual accessibility/zoom checks are recorded, and the preview deployment is verified. Record the run ID, commit SHA, artifact IDs, visual-review notes, performance results and any accepted limitations below.

| Evidence | Result |
|---|---|
| Historical baseline CI run | **Run [44090051170](https://github.com/oguzbay-del/brewery/actions/runs/44090051170) passed** at `ff8a1f423b1e7ddcf9cb1ed17109345eeeaf5c94`; retained as historical evidence |
| Responsive screenshot artifact | **#14330514342** (`marketing-responsive-qa-screenshots`, 12,185,910 bytes); review pending |
| Social-card artifact | **#14329774380** (`marketing-social-card-png`, 59,560 bytes); review pending |
| Historical GitHub Pages deployment | **Run [44090051147](https://github.com/oguzbay-del/brewery/actions/runs/44090051147) succeeded** for the older source tree; retained as historical evidence only |
| Current Cloudflare Pages branch preview | Pending verification on the W21 branch; no production deployment is part of this task |
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
| Compare audience imagery | **Resolved.** The two audience cards show the Homebrewer and Professional Brewery illustrations at desktop (1440 px) and mobile (390 px). Screenshot evidence: artifact #14763286642 (run #45044334275) and artifact #14767685935 (PR #192; compare captures at 390 and 1440 px). Source assets: `public/audience/homebrewer.svg` and `public/audience/professional-brewery.svg` |
| Manual accessibility | Keyboard-only full-site, screen-reader and 200% zoom review still pending |
| Stitch parity | Not verified: no active Stitch connector/export available in this environment |
| Production launch | Deferred; no production deployment initiated |

The mobile comparison table is intentionally horizontally scrollable within its labelled, keyboard-focusable region; this is not page-level horizontal overflow. The pricing preview keeps commercial pricing unannounced and explicitly states that proposed trials are not active. Do not treat either as a live offer. The GitHub Pages workflow uses `preview: false`; therefore the GitHub Pages URL is public (although noindex), not a private/ephemeral preview. The custom production domain remains untouched.

**The empty media-frame defect is resolved. W12 remains in progress.** Remaining items: route-by-route visual review, manual keyboard/screen-reader/200% zoom review, preview HTTP/content verification and performance review.


## Compare audience imagery — 19 November 2026

- Owner approved filling the two empty audience media frames on `/compare/`.
- Added two original, locally hosted SVG illustrations: `public/audience/homebrewer.svg` and `public/audience/professional-brewery.svg`; no external image requests, stock-photo licensing dependency, customer logos or product UI claims.
- The illustrations use the existing forest/stone/amber palette and 16:9 editorial composition. Meaningful alt text, explicit dimensions, lazy loading and async decoding are included.
- The visual result is verified in screenshot artifact #14763286642 (run #45044334275) at 1440 px and 390 px. PR #192's successful Marketing CI run #45055406807 also produced artifact #14767685935; its `/compare/` captures at 1440 px and 390 px show both audience illustrations, with no empty dashed frames.
- Source assets: `public/audience/homebrewer.svg` and `public/audience/professional-brewery.svg`. The visual defect is resolved; unrelated W12 acceptance items remain open.


## Lighthouse performance audit — 19 November 2026

- Added a repeatable Lighthouse CI audit to the marketing-site pull-request workflow.
- The audit runs against the built Astro static preview, not the production domain, for Home, Compare, Pricing and How it works in mobile and desktop modes.
- Captures Performance, Accessibility, Best Practices and SEO categories; publishes a score table in the GitHub Actions run summary and uploads full HTML/JSON reports as a 14-day artifact.
- Initial measurements are pending the PR workflow run. No performance score is claimed until the runner completes and reports are reviewed.
## Deployment guard investigation — 19 November 2026

**Historical incident — resolved 24 November 2026.** The Vercel automatic-deployment finding below records the situation at that time. It is not a current deployment guard or a current instruction to stop merges.

- Post-merge main workflows [marketing CI #44398637229](https://github.com/oguzbay-del/brewery/actions/runs/44398637229) and [GitHub Pages #44398637250](https://github.com/oguzbay-del/brewery/actions/runs/44398637250) both passed for commit `b92d3774e8fec92d74a66b31d1f4cc79c76615bb`.
- **Deployment boundary correction:** Vercel Git integration automatically created READY deployments with `target: production` after documentation-only commits landed on `main`. Latest deployment: `brewery-33oa7zrx5-demleme.vercel.app`, aliases `brewery-rho.vercel.app`, `brewery-demleme.vercel.app`, `brewery-git-main-demleme.vercel.app`. Vercel project details report SSO protection enabled for all deployments except custom domains, no custom domains attached, and `live: false`. This is not an unrestricted public custom-domain launch, but it is a Vercel production-target deployment and must not be described as "no production deployment."
- GitHub Pages is a separate public `noindex, nofollow` deployment at `https://oguzbay-del.github.io/brewery/`; its workflow uses `preview: false`.
- No `gristora.com` custom domain is attached to the Vercel project; no DNS/custom-domain launch, rollback, deployment cancellation or Vercel settings change was performed.
- **Historical guardrail at the time:** avoid further merges/pushes to `main` until the Vercel Git production-branch/automatic-deployment behavior was reviewed. This restriction was superseded by the owner's 24 November decision recorded at the end of this file. Live HTTP checks were blocked by DNS resolution failure in the current runtime and remained pending.


### Additional QA sample — 19 November 2026

- Reviewed screenshots for Homebrewers mobile (390 px), Breweries desktop (1440 px) and Pricing mobile (390 px). No obvious clipping/page-level horizontal overflow in these samples.
- Live feature-branch preview: homepage and `/pricing/` both returned HTTP 200. Both include `noindex, nofollow` and response header `X-Robots-Tag: noindex`.
- Pricing content explicitly discloses that monetary prices await commercial approval, planned trials are not active in preview, discounts are proposed/not approved, and checkout is not connected.
- The guardrail branch's Vercel deployment is READY with `target: null` (preview). Its PR CI was still running at time of note.


## Stitch alignment — 18 November 2026

- Compared with Stitch project `9024853279551774830` (read only). The shared `SiteHeader`/`SiteFooter` use the Stitch "set B" navigation on all pages. Feature detail and 404 pages use the Stitch layout. Every `<img>` has intrinsic dimensions. The empty dashed frame on How it works is removed.
- Local evidence: astro check 0 errors, validate:site 2304/2304, Playwright 117/117, Lighthouse mobile 99/100/100/100 and desktop 100/100/100/100 on Home, Compare, Pricing and How it works.
- Still open: manual keyboard/screen-reader/200% zoom review; Pages re-verification after the owner-approved merge. Details: `web-stratejisi/49-gristora-session-handoff.md`.


## Current-main QA refresh — 21 November 2026

- Curated feature photography PR [#177](https://github.com/oguzbay-del/brewery/pull/177) merged at commit [116fd00d4bf2ee7c53d05963e5d215429198dd2f](https://github.com/oguzbay-del/brewery/commit/116fd00d4bf2ee7c53d05963e5d215429198dd2f). Twelve owner-supplied original photos are served through responsive AVIF/WebP/JPEG variants.
- Current-main marketing CI [run #44825280339](https://github.com/oguzbay-del/brewery/actions/runs/44825280339) completed successfully for the photo-refresh commit. Build job passed; responsive screenshots and Lighthouse report artifacts were uploaded.
- Current-main GitHub Pages deployment [run #44825280408](https://github.com/oguzbay-del/brewery/actions/runs/44825280408) completed successfully for the same commit. The preview remains the public GitHub Pages copy at `https://oguzbay-del.github.io/brewery/`, configured noindex; this is not the custom-domain launch.
- Responsive screenshot artifact #14673700985 (31 screenshots) was downloaded and reviewed for Home desktop (1440 px) and Home mobile (320 px). No obvious clipping or page-level horizontal overflow in these samples; complete route-by-route visual review remains pending.
- Social-card artifact #14673467385 was generated. Latest Lighthouse report artifact #14673188743 contains mobile and desktop audits for Home, Compare, Pricing and How it works.
- Lighthouse scores (Performance / Accessibility / Best Practices / SEO): Home desktop 100/100/100/100; Home mobile **85/100/100/100**; Compare desktop/mobile 100/100/100/100; Pricing desktop/mobile 100/100/100/100; How it works desktop/mobile 100/100/100/100.
- Home mobile's Performance score is the only material score variance. Report identifies 590 ms total blocking time, 2.0 s main-thread work and an oversized responsive home-hero image (estimated 24 KiB savings); this needs investigation/optimization before W12 closure.
- Manual keyboard-only, screen-reader and 200% zoom review remain outstanding. Stitch parity remains unverified without a live Stitch connector/export.
- Production domain, DNS and Vercel project settings were not changed. At the time of this 21 November note, the Vercel automatic-deployment guard remained in force; it was superseded by the owner's 24 November decision recorded below.

**W12 status remains in progress.** Outstanding: investigate Home mobile Lighthouse 85, finish visual review across all routes and viewport widths, complete manual keyboard/screen-reader/zoom checks, and verify the deployed noindex Pages response.

## Turkish homepage hero parity — 22 November 2026

- PR [#192](https://github.com/oguzbay-del/brewery/pull/192) aligns the Turkish homepage hero with the English photo-led forest-green treatment, preserves Turkish copy and destinations, and keeps the shared GRISTORA wordmark legible on the dark hero.
- Marketing CI [run #45042837044](https://github.com/oguzbay-del/brewery/actions/runs/45042837044) passed on commit `313f95bab52ec9773f0905f57e9d6a985c00a299`: Astro/TypeScript checks, static build, route/metadata/link validation, responsive Playwright/axe checks and Lighthouse audits.
- Lighthouse artifact #14762377454: Home, Compare, Pricing and How it works scored **100/100/100/100** (Performance / Accessibility / Best Practices / SEO) on both mobile and desktop. Home mobile: TBT 50 ms, LCP 1.8 s, CLS 0.
- Responsive screenshot artifact #14762122124 contains 31 screenshots. English homepage mobile (390 px) was reviewed: no obvious clipping or page-level horizontal overflow. The artifact does not contain a Turkish homepage screenshot. The current PR adds `/tr/` to responsive and accessibility browser QA and adds mobile/desktop Lighthouse audits. The new artifact and Turkish visual review are pending.
- Cloudflare Pages branch preview deployed successfully for the PR head: [preview](https://fix-tr-homepage-hero-parity.gristora.pages.dev). Direct HTTP verification from this runtime is blocked by DNS resolution, so no live-response/header claim is made here.
- PR remains unmerged. No production deployment or custom-domain changes were made.

**Remaining for this change:** wait for the new CI run, review Turkish homepage screenshots at all configured widths, compare EN/TR header and hero composition side by side, and complete manual keyboard/screen-reader/200% zoom checks already tracked in W12. Do not weaken existing accessibility tests.


## Owner deployment decision — 24 November 2026 (current)

- The owner confirmed that Vercel is not used for GRISTORA. The previous Vercel automatic-deployment finding and the temporary main-branch stop instruction above are historical and resolved; they are not active guardrails.
- Branch previews are provided by Cloudflare Pages. Review the Cloudflare Pages preview for the relevant PR; do not treat a branch preview as a production release.
- This acceptance-record update does not authorize or perform a production deployment, public launch, production-domain attachment or DNS change. Those actions remain subject to explicit owner approval.
