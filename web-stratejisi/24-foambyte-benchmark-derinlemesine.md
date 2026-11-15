# Demleme — FoamByte Derin Benchmark ve Sayfa İyileştirme Kararları

**Work package:** WP-13  
**Tarih:** 15 Kasım 2026  
**Durum:** Benchmark ve ilk uygulama dalgaları tamamlandı; statik QA otomasyonu çalışıyor; gerçek tarayıcı QA ve Vercel Preview bekliyor.  
**Kapsam:** Demleme pazarlama sitesi ile FoamByte kamuya açık pazarlama sayfaları. Ürün uygulaması, API, iOS ve ürün geliştirme kapsam dışıdır.  
**Karar zemini:** Homebrewers + Professional Breweries; teknik konumlandırma; ABD/İngiltere önceliği; İngilizce; self-service öncelikli.

## 1. Yönetici kararı

FoamByte bugün pazarlama sitesinde daha tamamlanmış bir ticari edinim deneyimi sunuyor: gerçek ürün ekranları, ayrı özellik açıklamaları, ücretsiz hesaplayıcı dizini, fiyat/plan bilgisi, FAQ ve müşteri kanıtları. Ana sayfada iki hedef kitleyi birlikte ele alıyor; ancak Home ve nano brewery için farklı, derinleştirilmiş yolculuklar sınırlı.

Demleme'nin doğru stratejik avantajı FoamByte'ın daha fazla özelliğini taklit etmek değil. Demleme:
1. Home ve Business için gerçekten farklı bilgi mimarisi ve kullanıcı yolculuğu kurmalı.
2. Teknik iddiaları yöntem, girdi, çıktı, varsayım, sınır ve kanıtla açıklamalı.
3. Ürün olgunluğunu her sayfada tutarlı biçimde göstermeli.
4. Business tarafında profesyonel üretim kayıtları ve izlenebilirlik odağını ERP kapsamından açıkça ayırmalı.
5. Ürün hazır değilken sahte kayıt, trial, demo, fiyat, müşteri veya hesaplayıcı vaat etmemeli.

**Sonuç:** Demleme'nin mevcut yönü marka ve teknik ton açısından doğru; fakat mevcut Home ve Business sayfaları hâlâ kısa tanıtım/roadmap sayfaları. İçerik derinliği, süreç anlatımı ve kanıt katmanları eklenmeden FoamByte karşısında ikna edici bir teknik bilgi merkezi haline gelmez.

## 2. Denetim yöntemi ve sınırlar

- Demleme: 15 Kasım 2026 tarihinde GitHub main dalındaki Astro sayfaları, feature data, CSS ve WP-04–WP-12 strateji dokümanları incelendi. PR #40'ta hazırlanan claim-safe katalog değişiklikleri, GitHub Actions Astro check/build başarılı olduktan sonra PR #43 üzerinden main'e manuel squash-merge edildi.
- FoamByte: homepage, features, calculators ve FAQs kamuya açık sayfaları incelendi.
- Vercel canlı Demleme adresi bu incelemede tarayıcı aracıyla açılamadı. Bu nedenle gerçek render, piksel karşılaştırması, responsive davranış, hız, etkileşim ve erişilebilirlik sonucu verilmedi.
- Puanlar, sayfa içeriği ve gözlenebilen bilgi mimarisi üzerinden verilen göreli editoryal değerlendirmedir; kullanıcı testi veya laboratuvar ölçümü değildir.

Kaynaklar:
- https://foambyte.com/
- https://foambyte.com/features/
- https://foambyte.com/calculators/
- https://foambyte.com/faqs/
- Demleme kaynakları: marketing-site/src/pages/index.astro, homebrewers.astro, breweries.astro, src/data/features.ts, src/styles/global.css
- Strateji: WP-04, WP-05, WP-06, WP-07, WP-08, WP-09, WP-10, WP-11, WP-12

## 3. Genel karşılaştırma

