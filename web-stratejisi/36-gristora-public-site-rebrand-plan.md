# GRISTORA — Public marketing site rebrand plan

**Status:** Planned / gated. The owner has selected the first shared GRISTORA logo design as the direction to refine. Do not change the public website brand until that exact artwork is faithfully refined and approved, and the name-clearance gate is addressed.  
**Scope:** `marketing-site/` only. Do not change the product application or software.

## 1. Objective

Move the public marketing website to the GRISTORA brand. Preserve its current commercial strategy, audience hierarchy, product claims, technical architecture, SEO quality and release gates.

The website remains a marketing and self-service discovery surface. It is not the software application.

## 2. Inputs

- `33-gristora-brand-platform.md` — canonical brand platform.
- `34-gristora-brand-architecture.md` — brand and public-site architecture.
- `35-gristora-visual-identity-logo-brief.md` — logo and visual identity brief.
- `37-gristora-logo-production-and-qa.md` — production assets, vector quality, proof board and acceptance criteria.
- Existing public claim register and feature catalogue.
- Existing Astro design tokens, accessibility rules and responsive QA.

## 3. Public-site surfaces to update

- Header and footer wordmark.
- Favicon and browser theme color, after palette approval.
- Page titles, meta descriptions, Open Graph and Twitter metadata.
- Schema.org WebSite / WebPage name and descriptions.
- Social card source SVG and generated PNG pipeline.
- Homepage hero copy and brand descriptor.
- Navigation, CTA labels and audience landing pages.
- FAQ, technical principles, feature pages and all editorial content.
- Breadcrumbs, canonical metadata, sitemap and robots output where affected.
- Tests and validators that assert the old brand string.
- README, release register and deployment notes.

## 4. Preserve

- Professional Breweries is the primary commercial audience.
- Homebrewers is a separate secondary audience.
- English is the primary public language.
- US and UK are the initial markets.
- Self-service discovery is the commercial approach.
- Product feature stages and claims remain accurate.
- No invented customer proof, metrics, screenshots, pricing, trial, free tier, billing or sign-up.
- Existing Astro + TypeScript + static generation architecture and isolated marketing CI.
- Existing image-slot IDs and image replacement policy unless a separate visual task approves a change.

## 5. Explicit exclusions

Do not modify:
- Product app code in `web/`, `ios/`, `sunucu/`, `sozlesme/`, `altyapi/`.
- Product APIs, Rust calculation engine, schemas, database or deployment architecture.
- Product internal name, app display name, bundle IDs, package/crate names or identifiers.
- Product roadmap, product plan, Claude product-development task queue or business logic.
- Product screenshots or functionality.

Do not rename repository folders or product-internal paths as part of this public rebrand.

## 6. Implementation sequence

1. Complete trademark/domain screening and record the decision. Do not present screening as legal clearance.
2. Review at least three logo routes and select one with the product owner.
3. Produce final logo assets and brand tokens.
4. Create a dedicated branch for the marketing-site-only change.
5. Update public copy, brand assets and metadata in one coherent batch.
6. Update static validators and Playwright checks to reject old public brand strings.
7. Run Astro/TypeScript diagnostics, static build, site validation, social-card validation, Playwright and axe checks.
8. Review desktop, tablet and mobile screenshots. Keep manual keyboard, screen-reader, 200% zoom and real-device checks as release gates.
9. Verify final canonical origin, HTTPS social image reachability, sitemap, robots and platform previews after domain approval.
10. Open PR. Merge manually after CI and review. Do not use GitHub auto-merge.

## 7. Acceptance criteria

- All public marketing surfaces use GRISTORA consistently.
- No legacy public brand string remains in generated HTML, metadata, structured data, social card or visible copy.
- No product app or software files changed.
- No new unverified product claims.
- All existing marketing CI and accessibility checks pass.
- Desktop and mobile visual review is recorded.
- Public release remains gated by domain, hosting, legal, analytics/consent and contact decisions.

## 8. Work packages

| WP | Work | Dependency | Status |
|---|---|---|---|
| WP-15 | Approve brand platform and brand architecture | User's GRISTORA decision | documented |
| WP-16 | Refine owner-selected first logo design and complete visual QA | WP-15 + exact reference artwork | in progress |
| WP-17 | Apply approved GRISTORA identity to the public marketing website | WP-15, WP-16, legal/domain gate | todo |
| WP-18 | Final public-domain, social preview and launch QA | WP-17 and launch decisions | todo |

## 9. Handoff for Claude

Read these four documents before changing the public marketing website:
- `web-stratejisi/33-gristora-brand-platform.md`
- `web-stratejisi/34-gristora-brand-architecture.md`
- `web-stratejisi/35-gristora-visual-identity-logo-brief.md`
- `web-stratejisi/36-gristora-public-site-rebrand-plan.md`

The user has selected GRISTORA as the public brand and the first assistant-presented logo design as the visual direction to refine. The exact rendered reference must be accessed before faithful vector tracing; do not fabricate it. Final logo approval and domain clearance remain open. Do not modify product software. Do not apply the public-site rebrand until the logo direction is reviewed and the legal/domain gate is recorded.
