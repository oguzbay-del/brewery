# GRISTORA — Public marketing-site brand implementation map

**Status:** Preparatory inventory for WP-17. This is a source map, not authorization to change the site.
**Scope:** `marketing-site/` public marketing website only.

## 1. Rebrand gate

The owner selected the first GRISTORA logo design shown by the assistant as the visual direction to refine. The selected artwork must be faithfully traced and approved before WP-17 changes the live public brand. The legal/domain review is a separate gate. Do not use later exploration boards as replacement artwork.

## 2. Verified public-brand surfaces

The following files were inspected on the current `main` branch during the 15 Nov 2026 inventory. They still contain the legacy public brand and are candidates for WP-17 only after the gates are passed.

| Surface | Verified file | Brand-sensitive content |
|---|---|---|
| Homepage | `marketing-site/src/pages/index.astro` | Header/footer wordmark, homepage copy, page title, description, Open Graph/Twitter, WebSite JSON-LD, technical-mark graphic |
| Professional Breweries | `marketing-site/src/pages/breweries.astro` | Page title/description, WebPage JSON-LD, FAQ wording, header/footer |
| Homebrewers | `marketing-site/src/pages/homebrewers.astro` | Page title/description, WebPage JSON-LD, FAQ wording, header/footer |
| How it works | `marketing-site/src/pages/how-it-works.astro` | Page title, structured data, audience workflow narrative and repeated brand references |
| Technical principles | `marketing-site/src/pages/technical-principles.astro` | Page title/description, WebPage JSON-LD, technical editorial copy |
| FAQ | `marketing-site/src/pages/faq.astro` | Group headings, question/answer copy and product naming |
| Feature directory | `marketing-site/src/pages/features/index.astro` | Page title/description, CollectionPage JSON-LD, feature-directory copy |
| Social preview source | `marketing-site/public/social-card-source.svg` | Wordmark, descriptive text, palette and brand label; generated PNG is build output |
| Favicon | `marketing-site/public/favicon.svg` | Existing letterform and background treatment |

The homepage source also confirms that the current header/footer and several brand statements are authored directly in the page. Other pages import the shared stylesheet and breadcrumb component; do not assume that all brand surfaces are centralized in a single layout component.

## 3. Required WP-17 discovery before editing

Before implementation, run a repository-wide search scoped to `marketing-site/` for:
- Legacy public brand spelling, case variants, possessives and wordmark labels.
- `og:site_name`, `og:title`, Twitter title/description, canonical metadata and schema.org `name` fields.
- Header/footer wordmarks, navigation labels, mobile menu labels and accessible names.
- Favicon references, theme-color, social card text and any generated asset references.
- Static validation assertions, Playwright selectors/expected copy, README and deployment notes.

Then build a complete inventory of every public page and dynamic feature route. The list in section 2 is a verified starting point, not a claim that it is exhaustive.

## 4. Asset integration sequence

1. Receive the owner-selected source artwork in a form the working environment can inspect.
2. Create and review the editable SVG master under `web-stratejisi/assets/gristora/` according to document 37.
3. Generate favicon, header lockups, monochrome variants and the 1200×630 social preview from that approved master. Do not hand-edit raster derivatives.
4. Apply the approved design tokens to the marketing site without replacing the existing editorial illustrations unless a separate approved asset task calls for it.
5. Preserve existing image-slot IDs and all product feature-stage language.
6. Update public brand strings and metadata consistently across all discovered routes.
7. Extend regression checks to reject legacy public-brand strings in generated HTML and metadata, while explicitly excluding product app and internal code.
8. Run the isolated marketing CI and record visual QA at desktop/tablet/mobile sizes.

## 5. Content and claim protections

- Professional Breweries remains the primary commercial audience; Homebrewers remains a separate secondary path.
- English-first; US and UK initial markets; self-service discovery.
- Preserve product maturity labels. The product and website are in development; do not imply a live service.
- No invented pricing, trial, free tier, signup, production availability, integrations, customers, testimonials, metrics or product screenshots.
- Do not imply ERP, sales, orders, distribution, shipping, keg tracking, CRM or accounting.
- Do not change product app code, APIs, Rust engine, iOS app, schemas, data model, internal identifiers, roadmap or product-development queue.

## 6. Release dependencies

WP-17 remains blocked until:
- The selected logo is faithfully vectorized and owner-approved.
- Trademark/domain screening is recorded and qualified legal review is obtained as appropriate.
- Public origin, hosting plan, canonical URL and HTTPS are approved.
- Analytics/consent, contact route and any conversion destinations have explicit decisions.
- All public claims are checked against the current product plan and verified implementation state.

## 7. Source references

- Brand platform: `33-gristora-brand-platform.md`
- Brand architecture: `34-gristora-brand-architecture.md`
- Logo brief: `35-gristora-visual-identity-logo-brief.md`
- Rebrand plan: `36-gristora-public-site-rebrand-plan.md`
- Logo production/QA: `37-gristora-logo-production-and-qa.md`
- Launch decisions: `26-launch-decision-register.md`
