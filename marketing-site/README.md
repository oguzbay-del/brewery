# GRISTORA public marketing site — product roadmap and feature catalogue

An isolated Astro + TypeScript + static-site project for the GRISTORA public marketing website. Public content is being rolled out under the GRISTORA identity and continues to state that the product is in development.

## Brand and design source of truth

**Read [`DESIGN.md`](./DESIGN.md) before making any visual or UI change to the public marketing website.** It is the implementation source of truth for the owner-approved GRISTORA brand-board direction and applies only to `marketing-site/`.

Read [`BRAND-ASSETS.md`](./BRAND-ASSETS.md) for approved logo asset filenames, the recorded palette and asset usage rules. Read [`PHOTO-ASSETS.md`](./PHOTO-ASSETS.md) for the source archive, photo replacement map, responsive formats and build-time optimization. Track final release-readiness evidence in [`ACCEPTANCE.md`](./ACCEPTANCE.md); W12 must pass against the current `main` baseline, not an outdated preview branch.

Read these canonical documents before changing public brand, copy, metadata or visual assets:

- `../web-stratejisi/33-gristora-brand-platform.md`
- `../web-stratejisi/34-gristora-brand-architecture.md`
- `../web-stratejisi/35-gristora-visual-identity-logo-brief.md`
- `../web-stratejisi/36-gristora-public-site-rebrand-plan.md`
- `../web-stratejisi/37-gristora-logo-production-and-qa.md`
- `../web-stratejisi/38-gristora-public-site-brand-implementation-map.md`
- `../web-stratejisi/39-gristora-trademark-domain-preliminary-evidence-log.md`
- `../web-stratejisi/40-gristora-preliminary-goods-services-specification.md`
- `../web-stratejisi/41-gristora-public-marketing-claims-guardrails.md`
- `../web-stratejisi/42-gristora-marketing-site-ci-artifact-review.md`
- `../web-stratejisi/43-gristora-public-homepage-copy-blueprint.md`
- `../web-stratejisi/44-gristora-trademark-counsel-review-brief.md`
- `../web-stratejisi/45-gristora-pricing-membership-onboarding-plan.md`
- `../web-stratejisi/46-gristora-marketing-website-parallel-follow-through.md`
- `../web-stratejisi/47-gristora-marketing-funnel-measurement-spec.md`
- `../web-stratejisi/48-gristora-marketing-content-consistency-audit.md`

GRISTORA is the selected public brand, with the descriptor “Brewing Intelligence”. The owner selected the first logo design shared by the assistant and approved its vector assets in PR #148. Public routes use `public/brand/gristora-symbol.svg`; the full dark/light lockups and proof are in `public/brand/`. See `BRAND-ASSETS.md` and document 37 for asset usage and production/QA notes. Do not claim trademark or domain legal clearance.

**Hard boundary:** marketing-site only. Do not change the product application, APIs, Rust engine, data model, iOS app, internal product identifiers, product roadmap or product development queue.

**ChatGPT assignment rule:** `CHATGPT-SCOPE.md` defines the separate ChatGPT workstream, forbidden product areas, website quality gates and the stop condition. It does not change Claude's product-development instructions or task queue.

## Scope

- Responsive homepage.
- `/pricing/`, `/get-started/` and `/compare/` for the two-audience subscription discovery and configuration preview.
- Dedicated `/breweries/` and `/homebrewers/` audience landing pages. Professional Breweries is the primary commercial path; Homebrewers is distinct and secondary.
- `/how-it-works/` leads with the Professional Breweries workflow, then explains the separate Homebrewers workflow.
- `/technical-principles/` and `/faq/` for technical transparency and product-scope answers.
- Editorial + Technical + Human visual direction.
- No product application code, product API, product queue, or Claude development task changes.

## Custom domain and production launch

The owner confirms that `gristora.com` has been purchased. DNS, domain verification, HTTPS and production launch remain separate checks. Follow [`DOMAIN-SETUP.md`](./DOMAIN-SETUP.md); do not switch the shared Pages deployment from preview mode until those gates and owner launch approval are complete.

## Run locally

```sh
npm install
npm run optimize:photos
npm run dev
```

Pull requests touching `marketing-site/` run the isolated marketing CI: Astro/TypeScript diagnostics, static build, generated-route/metadata/link validation, social-card validation, responsive Playwright and axe accessibility checks. Workflow changes are linted separately with actionlint.

Check, build and preview:

```sh
npm run check
npm run build
npm run validate:site
npm run preview
```

Browser QA (install Chromium once per environment):

```sh
npx playwright install chromium
npm run test:browser
```

`npm run validate:site` runs against the generated `dist/` directory, so run it after `npm run build`. The browser suite checks responsive layouts and axe accessibility rules; it does not replace human keyboard, screen-reader, focus, 200% zoom or real-device review.

