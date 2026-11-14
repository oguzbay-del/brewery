# Demleme — Marketing Site Implementation Specification & Release Gate

**Work package:** WP-12  
**Status:** User authorized implementation of the marketing website on 15 November 2026. Vercel Preview is deferred; GitHub Actions, source review and local/static checks continue. Production release remains a separate approval.  
**Date:** 14 November 2026  
**Inputs:** WP-03 through WP-11 and the existing marketing-site prototype.  
**Scope:** Public marketing website only.

---

## 1. Objective

Deliver one coherent, technically credible Demleme marketing website for two clearly separated audiences: Homebrewers and Professional Breweries. The website explains and positions the product; it is not the product application.

The target experience is:
- technically precise and transparent;
- editorial, modern, restrained and human;
- distinct in content and workflow for Home and Business;
- discoverable through useful search content;
- honest about product maturity and limitations;
- responsive and accessible;
- maintainable through reusable Astro components and centralized content data.

## 2. Decisions already fixed

| Area | Decision |
|---|---|
| Audiences | Homebrewers and Professional Breweries |
| Positioning | Technical |
| Markets | United States and United Kingdom first |
| Language | English-first; US editorial default |
| Commercial model | Self-service first, but no invented signup/trial/pricing/demo |
| Framework | Astro + TypeScript + static generation |
| Hosting | Vercel as current deployment target; GitHub is source of truth |
| Product boundary | Marketing site only; no product API/app implementation |
| ERP boundary | No sales, orders, distribution/shipping, keg tracking, CRM or accounting claims |
| Brand tokens | `tasarim-sistemi/tokens.json` remains canonical |
| Imagery | Existing five image slots retained; no unapproved sourcing or generation |
| Merge | Manual PR merge after GitHub checks and source review; Vercel visual QA deferred and recorded as a post-deployment gate; no Auto-merge |

## 3. Target sitemap

### First release
- `/` — Homepage.
- `/homebrewers/` — Home audience page.
- `/breweries/` — Professional Breweries page.
- `/features/` and verified `/features/{slug}/` — capability discovery.
- `/how-it-works/` — two distinct workflows.
- `/technical-principles/` — calculation, units, validation and product-scope principles.
- `/faq/` — only product-owner-approved answers.
- `/learn/` — publish only when substantive reviewed articles exist.

### Later / gated
- `/calculators/` — only after WP-09 validation and public-use approval.
- `/use-cases/` — only when grounded in approved product workflows.
- `/pricing/` — commercial model, billing, limits and terms approved.
- `/roadmap/` — public disclosure policy and stage vocabulary approved.
- Sign-in / Get started — actual customer-facing account and onboarding flow available.

No empty route, placeholder category or thin SEO page should be published.

## 4. Design and content requirements

### Global shell
- Header, mobile menu, footer, breadcrumbs and metadata handled consistently.
- Homebrewers and Breweries visible as primary audience paths.
- No dead navigation or placeholder social links.
- No pricing or account CTA until real destinations exist.

### Home page
- Hero with concise product positioning and two audience routes.
- Distinct audience panels.
- Brewing lifecycle overview.
- Technical clarity and evidence.
- Verified feature overview.
- Published learning resources only.
- Product maturity statement.

### Homebrewers page
- Home-specific visual and language.
- Recipe → ingredients/water/calculations → brew day → fermentation observations → batch learning.
- No professional tank/role/quality workflows as Home defaults.
- Predictions and device automation only if specifically verified and approved.

### Professional Breweries page
- Separate brewery-specific visual and process.
- Recipe review → production planning → production records → fermentation/vessels → QC/traceability (stage-labelled).
- Visible ERP exclusion statement.
- No compliance guarantees or financial/accounting claims.

### Feature detail pages
- Problem, intended workflow, inputs, outputs/records, stage, dependencies, limitations, evidence and related resources.
- One source of truth for stage vocabulary.
- No “Available” unless deployed and verified.
- Unapproved analytical models remain under consideration or omitted.

### Technical content
- Direct answer, context, method, worked example, limitations, sources and related links.
- All units explicit; distinguish US gallon, Imperial gallon and metric.
- No unreviewed formula, regulatory or security claim.

## 5. Implementation workstreams

