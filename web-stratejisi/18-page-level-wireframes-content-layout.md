# Demleme — Page-Level Wireframes & Content-to-Layout Mapping

**Work package:** WP-07  
**Status:** Low-fidelity structural specification; not a visual mockup or implementation.  
**Date:** 14 November 2026  
**Depends on:** WP-04 sitemap/journeys, WP-05 messaging, WP-06 design system.  
**Scope:** Marketing site only. All product-stage labels remain subject to WP-03 and current product verification.

---

## 1. Page system and design rules

This package converts the approved information architecture and message briefs into repeatable page structures. It is intentionally content-first: every section has a user question, message job, evidence requirement and next step.

**Global rules**
- One H1 per page; one primary narrative per section.
- Lead with the visitor's task, not an exhaustive feature list.
- Home and Business share the design system, not identical page layouts.
- Use no fake product screenshots, customer logos, metrics, testimonials, pricing or unapproved integrations.
- Mark every capability with its verified stage; do not convert roadmap intent into availability.
- Preserve existing five `data-image-slot` identifiers. No new imagery is sourced or generated.
- Every CTA must resolve to a real page or verified product destination. If no valid destination exists, use an informational link or omit the CTA.
- Technical content follows: problem → context → method → example → limitation → related product area.

## 2. Global navigation and page shell

### Desktop header

```text
[Demleme]   Homebrewers   Breweries   Platform   Resources          [Sign in] [Primary CTA]
```

- Homebrewers and Breweries are first-level links.
- Platform groups Features / How it works / Technical principles only if those routes exist.
- Resources groups Learn / Calculators / FAQ only when each destination is published.
- Sign in and primary product CTA remain hidden or inactive until approved URLs and real product access flow are verified.
- No Pricing link until commercial model is approved.

### Mobile header

```text
[Demleme]                                                   [Menu]
---------------------------------------------------------------
Menu open:
Homebrewers
Professional Breweries
Platform
Resources
[verified account action, if available]
```

Menu must expose current-page state, support keyboard/touch, close on Escape, return focus to trigger and avoid body scroll trapping.

### Footer

Four compact groups: Product, For brewers, Resources, Company. Only published routes and real legal/social destinations appear. Do not render empty categories or placeholder social links.

## 3. Homepage — route `/`

**Primary job:** Explain Demleme and route two distinct audiences.  
**Search intent:** Brand/category discovery.  
**Primary CTA:** Explore Homebrewers / Explore Breweries.

| Order | Section | Layout | Content and evidence | Mobile behavior |
|---:|---|---|---|---|
| 1 | Header | Full-width restrained nav | Brand, audience routes, real account action only | Compact menu |
| 2 | Hero | 7/5 split; text + approved visual slot or typographic diagram | H1 positioning, short subhead, two audience links; no fake product UI | Stack text then visual |
| 3 | Audience choice | Two equal editorial panels | Different Home / Business problem statements, examples and destinations | Stack panels |
| 4 | Brewing lifecycle | Horizontal sequence or 3 editorial columns | Recipe → brew → record/review; stage labels where relevant | Vertical numbered sequence |
| 5 | Technical clarity | Text + method/evidence panel | Formula transparency, units, assumptions and limitations; link to principles | Single column |
| 6 | Feature overview | Grouped rows by lifecycle, not card wall | Only verified feature names and stage; audience tags | Accordion only if content length requires |
| 7 | Learning utility | Editorial resource list | Published guides and validated calculators only | Stacked list |
| 8 | Product maturity | Plain text status note | Current product state, no launch promise | Same |
| 9 | Footer | Four groups | Real destinations only | Stacked groups |

