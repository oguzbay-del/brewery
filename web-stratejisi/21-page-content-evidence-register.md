# Demleme — Page Content Backlog & Evidence Register

**Work package:** WP-10  
**Status:** Editorial backlog and evidence ownership model; no production copy or publishing authorization.  
**Date:** 14 November 2026  
**Depends on:** WP-03 claim audit, WP-04 sitemap, WP-05 messaging, WP-07 wireframes, WP-08 SEO standard, WP-09 calculator validation.  
**Markets:** US and UK; English-first.

---

## 1. Operating model

This register turns the strategy into a controlled content queue. It is not a promise that every listed capability is available or that every page should be published immediately.

**Publication states**
- **Brief:** page purpose and audience defined.
- **Draft:** content drafted; claims not yet approved.
- **Technical review:** formula, workflow, units, regulatory or security review in progress.
- **Product review:** product owner confirms capability stage and scope.
- **Approved:** evidence, editorial, SEO and product checks passed.
- **Published:** live route verified and indexed appropriately.
- **Hold:** blocked by a product decision, missing evidence, or no real destination.
- **Retired:** page no longer maintained; redirect/noindex plan required.

Every page has one accountable content owner role and named reviewers assigned by the project owner before drafting. Do not invent author names or credentials.

## 2. Priority backlog

| ID | Page / asset | Audience | Priority | Intent / purpose | Required evidence | Dependency | Initial state |
|---|---|---|---:|---|---|---|---|
| CNT-001 | Homepage | Both | P0 | Brand discovery and audience routing | Approved positioning, current product maturity, working routes | WP-05/07 | Brief |
| CNT-002 | Homebrewers | Home | P0 | Homebrewing software evaluation | Home scope, feature stages, formula claims | WP-03/05/07 | Brief |
| CNT-003 | Professional Breweries | Business | P0 | Professional brewing workflow evaluation | Business scope, stages, ERP exclusions | WP-03/05/07 | Brief |
| CNT-004 | Features index | Both | P0 | Capability discovery | 16-feature claim map, stage vocabulary | WP-03; PR #16 review | Hold |
| CNT-005 | Feature detail pages | Mixed | P1 | Explain one real user problem and capability | Product-plan row, audience, stage, limitations | WP-03 | Hold until each claim mapped |
| CNT-006 | How It Works — Home | Home | P1 | Explain Home journey | Approved workflow and stage labels | WP-04/07 | Brief |
| CNT-007 | How It Works — Business | Business | P1 | Explain production workflow | Production/tank/QC scope and stage labels | WP-04/07 | Brief |
| CNT-008 | Technical Principles | Both | P1 | Establish technical trust | Formula sources, unit rules, validation approach | WP-08/09 | Hold for source review |
| CNT-009 | Brewing calculations hub | Both | P1 | Explain calculation topics and route to tools | Public calculator scope approval | WP-09 | Hold |
| CNT-010 | Gravity / OG guide | Both | P1 | Answer gravity/extract question | Method, reference vectors, unit and measurement definitions | WP-09 | Technical review |
| CNT-011 | ABV guide | Both | P1 | Explain estimate vs measurement | Approved method, uncertainty and examples | WP-09 | Technical review |
| CNT-012 | Recipe scaling / volume guide | Both | P1 | Explain scaling and losses | Definitions, conversion tests and golden examples | WP-09 | Technical review |
| CNT-013 | IBU guide | Both | P2 | Explain bitterness calculation | Selected method and source version | WP-09 | Hold |
| CNT-014 | Colour guide | Both | P2 | Explain beer colour methods | Method and reference examples | WP-09 | Hold |
| CNT-015 | Water report guide | Home + Business | P2 | Explain water ions and units | Source references and supported ions | WP-09 | Technical review |
| CNT-016 | Mash pH guide | Home + Business | P2 | Explain measurement and estimate limits | Model decision and limitations | WP-03/09 | Hold |
| CNT-017 | Fermentation guide | Both | P2 | Explain measurements and observations | Product stage, measurement/device support | WP-03 | Brief |
| CNT-018 | Business lot-to-batch traceability use case | Business | P2 | Explain professional records | Data relationships and release stage | WP-03 | Hold |
| CNT-019 | Quality / QC guide | Business | P2 | Explain checks and records | Product scope, role and record behavior | WP-03 | Hold |
| CNT-020 | FAQ | Both | P1 | Resolve product and scope questions | Approved product, platform, privacy and commercial answers | WP-03/05 | Brief |
| CNT-021 | Pricing | Both | P3 | Commercial evaluation | Approved packages, limits, billing and terms | Product/commercial decision | Hold |
| CNT-022 | Roadmap | Both | P3 | Public maturity transparency | Public disclosure policy and approved vocabulary | Product-owner approval | Hold |
| CNT-023 | About | Both | P3 | Product philosophy and context | Verified company/product facts and author details | Product owner | Hold |

## 3. Audience content differentiation register

| Content dimension | Homebrewers | Professional Breweries |
|---|---|---|
| Core task | Formulate, calculate, brew, measure and learn | Review recipes, plan production, record batches and quality |
| Primary examples | Recipe size, ingredient weights, gravity, water, brew-day readings | Batch IDs, production plan vs actual, vessels, ingredient lots, QC records |
| Language | Recipe, brew day, fermenter, measurement, batch | Recipe approval, production plan, tank/vessel, lot, batch record |
| Visual context | Homebrew bench, kettle, ingredients, measurement | Brewhouse, stainless vessels, operator workflow |
| Technical depth | Explain methods and practical assumptions | Explain repeatability, records, roles and traceability |
| Primary content CTA | Guide, validated calculator, Home feature | Business workflow, scope and relevant feature |
| Exclude | Enterprise roles, professional compliance and industrial control as defaults | ERP, sales, orders, shipping, keg tracking, CRM, accounting |
| Evidence emphasis | Formula source, measurement and limitations | Workflow, record model, permissions and audit evidence |

