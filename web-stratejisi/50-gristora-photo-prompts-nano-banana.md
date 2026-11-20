# GRISTORA — Nano Banana fotoğraf promptları

**Durum:** Taslak (20 Kasım 2026). Sahip kararı: sitede fotoğrafsız sayfa kalmayacak; fotoğraflar Google Nano Banana (Gemini) ile üretilecek. LCH-07 bu yönde güncellenir: yapay zekâ ile üretilmiş, temsili görseller; marka, etiket, okunur yazı ve sahte ürün ekranı yok.

## Kullanım

1. Önce **Ortak stil** bloğunu, ardından sayfanın promptunu tek mesaj olarak yapıştırın.
2. Aspect ratio menüsünden tablodaki oranı seçin (hero: 16:9, diğerleri: 3:2).
3. Her sayfa için 2–4 sonuç üretin, en gerçekçisini seçin. Vana, boru, kapak, el ve parmaklara yakından bakın; yanlış ayrıntı varsa yeniden üretin.
4. Dosyayı tablodaki adla kaydedin (PNG, en az 2400 px genişlik). Siteye yerleştirirken AVIF/WebP'ye çeviririm.
5. Türkçe sayfalar (`/tr/...`) aynı fotoğrafları kullanır; ayrıca üretmeyin.

## Ortak stil (her promptun başına ekleyin)

```
Photorealistic editorial photograph for a premium brewing software website. Shot on a full-frame camera, 35mm lens, natural soft light, shallow depth of field, realistic textures, slight film grain. Color grade: deep forest green shadows (#0B2F26), warm amber highlights (#F4A261), stone beige neutrals (#F7F6F3). Calm, precise, professional mood. No text, no letters, no numbers, no logos, no brand names, no labels on bottles or tanks, no screens or user interfaces, no watermarks. Equipment must look technically correct and clean. People, if present, are seen from behind, from the side or as hands only; no direct eye contact with the camera.
```

## Ana sayfalar

| # | Sayfa | Dosya | Oran | Prompt |
|---|---|---|---|---|
| 1 | Ana sayfa (`/`) | `home-hero.png` | 16:9 | A modern craft brewery cellar at dawn: a row of polished stainless steel conical fermenters with valves, sample ports and glycol jackets. Warm amber light comes through a high window, the foreground is in soft forest-green shadow. Leave calm empty space on the left third for a headline. |
| 2 | Breweries (`/breweries`) | `breweries-hero.png` | 16:9 | A professional brewhouse floor: a three-vessel brewhouse (mash tun, lauter tun, kettle) in brushed stainless steel and copper accents, clean tiled floor, hoses neatly coiled. A brewer seen from behind checks a valve. Empty, darker area on the left for text. |
| 3 | Homebrewers (`/homebrewers`) | `homebrewers-hero.png` | 16:9 | A home kitchen brewing setup in a warm evening light: a 30-litre stainless kettle on an induction plate, a glass carboy fermenting amber beer with an airlock, a bowl of crushed malt and fresh green hops on a wooden counter. Cozy but tidy. No commercial tanks. Empty space on the left. |
| 4 | Features (`/features`) | `features-hero.png` | 16:9 | Overhead flat-lay on a dark slate surface: malted barley, whole hop cones, a yeast vial without label, a glass hydrometer, a refractometer, a notebook with blank pages and a pencil. Precise, symmetrical, editorial. |
| 5 | How it works (`/how-it-works`) | `how-it-works-hero.png` | 16:9 | A brewer's hands pour wort sample from a fermenter sample valve into a small glass cylinder, a hydrometer floating in it. Close-up, side light, stainless steel background softly blurred. |
| 6 | Compare (`/compare`) | `compare-hero.png` | 16:9 | Two tulip glasses of beer side by side on a wooden bar, one golden pale ale and one deep amber ale, condensation drops, soft backlight. Clean, balanced composition suggesting comparison. |
| 7 | Pricing (`/pricing`) | `pricing-hero.png` | 16:9 | A small taproom-brewery seen across a bar at golden hour: a short row of fermenters behind glass, wooden stools, warm amber light. Calm, inviting, uncluttered. |
| 8 | FAQ (`/faq`) | `faq-hero.png` | 16:9 | A quiet brewery office corner: a wooden desk with a coffee cup, a closed notebook, a tasting glass of amber beer, and a window that shows fermenters outside. Soft morning light. No computer screen visible. |
| 9 | Get started (`/get-started`) | `get-started-hero.png` | 16:9 | Hands sprinkle dry yeast into fresh wort in an open stainless fermenter, the moment a new batch begins. Warm light, gentle steam. |
| 10 | Technical principles (`/technical-principles`) | `technical-principles-hero.png` | 16:9 | A precise brewing lab bench: a calibrated digital scale, glass beakers, a pH meter probe in a sample, a pipette, all on a white stone counter. Cool clean light with a warm amber accent. Instrument displays are blank or out of focus. |
| 11 | 404 | `404.png` | 3:2 | An empty, clean stainless steel fermenter seen from the open top hatch, looking inside. Minimal, slightly playful, soft light. |

