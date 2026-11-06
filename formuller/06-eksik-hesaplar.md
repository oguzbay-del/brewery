# Eksik hesaplar — yazılacaklar listesi

Bu dosyadaki hesaplar `01`–`05` dosyalarındaki BeerSmith tabanlı spesifikasyonda **yok**. Kaynak: konsept ekranlarının ihtiyaçları ve profesyonel demleme yazılımlarıyla karşılaştırma. Karşılaştırma bilgiye dayanıyor; rakiplerin güncel özellik listeleriyle doğrulanmadı. Formüller henüz yazılmadı; her madde için kaynak seçilip test vektörüyle spesifikasyona eklenecek.

Aşamalar ürün planındaki yol haritasıyla aynı: **Ev MVP**, **İşletme çekirdek**, **İşletme 2. sürüm**, **Sonra**.

## Ev MVP

**Durum:** altı hesabın spesifikasyonu ve test vektörleri yazıldı → `07-ev-mvp-ek-hesaplar.md`, `testler/07_ev_mvp_ek_tv.py`.

| Hesap | Kullanan ekran | Not |
|---|---|---|
| Fermantasyon tahmini | Fermantasyon "Öne Çıkanlar", uyarı kuralları | SG okumalarına eğri oturtma (ör. lojistik/üstel azalma); tahmini FG ve bitiş tarihi, güven aralığıyla. Az veriyle tahmin gösterilmez. |
| Cihaz kalibrasyonu | Cihazlar | iSpindel: açı → SG polinomu (kullanıcı ölçümlerinden en küçük kareler). Tilt: sabit sapma veya doğrusal düzeltme. Sıcaklık sapması. |
| Isınma ve soğuma süresi | Demleme günü zaman çizelgesi, takvim | Isınma: kütle × ısı kapasitesi × ΔT ÷ (güç × verim); ekipman profiline güç ve verim alanı. Soğuma: soğutucu tipine göre basit model, geçmiş partilerden kalibre edilir. |
| Ekipmana göre ölçekleme | Ölçekleme | Hedef değerleri koru (OG, IBU, renk) ve yalnız oranla çarp seçenekleri; verim, kayıplar ve whirlpool soğuma eğrisi hedef ekipmana göre. Mevcut `ScaleGravity`/`ScaleBitterness` üzerine kurulur. |
| İyon dengesi ve sertlik birimleri | Su raporu | Katyon/anyon mEq/L toplamı ve fark yüzdesi (%10 üstü uyarı). °dH, °fH, mEq/L, "CaCO₃ olarak" dönüşümleri. |
| Dry hop, meyve ve kazan hop kayıpları | Hacim zinciri (05) | Pellet hop emilimi (L/kg), kazanda hop tortusu, meyve emilimi; hacim zincirine ayrı kayıp kalemleri olarak. |

## İşletme çekirdek

**Durum:** beş hesabın spesifikasyonu ve test vektörleri yazıldı → `08-isletme-cekirdek-hesaplar.md`, `testler/08_isletme_cekirdek_tv.py`.

| Hesap | Kullanan ekran | Not |
|---|---|---|
| Maliyet | Raporlar, stok | Lot fiyatıyla FIFO; malzeme + isteğe bağlı su, enerji, ambalaj. Parti, litre ve ambalaj birimi başına. |
| Harmanlama | Tank hareketi › Birleştir | Hacim ağırlıklı ABV, IBU (kayıpla), renk, CO₂; hedef değere ulaşmak için oran bulma. |
| Hasat edilen mayayı tekrar kullanma | Maya yönetimi | Slurry yoğunluğu (milyar hücre/mL), canlılık, tortu oranı → eklenecek slurry hacmi; jenerasyon sınırı uyarısı. |
| Profesyonel verim | Raporlar, kalibrasyon önerisi | Plato ve hL ile brewhouse verimi; mash dönüşüm verimi ve süzme verimi ayrı (Braukaiser yaklaşımı). |
| Malt analizinden ekstrakt potansiyeli | Kendi malzemen | Laboratuvar ekstraktı (kuru, ince öğütme), nem, ince/kaba farkı → potansiyel (01 §1.6 ile bağlanır). |

## İşletme 2. sürüm

**Durum:** üç hesabın spesifikasyonu ve test vektörleri yazıldı → `09-isletme-2-hesaplar.md`, `testler/09_isletme_2_tv.py`. Vergi oranları örnek; resmi kaynaktan doğrulanacak.

| Hesap | Kullanan ekran | Not |
|---|---|---|
| Etiket enerjisi | Etiket | AB kuralı: 100 mL başına kJ ve kcal; alkol ve karbonhidrat katsayıları mevzuattan alınacak. Mevcut 07 kalori hesabı ABD porsiyonu içindir. |
| Vergi ve beyan | Mevzuat raporu | Ülkeye göre: hacim ve ABV, ya da Plato bazlı vergi. Oranlar veri olarak tutulur, kodda değil. |
| Spunding | Tank detayı, araçlar | Kalan ekstrakt → üretilecek CO₂; hedef hacme göre ayar basıncı ve kapama SG'si. |

## Sonra

| Hesap | Not |
|---|---|
| Fıçı hattı dengesi | Hortum iç çapı ve malzemesine göre direnç, yükseklik farkı, hedef basınç → hat uzunluğu. |
| Meyve eklemesi | Meyvenin şeker, hacim ve asit katkısı; fermantasyonda eklenince sanal OG'ye (01 §3.4) girer. |
| pH sıcaklık düzeltmesi | Oda sıcaklığı ↔ mash sıcaklığı pH farkı; kaynatma sonrası ve fermentör pH tahmini. |
| Toplam asitlik (TA) | Cider ve şarapta TA ve asit ayarı (malik, tartarik, sitrik). |
| Ek modeller | Noonan IBU; Mosher ve Daniels renk; Terrill doğrusal/kübik ve Novotný refraktometre; BU:GU; hop yağı oranı (mL/L). mIBU zaten planda. |
