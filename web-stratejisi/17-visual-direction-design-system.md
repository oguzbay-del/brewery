# Demleme — Visual Direction & Design System

**Work package:** WP-06  
**Status:** Design specification; not production implementation.  
**Date:** 14 November 2026  
**Depends on:** WP-01/02 site and FoamByte audit, WP-03 claim audit, WP-04 journeys/sitemap, WP-05 messaging architecture.  
**Markets:** United States and United Kingdom first.  
**Language:** English-first (US editorial default).  
**Scope:** Marketing website only. No product UI, app, API, server, or live-site changes.

---

## 1. Design objective

Create a technically credible brewing-software brand with the clarity and finish of a high-quality modern product website, while retaining Demleme's existing editorial, technical and human direction. The site must explain brewing work rather than resemble a generic AI/SaaS template.

**Design principles**
1. **Editorial clarity:** confident typography, meaningful whitespace, short sections and visible hierarchy.
2. **Technical evidence:** numbers, units, process diagrams and method explanations are legible and contextual.
3. **Human relevance:** Home and Business use distinct imagery, examples and page composition.
4. **Restraint:** no decorative effects without a communication purpose; no fake social proof or fabricated UI.
5. **Truthful maturity:** visual status labels always match the product-plan stage.
6. **Progressive disclosure:** summary first, technical depth and limitations close at hand.

The existing `tasarim-sistemi/tokens.json` remains the source of truth for shared brand tokens. This document specifies marketing-site roles and responsive behavior; it does not authorize changing the global product design system.

## 2. Brand and visual boundaries

| Keep | Avoid |
|---|---|
| Existing neutral background, text and amber accent tokens | Inventing a new palette or treating sample hex values as approved brand colors |
| System font stack and editorial hierarchy | Adding licensed fonts or font downloads without approval |
| Fine rules, whitespace and deliberate image crops | Card-everything layouts, heavy shadows and ornamental borders |
| Real brewing context and explanatory diagrams | Stock technology imagery, fake dashboards and invented product screens |
| Quiet, purposeful transitions | Neon, purple/blue gradients, glow, glassmorphism, blur or 3D decoration |
| Clear labels and accessible contrast | Tiny low-contrast labels, color-only status and all-caps paragraphs |

Amber is a restrained accent for links, selected states and the primary action. It must not become a full-page background or substitute for semantic status colors.

## 3. Design tokens and implementation mapping

Use semantic aliases that resolve to existing shared tokens. Do not hardcode colors inside components.

| Semantic role | Existing token mapping | Usage |
|---|---|---|
| Page background | `bg` | Main canvas |
| Primary surface | `card` | Small number of meaningful surfaces |
| Primary text | `label` | Headings and body |
| Secondary text | `label2` | Supporting explanation |
| Accent | `tint` | Links, focus accents, active navigation |
| Primary action fill | `tint-fill` | One dominant CTA per context |
| Divider | `sep` | Section and table separators |
| Status | Existing semantic status tokens, if defined | Available / in development / planned / under consideration |

Before implementation, inspect the actual token file and document exact values and contrast ratios in code review. If a semantic status token is absent, define it in the marketing layer only after checking contrast; never infer a brand color from this document.

### Typography roles

| Role | Desktop target | Mobile target | Rules |
|---|---:|---:|---|
| Display / hero | 56–68 px | 38–44 px | 1.02–1.10 line height; max 2–3 lines |
| H1 | 44–56 px | 34–40 px | One per page |
| H2 | 32–40 px | 28–32 px | Section-level message |
| H3 | 22–26 px | 20–24 px | Subsection or card title |
| Body large | 18–20 px | 17–18 px | Hero and lead paragraphs |
| Body | 16–18 px | 16–17 px | Comfortable reading |
| Small / metadata | 13–14 px | 13–14 px | Never use for essential explanations |
| Technical value | 15–18 px | 15–17 px | Tabular numerals; unit always visible |

Use the system font stack already present in the project. Use monospace only for code, identifiers or technical values where alignment benefits comprehension. Body line length target: 60–75 characters; long-form technical pages may use a narrower measure.

### Spacing and geometry

Adopt a 4 px base scale, with common steps 4, 8, 12, 16, 24, 32, 48, 64, 80, 96 and 120 px. Prefer fewer, consistent values over one-off spacing.

| Layout | Container | Grid | Gutters |
|---|---:|---:|---:|
| Wide desktop ≥1440 | max 1240 px | 12 columns | 24 px |
| Desktop 1024–1439 | max 1160 px | 12 columns | 20–24 px |
| Tablet 768–1023 | fluid | 8 columns | 20–24 px |
| Mobile ≤767 | fluid | 4 columns | 20–24 px |

