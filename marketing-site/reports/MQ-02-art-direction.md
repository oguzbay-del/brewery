# MQ-02 — Art direction: Editorial Precision

27 November 2026. Issue #451. Marketing only. Follow DESIGN.md, BRAND-ASSETS.md and DESIGN-EXCELLENCE.md. This is an original pilot design proposal, not a screenshot of the current site or an approved replacement of its layout.

## References and translation

Reviewed the public page content on 27 November:
- [Linear homepage](https://linear.app/): a concise product proposition followed by a detailed product workspace, then sections grouped by workflow (intake, planning, automation, delivery). Translate the structure into one brewing question per section, followed by real GRISTORA evidence and one next action. Do not copy Linear UI, brand, assets, dark theme or text.
- [Aesop homepage](https://www.aesop.com/): thematic editorial introductions, product imagery, short descriptions and individual discovery links; longer stories and reading appear alongside product categories. [Design approach](https://www.aesop.com/philosophy-on-design.html). Translate the approach into restrained editorial rhythm, brewing materials, natural photography and generous whitespace. Do not copy retail patterns, assets or text.
- Reference review used retrieved HTML/text. Pixel geometry, animation, font metrics and responsive behavior of the external sites were not measured. The design decisions below are original GRISTORA proposals.

## Visual grammar

Keep forest #0B2F26 for brand anchors, stone #F7F6F3 for editorial canvas, slate #475569 for text and amber #F4A261 for emphasis. Use existing accessible dark amber text token on light surfaces. Preserve approved vector logo, Plus Jakarta Sans, Inter, occasional serif emphasis and existing radius tokens. Do not recolor the product screenshots. Do not introduce external fonts or dependencies.

Owner decision (27 November 2026, MQ-00b #456): Editorial Light is the single marketing design direction. Use stone paper, warm paper and white surfaces for editorial and technical content. Technical precision comes from hierarchy, whitespace, original product screenshots and restrained rules, not dark panels. Forest Green remains only in existing brand anchors (hero, header/footer as appropriate, and restrained closing CTA). No Technical Dark theme, ordinary dark-mode switch or recurring dark technical panels; high contrast remains an accessibility adaptation. Use one primary CTA per section; secondary actions are understated links. Keep one focal image per section. Prefer image/text split sections over repeating rows of small cards. Avoid decorative dashboards, heavy shadows, floating fake UI, parallax and autoplay.

Candidate layout: use existing 1240px shell initially, with 32px desktop gutters subject to MQ-03 audit; 12/8/4 conceptual column structure; mobile gutters from existing shared CSS. Use flexible grid with minmax(0,1fr). Prefer natural content height and wrap to a new line. Do not lock a paragraph or action group to a fixed height. Use existing type tokens before introducing new ones. Headings use display family, body Inter, serif only a short editorial accent. Proposed reading width 55–70 characters for longer prose; verify both languages. Maintain visible focus and accessible contrast.

## Original homepage pilot sketch

Desktop reading order, not fixed pixel geometry:

    [ shared header / logo / nav / language / primary action ]
    [ forest hero: clear proposition + brewery CTA | authentic brewery image ]
    [ stone: concise bridge — brewing work, not a feature list ]
    [ warm paper / white technical spotlight: real product screenshot | title + context + link ]
    [ editorial split: physical brewing detail | workflow narrative ]
    [ recipe / brew day / batch review — concise sequential evidence ]
    [ professional brewery path (primary) | homebrewer path (secondary) ]
    [ selected verified feature links / technical principles ]
    [ quiet plan discovery CTA ]
    [ shared footer ]

Mobile: header with native menu; proposition then actions then image where a background crop harms contrast or cuts off context; screenshot with caption below; each split section stacks in semantic reading order; feature and audience groups become one column where necessary; CTA and text wrap; footer remains accessible. Preserve existing routes and all approved content. Do not move primary audience behind secondary. Do not require horizontal scrolling except labelled intentional table or diagram regions.

## Pattern specifications

| Pattern | Treatment | Reuse / validation |
|---|---|---|
| Hero | One focal brewing photo, concise headline and two differentiated actions; dark forest overlay only if contrast is verified | Existing homepage hero and PagePhoto; check 320/390/768/1024/1440 and zoom |
| Technical spotlight | One large real screenshot in restrained white or warm-paper frame; adjacent short context and verified feature link; avoid tiny illegible composite grids | ProductScreens and MQ-08 register; verify source, readability, caption and image dimensions |
| Editorial split | Real production, human or ingredient detail with short narrative; alternate image side only when DOM order is preserved | PagePhoto, shared section pattern; preserve original crop and alt metadata |
| Workflow | Numbered sequential steps, examples clearly marked, links to actual scope | Existing WorkflowLinks and feature data; no new claim |
| Audience | Professional path visually primary, separate Home route; no competing identical primary actions | Existing audience layout and CTAs |
| Catalogue | Consistent card hierarchy; clear grouping and short labels, retain links to all details | Existing Stitch cards and feature/module templates |
| Commercial | Clear comparison and plan preview; keep disclaimers, keyboard navigation and intentional table scroll | Existing pricing/compare/wizard, no commercial changes |
| Closing | Calm final CTA on forest or stone; no fake sign-up or social proof | Existing closing and SiteFooter |

## Image and screenshot art direction

Select from owner-provided brewery photography. Prefer real tanks, brewery floors, hands and instruments, raw ingredient detail. Avoid stock-style staged imagery, alcohol lifestyle imagery unrelated to production, unlicensed assets, customer implications and heavy color grading. Keep original source unchanged; use existing AVIF/WebP/JPEG pipeline. Review focal point and brightness at both narrow and wide crop. Prefer meaningful alt text, intrinsic dimensions, lazy-load below-fold images and preserve hero priority. Use MQ-08 screenshot register and MQ-09 photo inventory rather than creating competing asset lists. English product screenshots remain the shared initial asset for both locales; caption accurately and never imply a localized UI.

## Review and acceptance

- Before implementation record current screenshot and commit baseline under MQ-01 and run MQ-11a early visual QA. Use the latest DESIGN.md and DESIGN-EXCELLENCE.md (MQ-00b #456). The above sketch is a proposal to test in MQ-07; it does not authorize a live release.
- At each of five marketing viewport widths, compare EN/TR visual hierarchy, line wrap, CTA visibility, image crop, screenshot readability, grid alignment and keyboard focus; check actual 200% zoom manually.
- Use existing Playwright/axe and Lighthouse. Keep current metadata, canonical routes, mobile navigation, reduced motion, and approved visual assets.
- Keep claim text unchanged until the G8 evidence and claim decisions permit a correction. No invented metrics, customer logos, trial, purchase or live integrations.
- Designer review uses MQ scorecard and actual before/after evidence. Obtain owner visual approval of homepage pilot before broad rollout. No unapproved changes to web/, ios/, product tokens, dependencies or production settings.
- MQ-03 should map these patterns to existing tokens; MQ-04 should extract only useful Astro components; MQ-06 establishes shared responsive layout. MQ-07 integrates after prerequisites. MQ-02 does not edit runtime files.

