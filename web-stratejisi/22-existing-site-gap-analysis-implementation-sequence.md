# Demleme — Existing Marketing Site Gap Analysis & Implementation Sequence

**Work package:** WP-11  
**Status:** Source-level audit and implementation recommendation; no production changes.  
**Date:** 14 November 2026  
**Reviewed:** Current `main` tree, Astro routes, feature data, global CSS, sitemap/robots endpoints, design tokens and WP-04–WP-10.  
**Scope:** `marketing-site/` only. Product app, API, iOS, server and product development queue excluded.

---

## 1. Executive assessment

The current marketing site is a valid, small static foundation: homepage, two audience landing pages, a feature directory and 16 generated feature-detail routes. It already has canonical metadata, Open Graph/Twitter tags, JSON-LD, sitemap/robots endpoints, a skip link, focus-visible styles and mobile navigation.

It is **not yet the complete marketing experience defined in WP-04–WP-10**. The biggest gap is information architecture and content depth—not a missing animation or decorative redesign. The current implementation is essentially a homepage + audience pages + feature catalogue. It does not yet provide the planned How It Works, Learn, Calculators, FAQ, Technical Principles or Use Cases destinations.

No claim is made here about live visual quality at 1440/390 px: this is a source-level review. A real browser/device QA pass is still required.

## 2. Current implementation inventory

| Area | Current evidence in repository | Assessment |
|---|---|---|
| Framework | Astro 7.x, TypeScript, static output | Good fit for content-led marketing |
| Routes | `/`, `/homebrewers/`, `/breweries/`, `/features/`, 16 `/features/{slug}/`, `/404`, sitemap and robots | Strong initial foundation; limited versus approved sitemap |
| Content model | Central `src/data/features.ts` drives index, detail pages and sitemap | Maintainable starting point; stage and claim validation still required |
| Audience split | Separate Homebrewers and Breweries pages and audience-filtered feature groups | Correct direction; needs deeper workflow and evidence narrative |
| SEO | Canonical, description, title, OG/Twitter, JSON-LD on main routes | Good baseline; metadata consistency and sitemap coverage need review |
| Crawl | Generated sitemap and robots.txt | Functional source exists; sitemap covers only current routes |
| Design | Editorial/technical style, system fonts, responsive CSS, SVG placeholders | Coherent start; token source is duplicated into CSS variables |
| Accessibility | Skip link, focus-visible, semantic nav, details/summary mobile menu | Good baseline; needs keyboard, screen reader, contrast and zoom QA |
| Imagery | Five named placeholders / SVG illustrations | Correctly avoids unapproved photography; retain slot IDs |
| Conversion | Audience and informational CTAs | Honest for product-in-development; no real product account flow |
| Analytics | No provider configured in reviewed package | Do not add until privacy/consent decisions |
| Build | Astro check/build workflow exists in repository | CI is necessary but does not replace browser QA |

## 3. Gap matrix against the approved strategy

