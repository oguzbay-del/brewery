# GRISTORA public marketing site — product roadmap and feature catalogue

An isolated Astro + TypeScript + static-site project for the GRISTORA public marketing website. The current implementation is being rolled out under the GRISTORA identity. Public content remains explicit that the product is in development.

## Brand source of truth

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

GRISTORA is the selected public brand, with the descriptor “Brewing Intelligence”. The owner selected the first logo design shared by the assistant as the direction to refine; it is not yet a production-approved master. See document 37 for the production/QA contract. Do not claim trademark or domain legal clearance. This repository documentation is the website handoff.

**Hard boundary:** marketing-site only. Do not change the product application, APIs, Rust engine, data model, iOS app, internal product identifiers, product roadmap or product development queue.

**ChatGPT assignment rule:** `CHATGPT-SCOPE.md` defines the separate ChatGPT workstream, forbidden product areas, website quality gates and the stop condition. It does not change Claude's product-development instructions or task queue.

## Scope

- Responsive homepage.
- Dedicated `/breweries/` and `/homebrewers/` audience landing pages. Professional Breweries is the primary commercial path; Homebrewers is distinct and secondary.
- `/how-it-works/` leads with the Professional Breweries workflow, then explains the separate Homebrewers workflow.
- `/technical-principles/` and `/faq/` for technical transparency and product-scope answers.
- Editorial + Technical + Human visual direction.
- No product application code, product API, product queue, or Claude development task changes.

## Custom domain and production launch\n\nThe owner confirms that `gristora.com` has been purchased. DNS, GitHub Pages domain verification, HTTPS and production launch remain separate checks. Follow [`DOMAIN-SETUP.md`](./DOMAIN-SETUP.md); do not switch the shared Pages deployment from preview mode until those gates and owner launch approval are complete.\n\n## Run locally

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
- Public routes now reference the generated social-card PNG through the active site origin. Verify preview metadata and final-domain platform previews before release.

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

Vercel is the intended preview and future production host for the marketing website only. Production deployment is deferred by owner instruction. Keep the GitHub repository as the source of truth.

1. Create a **dedicated marketing project** from `oguzbay-del/brewery`; never reuse the product's Vercel project.
2. Set **Root Directory** to `marketing-site`.
3. Select **Astro** as the framework preset.
4. Use install `npm install`, build `npm run build`, output `dist`.
5. Do not add an Astro server adapter; this site is statically generated.
6. Preview builds use `VERCEL_URL` for their own canonical origin. Production later uses `SITE_URL=https://gristora.com` and `SITE_BASE=/`.
7. Production deployment and DNS are intentionally deferred. Do not configure the custom domain or switch public traffic until explicitly requested.

The Astro config uses the root path for Vercel and `/brewery` only for the GitHub Pages fallback. Preview canonical URLs use `VERCEL_URL`; the future production origin is supplied through `SITE_URL`. Robots and sitemap are generated from the active site/base URL.

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
