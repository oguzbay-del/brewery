# Demleme — Mevcut Site ve FoamByte Başlangıç Denetimi

**Durum:** WP-01 tamamlandı; WP-02 masa başı içerik/benchmark incelemesi tamamlandı, görsel tarayıcı QA bekliyor.  
**Tarih:** 14 Kasım 2026  
**Kapsam:** Yalnız pazarlama sitesi, içerik ve bilgi mimarisi. Ürün uygulaması, `web/`, `ios/`, API, sunucu ve Claude ürün geliştirme kuyruğu kapsam dışıdır.

## 1. Yönetici özeti

Mevcut Demleme sitesi iyi bir başlangıç iskeletine sahip: Astro tabanlı statik yapı, Homebrewers ve Professional Breweries için ayrı landing page'ler, merkezi özellik kataloğu, ürünün geliştirme aşamasında olduğunu belirten mesajlar ve temel teknik SEO altyapısı mevcut.

Buna karşın mevcut deneyim henüz güçlü bir teknik bilgi ve edinim merkezi değil. Home ve Business sayfaları ağırlıklı olarak roadmap özellik kartlarına yönlendiriyor. Ziyaretçinin gerçek demleme problemini anlayıp yöntem, örnek, sınırlama ve ürün bağlantısı üzerinden ilerlemesini sağlayan How It Works, doğrulanmış Calculators, Knowledge Center, Use Cases ve kapsamlı FAQ alanları henüz uygulanmış değil.

FoamByte'ın güncel pazarlama sitesi özellikle ürün kabiliyetlerini somutlaştırma, tekil özellik sayfaları, ücretsiz hesaplayıcı dizini, sosyal kanıt, fiyatlandırma ve FAQ ile ziyaretçiye daha tamamlanmış bir ticari yol sunuyor. Demleme bunu birebir kopyalamamalı: ürün olgunluğu ve ticari kararlar farklı. Demleme'nin ayrışma alanı teknik doğruluk, açıklanabilir hesaplama, derin içerik, Home/Business ayrımının daha belirgin olması ve gerçek ürün kapsamına sadakat olmalı.

**Önemli doğrulama sınırı:** Bu denetim repo kaynak kodu ve erişilebilen FoamByte sayfa içerikleri üzerinden yapıldı. Vercel canlı site bu inceleme oturumunda tarayıcıyla açılamadı. Bu nedenle piksel düzeyinde görsel kalite, gerçek mobil davranış, sayfa hızı ve etkileşimlerin canlı testi tamamlandı olarak işaretlenmiyor.

## 2. Mevcut Demleme site envanteri