Section spacing target: desktop 88–120 px; tablet 72–88 px; mobile 56–72 px. These are design targets, not rigid values: dense technical tables and compact FAQs may use less.

Radius should be modest and consistent with the existing token system. Avoid shadow as a default depth cue; use surface contrast, whitespace and fine dividers first.

## 4. Responsive behavior

### Breakpoint behavior

- **≥1200 px:** full desktop navigation; two-column hero; multi-column editorial sections.
- **768–1199 px:** compact navigation or menu transition; reduce hero scale; use 8-column grid; prevent dense 3-column cards.
- **≤767 px:** menu button; single-column content; audience cards stacked; tables become labelled stacked rows or horizontally scroll within a clearly indicated region.
- **≤390 px:** preserve 20 px side gutters where possible; no clipped headings, CTA overflow or horizontal page scroll.

Breakpoints must be content-driven and verified at 1440, 1024, 768, 390 and 320 px. Do not simply scale desktop compositions down.

### Mobile-specific rules

- Header remains compact; no fixed bottom CTA bar in the first release.
- Main actions are full-width only when this improves tap clarity; otherwise use naturally sized buttons.
- Tap targets at least 44 × 44 px.
- Long tables must retain column labels and meaningful reading order.
- Diagrams simplify to vertical sequences; labels remain readable without zoom.
- Avoid hover-only information and hover-triggered animation.

## 5. Page composition patterns

### Homepage

1. Header with wordmark and audience-first navigation.
2. Hero: one positioning statement, one short explanation, two audience routes.
3. Audience split: visually distinct Homebrewers and Professional Breweries paths.
4. Brewing lifecycle overview: concise sequence, not a feature-card wall.
5. Technical principles: methods, units and transparency.
6. Learn / validated calculators: real resources only.
7. Product maturity note: accurate current stage.
8. Footer with working links only.

The homepage is a routing and positioning page, not a full product catalog.

### Homebrewers page

- Lead with recipe intent, ingredient context and the home brew-day journey.
- Use one homebrewing-specific visual slot; avoid commercial tank photography.
- Explain calculations and learning with worked examples only when independently validated.
- Use a readable step sequence for recipe → preparation → brew day → fermentation/records, marking product stage accurately.
- Keep professional tank allocation, team permissions and brewery quality workflows out of the primary narrative.

### Professional Breweries page

- Lead with recipe-to-production continuity and operational records.
- Use a separate brewery-specific image slot; do not reuse the Home hero image.
- Prefer process map / responsibility matrix / record relationships over decorative feature cards.
- State the non-ERP boundary in a visible scope section: sales, orders, distribution/shipping, keg tracking, CRM and accounting are not represented as product scope.
- Distinguish planned production, actual records and traceability maturity.

### Features and technical content

- Feature index: group by brewing lifecycle and audience, with explicit status.
- Feature detail: problem → context → method/workflow → example → limitation → product relationship.
- Calculator detail: inputs/units → method/formula → worked example → boundary behavior → limitations → references.
- Technical articles: direct answer first; then reasoning, examples, caveats and sources.

## 6. Component inventory

| Component | Purpose | Required behavior |
|---|---|---|
| Header / mobile menu | Navigation | Keyboard-operable, focus-managed menu; audience routes prominent |
| Breadcrumbs | Orientation | Real hierarchy and working destinations |
| Hero | Positioning | One H1, concise lead, no fabricated UI |
| Audience route panel | Audience separation | Different copy, examples and image slot per audience |
| Lifecycle sequence | Explain process | Ordered steps with status and accessible text alternative |
| Feature row / feature group | Explain capability | Audience, problem, stage, limitation and related link |
| Status label | Product maturity | Text + optional icon; never color alone |
| Evidence / method panel | Technical trust | Source, assumptions, units, validation state and limitations |
| Data table | Compare or explain | Caption, header cells, responsive fallback |
| Calculator shell | Host validated utility | Labelled fields, units, validation, result explanation and reset; only after WP-09/10 gates |
| FAQ disclosure | Resolve questions | Native details/summary or equivalent keyboard-accessible disclosure |
| Article callout | Clarification / caveat | Semantic note; not a decorative colored box |
| CTA link/button | Next action | Real destination; no dead or invented signup/demo flow |
| Footer | Secondary navigation | Only published routes and real legal/social destinations |

Avoid building a component for every one-off block. Reuse should follow repeated information patterns, not force all content into identical cards.

## 7. Home vs Business visual distinction

