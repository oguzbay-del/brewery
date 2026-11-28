# MQ-03 — Marketing token inventory

Status: initial source inventory, not a completed visual change.
Source: main, 28 November 2026; global.css blob 1488817355ffb0a46b1a7af3e7f69a46d3c76f22 and stitch-layout.css blob 750bdfdd9e87e9033149fd968d2e3842f75542e3. DESIGN.md governs the target.

| Role | Current variable/value | Target |
|---|---|---|
| Paper | --page #F7F6F3 | Keep |
| Editorial band | --sx-band #F0EFEB | Keep |
| Card | --surface and --sx-card #FFFFFF | Alias --sx-card to --surface after cascade review |
| Primary text | --ink and --forest #0B2F26 | Preserve brand value; alias only if semantics remain distinct |
| Supporting text | --muted #475569 | Keep |
| Decorative boundary | --line rgba(11,47,38,.18) | Decoration only |
| Meaningful boundary | Candidate #777C73 in DESIGN.md | Verify computed contrast and focus before use |
| Accent text | --amber #8B4A1F | Keep |
| Accent fill | --amber-fill #F4A261 | Keep |
| Accent ink | --amber-ink #0B2F26 | Keep |
| Quiet panels | --soft #E8E7E3; --sx-soft #F4F3F0 | Distinct surfaces; do not merge without checking usage |
| Brand anchor | --sx-on-dark #F7F6F3; --sx-on-dark-muted #C9D3CF; --sx-amber-on-dark #F4A261 | Limit to approved brand anchors |
| Content width | --content 1240px | Keep |
| Typography | Inter and Plus Jakarta Sans imports; --serif Georgia; --sans initial system stack | Verify effective later declarations and font use |

## Implementation order
1. Read the complete effective cascade and layout imports. Inventory existing radius, font, shadow and breakpoint declarations before changing them.
2. Map the approved 4/8/16/24/32/48/64/96 spacing candidates to existing usage. Preserve responsive gutters and use logical dimensions. Avoid a duplicate palette.
3. Convert recurring dark technical/editorial sections to light surfaces through paired foreground and background changes, not independent color changes. Keep hero/header/footer/closing CTA brand anchors where approved.
4. Inspect MQ-11a contrast reports by state and selector. Forced colors and image overlays require separate checks; do not assume all 72 observations have one cause.
5. Use exact-head browser and Lighthouse evidence; compare EN/TR at five widths and review focus, 200% zoom and long strings. Keep report-mode findings separate from acceptance.

No product tokens, dependencies, baseline changes or new theme. This initial commit does not change CSS or certify visual quality.
