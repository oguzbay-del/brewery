# GRISTORA marketing brand assets

## Brand lockup

- Public brand: **GRISTORA**
- Descriptor: **Brewing Intelligence**
- Supporting line in the selected identity: **Clarity for every brew.**
- Primary audience: Professional Breweries
- Secondary audience: Homebrewers

The owner selected the first GRISTORA logo direction shown in the supplied brand board and explicitly approved PR #148. The approved vector assets are `public/brand/gristora-logo.svg`, `public/brand/gristora-logo-light.svg`, `public/brand/gristora-symbol.svg` and `public/brand/gristora-logo-proof.svg`. The `*-draft.svg` files remain as review history only; public routes use the approved symbol asset.

## Approved visual direction and palette

The supplied brand board defines these exact core colours:

| Token | Hex | Use |
|---|---|---|
| Forest Green | `#0B2F26` | Primary brand, dark surfaces, headings |
| Amber Gold | `#F4A261` | Accent, key highlights and primary action |
| Stone Beige | `#F7F6F3` | Main light canvas and negative space |
| Slate Gray | `#475569` | Secondary text and supporting UI |

The shared stylesheet exposes these as CSS custom properties. Keep contrast strong, especially for small text on amber and forest surfaces.

## Typography

- Display/editorial headings: serif treatment for editorial contrast; approved brand-board display family is Plus Jakarta Sans for sans-serif headings.
- Interface and body: Inter/system sans-serif.
- Wordmark artwork should retain generous tracking and a distinct descriptor line. Do not approximate the selected emblem with generic brewing clip-art.

## Current website assets

- `public/favicon.svg`: GRISTORA monogram/favicon using forest and amber.
- `public/social-card-source.svg`: editable 1200×630 social preview source.
- `public/social-card.png`: generated during build from the SVG; do not edit or commit generated output.
- Public routes use the generated 1200×630 social card for large-image Open Graph and Twitter previews, with descriptive image alt text.
- Preview and non-production hosts emit `noindex` metadata and `Disallow: /`; only `https://gristora.com` is crawlable.
- Navigation and footer use `public/brand/gristora-symbol.svg`, the owner-approved selected emblem, beside the GRISTORA wordmark.
- The full approved lockups are available for future brand placements; the proof asset includes the supporting line “Clarity for every brew.”

## Claims and maturity

- The product and website remain in development.
- Public pages must not imply current production availability, sign-up, trial, pricing, customer adoption, integrations, certifications or validated predictive functionality.
- All 16 feature pages must preserve their roadmap-stage and research-only disclosures.
- Never publish trademark clearance or legal approval claims based only on domain purchase.

## Production deployment

Production deployment is intentionally deferred. Continue local/CI/Preview work only. Do not configure DNS, activate `gristora.com`, or switch public traffic until the owner explicitly asks.