| Strategy requirement | Current state | Gap / risk | Priority |
|---|---|---|---:|
| Homepage routes two audiences clearly | Present | Strengthen section order and technical proof; avoid broad product claims | P1 |
| Homebrewers journey | Separate page exists | Needs explicit step-by-step recipe → brew → measure → learn story, with verified stages | P1 |
| Professional Breweries journey | Separate page exists | Needs recipe → production → batch/QC story and prominent ERP exclusions | P1 |
| Audience-specific examples and terminology | Some separate copy and feature filtering | Need systematically different examples, process maps and records | P1 |
| Feature taxonomy | 16 data-driven areas | Product-claim audit WP-03 not yet integrated on main; resolve status conflicts before copy finalization | P0 |
| Feature detail template | Present | Add problem, workflow inputs/outputs, evidence, limitations and related resources; status currently risks generic “Planned capability” treatment | P0 |
| How It Works | Not present | New route and two audience journeys | P1 |
| Learn / knowledge centre | Not present | No technical search content destination | P1 |
| Calculators | Not present | Public scope and formula validation are not approved; create route only after gate | Hold |
| FAQ | Not present | Product/scope questions lack a central answer destination | P1 |
| Technical Principles | Not present | Technical trust promise lacks a dedicated evidence/method page | P1 |
| Use Cases | Not present | No grounded scenario pages | P2 |
| Pricing / Roadmap | Not present | Correctly absent while decisions are open | Hold |
| Internal linking | Current nav and page links | Need contextual links between journeys, feature pages and technical resources | P1 |
| SEO sitemap | Includes current routes and feature pages | Future routes need deliberate indexability; no `lastmod` currently emitted | P1 |
| Structured data | WebSite, WebPage, CollectionPage JSON-LD | Ensure schema matches visible content; do not add ratings/offers or unsupported SoftwareApplication facts | P1 |
| Shared design tokens | CSS custom properties reflect current token values | Values are copied into CSS rather than generated/verified from `tasarim-sistemi/tokens.json`; drift risk | P1 |
| Reusable components | Shared CSS classes; page markup repeated | Header, metadata, breadcrumbs and footer are duplicated across pages | P1 |
| Accessibility verification | Source-level basics present | No documented WCAG 2.2 AA audit, keyboard/screen-reader/200% zoom test | P0 before launch |
| Responsive verification | CSS media queries present | No verified browser pass at 1440/1024/768/390/320 | P0 before launch |
| Performance verification | Static Astro output | No recorded real-user Core Web Vitals or repeatable Lighthouse baseline | P2 |
| Conversion / account CTA | Informational paths only | Correct until real product destination exists; do not invent signup/trial/demo | Hold |

## 4. High-risk content findings

1. **Feature catalogue status:** detail pages currently show a general “IN DEVELOPMENT” eyebrow and “Planned capability” tag while listing multiple release stages in a sidebar. This can flatten distinctions between planned, under consideration and requires validation. WP-03 status vocabulary must become the single source of truth.
2. **Analytical claims:** fermentation prediction, expected-curve deviation and calibration conflicts identified in WP-03 must be resolved or conservatively labelled. The feature pages must not imply approval.
3. **Homepage/feature headline:** “A complete brewing workspace, from recipe to release” is broad and may imply completeness beyond the actual current product stage. Reassess against WP-05 positioning and explicit Business exclusions.
4. **Business scope:** ensure the visible page clearly says Demleme is not a general ERP and excludes sales, orders, distribution/shipping, keg tracking, CRM and accounting.
5. **Product state:** “in development” language is appropriate while no customer-facing app flow is live. Do not replace with available/present-tense capability claims based on roadmap or engine code.
6. **CTA state:** existing “Find your path” / “Explore” routes are safe informational CTAs. Keep Sign in, Start free, Trial, Demo, Pricing and Waitlist absent until their destinations and terms exist.

## 5. SEO and technical observations

- Homepage and audience pages have distinct title/description and canonical URLs.
- Feature index and detail pages generate JSON-LD; the visible page and structured data must stay aligned.
- Feature-detail pages currently set a Twitter card type but do not visibly set a dedicated Twitter title/description in the inspected head; add consistency in a shared metadata component.
- Sitemap route list is manually assembled from the three primary pages and feature data. Add future routes only when published/indexable; do not expose drafts or placeholder routes.
- Sitemap currently emits no `lastmod`. Add it only when a reliable content-update timestamp exists; do not use build time as a false update.
- Robots endpoint allows all and points to the active sitemap. Verify production host, preview noindex and base-path behavior after deployment.
- Astro config supports Vercel and GitHub Pages base paths. Verify canonical and sitemap URLs for the actual production domain, not only a preview host.
- No analytics package is configured in the inspected marketing-site package. Keep that state until privacy/consent decisions are approved.

## 6. Design-system and component gaps

