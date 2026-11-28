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
| Focus | --focus-ring #0066cc; existing link and summary focus color, 3px outline and 4px offset |
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
- The quote and technical story now use paper surfaces with matching text, emphasis, link and boundary colors. Retain hero/closing CTA brand anchors. Further recurring dark panels require paired foreground/background review, not a global inversion.
- Candidate status colors and stronger boundaries in DESIGN.md are available for future real controls/feedback; do not add unused status styles.

## Evidence and remaining work

The CSS commits c3b8f02 and 86d36f0 apply the above aliases, spacing, gutters, focus and paired light surfaces. The exact CSS head 86d36f03e8d7bf92c360b4f85e2cba6a9a7af9cd passed marketing CI 46123275378, protected-path guard 46123275347 and 296 browser tests. Preview: https://30e23af0.gristora.pages.dev. The screenshot upload and Lighthouse steps passed; the report-only results are not visual acceptance. Verify the final documentation head CI before merge. Review the remaining report-mode contrast, image bounds and actual before/after screenshots. High contrast and 200% zoom still need visual verification. MQ-04 and MQ-06 consume the shared contract after acceptance. No product tokens, protected baselines, new dependencies or production settings changed.
