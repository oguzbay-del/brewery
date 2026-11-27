# G8 — Marka ve ürün tutarlılığı

27 Kasım 2026 — Taslak inceleme. Referans: [ADR-0012](../../adr/0012-urun-markasi.md), onaylı logo [#148](https://github.com/oguzbay-del/brewery/pull/148), [G5 #429](https://github.com/oguzbay-del/brewery/pull/429) ve [G6 #431](https://github.com/oguzbay-del/brewery/pull/431) (ikisi de birleştirilmiş).

## 1. Uyumsuzluklar ve öneriler

| Bulgu | Kaynak | Öneri |
|---|---|---|
| iOS görünen adı GRISTORA, ancak uygulama bundle kimliği hâlâ `app.demleme.Demleme`. ADR-0012 §4 hedefi `com.gristora.app`. | [ios/project.yml](../../ios/project.yml), ADR-0012 §4 | İlk TestFlight yüklemesinden önce uygulama ve test hedeflerinin kimliklerini ADR ile eşleştir; imzalama, Apple giriş ve universal link etkilerini Claude ile doğrula. İç proje ve hedef adlarının değişmesi gerekmez. |

## 2. Doğrulanmış ortak öğeler

| Öğe | Web uygulaması | iOS | Pazarlama |
|---|---|---|---|
| Görünen ad | [index.html](../../web/index.html): GRISTORA; [manifest](../../web/public/manifest.webmanifest): name ve short_name GRISTORA | [project.yml](../../ios/project.yml): INFOPLIST_KEY_CFBundleDisplayName GRISTORA | [BRAND-ASSETS.md](../../marketing-site/BRAND-ASSETS.md): GRISTORA |
| Favicon | [favicon.svg](../../web/public/favicon.svg) | AppIcon ayarı var; varyantların cihazda görsel doğrulaması açık | [favicon.svg](../../marketing-site/public/favicon.svg); web uygulamasındaki SVG içeriğiyle birebir aynı |
| Logo | #429: başlık sembolü ve duyarlı yazı | #431: açık, koyu ve renklendirilmiş ikon, launch screen; cihaz testi henüz doğrulanmadı | [BRAND-ASSETS.md](../../marketing-site/BRAND-ASSETS.md): #148 onaylı varlıklar; gerçek rota kullanımı ayrıca incelenecek |

## 3. Ürün iddiaları

İddiaların rota bazlı envanteri ve her birinin uygulama/API veya ISLER satırı ile eşlemesi henüz tamamlanmadı. Bu taslak ürünün yayınlanabilirliğini veya özelliklerin mevcut olduğunu onaylamaz.

İnceleme devamında EN/TR rotaları, özellik metinleri, ekran görüntüleri, sunucu uçları ve gerçek kullanıcı akışları kontrol edilecek. Pazarlama metni veya uygulama kodu bu rapor kapsamında değiştirilmez.

### 3.1 İlk özellik kataloğu karşılaştırması

Kaynak: [yayındaki özellik sayfalarının veri dosyası](../../marketing-site/src/data/features.ts). Uygulama tarafındaki ilk kanıt: [API modül envanteri](../../sunucu/crates/api/CLAUDE.md). API kanıtı, web veya iOS ekranının tamamlandığı veya ürünün yayında olduğu anlamına gelmez. Aşağıdaki satırlar grup düzeyinde ilk eşlemedir; kalan madde ve rota denetimi açık.

| Özellik rotası | Kodda belgelenen karşılık | Açık iddialar ve öneri |
|---|---|---|
| recipe-design | T2.21: [recipes.rs](../../sunucu/crates/api/src/recipes.rs), [recipe_org.rs](../../sunucu/crates/api/src/recipe_org.rs): sürüm, yayınlama, ölçekleme önizlemesi, karşılaştırma, klasör, etiket, favori ve yumuşak kilit | Başlangıç şablonları, ekstrakt dönüşümü, tüm stil rehberleri, özel alanlar ayrı doğrulanmalı. T2.26 BeerXML/BeerJSON/Brewfather JSON aktarımı var; BSMX örnek dosya bekliyor; diğer formatlara uyumluluk iddiası yok. |
| brewing-calculations | T2.16: [engine.rs](../../sunucu/crates/api/src/engine.rs): sunucu motoruna gRPC, sürüm damgası ve hesap sonucu kaydı | Her özel hesap modeli, yöntem seçimi, sonuç açıklaması ve referans doğrulaması ayrı kontrol edilmeli. |
| water-mash-yeast | Motor bağlantısı T2.16; bu alanın her işlevi için doğrudan kanıt henüz okunmadı | Su raporu, tuz/asit, mayşeleme ve maya hazırlığı maddelerini tek tek doğrula. |
| brew-day | T2.22: [batches.rs](../../sunucu/crates/api/src/batches.rs), [brew_day.rs](../../sunucu/crates/api/src/brew_day.rs): onaylı tariften parti, adımlar, ölçüm, sapma, imzalı paket | Live Activity, zamanlayıcı, baskı, QR, belge tarama ve OCR kullanıcı akışları henüz doğrulanmadı. T2.18 [files.rs](../../sunucu/crates/api/src/files.rs) yalnız belge saklama uçlarının kanıtı. |
| fermentation-monitoring | T2.22 parti aşamaları ve ölçümleri; T2.25 [devices.rs](../../sunucu/crates/api/src/devices.rs); T2.24 [notifications.rs](../../sunucu/crates/api/src/notifications.rs) | Grafikler, kaynak etiketleri, öneriler, widget, vardiya ve tadım akışları ayrı doğrulanmalı. |
| equipment-calibration | T2.37 [trash.rs](../../sunucu/crates/api/src/trash.rs) ekipman geri yükleme desteğini belgeler; profil ve kalibrasyon ayrıntıları henüz doğrulanmadı | Tank basıncı, doluluk, temizlik ve kalibrasyon özellikleri için doğrudan kanıt gerekli. |
| planning-scheduling | T2.22: takvim ve geriye planlama | Isıtma/soğutma tahmini, kapasite optimizasyonu, kişiselleştirilmiş üretim panosu henüz doğrulanmadı. |
| inventory-costing | T2.23 [stock.rs](../../sunucu/crates/api/src/stock.rs): lot, hareket defteri, giriş/tüketim/sayım, eski lottan düşüm, alışveriş listesi | FIFO tüketim tek başına FIFO maliyetleme kanıtı değildir. Barkod, COA, maliyet, rezervasyon ayrı kontrol edilmeli. |
| production-management | T2.22 parti/adım/ölçüm; T2.23 stok hareketleri | Tank transfer/bölme/birleştirme, geri çağırma, temizlik uyarısı ve vardiya panosu ayrı incelenmeli. ERP kapsam dışı ifadesi korunmalı. |
| quality-laboratory | Henüz doğrudan kalite modülü kod kanıtı bu denetimde okunmadı | Kalite limitleri, laboratuvar, serbest bırakma, HACCP ve çift onay için kanıt bulunmadan mevcut özellik olarak tanıtma. |
| packaging-compliance | T2.22 parti aşamaları; paketleme ve CIP ayrıntısı henüz doğrulanmadı | Vergi/ülke uyumu iddiaları için mevzuat ve uygulama kanıtı gerekir. |
| team-notifications | T2.13 rol/üye/davet/Home paylaşımı; T2.12 ortak cihaz/PIN; T2.24 bildirim kutusu, tercihler ve uyarı kuralları | Mod geçişi, rol kapsamı, vardiya ve onay kuyrukları ayrı doğrulanmalı. |
| connected-devices | T2.25 cihaz kaydı, parti eşleme, anahtar yenileme; T2.24 uyarılar | Belirli marka ve protokol bağlantıları doğrulanmadı; salt okunur ve geleceğe dönük ifadeler korunmalı. |
| imports-exports-tools | T2.26 [import_export.rs](../../sunucu/crates/api/src/import_export.rs): üç doğrulanmış aktarım biçimi, aktarımı uygulama, dışa aktarma işi ve indirme bağlantısı | BSMX açık; diğer araç ve biçimler tek tek doğrulanmalı. |

**Öncelikli metin riski:** hesap motoru, reçete ve üretim sayfalarındaki kesin şimdiki zaman ifadeleri, bir API modülünün varlığıyla tüm alt özelliklerin çalıştığı izlenimini verebilir. Mevcut işlevi yalnız doğrulanan maddeye bağla; doğrulanmayan iddiaları yayın kapsamından çıkar. Bu rapor henüz tüm özellik maddelerini kapsamıyor.
