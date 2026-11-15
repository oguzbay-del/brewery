# GRISTORA pricing, membership and self-service onboarding — implementation plan

**Decision owner:** Product/website owner  
**Date:** 15 November 2026  
**Status:** Website experience approved for implementation; monetary price points and payment operations remain launch decisions.  
**Scope:** Public marketing website and website-specific handoff only. No product app, account service, API, database, billing provider or Claude work is changed.

## 1. Commercial model

Two separate purchase journeys must be presented from the first decision:

- **Homebrewer:** one individual user; fixed one-seat plan; three-month introductory trial planned.
- **Professional Brewery:** team subscription, minimum five seats, increments of five (5 / 10 / 15 / 20). More than 20 seats routes to a direct email contact path, whose public address must be configured before launch. One-month introductory trial planned.

Both audiences can choose monthly, annual, two-year or three-year billing. Proposed discounts to display in the pricing preview are:
- Monthly: 0% (reference price)
- Annual: 15%
- Two years: 25%
- Three years: 35%

These are proposed commercial discounts, not published contractual offers. Monetary base prices are intentionally not fabricated: the previous FoamByte benchmark documents plan/billing transparency and a Free/Plus structure, but does not preserve verified numeric price points in this repository. Before launch, the owner must approve Home monthly price, Business per-seat monthly price, currency/tax display, discount basis, billing prepayment and cancellation terms. The discount percentages are editable constants in the pricing wizard and must be reconciled with approved final prices.

The intended pricing unit is per account for Home and per named user/seat for Business. A Business quote above 20 seats is an assisted email enquiry, not an automated price estimate.

## 2. Experience architecture

### Public routes
- `/pricing/`: plan overview, transparent trial/term summary, pricing wizard entry and clear pre-launch status.
- `/get-started/`: Apple-inspired, progressive step-by-step configuration flow. No giant form.
- `/compare/`: direct Homebrewer vs Professional Brewery function comparison table, with stage labels and links to the canonical feature detail pages.
- Existing `/features/`: full feature catalogue remains the source of detailed product scope. Pricing and audience pages link to the comparison table and feature details.

### Wizard sequence
1. **Choose your brewing path:** Homebrewer or Professional Brewery. Explain who each is for; link to the function comparison.
2. **Choose seats:** Home fixed at one. Business selects 5, 10, 15 or 20; over 20 goes to the configured sales email. Never silently cap or price a larger team.
3. **Choose billing:** monthly, annual, two-year or three-year. Show the proposed discount against monthly reference, term, and trial duration.
4. **Review:** audience, seat count, billing term, trial length, proposed discount, and payment method intent (credit/debit card). Do not show a fabricated amount.
5. **Checkout handoff:** payment integration is a later phase. For now the flow ends in a clear “not connected yet” state; no account, subscription, card data or payment is collected. Never present the preview as a working checkout or active trial.

A progress indicator, one decision per step, back/continue actions, keyboard operation, focus management, mobile layout and a persistent summary create the Apple-like guided feel. Use accessible native controls and no external UI dependency.

## 3. Work packages and parallel execution

| WP | Package | Deliverable | Depends on | Can run in parallel with |
|---|---|---|---|---|
| P0 | Commercial rules and handoff | This decision record, pricing constants, open-decision register | Owner direction | P1–P4 |
| P1 | Pricing landing page | `/pricing/`, audience plan cards, trial/term summary, comparison links | P0 | P2, P3, P4 |
| P2 | Guided configuration | `/get-started/`, four-step stateful wizard, summary and non-live checkout state | P0 | P1, P3, P4 |
| P3 | Feature comparison | `/compare/`, two-audience feature matrix, stage-safe labels, links to feature catalogue | Product feature catalogue (read-only) | P1, P2, P4 |
| P4 | Site integration | Header/footer/navigation links, audience page CTAs, homepage pricing entry, sitemap and FAQ updates | P1–P3 route names | P1–P3 |
| P5 | Quality and release | Astro/TS, generated-route validation, link/SEO, responsive Playwright and axe, screenshot review | P1–P4 | Documentation review |
| P6 | Billing activation (future, not this task) | Real auth, trial provisioning, payment provider, subscription lifecycle, receipts/tax, cancellations, webhook and entitlement handling | Product/API decisions, legal and provider approval | Separate product workstream only |

Parallel sessions should own separate packages/files and coordinate through this document. P1, P2 and P3 are intentionally independent. P4 begins once route names are fixed. P5 gates merge. P6 is explicitly out of scope.

## 4. Product and marketing guardrails

- The product remains in development. The wizard is a website configuration preview, not registration or checkout.
- Trial terms are proposed plan terms and are not activated until account provisioning and entitlement logic exist.
- Do not request, store or transmit card details. Do not add Stripe, Paddle, Apple, or other provider SDKs.
- Do not fabricate prices, savings in currency, tax, invoices, refunds, cancellation, uptime, security or support guarantees.
- No unsupported claims about Home/Business feature availability. The comparison table distinguishes planned stage from live availability.
- Business is the primary commercial path; Homebrewers remains a complete, separate secondary path.
- Business seat selection is 5-seat increments through 20. More than 20 is an email enquiry after a real sales address is configured.
- Feature comparison is an orientation tool; the existing feature catalogue remains canonical for detailed scope and maturity.
- No product code, product roadmap, task queue, APIs, Rust, database, iOS, or Claude instructions may be changed.

## 5. Launch decisions still required

1. Home monthly base price and Business monthly per-seat base price.
2. Confirm proposed 15% / 25% / 35% discounts and whether applied to pre-tax monthly equivalent.
3. Currency policy for US and UK, FX policy if relevant, tax/VAT display and invoice entity.
4. Whether multi-year terms are paid upfront or billed annually; renewal and price-lock rules.
5. Trial eligibility, start trigger, card requirement during trial, conversion timing and cancellation.
6. Public sales/contact email for Business teams above 20 seats.
7. Billing provider, supported cards, 3DS/SCA, refunds, cancellations, receipts and support path.
8. Privacy, data retention, legal terms and consumer/business jurisdiction review.
9. Verified app registration URL and provisioning flow.

## 6. Acceptance criteria

- A visitor can distinguish Home and Business before seeing a package.
- Home always resolves to one user; Business offers only 5-seat increments through 20 and a contact route beyond 20 once configured.
- Monthly, annual, two-year and three-year options visibly show proposed discount and correct trial duration.
- The wizard supports forward/back, selected-state persistence while navigating, accessible labels/status and a clear final summary.
- The comparison table is linked from pricing and wizard and links to feature detail pages.
- All public copy says the product is in development and the trial/checkout are not active.
- No card fields, network calls, account creation or payment collection exist.
- Navigation, sitemap, metadata, responsive and accessibility checks pass.
- Website-only changes; manual merge only after CI passes. No production launch.

## 7. Benchmark note

The existing FoamByte benchmark (WP-13, `web-stratejisi/24-foambyte-benchmark-derinlemesine.md`) records a Free/Plus structure and the importance of visible plan limits, billing and checkout transparency. It does not record numeric competitor prices or a verified currency/price table. Therefore this implementation adopts the transparent subscription structure, but leaves monetary prices as owner-approved launch inputs rather than inventing a competitor-derived figure.