**Wireframe**
```text
┌────────────────────────────────────────────────────────────────┐
│ DEMLEME   Homebrewers  Breweries  Platform  Resources  Account │
├────────────────────────────────────────────────────────────────┤
│ [H1: brewing-specific positioning]       [approved visual slot]│
│ Short explanation of who it serves and why                    │
│ [Explore Homebrewers] [Explore Breweries]                      │
├────────────────────────────────────────────────────────────────┤
│ TWO DISTINCT WAYS TO BREW                                      │
│ [HOME: recipe → brew → learn] [BUSINESS: recipe → production]  │
├────────────────────────────────────────────────────────────────┤
│ THE BREWING LIFECYCLE                                           │
│ Plan ───────── Brew ───────── Record ───────── Review           │
├────────────────────────────────────────────────────────────────┤
│ TECHNICAL CLARITY              [method / units / assumptions]   │
├────────────────────────────────────────────────────────────────┤
│ EXPLORE THE PLATFORM                                            │
│ Feature group rows with audience and verified status           │
├────────────────────────────────────────────────────────────────┤
│ LEARN / VALIDATED CALCULATORS                                   │
├────────────────────────────────────────────────────────────────┤
│ Product maturity note                                           │
├────────────────────────────────────────────────────────────────┤
│ Footer                                                          │
└────────────────────────────────────────────────────────────────┘
```

## 4. Homebrewers — route `/homebrewers/`

**Primary job:** Show how Demleme's intended Home workflow maps to a homebrewer's real process.  
**Search intent:** Homebrewing software / recipe and batch workflow evaluation.  
**Primary next step:** Explore a verified feature, technical guide or validated calculator.

| Order | Section | Layout | Content and evidence | Mobile behavior |
|---:|---|---|---|---|
| 1 | Hero | Text-led with Home-specific slot | Homebrewing value proposition; no professional tank image | Text then visual |
| 2 | Who it is for | Short editorial copy | Curious, detail-oriented homebrewers; no invented persona stats | Natural flow |
| 3 | Recipe to batch journey | Vertical/horizontal 4–5 step sequence | Recipe, ingredients/water, preparation, brew day, fermentation/records; status mapped | Vertical sequence |
| 4 | Technical calculations | Two-column method explanation | Explain validated formula topics, units, assumptions and boundaries | Single column |
| 5 | Feature groups | Grouped editorial rows | Recipe formulation, brew-day context, measurement and records only at actual stage | Stacked rows |
| 6 | Learn resources | Guide list | Relevant substantive guides | Stacked |
| 7 | Scope / maturity | Plain text | Device, prediction and automation boundaries; no unapproved claims | Same |
| 8 | Related FAQ | Disclosure list | Direct Home questions | Native disclosure |
| 9 | Closing route | Text link(s) | Return to guides / feature scope | Same |

**Home page diagram**
```text
Recipe intent
    ↓
Ingredients, water and calculations
    ↓
Brew-day preparation and execution
    ↓
Fermentation observations and measurements
    ↓
Batch records and learning
```

Do not imply that fermentation predictions, automatic device control, calibration or every listed record function is currently available. Product stage is attached to each item.

## 5. Professional Breweries — route `/breweries/`

**Primary job:** Explain the professional brewing workflow and explicitly define the product boundary.  
**Search intent:** Brewery production software / recipe management / batch records.  
**Primary next step:** Review the Business workflow and product scope.

| Order | Section | Layout | Content and evidence | Mobile behavior |
|---:|---|---|---|---|
| 1 | Hero | Text-led with separate brewery slot | Professional value proposition: recipe-to-production continuity | Text then visual |
| 2 | Operational context | Editorial lead + concise process diagram | Team roles and operational coordination without invented customer story | Stack |
| 3 | Recipe to production journey | Process map | Recipe review → production planning → brew/production records → fermentation/tank → QC/traceability (stage-specific) | Vertical sequence |
| 4 | Records and accountability | Table or relationship diagram | Planned vs actual, batch and lot relationships; only approved scope | Simplified labelled diagram |
| 5 | Feature groups | Process-aligned rows | Recipe, production, batch, quality and traceability with stage labels | Stacked rows |
| 6 | Explicit product boundary | Plain, visible scope section | Not a general ERP: no sales, orders, distribution/shipping, keg tracking, CRM or accounting | Same; never hide in footer |
| 7 | Platform / security status | Evidence-led text | State only implemented and reviewed controls; distinguish design intent | Same |
| 8 | FAQ | Native disclosures | Roles, records, data, scope, product maturity | Same |
| 9 | Closing route | Relevant informational CTA | Explore workflow / review product scope | Same |

**Business page diagram**
```text
Recipe formulation and review
          ↓
Production planning and allocation
          ↓
Brew / production records
          ↓
Fermentation and vessel workflow
          ↓
Quality checks and batch / lot records
```

This is a conceptual workflow map, not a claim that all stages are implemented. Packaging, regulatory filing, commercial operations and distribution are excluded unless product scope is explicitly approved later.

