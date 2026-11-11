# Elle giriş tahmini, ekstrakt araçları, soğutmasız IBU: spesifikasyon

Planın "Ek özellikler" tablosundaki üç Ev MVP satırının hesapları:
- az okumayla fermantasyon tahmini,
- ekstrakt biracısı araçları,
- soğutmasız (no-chill) IBU ayarı.

Bunlar BeerSmith'ten çıkarılmadı; bizim tasarımımız. Referans uygulama ve test vektörleri: `testler/10_elle_ekstrakt_nochill_tv.py` (`python3 -I testler/10_elle_ekstrakt_nochill_tv.py`).

**Karşılaştırma ilkesi** (00-OKU-BENI.md › Önemli genel notlar): bu dosyadaki bütün gravite eşikleri (0.0005, 0.001, 0.003, 0.004, %30) tam sayı birimde karşılaştırılır; 1 birim = 0.00001 SG (0.01 puan), `u(SG) = round(SG × 100000)`. Sürekli büyüklüklerde (oran, eğim) açık tolerans 1e-9 kullanılır. Gerekçe: `1.014 − 1.013 = 0.001000000000000112` olduğundan `≤ 0.001` ikili kayan noktada yanlış çıkar.

İşaretler: **[kesin]** tanım veya birim dönüşümü; **[tasarım]** bizim seçtiğimiz yöntem; **[doğrulanacak]** varsayılan değer.

Birimler: °C, L, g, kg, dakika, saat; yoğunluk SG.

---

## 1. Az okumayla fermantasyon tahmini

07 §1'deki model saatlik cihaz verisi ister (en az 12 nokta). Cihazı olmayan kullanıcı ise genelde 2–3 kez hidrometre veya refraktometreyle ölçer. Üç parametreli modeli 2–3 noktaya oturtmak mümkün değil. Bu bölüm FG'yi tariften alıp yalnız hızı tahmin eder.

### 1.1 Hangi yöntem ne zaman [tasarım]
- Partide 07 §1.4 koşulları sağlanıyorsa (cihaz verisi yeterli) 07 §1 kullanılır.
- Sağlanmıyorsa ve elle okumalar varsa bu bölüm kullanılır.
- Ölçümün **kaynak etiketi** (cihaz / elle / kağıt) seçimi belirler. Cihaz verisi olan partide elle hidrometre okuması ayrı nokta olarak modele girmez; cihazın sapma düzeltmesinde kullanılır (07 §2.2).
- Refraktometre okuması fermantasyon başladıktan sonra alkol düzeltmesiyle SG'ye çevrilir (01 §9.2, tip 1). Düzeltilmemiş Brix girilemez.

### 1.2 Model [tasarım]
```
SG(t) = FG + (OG − FG) · e^(−k·(t − L))     t > L
```
- `FG`: tarifin tahmini FG'si (maya attenüasyonu ve mash sıcaklığı düzeltmesiyle, 01). Sabit kabul edilir.
- `OG`: demleme günü ölçülen OG; parti başlangıcı `t = 0`.
- `L`: gecikme fazı, varsayılan **12 saat** [doğrulanacak]. Bu süreden önceki okumalar kullanılmaz.
- `k`: tek bilinmeyen.

### 1.3 Oturtma [tasarım]
Her okuma için `x = t − L`, `y = ln((SG − FG)/(OG − FG))`. Orijinden geçen **ağırlıklı** en küçük kareler (F1 kararı 11):
```
k = −Σ(w·x·y) / Σ(w·x²)
w = min(1, (SG − FG) / 0.003)          (tam sayı birimde: min(1, (u(SG) − u(FG)) / 300))
```
- `SG − FG ≥ 0.003` olan okumada `w = 1`; FG'ye 3 puandan yakın okumanın ağırlığı doğrusal düşer. Gerekçe: logaritmik uzayda FG'ye yakın okuma `y`'yi çok büyütür ve bir puanlık okuma hatası bitiş zamanını ±22 saat kaydırırdı; ağırlık bu etkiyi bastırır (vektör 1h: 84. saatteki 1.013 / 1.014 okuması bitişi ağırlıksız 20.3 saat, ağırlıklı 7.6 saat oynatır).
- `SG − FG ≤ 0.0005` (`u(SG) − u(FG) ≤ 50`) olan okuma kullanılmaz (log tanımsızlaşır).
- Ağırlık, oturtulan FG'ye göre hesaplanır; aralık için FG ± σ ile yeniden oturtmada ağırlıklar da yeniden hesaplanır.
- Tutarsızlık denetimi (§1.6) tek tek `k` değerlerinin oranına bakar, ağırlıktan etkilenmez.
- Çözücü gerekmez, her zaman tek sonuç verir.