| Workstream | Main tasks | Deliverable |
|---|---|---|
| A. Claim integrity | Integrate WP-03; reconcile feature data; resolve ambiguous stage labels | Authoritative claim map and corrected feature pages |
| B. Shared architecture | Astro layouts, metadata helper, header, footer, breadcrumbs, CTA and status components | Reusable accessible page shell |
| C. Visual system | CSS token generation/parity; typography, spacing, grid and component styles | Marketing design system aligned to canonical tokens |
| D. Audience pages | Rebuild Home and Business sections to WP-07 structures | Two distinct audience experiences |
| E. Feature catalogue | Status-aware index and detail pages; audience/lifecycle navigation | Accurate capability discovery |
| F. Core content | How It Works, Technical Principles, FAQ and Learn foundation | Complete first-release sitemap |
| G. SEO | Titles, descriptions, canonical, OG/Twitter, JSON-LD, sitemap, robots, redirects | Accurate crawl/index setup |
| H. QA | Responsive, accessibility, links, structured data, performance and visual QA | Recorded acceptance evidence |
| I. Release | Vercel preview, final product-owner review, production deployment | Approved public release |

## 6. Work order and dependency gates

1. Integrate WP-03 feature-claim audit into main and resolve product-owner contradictions.
2. Review PR #16 feature catalogue changes; merge only if claims and code pass the same stage/evidence gates.
3. Implement shared layout and metadata components.
4. Refine homepage and both audience pages.
5. Remediate feature directory and detail pages.
6. Add How It Works and Technical Principles.
7. Add FAQ and substantive Learn pages.
8. Add calculator route only after WP-09 formula validation and public-scope approval.
9. Complete SEO and structured data.
10. Run full responsive, accessibility, performance and link QA.
11. Obtain explicit final release approval and deploy.

No workstream may bypass product-claim, technical evidence or accessibility gates.

## 7. Quality acceptance criteria

### Content and claims
- [ ] Home and Business have distinct value propositions, examples, terminology and process diagrams.
- [ ] Every feature claim maps to product plan and approved status.
- [ ] Business exclusions are explicit and consistent.
- [ ] No unsupported predictions, AI, device, security, regulatory or performance claims.
- [ ] No fabricated customers, metrics, reviews, testimonials or pricing.

### Design and usability
- [ ] Existing canonical tokens are used without palette drift.
- [ ] Page hierarchy and section rhythm follow WP-06/07.
- [ ] No fake product UI or unapproved imagery.
- [ ] Navigation, disclosures and controls work with keyboard and touch.
- [ ] No horizontal overflow at 320 px.

### SEO and technical
- [ ] Unique metadata and canonical for each indexable route.
- [ ] Sitemap includes only published canonical routes.
- [ ] Robots and preview indexation are correct.
- [ ] JSON-LD matches visible facts.
- [ ] No broken internal links or dead CTAs.
- [ ] Astro check and production build pass.

### Accessibility and performance
- [ ] WCAG 2.2 AA target verified for new/changed components.
- [ ] Contrast, focus, heading order, labels, alt text and screen-reader flow checked.
- [ ] Reduced-motion and 200% zoom checked.
- [ ] Visual QA at 1440 / 1024 / 768 / 390 / 320 px.
- [ ] Lighthouse diagnostic and real-user Core Web Vitals plan recorded.
- [ ] Analytics and cookies remain absent until privacy/consent approval.

## 8. Release and merge protocol

- Every workstream uses a focused feature branch and PR.
- GitHub Actions checks and source/diff review must complete successfully. Vercel Preview and browser visual QA are deferred while Vercel is rate-limited; they remain mandatory before public release.
- PR must be clean/mergeable; changed files and diff are reviewed.
- Manual squash merge with expected head SHA only.
- No GitHub Auto-merge.
- Production deployment follows explicit release approval.
- GitHub Actions build success is not visual QA. Record Vercel Preview + desktop/mobile render QA as pending and complete them when Vercel is available.

## 9. Scope exclusions

This implementation specification does not authorize:
- product application, API, iOS, server or Rust engine changes;
- formula changes or calculator launch;
- photography purchase, sourcing or generation;
- analytics, tracking, forms or consent tools;
- public pricing, roadmap dates, customer proof or security/compliance badges;
- deployment to production without final approval.

## 10. Current state and next action

WP-03 claim audit and PR #16 require clean-base review/integration. WP-05 through WP-11 define the messaging, design, wireframes, SEO, calculator validation, content evidence and existing-site gaps.

**Implementation authorization:** The user explicitly authorized continued marketing-site code implementation on 15 November 2026, using GitHub as the source of truth while Vercel is unavailable. This authorizes the marketing website only. Product app/API/iOS/Rust, calculators, imagery sourcing/generation, analytics/forms/consent, pricing and public production release remain outside scope unless separately approved. GitHub Actions, static source checks and PR review continue. Vercel Preview, responsive browser QA and final public release remain deferred gates.
