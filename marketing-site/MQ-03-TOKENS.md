# MQ-03 — Marketing token contract

Scope: marketing-site only. DESIGN.md governs the target. This records the implemented CSS foundation, not a visual acceptance or product token change.

## Effective token map

| Role | CSS variable and value |
|---|---|
| Paper | --page #F7F6F3 |
| Editorial band | --sx-band #F0EFEB |
| Card | --surface #FFFFFF; --sx-card aliases --surface |
| Primary text and brand | --ink and --forest #0B2F26; separate semantic roles |
| Supporting text | --muted #475569 |
| Decorative boundary | --line rgba(11,47,38,.18); not a control boundary |
| Strong boundary | --border-strong #777C73; candidate for meaningful boundaries, verify actual state |
| Focus | --focus-ring #0066cc on light surfaces; --focus-ring-on-dark aliases --amber-fill #F4A261; 3px outline and 4px offset |
| Accent text | --amber #8B4A1F |
| Accent fill | --amber-fill #F4A261 |
| Accent ink | --amber-ink #0B2F26 |
| Quiet panels | --soft #E8E7E3 and --sx-soft #F4F3F0; distinct surfaces |
| Brand anchor | --sx-on-dark aliases --page; --sx-on-dark-muted #C9D3CF; --sx-amber-on-dark aliases --amber-fill |
| Content width | --content 1240px |
| Font body | --sans Inter Variable, Inter and system fallbacks (effective later root) |
| Font headings | --display Plus Jakarta Sans Variable, Plus Jakarta Sans, Inter Variable, sans-serif |
| Editorial accent | --serif Georgia, Times New Roman, serif |
| Radii | --radius-card 18px; --radius-panel 24px; --radius-chip and --radius-control 999px |
| Shadow | --card-shadow existing two-layer low-opacity forest shadow |

## Layout and interaction

- Shared spacing variables --space-4/8/16/24/32/48/64/96 use the corresponding pixel values. Use them for new patterns; do not rewrite every existing spacing value in this package.
- The shared shell keeps the 1240px cap with symmetric --gutter. Existing responsive gutters are preserved: 32px default, 24px at <=1050px, 22px at <=760px and 18px at <=380px.
- Keep existing heading sizes until MQ-06 verifies the proposed fluid type scale and long TR text. The conceptual 12/8/4 grid guides layout; avoid fixed tracks when content needs flexibility. Use minmax(0,1fr), logical dimensions and natural height.
- Preserve the existing skip-link layer (20), native focus and reduced-motion rules. Do not introduce a second theme or global stacking scale without checking component needs.
- The quote band now uses paper surfaces with paired text, emphasis and link colors. The technical-section selectors define a paper-surface style, but no built page uses them at the reviewed head. The module-boundary change in modules.css is part of this shared surface work; audience-pages.css was not changed, and an ERP-specific conversion is not claimed. Retain hero/closing CTA brand anchors. Further recurring dark panels require paired foreground/background review, not a global inversion.
- Candidate status colors and stronger boundaries in DESIGN.md are available for future real controls/feedback; do not add unused status styles.

## Evidence and remaining work

The CSS commits c3b8f02 and 86d36f0 apply the above aliases, spacing, gutters, focus and paired light surfaces. The exact CSS head 86d36f03e8d7bf92c360b4f85e2cba6a9a7af9cd passed marketing CI 46123275378, protected-path guard 46123275347 and 296 browser tests. Preview: https://30e23af0.gristora.pages.dev. The screenshot upload and Lighthouse steps passed; the report-only results are not visual acceptance. Verify the final documentation head CI before merge. Review the remaining report-mode contrast, image bounds and actual before/after screenshots. High contrast and actual 200% browser zoom still need visual verification. MQ-04 and MQ-06 consume the shared contract after acceptance. No product tokens, protected baselines, new dependencies or production settings changed.

## Round 1 evidence and open items

Claude review on PR #467, head 847cd60, compared base d980641 and head in Chromium. The following contrast ratios are from that review, not a new measurement.

| Pair | Contrast ratio |
|---|---:|
| Ink / page; band | 13.4; 12.6 |
| Muted / page; band | 7.0; 6.6 |
| Amber / page; band; white; soft | 6.3; 5.9; 6.8; 5.5 |
| Amber ink / amber fill | 7.0 |
| On-dark muted / forest | 9.5 |
| Strong border / page; band; white | 3.95; 3.71; 4.27 |
| Blue focus / page; band; white | 5.2; 4.8; 5.6 |
| Blue focus / forest | 2.6 (below 3:1; needs scoped correction) |

- All 54 pages at 320, 390, 768, 1024, 1440 px had no page overflow in Claude's head scan. Base TR breweries overflowed at 320 px (336 px scroll width). The 640 and 720 px tests are zoom proxies, not actual browser zoom.
- Claude viewed the quote band (EN desktop and TR 390) and focused skip link (390). Skip link measured 172×46 px and did not overlap the home link; in-flow focus moves page content down 58 px. The quote edge is faint (decorative, 1.07:1).
- Forest focus: inspect home hero and pricing forest surfaces with keyboard focus. Use the existing amber-on-dark semantic color where the blue outline lacks contrast. Do not claim browser verification until screenshots are reviewed.
- Forced-colors overrides use !important only inside that media query so user-selected system colors win over local component colors. Actual forced-colors screenshots remain open.
- MQ-06 backlog: legacy 6.5 px wordmark subtitle and 8 px note-kicker; check legibility and responsive type rather than changing in this package.
- Current reviewed head 847cd60: marketing CI 46133802751 and guard 46133802753 passed. Full EN/TR before/after screenshot set, real browser zoom and forest focus screenshots remain open. No visual acceptance.

## Scoped focus correction

Commit eadc01a adds --focus-ring-on-dark (alias of --amber-fill) and applies it to the home hero header and action area only. The solid forest header is a confirmed source-level dark surface. Amber-fill / forest is 7.0:1 in the round 1 contrast measurement above; blue / forest was 2.6:1. This ratio is not a browser test of every focused element or the photo overlay. The effective stitch-layout.css rules make pricing-page plan-business and start-page wizard-summary light cards, so do not blindly assign a dark-surface token to those classes. Compare-page table headers are also light in that cascade. Existing dark CTA and audience band focus styles remain. Check actual focus on EN/TR home and pricing, forced-colors, and focus visibility at mobile and desktop. No visual approval yet.
