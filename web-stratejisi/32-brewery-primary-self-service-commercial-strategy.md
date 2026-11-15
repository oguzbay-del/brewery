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

- Product launch and Business onboarding date/URL
- Workspace, seat, batch and feature limits
- Free tier, trial and pricing
- Billing provider, tax/invoice and cancellation policy
- Business permissions, data ownership, security and support evidence
- Approved contact route for exceptional enterprise requirements
- Analytics provider, consent and retention policy

Until these are decided, the website must not claim that registration, trial, purchase, demo, pricing or enterprise support is live.

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

Current code snapshot: `2d00b265b5e739100dd90eeda20541d892200557`. GitHub Pages preview: https://oguzbay-del.github.io/brewery/ . Vercel Preview remains blocked by the Hobby build-rate limit; this does not indicate a failed site build.

Navigation accessibility work is in progress in PR #138: desktop and mobile navigation will identify the current page with `aria-current="page"`, including nested feature pages. Do not mark this work complete until its responsive and accessibility CI passes and the PR is merged.

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