### 1.4 Ne zaman gösterilir [tasarım]
Dört koşul birlikte:
- gecikme fazından sonra en az **2 okuma**,
- son okuma beklenen düşüşün en az **%30**'unu tamamlamış (07 §1.4 ile aynı eşik; tam sayı birimde `10·(u(OG) − u(SG_son)) ≥ 3·(u(OG) − u(FG))`),
- `k > 0`,
- durgunluk kuralı (§1.5) devreye girmemiş.

**Değerlendirme sırası** (referans uygulama): (1) durgunluk kuralı (§1.5; gecikme öncesi okumalar da sayılır, veri yeterliliğinden önce çalışır), (2) gecikme sonrası en az 2 okuma, (3) %30 koşulu, (4) `SG_son ≤ FG + 0.0005` ise "hedefe ulaştı; durgunluk için 48 saat sonra tekrar ölç", (5) `k > 0`.

Koşul sağlanmazsa "veri yetersiz" yazılır ve hatırlatıcı bir sonraki okumayı ister (plan: Bildirim yapısı › Hatırlatıcılar).

### 1.5 Durgunluk kuralı [tasarım]
Son iki okuma en az **48 saat** arayla alınmış ve farkı **≤ 0.001** (`|u(SG₁) − u(SG₂)| ≤ 100`) ise:
- `SG_son ≤ FG + 0.004` (`u(SG_son) ≤ u(FG) + 400`) → "bitmiş görünüyor". Ambalaj adımı önerilir.
- değilse → "takılmış olabilir" uyarısı. Uyarı kuralları ekranındaki "takılma" kuralını tetikler.

Bu, ev biracılarının bilinen "iki üç gün aynı okuma" kuralının sayısal hali.

### 1.6 Çıktılar [tasarım]
- **Bitiş zamanı:** `t_bitiş = L + ln((OG − FG)/ε)/k`, `ε = 0.0005` (07 §1.5 ile aynı).
- **Aralık:** FG ön tahmini `±σ` kaydırılıp (`σ = 0.003` [doğrulanacak]) `k` yeniden oturtulur. Üç bitiş zamanının en küçüğü ve en büyüğü alınır.
- **Gösterim:** gün çözünürlüğünde aralık ("2–7 gün içinde") ve "yaklaşık" etiketi. Saat verilmez. Elle okumada saat hassasiyeti yanıltıcıdır.
- **Tutarsızlık:** okumaların tek tek verdiği `k` değerlerinin oranı 2'den büyükse (`> 2 + 1e-9`) "okumalar birbiriyle uyuşmuyor" notu çıkar (yanlış okuma, sıcaklık düzeltmesi unutulmuş ya da profil adımı değişmiş olabilir).
- Grafikte elle okumalar ayrı işaretle çizilir; tahmin eğrisi kesikli çizgidir.

### 1.7 Bilinen sınırlar
- FG tariften geldiği için tahmin, maya beklenenden farklı attenüe ederse kayar. Bunu durgunluk kuralı yakalar.
- 07 §1 ile aynı nedenle sıcaklık profili adımları modelde yok.

---

## 2. Ekstrakt biracısı araçları

### 2.1 Puan katsayısı [kesin]
```
PPG = yield% × 46 / 100                  (01 §1.6)
puan/(kg/L) = PPG × 8.3454               (1 kg/L = 8.3454 lb/gal)
```

Tipik değerler (malzeme veritabanındaki değer önceliklidir) [doğrulanacak]:

