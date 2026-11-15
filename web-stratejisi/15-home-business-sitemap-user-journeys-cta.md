# Demleme — Home & Business Sitemap, User Journeys and CTA Architecture

> **Commercial-priority update (15 November 2026):** Professional Breweries (Business) is the primary commercial audience and self-service acquisition path. Homebrewers (Home) remains a distinct secondary audience. The route and journey design below still applies to both; where priority, homepage order, content investment or conversion measurement is unspecified, Business takes precedence. This decision supersedes older wording that implied equal commercial priority.

**Work package:** WP-04  
**Status:** Strategy proposal complete; product-owner and user review gates remain open.  
**Date:** 14 November 2026  
**Audience:** Homebrewers (Home) and Professional Breweries (Business)  
**Markets:** United States and United Kingdom first; English-first  
**Positioning:** Technical, evidence-led brewing software  
**Commercial model:** Self-service first; no demo-led funnel assumption  
**Source of truth:** `plan/demleme-plani.md`, `web-stratejisi/12-site-revizyonu-ve-uygulama-plani.md`, `web-stratejisi/13-mevcut-site-ve-foambyte-denetimi.md`, `web-stratejisi/14-feature-claim-denetimi.md`.

---

## 1. Decisions and design principles

1. The two audiences must have distinct entry pages, terminology, examples, navigation cues and conversion paths.
2. The homepage is a routing and positioning page, not a diluted combination of the two audience pages.
3. Every feature claim inherits its approved roadmap state. Planned, in-development, under-consideration and available are not interchangeable.
4. Until the product has a verified public onboarding path, primary CTAs are educational/exploratory rather than fake signup, trial or purchase actions.
5. Calculators and Learn content are acquisition paths in their own right; each must connect naturally to the relevant audience and product area.
6. Business messaging must state that Demleme is not a general ERP: sales, orders, distribution/shipping, keg tracking, CRM and accounting are out of scope.
7. Do not invent customers, testimonials, performance metrics, product screens, integrations, pricing, launch dates or regulatory assurances.
8. Keep the main navigation short. Audience pages are first-class; deep topic pages belong in contextual navigation and footer groups.
9. English is the primary publishing language. US/UK spelling and brewing-unit differences must be handled deliberately in technical content.
10. WP-04 defines information architecture and journeys only; it does not authorize changes to the live marketing site or product application.

## 2. Proposed global sitemap

| Level | Route | Page / content | Primary audience | Priority | State / dependency |
|---|---|---|---|---|---|
| 0 | `/` | Home: Business-first product positioning and secondary Home route | Business primary / Home secondary | P0 | Existing; revise to commercial strategy |
| 1 | `/homebrewers/` | Homebrewers overview | Home | P1 — secondary | Existing; retain separate journey |
| 1 | `/breweries/` | Professional Breweries overview | Business | P0 — primary | Primary commercial landing page; deepen operations and scope |
| 1 | `/features/` | Feature directory with audience filters | Both | P0 | Existing; claim audit required |
| 2 | `/features/[slug]/` | Individual capability pages | Both / Business | P0 | Existing; preserve roadmap status |
| 1 | `/how-it-works/` | Two distinct end-to-end workflow overviews | Both | P1 | New |
| 2 | `/how-it-works/homebrewing/` | Home workflow | Home | P1 | New |
| 2 | `/how-it-works/brewery-production/` | Business workflow | Business | P1 | New |
| 1 | `/calculators/` | Verified calculator directory | Both | P1 | New; only publish validated tools |
| 2 | `/calculators/[slug]/` | One task per calculator, formula and limits | Depends on tool | P1 | New; formula QA gate |
| 1 | `/learn/` | Knowledge Center landing and topic index | Both | P1 | New |
| 2 | `/learn/[topic]/` | Topic hubs / pillar guides | Both | P1 | New |
| 3 | `/learn/[topic]/[article]/` | Supporting technical articles | Depends on topic | P1 | New |
| 1 | `/use-cases/` | Scenario directory | Both | P1 | New |
| 2 | `/use-cases/[slug]/` | Problem → process → records → limits | Home / Business | P1 | New |
| 1 | `/faq/` | Searchable/grouped FAQ | Both | P1 | New |
| 1 | `/technical-principles/` | Calculations, units, sources, validation and data principles | Both | P1 | New |
| 1 | `/about/` | Product philosophy, scope and development principles | Both | P2 | New / content approval |
| 1 | `/roadmap/` | Public roadmap only if statuses and disclosure are approved | Both | P2 | Optional; product-owner approval |
| 1 | `/pricing/` | Pricing and plan comparison | Both | Hold | Do not publish until pricing decision |
| 1 | `/sign-in/`, `/get-started/` | Product entry points | Both | Hold | Only when real authenticated product flow exists |
| Utility | `/privacy/`, `/terms/`, `/contact/` | Legal and contact | Both | P0 before relevant data capture | Legal/operational approval |
| Utility | `/sitemap.xml`, `/robots.txt`, `/404` | Technical SEO and recovery | All | P0 | Existing; live HTTP QA open |

