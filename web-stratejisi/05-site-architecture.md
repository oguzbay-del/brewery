# Site haritası ve sayfa briefleri

> **Commercial hierarchy (15 November 2026):** Professional Breweries (Business) is the primary commercial audience. Homebrewers (Home) remains a distinct secondary audience. The homepage, navigation, content investment and primary CTA lead with Breweries. This document describes the current marketing-site structure and separately marks gated future routes.

## Current public sitemap

| Route | Page | Audience / role | Priority |
|---|---|---|---|
| `/` | Homepage | Business-first positioning and routing; Homebrewers secondary | P0 |
| `/breweries/` | Professional Breweries | Primary commercial landing page | P0 — primary |
| `/homebrewers/` | Homebrewers | Distinct secondary audience journey | P1 — secondary |
| `/features/` | Product features and roadmap | Capability discovery, with explicit maturity labels | P0 |
| `/features/[slug]/` | Feature detail | Evidence, scope, dependencies and status | P0 |
| `/how-it-works/` | Workflows | Separate Home and Business journeys | P1 |
| `/technical-principles/` | Technical principles | Methods, units, measurement and validation limits | P1 |
| `/faq/` | FAQ | Product scope, status and technical questions | P1 |
| `/learn/` | Learning content | Publish only substantive, reviewed articles | Gated |

The route table above reflects the current marketing site. Do not infer that a route exists because it appears in an older proposal or because it is listed as a future candidate.

## Gated future routes

| Candidate route | Purpose | Release condition |
|---|---|---|
| `/calculators/` | Public brewing calculators | WP-09 formula sources, independent golden vectors, boundary tests and technical approval |
| `/use-cases/` | Detailed brewery scenarios | Each scenario mapped to approved product workflows and maturity |
| `/pricing/` | Plans, prices and limits | Pricing, billing, tax, cancellation and terms approved |
| Sign-in / app destination | Product access | Real customer-facing account and onboarding flow available |
| `/security/` | Security and data approach | Publish only verifiable architecture, controls and evidence |

## Homepage

Purpose: position Demleme for professional breweries within five seconds and route the visitor into the Business journey. Homebrewers remains a clearly visible secondary path.

Priority order:
1. Business-first hero, product maturity and primary CTA to `/breweries/`.
2. Secondary Homebrewers route to `/homebrewers/`.
3. Business and Home audience cards in that order.
4. Shared technical approach and verified feature overview.
5. Product status and scope limits.

No fake signup, demo, waitlist, trial, pricing or product UI is shown.

## Professional Breweries page

Primary commercial landing page. Focus on recipe standards, production context, batch records and technical clarity only where supported by the approved product plan. State ERP exclusions explicitly. Self-service evaluation is the intended motion; do not imply a demo-led sales process.

## Homebrewers page

A complete but secondary audience journey. Focus on home brewing recipes, calculations, ingredients, brew-day planning and learning from measurements. Keep Home-specific terminology and visual direction; do not use professional tank/role/QC workflows as defaults.

## Features and technical content

Every feature page states:
- Intended user problem and workflow.
- Inputs and outputs/records.
- Maturity label: Available, In development, Planned, Under consideration or Requires validation.
- Dependencies, limitations and evidence.
- Related pages and audience.

No feature is marked Available unless deployed and verified. Calculators remain gated until WP-09 evidence and product-owner approval pass.

## Navigation

Product, Breweries, Homebrewers, How it works, Technical principles and FAQ. Breweries appears before Homebrewers in desktop and mobile navigation. Pricing and Sign in remain hidden until real commercial and account destinations are approved.