| Ekstrakt | yield % | PPG | puan/(kg/L) | Hacim dönüşümü |
|---|---|---|---|---|
| LME (sıvı) | 78 | 35.9 | 299.4 | yoğunluk 1.42 g/mL |
| DME (toz) | 95 | 43.7 | 364.7 | yığın yoğunluğu 0.60 g/mL (gevşek, kaba) |
| Bal | 75 | 34.5 | 287.9 | yoğunluk 1.42 g/mL |

### 2.2 Gram ↔ mL [kesin tanım, doğrulanacak yoğunluk]
- `mL = g / yoğunluk`, `g = mL × yoğunluk`.
- DME için hacim ölçüsü sıkıştırmaya göre çok değişir. Ekranda "tartarak ölçün" notuyla gösterilir.
- ABD için ölçü kabı (cup = 236.588 mL), yemek kaşığı (14.787 mL) ve lb (453.592 g) seçilebilir.

### 2.3 Ekstrakt değiştirme [kesin]
Aynı puanı verecek miktar:
```
m₂ = m₁ · yield₁ / yield₂
```
Örnek: LME → DME ≈ × 0.82.

### 2.4 Hedef OG için miktar [kesin]
```
g = (OG − 1)·1000 · V_son / puan_kgL · 1000
```
Ekstraktta mash verimi yoktur (%100 kabul). Tahıl demlemesiyle (steep) gelen puanlar 01'deki steep verimiyle ayrıca eklenir.

### 2.5 Kısmi kaynatma ve üstüne su [tasarım]
Kazanda `V_k` litre kaynatılır, fermentörde `V_son`a suyla tamamlanır.
```
fermentöre giden = V_k − kayıp
eklenecek su     = V_son − (V_k − kayıp) = V_son − V_k + kayıp
puan_kazan       = puan_son · V_son / (V_k − kayıp)
```
`V_k` kaynatma sonu kazan hacmi, `kayıp` kazanda kalan hacimdir (trub, hop emilimi). Kayıpla birlikte şeker de kazanda kaldığı için fermentöre giden şıranın gravitesi, kayıpsız hesaba göre yüksek olmalı; açığı su kapatır (kayıp arttıkça eklenecek su artar). `V_k − kayıp > V_son` ise kısmi kaynatma anlamsız, uyarı verilir.
- **IBU:** Tinseth kullanımı **kazandaki gravitede** hesaplanır, mg/L **son hacimde**: `IBU = U(SG_kazan, t) · AA · g · 1000 / V_son`. Yüksek kazan gravitesi kullanımı düşürür. Örnek (§4): aynı hop, kısmi kaynatmada 28 IBU, tam hacimde 44 IBU.
- **Geç ekstrakt:** Ekstraktın bir kısmı kaynatma sonunda eklenirse kazan gravitesi zamana göre değişir. 02'deki geç ekstrakt SG segmentleri kullanılır.
- **Renk:** Morey son hacimle (değişiklik yok). Kısmi kaynatmadaki ek koyulaşma modellenmez.
- Yüksek hop yükünde kazandaki alfa asit derişimi Tinseth'in doğrulandığı aralığın (ve çözünürlük sınırının) üstüne çıkabilir (§4 2h örneğinde 560 mg/L); sonuç iyimser olabilir, ekranda "yaklaşık" etiketi gösterilir.
- BeerSmith'in kısmi kaynatmada IBU'yu hangi graviteyle hesapladığı [doğrulanacak]. Karşılaştırma ekranında fark not edilir.

### 2.6 Üstüne eklenen suyun sıcaklığı [yaklaşık fizik]
Hedef maya atma sıcaklığına ulaşmak için:
```
T_su = (T_hedef · (V_şıra·c + V_su) − V_şıra·c·T_şıra) / V_su       c ≈ 1
```
Sonuç 1 °C'nin altındaysa "buz ya da soğutma gerekir" uyarısı verilir.

---

## 3. Soğutmasız (no-chill) IBU ayarı

Şıra soğutulmadan kapalı bir küpe (genelde 20–25 L HDPE) aktarılır ve bir gecede kendiliğinden soğur. Avustralya ve İngiltere'de yaygın. Soğuma uzun sürdüğü için hop izomerizasyonu devam eder; acılık hesaplanandan yüksek çıkar.

