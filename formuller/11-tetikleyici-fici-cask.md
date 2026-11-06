# Gravite tetikleyicisi, fıçıda priming, cask: spesifikasyon

Plandaki "Eksik hesaplar" tablosunda yazılmamış kalan üç hesap:

| Hesap | Aşama |
|---|---|
| Fermantasyon adımına gravite tetikleyicisi (diacetyl rest, FG sabit, lagering) | Ev MVP |
| Fıçıda priming miktarı | Ev MVP |
| Cask (İngiltere): hedef CO₂, priming, durultucu, kondisyon süresi | İşletme çekirdek |

Referans uygulama ve test vektörleri: `testler/11_tetikleyici_fici_cask_tv.py` (`python3 -I testler/11_tetikleyici_fici_cask_tv.py`).

İşaretler: **[kesin]** tanım, stokiyometri veya fizik; **[tasarım]** bizim seçtiğimiz yöntem; **[doğrulanacak]** varsayılan değer.

Birimler: °C, L, g, saat; SG; basınç bar (gösterge). Kalıntı CO₂ ve denge basıncı 04 §3.2 ve §3.5'teki formüllerdir.

---

## 1. Fermantasyon adımına gravite tetikleyicisi

Fermantasyon profilindeki bir adım süre yerine gravite koşuluyla başlayabilir: "FG'ye 4 puan kala diacetyl rest'e geç", "FG sabitlenince cold crash". Tetiklenen adım Bugün'e görev, bildirim kutusuna hatırlatıcı olarak düşer; kullanıcı onaylamadan kontrolcüye komut gitmez (plan: Cihaz entegrasyonu ilkeleri).

### 1.1 Referans FG [tasarım]
- Varsayılan `FG_ref = FG_tarif` (maya attenüasyonu ve mash sıcaklığı düzeltmesiyle, 01).
- Fermantasyon tahmini gösteriliyorsa (07 §1.4) ve `|FG∞ − FG_tarif| > 0.004` ise `FG_ref = FG∞`. Maya beklenenden farklı attenüe ediyorsa tetik tarifin FG'sine göre ya hiç gelmez ya da erken gelir. Ekranda "tahmine göre" notu çıkar.

### 1.2 "Puan kala" koşulu [tasarım]
```
tetik: SG ≤ FG_ref + Δ
```
- **Cihaz verisinde:** saatlik medyan (07 §1.1) art arda **3 noktada** koşulu sağlamalı. Tek bir gürültülü okuma tetiklemez. Tetik zamanı, üç noktanın ilki.
- **Elle okumada:** tek okuma yeter; görev "tekrar ölçerek doğrula" notuyla gelir.
- Varsayılan Δ: lager diacetyl rest **0.004**, ale **0.002** [doğrulanacak]. Profilde değiştirilebilir. Ekranda yüzde karşılığı da yazar: `p = (OG − SG)/(OG − FG_ref)`.

### 1.3 Tetik zamanı tahmini [tasarım]
07 §1 modeli oturmuşsa (`SG(t) = FG∞ + A·e^(−k·t)`):
```
t_tetik = ln(A / (FG_ref + Δ − FG∞)) / k        (FG_ref + Δ > FG∞ ise)
```
Takvim ve Bugün "tahmini diacetyl rest: Perşembe öğleden sonra" gösterir. Az okumalı partide 10 §1 modeliyle aynı hesap gün çözünürlüğünde yapılır.

### 1.4 "FG sabit" koşulu [tasarım]
- **Cihaz:** son 48 saatin saatlik medyanlarına doğrusal eğim; en az 24 nokta. `|eğim| × 24 ≤ 0.0005` (günde yarım puandan az) ise sabit.
- **Elle:** 10 §1.5 durgunluk kuralı (48 saat arayla iki okuma, fark ≤ 0.001).
- Cold crash ve "ambalaja hazır" önerisi bu koşulu kullanır.

