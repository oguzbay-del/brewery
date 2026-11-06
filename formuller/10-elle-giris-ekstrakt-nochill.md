# Elle giriş tahmini, ekstrakt araçları, soğutmasız IBU: spesifikasyon

Planın "Ek özellikler" tablosundaki üç Ev MVP satırının hesapları:
- az okumayla fermantasyon tahmini,
- ekstrakt biracısı araçları,
- soğutmasız (no-chill) IBU ayarı.

Bunlar BeerSmith'ten çıkarılmadı; bizim tasarımımız. Referans uygulama ve test vektörleri: `testler/10_elle_ekstrakt_nochill_tv.py` (`python3 -I testler/10_elle_ekstrakt_nochill_tv.py`).

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
Her okuma için `x = t − L`, `y = ln((SG − FG)/(OG − FG))`. Orijinden geçen en küçük kareler:
```
k = −Σ(x·y) / Σ(x²)
```
- `SG − FG ≤ 0.0005` olan okuma kullanılmaz (log tanımsızlaşır).
- Çözücü gerekmez, her zaman tek sonuç verir.

### 1.4 Ne zaman gösterilir [tasarım]
Dört koşul birlikte:
- gecikme fazından sonra en az **2 okuma**,
- son okuma beklenen düşüşün en az **%30**'unu tamamlamış (07 §1.4 ile aynı eşik),
- `k > 0`,
- durgunluk kuralı (§1.5) devreye girmemiş.

Koşul sağlanmazsa "veri yetersiz" yazılır ve hatırlatıcı bir sonraki okumayı ister (plan: Bildirim yapısı › Hatırlatıcılar).

### 1.5 Durgunluk kuralı [tasarım]
Son iki okuma en az **48 saat** arayla alınmış ve farkı **≤ 0.001** ise:
- `SG_son ≤ FG + 0.004` → "bitmiş görünüyor". Ambalaj adımı önerilir.
- değilse → "takılmış olabilir" uyarısı. Uyarı kuralları ekranındaki "takılma" kuralını tetikler.

Bu, ev biracılarının bilinen "iki üç gün aynı okuma" kuralının sayısal hali.

### 1.6 Çıktılar [tasarım]
- **Bitiş zamanı:** `t_bitiş = L + ln((OG − FG)/ε)/k`, `ε = 0.0005` (07 §1.5 ile aynı).
- **Aralık:** FG ön tahmini `±σ` kaydırılıp (`σ = 0.003` [doğrulanacak]) `k` yeniden oturtulur. Üç bitiş zamanının en küçüğü ve en büyüğü alınır.
- **Gösterim:** gün çözünürlüğünde aralık ("2–7 gün içinde") ve "yaklaşık" etiketi. Saat verilmez. Elle okumada saat hassasiyeti yanıltıcıdır.
- **Tutarsızlık:** okumaların tek tek verdiği `k` değerlerinin oranı 2'den büyükse "okumalar birbiriyle uyuşmuyor" notu çıkar (yanlış okuma, sıcaklık düzeltmesi unutulmuş ya da profil adımı değişmiş olabilir).
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
puan_kazan = puan_son · V_son / V_k
eklenecek su = V_son − V_k − kayıplar
```
- **IBU:** Tinseth kullanımı **kazandaki gravitede** hesaplanır, mg/L **son hacimde**: `IBU = U(SG_kazan, t) · AA · g · 1000 / V_son`. Yüksek kazan gravitesi kullanımı düşürür. Örnek (§4): aynı hop, kısmi kaynatmada 28 IBU, tam hacimde 44 IBU.
- **Geç ekstrakt:** Ekstraktın bir kısmı kaynatma sonunda eklenirse kazan gravitesi zamana göre değişir. 02'deki geç ekstrakt SG segmentleri kullanılır.
- **Renk:** Morey son hacimle (değişiklik yok). Kısmi kaynatmadaki ek koyulaşma modellenmez.
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
τ = τ₂₀ · (V/20)^(1/3)          τ₂₀ = 4 saat [doğrulanacak]
```
Eşdeğer kaynatma dakikası, sıcaklığa bağlı göreli hızın (02 §3, Malowicki–Shellhammer) integralidir:
```
t_eş = ∫ Ftemp(T(t)) dt         (T < 60 °C katkısı ihmal)
Ftemp(T) = 2.39·10¹¹ · e^(−9773/(T + 273.15))
```
Her katkı `t + t_eş` süresiyle Tinseth'e girer. Bu, Brewtarget'ın mIBU (Hosom) yaklaşımıyla aynı fikir; yalnız soğuma eğrisi küp için yavaş.

- `T₀`: küpe dolum sıcaklığı. Varsayılan 95 °C. Whirlpool sonrası doluyorsa whirlpool bitiş sıcaklığı.
- `T_ortam`: varsayılan 20 °C.
- Model τ'ya çok duyarlı (τ 3 → 6 saat: 25 → 50 dk). Bu yüzden ilk sürümde varsayılan basit kuraldır. Model, kullanıcı kendi küpünün soğuma süresini girerse veya ölçüm biriktikçe açılır.

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
| 1f | 48 sa 1.020, 96 sa 1.019 | k 0.0237/sa, aralık 6–10. gün, tutarsız |
| 2a | PPG → puan/(kg/L) | 8.3454 |
| 2b | LME / DME / bal | 299.4 / 364.7 / 287.9 puan/(kg/L) |
| 2c | 1500 g LME; 1 lb LME | 1056.3 mL; 319.4 mL |
| 2d | 500 mL bal; 1 cup DME | 710 g; 142 g |
| 2e | 3000 g LME → DME | 2463 g |
| 2f | OG 1.050, 20 L | LME 3340 g; DME 2742 g |
| 2g | OG 1.050, 20 L, kazanda 10 L | kazan SG 1.100, 10 L su |
| 2h | 28 g %10 60 dk + 28 g %10 10 dk, 20 L | kısmi (SG 1.100) 28.1 IBU; tam hacim 44.0 IBU |
| 2i | 10 L şıra 30 °C + 10 L su, hedef 20 °C | su 10.0 °C |
| 3a | T₀ 95, ortam 20; τ 3 / 4 / 6 sa | t_eş 24.8 / 33.1 / 49.5 dk |
| 3b | τ, 40 L | 5.04 sa |
| 3c | 28 g %10 60 dk + 28 g %10 0 dk, 1.050, 20 L, kural | 32.3 → 53.6 IBU |
| 3d | aynı, model τ 4 sa | 32.3 → 60.7 IBU (t_eş 33.1 dk) |
| 3e | aynı, T₀ 80 | 32.3 → 45.6 IBU (t_eş 10.5 dk) |

## 5. Açık noktalar
- **Doğrulanacak:** gecikme fazı (12 sa), FG belirsizliği (±0.003), ekstrakt yoğunlukları ve yield değerleri, küp τ değeri.
- No-chill modeli ölçülmüş IBU verisiyle (laboratuvar analizi) karşılaştırılmadan varsayılan yapılmayacak.
- Elle okumada ilk gerçek kullanıcı partileriyle tahmin hatası ölçülecek; σ buna göre ayarlanacak.