## Özellik sayfaları (`/features/<slug>`)

| # | Slug | Dosya | Prompt |
|---|---|---|---|
| 12 | `recipe-design` | `feature-recipe-design.png` | Small glass bowls of different malts (pale, crystal, chocolate) and hop pellets arranged in a row on a wooden board, a blank recipe notebook beside them. |
| 13 | `brewing-calculations` | `feature-brewing-calculations.png` | A brewer measures wort gravity with a refractometer held up to the light, close-up of hands and the instrument, kettle softly blurred behind. |
| 14 | `water-mash-yeast` | `feature-water-mash-yeast.png` | Thick porridge-like mash being stirred with a stainless paddle in a mash tun, steam rising, a thermometer probe in the mash. |
| 15 | `brew-day` | `feature-brew-day.png` | Rolling boil in a stainless kettle, a hand adds green hop pellets from a small scoop, dense steam lit by warm light. |
| 16 | `fermentation-monitoring` | `feature-fermentation-monitoring.png` | Close-up of an active fermentation through a glass fermenter: krausen foam, rising bubbles, an airlock with a bubble. |
| 17 | `equipment-calibration` | `feature-equipment-calibration.png` | A technician's hands check an analog pressure gauge and a temperature sensor on a stainless tank, a small wrench nearby. Gauge face without readable numbers. |
| 18 | `planning-scheduling` | `feature-planning-scheduling.png` | A wide view of a brewery cellar with fermenters in different states: some with frost on the cooling jackets, some idle and clean. Orderly, calm, sense of a planned sequence. |
| 19 | `inventory-costing` | `feature-inventory-costing.png` | A tidy brewery storeroom: stacked unlabeled malt sacks on pallets, sealed hop boxes on shelves, a pallet jack. Even, soft light. |
| 20 | `production-management` | `feature-production-management.png` | A brewer seen from behind walks along a row of tall fermenters on a catwalk, wide angle, sense of scale and order. |
| 21 | `quality-laboratory` | `feature-quality-laboratory.png` | A small QC lab: a microscope, petri dishes, a row of beer samples in glass tubes on a rack, white counter, soft clean light. |
| 22 | `packaging-compliance` | `feature-packaging-compliance.png` | Unlabeled aluminium cans moving on a small canning line, slight motion blur, stainless machinery. No text on cans. |
| 23 | `team-notifications` | `feature-team-notifications.png` | Two brewers in work aprons talk beside a fermenter, seen from the side, one points at a valve. Natural, candid, faces not in focus. |
| 24 | `connected-devices` | `feature-connected-devices.png` | A wireless temperature probe and a small sensor clamped on a stainless fermenter, cable-free, close-up with shallow depth of field. No displays. |
| 25 | `imports-exports-tools` | `feature-imports-exports-tools.png` | A wooden shelf with old handwritten brew logbooks and recipe cards next to a closed laptop, warm light, sense of moving from paper records to software. |
| 26 | `platform-security-data` | `feature-platform-security-data.png` | A locked stainless steel cabinet door in a clean brewery, a key in the lock, cool light with an amber reflection. Calm and secure mood. |
| 27 | `analytics-reports` | `feature-analytics-reports.png` | Five tasting glasses of the same amber beer in a row, from slightly lighter to slightly darker, on a white surface, suggesting batch-to-batch comparison. |

Özellik sayfalarının hepsi 3:2 orandadır.

## Kontrol listesi (seçerken)

- Tank ve vanalar gerçekçi mi? Bağlantısı olmayan boru, eriyen metal ya da garip kapak var mı?
- Eller ve parmaklar doğru mu?
- Görselde harf, rakam, logo ya da etiket kalmış mı? Varsa yeniden üretin.
- Homebrewers sayfasında ticari tank yok.
- Renk tonları sitedeki orman yeşili ve amber paletiyle uyumlu.

## Açık konular

- Nano Banana çıktılarında görünmez SynthID filigranı vardır; silinmez ve silinmemelidir.
- Gemini kullanım koşullarında ticari kullanım hakkı `to be verified` (sahip kontrol eder).
- Sitede "temsili görsel" notu gösterilip gösterilmeyeceği sahip kararıdır.
