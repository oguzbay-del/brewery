# Commercial strategy — Professional Breweries first, self-service by design

**Decision date:** 15 November 2026  
**Status:** Canonical commercial direction for the Demleme marketing website.  
**Scope:** Positioning, audience priority, acquisition and website conversion only. This does not approve product functionality, pricing, onboarding, analytics or production release.

## 1. Strategic decision

- **Primary commercial market:** Professional Breweries (Business).
- **Secondary audience:** Homebrewers (Home), retained as a separate product path and content audience.
- **Commercial motion:** Self-service first. The website should educate, qualify and enable independent product evaluation; a demo-led sales process is not the default.
- **Initial geography:** United States and United Kingdom.
- **Primary language:** English.
- **Product boundary:** Demleme is brewing software, not a general ERP. Do not imply sales, order management, shipping/distribution, keg tracking, CRM, invoicing or accounting.

“Both audiences” describes product reach, not equal commercial priority. Resource allocation, homepage hierarchy, primary CTA, acquisition content and conversion measurement should lead with Business.

## 2. Ideal initial Business audience

Start with independent small and medium-sized breweries and the people responsible for repeatable production: brewery owners, head brewers, production leads, operators and quality staff where the relevant product scope is verified. Larger multi-site or enterprise breweries are a later segment until permissions, security, data ownership, onboarding and support capabilities are verified.

## 3. Business value proposition

**Bring consistency to every batch. Give your brewing team a clearer technical workspace.**

Message pillars:
1. Recipe standards — make recipe intent and approved versions easier to understand.
2. Production context — connect planned work with actual batch records where the roadmap supports it.
3. Measurement clarity — distinguish targets, observations and calculated values.
4. Team continuity — reduce dependence on individual memory through structured records, without claiming unverified collaboration or access-control features.
5. Technical transparency — explain units, assumptions, method limits and validation status.

Every statement remains stage-labelled against the approved product plan. No customer outcomes, efficiency gains, compliance guarantees, security certifications or integrations may be invented.

## 4. Self-service acquisition journey

1. **Discover:** Search/content entry focused on brewery recipe management, batch records, production consistency and brewing workflow problems.
2. **Understand:** `/breweries/` explains the Business audience, relevant workflow, roadmap status and explicit ERP exclusions.
3. **Evaluate:** Feature pages, How It Works, Technical Principles and FAQ answer fit, inputs, outputs, limitations and maturity.
4. **Start:** Only when a real Business onboarding URL is available, offer account/workspace creation. Until then, keep CTAs informational and honest.
5. **Activate:** The intended first-value milestone is a meaningful Business recipe or batch record; actual onboarding steps must match shipped product.
6. **Convert:** Pricing, limits, billing and paid-plan messaging appear only after commercial decisions and working payment paths are approved.

Do not insert a demo call, lead form, waitlist, trial or email capture as a substitute for self-service. Any future assisted-sales path is secondary and requires a real, staffed process.

## 5. Website hierarchy

- Homepage: Business-first positioning; primary CTA to `/breweries/`; secondary Homebrewers route.
- Header: Breweries before Homebrewers.
- Audience cards: Professional Breweries first, Homebrewers second.
- Breweries landing page: primary commercial landing page.
- Features and How It Works: Business examples and production records first where the feature is relevant; preserve Home-specific differences.
- Homebrewers landing page: a complete, distinct journey, not a deleted or hidden segment.
- Learn/SEO: prioritise professional brewing intent and Business problem clusters; retain high-value homebrewing educational topics where relevant.

## 6. Measurement model

Primary funnel: Business-intent sessions → Breweries page engagement → relevant feature progression → Business CTA click → account/workspace start and completion after onboarding exists → first meaningful Business recipe/batch record → team activation → paid conversion and retention.

Track Home metrics separately. Do not optimise for raw traffic or Homebrewers content volume at the expense of qualified Business acquisition.

Analytics vendor, consent requirements, event payloads and retention must be approved before collection is implemented.

## 7. Decisions intentionally still open

The website now has a pricing overview, a two-audience function comparison and a guided plan-configuration preview. Commercial structure is defined as a proposal, but monetary prices and transactional operations remain open.

- Product launch and verified account/onboarding URL
- Home monthly base price and Business monthly price per seat
- Approval of proposed term discounts: 15% annual, 25% two-year, 35% three-year
- Currency, tax/VAT, multi-year prepayment, renewal and cancellation terms
- Trial activation, card requirement and conversion rules
- Billing provider, invoices, refunds and payment operations
- Public sales email for Business teams above 20 seats
- Business permissions, data ownership, security and support evidence
- Analytics provider, consent and retention policy

