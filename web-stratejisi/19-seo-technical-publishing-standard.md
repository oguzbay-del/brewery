# Demleme — SEO, Technical Publishing & Content Quality Standard

**Work package:** WP-08  
**Status:** Publishing specification; no SEO code or live content changes.  
**Date:** 14 November 2026  
**Depends on:** WP-01/02 audit, WP-03 claim audit, WP-04 sitemap, WP-05 messaging, WP-06 design system, WP-07 page wireframes.  
**Markets:** United States and United Kingdom first.  
**Language:** English-first; US editorial default with UK terminology where appropriate.

---

## 1. SEO objective and operating principles

Build durable search visibility by answering real brewing questions with technically reliable, useful pages. Search optimization must follow user intent and evidence quality; it must not create thin pages, duplicate audience routes or inflate product capability.

**Rules**
- One primary search intent per indexable URL.
- A page must add original utility, technical explanation or product-specific scope—not merely repeat keywords.
- No keyword stuffing, doorway pages, fabricated expert authorship, fake reviews, or unsupported comparative claims.
- Do not index empty category pages, placeholder pages, thin feature pages or unvalidated calculators.
- Search snippets must reflect the actual page and verified product maturity.
- SEO metadata is not a substitute for useful content or technical proof.

## 2. Search intent and URL ownership

| URL group | Primary intent | Audience | Indexing rule |
|---|---|---|---|
| `/` | Brand and category discovery | Both | Index when positioning and routes are complete |
| `/homebrewers/` | Homebrewing software evaluation | Home | Index; distinct from Business |
| `/breweries/` | Brewery production workflow evaluation | Business | Index; explicit ERP boundary |
| `/features/` | Capability discovery | Both | Index only with substantive grouped content |
| `/features/{slug}/` | One capability / problem | Home, Business or Both | Index only when unique, verified and useful |
| `/how-it-works/` | Workflow understanding | Both | Index when both paths are explained accurately |
| `/calculators/` | Brewing calculation utility discovery | Both | Index only when real validated tools exist |
| `/calculators/{slug}/` | Specific calculation | Both | Index only after WP-09/10 validation |
| `/learn/{slug}/` | Informational technical question | Both | Index after expert review, sources and unique value |
| `/use-cases/{slug}/` | Scenario evaluation | Audience-specific | Index only if grounded in approved product scope |
| `/faq/` | Product and scope questions | Both | Index when answers are complete and accurate |
| `/technical-principles/` | Methods, units and technical trust | Both | Index when evidence and limitations are documented |
| `/pricing/`, `/roadmap/` | Commercial / roadmap evaluation | Both | Hold until public decisions and content are approved |

### URL conventions
- Lowercase, stable, descriptive slugs.
- Hyphen-separated words; avoid dates unless content is inherently time-bound.
- One canonical URL per content item.
- Avoid changing established URLs without a redirect plan.
- Do not create parallel US/UK URL trees until there is enough genuinely localized content to justify them.

## 3. Keyword and topic architecture

Keyword research is a hypothesis generator, not a reason to publish a page. Assign each page one primary topic and a small set of semantically related questions.

| Cluster | Primary page | Supporting topics | Audience |
|---|---|---|---|
| Homebrewing software | `/homebrewers/` | Recipe planning, batch records, brew-day workflow | Home |
| Brewery production software | `/breweries/` | Recipe versioning, production planning, batch records | Business |
| Brewing calculations | `/calculators/` | Gravity, ABV, IBU, colour, scaling | Both; only validated tools |
| Recipe formulation | Learn / feature page | Ingredients, style targets, scaling, versioning | Both |
| Water and mash | Learn | Water profiles, mineral additions, mash pH caveats | Primarily Home; relevant Business |
| Fermentation | Learn / feature | Gravity readings, temperature, observations | Both |
| Brewery traceability | Business use case / feature | Ingredient lot, batch genealogy, QC records | Business |
| Brewing technical methods | Technical Principles | Units, precision, calibration, measurement | Both |

