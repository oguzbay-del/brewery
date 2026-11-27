# GRISTORA Marketing Website Design System

**Status:** Owner-approved visual direction for the public marketing website.  
**Scope:** This document applies only to files and experiences under `marketing-site/`.  
**Primary reference:** The owner-approved GRISTORA brand-board image shared in the project conversation (logo options, brand architecture, palette, typography and website hero concept).

This is the implementation source of truth for the marketing website's visual language. It does not define or govern the product application's UI, product architecture, APIs, data model, mobile app, roadmap or Claude's product-development work.

## 1. Design intent

**Owner decision — 27 November 2026: Editorial Light is the single marketing design direction.** Linear contributes precision, content hierarchy and readable product evidence; Aesop contributes warm paper surfaces, authentic photography and editorial whitespace. No separate Technical Dark theme, dark-mode switch or recurring dark technical sections. Keep Forest Green brand anchors (the existing hero, header/footer where appropriate, restrained closing CTA). Product screenshots retain their original colours. High-contrast accessibility is not a second brand theme. This decision applies only to marketing-site.

The website should feel **premium, confident, editorial, technical and human**. It speaks first to professional breweries, while keeping the Homebrewers audience distinct and secondary.

The approved visual reference combines:
- Deep forest-green brand surfaces and warm amber-gold accents.
- Warm stone-beige canvases and restrained slate supporting text.
- Generous whitespace, calm editorial hierarchy and clear conversion paths.
- Rounded cards, panels, image frames and pill-shaped calls to action.
- Real, authentic brewery photography when licensed assets are available.
- A disciplined combination of modern sans-serif typography and occasional editorial serif accents.

Do not turn the site into a generic dark-tech dashboard, a playful craft-beer shop, or a product-app mockup.

## 2. Brand palette

Use these approved core colours consistently. Existing CSS custom properties should remain the implementation interface; avoid scattering new literal hex values through components.

| Token | Value | Intended use |
|---|---|---|
| Forest Green | `#0B2F26` | Primary brand, hero/dark panels, strong headings and footer accents |
| Amber Gold | `#F4A261` | Primary action, highlights, small brand accents and active states |
| Stone Beige | `#F7F6F3` | Main light canvas, quiet surfaces and whitespace |
| Slate Gray | `#475569` | Secondary text, supporting labels and subdued UI |

### Editorial Light surface and semantic tokens

Reuse the existing marketing CSS variables in `src/styles/global.css` and `src/styles/stitch-layout.css`. The names below describe semantic roles; MQ-03 maps them to the existing variables or introduces aliases only where needed. Do not create a second token system or edit the product token file.

| Role | Value | Existing interface / use |
|---|---|---|
| surface-0 | `#F7F6F3` | --page; default stone-paper canvas |
| surface-1 | `#F0EFEB` | --sx-band; quiet editorial bands |
| surface-2 | `#FFFFFF` | --surface / --sx-card; cards and screenshot frames |
| text-primary | `#0B2F26` | --ink / --forest; headlines and primary text |
| text-secondary | `#475569` | --muted; readable supporting copy |
| border-subtle | existing --line | Decoration only; not a meaningful control boundary |
| border-strong | `#777C73` | Candidate meaningful control and focus boundary; verify actual states |
| accent-text | `#8B4A1F` | --amber; accessible text tone on paper |
| accent-fill | `#F4A261` | --amber-fill; single amber primary action |
| accent-ink | `#0B2F26` | --amber-ink; text on amber |
| success / warning / error / info | `#24643B` / `#875000` / `#A12727` / `#225EAA` | Candidate semantic text colours on paper, only for real feedback; icon and meaningful text required |

Computed WCAG contrast on the three opaque light surfaces, minimum ratio: primary 12.60:1, secondary 6.59:1, accent text 5.89:1, strong border 3.71:1, success 6.16:1, warning 5.74:1, error 6.44:1, info 5.62:1. Forest text on amber 7.03:1; amber fill against stone paper 1.91:1, so use a distinguishable boundary when required. These are sRGB colour-pair calculations, not a page accessibility certification. Check computed states, opacity, images and focus separately.

