# GRISTORA marketing website — parallel follow-through plan

**Date:** 15 November 2026  
**Scope:** Public marketing website only. Production remains intentionally offline.  
**Purpose:** Continue improving the visitor journey in independently mergeable packages while preserving the boundary between a marketing preview and a functioning SaaS product.

## Current baseline

- Pricing, Homebrewer vs Professional Brewery comparison and the guided plan configurator are merged.
- Home is one user with a proposed three-month trial.
- Professional Brewery starts at five seats and scales in increments of five through 20, with a proposed one-month trial.
- More than 20 seats routes to a custom-quote explanation; the sales email remains a launch input.
- Proposed billing terms: monthly, annual (15% discount), two years (25%), three years (35%).
- Monetary prices, currency/tax, multi-year payment collection, and contract terms are deliberately unpublished until approved.
- Card payments, registration, account provisioning and trial activation are not implemented.
- GitHub Pages is a noindex preview, not the production site. Do not connect the domain or claim launch.

## Parallel work packages

| ID | Package | Deliverable | Files owned | Dependencies | Gate |
|---|---|---|---|---|---|
| W7 | Homepage commercial entry | Add clear pricing and compare entry points without displacing Business-first positioning | `marketing-site/src/pages/index.astro` | W0 baseline | CI + responsive |
| W8 | Wizard clarity | Show trial in persistent summary; explain 20+ custom quote without pretending a live request was sent | `marketing-site/src/pages/get-started.astro` | W0 baseline | CI + wizard browser tests |
| W9 | Comparison comprehension | Explain stage labels, accessible table caption, pricing/configuration links and mobile behavior | `marketing-site/src/pages/compare.astro`, `src/styles/comparison.css` | Read-only feature catalogue | CI + accessibility + responsive |
| W10 | Funnel analytics specification | Define privacy-conscious events and measurement plan only; no tracking SDK until consent/legal review | website-specific documentation | W7–W9 merged | Documentation review |
| W11 | Content consistency audit | Check every public route for consistent development, trial, price, payment and availability disclosures | website routes + audit report | W7–W9 merged | Route crawl + copy review |
| W12 | Final website release readiness | Full links, metadata, structured data, sitemap, robots, 404, visual QA, accessibility and performance pass | marketing site only | W10–W11 | All CI gates green |

W7, W8 and W9 are intentionally parallel and own separate route files. W10 can be drafted in parallel but should consume the final wording and event destinations from W7–W9. W11 begins after those merges to avoid auditing stale content. W12 is the final sequential release gate.

## Future, owner-dependent decisions (not blockers for preview development)

1. Approve Home monthly price and Professional Brewery per-seat monthly price.
2. Approve 15%, 25%, and 35% discounts and their calculation basis.
3. Confirm USD/GBP currency presentation, tax/VAT and invoicing entity.
4. Decide if two- and three-year terms are prepaid or annually billed; define renewal and price-lock rules.
5. Approve trial eligibility, activation trigger, card requirement, conversion and cancellation terms.
6. Provide the public sales email for teams above 20 seats.
7. Approve privacy, retention, consumer/business jurisdiction and legal copy.
8. Confirm the real application registration URL and the user provisioning journey.
9. Separately authorize and scope the billing/account product workstream.

## Non-negotiable boundaries

- Keep all implementation in `marketing-site/` and website-specific documentation in `web-stratejisi/`.
- Product specifications and feature catalogue are read-only inputs.
- No product app, API, database, Rust engine, iOS, task queue, or Claude work.
- No payment provider SDK, card fields, account creation, live email submission, analytics SDK or tracking pixels in the preview.
- Never invent monetary prices, savings in currency, tax, contractual terms, contact addresses or product availability.
- Keep production domain, DNS, Vercel and public launch deferred.
- Manual merge only after CI passes; no auto-merge.

## Session handoff

A new session should:
1. Read this document and `marketing-site/CHATGPT-SCOPE.md`.
2. Inspect open marketing PRs and verify their head SHA and CI before editing.
3. Own one package/file set from the table; avoid overlapping edits.
4. Run the marketing CI and responsive/accessibility gates.
5. Merge manually only after checks pass.
6. Update this document with merged PR numbers, remaining decisions and next package.
7. Stop when W12 passes. Do not expand into product engineering.
