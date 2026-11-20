# GRISTORA marketing-site photo assets

**Scope:** marketing website only. Raw originals are retained in `../fotograflar-ham/`; generated delivery assets are written to `public/images/photos/`.

## Curated feature-photo refresh

The owner-supplied `../fotograflar-ham/brewing-editorial-photos.zip` is the source of truth for the following 12 feature images. The archive contains 896 × 1200 px JPEG originals. The build maps them to stable website keys and replaces the previous feature-photo derivatives without changing the page copy or product claims.

| Archive file | Website photo key | Use |
|---|---|---|
| `active-fermentation-carboy.jpg` | `feature-fermentation-monitoring` | Fermentation monitoring |
| `Feature-equipment-calibration.jpg` | `feature-equipment-calibration` | Equipment calibration |
| `feature-planning-scheduling.jpg` | `feature-planning-scheduling` | Planning and scheduling |
| `feature-inventory-costing.jpg` | `feature-inventory-costing` | Inventory and costing |
| `feature-production-management.jpg` | `feature-production-management` | Production management |
| `feature-quality-laboratory.jpg` | `feature-quality-laboratory` | Quality laboratory |
| `feature-packaging-compliance.jpg` | `feature-packaging-compliance` | Packaging and compliance |
| `feature-team-notifications.jpg` | `feature-team-notifications` | Team notifications |
| `feature-connected-devices.jpg` | `feature-connected-devices` | Connected devices |
| `Feature-imports-exports-tools.jpg` | `feature-imports-exports-tools` | Imports and exports |
| `Feature-platform-security-data.jpg` | `feature-platform-security-data` | Platform security |
| `feature-analytics-reports.jpg` | `feature-analytics-reports` | Analytics and reports |

## Delivery formats and dimensions

- AVIF is served first for efficient modern-browser delivery.
- WebP is the second choice.
- JPEG is the final fallback for the curated photos; the original ZIP remains unchanged.
- Each curated image is cropped to the existing 3:2 feature-photo composition and emitted at 640 px and its native 896 px width. The optimizer does not upscale the source.
- The curated photo figure is capped at 896 px on desktop so the site does not enlarge a smaller source beyond its native width.
- The existing 640/960/1152 feature variants and 640/960/1408 page-hero variants remain in place for all other photos.

The existing `<picture>` implementation therefore keeps AVIF/WebP delivery while adding a universally readable JPEG fallback. AVIF/WebP are supported by current major browsers; the JPEG fallback covers older clients and constrained image decoders. See [MDN's image format guide](https://developer.mozilla.org/en-US/docs/Web/Media/Guides/Formats/Image_types).

## Build

`npm run build` runs `npm run optimize:photos` before Astro builds the static site. The optimizer reads the raw photo directory and the curated ZIP, then writes the generated AVIF, WebP and JPEG derivatives. GitHub Actions and Vercel therefore build the same assets from the same committed source archive.

Run `npm run optimize:photos` before `npm run dev` when previewing changed source photos locally.

## Guardrails

- Do not overwrite or recompress the ZIP originals.
- Do not change page copy, claims, product functionality or unrelated visuals as part of photo work.
- Keep alt text in `src/data/photos.ts` accurate in both English and Turkish.
- Confirm visual crop, focus and mobile presentation in the built site before accepting the change.
- The images are editorial/representative photography, not screenshots or evidence of released product functionality.