Retain existing on-dark brand-anchor colours only for the permitted brand sections; no dark technical design mode. Respect prefers-contrast: more and forced-colors: active with visible boundaries, readable system colours and focus; avoid blanket forced-color-adjust:none. Do not add an ordinary theme selector.

### Colour rules
- Forest Green is the anchor; Amber Gold is an accent, not a second dominant background.
- Use Stone Beige instead of stark white for large light surfaces where appropriate.
- Use Slate Gray for supporting information, not low-contrast body text.
- Preserve accessible text contrast. Do not place small amber text on light beige or small dark-green text on amber unless contrast is verified.
- Do not introduce extra brand colours without a clear semantic need and documented rationale.
- Gradients, glow effects and saturated decorative colour fields are not part of the core identity.

## 3. Logo and brand lockups

Use only the owner-approved GRISTORA vector artwork already maintained in `public/brand/`.

- Primary public identity: GRISTORA.
- Descriptor: **Brewing Intelligence**.
- Supporting brand line where suitable: **Clarity for every brew.**
- Use the approved full lockup when space permits; use the approved symbol for compact navigation, favicon and small-format contexts.
- Select the dark or light lockup according to the background; maintain clear contrast and adequate clear space.
- Preserve original proportions, spacing, lettering and emblem geometry. Never redraw, skew, stretch, recolour arbitrarily or recreate the wordmark with ordinary text.
- Do not use draft logo files for public-facing pages.
- Do not add unapproved slogans or change the descriptor.

## 4. Typography

Use the brand-board typography direction:

| Role | Family | Guidance |
|---|---|---|
| Display and sans-serif headings | Plus Jakarta Sans | Strong, clear, contemporary; use restrained weight and tight but readable tracking |
| Body, navigation and interface | Inter | Highly legible, neutral and consistent |
| Editorial accent | Existing approved serif treatment | Use sparingly for short emphasis or supporting editorial lines, not for UI controls or long body copy |

### Type hierarchy
- Editorial Light candidate scale: fluid hero 40–72px, section 30–48px, body 16–18px, longer reading measure about 55–70 characters. MQ-03/06 validate existing values and long TR text before implementation; never force line counts or fixed heights.
- One clear H1 per page, followed by logically nested H2/H3 sections.
- Headlines should be concise and confident; avoid oversized all-caps blocks.
- Body copy should be comfortable to scan, with adequate line height and sensible line length.
- Use consistent font weights; avoid excessive bold text and letter spacing.
- Load web fonts efficiently and retain robust system-font fallbacks.

## 5. Shape, radius and component geometry

Rounded geometry is a defining part of the approved reference. Avoid sharp, square-cornered UI.

Recommended shared tokens:

| Token | Value | Apply to |
|---|---:|---|
| Small control / chip | 10–12px | Compact inputs, tags and small utility surfaces |
| Card | 18px | Audience, feature, workflow and content cards |
| Large panel | 24px | Hero containers, large feature panels and prominent visual sections |
| Pill | 999px | Primary/secondary CTA buttons and pill controls |

- Cards should have consistent radii within a component family.
- Use subtle borders and restrained shadows; do not stack heavy borders and heavy shadows.
- Buttons should be pill-shaped, with comfortable horizontal padding and clear hover/focus/disabled states.
- Inputs must remain accessible and usable; rounded styling must not reduce clarity.
- Avoid nested rounded boxes without a meaningful content hierarchy.

## 6. Layout, spacing and responsive behaviour

Use the existing 1240px maximum content shell as the starting point. Candidate shared spacing scale 4/8/16/24/32/48/64/96px, conceptual grid 12/8/4 desktop/tablet/mobile, fluid clamp values where appropriate. Prefer CSS Grid/Flexbox and logical properties; set min-inline-size:0 for flexible content. Avoid hiding overflow to conceal errors. Technical sections use paper/white surfaces, legible original product screenshots and precise captions rather than dark panels.

- Use a consistent centred content container and shared spacing scale.
- Prefer generous section spacing and whitespace over decorative separators.
- Align headings, cards, images and CTA groups to a clear grid.
- Use consistent card gaps; do not let each page invent its own spacing rules.
- On desktop, use editorial split layouts where useful: message and CTA on one side, supporting brewery visual on the other.
- On tablet, simplify multi-column layouts before content becomes cramped.
- On mobile, stack content in a deliberate reading order, preserve comfortable gutters and make CTAs easy to tap.
- Prevent horizontal overflow at narrow widths; test at 320px and 390px as well as tablet and desktop widths.
- Preserve visible keyboard focus and usable layouts at 200% zoom.