## 6. Features index and detail

### Index

- Intro explains how capability groups map to brewing work.
- Filters: audience (Home / Business / Both) and lifecycle group only if they are functional and accessible.
- Each row shows name, concise problem statement, audience and verified status.
- Avoid 16 equally weighted feature cards; group by lifecycle and highlight the most relevant paths.
- Empty or unverified feature groups are not published.

### Detail template

1. Breadcrumb and audience/status metadata.
2. H1 naming the user problem or capability.
3. One-paragraph outcome framed without guarantee.
4. Workflow: inputs → action → outputs/records.
5. Technical explanation / assumptions.
6. Example only if product plan and formula/workflow are validated.
7. Limitations and dependencies.
8. Product stage and last verification date.
9. Related audience page, guide and feature.

No detail page uses “Available” unless deployed and release-verified.

## 7. How It Works

Use two tabs or two clearly separated sections only if keyboard and deep-link behavior is sound. Otherwise use separate anchored Home and Business chapters.

Each journey step contains:
- User intent.
- Inputs / decisions.
- Resulting record or next step.
- Audience-specific terminology.
- Product stage.
- Related feature and technical guide.

Do not depict hypothetical screens as screenshots. Use accessible process diagrams with text equivalents.

## 8. Calculators and Learn

### Calculator directory

- Group by task (recipe, gravity/ABV, bitterness/colour, water/mash) only when each tool is validated.
- Each entry names the calculation and supported inputs/units.
- State whether the tool is an educational web utility or connected product feature; never imply account synchronization without proof.
- Do not publish functional calculators until formula validation and WP-09/10 release gates pass.

### Calculator detail

Task → inputs and units → method/formula → worked example → edge cases → limitations → references → related guide.

### Learn article

Direct answer → context → method → worked example → caveats → sources → related calculator/feature. Author credentials must be real and approved. Technical content requires review.

## 9. FAQ and Technical Principles

FAQ answers begin with the direct answer, then a short explanation and exact supporting link. Use categories only if there are enough questions to justify them.

Technical Principles should be structured as:
- Calculation methodology and sources.
- Unit systems and conversions.
- Rounding and precision.
- Validation and test approach.
- Data and record principles.
- Security/privacy statements supported by evidence.
- Known limitations and review date.

Do not publish a security badge or compliance claim from architecture intent alone.

## 10. Cross-page linking and CTA rules

| Source context | Preferred next link | Avoid |
|---|---|---|
| Homepage audience panel | Relevant audience page | Generic signup |
| Home recipe / brew-day section | Relevant feature or technical guide | Business production CTA |
| Business production section | Business feature / scope page | Homebrew beginner guide |
| Calculator result | Method explanation and related guide | “Save to account” unless implemented |
| Learn article | Relevant validated calculator or feature | Irrelevant product pitch |
| Feature detail | Audience workflow + technical evidence | Pricing/trial claim |
| FAQ | Exact page that substantiates answer | Unrelated generic CTA |

CTA vocabulary is gated by actual product flow. Until real account access and onboarding exist, use navigation and educational actions: “Explore”, “Read”, “See how it works”, “Review scope”. Do not use Start free, Start trial, Book demo, Buy now or Waitlist without the corresponding approved flow.

## 11. Content and design acceptance checklist

- [ ] Each page has one clear audience and search intent.
- [ ] H1 and first viewport explain page purpose.
- [ ] Home and Business use different examples, terminology and process structures.
- [ ] Every feature claim maps to WP-03 status and product plan.
- [ ] Business ERP exclusions are visible and explicit.
- [ ] No fake product UI, customer proof, pricing, integrations or metrics.
- [ ] Every CTA has a real destination.
- [ ] Technical examples include units, assumptions and limitations.
- [ ] Mobile layout preserves hierarchy and readable tables/diagrams.
- [ ] Keyboard, touch, focus and reduced-motion behavior meet WP-06.
- [ ] Existing image slots remain stable pending approved assets.

## 12. Next gate

WP-07 completes page-level structural wireframes and content-to-layout mapping. Next is a design review against the live marketing prototype at desktop and mobile sizes, then an explicit user approval gate for implementation. This document does not authorize production code, asset sourcing or live-site changes.