| Alan / route | Mevcut durum | Güçlü taraf | Eksik / denetim notu |
|---|---|---|---|
| `/` | Ana sayfa mevcut | Net marka girişi; ürünün in-development olduğu açık; iki audience yolu | Problem odaklı içerik derinliği sınırlı; How It Works ve bilgi merkezi yok |
| `/homebrewers/` | Ayrı Home landing page | Home'a özgü giriş mesajı ve feature filtreleme | Akış ve gerçek senaryolar yerine ağırlıkla planlanan özellik kartları |
| `/breweries/` | Ayrı Business landing page | Profesyonel üretim odağı ve ERP kapsam dışı notu | Operasyonel iş akışı, rol, kayıt, kalite ve izlenebilirlik örnekleri derinleştirilmeli |
| `/features/` | Özellik kataloğu | Merkezi veri kaynağından üretilen katalog | Kullanıcı problemi ve kanıt yerine kabiliyet listesi olma riski |
| `/features/[slug]/` | 16 özellik alanı için detay route yapısı | Bilgi mimarisi geniş; roadmap stage bilgileri mevcut | Her iddia için ürün planı eşleştirmesi ve daha tutarlı kanıt/sınırlama anlatımı gerekli |
| `/404` | Özel 404 route mevcut | Temel hata sayfası | Canlı yönlendirme ve bulunamayan URL testi gerekli |
| `/sitemap.xml` | Astro endpoint mevcut | Site haritası dinamik üretilebiliyor | Vercel production/preview çıktısı ve canonical URL eşleşmesi HTTP üzerinden doğrulanmalı |
| `/robots.txt` | Astro endpoint mevcut | Ortama göre üretim yaklaşımı | Preview ve production kuralları canlı kontrol edilmeli |
| Global navigation | Desktop nav + native `details/summary` mobile menu | Basit, JS bağımlılığı düşük, skip link mevcut | Gerçek mobil cihazda açılma/kapanma, focus ve menü kullanılabilirliği QA bekliyor |
| SEO metadata | Canonical, OG, Twitter, WebSite/WebPage JSON-LD | Temel teknik SEO temeli var | Her yeni sayfa için benzersiz metadata, breadcrumb ve schema uygunluğu kontrol edilmeli |
| Görseller | SVG/placeholder ve beş image slot | Lisanssız/uydurma fotoğraf yok; slotlar tanımlı | Gerçek fotoğraf ve ürün ekranları kullanıcı varlığı/onayı bekliyor |
| Tasarım sistemi | Global CSS tokenları, responsive kurallar | Editorial serif + sans, teknik mono, amber/olive palet, açık/koyu bölüm kontrastı | Semantic token seti, tema stratejisi, component states ve görsel tutarlılık sistematikleştirilmeli |

### 2.1 Mevcut teknik/UX olumlu noktaları

- Astro + TypeScript statik üretim, pazarlama sitesi için uygun ve hafif bir temel.
- Home ve Business ayrı route'larda; ana sayfa iki yolu da görünür kılıyor.
- Native mobil menü ve skip-link/focus-visible temelleri mevcut.
- Canonical, Open Graph, Twitter metadata ve uygun JSON-LD temel sayfalarda uygulanmış.
- Özellik kataloğu merkezi `features.ts` veri kaynağıyla üretiliyor; tekrar eden sayfa yapısını yönetmek kolay.
- Ürün geliştirme aşamasında olduğu açıkça yazılıyor; sahte canlı ürün deneyimi iddiası azaltılmış.
- Görsel slotları tanımlı; izinsiz fotoğraf ekleme yapılmamış.

### 2.2 Mevcut içerik/UX boşlukları

1. Ana sayfanın ilk ekranı ürünün ne olduğunu söylüyor; ancak hedef kitlenin hangi somut sorununu nasıl ele aldığını yeterince kanıtlamıyor.
2. Home ve Business sayfaları ayrılmış olsa da ikisi de benzer bir kart kataloğuna bağlanıyor; kullanıcı yolculukları yeterince farklılaşmıyor.
3. Özellik sayfaları kapsamı görünür kılıyor; fakat problem → yöntem → örnek → sınır → kanıt → ürün aşaması anlatım standardı tüm sayfalarda zorunlu hale getirilmemiş.
4. Hesaplayıcılar, Knowledge Center, FAQ, How It Works ve Use Cases henüz bağımsız trafik/öğrenme girişleri olarak yok.
5. Teknik güven için formül kaynakları, varsayımlar, birim açıklamaları ve doğrulama örnekleri yayınlanabilir bir içerik sistemi halinde değil.
6. Ziyaretçinin ürünün bugünkü durumu ile roadmap ayrımını her özellik seviyesinde kolayca görmesi gerekiyor.
7. Görsel tasarım kaynakta tutarlı bir yön gösteriyor; ancak gerçek tarayıcıda desktop/tablet/mobile değerlendirmesi yapılmadan görsel kalite sonucu verilemez.

## 3. FoamByte benchmark — erişilebilen güncel içerik