| Boyut | FoamByte — gözlenen durum | Demleme — main kaynak durumu | Karar |
|---|---|---|---|
| İlk ekran / değer önerisi | Ürünü ve ana faydayı doğrudan anlatıyor; Start Free CTA var | Güçlü editorial ton; ürünün geliştirme aşamasında olduğu açık | Demleme dürüstlük avantajını koruyup kullanıcı problemini daha somut anlatmalı |
| Ürün somutluğu | Recipe Studio, Batch Tracking, Fermentation, Water Chemistry gibi modüller; ekran görselleri | 16 feature alanı; gerçek ürün ekranı yok; roadmap kartları | Sahte UI ekleme; süreç diyagramı, örnek kayıt şeması ve teknik açıklama kullan |
| Özellik anlatımı | Her ana özellik için ayrı sayfa ve örnek görseller | Katalog ve detay route'ları var; anlatı çoğunlukla roadmap açıklaması | Her detay sayfası problem → workflow → inputs/outputs → stage → limits → evidence standardına geçmeli |
| Teknik bilgi | Hesaplayıcı açıklamaları formül/yöntem, birim ve kullanım amacına değiniyor | Henüz yayınlanmış teknik bilgi merkezi yok | Doğrulanmış teknik rehberler Demleme'nin ana farklılaştırıcısı olmalı |
| Hesaplayıcı edinimi | Kategorili, aranabilir, çok sayıda ücretsiz hesaplayıcı | Public calculator route yok; WP-09 doğrulaması bekliyor | Doğrulama tamamlanmadan yayınlama; sonrasında tekil, kaynaklı araç sayfaları |
| Audience ayrımı | Homepage iki gruba hitap ediyor; ortak feature/FAQ/fiyat akışı | Home ve Business için ayrı route ve farklı giriş metni | Demleme'nin yapısal avantajı var; içerik ve örnekler daha fazla ayrışmalı |
| Sosyal kanıt | İsimli yorumlar ve işletme logoları | Onaylı testimonial/logo/metric yok | Kanıt gelene kadar boş sosyal kanıt alanı açma |
| Ticari şeffaflık | Free/Plus planı, limitler, billing ve ödeme sağlayıcısı anlatılıyor | Fiyat, trial ve paket kararı yok | Ticari karar verilmeden pricing sayfası üretme |
| FAQ | Kullanım, kapsam, limit, cihaz ve billing sorularını yanıtlıyor | Ayrı FAQ sayfası henüz yok | Ürün durumu, kapsam dışı alanlar ve veri/cihaz sorularını cevaplayan FAQ hazırla |
| Görsel dil | Gerçek ürün ekranları + bira üretim bağlamı | Editorial/technical CSS, özgün SVG ve görsel slotları | Gerçek ürün ekranı yokken teknik diyagramlar ve fotoğraf slotları; piksel QA ayrıca |
| Navigasyon | Product, Resources, Calculators, FAQs ve Get Started | Product, Homebrewers, Breweries, Our approach | Learn ve doğrulanmış Calculators yayınlandığında ana navigasyona ekle |
| Olgunluk yönetimi | Available ve Coming Soon ayrımı bazı özelliklerde görünür | In development / planned / under consideration ayrımı daha katı olmalı | Her claim için tek sözlük ve aynı görsel dil |
| Dönüşüm | Çalışan ürün hesabına ve fiyat planına yönlendiriyor | Şimdilik keşif ve audience yönlendirme CTA'ları | Gerçek ürün onboarding'i yokken mevcut CTA yaklaşımı doğru |

## 4. Ayrı profil karşılaştırmaları

### 4.1 Homebrewers

| Boyut | FoamByte | Demleme mevcut | Demleme için hedef |
|---|---|---|---|
| Kullanıcı niyeti | Reçete oluşturmak, hesaplamak ve batch geçmişini takip etmek | Teknik detaylara önem veren homebrewer için genel proje anlatısı | Somut iş sırası: recipe → ingredients/water → calculations → brew day → measurements → batch review |
| Açılış mesajı | Basitlik ve daha iyi bira üretme sonucu | “Make every brew a learning experience” | Öğrenme vaadini koru; hangi kararların öğrenildiğini örnekle |
| Özellik keşfi | Recipe Studio, Water Chemistry, Profiles ve Batch Tracking | Home'a filtrelenmiş roadmap kartları | Kartları süreç sırasına koy; her birinde bugün/plan/araştırma durumunu göster |
| Teknik güven | Ürün ekranı, BJCP referansı ve hesaplayıcılar | Teknik ton var; yayınlanmış yöntem/örnek henüz yok | OG/FG, gravity, volume, water ve unit konularında kaynaklı rehberler |
| İçerik fırsatı | Araç odaklı SEO | Teknik öğrenme merkezi henüz yok | “Why the number changes” yaklaşımı: yöntem, input, varsayım, örnek ve hata sınırı |
| CTA | Start Free | Explore homebrewing / Explore planned capabilities | Ürün erişimi açılana kadar öğrenme ve kapsamı inceleme CTA'ları |

