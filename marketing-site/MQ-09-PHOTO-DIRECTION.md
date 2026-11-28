# MQ-09 — Editorial photo composition

Initial implementation plan and source inventory, 28 November 2026. The source of truth for photo descriptions is src/data/photos.ts; for approved source assets and the build process see PHOTO-ASSETS.md. This is not a visual inspection or a rights certification.

## Direction

Use one focal subject per section, natural brewery material, quiet paper backgrounds and a clear reading order. Do not use a brewery image as evidence of a GRISTORA customer or released feature. Keep the approved originals and existing derived assets. Do not import Linear or Aesop assets. Do not add a dark technical theme.

## Existing rendering contract

Source: src/components/PagePhoto.astro, blob 42cd836ccfeb399389529acf61a554a00798e972.

| Family | Declared intrinsic frame | Candidate widths (CSS source descriptors) | Fallback |
|---|---|---|---|
| Curated feature | 896 x 597 | 640, 896 | JPEG 896 |
| Other feature | 1152 x 768 | 480, 640, 768, 960, 1152 | WebP 960 |
| Page hero | 1408 x 768 | 480, 640, 768, 960, 1408 | WebP 960 |

All use AVIF first, WebP second, localized descriptive alt text, lazy loading except explicitly eager images, and asynchronous decoding. The eager image receives high fetch priority. The homepage hero sizes contract differs from the other page photos: 1440px maximum and narrow gutters. Curated photos are capped at 896px in the sizes expression. Verify that emitted optimizer files match every candidate and actual rendered sizes before making performance claims.

## Crop review checklist

| Photo key or family | Subject to retain (from existing alt metadata, not image inspection) | Check |
|---|---|---|
| home-hero | Brewer and row of fermenters | Both locales use same approved source; verify text overlay readability and subject at 320/390/768/1024/1440 |
| breweries-hero | Brewer at stainless-steel brewhouse | Keep subject readable, avoid clipping equipment context |
| homebrewers-hero | Kettle, fermenter and ingredients | Preserve recognizable brewing context |
| features-hero | Malt, hops and measurement tools | Avoid cropping out the technical subject |
| how-it-works-hero | Brewer drawing a sample | Preserve action and sample |
| technical-principles-hero | Lab tools and probe | Preserve technical focal point |
| compare-hero, pricing-hero, faq-hero, get-started-hero, 404 | Existing localized metadata | Confirm each crop and contrast in exact-head preview |
| feature-* | Existing individual descriptions in photos.ts | Preserve principal subject; compare native aspect and 3:2 delivery |

Do not set a new crop position from the alt description alone. Inspect the actual asset and rendered frame before choosing a focal point. Keep image alt text accurate; decorative photos must not repeat nearby text. Do not upscale curated originals or modify the raw archive.

## Evidence and remaining work

- Inspect all actual photo placements and source files, the optimizer output dimensions and responsive CSS; record specific crop/focal decisions per key and both locales.
- Verify full-page and section previews at the five standard widths; note any source mismatch or missing image and confirm hero overlay contrast and fetch priority. Use same-environment Lighthouse comparison.
- Record exact head, CI run, screenshot artifact and actual human review result. No claims of visual acceptance until images have been inspected.
- No source photos, public copy, CSS or optimizer changes in this initial commit. No new dependencies or production changes.

## Verified optimizer contract

Source: `scripts/optimize-photos.py` blob `9ce9cc376f87715158a12334faadb7ee0f96fb43`. The candidate width arrays match PagePhoto for all three families. Feature photos are cropped to 3:2 before encoding; the named FOCUS_Y overrides are all 0.50 and the default is 0.50. Hero images preserve source aspect in the optimizer. Global CSS uses a 1408:768 hero frame, a 3:2 feature frame, and 4:3 at <=640px. The CSS therefore performs a second crop on some mobile images. Verify actual subject retention before changing focal position. AVIF quality 65 and WebP quality 84 are existing settings; curated sources also receive progressive JPEG quality 90. The optimizer reads the existing raw JPG/PNG files and the owner-supplied ZIP; it skips superseded curated keys and writes only derived files. Actual rendered sizes, visual crops and Lighthouse comparisons still require evidence.