İncelenen kaynaklar:
- Homepage: https://foambyte.com/
- Features: https://foambyte.com/features/
- Calculators: https://foambyte.com/calculators/
- Pricing: https://foambyte.com/pricing/
- FAQs: https://foambyte.com/faqs/
- Yeni site anlatısı: https://foambyte.com/blog/new-foambyte-website-built-for-small-beer-producers-and-homebrewers/
- Recipe Studio lansman yazısı: https://foambyte.com/blog/recipe-studio-is-now-live-built-with-brewers-not-behind-closed-doors/

### 3.1 İçerik, bilgi kalitesi ve dönüşüm karşılaştırması

| Boyut | FoamByte — gözlenen yaklaşım | Demleme — mevcut durum | Demleme için fırsat |
|---|---|---|---|
| Konumlandırma | “Brewing software simplified”; homebrewers ve nano breweries | Technical brewing software; Home + Professional Breweries | Basitlik yerine teknik açıklık ve doğrulanabilir yöntemle ayrış |
| Ürün anlatımı | Recipe Studio, Batch Tracking, Fermentation, Water Chemistry, Profiles gibi adlandırılmış modüller | 16 özellik alanına yayılan geniş roadmap kataloğu | Özellik sayfası yanında kullanıcı işi ve süreç anlatımı ekle |
| Somutluk | Ürün görselleri ve örnek vitals ekranları kullanıyor | Gerçek ürün ekranı yok; SVG/placeholder var | Sahte ekran üretme; şema, süreç diyagramı ve formül örneği kullan |
| Hesaplayıcılar | 25+ ücretsiz hesaplayıcıyı ayrı dizinde sunuyor | Henüz yayınlanmış calculator merkezi yok | WP-09/10 formül doğrulamasından sonra tek tek yayınla |
| Teknik içerik | Her hesaplayıcı için ayrı konu ve kullanım niyeti | Planlanan Learn/Knowledge Center henüz yok | Kaynak, varsayım, örnek ve sınırlama içeren teknik rehberler |
| Audience ayrımı | Homebrewers ve nano breweries aynı genel anlatıda | Home ve Business ayrı landing page | Ayrı problem, terminoloji, iş akışı ve CTA ile daha keskin ayrım |
| Sosyal kanıt | İsimli brewer testimonial'ları ve işletme bağlamı | Doğrulanmış testimonial/logo/metric yok | Kanıt elde edilene kadar sosyal kanıt şeridi kullanma |
| Fiyat | Free ve Plus (Coming Soon) katmanları, limitler ve ödeme açıklaması | Fiyat, trial, paket ve ödeme kararları açık değil | Fiyat kararı verilmeden fiyat tablosu yayınlama |
| FAQ | Kapsam, limit, cihaz, billing ve kullanım soruları | Ayrı kapsamlı FAQ sayfası henüz yok | Ürün durumu ve kapsam sınırlarını dürüstçe yanıtlayan FAQ |
| Ürün olgunluğu | Canlı özelliklerle Coming Soon özellikleri aynı sayfada ayrılıyor | Ürün geliştirme aşamasında; planlanan kabiliyetler listeleniyor | Her kabiliyette Planned / In development / Available ayrımını koru |
| Navigasyon | Product, calculators, pricing, FAQ gibi net girişler | Product, Homebrewers, Breweries, Our approach | Learn/Calculators gibi arama niyetli alanları ekle; menüyü sade tut |
| Dönüşüm | Start Free, pricing ve ürün erişimi | Ürün erişimi hazır değil; keşif CTA'ları | Gerçek ürün akışı hazır olana kadar keşif/öğrenme CTA'ları |

### 3.2 FoamByte'dan alınabilecek iyi pratikler

- Tek bir özellik listesi yerine adlandırılmış ve ayrı açıklanmış konu/ürün alanları.
- Ücretsiz hesaplayıcıların SEO giriş sayfaları olarak ayrı dizinlenmesi.
- FAQ ile kullanım, ürün sınırı ve ticari soruların yanıtlanması.
- Homebrewer ve küçük profesyonel üreticinin gündelik işlerini somutlaştıran dil.
- Canlı ve yakında gelecek özellikleri açıkça ayırma.
- Ürün görselleri varsa belirli bir işi/ekranı kanıtlamak için kullanma.