**Home sayfası için kritik düzeltmeler**
- “A complete toolkit for your next brew” ifadesi, ürünün tamamlanmış olduğu izlenimini verebilir. “A connected approach to planning, measuring and learning from each brew” gibi kapsamı aşmayan bir anlatı tercih edilmeli.
- “selected advanced features available to enable” ifadesi mevcut ürün yeteneği izlenimi yaratabilir. Ürün planında kanıtlanmadıkça kaldırılmalı veya açıkça gelecekteki tasarım ilkesi olarak yazılmalı.
- Feature kartları alfabetik/katalog mantığı yerine gerçek homebrewing iş akışına göre gruplanmalı.
- Bir hesaplayıcı yayına alınmadan önce formül sahibi, kaynak, test vektörleri, birim ve tolerans kaydı olmalı.

### 4.2 Professional Breweries

| Boyut | FoamByte | Demleme mevcut | Demleme için hedef |
|---|---|---|---|
| Kullanıcı niyeti | Nano brewery ve küçük üretici için recipe, batch, tank ve üretim bilgisi | Profesyonel ekipler için teknik yazılım yaklaşımı | Head brewer, production operator ve quality rollerinin ayrı ihtiyaçlarını anlat |
| Açılış mesajı | Basit, uygun fiyatlı ve organize üretim | “Consistency is built batch by batch” | Tutarlılık mesajını ölçülebilir kayıt ve review disipliniyle açıkla |
| Workflow | Recipe → batch → fermentation / production | Recipe standardisation, production records, tank, traceability, quality roadmap | Recipe review → production plan → actual records → fermentation observations → QC/traceability (stage labelled) |
| Ürün kanıtı | Batch ekranları ve görseller | Ürün ekranı yok | Uydurma ekran yerine kayıt ilişkisi ve süreç diyagramı |
| Audience derinliği | Home ve nano brewery aynı geniş sayfa anlatısında | Ayrı Business route | Demleme burada ayrışabilir: rol, kayıt, lot/batch ilişkisi ve operasyonel sorumluluk |
| Scope | Üretim yönetimi alanları genişliyor | ERP olmadığı açıkça belirtiliyor | ERP dışı alanlar görünür ve açık kalmalı |

**Business sayfası için kritik düzeltmeler**
- “From formulation to production control” ifadesi mevcut kapsamdan daha ileri bir kontrol/otomasyon algısı yaratabilir. “From recipe review to production records” daha güvenli.
- “tank workflows, traceability, quality and team operations” başlıkları roadmap durumlarıyla tek tek etiketlenmeli; hepsi mevcut kabiliyet gibi sunulmamalı.
- ERP sınırı yalnız dipnot olmamalı. Sayfada ayrı, görünür bir “What Demleme is — and is not” bölümü olmalı.
- Gerçek örnek, müşteri hikâyesi veya uyum iddiası yoksa bunları taklit eden görsel/istatistik eklenmemeli.

## 5. Bilgi akışı: önerilen sayfa anlatısı

### Homepage
1. Kısa teknik konumlandırma ve ürünün geliştirme aşaması.
2. İki ayrı audience seçimi; iki yolun neyi farklı ele aldığı.
3. Brewing lifecycle — yalnız onaylı kapsamı göster.
4. Technical principles: kaynak, hesaplama, ölçüm ve sınırlama yaklaşımı.
5. Home ve Business için ayrı değer önerisi kartları.
6. Gerçekten yayınlanmış içeriklere bağlantı.
7. Net ürün olgunluğu ve güvenli sonraki adım.

### Homebrewers
1. Kime yönelik ve hangi ev üretimi kararlarını desteklemeyi hedefliyor?
2. Recipe → water/ingredients → calculations → brew day → fermentation records → review.
3. Her adım için bir örnek soru ve ilgili feature.
4. Teknik rehber / doğrulanmış calculator bağlantısı (yalnız hazırsa).
5. Available / In development / Planned / Under consideration ayrımı.
6. Home FAQ ve gerçek sonraki adım.