Avoid cannibalization:
- Homebrewers owns homebrew workflow intent; Breweries owns professional production intent.
- A feature page explains a capability; a Learn article answers a technical question.
- A calculator page owns the interactive task; its guide explains method and assumptions.
- Do not publish multiple near-identical articles targeting the same query.

## 4. On-page content requirements

Every indexable page must have:
1. Unique, descriptive title tag (normally 50–60 characters as a drafting guide, not a hard limit).
2. One clear H1 aligned to the page's search intent.
3. A concise opening answer/value statement.
4. Logical H2/H3 structure that reflects real questions.
5. Original explanatory content, examples or useful navigation.
6. Internal links to the next relevant page in the user journey.
7. Evidence and limitations for technical/product claims.
8. Descriptive image alt text; decorative imagery uses empty alt.
9. Accurate canonical and share metadata.
10. Last-reviewed date for technical content where maintenance matters.

Meta descriptions should summarize the actual page in natural language (often around 140–160 characters as a drafting aid). Do not force a keyword or promise a capability absent from the page.

### Content depth standard
No fixed word count guarantees quality. A page is ready when it answers its target question completely, explains important assumptions and limitations, cites authoritative sources where appropriate, and provides a useful next step. Short pages are acceptable when the question is narrow and fully answered.

## 5. Technical SEO requirements

### Crawl and indexation
- Maintain a valid XML sitemap containing canonical, indexable URLs only.
- Sitemap `lastmod` must represent meaningful content updates, not build time.
- `robots.txt` must reference the canonical sitemap URL.
- Do not block CSS, JS or assets required to render meaningful content.
- Use `noindex` for staging, preview, search/filter states and pages intentionally withheld from search.
- Do not rely on robots.txt to remove an already indexed URL.
- Avoid accidental noindex on production routes and accidental indexing of previews.

### Canonicals and redirects
- Every indexable page has a self-referencing absolute canonical.
- Redirect alternate host/protocol/trailing-slash variants consistently.
- Use 301 redirects for permanent URL changes; update internal links and sitemap.
- Avoid redirect chains and loops.
- Do not canonicalize distinct Home and Business pages to one another.

### Metadata and social sharing
- Unique title and description per route.
- Open Graph and Twitter metadata use the correct page title, description and approved image.
- Do not invent social preview imagery; retain a valid brand fallback if available.
- Favicon and manifest references must resolve.

### Structured data
Use JSON-LD only when the page visibly supports the data and it is accurate.
- Organization / WebSite: only verified organization and official URL details.
- SoftwareApplication: only if public product facts (name, category, platform and availability) can be stated accurately; do not invent ratings, offers or pricing.
- Article: real author/editor information, dates and article content.
- BreadcrumbList: visible breadcrumb hierarchy.
- FAQPage: do not add solely to chase rich results; markup must match visible Q&A and current search-engine eligibility.
- HowTo: only for genuine step-by-step instructions that meet current search engine policies.
- No fabricated AggregateRating, Review, Product offer, price or customer evidence.

## 6. International and technical localization

- English-first with US spelling as default.
- Use UK spelling naturally on UK-specific pages, but do not create superficial duplicate pages.
- Distinguish US gallon from Imperial gallon.
- Every numerical input/output shows a unit.
- Use `hreflang` only when equivalent, genuinely localized pages exist for each locale; do not add speculative tags.
- If separate locale URLs are introduced later, use reciprocal hreflang annotations and a valid x-default only when appropriate.
- Calculator number formatting must be locale-aware and must never silently change the physical unit.

## 7. Internal linking model

Each page should link to:
- Its parent category or audience page.
- One or more directly related feature pages.
- The relevant technical guide or method explanation.
- The next useful action in the user's journey.

Rules:
- Use descriptive anchor text, not repeated “click here”.
- Do not link to unpublished or placeholder routes.
- Related links must be genuinely related; avoid sitewide keyword-heavy footer links.
- Breadcrumbs reflect actual hierarchy.
- Home and Business paths remain distinct; cross-link only when the user has a clear reason.

## 8. Technical performance and accessibility

