# GRISTORA — Public marketing claims and content guardrails

## Ürün sahibi kararı, 16 Kasım 2026

Ürün sahibi kararı (16 Kasım 2026, Oğuz): herkese açık site ürün olgunluğu uyarılarını göstermez. Ürün–site tutarlılığını ürün sahibi doğrular. Bu karar, aşağıdaki "uyarılar görünür kalmalı" kuralının yerine geçer. Müşteri, ölçü, fiyat, entegrasyon, sertifika, uyumluluk ve tahmin/AI iddiası ekleme yasağı geçerlidir.

**Status:** WP-17 preparation; source audit of current `main` branch on 15 November 2026.  
**Scope:** Public marketing content only. This is not product specification authority and does not approve release claims.

## 1. Product maturity statement

The current marketing site repeatedly and explicitly states that the product and website are in development, are not a production service, and do not offer signup, a trial or live production workflows. Preserve this disclosure through the brand transition until the actual product release state changes and is independently verified.

Approved-safe framing:
- “In development”
- “Planned workflow”
- “The roadmap includes…”
- “Being designed for…”
- “Subject to implementation, testing and validation”

Do not convert roadmap language into present-tense availability claims during rebranding.

## 2. Audience and positioning

- Primary commercial audience: Professional Breweries.
- Secondary audience: Homebrewers, with a distinct journey.
- Positioning: technical brewing software/workspace, not a generic brewery ERP.
- The primary public conversion path is discovery of the professional brewery workflow; Homebrewers remains visible but secondary.

## 3. Current feature catalogue — all 16 areas are roadmap descriptions

The current feature catalogue data source is `marketing-site/src/data/features.ts`. It contains 16 feature areas with stage tags; the tags are roadmap metadata, not evidence of production availability.

| Feature area | Current stage tags | Public-copy constraint |
|---|---|---|
| Recipe design & management | Homebrew MVP; Business core | Planned recipe/version workflows; do not imply released functionality. |
| Brewing calculations | Homebrew MVP; Business core; Business second release; Under consideration | Validation is required. No claim of validated or live calculator results until the release gate passes. |
| Water, mash & yeast | Homebrew MVP; Under consideration | Keep specialist methods and model-dependent estimates qualified. |
| Brew day & production records | Homebrew MVP; Business core | Planned workflow; retain target-versus-actual distinction. |
| Fermentation & batch tracking | Homebrew MVP; Business core | Manual/connected observations are planned; predictive outcomes are not approved. |
| Equipment & calibration | Homebrew MVP; Business core | Equipment modelling and measured calibration are planned, not live. |
| Planning & scheduling | Homebrew MVP; Business core; Business second release | Professional scheduling is roadmap functionality; no current operational availability claim. |
| Inventory, ingredients & costing | Homebrew MVP; Business core; Business second release; Under consideration | Lot-level traceability and cost functions require stage-specific qualification; no ERP claim. |
| Brewery production management | Business core; Business second release | Limited brewing-production workspace; explicitly not a general-purpose ERP. |
| Quality, laboratory & release | Business core; Business second release; Under consideration | Quality records, lot relationships and release workflows require product and regulatory validation. |
| Packaging, CIP & compliance | Business core; Business second release | Planned professional-production scope; do not imply regulatory certification or automatic compliance. |
| Teams, roles & notifications | Homebrew MVP; Business core | Planned role/notification workflows; no live team availability claim. |
| Connected devices & integrations | Homebrew MVP; Business second release; Later; Under consideration | Read-only fermentation device data is the roadmap starting point; professional sensor/gateway scope is later. No integrations are currently publicly available. |
| Import, export & brewing tools | Homebrew MVP; Homebrew second release | Mention named import formats only with a verified implementation/release gate. |
| Platform, security & data ownership | Homebrew MVP; Business core | Do not claim certifications, audits, uptime/SLA or production security posture without evidence. |
| Reports, insights & sustainability | Homebrew MVP; Business core; Business second release; Under consideration | Separate operational reporting from research-only analytical ideas; no AI/predictive claim without approval. |

## 4. Explicit negative claims to preserve

Current FAQ and audience pages explicitly rule out:
- General-purpose brewery ERP.
- Sales, customer orders, distribution, shipping, keg tracking, CRM and accounting.
- Public calculators in the current website release.
- Predictive final gravity or fermentation finish-date capabilities.
- Publicly available APIs, webhooks and device integrations.
- Current production availability, signup, trial, pricing and free-tier claims.

Do not soften these boundaries in new brand copy. If product scope changes, update the product-authoritative source first and require evidence before changing marketing text.

## 5. High-risk claims requiring evidence before publication

| Claim type | Minimum evidence before use |
|---|---|
| “Live”, “available”, “production-ready” | Release approval, deployed product evidence, operational support readiness. |
| Calculation accuracy or validated results | Approved reference vectors, real-brew validation, documented method/version and QA. |
| Integrations / device compatibility | Named device or API, implemented connector, test evidence and support boundary. |
| Traceability / compliance | Exact jurisdiction and process scope, product evidence, regulatory/legal review. |
| Security / privacy / data ownership | Implemented controls, policy, architecture review, test/audit evidence where claimed. |
| AI / predictive analytics | Approved model, validation data, documented limitations and explicit product release approval. |
| Customers, case studies, metrics or testimonials | Verifiable customer permission and source evidence. |
| Pricing, trial, free tier or self-service signup | Approved commercial decision and functioning user journey. |

## 6. Brand-transition content principles

- Keep the product maturity disclosure visible in the homepage hero, audience pages, feature directory and FAQ.
- Use GRISTORA as the public brand and “Brewing Intelligence” as the descriptor; do not invent module brands.
- Do not replace the current editorial illustrations with fabricated product screenshots.
- Current social-card source is an abstract brewing-vessel illustration and legacy wordmark, not a product screenshot. Replace its brand layer only after the master logo is approved; keep the art direction aligned to the selected logo and avoid brewing clip-art clichés.
- Keep page title, canonical metadata, Open Graph, Twitter metadata, JSON-LD, navigation, accessible names and favicon consistent after the rebrand.
- Preserve all current route semantics and audience hierarchy.

## 7. Source files inspected

- `marketing-site/src/pages/index.astro`
- `marketing-site/src/pages/breweries.astro`
- `marketing-site/src/pages/homebrewers.astro`
- `marketing-site/src/pages/how-it-works.astro`
- `marketing-site/src/pages/technical-principles.astro`
- `marketing-site/src/pages/faq.astro`
- `marketing-site/src/pages/features/index.astro`
- `marketing-site/src/data/features.ts`
- `marketing-site/public/social-card-source.svg`
- `marketing-site/public/favicon.svg`

This inventory covers the inspected high-level routes and catalogue source, not every dynamic feature route or every test assertion. WP-17 must still run a repository-wide scan and generated-site regression tests.

## 8. Scope boundary

This document authorises no public-site source change by itself. It does not authorise product application, API, Rust engine, iOS app, data model, internal identifier, roadmap or product-development queue changes.