### Token source
The CSS defines values such as page, surface, ink, muted, amber and line. These broadly correspond to `tasarim-sistemi/tokens.json`, but they are hardcoded copies. The implementation should either:
- generate a CSS token file from the canonical token source; or
- add a documented parity test that fails when values drift.

Do not introduce new palette values or change the product-wide token source as part of the marketing refresh without approval.

### Shared components
Refactor repeated page shell patterns only when implementation begins:
- `BaseLayout` / document head metadata.
- Header and mobile navigation.
- Footer.
- Breadcrumbs.
- Status badge/label.
- Feature group and feature detail section.
- Technical evidence/method panel.
- CTA link.
- FAQ disclosure.
- Table/diagram primitives.

Avoid over-componentizing one-off editorial sections.

## 7. Recommended implementation sequence

| Sequence | Work | Output | Gate |
|---:|---|---|---|
| 0 | Resolve WP-03 and PR #16 feature data | One authoritative claim/status map | Product owner and clean PR review |
| 1 | Shared layout and metadata components | Consistent head, header, nav, footer and breadcrumbs | CI + accessibility checks |
| 2 | Homepage and audience page content/layout alignment | Distinct Home/Business narratives and process maps | Product claim review |
| 3 | Feature catalogue remediation | Status vocabulary, limitations, evidence and contextual links | WP-03 complete |
| 4 | How It Works and Technical Principles | New workflow and proof destinations | Approved product facts |
| 5 | FAQ and Learn foundations | Useful answer pages and content collection | Technical/editorial review |
| 6 | Calculator directory / tools | Only validated and approved calculators | WP-09 acceptance + public-use approval |
| 7 | Use cases | Grounded scenarios | Product owner evidence |
| 8 | SEO, sitemap and structured data | Accurate indexable routes and metadata | WP-08 checks |
| 9 | Responsive/accessibility/performance QA | Recorded browser and automated QA | WCAG 2.2 AA target; all critical tests pass |
| 10 | Production release | Approved public site | Explicit user release approval |

## 8. Proposed first-release scope

**Include**
- Homepage.
- Homebrewers and Professional Breweries.
- Feature directory and only those detail pages whose claims are reconciled.
- How It Works (two paths).
- Technical Principles (evidence-backed, careful scope).
- FAQ (only answered questions).
- Learn landing page only when substantive articles exist.

**Defer**
- Public calculators until WP-09 formula and public-scope gates pass.
- Pricing, roadmap dates, customer stories and testimonials until verified and approved.
- Use-case pages until they can be grounded in a real approved workflow.
- Login and account conversion until the product has a real customer-facing flow.

## 9. Release QA checklist

- [ ] Product feature status reconciled against plan and WP-03.
- [ ] Business ERP exclusions visible and consistent on every relevant page.
- [ ] No unapproved prediction, calibration, integration, security or regulatory claims.
- [ ] All routes and CTA links resolve.
- [ ] Canonicals and sitemap use the correct production origin.
- [ ] Preview/staging is not accidentally indexable.
- [ ] JSON-LD matches visible content.
- [ ] Mobile menu, keyboard, focus, reduced motion and 200% zoom verified.
- [ ] Screenshots captured at 1440, 1024, 768, 390 and 320 px.
- [ ] Contrast and WCAG 2.2 AA checks recorded.
- [ ] Performance baseline recorded; no avoidable layout shift or blocking scripts.
- [ ] Privacy/consent reviewed before analytics or forms.
- [ ] User explicitly approves production implementation and release.

## 10. Current implementation gate

This audit is a source-level gap analysis only. It does not authorize modifying `marketing-site/`, adding assets, launching calculators, changing product claims, connecting analytics or deploying a redesigned public site.

**Next:** WP-12 — consolidate the implementation specification and acceptance checklist, then request the explicit production implementation gate. Until approval, continue documentation, claim reconciliation and preview-based QA only.