## 7. Photography and illustration

The reference direction favours authentic brewery environments: stainless-steel fermentation tanks, production floors, brewing teams and real brewing activity.

- Prefer high-quality, rights-cleared photography for hero and audience imagery when available.
- Do not download, publish or imply permission for unlicensed images.
- Until approved photography is supplied, retain the existing original editorial SVG illustrations as clearly illustrative placeholders.
- Do not present illustrations as customer sites, product screenshots or evidence of live product functionality.
- Keep image treatment restrained, with rounded frames and natural tonal integration into the palette.
- Maintain descriptive alt text for informative images; decorative images should not create redundant screen-reader output.

## 8. Page and conversion patterns

- Professional Breweries is the primary commercial path and should receive the clearest homepage hierarchy and primary CTA.
- Homebrewers is a separate, secondary audience path; do not blend the two audiences into one ambiguous offer.
- Use a consistent header, footer, navigation, breadcrumb and CTA language across all public pages.
- Make primary and secondary actions visually distinct; avoid multiple competing primary buttons in one section.
- Pricing, plan comparison and configuration-preview pages must preserve their existing non-live/disclaimer language.
- Do not imply that login, account creation, purchase, payment, trial, subscription or production workflows are live unless explicitly verified and approved.
- Do not invent customer logos, testimonials, usage statistics, outcomes, certifications, integrations or product screenshots.

## 9. Motion and interaction

- Keep motion subtle, purposeful and short; it should clarify interaction rather than decorate the page.
- Respect `prefers-reduced-motion`.
- Provide visible hover, focus, active and disabled states.
- Do not rely on colour alone to communicate state.
- Avoid parallax, looping animation, excessive transitions and motion that distracts from the content.

## 10. Accessibility and quality gates

Every marketing-site change must preserve:
- Semantic HTML and a logical heading hierarchy.
- Keyboard access to all interactive controls.
- Visible focus indicators and accessible names.
- Sufficient colour contrast.
- Responsive behaviour at 320, 390, 768, 1024 and 1440px widths.
- Usability at 200% zoom.
- Reduced-motion support.
- Meaningful image alt text and no fabricated UI affordances.

Run the marketing-site checks and browser/accessibility tests documented in `README.md`. Follow the mandatory [Marketing VQG visual test flow](DESIGN-EXCELLENCE.md#61-repeatable-visual-test-flow-owner-decision-27-november-2026) on each applicable visual change: exact-head evidence, EN/TR responsive screenshots, geometry, contrast/focus/axe, reviewed before/after and defect triage. Automated checks do not replace visual review on desktop and mobile. No open P0/P1 visual defects at visual acceptance. Keep existing hard checks; new geometry checks start in report mode, then become gates after validation. Protected screenshot baselines and workflow rules remain unchanged.

## 11. Implementation rules

- Keep all work within `marketing-site/`.
- Prefer shared CSS variables and reusable components over page-specific visual overrides.
- When adding a new component, use the existing tokens and established site patterns.
- Do not create a competing palette, radius scale, type system or one-off button language.
- Do not edit product application code, APIs, Rust engine, data model, iOS app, internal product identifiers, product roadmap or product-development task queues.
- This document governs only the marketing website. Product UI and product engineering must follow their own separate instructions and sources of truth.

## 12. Conflict resolution

For the public marketing website:
1. This `marketing-site/DESIGN.md` governs visual implementation.
2. `marketing-site/BRAND-ASSETS.md` governs approved asset filenames, logo asset usage and the recorded brand palette.
3. The owner-approved brand-board image is the visual reference for the overall look and feel.
4. `marketing-site/README.md` governs website scope, build, QA and deployment instructions.

If older general strategy notes or exploratory design directions conflict with this document, do not silently reintroduce the conflicting direction into the marketing website. Keep the change within `marketing-site/` and request owner clarification only when the approved reference itself is ambiguous.
