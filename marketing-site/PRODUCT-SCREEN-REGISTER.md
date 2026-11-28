# MQ-08 — Product screen register

Initial source inventory, 28 November 2026. Source: src/data/productScreens.ts blob c13364b13ac3da8f8a3d6fe8bcba8d728eb29009 and src/components/ProductScreens.astro blob bfeca9dc962d7315dd5fddc27361d58888238ecf. The component uses the key to construct public/images/product/<key>.webp. Both EN and TR use the same image; the data file is the source of truth for bilingual captions and alt/title text. The owner prefers English images for both locales; actual image language, original file, capture commit/date, platform and rights/approval must be verified individually before acceptance. The existing bd caption mentions iPad, but this is not independent capture provenance.

| Asset key | Screen EN / TR | Declared dimensions | Placement keys (both locales where used) |
|---|---|---|---|
| ed | Recipe editor / Tarif editörü | 1209×842 | home; features/recipe-design |
| bd | Brew day / Demleme günü | 1209×842 | home; how-it-works; homebrewers; features/brew-day |
| fe | Fermentation overview / Fermantasyon özeti | 1209×842 | how-it-works; features/fermentation-monitoring |
| bs | Batch summary / Parti özeti | 1209×929 | home; how-it-works; features/production-management |
| pr | Production board / Üretim panosu | 1209×875 | breweries; modules; compare; features/production-management |
| tk | Tank detail / Tank detayı | 1716×924 | breweries; features/production-management |
| qa | Quality record / Kalite kaydı | 1716×1017 | breweries; modules; features/quality-laboratory |
| inv | Inventory by lot / Lot bazında stok | 1209×902 | modules; features/inventory-costing |
| lt | Lot traceability / Lot izlenebilirliği | 1716×615 | features/inventory-costing |
| wiz | Recipe wizard / Tarif sihirbazı | 1209×842 | homebrewers; compare |
| cmp | Recipe versions / Tarif sürümleri | 1716×677 | homebrewers; features/recipe-design |
| cal | Brewing calendar / Üretim takvimi | 1209×764 | how-it-works; features/planning-scheduling |
| tools | Measurement tool / Ölçüm aracı | 1209×791 | technical-principles; features/brewing-calculations |
| hw | Value explanation / Değer açıklaması | 1209×842 | technical-principles |
| cip | Cleaning and packaging record / Temizlik ve dolum kaydı | 1209×792 | features/packaging-compliance |
| cl | Equipment calibration / Ekipman kalibrasyonu | 1209×1137 | features/equipment-calibration |

## Presentation and refresh
- Each figure links to the full-size WebP and uses the localized title as alt text and a localized caption from the data file.
- Desktop uses full width, natural aspect ratio. At <=600px the component uses a centered 135% image with -17.5% start margin inside a clipped frame. This is intentional CSS, but whether the crop removes meaningful UI needs visual review; MQ-11a recorded repeated image-bound observations.
- For a replacement, verify original capture provenance, language, real feature and platform; retain the key if the screen meaning is unchanged, update dimensions and bilingual text when necessary, and rerun all placements in both languages. Do not use screenshots alone to assert feature availability.
- TODO: verify actual asset files and original provenance per key, resolve G8 claims separately, inspect full-frame vs crop previews, add exact-head evidence and record approval. The table is a code-derived inventory, not a completed visual inspection.