### 2.1 Main navigation proposal

| Navigation item | Destination | Menu treatment | Rationale |
|---|---|---|---|
| Product | `/features/` | Compact dropdown: Features, How it works | Understand the product without a huge mega-menu |
| Homebrewers | `/homebrewers/` | Direct link | First-class audience path |
| Breweries | `/breweries/` | Direct link | First-class audience path |
| Calculators | `/calculators/` | Direct link | High-intent utility and SEO entry |
| Learn | `/learn/` | Compact dropdown: Learn, Use cases, FAQ, Technical principles | Trust and education |
| Primary header CTA | Context-aware, not a fake signup | Label changes only when a real action exists | Avoid dead ends |

Footer groups:
- Product: Features, How it works, Roadmap (only if approved).
- For brewers: Homebrewers, Breweries, Use cases.
- Learn: Calculators, Knowledge Center, FAQ, Technical principles.
- Company/legal: About, Contact, Privacy, Terms.

Mobile navigation should preserve the same hierarchy, expose both audience paths without a second interaction, and use accessible focus/expanded states.

## 3. Audience journey — Homebrewers

### 3.1 User intent and questions

A homebrewer usually arrives with a specific practical question or a desire to improve consistency. They need to understand whether the product helps with recipe design, brewing calculations, water/mash decisions, brew-day execution, fermentation records and learning from prior batches.

The site should answer in this order:
1. Is this built for my kind of brewing?
2. Which part of my brewing process does it help organize or explain?
3. Are the calculations transparent and trustworthy?
4. What is planned versus usable today?
5. What can I do next without being pushed into a nonexistent signup?

### 3.2 Journey map

| Step | Visitor question / intent | Best entry page | Content needed | Next internal link | CTA / action |
|---:|---|---|---|---|---|
| 1 | “I need brewing software / recipe help” | Home, Homebrewers, Learn article or calculator | Clear homebrewer proposition and scope | Homebrewers overview | Explore Homebrewing |
| 2 | “Can it help me formulate a recipe?” | Homebrewers | Recipe workflow, versioning, style context and roadmap state | Recipe design feature; recipe use case | Explore recipe workflow |
| 3 | “Can I trust the numbers?” | Calculator or Technical Principles | Formula, inputs, units, assumptions, reference example and limits | Relevant calculator explainer / Learn guide | Use calculator (only if verified) |
| 4 | “How does this fit brew day?” | Home How It Works | Plan vs actual, equipment and measurement flow | Brew-day feature; batch-record use case | See the brew-day workflow |
| 5 | “What happens during fermentation?” | Fermentation feature / Learn | Manual measurements, device scope, current roadmap status; no unapproved prediction promises | Fermentation guide; device integration scope | Understand tracking approach |
| 6 | “Is it ready and what is available?” | FAQ / Features | Honest availability and platform statements | Relevant feature pages | Follow real product update channel only if configured |
| 7 | “I want to start using it” | Product entry (future) | Real onboarding, account and privacy explanation | Get started | Get started only after verified launch |