### Professional Breweries
1. Hangi büyüklükte/iş yapışında profesyonel üretim ekibine hitap ediyor?
2. Recipe review → production planning → batch records → fermentation/vessel observations → QC/traceability.
3. Brewer, operator ve quality rollerinin bilgi ihtiyaçları.
4. Kayıt, lot ve batch ilişkisini gösteren basit şema.
5. Açık ERP sınırı ve desteklenmeyen alanlar.
6. Business FAQ ve gerçek sonraki adım.

### Feature detail template
Her sayfa aynı zorunlu sırayı izlemeli:
1. User problem.
2. Intended workflow.
3. Inputs and outputs / records.
4. Product stage and dependencies.
5. What it does not do / limitations.
6. Evidence or method reference.
7. Related audience workflow and technical resources.

## 6. Tasarım ve görsel sistem kararı

Kaynak koddan görülen Demleme yönü editorial + technical: serif başlıklar, sade sans gövde, teknik mono etiketler, kontrollü amber/olive tonlar, geniş boşluk ve SVG illüstrasyon. Bu yaklaşım korunmalı.

FoamByte'ın görsel avantajı yalnız estetik değil, **ürünün nasıl çalıştığını gösteren gerçek ekran görüntüleri** kullanmasıdır. Demleme'de henüz gerçek ürün ekranı olmadığı için dashboard mockup veya gerçekmiş gibi görünen sahte UI kullanılmamalı.

Önerilen görsel dil:
- Home: gerçek homebrew tezgâhı, ölçüm, malt/hop ve kettle bağlamı; görselin yanında kısa teknik annotation.
- Business: profesyonel brewhouse, ekip, vessel ve batch-record akışı; insan/operasyon ölçeği.
- Technical Principles: basit, etiketli süreç ve veri akış diyagramları.
- Calculator/technical article: birim, formül girdisi ve örnek çıktı görselleştirmesi; yalnız doğrulama sonrası.
- Her görselin amacı belirli bir bilgiyi açıklamak olmalı; dekoratif stok görsel doldurma yapılmamalı.

**Görsel QA açık:** 1440, 1024, 768, 390 ve 320 px ekranlarında gerçek render görülmeden tipografi, spacing, crop, contrast, menu ve overflow sonucu verilmemeli. Vercel erişimi açıldığında ayrı QA kaydı oluşturulmalı.

## 7. Önceliklendirilmiş iyileştirme backlog'u

| ID | İş | Öncelik | Bağımlılık / kabul ölçütü |
|---|---|---:|---|
| FB-01 | Home sayfasındaki “complete toolkit” ve “available to enable” iddialarını düzelt | P0 | WP-03 claim map ile uyum |
| FB-02 | Business “production control” başlığını kayıt odaklı ve stage-safe hale getir | P0 | WP-03 ve ürün planı |
| FB-03 | Home sayfasına gerçek süreç sıralı bilgi mimarisi ekle | P1 | WP-07 wireframe; her adımda stage |
| FB-04 | Business sayfasına rol + workflow + record model bölümleri ekle | P1 | Ürün planında onaylı kayıt kapsamı |
| FB-05 | İki sayfanın FAQ bloklarını hazırla | P1 | Ürün, privacy ve platform cevapları onaylı olmalı |
| FB-06 | How It Works — Home ve Business sayfalarını üret | P1 | WP-04 / WP-07 |
| FB-07 | Technical Principles sayfası | P1 | Kaynak ve teknik reviewer onayı |
| FB-08 | Feature detail şablonuna evidence / limits / inputs-outputs ekle | P1 | PR #40 claim-safe katalog merge ve WP-03 |
| FB-09 | Learn topic hub ve ilk teknik rehberler | P1 | WP-10 evidence register |
| FB-10 | Calculators hub ve araçlar | HOLD | WP-09 formül doğrulama + public-use approval |
| FB-11 | Orijinal görsel varlıkların eklenmesi | Kısmen tamamlandı | On bir özgün public SVG + inline homepage hero; gerçek fotoğraf ve ürün ekranı ayrı onay bekliyor |
| FB-12 | Otomatik QA ve ilk screenshot incelemesi tamamlandı; kapsamlı manuel review bekliyor | P0 launch gate | 40 Playwright testi başarılı; 23 screenshot üretildi ve ilk tasarım incelemesi yapıldı. Keyboard/focus, screen-reader, contrast, 200% zoom ve Vercel görsel doğrulaması açık |
| FB-13 | Kısmen tamamlandı | Statik route/SEO/accessibility smoke checks ve Playwright browser smoke tests CI'da. Screen-reader, contrast, keyboard focus, zoom ve manuel erişilebilirlik QA ayrıca gerekli |

