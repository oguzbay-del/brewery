# GRISTORA — Logo production and QA specification

**Status:** Production specification; owner-preferred first design is the fixed creative direction. This document does not approve a substitute logo or claim that the reference artwork has been traced.

## 1. Source-of-truth rule

- Master artwork must be traced/refined from the first GRISTORA logo shown by the assistant and identified by the owner in the shared reference: https://chatgpt.com/s/m_6af9ac3f07e48191aa8504a2e22a77a9
- Preserve its recognizable composition, letterforms, symbol relationship and visual character.
- The later exploration boards are not substitutes and must not be silently promoted to master status.
- If the original image cannot be accessed by the working environment, do not fabricate a faithful reconstruction. Continue only with the specifications and QA preparation below.

## 2. Deliverable structure

When the reference artwork is available, create an isolated public-brand asset set:

- `web-stratejisi/assets/gristora/`
  - `gristora-master.svg` — editable, clean vector master
  - `gristora-horizontal.svg` — primary horizontal lockup, only if consistent with source design
  - `gristora-stacked.svg` — stacked lockup, only if composition permits
  - `gristora-symbol.svg` — symbol-only mark, only if source contains a separable symbol
  - `gristora-mono-dark.svg` and `gristora-mono-light.svg`
  - `gristora-favicon.svg`
  - `README.md` — usage, variants, source and export notes

Raster derivatives are generated from SVG at build/export time; do not hand-edit raster copies as masters.

## 3. Vector quality requirements

- Use explicit paths for custom lettering and geometry where practical; avoid relying on installed fonts for final wordmark rendering.
- No embedded raster image in the master SVG.
- Use a consistent viewBox, tight but optically balanced bounds, and no unnecessary metadata or editor-specific namespaces.
- Keep fills/strokes editable and avoid filters, blur, complex masks or gradients unless they are intrinsic to the selected reference.
- Use optical rather than purely mathematical spacing; inspect kerning pairs at full size and at reduced sizes.
- Ensure SVG is self-contained and renders consistently in Chromium, Safari/WebKit and common vector editors.
- Preserve the original design's proportions; responsive variants may rearrange elements only when that does not alter its identity.

## 4. Required proof board

Show the same selected design—not alternate concepts—in these contexts:

1. Primary header lockup on warm ivory.
2. Primary header lockup on deep forest/dark background.
3. Single-colour dark and reversed white versions.
4. Social avatar 1:1, with and without a wordmark only where the source supports both.
5. Favicon at 16, 32 and 48 px.
6. Marketing social card at 1200×630.
7. Small-size legibility at 24, 16 and 12 px where applicable.

The proof board must label any crop or responsive rearrangement. Never redraw the mark differently just to fit a small canvas; if the mark does not survive, record the limitation.

## 5. Colour and accessibility checks

Initial palette is a working brand palette, not a locked requirement:

- Deep forest: `#17372E`
- Warm ivory: `#F5F1E8`
- Malt copper: `#C88B4B`
- Slate: `#68736D`

Test the mark in pure black and white independently of the palette. A logo itself is not a substitute for accessible text contrast; adjacent descriptor and navigation text must meet WCAG 2.2 AA for their size.

## 6. Acceptance checklist

- [ ] Artwork visibly matches the owner's first-design reference.
- [ ] Owner reviews the rendered proof before final approval.
- [ ] Wordmark remains legible at intended header size.
- [ ] Symbol, if present, remains recognizable at 32 px; 16 px behaviour is documented.
- [ ] Monochrome variants preserve the identity.
- [ ] SVG is self-contained, accessible when inline, and free of external dependencies.
- [ ] No clipping, stray paths, inconsistent stroke joins or unintended overlaps.
- [ ] Header, avatar and favicon proofs are generated from the same approved master.
- [ ] Trademark/domain clearance gate is reviewed separately; design quality is not legal clearance.
- [ ] Marketing-site integration remains blocked until owner approval and clearance gates are satisfied.

## 7. Explicit scope boundary

This specification applies only to GRISTORA's public brand and public marketing website. It does not authorize changes to the software product, app icon, product UI, APIs, Rust engine, iOS app, internal identifiers, data model, product roadmap, or development task queue.
