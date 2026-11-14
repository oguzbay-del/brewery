# Demleme marketing site — product roadmap and feature catalogue

An isolated Astro + TypeScript + static-site project for the Demleme marketing website.

## Scope

- Responsive homepage.
- Dedicated `/homebrewers/` and `/breweries/` audience landing pages, with distinct Homebrew learning-loop and Business record-relationship explanations.
- `/how-it-works/` with separate Home and Business workflows.
- `/technical-principles/` and `/faq/` for technical transparency and product-scope answers.
- Editorial + Technical + Human visual direction.
- No product application code, product API, product queue, or Claude development task changes.

## Run locally

```sh
npm install
npm run dev
```

Pull requests touching `marketing-site/` run an isolated GitHub Actions check for Astro/TypeScript and static build.

Check and build:

```sh
npm run check
npm run build
npm run preview
```

## Design notes

- The homepage uses the approved English messaging direction.
- The hero and audience visuals are original editorial SVG illustrations. They are not product screenshots or representations of product functionality.
- Real homebrewing and brewery photography can replace these illustration slots after usage rights and final assets are confirmed.
- Product features are presented as planned capabilities, not currently available functionality. Homebrew MVP, Business core, later releases and unapproved ideas are clearly distinguished.\n- The product and website are both in development; no login, subscription, purchase or production workflow is live.\n- No fabricated product screenshots, metrics, testimonials, customer logos or pricing are included.
- The homepage audience CTAs route to the two dedicated audience pages. App URL and sign-in URL are not yet configured.
- Colour and spacing values follow the existing `tasarim-sistemi/tokens.json` source. The marketing site applies them in a separate editorial layout.

## Product content source\n\n- Canonical product plan: `../plan/demleme-plani.md`.\n- Feature catalogue data: `src/data/features.ts`.\n- Add or revise feature groups in the data file; the feature index, detail pages and sitemap are generated from the same source.\n- Keep research-only analytical models marked as under consideration; do not present them as committed capabilities.\n- Do not add ERP functions excluded by the product plan (sales, orders, distribution, keg tracking, CRM or accounting).\n\n## Image replacement map

Search the `data-image-slot` attributes or `IMAGE SLOT:` comments to find every temporary visual:

| Slot | File / location | Replace with |
|---|---|---|
| `home-hero` | `src/pages/index.astro` hero-art | Wide editorial brewing-process photo |
| `audience-homebrewers` | `src/pages/index.astro` audience card | Homebrew setup / brewer at work |
| `audience-breweries` | `src/pages/index.astro` audience card | Professional brewery / stainless tanks |
| `homebrewers-hero` | `src/pages/homebrewers.astro` | Homebrewer working on a batch |
| `breweries-hero` | `src/pages/breweries.astro` | Professional brewery team / production floor |

Keep the slot ID when replacing an image. Add final assets under `public/images/`, use descriptive filenames, and confirm usage rights before publishing. Do not use product UI screenshots unless supplied and verified.

## Vercel deployment

The production target is Vercel. Keep the GitHub repository as the source of truth.

1. Import `oguzbay-del/brewery` into Vercel.
2. Set **Root Directory** to `marketing-site`.
3. Select **Astro** as the framework preset.
4. Use the defaults: install `npm install`, build `npm run build`, output `dist`.
5. Do not add an Astro server adapter; this site is statically generated.
6. Vercel's `VERCEL_PROJECT_PRODUCTION_URL` / `VERCEL_URL` values provide the deployment host. Set `SITE_URL` to the final canonical origin once the production domain is confirmed.
7. Connect the production domain and verify HTTPS, canonical URLs, `/robots.txt`, `/sitemap.xml`, and all three pages before switching public traffic.

The Astro config uses the root path on Vercel and `/brewery` only for the GitHub Pages fallback. The robots and sitemap endpoints are generated at build time from the active site/base URL.

**Plan note:** Vercel Hobby is for personal, non-commercial use. Use it for private setup/preview only; upgrade to Pro before operating this commercial marketing site publicly.

## GitHub Pages fallback

- **Project website:** https://oguzbay-del.github.io/brewery/
- Workflow: `.github/workflows/marketing-site-pages.yml`.
- The workflow sets `GITHUB_PAGES=true` to build with the `/brewery` base path.
- GitHub Pages is the temporary preview fallback while Vercel is rate-limited. Vercel Preview and final responsive browser QA remain mandatory before public release.

## Final QA checklist

- Review homepage, audience pages, How It Works, Technical Principles, FAQ and feature details at 1440 / 1024 / 768 / 390 / 320 px.
- Verify mobile menu, all internal links, keyboard navigation, native FAQ disclosures, sitemap routes and 404 handling.
- Verify page titles, descriptions, Open Graph, canonical URLs, structured data, robots and sitemap on the production domain.
- Configure product/app/sign-in URLs only after they are confirmed.


## Search metadata

- The homepage publishes schema.org `WebSite` JSON-LD.
- The Homebrewers and Professional Breweries pages publish schema.org `WebPage` JSON-LD with their canonical URLs and the site as their parent.
- Structured data describes the marketing pages only. It does not claim that the product is generally available or that planned functionality is already usable.