## Design notes

- The homepage uses the approved English messaging direction.
- The hero and audience visuals are original editorial SVG illustrations. They are not product screenshots or representations of product functionality.
- Real homebrewing and brewery photography can replace these illustration slots after usage rights and final assets are confirmed.
- Product features are presented as planned capabilities, not currently available functionality. Homebrew MVP, Business core, later releases and unapproved ideas are clearly distinguished.
- The product and website are both in development; no login, subscription, purchase or production workflow is live.
- No fabricated product screenshots, metrics, testimonials, customer logos or monetary prices are included. The pricing preview shows proposed term discounts only; final prices and contractual terms remain owner decisions.
- The homepage primary CTA leads to Professional Breweries; Homebrewers remains available as a separate audience path. Pricing navigation leads to a plan overview, a Home-vs-Brewery feature matrix and an Apple-inspired configuration wizard. The wizard is a non-live preview: no account, trial, card or payment is created. App URL and sign-in URL are not yet configured.
- Core colour values follow the selected GRISTORA brand board and are documented in `BRAND-ASSETS.md`. Layout spacing and component styling remain isolated to the marketing site.

## Social preview asset

- Editable source: `public/social-card-source.svg`.
- Generated output: `public/social-card.png` (1200×630), rendered by Resvg as part of `npm run build` before Astro copies public assets.
- The generated PNG is ignored by Git and must not be committed or edited directly. Change the SVG source and let the build regenerate it.
- CI validates the PNG signature, exact dimensions and a 512 KiB size ceiling, then uploads the image as a short-lived artifact for visual review.
- The generated raster uses explicit DejaVu Sans/Serif font families for consistent typography.
- Public routes reference the generated social-card PNG through the active site origin. Verify preview metadata and final-domain platform previews before release.

## Product content source

- Canonical product plan: `../plan/demleme-plani.md`.
- Feature catalogue data: `src/data/features.ts`.
- Add or revise feature groups in the data file; the feature index, detail pages and sitemap are generated from the same source.
- Keep research-only analytical models marked as under consideration; do not present them as committed capabilities.
- Do not add ERP functions excluded by the product plan (sales, orders, distribution, keg tracking, CRM or accounting).

## Layout

The site follows the approved Stitch layout. `src/styles/stitch-layout.css` holds the Stitch page patterns and tokens (`--sx-*`), with rules scoped by page class such as `.features-page`, `.feature-detail-page` and `.not-found-page`. `src/components/SiteHeader.astro` (nav set B, with a `current` prop for `aria-current`) and `src/components/SiteFooter.astro` (with back-link props) are shared by every page. Change the navigation or footer in those two components only.

## Cloudflare Pages hosting and production gate

Cloudflare Pages is the hosting platform for the GRISTORA marketing website. GitHub remains the source repository and review/CI system. The Cloudflare Pages project is named `gristora`; its deployment integration publishes branch previews and the production branch.

- Repository: `oguzbay-del/brewery`
- Site root: `marketing-site/`
- Framework: Astro static output
- Build command: `npm run build`
- Output directory: `dist`
- Canonical production origin: `https://gristora.com`
- Preview builds use the Cloudflare-provided `CF_PAGES_URL` as their own canonical origin and remain `noindex, nofollow`.
- The production branch uses `https://gristora.com` and is indexable. Do not change DNS or production routing as part of ordinary feature work.

The Astro config has no Vercel environment-variable dependency. Vercel is not part of the website deployment path; do not recreate a Vercel project or add Vercel deployment actions.

## GitHub Pages fallback

- Project website: https://oguzbay-del.github.io/brewery/
- Workflow: `.github/workflows/marketing-site-pages.yml`.
- The workflow sets `GITHUB_PAGES=true` and builds with the `/brewery` base path.
- GitHub Pages remains a separate public `noindex, nofollow` fallback, not the canonical production host.

## Final QA checklist

- Review homepage, audience pages, How It Works, Technical Principles, FAQ and feature details at 1440 / 1024 / 768 / 390 / 320 px and at 200% zoom.
- Verify mobile menu, all internal links, keyboard navigation, native FAQ disclosures, sitemap routes and 404 handling.
- Verify page titles, descriptions, Open Graph, canonical URLs, structured data, robots and sitemap on the production domain.
- Configure product/app/sign-in URLs only after they are confirmed.

## Search metadata

- The homepage publishes schema.org `WebSite` JSON-LD.
- The Homebrewers and Professional Breweries pages publish schema.org `WebPage` JSON-LD with their canonical URLs and the site as their parent.
- Structured data describes the marketing pages only. It does not claim that the product is generally available or that planned functionality is already usable.