### 3.3 Homebrewer content sequence

Recommended page narrative:
1. Outcome-led headline: plan, measure and learn from each batch (avoid claiming unavailable functions as live).
2. “Built for homebrewers” — who it serves and what is intentionally not required.
3. Brewing journey strip: recipe → calculations → brew day → fermentation → batch review.
4. Deep links to recipe, calculations, water/mash/yeast, equipment, brew-day and fermentation topics.
5. Technical trust module: formula transparency, units, assumptions and validation principles.
6. Two or three carefully selected use cases.
7. Platform and device scope with exact planned/available labels.
8. FAQ and contextual CTA.

Home-specific examples:
- Scale a 20 L recipe to a different batch size.
- Compare target and measured OG/FG.
- Review water-profile and mineral-addition assumptions.
- Keep a clear record of brew-day target versus actual values.
- Compare completed batch notes without claiming automated predictions.

## 4. Audience journey — Professional Breweries

### 4.1 User intent and questions

A professional visitor is evaluating operational fit, not looking for a hobby recipe calculator. Likely roles include owner/manager, head brewer, production lead, operator and quality/lab staff.

The site should answer:
1. Does this fit a small professional brewing operation?
2. Which production workflows are in scope?
3. How do recipes, batches, tanks, ingredients, quality records and roles relate?
4. What is explicitly not included (ERP exclusions)?
5. What is the maturity and deployment status of each workflow?
6. Can the business assess the fit without being forced into a sales demo?

### 4.2 Journey map

| Step | Visitor question / intent | Best entry page | Content needed | Next internal link | CTA / action |
|---:|---|---|---|---|---|
| 1 | “I need brewery production software” | Home, Breweries, relevant Learn article | Clear professional-brewery positioning and business scope | Breweries overview | Explore brewery workflows |
| 2 | “Can we standardize recipes and approvals?” | Breweries / Recipe feature | Recipe versioning, review and approval, role/stage limits | Recipe management; approval use case | Review recipe workflow |
| 3 | “Can we plan production and tanks?” | Production planning feature / How It Works | Production board, tank allocation and schedule-stage detail | Tank scheduling use case | Explore production planning |
| 4 | “Can operators record actual work?” | Brew-day / Production records | Plan vs actual, operator records, shifts and audit-history status | Production records use case | See production record flow |
| 5 | “Can we manage quality and traceability?” | Quality / Inventory / Feature pages | QC records, deviations, lot-to-batch linkage and release stages | Traceability and QC use cases | Explore quality and traceability |
| 6 | “Is this an ERP?” | Breweries / FAQ | Explicit exclusions: sales, orders, distribution/shipping, keg tracking, CRM, accounting | Scope FAQ | Check product fit |
| 7 | “Can we adopt it?” | Product entry (future) | Verified business onboarding, roles, data and deployment details | Get started | Start only when real product flow exists |

### 4.3 Business content sequence

Recommended page narrative:
1. Operational headline focused on recipe-to-production discipline, not generic “all-in-one brewery management”.
2. Define the intended customer: professional/small brewery and production team; do not invent a capacity threshold.
3. Production lifecycle: recipe review → planning/tank allocation → production records → fermentation/tank workflow → quality/traceability → packaging/compliance records (only within roadmap stage).
4. Role-oriented entry points: owner/manager, head brewer, operator, quality/lab.
5. Operational records and traceability explanation.
6. Clear “What Demleme is not” section with ERP exclusions.
7. Product stage and platform statements.
8. Business FAQs and contextual CTA.