A page that merely swaps the audience label but keeps the same examples and story is not considered audience-specific.

## 4. Claim and evidence register schema

Maintain one row per material public claim. Required fields:

| Field | Definition |
|---|---|
| Claim ID | Stable identifier, e.g. `CLM-HOME-001` |
| Page / section | Exact route and section |
| Claim text | Exact proposed sentence or visual label |
| Audience | Home / Business / Both |
| Product-plan reference | Exact heading or row |
| Stage | Available / In development / Planned / Under consideration / Requires validation |
| Evidence | Test, reference, deployed workflow, approved product decision |
| Evidence owner | Role responsible for supplying proof |
| Reviewer | Technical / product / legal reviewer as relevant |
| Limitations | Assumptions, unsupported cases and caveats |
| Last verified | Date of evidence check |
| Expiry trigger | Release change, formula change, regulation change or scheduled review |
| Decision | Approved / revise / remove / hold |

No claim is approved solely because it appears in the product plan or source code. Availability requires a deployed and verified customer-facing path.

## 5. Evidence pack by content type

| Content type | Minimum evidence pack | Additional review |
|---|---|---|
| Audience landing page | Product scope, audience journey, verified stage labels, working links | Product owner + editor |
| Feature page | Product-plan mapping, claim row, stage, inputs/outputs, limitations | Product owner + technical reviewer |
| Calculator page | Method/version, authoritative source, golden vectors, boundary/unit tests, tolerance, public-use approval | Formula owner + product owner |
| Technical article | Primary sources, definitions, units, examples, caveats, review date | Subject-matter reviewer |
| Device page | Exact model/protocol, read/control direction, firmware and supported stage | Integration owner |
| Security/privacy page | Implemented controls, policy, data flow, review evidence and jurisdiction | Security/privacy reviewer |
| Regulatory article | Official current jurisdiction sources and specialist review | Legal/regulatory specialist |
| Use case | Approved workflow, actual product stage and non-fictional example | Product owner |
| Customer story | Customer consent, verifiable facts, methodology and approved quote | Customer + legal |
| Pricing page | Approved plan, limits, billing, taxes, terms and working checkout | Commercial + legal |

## 6. Content production sequence

1. Select one P0 page and lock its search intent.
2. Create a page brief with audience, section structure and internal links.
3. Build a claim list for every product statement.
4. Collect evidence before writing benefit language.
5. Draft concise content with explicit limitations.
6. Technical reviewer checks formulas, units, workflows and sources.
7. Product owner checks stage and scope.
8. Editor checks clarity, terminology and audience distinction.
9. SEO and accessibility review.
10. Publish only after the destination and all CTA actions work.
11. Record live URL, publication date and review trigger.
12. Revalidate after product releases, formula changes or source/regulatory changes.

## 7. Initial production order

**Wave 1 — Product identity and audience fit**
1. Homepage.
2. Homebrewers.
3. Professional Breweries.
4. Features index (after claim mapping is available on main).
5. How It Works — Home and Business.

**Wave 2 — Technical trust**
6. Technical Principles.
7. Gravity / OG and ABV explainers.
8. Recipe scaling / volume.
9. Water and mash only after formula/model review.
10. FAQ with product-owner-approved answers.

**Wave 3 — Professional depth and utility**
11. IBU and colour explainers.
12. Fermentation and device support pages.
13. Business traceability and QC use cases.
14. Calculators only after WP-09 acceptance and explicit public-use approval.

**Hold**
- Pricing, public roadmap dates, customer stories, testimonials, security badges, compliance guarantees, AI/prediction claims and any page without evidence.

## 8. Release readiness scorecard

A page is ready only when all critical gates pass. Do not average a failed product-claim gate away with a good SEO score.

| Gate | Pass / fail |
|---|---|
| Audience and search intent are unique | Required |
| Product scope and maturity verified | Required |
| Material claims have evidence | Required |
| Technical and regulatory content reviewed where relevant | Required |
| CTA destinations work | Required |
| SEO metadata, canonical and indexation correct | Required |
| Mobile, accessibility and performance checks recorded | Required |
| Privacy / consent reviewed for scripts and forms | Required |
| Editorial and legal approval complete where applicable | Required |

## 9. Current blockers and decisions

- WP-03 feature claim audit exists on an open historical PR (#19) and is not yet on main; review/clean-base integration is required before treating its document as the canonical repo reference.
- PR #16 contains feature catalogue work; it remains separate from the strategy documents and must be reviewed for current diff, claims and implementation readiness before merge.
- Public calculator scope and formula ownership remain product-owner decisions.
- Analytics provider, consent approach, pricing, public roadmap disclosure and real account CTA destinations remain unapproved.
- Real photography and actual product screenshots remain user-provided/approved asset dependencies.

## 10. Next work package

**WP-11 — Existing marketing-site implementation gap analysis.** Map the current `marketing-site/` routes, content, components, SEO metadata and CTA behavior against WP-04 through WP-10. Produce a prioritized, evidence-backed gap list and implementation sequence. Do not change production code until the implementation gate is explicitly approved.
