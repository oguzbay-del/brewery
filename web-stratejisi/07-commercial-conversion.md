# Self-service ticari akış ve paket seçimi

## Ticari öncelik
**Professional Breweries birincil ticari segmenttir.** Homebrewers ayrı ve tam bir ürün yolu olarak korunur; ana sayfa, edinim ve Business dönüşüm hiyerarşisi Professional Breweries'i öne çıkarır.

## Keşiften satın almaya hedef yolculuk
1. **Discover:** Search/content entry → homepage veya ilgili audience page.
2. **Understand:** `/breweries/` veya `/homebrewers/` üzerinden hedef kitle ve workflow seçimi.
3. **Evaluate:** `/features/`, `/compare/`, How It Works, Technical Principles ve FAQ.
4. **Configure:** `/pricing/` plan özetinden `/get-started/` adım adım plan sihirbazına.
5. **Review:** Kullanıcı tipi, koltuk sayısı, ödeme dönemi, önerilen indirim ve deneme süresinin özeti.
6. **Checkout:** Gelecek faz. Şimdilik gerçek hesap açma, trial başlatma, kart verisi veya ödeme yok.
7. **Activate/convert:** Yalnız ürün onboarding ve billing altyapısı gerçek anlamda devreye girdikten sonra.

## Paket mimarisi
- **Homebrewer:** 1 kullanıcı; 3 aylık deneme süresi planlanıyor.
- **Professional Brewery:** en az 5 kullanıcı; 5'erli artışlarla 5 / 10 / 15 / 20 kullanıcı; 1 aylık deneme süresi planlanıyor.
- **20 üzeri kullanıcı:** standart self-service koltuk seçimi yerine özel teklif ve e-posta iletişimi. Genel satış e-posta adresi yayına çıkmadan önce yapılandırılmalı.
- Her iki segment: aylık, yıllık, 2 yıllık ve 3 yıllık dönem seçimi.
- Sihirbazda gösterilecek önerilen indirim: aylık %0, yıllık %15, 2 yıllık %25, 3 yıllık %35. Bunlar nihai sözleşme değil, launch pricing önerisidir.

## Fiyat ve ödeme durumu
Önceki FoamByte benchmark'ı (WP-13) Free/Plus plan yapısı ve fiyat/plan/billing şeffaflığına dikkat çeker; repoda doğrulanmış rakip fiyat tutarları bulunmaz. Bu nedenle Home aylık fiyatı ve Business kullanıcı başı aylık fiyatı uydurulmaz. Para birimi, vergi/VAT, çok yıllı peşin ödeme/yenileme, iptal ve iade kuralları sahibi tarafından onaylanana kadar tutar gösterilmez.

Ödeme niyeti kredi/banka kartı olarak gösterilir; ancak provider SDK, kart alanı, tokenizasyon, checkout, abonelik, tahsilat, trial entitlement veya webhook bu website-only iş paketine dahil değildir. Site üzerinde konfigürasyon önizlemesi, gerçek satın alma gibi sunulmaz.

## Fonksiyon karşılaştırması
`/compare/` Home ve Business kapsamını aynı tabloda gösterir. Her satır canonical feature detail sayfasına gider. Aşama etiketleri mevcut feature catalogue ile tutarlı olmalı; "Homebrew MVP" ve "Business core" canlı kullanılabilirlik anlamına gelmez. `/features/` detay kapsamının kaynağı olmaya devam eder.

## Site hiyerarşisi
- Ana navigasyonda Pricing erişilebilir.
- Pricing sayfasında iki ayrı plan kartı, deneme süresi, ödeme dönemi ve proposed discount görünür.
- Pricing ve wizard içinden function comparison bağlantısı bulunur.
- Breweries ve Homebrewers audience sayfaları kendi plan yapılandırma yoluna yönlendirir.
- Business birincil commercial CTA olarak kalır; Home yolu gizlenmez.

## Ürün olgunluğu / güven
GRISTORA in development. Plan, trial, checkout, account ve ödeme işlemleri canlı değildir. Fiyatlandırma sayfaları ürün özelliği, müşteri, SLA, güvenlik sertifikası veya ödeme sağlayıcısı iddiası üretmez. Site yalnızca plan seçimi ve kullanıcı niyetini gösteren statik bir preview sağlar.

## Sonraki fazlar
- **Website (current):** pricing page, audience chooser, seat selector, term discounts, summary, compare table, responsive/accessible QA.
- **Product/Billing (future, separate):** auth, workspace creation, trial start/expiry, seat entitlements, subscription state, payment provider, card security/3DS, tax, invoices, cancellation, refunds, renewals, webhook reconciliation and support.
- **Launch operations:** approved price list, contact email, terms/privacy, currency/tax, jurisdiction review and app URL.

## Açık owner decisions
1. Home aylık tutar ve Business kullanıcı başı aylık tutar.
2. %15 / %25 / %35 indirimlerin onayı ve hesaplama temeli.
3. USD/GBP ve diğer para birimi politikası, VAT/sales tax gösterimi.
4. 2/3 yıllık dönemlerin peşin mi yıllık mı tahsil edileceği; yenileme ve fiyat sabitleme.
5. Trial başlangıcı, kart zorunluluğu, deneme sonunda ücretli dönüşüm ve iptal.
6. 20+ kullanıcı için yayınlanacak sales e-posta adresi.
7. Ödeme sağlayıcısı, 3DS/SCA, iade, fatura ve abonelik yaşam döngüsü.
