# MQ-08 — Product screen register

Initial source inventory, 28 November 2026. Source: src/data/productScreens.ts blob c13364b13ac3da8f8a3d6fe8bcba8d728eb29009 and src/components/ProductScreens.astro blob bfeca9dc962d7315dd5fddc27361d58888238ecf. The component uses the key to construct public/images/product/<key>.webp. Both EN and TR use the same image; the data file is the source of truth for bilingual captions and alt/title text. Owner decision recorded in #465 on 28 November 2026: keep all 16 Turkish interface concept images in both locales; label each concept preview. Original file, capture commit/date, platform and rights remain unverified. The unverified iPad reference is removed.

| Asset key | Screen EN / TR | Declared dimensions | Placement keys (both locales where used) | UI language | Status | Old full-frame scale at 390 px |
|---|---|---|---|---|---|---|
| ed | Recipe editor / Tarif editörü | 1209×842 | home; features/recipe-design | TR | concept (per #354) | 26.6% |
| bd | Brew day / Demleme günü | 1209×842 | home; how-it-works; homebrewers; features/brew-day | TR | concept (per #354) | 26.6% |
| fe | Fermentation overview / Fermantasyon özeti | 1209×842 | how-it-works; features/fermentation-monitoring | TR | concept (per #354) | 26.6% |
| bs | Batch summary / Parti özeti | 1209×929 | home; how-it-works; features/production-management | TR | concept (per #354) | 26.6% |
| pr | Production board / Üretim panosu | 1209×875 | breweries; modules; compare; features/production-management | TR | concept (per #354) | 26.6% |
| tk | Tank detail / Tank detayı | 1716×924 | breweries; features/production-management | TR | concept (per #354) | 18.8% |
| qa | Quality record / Kalite kaydı | 1716×1017 | breweries; modules; features/quality-laboratory | TR | concept (per #354) | 18.8% |
| inv | Inventory by lot / Lot bazında stok | 1209×902 | modules; features/inventory-costing | TR | concept (per #354) | 26.6% |
| lt | Lot traceability / Lot izlenebilirliği | 1716×615 | features/inventory-costing | TR | concept (per #354) | 18.8% |
| wiz | Recipe wizard / Tarif sihirbazı | 1209×842 | homebrewers; compare | TR | concept (per #354) | 26.6% |
| cmp | Recipe versions / Tarif sürümleri | 1716×677 | homebrewers; features/recipe-design | TR | concept (per #354) | 18.8% |
| cal | Brewing calendar / Üretim takvimi | 1209×764 | how-it-works; features/planning-scheduling | TR | concept (per #354) | 26.6% |
| tools | Measurement tool / Ölçüm aracı | 1209×791 | technical-principles; features/brewing-calculations | TR | concept (per #354) | 26.6% |
| hw | Value explanation / Değer açıklaması | 1209×842 | technical-principles | TR | concept (per #354) | 26.6% |
| cip | Cleaning and packaging record / Temizlik ve dolum kaydı | 1209×792 | features/packaging-compliance | TR | concept (per #354) | 26.6% |
| cl | Equipment calibration / Ekipman kalibrasyonu | 1209×1137 | features/equipment-calibration | TR | concept (per #354) | 26.6% |

## Presentation and refresh
- Each figure links to the full-size WebP and uses the localized title as alt text and a localized caption from the data file.
- Desktop and mobile now use full width and natural aspect ratio (commit 6c60729). Previously, at <=600px, a centered 135% image was clipped. The full-size link remains for detail. Check the new preview for legibility and confirm the image-bound reports with exact-head CI.
- For a replacement, verify original capture provenance, language, real feature and platform; retain the key if the screen meaning is unchanged, update dimensions and bilingual text when necessary, and rerun all placements in both languages. Do not use screenshots alone to assert feature availability.
- TODO: verify original capture provenance per key, resolve G8 claims separately, inspect full-frame vs crop previews, add exact-head evidence and record approval. The table is a code-derived inventory, not a completed visual inspection.


## Verified repository asset provenance

The repository directory lists all 16 WebP assets. The directory history records introduction in commit `8230b7aa1d2f8998ceed77e6ab950b6861343236` (2026-11-26, W353 / PR #354). This is the repository import date, not the image capture date. The blob IDs identify the current stored files, not the original capture or feature availability.

| File under public/images/product/ | Git blob SHA |
|---|---|
| bd.webp | `21b59b4f461dfb28ec770d803bd4910c1cc8110d` |
| bs.webp | `c2ff89c39c33771bfd19ac26698b23dcddb9e614` |
| cal.webp | `33797812169dda04740845cb7baebdc857b38af7` |
| cip.webp | `965a7d08947fbc2699dc650884fc219d3d61cd7a` |
| cl.webp | `1b5ed65fdb1247f47c78107103df69a157d24346` |
| cmp.webp | `e05b052ba78a69fd38a6da102c5501fe282d8ef9` |
| ed.webp | `dc62f5b0a89f41825e18c651642d9d5c181f2577` |
| fe.webp | `ec6690693611a6b0a051cb57187dd3133026e41c` |
| hw.webp | `665d23b26e19e004f4dd81506a2fd9dc4e4db8d3` |
| inv.webp | `5741dba0adc1ba2b2e0673ff1260910c32d11203` |
| lt.webp | `8c630872d61f07636c9bde31640d1283df32ddd2` |
| pr.webp | `f05a371fad15cd4165ad665a6947cf905212464f` |
| qa.webp | `46d612e918b82f5c9c1cf1eae0d6508e46f7a565` |
| tk.webp | `2a614ef6ce53d8a86742b5f433bbd65ba1618ebe` |
| tools.webp | `2a682cd626bb04333029bc316afddd7846c9817f` |
| wiz.webp | `933782c92da3c0c10ea23ec676daad400ea39cb7` |

Platform, original capture path/date and feature evidence remain unverified. Claude reviewed all 16 images and confirmed Turkish UI. Do not infer them from dimensions.

## Round 1 visual review and correction

Claude review on PR #468, head 33f2081: all 16 source dimensions and 17 page placements in both languages verified. At 390 px the old full-frame width is 322 px; the wide screens tk, qa and lt are not legible at this scale. The previous 135% crop showed about 435 px but clipped content. The new mobile presentation uses one horizontally scrollable 860 px detail view for all four 1716 px assets (tk, qa, lt, cmp), about 50% scale, with a visible hint and keyboard focus. Full-size link remains. Verify actual legibility, scroll containment, keyboard access and page overflow in current-head CI and screenshots; not yet accepted. The source files inv, fe and hw are cut at the bottom (Claude review); the component labels this limitation. Replacement source images are required to restore missing pixels; do not fabricate them. Other mobile legibility not checked. Capture before/after PNGs for home, inventory-costing and breweries, EN/TR, 390 and 1440. 