### 3.1 Basit kural (varsayılan) [tasarım, topluluk kuralı]
Her hop katkısının süresine **+20 dk** eklenir; kaynatma sonu (0 dk) katkısı 20 dk sayılır. Tinseth'in zaman fonksiyonu doyduğu için uzun katkılar az etkilenir (60 → 80 dk: +%5), geç katkılar çok etkilenir.

### 3.2 Model (gelişmiş) [tasarım, doğrulanacak]
Küpün soğuma eğrisi Newton yasasıyla:
```
T(t) = T_ortam + (T₀ − T_ortam) · e^(−t/τ)
τ = τ₂₀ · (V/20)^(1/3)          τ₂₀ = 5 saat [doğrulanacak]
```
Eşdeğer kaynatma dakikası, sıcaklığa bağlı göreli hızın (02 §3, Malowicki–Shellhammer) integralidir:
```
t_eş = ∫ Ftemp(T(t)) dt         (T < 40 °C katkısı ihmal)
Ftemp(T) = 2.39·10¹¹ · e^(−9773/(T + 273.15))      T ≥ 100 °C için Ftemp = 1 (kırpma)
```
- **Sayısal yöntem:** sol Riemann toplamı, adım `dt = 0.5 dk`. Test vektörü 3a/3d/3e bu yöntemle üretilir; ince adımlı integral (dt = 0.01 dk) T₀ 95 °C, τ = 5 sa için 44.9 dk verir (vektör 45.0 dk). Rust motoru aynı yöntemi kullanır ya da vektörü ±0.3 dk toleransla geçer.
- **40 °C kesmesi bilinçli bir yaklaşımdır [F1 kararı 6]:** önceki kesme 60 °C idi, ama `Ftemp(60 °C) = 0.0435` ihmal edilebilir değildir; `Ftemp(40 °C) = 0.00668`. Kesmesiz integrale göre 40 °C kesmesi eşdeğer süreyi τ = 5 sa için T₀ 95 °C'de yaklaşık %4, T₀ 80 °C'de yaklaşık %10 azaltır (60 °C kesmesi %12 ve %30 azaltırdı) **[doğrulanacak]**.
- **Aktivasyon sabiti 02 ile aynı kalır [F1 kararı 6]:** `2.39·10¹¹ · e^(−9773/T)` (02 §3'teki IBU motoru), yayımlanmış Malowicki–Shellhammer sabitiyle (`7.9·10¹¹ · e^(−11858/T)`) farklı sonuç verir (τ = 4 sa, T₀ 95 °C örneğinde bizim sabitle 32.9 dk, Malowicki–Shellhammer sabitiyle 25.3 dk; ±%25 belirsizlik). İki motor tutarlı kalsın diye sabit değiştirilmez; fark burada not olarak durur ve ölçülmüş IBU ile kıyaslanana kadar model varsayılan olmaz (basit kural varsayılandır, §3.1).
Her katkı `t + t_eş` süresiyle Tinseth'e girer. Bu, Brewtarget'ın mIBU (Hosom) yaklaşımıyla aynı fikir; yalnız soğuma eğrisi küp için yavaş.

- `T₀`: küpe dolum sıcaklığı. Varsayılan 95 °C. Whirlpool sonrası doluyorsa whirlpool bitiş sıcaklığı.
- `T_ortam`: varsayılan 20 °C.
- Model τ'ya çok duyarlı (τ 3 → 6 saat: 27 → 54 dk). Bu yüzden ilk sürümde varsayılan basit kuraldır. Model, kullanıcı kendi küpünün soğuma süresini girerse veya ölçüm biriktikçe açılır.

### 3.3 Ekran [tasarım]
- Ekipman profilinde "Soğutma: soğutmasız (küp)" seçeneği. Seçilince tarifteki IBU "no-chill düzeltmeli" yazılır ve düzeltme miktarı gösterilir (ör. "+21 IBU, kural: +20 dk").
- Acılık ayarlama (02) düzeltilmiş IBU'yu hedefler; kullanıcı isterse geç hopları otomatik azaltır.

---

## 4. Test vektörleri

`python3 -I testler/10_elle_ekstrakt_nochill_tv.py` çıktısı.

| # | Girdi | Beklenen |
|---|---|---|
| 1a | OG 1.050, FG 1.012, L 12 sa; 48 sa 1.030, 96 sa 1.018 | k 0.0218/sa, bitiş 210.8 sa, aralık 6–11. gün, kalan 2–7 gün, tutarlı |
| 1b | tek okuma 48 sa 1.030 | veri yetersiz (2 okuma gerekir) |
| 1c | 24 sa 1.046, 36 sa 1.042 | veri yetersiz (%30 tamamlanmadı) |
| 1d | 96 sa 1.018, 168 sa 1.013, 216 sa 1.013 | bitmiş görünüyor, FG 1.013 |
| 1e | 48 sa 1.030, 120 sa 1.024, 168 sa 1.024 | takılmış olabilir |
| 1f | 48 sa 1.020, 94 sa 1.019 (46 sa arayla) | k 0.0243/sa, aralık 6–10. gün, tutarsız |
| 1g | 96 sa 1.018, 168 sa 1.014, 216 sa 1.013 (1 puanlık fark, 48 sa) | bitmiş görünüyor, FG 1.013 |
| 1h | OG 1.050, FG 1.012; 48 sa 1.030, 84 sa 1.014 (SG − FG = 0.002, ağırlık 0.667); ağırlıklar w(1.030) = 1, w(1.0125) = 0.167 | k ağırlıklı 0.03540 (ağırlıksız 0.03687); bitiş 134.3 sa (ağırlıksız 129.5), aralık 3–8. gün, kalan 0–4 gün. 84. saatte 1.013 okunsaydı: 126.7 sa (ağırlıksız 109.2); yani 1 puanlık hata ağırlıklı 7.6 sa, ağırlıksız 20.3 sa kaydırır |
| 2a | PPG → puan/(kg/L) | 8.3454 |
| 2b | LME / DME / bal | 299.4 / 364.7 / 287.9 puan/(kg/L) |
| 2c | 1500 g LME; 1 lb LME | 1056.3 mL; 319.4 mL |
| 2d | 500 mL bal; 1 cup DME | 710 g; 142 g |
| 2e | 3000 g LME → DME | 2463 g |
| 2f | OG 1.050, 20 L | LME 3340 g; DME 2742 g |
| 2g | OG 1.050, 20 L, kazanda 10 L | kazan SG 1.100, 10 L su |
| 2h | 28 g %10 60 dk + 28 g %10 10 dk, 20 L | kısmi (SG 1.100) 28.1 IBU; tam hacim 44.0 IBU |
| 2j | 2g ama kazanda 1 L kayıp | kazan SG 1.1111, 11 L su |
| 2i | 10 L şıra 30 °C + 10 L su, hedef 20 °C | su 10.0 °C |
| 3a | T₀ 95, ortam 20; τ 3 / 4 / 6 sa (40 °C kesmesi) | t_eş 27.1 / 36.1 / 54.0 dk |
| 3b | τ, 40 L (τ₂₀ 5 sa) | 6.30 sa |
| 3f | Ftemp(40 °C); Ftemp(60 °C); ince adım (dt 0.01 dk), T₀ 95, τ 5 sa | 0.00668; 0.0435; 44.9 dk |
| 3c | 28 g %10 60 dk + 28 g %10 0 dk, 1.050, 20 L, kural | 32.3 → 53.6 IBU |
| 3d | aynı, model τ 5 sa (varsayılan τ₂₀), T₀ 95 | 32.3 → 64.6 IBU (t_eş 45.0 dk) |
| 3e | aynı, T₀ 80 | 32.3 → 51.3 IBU (t_eş 16.9 dk) |

## 5. Açık noktalar
- **Doğrulanacak:** gecikme fazı (12 sa), FG belirsizliği (±0.003), ekstrakt yoğunlukları ve yield değerleri, küp τ değeri.
- No-chill modeli ölçülmüş IBU verisiyle (laboratuvar analizi) karşılaştırılmadan varsayılan yapılmayacak.
- Elle okumada ilk gerçek kullanıcı partileriyle tahmin hatası ölçülecek; σ buna göre ayarlanacak.
