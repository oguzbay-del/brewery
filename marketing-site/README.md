# GRISTORA public marketing site — product roadmap and feature catalogue

An isolated Astro + TypeScript + static-site project for the GRISTORA public marketing website. The current implementation still contains the legacy Demleme public branding until the planned rebrand work package is approved and completed.

## Brand source of truth

Read these canonical documents before changing public brand, copy, metadata or visual assets:

- `../web-stratejisi/33-gristora-brand-platform.md`
- `../web-stratejisi/34-gristora-brand-architecture.md`
- `../web-stratejisi/35-gristora-visual-identity-logo-brief.md`
- `../web-stratejisi/36-gristora-public-site-rebrand-plan.md`
- `../web-stratejisi/37-gristora-logo-production-and-qa.md`

GRISTORA is the selected public brand. Do not use Demleme in public marketing surfaces after WP-17 is implemented. The owner has selected the first logo design shared by the assistant as the direction to refine; it is not yet a production-approved master. See document 37 for the production/QA contract. The exact source artwork must be available before faithful vector tracing. Domain clearance remains open. Do not start the public rebrand before the identity and legal/domain gates are recorded. This repository documentation is the handoff for Claude.

**Hard boundary:** marketing-site only. Do not change the product application, APIs, Rust engine, data model, iOS app, internal product identifiers, product roadmap or product development queue.

## Scope

- Responsive homepage.
- Dedicated `/breweries/` and `/homebrewers/` audience landing pages. Professional Breweries is the primary commercial path; Homebrewers is distinct and secondary.
- `/how-it-works/` leads with the Professional Breweries workflow, then explains the separate Homebrewers workflow.
- `/technical-principles/` and `/faq/` for technical transparency and product-scope answers.
- Editorial + Technical + Human visual direction.
- No product application code, product API, product queue, or Claude development task changes.

## Run locally

```sh
npm install
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
- No fabricated product screenshots, metrics, testimonials, customer logos or pricing are included.
- The homepage primary CTA leads to Professional Breweries; Homebrewers remains available as a separate audience path. App URL and sign-in URL are not yet configured.
- Colour and spacing values follow the existing `tasarim-sistemi/tokens.json` source. The marketing site applies them in a separate editorial layout.

## Social preview asset

- Editable source: `public/social-card-source.svg`.
- Generated output: `public/social-card.png` (1200×630), rendered by Resvg as part of `npm run build` before Astro copies public assets.
- The generated PNG is ignored by Git and must not be committed or edited directly. Change the SVG source and let the build regenerate it.
- CI validates the PNG signature, exact dimensions and a 512 KiB size ceiling, then uploads the image as a short-lived artifact for visual review.
- The approved raster uses explicit DejaVu Sans/Serif font families for consistent typography.
- Do not wire absolute `og:image` or `twitter:image` URLs until the final public origin is approved and the image is reachable over HTTPS. Verify final-domain metadata and platform previews before release.

## Product content source

- Canonical product plan: `../plan/demleme-plani.md`.
- Feature catalogue data: `src/data/features.ts`.
- Add or revise feature groups in the data file; the feature index, detail pages and sitemap are generated from the same source.
- Keep research-only analytical models marked as under consideration; do not present them as committed capabilities.
- Do not add ERP functions excluded by the product plan (sales, orders, distribution, keg tracking, CRM or accounting).

## Image replacement map

Search the `data-image-slot` attributes or `IMAGE SLOT:` comments to find every temporary visual:

| Slot | File / location | Replace with |
|---|---|---|
| `home-hero` | `src/pages/index.astro` hero-art | Wide editorial brewing-process photo |
| `audience-homebrewers` | `src/pages/index.astro` audience card | Homebrew setup / brewer at work |
| `audience-breweries` | `src/pages/index.astro` audience card | Professional brewery / stainless tanks |
| `homebrewers-hero` | `src/pages/homebrewers.astro` | Homebrewer working on a batch |
| `breweries-hero` | `src/pages/breweries.astro` | Professional brewery team / production floor |

Keep the slot ID when replacing an image. Add final assets under `public/images/`, use descriptive filenames, and confirm usage rights before publishing. Do not use product UI screenshots unless supplied and verified.

## Vercel preview and production gate

Vercel is currently integrated for preview. The commercial production host, plan and public domain are not approved yet. Keep the GitHub repository as the source of truth.

1. Import `oguzbay-del/brewery` into Vercel.
2. Set **Root Directory** to `marketing-site`.
3. Select **Astro** as the framework preset.
4. Use the defaults: install `npm install`, build `npm run build`, output `dist`.
5. Do not add an Astro server adapter; this site is statically generated.
6. Vercel's `VERCEL_PROJECT_PRODUCTION_URL` / `VERCEL_URL` values provide the deployment host. Set `SITE_URL` to the final canonical origin once the production domain is confirmed. Do not repeatedly retry Preview deployments while the Hobby quota window is active; GitHub Pages is the temporary preview fallback.
7. Only after the production host and domain are approved, connect the domain and verify HTTPS, canonical URLs, `/robots.txt`, `/sitemap.xml`, and all routes before switching public traffic.

The Astro config uses the root path for the Vercel preview and `/brewery` only for the GitHub Pages fallback. The robots and sitemap endpoints are generated at build time from the active site/base URL.

**Plan note:** Vercel Hobby is for personal, non-commercial use. Use it for private setup/preview only; upgrade to Pro before operating this commercial marketing site publicly.

## GitHub Pages fallback

- **Project website:** https://oguzbay-del.github.io/brewery/
- Workflow: `.github/workflows/marketing-site-pages.yml`.
- The workflow sets `GITHUB_PAGES=true` to build with the `/brewery` base path.
- GitHub Pages is the temporary preview fallback while Vercel is rate-limited. Before public release, complete final responsive browser QA on the approved production preview environment.

## Final QA checklist

- Review homepage, audience pages, How It Works, Technical Principles, FAQ and feature details at 1440 / 1024 / 768 / 390 / 320 px and at 200% zoom.
- Verify mobile menu, all internal links, keyboard navigation, native FAQ disclosures, sitemap routes and 404 handling.
- Verify page titles, descriptions, Open Graph, canonical URLs, structured data, robots and sitemap on the production domain.
- Configure product/app/sign-in URLs only after they are confirmed.


## Search metadata

- The homepage publishes schema.org `WebSite` JSON-LD.
- The Homebrewers and Professional Breweries pages publish schema.org `WebPage` JSON-LD with their canonical URLs and the site as their parent.
- Structured data describes the marketing pages only. It does not claim that the product is generally available or that planned functionality is already usable.
