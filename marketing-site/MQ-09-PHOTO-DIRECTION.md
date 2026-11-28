# MQ-09 — Editorial photo composition

Source inventory and round 1 review, 28 November 2026. See src/data/photos.ts for photo descriptions, PHOTO-ASSETS.md for source assets and the build process. This document records the evidence available; it is not visual acceptance or rights certification.

## Direction

Use one focal subject per section, natural brewery material, quiet paper backgrounds and a clear reading order. Do not use a brewery image as evidence of a GRISTORA customer or released feature. Keep the approved originals and existing derived assets. Do not import Linear or Aesop assets. Do not add a dark technical theme.

## Existing rendering contract

Source: src/components/PagePhoto.astro, blob 42cd836ccfeb399389529acf61a554a00798e972.

| Family | Declared intrinsic frame | Candidate widths | Fallback |
|---|---|---|---|
| Curated feature | 896 x 597 | 640, 896 | JPEG 896 |
| Other feature | 1152 x 768 | 480, 640, 768, 960, 1152 | WebP 960 |
| Page hero | 1408 x 768 | 480, 640, 768, 960, 1408 | WebP 960 |

All use AVIF first, WebP second, localized descriptive alt text, lazy loading except explicitly eager images, and asynchronous decoding. The eager image receives high fetch priority. The homepage hero sizes expression specifies 1440px maximum and narrow gutters; this is not the actual rendered width (see defect below). Curated photos are capped at 896px in the sizes expression.

## Verified optimizer contract

Source: scripts/optimize-photos.py blob 9ce9cc376f87715158a12334faadb7ee0f96fb43. Feature photos are cropped to 3:2 before encoding; the named FOCUS_Y overrides and default are all 0.50. Hero images preserve source aspect in the optimizer. Global CSS uses a 1408:768 hero frame, a 3:2 feature frame, and 4:3 at <=640px. The CSS therefore performs a second crop on some mobile images. AVIF quality 65 and WebP quality 84 are existing settings; curated sources also receive progressive JPEG quality 90. The optimizer reads raw JPG/PNG files and the owner-supplied ZIP; it skips superseded curated keys and writes only derived files. The twelve curated originals are 896×1200 portrait JPEGs; the 896×597 center crop retains approximately half their height.

## Round 1 crop evidence

Source: Claude review on PR #469 at head ccdae1ee9fa2f47f5f4e7a52d7146bdafd2cc314. Claude checked all 27 keys, candidate files and fallbacks (zero missing), compared a curated source/output contact sheet, and inspected element screenshots at 320/390/768/1024/1440 for four representative pages. The following records are limited to that review; no independent screenshot inspection by ChatGPT is claimed.

| Key | Current focus / delivery | 320 | 390 | 768 / 1024 | 1440 | Status |
|---|---|---|---|---|---|---|
| home-hero | CSS 62% 50% at <=760px; 50% 48% desktop | No specific finding recorded | Subject very dark; headline readable | No specific finding recorded | Empty forest strip with hard edge | MQ-07 defects |
| breweries-hero | Center; hero frame, 4:3 mobile | Brewer retained; right vessel clipped | No separate finding | No separate finding | No separate finding | Partial evidence |
| features-hero | Center; hero frame, 4:3 mobile | Subject retained in review | No separate finding | No separate finding | No separate finding | Partial evidence |
| how-it-works-hero | Center; hero frame, 4:3 mobile | Subject retained in TR review | No separate finding | No separate finding | No separate finding | Partial evidence |
| homebrewers-hero | Hero frame | Not checked | Not checked | Not checked | Not checked | Open |
| technical-principles-hero | Hero frame | Not checked | Not checked | Not checked | Not checked | Open |
| compare-hero | Hero frame | Not checked | Not checked | Not checked | Not checked | Open |
| pricing-hero | Hero frame | Not checked | Not checked | Not checked | Not checked | Open |
| faq-hero | Hero frame | Not checked | Not checked | Not checked | Not checked | Open |
| get-started-hero | Hero frame | Not checked | Not checked | Not checked | Not checked | Open |
| 404 | Hero frame | Not checked | Not checked | Not checked | Not checked | Open |

The review reported subject retention on the features and how-it-works hero crops generally; individual width-specific results other than the breweries 320px finding were not separately enumerated. Verify each before acceptance.

### Curated feature crop findings

| Feature key (slug) | Contact sheet finding | Proposal (not applied) |
|---|---|---|
| fermentation-monitoring | Top of airlock is lost | Try FOCUS_Y 0.25, compare original and derived crop |
| team-notifications | One brewer's face cut at right edge | Inspect horizontal crop and mobile frame first; a vertical focal shift alone may not solve this |
| production-management | Principal subject retained | Keep existing focus pending responsive review |
| quality-laboratory | Principal subject retained | Keep existing focus pending responsive review |
| packaging, analytics, inventory | Principal subjects retained per review | Confirm exact key mapping and responsive results |
| Remaining curated and other feature keys | No individual finding supplied | Not checked |

These are contact-sheet findings, not viewport-specific acceptance. The proposed fermentation value is a candidate, not a measured optimum. Do not modify the optimizer, raw ZIP or derived assets until the actual before/after crops are reviewed. Do not set a focal point from alt text alone.

## Defects for MQ-07 (homepage pilot)

- At 1440px, Claude found the global .page-photo max-width of 1180px limits the photo inside a 1408px hero frame. The right 228px is empty forest color with a hard edge. Fix belongs to MQ-07, not global CSS changes in MQ-09.
- At 390px, the photo overlay makes fermenters and brewer barely recognizable. White headline remains readable. Compare overlay and text contrast before making a change.
- Claude took and viewed screenshots, including ph-home-390. The PNG bytes are not attached here; request or link the actual evidence before visual approval.

## Remaining work

- Complete the per-key source file and size inventory from PHOTO-ASSETS.md and photos.ts. Verify the 16 feature keys through featurePhoto(slug), and list actual EN/TR page-to-key placement from built HTML. The review did not verify all feature placements.
- Review every actual frame at five widths and both languages. Record missing evidence as not checked. Keep image alt text accurate; the review found localized strings for all 27 keys.
- Current reviewed head marketing CI 46123343127 and guard 46123343070 passed. Lighthouse job passed in CI, but comparative Lighthouse measurement was not run by Claude. No comparative performance claim.
- The owner supplied the raw archive. Rights and original source provenance remain unverified; do not certify them.
- No source photos, CSS, optimizer, public copy, dependencies or production settings changed in this PR. Complete screenshot review and exact-head CI before acceptance.