The public website must clearly label plan selection, trials and checkout as previews until real account and billing services are approved and available.

## 8. Immediate website changes

1. Put Professional Breweries first in homepage hero CTA, audience-card order and navigation.
2. Make the Business proposition the lead in homepage metadata and introduction.
3. Keep Homebrewers as a clearly visible secondary route.
4. Update strategic brief, conversion plan, implementation specification and handoff references to remove equal-priority ambiguity.
5. Add Business-first acquisition intent to content briefs and future SEO planning.
6. Re-run route, metadata, accessibility and responsive QA. No Vercel deployment is required for this source change.

## 9. Website implementation status — 15 November 2026

The following website changes implement the Business-first direction. They change marketing hierarchy and page descriptions only; they do not change product availability.

| Work | Result | Evidence |
|---|---|---|
| Business-first homepage feature order | Brewery-only product areas appear before shared Home/Business areas; the existing feature catalogue remains the source | PR #129 |
| FAQ audience hierarchy | Professional brewery teams lead the FAQ title, description and introduction; Homebrewers remain a separate secondary path | PR #131 |
| Brewery landing metadata | Search and social descriptions explain the planned professional brewery workflows and state that the product is in development | PR #134 |
| Automated browser and static checks | Marketing-site CI, generated-route validation, responsive browser QA and axe checks pass on the deployed code snapshot | CI run #197 |
| GitHub Pages preview | The PR #134 code snapshot is published successfully | Pages deployment #65 |
| Current-page navigation | Desktop/mobile current-page semantics and visible state, including feature detail routes | PR #138; main CI run #43710054711 (85/85); Pages run #43710054632 |
| Analytical-claim guardrails | Unapproved analytical models are separated from planned feature copy; generated HTML checks enforce the caveats | PR #141; main CI run #43710624091 (85/85, 1,745 static checks); Pages run #43710624041 |

Current main snapshot: `53af627bdd9881efe3435d3b78a0d355dd2ee6b7`. GitHub Pages preview: https://oguzbay-del.github.io/brewery/ . Vercel Preview remains blocked by the Hobby build-rate limit; this does not indicate a failed site build. The current-page navigation and analytical-claim guardrails are included in the published main snapshot.

Navigation accessibility is complete. PR #138 adds a visible current-page state and `aria-current="page"` to desktop and mobile navigation, including nested feature pages. Main CI run #43710054711 passed 85/85 browser tests and the GitHub Pages deployment run #43710054632 succeeded on the current main commit.

## 10. Remaining release decisions and gates

The following remain intentionally unresolved and must not be replaced with invented defaults:

- Public production domain and commercially appropriate hosting plan.
- Verified app, sign-in and Business onboarding URLs.
- Pricing, plans, limits, billing, trial and cancellation terms.
- Privacy/legal copy, analytics provider, consent and retention.
- A real, staffed contact route if an assisted path is later approved.
- Human review of keyboard/focus behavior, screen-reader flow and actual 200% browser zoom.
- Final-domain canonical, sitemap, robots and social-preview validation.

Until those decisions are approved, keep the site informational and Business-first. Do not add signup, trial, purchase, demo, waitlist or lead-capture CTAs.

The feature-claim audit now distinguishes basic planned records/calculations from research-only analytical models. Models 1–3, 6 and 9–15 remain unapproved in the product plan and are labelled “Under consideration” wherever they appear in marketing copy. PR #141 does not approve or change those product decisions.


## 11. Pricing and membership implementation — 15 November 2026

The owner has approved the following website-level commercial structure for implementation:

- Homebrewer: one user; proposed three-month introductory trial.
- Professional Brewery: minimum five users, seat increments of five through 20; teams above 20 follow a custom email enquiry path; proposed one-month introductory trial.
- Billing choices for both audiences: monthly, annual, two-year and three-year.
- Proposed discounts versus the monthly reference: 0%, 15%, 25% and 35%, respectively.
- Pricing overview, Apple-inspired step-by-step configuration wizard and Home-vs-Brewery function comparison are separate public routes.

Implementation plan and dependencies are in `web-stratejisi/45-gristora-pricing-membership-onboarding-plan.md`. The exact monthly price points are intentionally not fabricated: the FoamByte benchmark records Free/Plus and billing transparency but does not contain verified numeric competitor prices. The website preview does not collect cards, create accounts, start trials or process payments. Real billing is a separate future product/infrastructure workstream.

Current work branch: `marketing/gristora-pricing-onboarding`. Merge only after static, generated-route, responsive and accessibility CI pass.
