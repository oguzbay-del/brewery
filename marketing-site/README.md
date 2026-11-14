# Demleme marketing site — homepage prototype

An isolated Astro + TypeScript + SSG project for the Demleme marketing website.

## Scope

- Desktop and mobile homepage prototype.
- Two audience paths: Homebrewers and Professional Breweries.
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
- No product dashboard, metrics, testimonials, customer logos, pricing, or unverified feature claims are included.
- The CTA currently anchors to the audience selector. The app URL and sign-in URL are not yet configured.
- Colour and spacing values follow the existing `tasarim-sistemi/tokens.json` source. The marketing site applies them in a separate editorial layout.

## Next review

Review the desktop composition at 1440px and mobile composition at 390px. Confirm the visual direction before adding the two audience landing pages or wiring production URLs.