### 1.5 Sıcaklık rampası ve lagering [tasarım]
```
rampa süresi (saat) = |T_hedef − T_başlangıç| / hız × 24
```
- Varsayılan hız: lager soğutma **2 °C/gün** [doğrulanacak]; diacetyl rest'e ısınma ve cold crash "olabildiğince hızlı" (ekipman profilindeki soğutma hızı).
- **Lagering süresi önerisi:** her 8 OG puanı için 1 hafta, en az 2 en çok 12 hafta [doğrulanacak; yaygın kaba kural]. Yalnız öneri; kullanıcı profilde değiştirir.

---

## 2. Fıçıda priming

BeerSmith ve Brewtarget fıçı için ayrı bir kural uygulamıyor (04 §3.3: `CarbKegFactor = 0.5` tanımlı ama kullanılmıyor). "Fıçıya şişenin yarısı kadar şeker" önerisi topluluk alışkanlığı. Fiziksel olarak fıçı daha az değil, biraz **daha fazla** şeker ister, çünkü kafa boşluğundaki gaz da basınçlanır. Uygulama iki modu ayrı ayrı sunar.

### 2.1 Biradaki CO₂ [kesin]
Planda seçilen yöntem: Brewtarget stokiyometrisi + BeerSmith kalıntı CO₂'si.
```
n_bira (mol) = (hedef_vol − kalıntı_vol(T_fermantasyon_max)) × V_bira / 22.414
```

### 2.2 Kafa boşluğundaki CO₂ [kesin fizik]
Denge basıncı 04 §3.5 polinomundan (kondisyon sıcaklığında). Tablo saf CO₂ kafa boşluğu varsayar, dolayısıyla CO₂ kısmi basıncı = gösterge + 1 atm:
```
n_boşluk = P_abs × V_boşluk / (R × T_K) − n_başlangıç
n_başlangıç = 1 atm × V_boşluk / (R × T_K)   (fıçı CO₂ ile süpürüldüyse; havayla ise 0)
```

### 2.3 Şeker [kesin stokiyometri]
| Şeker | Mol CO₂ / mol şeker | Molar kütle |
|---|---|---|
| Dekstroz monohidrat (mısır şekeri) | 2 | 198.17 g/mol |
| Sakaroz | 4 | 342.30 g/mol |
| İnvert şeker | 4 | 360.31 g/mol |

`şeker (g) = (n_bira + n_boşluk) / (mol CO₂ / mol şeker) × M`

### 2.4 Modlar [tasarım]
- **Tam doğal karbonasyon:** §2.1 + §2.2. 19 L birada 2 L boşluk ≈ %11 fazla şeker (süpürülmüş fıçı).
- **Kısmi + CO₂ ile tamamla:** tam miktarın **1/3**'ü [doğrulanacak]. Kapak contasını oturtur ve oksijeni süpürür; servis sıcaklığında hedef vol için gereken CO₂ basıncı (04 §3.5) ayrıca gösterilir. Daha az tortu, daha hızlı servis.
- Kafa boşluğu ekipman profilindeki fıçı hacminden (Kaplar) ve doldurulan litreden gelir.

---

## 3. Cask (İngiltere)

Cask ale, fıçıda ikinci fermantasyonla düşük karbonasyona getirilir ve el pompasıyla servis edilir. Karbonasyon mahzende (cellar) spile ile ayarlanır.

### 3.1 Hedef CO₂ [doğrulanacak]
Varsayılan **1.1 vol**, aralık 0.9–1.4. Mahzen sıcaklığı 11–13 °C.

### 3.2 Kalan ekstrakttan gelen CO₂ [kesin: Balling]
Cask çoğu zaman FG'ye 1–2 puan kala doldurulur; kalan ekstrakt CO₂ üretir.
```
ΔAE (°P)   = Plato(SG_dolum) − Plato(FG_ref)
ΔRE (g/L)  = 0.8192 × ΔAE × 10 × SG_dolum            (gerçek ekstrakt; Balling RE = 0.1808·OE + 0.8192·AE)
CO₂ (g/L)  = ΔRE × 0.9565 / 2.0665                    (Balling: 2.0665 g ekstrakt → 0.9565 g CO₂)
CO₂ (vol)  = CO₂ (g/L) / 1.9635
```
Bir puan kalan ekstrakt yaklaşık **0.5 vol** verir; iki puan hedefin üstüne çıkar.

