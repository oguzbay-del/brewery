# Marketing-site release gates and public-claim register

**Status:** working checklist — 15 November 2026  
**Scope:** public-facing Demleme marketing website only. This document does not approve product features, pricing, launch dates or production operations.

## Release gates

| Gate | Evidence required | Current status |
|---|---|---|
| Product truth | Every feature statement maps to an approved product-plan item and carries the correct maturity label | Ongoing; roadmap language is intentionally explicit |
| Commercial path | Confirmed product/app URL, sign-in or registration flow, pricing, trial, limits and billing | Blocked on owner decisions |
| Hosting and domain | Commercially appropriate hosting plan, confirmed production domain, HTTPS and canonical origin | Not ready; Vercel Hobby must not be used for commercial public operation |
| Search and social | Unique title/description, canonical URL, valid Open Graph/Twitter metadata, raster social image and final-domain validation | Partial; SVG source exists, raster export and metadata image wiring remain open |
| Privacy and measurement | Approved analytics provider, consent requirements, event taxonomy and privacy copy | Blocked on owner decisions |
| Accessibility | Automated checks plus human keyboard, screen-reader, focus, zoom and reflow review | Automated checks exist; human review remains open |
| Responsive QA | Real browser review at 1440, 1024, 768, 390 and 320 CSS px | CI screenshots exist; final manual review remains open |
| Content and legal | Approved photography/licences, privacy/terms/contact details and final copy review | Open |
| Release operations | Deployment smoke test, redirects/404, sitemap/robots, rollback owner and post-release monitoring | Not approved |

A successful CI build is necessary but is not equivalent to a production release or a human visual/accessibility review.

## Public-claim rules

1. Use the product maturity vocabulary consistently: **Available**, **In development**, **Planned**, **Under consideration**, and **Requires validation**.
2. Do not describe planned functionality as currently usable, generally available, production-ready or integrated.
3. Do not invent customer names, testimonials, case studies, usage figures, certifications, partnerships, integrations, pricing or launch dates.
4. Keep Homebrewers and Professional Breweries as separate audience journeys; do not collapse their distinct record models.
5. Preserve the product boundary: Demleme is brewing software, not a general ERP. Sales, orders, distribution/shipping, keg tracking, CRM and accounting remain outside stated scope.
6. Do not imply a demo, waitlist, free trial, registration or purchase flow until the destination and operational process exist.
7. Use only user-approved or properly licensed photography. Existing illustrations are editorial concepts, not product screenshots or evidence of shipped functionality.

## Social preview completion sequence

- [x] Create branded 1200×630 SVG source artwork.
- [ ] Export a 1200×630 PNG through a reproducible build step or approved design workflow.
- [ ] Verify PNG dimensions, file size, contrast and text legibility at small preview sizes.
- [ ] Add absolute HTTPS `og:image` and `twitter:image` URLs only after the final public origin is confirmed.
- [ ] Set `twitter:card=summary_large_image` only when the raster image is present and reachable.
- [ ] Validate the deployed image and metadata on the final domain; check representative social preview debuggers.
- [ ] Keep page-specific titles/descriptions accurate; Twitter title should match Open Graph title.

Do not point social metadata at the SVG source as a substitute for a verified raster preview. Do not claim platform validation based only on a successful static build.

## Final owner decisions

- Production domain and commercial hosting plan.
- App URL, sign-in/registration and intended acquisition CTA.
- Pricing, free tier/trial, limits and billing.
- Analytics/consent provider and event taxonomy.
- Approved contact, privacy and legal content.
- Final photography and any real product screenshots.

## Handoff

Update this register when evidence changes. Record the exact CI run, deployment URL, manual reviewer and date for each gate; never mark a gate complete based on an assumption.