Business-specific examples:
- Turn an approved recipe into a production batch.
- Review tank schedule and capacity conflicts (only at the relevant planned stage).
- Compare planned and actual production measurements.
- Trace an ingredient lot to a production batch.
- Record quality checks, deviations and release decisions (only as roadmap permits).

## 5. Side-by-side distinction — required on the site

| Dimension | Homebrewers | Professional Breweries |
|---|---|---|
| Primary job | Make better, repeatable batches and understand brewing decisions | Coordinate repeatable production and maintain reliable operational records |
| Unit of work | Recipe and individual batch | Approved recipe, production batch, tank, lot and team record |
| Core narrative | Learn → formulate → calculate → brew → measure → review | Standardize → approve → schedule → produce → inspect → trace |
| Main value language | Clarity, learning, consistency, control of variables | Standardization, coordination, accountability, traceability |
| Data examples | Recipe parameters, water, OG/FG, mash, fermentation notes | Recipe versions, tank allocation, ingredient lots, operator logs, QC |
| Typical user | Individual brewer | Owner, head brewer, operator, quality staff |
| Technical trust | Explain formulas, units and measurement assumptions | Explain data relationships, permissions, record history and process controls |
| CTA style | Try a verified utility / learn a method / future self-service start | Assess workflow fit / inspect scope / future business onboarding |
| Avoid | Industrial operations language as the default | Hobbyist tone and broad ERP promises |

The site must not simply swap “homebrewer” for “brewery” while keeping identical sections and examples. Shared product capabilities may be linked, but their value and workflow context must change.

## 6. CTA architecture and lifecycle gates

### 6.1 CTA states

| Product / service state | Primary CTA | Secondary CTA | Forbidden CTA / implication |
|---|---|---|---|
| No public product access | Explore workflows; Read the guide; See how it is calculated | Explore features; View FAQ | Start free, Sign up, Start trial, Buy now, Book demo |
| Verified public calculator | Calculate now; Open calculator | How this is calculated; Read method | Claim that calculator is the product app or account-connected |
| Approved waitlist/update channel exists | Get product updates | Explore roadmap | “Join waitlist” unless a real, privacy-reviewed endpoint exists |
| Verified self-service product launch | Get started / Create account | Explore features / Pricing | Any action not connected to a working route |
| Pricing approved and purchasable | Choose a plan / Start plan | Compare plans / FAQ | Unapproved prices, discounts, trial periods or plan limits |
| Business workflow genuinely available | Explore Business / Get started | Review security and scope | Book a demo as default if self-service remains the strategy |

### 6.2 CTA placement rules

- One dominant CTA per page section; avoid competing button clusters.
- The homepage's two audience cards are routing links, not conversion buttons.
- Feature pages link to the audience workflow and a relevant educational resource.
- Calculator pages link to the method and one relevant feature page.
- Learn articles link to a relevant calculator only when it exists and is validated; otherwise link to the method/feature page.
- FAQ answers link to the exact supporting page rather than a generic homepage.
- No sticky CTA that obscures mobile content or implies immediate product availability.
- CTA labels must describe the destination/action precisely.

## 7. Internal-linking and SEO intent map

| Page group | Primary intent | Link in | Link out |
|---|---|---|---|
| Home | Brand / category discovery | External brand searches, all main nav | Homebrewers, Breweries, Features, Calculators, Learn |
| Homebrewers | Homebrewing software evaluation | Home, Home Learn and calculator pages | Recipe, calculations, water/mash, brew-day, fermentation, use cases |
| Breweries | Brewery production software evaluation | Home, Business Learn and feature pages | Recipe approval, production planning, records, inventory, QC, traceability |
| Feature pages | Capability-specific evaluation | Audience pages, directory, use cases | Audience page, workflow, FAQ, technical article |
| Calculators | Task completion / formula search | Nav, Learn articles, audience pages | Method, Technical Principles, related feature and audience |
| Learn | Informational / technical research | Search, nav, calculator pages | Related guide, calculator, feature, use case |
| Use cases | Problem-solution evaluation | Audience pages, feature pages | Relevant features, workflow, FAQ |
| FAQ | Objection resolution | All pages and footer | Exact feature, platform, privacy, scope and technical pages |
| Technical Principles | Trust / validation | Calculator, feature, FAQ | Formula articles, calculators, feature pages |