| Dimension | Homebrewers | Professional Breweries |
|---|---|---|
| Core story | Plan, understand and learn from each brew | Structure recipe, production and batch records |
| Visual context | Homebrew bench, kettle, ingredients, measurement | Brewhouse, stainless vessels, production team |
| Content rhythm | More explanatory, approachable, learning-led | More structured, operational, process-led |
| Primary diagrams | Recipe and brew-day sequence | Recipe approval, production, batch/lot records |
| Data examples | Recipe quantities, gravity, water, fermentation readings | Batch IDs, planned/actual values, lots, QC records |
| Vocabulary | Recipe, brew day, fermenter, batch | Production plan, vessel/tank, lot, batch record |
| CTA | Explore Homebrewing / Read a guide / validated calculator | Explore Brewery workflow / Review scope |
| Exclusions | Do not imply pro operations by default | Explicitly exclude ERP and commercial workflows |

This difference must be visible in page structure and examples, not only in a changed heading and photograph.

## 8. FoamByte benchmark — design lessons, not imitation

| Dimension | FoamByte benchmark observation | Demleme design response |
|---|---|---|
| Feature discoverability | Named feature areas make breadth scannable | Organize by brewing lifecycle and audience; retain clear status |
| Calculator visibility | Calculator directory creates utility entry points | Publish only validated tools with formula and limitation transparency |
| Commercial proof | Pricing / FAQ / proof can reduce evaluation friction | Add only when Demleme has approved pricing and verifiable evidence |
| Information density | Large catalogs can become overwhelming | Progressive disclosure, short summaries and focused detail pages |
| Audience fit | General product presentation may flatten different jobs | Distinct Home and Business page compositions and examples |
| Visual system | Reusable patterns create consistency | Use a restrained Demleme-native system; do not copy layout, copy or visual assets |

## 9. Imagery and asset-slot policy

- Do not generate, source, purchase or add photography as part of this work package.
- Preserve the existing five `data-image-slot` placeholders and their identifiers unless a later approved asset plan explicitly changes them.
- Every slot must have a documented purpose, intended audience, aspect ratio/crop guidance, alt-text intent and replacement note.
- Home and Business hero assets must be different and audience-specific.
- No screenshots unless taken from the real current product and approved for public use.
- If a suitable real image is unavailable, use whitespace, typography or a clearly explanatory diagram rather than a fake interface.
- Asset filenames should be descriptive and stable (for example, `homebrewing-workspace-hero` and `professional-brewhouse-hero`); do not create files until assets are approved.

## 10. Accessibility and interaction requirements

Target WCAG 2.2 AA for all new or revised marketing-site components.

- Text contrast at least 4.5:1 for normal text and 3:1 for large text; meaningful UI boundaries and focus indicators at least 3:1 against adjacent colors.
- Visible keyboard focus; logical tab order; skip link; semantic landmarks and heading hierarchy.
- Menus and disclosures support keyboard and screen-reader use; focus is not trapped unexpectedly.
- Every form field has a persistent label, unit and useful error message. Placeholder text is not a label.
- Status is communicated in text, not color alone.
- Images use meaningful alt text or empty alt for decoration; diagrams have equivalent text summaries.
- Respect `prefers-reduced-motion`; no autoplay, scroll-jacking or motion-dependent meaning.
- Minimum 44 × 44 px touch targets where practical.
- No horizontal page overflow at 320 px; zoom to 200% must preserve content and functionality.

## 11. Motion and interaction

Use motion only to clarify state or spatial relationship. Default transitions should be short and subtle; reduced-motion mode removes nonessential transitions. No carousel, parallax, scroll-triggered reveal dependency, autoplay video or animated counters. Interactive controls must work without hover and provide clear focus, active, disabled and error states.

## 12. Design QA and acceptance criteria

Review at 1440, 1024, 768, 390 and 320 px.

**Visual**
- Hierarchy and section rhythm are consistent.
- Home and Business are unmistakably distinct.
- No fake product UI, unapproved photos, invented metrics or dead CTA.
- Existing brand tokens are reused; no accidental palette drift.
- Long content, tables and diagrams remain readable.

**Functional**
- All navigation and CTA destinations resolve.
- Menu, disclosures and any calculator controls work with keyboard and touch.
- No hover-only behavior or focus loss.
- Status labels match the authoritative product stage.

**Accessibility**
- WCAG 2.2 AA contrast target checked.
- Heading order, landmarks, labels, alt text and focus states checked.
- Reduced-motion and 200% zoom checked.

**Technical**
- No console errors or broken assets.
- Astro/TypeScript checks and production build pass.
- Canonical, metadata, sitemap and robots remain correct after route changes.
- Lighthouse/accessibility review is recorded, not assumed from build success.

## 13. Work-package completion and next gate

WP-06 defines the visual direction, token roles, typography, layout, responsive behavior, audience-specific compositions, component inventory, asset-slot policy and accessibility acceptance criteria.

**Next:** WP-07 — page-level low-fidelity wireframe refinement and content-to-layout mapping, followed by a design review gate. Production implementation remains blocked until the user explicitly authorizes it. This document does not authorize changing `marketing-site/`, live deployment or product application code.