### 3.3 Kopyalanmaması veya Demleme'ye doğrudan taşınmaması gerekenler

- FoamByte'ın metinleri, görsel kompozisyonu, isimlendirmesi ve marka dili.
- Demleme'de henüz bulunmayan hesaplayıcı veya ürün işlevini canlıymış gibi sunmak.
- FoamByte'ın fiyat, ücretsiz plan limitleri, testimonial veya müşteri kanıtlarını Demleme'ye uyarlamak.
- Demleme ürün planı dışında kalan ERP, satış, sipariş, sevkiyat, CRM ve muhasebe kapsamını ima etmek.

## 4. Home ve Business profilleri — içerik/UX farkı

| Karar alanı | Homebrewers (Home) | Professional Breweries (Business) |
|---|---|---|
| Birincil kullanıcı | Kendi ekipmanı ve partisiyle çalışan bireysel üretici | Üretim ekibi, head brewer, operator ve kalite sorumlusu |
| Temel problem | Reçete, su, ekipman ve ölçüm kararlarını anlamak; tekrarlanabilir sonuç almak | Reçete standardı, üretim planı, ekip koordinasyonu, kayıt ve izlenebilirlik |
| İlk anlatılacak konu | Recipe design → brewing calculations → brew day → fermentation → batch learning | Recipe approval → production planning → tank/batch workflow → quality → traceability |
| İçerik tonu | Öğretici, açıklayıcı, adım adım; teknik terimi açıkla | Operasyonel, teknik, ekip ve süreç odaklı; rol ve kayıtları açıkla |
| Örnekler | 20 L reçete ölçekleme, OG/FG, su profili, mash sıcaklığı | Reçete onayı, tank kapasite çakışması, lot-batch bağlantısı, sapma kaydı |
| Görsel yaklaşım | Ev tipi ekipman, ölçüm, reçete/hesap şeması | Üretim alanı, tank akışı, ekip/rol ve kayıt şeması |
| SEO kümeleri | Home brewing software, recipe calculator, water chemistry, ABV, IBU, fermentation | Brewery software, recipe management, production planning, tank scheduling, batch records, traceability |
| Birincil CTA | Ürün hazır olunca Get started; öncesinde yalnız gerçek bir güncelleme/bekleme listesi | Business kapsamı/erişimi hazır olunca ilgili aksiyon; öncesinde kapsamı öğrenme |
| Güven kanıtı | Formül, birim, varsayım, eğitim içeriği ve örnek hesap | Süreç, kayıt modeli, yetki, audit trail ve izlenebilirlik tasarımı |
| Kapsam dışı | Profesyonel ERP varsayımı | Satış, sipariş, dağıtım, keg tracking, CRM ve muhasebe |

## 5. Önceliklendirilmiş fark listesi

| Öncelik | Eksik / fırsat | Neden önemli | Bağlı iş paketi |
|---|---|---|---|
| P0 | Her sayfa için audience, arama niyeti, ürün aşaması ve iddia kaynağı | Yanlış vaat ve karışık mesajı önler | WP-03, WP-05 |
| P0 | Home/Business farklı iş akışları ve görünür fark tabloları | İki kitlenin aynı genel sayfada kaybolmasını önler | WP-04, WP-08 |
| P1 | How It Works | Özellikleri gerçek kullanıcı yolculuğuna bağlar | WP-05, WP-08 |
| P1 | Calculators | Yüksek niyetli teknik arama girişleri ve faydalı etkileşim | WP-09, WP-10 |
| P1 | Knowledge Center / Learn | Teknik otorite, uzun kuyruklu SEO ve ürün eğitimi | WP-11 |
| P1 | Use Cases | İşlevleri gerçek problem bağlamında açıklar | WP-08 |
| P1 | FAQ ve Technical Principles | İtirazları, ürün sınırlarını ve güven sorularını yanıtlar | WP-11 |
| P1 | Görsel sistem ve responsive QA | Tasarımın gerçekten kaliteli ve kullanılabilir olduğunu kanıtlar | WP-06, WP-07, WP-16 |
| P2 | Pricing / trial / waitlist | Ticari kararlar henüz kesin değil | WP-00, WP-15, WP-17 |
| P2 | Gerçek ürün ekranları ve fotoğraflar | Güven ve bağlamı artırır; varlık/onay bekliyor | WP-07, WP-13 |