SEO guardrails:
- One primary search intent per URL; do not create near-duplicate pages for minor keyword variants.
- Use audience-specific language and examples, not doorway pages.
- Do not publish thin calculator stubs or placeholder Learn articles.
- Keep US/UK spelling variants natural; handle units explicitly (US customary/Imperial and metric where relevant).
- Internal links should be descriptive and contextual; avoid repeated exact-match anchors.
- Each page needs a unique title, H1, description, canonical and appropriate structured data.
- Use Article/HowTo/FAQ structured data only when page content and current search-engine eligibility genuinely qualify; do not add schema solely for rich-result promises.
- No fabricated review, product, organization, software-application or rating schema claims.

## 8. Content templates and acceptance criteria

### 8.1 Audience landing page template

1. Audience-specific hero: problem, audience, outcome; accurate stage wording.
2. Who it is for / not for.
3. Workflow overview.
4. Priority problems and use cases.
5. Relevant feature groups with stage labels.
6. Technical trust / method / records.
7. Platform and integration boundaries.
8. FAQ.
9. One primary and one secondary CTA.

Acceptance:
- Home and Business pages have materially different workflows and examples.
- Each claim maps to product plan and WP-03 status.
- No unsupported availability, accuracy, customer or business-result claims.
- All links have meaningful destinations.

### 8.2 Use-case template

1. User and context.
2. Problem / trigger.
3. Inputs and assumptions.
4. Steps and decisions.
5. Expected records/output (not unproven outcomes).
6. Product feature mapping and status.
7. Limitations / out-of-scope work.
8. Related guide, calculator and FAQ.

### 8.3 Feature template

1. Problem and audience.
2. What the capability is intended to do.
3. Workflow and inputs.
4. Output / record / decision.
5. Home vs Business differences.
6. Current roadmap status.
7. Dependencies and limitations.
8. Related how-it-works, use case, Learn and FAQ links.

## 9. Open decisions and dependencies

| Decision | Why it matters | Safe default until decided |
|---|---|---|
| Public roadmap visibility | May expose internal sequencing or imply commitments | Keep stage labels on feature pages; no dated public roadmap |
| Product update / waitlist mechanism | Determines whether an update CTA can collect data | No waitlist CTA or email capture until endpoint, consent and privacy are approved |
| First validated web calculators | Determines which SEO landing pages can be published | Publish no calculator page as functional until WP-09/10 validation |
| Public Business onboarding | Determines whether Business can have self-service CTA | Educational workflow-fit CTA only |
| Public product launch state | Controls sign-up, trial, pricing and purchase CTA | No account or pricing CTA until a real flow is verified |
| Exact device integrations | Impacts technical promise and user trust | Name only confirmed models, protocols and data direction |
| Regulatory jurisdictions | Impacts Business claims | No compliance guarantee; educational explanation only |
| Public security/deployment facts | Impacts enterprise evaluation | Describe product principles, not unverified controls/certifications |

## 10. WP-04 completion and next work package

WP-04 defines:
- Global sitemap and proposed navigation.
- Separate Homebrewers and Professional Breweries journeys.
- A clear comparison of the two profiles.
- CTA labels and product-lifecycle gates.
- Internal-linking and SEO intent map.
- Reusable page templates and acceptance checks.
- Explicit unresolved decisions and conservative defaults.

**Next package: WP-05 — content and messaging architecture.** Build page-by-page messaging matrices, headline/value proposition options, content briefs, proof requirements, terminology and SEO intent without drafting final production copy or touching the live site. WP-05 must preserve WP-03 claim statuses and WP-04 CTA gates.