## 7A. Uygulama durumu — 15 Kasım 2026

| Backlog ID | Durum | Uygulama / karar |
|---|---|---|
| FB-01 | Tamamlandı | Home sayfasındaki “complete toolkit” ve “available to enable” riskli ifadeleri claim-safe içerikle değiştirildi (PR #43). |
| FB-02 | Tamamlandı | Business hero “From recipe review to production records” olarak düzeltildi (PR #43). |
| FB-03 | Tamamlandı | Homebrew record loop tablosu eklendi: recipe/targets → ingredients/water → brew-day actuals → fermentation observations → tasting/review (PR #53). |
| FB-04 | Tamamlandı | Business record model eklendi: recipe version → batch record → ingredient lots → vessel events → quality records; release stage etiketleri var (PR #52). |
| FB-05 | Tamamlandı | Home ve Business audience FAQ'ları ile merkezi `/faq/` sayfası mevcut (PR #47 ve audience sayfaları). |
| FB-06 | Tamamlandı | `/how-it-works/` Home ve Business workflow'larını ayrı anlatıyor (PR #46). |
| FB-07 | Tamamlandı | `/technical-principles/` method, units, planned-vs-actual, measurement context ve model validation ilkelerini anlatıyor (PR #46). |
| FB-08 | Tamamlandı | Feature detail şablonunda inputs/records, outputs, limitations ve evidence status bölümü var; tüm feature sayfaları roadmap olarak işaretli (PR #47/#50). |
| FB-09 | Beklemede / kanıt kapısı | Learn hub ve teknik rehberler, WP-10'da tanımlı birincil kaynak, yöntem, örnek ve subject-matter review tamamlanmadan yayınlanmayacak. |
| FB-10 | HOLD | Calculators yalnız WP-09 formül/test vektörü doğrulaması ve public-use approval sonrasında. |
| FB-11 | Kısmen tamamlandı | Dokuz özgün public SVG ve homepage inline SVG eklendi; fotoğraf ve gerçek ürün ekranı ayrı onay kapısında. |
| FB-12 | Otomatik tarayıcı QA eklendi; manuel görsel QA bekliyor | Playwright Chromium testleri 1440/1024/768/390/320 px için eklendi; screenshot artefact'ları CI'da saklanacak. İnsan görsel incelemesi ayrıca gerekli. |
| FB-13 | Kısmen tamamlandı | Statik route/sitemap/link/metadata/JSON-LD, temel accessibility smoke checks ve Playwright browser smoke testleri CI'a eklendi. Screen-reader, kontrast, focus görünümü, zoom ve manuel keyboard QA ayrıca gerekli. |

GitHub Pages geçici preview olarak yayınlanıyor: https://oguzbay-del.github.io/brewery/. GitHub Actions run #77 ve Pages deployment run #28 başarılı. Bu başarı gerçek tarayıcı görsel QA yerine geçmez. Vercel Preview yeniden kullanılabilir olduğunda ayrı QA kaydı tamamlanmadan public release kapatılmaz.

## 8. Ölçülebilir kabul kriterleri

- Home ve Business sayfalarında aynı metnin yalnız audience adı değiştirilmiş kopyası bulunmayacak.
- Home'da en az 5 farklı homebrewing işi; Business'ta en az 5 farklı profesyonel üretim işi anlatılacak; her biri ürün aşamasıyla etiketlenecek.
- Her feature claim, WP-03 claim kaydına ve ürün planı referansına bağlanacak.
- Her teknik içerikte kaynak, yöntem, birim, varsayım, örnek ve sınırlama bulunacak.
- Business ERP dışı alanları açıkça belirtecek.
- Yayınlanmamış calculator, pricing, trial, signup, testimonial, metric veya customer logo CTA/kanıt olarak kullanılmayacak.
- Görsel QA tüm beş viewport genişliğinde kaydedilecek; 320 px'de yatay taşma olmayacak.
- Build başarısı, canlı görsel QA veya ürün claim onayı yerine sayılmayacak.

## 9. Sıradaki uygulama sırası

1. FB-01 ve FB-02 metin risklerini düzelt.
2. Home ve Business sayfaları için WP-07 wireframe'i bu benchmark kararlarına göre güncelle.
3. Her iki sayfanın içerik taslağını WP-10 evidence register'a bağla.
4. İçerik ve claim review tamamlandıktan sonra ayrı bir uygulama PR'ı aç.
5. Vercel Preview ve tarayıcı erişimi geldiğinde gerçek responsive/design QA yap.
6. Hesaplayıcılar, pricing ve gerçek ürün CTA'larını yalnız ilgili onay kapıları geçildikten sonra ekle.

## 10. Değişiklik ve yayın sınırı

Bu doküman strateji/benchmark çıktısıdır. Kullanıcı 15 Kasım 2026'da marketing-site uygulamasına açıkça devam etme onayı verdi. Ürün app/API değişikliği, calculator launch, fotoğraf satın alma/üretme, analytics ve production deployment bu onayın dışındadır. Vercel Preview ve gerçek tarayıcı QA, Vercel yeniden kullanılabilir olduğunda tamamlanacak release gate olarak kalır.


## 7B. Responsive browser QA automation — 15 Kasım 2026

- Added Playwright Chromium smoke coverage for seven key routes at 1440, 1024, 768, 390 and 320 CSS pixels.
- Checks include HTTP success, one main landmark and H1, image alt text and successful image decoding, browser page errors, and document/body horizontal overflow.
- Added primary navigation interaction checks at all five widths.
- CI uploads full-page screenshots for the homepage and both audience pages at all widths, plus key technical pages at desktop and narrow mobile widths.
- This is automated browser QA, not a substitute for human visual, keyboard, screen-reader, contrast or zoom review.


## 7C. Accessibility automation — 15 Kasım 2026

- Added axe-core Playwright audits for seven routes at 1440 px and 390 px using WCAG 2.0/2.1/2.2 A/AA rule tags.
- Added keyboard checks for skip-link navigation, mobile menu disclosure and FAQ disclosure.
- Added a 720 px narrow-desktop layout proxy; actual 200% browser zoom and manual screen-reader review remain open.
- Automated audit findings are triaged before any accessibility-conformance claim.


## 7D. Axe findings and remediation — 15 Kasım 2026

- First axe run found low-contrast 11px hero-note and 9px footer metadata text (3.96–3.98:1, below 4.5:1) and a non-focusable horizontally scrollable role table.
- Remediation: darker text tokens for these small labels and a labelled/focusable role-table region with visible focus styling.
- Re-run required; no WCAG conformance claim until all automated and manual checks are complete.


- Follow-up axe run isolated the same scrollable-region issue on the Homebrewers record-loop table. Added a labelled focusable region there as well; final rerun required.


## 7E. Accessibility automation results — 15 Kasım 2026

- PR #71 final CI passed 60/60 browser tests: 40 responsive checks, 14 axe scans (seven routes at desktop/mobile), three keyboard interaction checks and three 720px layout proxies.
- Axe findings remediated: hero note and footer metadata contrast now exceed 5:1; Homebrewers and Breweries horizontal tables are labelled/focusable regions with visible focus treatment.
- Automated axe scans are clean for the covered WCAG A/AA tags. This is not a WCAG conformance certification; manual screen-reader, full keyboard, real 200% zoom and SVG text contrast review remain open.


## 7F. Visual asset expansion — 15 Kasım 2026

- Added two more original SVGs: a simulated fermentation-observation timeline for Homebrewers and a conceptual equipment-calibration record model for Technical Principles.
- Total public original SVG assets: eleven; homepage hero remains an existing inline SVG. Product screenshots and photography remain separately gated.
- Both new visuals include explicit simulation/conceptual caveats and accessible title/description/alt text.


## 7G. Visual phase 4 delivery — 15 Kasım 2026

- PR #73 merged with two new original SVG visuals: a simulated fermentation-observation timeline and conceptual equipment-calibration record model.
- Total public original SVGs: eleven; homepage hero remains an existing inline SVG. New assets are caveated as simulated/conceptual, not released product output.
- Detailed diagrams use labelled, keyboard-focusable horizontal scroll regions on narrow screens. Responsive and axe/keyboard tests passed (60/60).
- Main GitHub Pages deployment and checks are being verified after merge.


## 7H. Audience differentiation matrix — 15 Kasım 2026

- Added a side-by-side Homebrewers / Professional Breweries comparison table to How It Works, covering six dimensions: user, work unit, records, measurement context, key question and scope boundary.
- The matrix makes the distinction explicit without presenting roadmap items as released features; the table scroll region is keyboard accessible on narrow screens.