## 6. İlk tasarım/ içerik yönlendirmesi

1. Ana sayfa iki kitleyi hızlıca ayırmalı; her iki yolun altında farklı problem ve sonuç anlatılmalı.
2. Home landing page bir homebrewing sürecini baştan sona anlatmalı; yalnız özellik kartlarına bağlanmamalı.
3. Business landing page profesyonel üretim döngüsünü, roller ve kayıtlarıyla anlatmalı; genel “brewery management” vaatlerine kaymamalı.
4. Özellik sayfaları “ne yapar?” yanında “hangi problemi çözer, hangi girdiyi ister, nasıl işler, hangi sınırlamaya sahiptir, roadmap durumu nedir?” sorularını yanıtlamalı.
5. Calculators, doğruluk testleri tamamlanmadan yayınlanmamalı; her sonuçta formül, birim, varsayım ve kaynak açıklanmalı.
6. Knowledge Center içeriği önce doğru ve yararlı olmalı; SEO anahtar kelime tekrarıyla şişirilmemeli.
7. Gerçek kanıt yoksa testimonial/logo/metric alanı tasarımdan çıkarılmalı.
8. Mevcut fotoğraf slotları korunmalı; kullanıcı varlığı ve onayı gelmeden fotoğraf eklenmemeli.
9. Apple/Vercel/Stripe kalite çıtası; sade hiyerarşi, güçlü tipografi, grid, tutarlı boşluk, hızlı yükleme ve kusursuz responsive davranış olarak yorumlanmalı. Marka renkleri/kompozisyonları kopyalanmamalı.

## 7. Açık doğrulamalar ve sonraki işler

- [ ] Vercel Preview ve production sayfalarını gerçek tarayıcıda 1440px, tablet ve 390px viewport'larda incele.
- [ ] Sitemap, robots.txt, canonical ve 404 yanıtlarını HTTP üzerinden doğrula.
- [ ] Tüm route'lar, dahili linkler ve CTA hedeflerini otomatik kontrol et.
- [ ] Her özellik iddiasını `plan/demleme-plani.md` ile satır bazında eşleştir.
- [ ] FoamByte'ın ilgili feature/calculator alt sayfalarında örnek derinliği ve formül açıklamasını ayrıca karşılaştır.
- [ ] Rakip karşılaştırmasını bir veya iki ek referansla genişletmeden önce kullanıcı tarafından belirlenen rakip kapsamını koru.
- [ ] WP-04 ve WP-05'e geçmeden önce açık kullanıcı girdilerini/kararlarını WP-00 karar günlüğüne işle.

## 8. Denetim sonucu

WP-01'in kaynak kodu ve repo envanteri tamamlandı. FoamByte'ın ana sayfa, features, calculators, pricing, FAQ ve yeni site anlatısı üzerinden içerik benchmark'ı yapıldı. Canlı görsel QA, HTTP SEO endpoint testi ve ürün-planı satır bazlı iddia denetimi ayrı doğrulama adımları olarak açık tutuluyor.

**Önerilen sonraki paket:** WP-03 ürün-planı ve iddia denetimi. Bu paket, 16 feature sayfasındaki her ifadeyi ürün planı ve roadmap aşamasıyla eşleştirerek site içeriğinin güvenli sınırlarını belirler. WP-02'nin ek rakip kapsamı, yeni rakip listesi verilmedikçe FoamByte ile sınırlı tutulur.