### 3.3 Priming [kesin stokiyometri]
```
eksik_vol = hedef − kalıntı_vol(T_fermantasyon_max) − CO₂_ekstrakttan
şeker = §2.3 ile (varsayılan invert şeker)
```
- `eksik_vol < 0` ise şeker önerilmez; "fazla CO₂: mahzende spile ile boşaltılır" notu çıkar.
- Kafa boşluğu cask'ta küçük ve servis basınçsız olduğu için §2.2 uygulanmaz.

### 3.4 Durultucu (finings) [doğrulanacak]
```
doz (mL) = V (L) × oran (mL/L)        varsayılan isinglass 2 mL/L
```
Doz ürüne ve biraya göre çok değişir. Ekran, ölçüm silindirinde durultucu testi yapılmasını önerir; test sonucu girilirse oran o biranın tarifine kaydedilir.

### 3.5 Kondisyon süresi [tasarım, doğrulanacak]
```
gün = 3 × 2^((12 − T_mahzen) / 10)
```
12 °C'de 3 gün. Süre dolunca "berraklık ve CO₂'yi kontrol et (spile)" görevi Bugün'e düşer. Ambalaj kaydında cask satırı bu tarihle "servise hazır" olur.

---

## 4. Test vektörleri

| # | Girdi | Beklenen |
|---|---|---|
| 1a | SG = 1.012 + 0.036·e^(−0.03t), Δ 0.004, saatlik | tetik 74. saat |
| 1b | Aynı eğri, model tahmini | 73.2 saat |
| 1c | 13. saatte tek nokta eşik altında, 14'te üstünde, 15–17 altında | tetik 15. saat (13 tetiklemez) |
| 1d | FG tarif 1.012, tahmin 1.018 gösteriliyor | FG_ref 1.018 (tahmin) |
| 1e | Son 48 saat eğimi −0.00024/gün | sabit |
| 1f | Son 48 saat eğimi −0.0012/gün | sabit değil |
| 1g | 15 → 2 °C, 2 °C/gün | 156 saat |
| 1h | OG 1.048; 1.072 | 7 hafta; 10 hafta |
| 2a | 2.4 vol, fermantasyon 20 °C, 19 L, boşluk 0 | dekstroz 129.2 g (şişeyle aynı) |
| 2b | Aynı, 2 L boşluk, CO₂ ile süpürülmüş | 143.8 g (+%11.3), denge 1.80 bar |
| 2c | Aynı, havayla | 152.1 g (+%17.7) |
| 2d | Kısmi 1/3, servis 4 °C | 47.9 g; CO₂ ile 0.74 bar |
| 2e | 2b, sakaroz | 124.2 g |
| 3a | Firkin 40.9 L, hedef 1.1 vol, fermantasyon 19 °C, FG'de dolum | kalıntı 0.89 vol, eksik 0.21 vol, invert 34.9 g (0.85 g/L) |
| 3b | 1 puan kala dolum | ekstrakttan 0.50 vol, şeker yok, fazla |
| 3c | 2 puan kala dolum | ekstrakttan 0.99 vol, şeker yok, fazla |
| 3e | Durultucu 2 mL/L, firkin | 82 mL |
| 3f | Mahzen 12 °C; 8 °C | 3.0 gün; 4.0 gün |

## 5. Açık noktalar
- **Doğrulanacak:** Δ varsayılanları, lager soğutma hızı, lagering kuralı, kısmi priming oranı (1/3), cask hedef vol, isinglass oranı, cask kondisyon süresi.
- Cask değerleri İngiltere'deki bir veya iki üreticiyle ve Cask Marque yönergeleriyle kontrol edilecek.
- 04 §3.5 basınç polinomunun 0 °C altında ve 3.5 vol üstünde güvenilirliği bilinmiyor; fıçı hesabında bu aralık dışı uyarı verir.
