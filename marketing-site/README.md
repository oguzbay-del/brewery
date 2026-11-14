# Demleme marketing site — homepage prototype

An isolated Astro + TypeScript + SSG project for the Demleme marketing website.

## Scope

- Desktop and mobile homepage prototype.
- Dedicated `/homebrewers` and `/breweries` audience landing pages.
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
- No product dashboard, metrics, testimonials, customer logos, pricing, or unsupported outcome promises are included. Audience copy describes areas of focus rather than asserting unverified product functionality.
- The homepage audience CTAs route to the two dedicated audience pages. App URL and sign-in URL are not yet configured.
- Colour and spacing values follow the existing `tasarim-sistemi/tokens.json` source. The marketing site applies them in a separate editorial layout.

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

## Next review

Review the homepage and both audience pages at 1440px desktop and 390px mobile. Configure production app/sign-in URLs only after they are confirmed.

## GitHub Pages preview

- **Project website:** https://oguzbay-del.github.io/brewery/
- **Repository root URL:** https://oguzbay-del.github.io/ is a separate user-site address and is not the Demleme project preview.
- Astro uses `site: https://oguzbay-del.github.io` and `base: /brewery`; internal page links respect this base path.
- Deployment workflow: `.github/workflows/marketing-site-pages.yml` (default-branch only; it builds and deploys the marketing site to the project URL).
- Pushes to `main` that change `marketing-site/**` or this workflow build and deploy `marketing-site/dist` only. The PR check workflow validates the marketing site before merge; Pages deployment happens from the default branch.
- The separate `marketing-site.yml` workflow runs PR checks.
- After the PR checks and scope review pass, merge manually. Then verify the published project site at desktop and mobile sizes.