SEO quality depends on a fast, usable and accessible page.

- Prefer static generation for public marketing content.
- Optimize image dimensions, format and loading priority; do not lazy-load the actual LCP hero image.
- Reserve image dimensions to prevent layout shift.
- Avoid unnecessary client-side JavaScript and third-party scripts.
- Track Core Web Vitals with real-user data when a privacy-approved analytics solution exists.
- Use Lighthouse as a diagnostic, not a substitute for real-user data.
- Meet WP-06 accessibility requirements: semantic HTML, keyboard access, contrast, labels, alt text, reduced motion and mobile usability.
- Keep sitemap, canonical, metadata and robots tests in CI where feasible.

## 9. Measurement and privacy

Do not add analytics, pixels, session replay or marketing cookies until provider, purpose, retention, consent and privacy requirements are approved.

When measurement is approved, start with:
- Search Console: indexed pages, queries, impressions, clicks and CTR.
- Privacy-respecting site analytics: landing pages, audience route selection and outbound product CTA clicks.
- Conversion events only for real, functioning actions.
- Segment reporting by Home and Business landing paths without collecting unnecessary personal data.

Do not define success by raw traffic alone. Evaluate qualified search visits, useful resource engagement, audience-path progression and verified product-entry actions.

## 10. Content review and publication gates

| Gate | Owner / review | Pass condition |
|---|---|---|
| Brief | Content owner | Audience, intent, URL and user question defined |
| Product claim | Product owner | Each feature statement matches current product plan/stage |
| Technical | Subject reviewer | Formula, units, sources, assumptions and limitations checked |
| Editorial | Editor | Clear, original, audience-specific language |
| SEO | Web owner | Unique intent, metadata, links, canonical and indexation checked |
| Accessibility | Web owner | Semantic structure, keyboard, contrast, alt and responsive checks |
| Legal/privacy | Relevant owner | Required notices, consent and claims reviewed |
| Release | Product/site owner | Destination works; content is actually available |

### Technical article review record
Each technical article stores:
- Content ID and canonical route.
- Audience and search intent.
- Author and technical reviewer (real identities/credentials only).
- Primary references and source dates.
- Formula/version or product-plan references, if relevant.
- Known limitations and jurisdiction notes.
- First-published and last-reviewed dates.
- Next review date or review trigger.

## 11. FoamByte benchmark and Demleme response

| Benchmark lesson | Demleme response |
|---|---|
| Broad feature and calculator discovery can attract task-oriented visitors | Publish a clear feature taxonomy and useful calculator directory, but only after validation |
| FAQ and pricing can reduce commercial uncertainty | Answer real questions now; publish pricing only after approval |
| Many pages can create topical breadth | Prefer fewer complete pages over thin programmatic SEO |
| Product pages should connect capability to a user problem | Use problem → method → example → limitation → product scope |
| Search content can feed product discovery | Keep contextual internal links, not aggressive generic sales CTAs |

Do not copy FoamByte's text, page design, keyword targeting, imagery or claims.

## 12. Pre-publication SEO checklist

- [ ] Unique search intent and canonical URL.
- [ ] Page contains useful, complete and original information.
- [ ] H1/title/description accurately describe the page.
- [ ] Every product claim is verified against WP-03 and the current plan.
- [ ] Technical claims include sources, units, assumptions and limitations.
- [ ] Internal links resolve and follow the Home/Business journey.
- [ ] Canonical, robots, sitemap and social metadata are correct.
- [ ] Structured data matches visible content and verified facts.
- [ ] No fake pricing, reviews, metrics, authorship or availability.
- [ ] Mobile, accessibility and performance checks are recorded.
- [ ] Privacy and consent requirements are met before analytics/scripts.
- [ ] Page is genuinely ready for indexing; otherwise keep it out of sitemap and noindex.

## 13. Next work package

**WP-09 — Technical content and calculator validation backlog.** Build a prioritized evidence-backed content queue and a formula validation matrix before any calculator is implemented or published. No production code or live content changes are authorized by this document.
