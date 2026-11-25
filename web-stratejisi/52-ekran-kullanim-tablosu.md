# Ürün ekranları · Kullanım ve yenileme tablosu

[Site planı](51-site-hikayesi.md) · Kaynak sürümü `76b3090` · 25 Kasım 2026

İlk envanterdeki 16 ekran, repodaki [etkileşimli arayüz konseptinden](../arayuz/demleme-arayuz-konsepti.html) seçildi. Kaynak bir HTML prototipidir. “iPad / web” ortak tasarımı ifade eder. Çalışan web uygulamasından alınan görüntü ile iPad simülatöründen alınan görüntü ayrı kaynak türleri olarak kaydedilir. İncelenen main sürümünde bu seçkideki iş akışları için ayrı, hazır web ve iPad yakalamaları doğrulanmadı.

Her satır bir kaynak ekrandır. Yerleşimler iki dil için öneridir. Bu sürüm planlama aşamasındadır; canlı kullanım ve son yayın sütunundaki tire henüz yayın kaydı bulunmadığını belirtir. Kaynak arayüzün mevcut dili iki sitede de kullanılabilir.

| Ekran anahtarı | Ekran ve kaynak sekmesi | Tasarım yüzeyi | Planlanan sayfa ve bölüm | Mobil kırpım odağı | Canlı kullanım / son yayın |
|---|---|---|---|---|---|
| `ed` | [Tarif editörü](https://github.com/oguzbay-del/brewery/blob/76b3090fa8b1e9aed8e2860e515eebfe9631c8ae/arayuz/demleme-arayuz-konsepti.html#L690) · `#pane-ed` | iPad / web | Ana sayfa · reçete hedefi; features/recipe-design · düzenleme | Hedef ve Inspector | — |
| `bd` | [Demleme günü](https://github.com/oguzbay-del/brewery/blob/76b3090fa8b1e9aed8e2860e515eebfe9631c8ae/arayuz/demleme-arayuz-konsepti.html#L796) · `#pane-bd` | iPad | Ana sayfa · aktarım; how-it-works · uygulama; homebrewers · günlük | Aktif adım ve sayaç | — |
| `fe` | [Fermantasyon özeti](https://github.com/oguzbay-del/brewery/blob/76b3090fa8b1e9aed8e2860e515eebfe9631c8ae/arayuz/demleme-arayuz-konsepti.html#L873) · `#pane-fe` | iPad | how-it-works · izleme; features/fermentation-monitoring · parti seyri | Seçili parti ve ölçüm | — |
| `bs` | [Tamamlanan parti](https://github.com/oguzbay-del/brewery/blob/76b3090fa8b1e9aed8e2860e515eebfe9631c8ae/arayuz/demleme-arayuz-konsepti.html#L1312) · `#pane-bs` | iPad / web | Ana sayfa · parti özeti; how-it-works · kapanış | Sonuç ve hedef karşılaştırması | — |
| `pr` | [Üretim panosu](https://github.com/oguzbay-del/brewery/blob/76b3090fa8b1e9aed8e2860e515eebfe9631c8ae/arayuz/demleme-arayuz-konsepti.html#L3689) · `#pane-pr` | iPad / web | breweries · vardiya; compare · İşletme; modules · üretim | Tankların durumları | — |
| `tk` | [Tank detayı](https://github.com/oguzbay-del/brewery/blob/76b3090fa8b1e9aed8e2860e515eebfe9631c8ae/arayuz/demleme-arayuz-konsepti.html#L3017) · `#pane-tk` | iPad / web | breweries · tank kararı; features/production-management · tank | Tek tank ve bağlı parti | — |
| `qa` | [Kalite ve serbest bırakma](https://github.com/oguzbay-del/brewery/blob/76b3090fa8b1e9aed8e2860e515eebfe9631c8ae/arayuz/demleme-arayuz-konsepti.html#L1543) · `#pane-qa` | iPad / web | breweries · kalite kararı; modules · kalite; features/quality-laboratory | Ölçüm ve spesifikasyon | — |
| `inv` | [Lot bazlı stok](https://github.com/oguzbay-del/brewery/blob/76b3090fa8b1e9aed8e2860e515eebfe9631c8ae/arayuz/demleme-arayuz-konsepti.html#L3467) · `#pane-inv` | iPad / web | modules · stok; features/inventory-costing | Malzeme ve lot | — |
| `lt` | [Lot izlenebilirliği](https://github.com/oguzbay-del/brewery/blob/76b3090fa8b1e9aed8e2860e515eebfe9631c8ae/arayuz/demleme-arayuz-konsepti.html#L3062) · `#pane-lt` | iPad / web | features/inventory-costing · partinin kaynağı | Lot ile parti bağlantısı | — |
| `wiz` | [Tarif sihirbazı](https://github.com/oguzbay-del/brewery/blob/76b3090fa8b1e9aed8e2860e515eebfe9631c8ae/arayuz/demleme-arayuz-konsepti.html#L647) · `#pane-wiz` | iPad | homebrewers · başlangıç; compare · Ev | İlk seçim | — |
| `cmp` | [Tarif karşılaştırma](https://github.com/oguzbay-del/brewery/blob/76b3090fa8b1e9aed8e2860e515eebfe9631c8ae/arayuz/demleme-arayuz-konsepti.html#L1248) · `#pane-cmp` | iPad / web | homebrewers · yeniden demleme; features/recipe-design · sürümler | İki sürüm arasındaki fark | — |
| `cal` | [Geriye planlama](https://github.com/oguzbay-del/brewery/blob/76b3090fa8b1e9aed8e2860e515eebfe9631c8ae/arayuz/demleme-arayuz-konsepti.html#L2032) · `#pane-cal` | iPad / web | how-it-works · planlama; features/planning-scheduling | Takvim ve üretim günü | — |
| `tools` | [Hesap araçları](https://github.com/oguzbay-del/brewery/blob/76b3090fa8b1e9aed8e2860e515eebfe9631c8ae/arayuz/demleme-arayuz-konsepti.html#L1357) · `#pane-tools` | iPad / web | technical-principles · ölçüm; features/brewing-calculations | Girdi, sonuç ve birim | — |
| `hw` | [Değer açıklaması](https://github.com/oguzbay-del/brewery/blob/76b3090fa8b1e9aed8e2860e515eebfe9631c8ae/arayuz/demleme-arayuz-konsepti.html#L2895) · `#pane-hw` | iPad / web | technical-principles · sayı nereden geliyor | Açık açıklama penceresi | — |
| `cip` | [Temizlik ve ambalaj kaydı](https://github.com/oguzbay-del/brewery/blob/76b3090fa8b1e9aed8e2860e515eebfe9631c8ae/arayuz/demleme-arayuz-konsepti.html#L1590) · `#pane-cip` | iPad / web | features/packaging-compliance · parti sonu | CIP veya dolum kaydı | — |
| `cl` | [Ekipman kalibrasyonu](https://github.com/oguzbay-del/brewery/blob/76b3090fa8b1e9aed8e2860e515eebfe9631c8ae/arayuz/demleme-arayuz-konsepti.html#L3356) · `#pane-cl` | iPad / web | features/equipment-calibration · ekipman profili | Kalibrasyon girdisi | — |

## “Web ekranlarını güncelle” dendiğinde

1. Tabloyu aç. Web ve ortak iPad/web satırlarından canlı yerleşimi olanları seç. Yalnız iPad satırlarını ayrı tut. Planlanan ancak henüz yayımlanmamış ekranları değişiklik özetinde ayrıca göster.
2. En son ürün sürümünde aynı ekranı ve aynı örnek veri durumunu bul. Konsept için ilgili sekmeyi aç ve panel içindeki cihazı yakala. Gerçek web ekranı için uygulama rotasını, iPad için uygulama sürümü, cihaz yönü ve görünümü kaydet. Tasarımın yerini çalışan ekran aldığında kaynak türünü değiştir.
3. Yeniden kullanılan her dosyaya sabit bir anahtar ver. İlk yerleştirme PR’ında her kullanım için aşağıdaki tabloya bir satır ekle. Bir görsel üç sayfadaysa üç kullanım kaydı olmalı. EN ve TR aynı dosyayı paylaşıyorsa iki rotayı aynı satırda sakla.
4. Önce-sonra önizlemesinde kırpım, yazı okunurluğu, örnek veri, dil eşleri ve altyazıyı karşılaştır. Kaynak revizyonu, kullanılan dosya ve son yayın commit’ini gerçek değerleriyle güncelle. Kaldırılmış yerleşimi “kaldırıldı” olarak işaretle.

### Yayındaki yerleşimler

| Ekran anahtarı | Kaynak türü, revizyon ve sekme/rota | Görsel dosyası | EN + TR sayfa/bölüm | Kırpım ve altyazı kaynağı | Durum ve son yayın commit’i |
|---|---|---|---|---|---|

İlk yerleştirme PR’ı ile doldurulur. Bölüm adresi olarak mevcut bir HTML kimliği veya Astro bileşeninin yolu kullanılabilir.
