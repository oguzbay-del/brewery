# Ev MVP ek hesapları — spesifikasyon

`06-eksik-hesaplar.md` listesindeki **Ev MVP** hesapları. Bunlar BeerSmith'ten çıkarılmadı; genel fizik, kimya ve istatistik yöntemleriyle bizim tasarımımız. Referans uygulama ve test vektörleri: `testler/07_ev_mvp_ek_tv.py` (`python3 -I testler/07_ev_mvp_ek_tv.py`, yalnız standart kütüphane).

İşaretler: **[kesin]** fizik/kimya sabiti veya tanım; **[tasarım]** bizim seçtiğimiz yöntem; **[doğrulanacak]** varsayılan değer, ölçümle veya kaynakla kontrol edilecek.

Birimler: °C, L, kg, g, dakika, saat; yoğunluk SG. Su yoğunluğu 1 kg/L kabul edilir.

**Karar (T1.12 C9): motor ham değer döndürür.** Aşağıdaki işlevler (tahmin, iyon dengesi) gösterim için yuvarlama yapmaz; sınıf ve uyarı kararları da ham değerden verilir. Yuvarlama sunumda yapılır. §7 tablosundaki değerler gösterim yuvarlamasıyla yazılmıştır.

---

## 1. Fermantasyon tahmini

Fermantasyon ekranındaki "Öne Çıkanlar" kartı ("2 gün içinde bitecek") ve uyarı kuralları bu hesabı kullanır.

### 1.1 Ön işleme [tasarım]
- Cihaz okumaları (Tilt, iSpindel, RAPT) gürültülüdür. Önce **saatlik medyan** alınır.
- Kalibrasyon (§2) ve hidrometre sıcaklık düzeltmesi (01 §9.1) medyandan önce uygulanır.
- Kuru hop, meyve ya da şeker eklemesi gibi gravite sıçratan olaylar parti günlüğünden bilinir; olaydan sonraki 6 saat atılır.

### 1.2 Model [tasarım]
Aktif fazdan sonraki SG düşüşü üstel azalma kabul edilir:

```
SG(t) = FG∞ + A · e^(−k·t)
```

- `FG∞`: tahmini son gravite.
- `k`: hız (1/saat).
- `A`: ölçek. Zaman parti başlangıcından saat cinsinden.

### 1.3 Oturtma [tasarım]
1. `FG∞` bir aralıkta ızgarayla taranır (400 adım):
   - alt sınır `FG_tarif − 0.015`,
   - üst sınır `SG_son − 0.0002`.
2. Her `FG∞` için `ln(SG − FG∞)` zamana karşı doğrusal regresyonla oturtulur → `k = −eğim`, `A = e^kesişim`.
3. Hata kareler toplamı (RSS) en küçük olan `FG∞` seçilir.

Bu yöntemin üç faydası var:
- Doğrusal olmayan bir çözücü gerektirmez.
- Rust'ta ve WebAssembly'de birebir aynı sonucu verir.
- Her zaman yakınsar.

### 1.4 Ne zaman gösterilir [tasarım]
Tahmin yalnız şu üç koşulda gösterilir:
- en az **12 saatlik nokta** var,
- SG, beklenen düşüşün en az **%30**'unu tamamlamış: `OG − SG_son ≥ 0.30 · (OG − FG_tarif)`,
- `k > 0`.

Koşullar sağlanmazsa ekranda "veri yetersiz" yazar, tahmin yapılmaz.

**Karar (T1.12 C3): gravite eşikleri tam sayı birimde.** 00-OKU-BENI kuralı 07'yi de kapsar: `u(SG) = round(SG × 100000)` (yarım çifte, Python `round`). %30 koşulu `u` ile yazılır: veri yetersizdir ⇔ `(u(OG) − u(SG_son)) < min_düşüş · (u(OG) − u(FG_tarif)) − 1e-9` (varsayılan `min_düşüş` 0.30; tam sayı biçimi `10·Δ_son < 3·Δ_tarif`). Tam %30 düşüş "yeterli" sayılır. En az nokta sayısı da tam eşittir (`n < min_n` yetersiz).

**Karar (T1.12 C5):** `min_n < 3` geçersizdir, "veri yetersiz" döner (aralık formülünde `n − 2` sıfır olamaz). Izgarada `SG − FG∞ > 0` ve `k > 0` sağlayan hiç nokta yoksa (düz ya da yükselen SG) sonuç "veri yetersiz"dir. Motor sonlu olmayan bir `k` ya da RSS üreten ızgara noktasını atlar.

Cihazı olmayan partide (yalnız elle veya kağıt okumalar) bunun yerine `10-elle-giris-ekstrakt-nochill.md` §1 kullanılır: FG tariften alınır, 2 okumayla gün aralığında tahmin verilir.

### 1.5 Çıktılar [tasarım]
- **FG:** `FG∞`.
- **Belirsizlik aralığı:** `RSS(FG∞) ≤ RSS_min · (1 + 4/(n−2))` koşulunu sağlayan `FG∞` değerlerinin en küçüğü ve en büyüğü. Yaklaşık %95 profil aralığıdır (F≈4).
- **Bitiş zamanı:** `t_bitiş = ln(A/ε) / k`, `ε = 0.0005`; yani SG'nin FG'ye 0.0005'ten fazla yaklaşmadığı an. Kalan süre `t_bitiş − t_son`.
- **Uyarı:** `|FG∞ − FG_tarif| > 0.010` ise "beklenenden farklı bitiyor" (takılma ya da enfeksiyon şüphesi). Uyarı kuralları ekranı bunu kullanır.
- **Karar (T1.12 C3, C9):** uyarı tam sayı birimde ve yuvarlanmamış `FG∞` ile verilir: `|u(FG∞) − u(FG_tarif)| > 1000`. Tam 0.010 fark uyarı vermez. FG, aralık, k, saatler yuvarlanmadan döner.

### 1.6 Bilinen sınırlar
- Üstel model, gecikme fazını ve en hızlı fermantasyon anını iyi temsil etmez. Bu yüzden §1.4'teki %30 eşiği var.
- Sıcaklık profili adımları (diacetyl rest) hızı değiştirir. Profil adımı değişince oturtma yalnız yeni adımdaki verilerle yeniden yapılabilir. Bu seçenek ilk sürümde yok.

---

## 2. Cihaz kalibrasyonu

### 2.1 iSpindel: açı → SG [tasarım]
- Kullanıcı farklı gravitelerde hidrometre ölçümü ve iSpindel açısı girer.
- Polinom en küçük karelerle oturtulur:

```
SG = c0 + c1·α + c2·α²        (varsayılan derece 2; 3 seçilebilir)
```

- **En az nokta sayısı:** derece + 2 (derece 2 için 4).
- **Karar (T1.12 C4): farklı açı.** Nokta sayısı değil **farklı açı** sayısı en az derece + 2 olmalıdır; değilse hata. Referans değerleri hep aynıysa (R² paydası sıfır) hata. Yalnız iki farklı açılı noktalar derece 2 için sahte "mükemmel" oturtma verirdi.
- **Karar (T1.12 C4) [tasarım, doğrulanacak]:** iki uyarı kodu. `aci_araligi_dar`: kalibrasyon açı aralığı (en büyük − en küçük) `< 10° − 1e-9`. `aci_aralik_disi`: SG hesaplanan açı kalibrasyon aralığının dışında (`açı < en küçük − 1e-9` ya da `açı > en büyük + 1e-9`); polinom aralığın dışında dışdeğerlemedir. 10° eşiği ve uyarı metni ölçümle doğrulanacak. Derece 3'ün kötü koşulluluğu (açı merkezleme ya da QR) T1.16'ya bırakıldı.
- **Gösterilenler:** R² ve en büyük artık. En büyük artık 0.002'yi aşarsa "noktaları kontrol et" uyarısı çıkar.
- **Referans ölçümler:** önce hidrometre sıcaklık düzeltmesinden (01 §9.1) geçer.
- Katsayılar cihaz kaydında saklanır. Cihaz firmware'inde polinom varsa o da kullanılabilir; uygulama hangisinin aktif olduğunu gösterir.

### 2.2 Tilt: sapma düzeltmesi [tasarım]
- **1 nokta:** sabit sapma `SG_düz = SG_tilt + (SG_ref − SG_tilt)`.
- **2 ve daha çok nokta:** doğrusal regresyon `SG_düz = eğim · SG_tilt + sapma`.
- **Sıcaklık:** ayrı ve sabit bir sapma (°C).

### 2.3 Genel
- Kalibrasyon bir tarihle birlikte saklanır.
- Kalibrasyondan önce alınmış okumalar yeniden hesaplanabilir. Ham okuma her zaman saklanır, düzeltilmiş değer türetilir.

---

## 3. Isınma ve soğuma süresi

Demleme günü zaman çizelgesi ve takvimde geriye planlama bu hesabı kullanır.

### 3.1 Isınma [kesin fizik, tasarım varsayılanları]

```
C   = V_su·c_su + m_tahıl·c_tahıl + m_kazan·c_çelik        (kJ/K)
t   = C · (T1 − T0) / (P · η) / 60                         (dakika)
```

| Sabit | Değer | Not |
|---|---|---|
| `c_su` | 4.186 kJ/(kg·K) | [kesin] |
| `c_tahıl` | 1.59 kJ/(kg·K) | 05'teki `O_MALT_SPECIFIC_HEAT` 0.38 cal/(g·°C) ile aynı |
| `c_çelik` | 0.50 kJ/(kg·K) | [kesin, paslanmaz çelik] |
| `η` elektrik | 0.90 | [doğrulanacak] ekipman profilinde değiştirilebilir |
| `η` gaz | 0.40 | [doğrulanacak] |

- Ekipman profiline iki alan eklenir: **ısıtıcı gücü** (kW) ve **ısıtma verimi** (η).
- Kaynamaya ısınmada `T1` rakıma göre kaynama noktasıdır (05 §6.1).

### 3.2 Kaynatma gücünden buharlaşma [kesin fizik]

```
buharlaşma (L/saat) ≈ P · η_kaynatma · 3600 / 2257
```

Ekipman profilindeki buharlaşma oranıyla karşılaştırılır. Büyük fark varsa kullanıcıya hangisinin doğru olduğu sorulur. Bu, kalibrasyona yardım eden bir kontroldür.

### 3.3 Daldırma soğutucu [tasarım]
Newton soğuma yasası:

```
T(t) = T_su + (T0 − T_su) · e^(−t/τ)
t    = τ · ln((T0 − T_su) / (T − T_su))
```

- **τ'nın kaynağı:** geçmiş partilerden. `τ = t_ölçülen / ln((T0 − T_su)/(T − T_su))` ve son 3 partinin ortalaması alınır.
- **Hacim farkı:** τ hacimle doğrusal ölçeklenir, `τ_V = τ_ref · V / V_ref`. Bu bir yaklaşım [tasarım].
- **Varsayılan:** geçmiş parti yoksa 24 L için τ = 9 dk [doğrulanacak].
- **Şebeke suyu sıcaklığı:** mevsime göre değişir. Bölge ayarından aylık tablo alınır ya da kullanıcı girer.

### 3.4 Plakalı ve ters akışlı soğutucu [tasarım]

```
t = V / debi
```

Varsayılan debi 4 L/dk [doğrulanacak]. Çıkış sıcaklığı hedefe ulaşmıyorsa debiyi düşürme önerisi verilir; ilk sürümde bu yalnız bir uyarı.

---

## 4. Ekipmana göre ölçekleme

**Karar (T1.16 C) (C12): tek açık giriş `scale_factor`.** Kural tablosunun tamamını (mash, tuz, "parti başına" misc, oran modu) uygulayan işlev `home::scale_factor` olarak açıktır; liste biçimindeki `scale_recipe` Rust'ta yoktur (her malzeme için `scale_factor` çağrısı ve çarpma yeterlidir; referansta kolaylık olarak kalır, yalnız kısmi kuralları vardır). Vektördeki `scale_recipe` vakaları Rust testinde bu şekilde sınanır.

Ölçekleme ekranı bu hesabı kullanır. Mevcut `ScaleGravity` (01 §3.7), `ScaleBitterness` (02 §6.1) ve `ScaleFromGrainWeight` (01 §3.8) üzerine kurulur.

### 4.1 Mod: "Hedef değerleri koru" [tasarım]
Önce her malzemenin miktarı çarpanla ilk tahmine ölçeklenir. `r = V1 / V0` (fermentöre giden hacim):

| Malzeme | Çarpan | Ardından |
|---|---|---|
| Mash'lenen tahıl | `r · η0 / η1` | `ScaleGravity` ile OG tam tutturulur |
| Şeker, ekstrakt (verimden bağımsız) | `r` | aynı |
| Acılık hopları (kaynatma) | `r` (ilk tahmin) | `ScaleBitterness` hedef ekipmanla IBU'yu tutturur: kaynatma SG'si, whirlpool sıcaklık düşüşü ve rakım yeni ekipmana göre |
| Aroma hopları (whirlpool, dry hop) | `r` (g/L korunur) | — |
| Tuzlar | su hacimleriyle (mash ve sparge ayrı) | ppm korunur |
| Asit | — | su ve pH modeliyle (03) yeniden hesaplanır |
| Maya | — | hücre ihtiyacı (04 §1.8) yeniden, paket sayısı yukarı yuvarlanır |
| Diğer (misc) | `r` | "parti başına" işaretliyse sabit kalır |
| Mash profili | — | `ScaleFromGrainWeight`, su/tahıl oranı korunur |

### 4.2 Mod: "Yalnız oranla çarp" [tasarım]
Her şey `r` ile çarpılır; hiçbir hedef korunmaz. Ekipman değişmiyorsa iki mod aynı sonucu verir.

### 4.3 Yuvarlama [tasarım]

| Malzeme | Adım |
|---|---|
| Malt | 10 g |
| Hop | 1 g |
| Tuz | 0.1 g |
| Asit | 0.1 mL |
| Maya | paket |

Bölge ayarı imparatorluk birimindeyse yuvarlama adımları o birimde seçilir. Yuvarlamadan sonra OG ve IBU yeniden hesaplanıp ekranda gösterilir.

**Karar (T1.12 C15): yarım yukarı.** `yuvarla(q, adım) = ⌊q/adım + 0.5 + 1e-9⌋ · adım`; tam yarım (0.125, adım 0.25) yukarı gider (0.25), çifte gitmez. 1e-9 payı `0.35/0.1 = 3.4999999999999996` gibi ikili artıkları yarım sayar. Sıfıra yuvarlanan malzeme için "en az bir adım" kuralı açık nokta (karar listesinde).

### 4.4 Kayıt
- Tarifte ölçekleme yeni bir sürüm oluşturur.
- Parti başlatmada ölçekleme yalnız o partiye uygulanır, tarif değişmez.

---

## 5. İyon dengesi ve sertlik birimleri

Su raporu ekranı bu hesabı kullanır.

### 5.1 Eşdeğer ağırlıklar [kesin]
mEq/L = mg/L ÷ eşdeğer ağırlık.

| İyon | g/eq | İyon | g/eq |
|---|---|---|---|
| Ca²⁺ | 20.039 | SO₄²⁻ | 48.031 |
| Mg²⁺ | 12.153 | Cl⁻ | 35.453 |
| Na⁺ | 22.990 | HCO₃⁻ | 61.017 |
| K⁺ | 39.098 | CO₃²⁻ | 30.004 |
| | | NO₃⁻ | 62.004 |

### 5.2 Denge [kesin tanım, tasarım eşikleri]

```
Σkatyon = Σ mg/L ÷ g/eq   (Ca, Mg, Na, K)
Σanyon  = Σ mg/L ÷ g/eq   (SO₄, Cl, HCO₃, CO₃, NO₃)
denge % = 100 · (Σkatyon − Σanyon) / (Σkatyon + Σanyon)
```

| \|denge\| | Sınıf | Ekranda |
|---|---|---|
| ≤ %5 | iyi | yeşil |
| %5–10 | kabul edilebilir | sarı |
| > %10 | kontrol et | sarı uyarı: "raporda eksik iyon olabilir (ör. K, NO₃)" |

**Karar (T1.12 C7, C9):** sınıf ham yüzdeden verilir ve sınırlar sürekli büyüklük kuralıyla yazılır: iyi ⇔ `|denge| ≤ 5 + 1e-9`, kabul ⇔ `|denge| ≤ 10 + 1e-9`, aksi halde kontrol et. Toplamlar ve yüzde yuvarlanmadan döner.

Laboratuvar standartları düşük toplamlarda daha sıkı sınır kullanır. Bu eşikler, demleme için bir girdi kontrolü olarak seçildi [tasarım].

### 5.3 Sertlik ve alkalinite [kesin]

```
Toplam sertlik (CaCO₃ olarak) = Ca · 50.04/20.039 + Mg · 50.04/12.153     (= 2.497·Ca + 4.118·Mg)
Alkalinite   (CaCO₃ olarak) = HCO₃ · 50.04/61.017                       (pH < 8.3 için)
°dH = CaCO₃ / 17.848     °fH = CaCO₃ / 10.009     mEq/L = CaCO₃ / 50.04
```

- **Ters yön:** rapor yalnız toplam sertlik ve Ca veriyorsa Mg türetilir: `Mg = (sertlik − 2.497·Ca) / 4.118`. Sonuç negatifse uyarı verilir.
- **Rapor alkaliniteyi CaCO₃ olarak veriyorsa:** HCO₃'e ters formülle çevrilir. 03'teki alkalinite hesabı bu değeri kullanır.

---

## 6. Hop ve meyve kayıpları

Hacim zinciri (05 §5.11) bu kayıpları kullanır.

### 6.1 Varsayılan emilim [doğrulanacak]

| Form | L/g |
|---|---|
| Pellet | 0.0070 |
| Yaprak | 0.0130 |
| Cryo / lupulin | 0.0045 |

Değerler ekipman profilinde değiştirilebilir. Geçmiş partilerdeki plan–gerçekleşen hacim farkından kalibrasyon önerilir. Kaynak taraması yapılıp güncellenecek.

### 6.2 Hacim zincirine eklenmesi [tasarım]

```
Kazan kaybı        += Σ (kaynatma + whirlpool hopları · emilim)
Fermentör kaybı    += Σ (dry hop · emilim) + meyve tortusu
Ambalajlanan hacim  = fermentör hacmi + meyve hacmi − fermentör trub kaybı − dry hop emilimi − meyve tortusu
```

- Mevcut zincirde kazan kaybı tek bir sabit. Hop emilimi ayrı bir kalem olur; sabit kayıp yalnız ekipmanın ölü hacmini temsil eder.
- **Geri hesaplama:** hedef ambalaj hacmi girilmişse, eklenen kayıplar kadar daha fazla şıra planlanır (05'teki geri hesaplama).

### 6.3 Meyve [tasarım]

```
eklenen hacim = kg / yoğunluk        (püre için varsayılan 1.05 kg/L) [doğrulanacak]
tortu kaybı   = eklenen hacim · 0.20 [doğrulanacak]
```

Meyvenin şeker ve asit katkısı ayrı bir maddedir ("Sonra" aşaması); burada yalnız hacim hesaplanır.

---

## 7. Test vektörleri

`python3 -I testler/07_ev_mvp_ek_tv.py` çıktısından.

| No | Girdi | Beklenen |
|---|---|---|
| 1.1 | OG 1.052, gerçek FG 1.0110, k 0.045/saat, gecikme 12 saat; 15 dk'da bir okuma, gürültü ±0.0006; 24–72. saat verisi; tarif FG 1.012 | FG 1.0109, aralık 1.0108–1.0111, k 0.0448, bitiş 110.4. saat (gerçek 109.9), kalan 38.9 saat, uyarı yok |
| 1.2 | aynı, yalnız 24–30. saat | veri yetersiz |
| 2.1 | iSpindel (25°, 1.000), (38.2°, 1.020), (49.0°, 1.040), (58.7°, 1.060), (66.5°, 1.080); derece 2 | c0 0.97735571, c1 0.00053295, c2 1.511e-05; R² 0.99981; en büyük artık 0.0007 |
| 2.2 | 2.1 ile 45° | SG 1.0319 |
| 2.3 | Tilt 1.0545, hidrometre 1.0520 | eğim 1, sapma −0.0025 |
| 2.4 | Tilt (1.0020 → 1.0000), (1.0545 → 1.0520) | eğim 0.9905, sapma 0.0075 |
| 3.1 | 15 L su 15→72 °C, kazan 8 kg çelik, 2.5 kW, η 0.90 | 28.2 dk |
| 3.2 | 15 L + 5 kg tahıl 64→72 °C, aynı ekipman | 4.4 dk |
| 3.3 | 28 L 76→100 °C | 21.5 dk |
| 3.4 | 2.5 kW, η_kaynatma 0.85 | 3.39 L/saat |
| 3.5 | 24 L, 100→20 °C, şebeke 15 °C, 25 dk | τ 8.82 dk |
| 3.6 | aynı soğutucu, 40 L, şebeke 18 °C, 100→20 °C | 54.6 dk |
| 3.7 | plakalı, 23 L, 4 L/dk | 5.75 dk |
| 4.1 | 23 L → 40 L, η %72 → %78; Pale 4.50 kg, Magnum 19 g | Pale 7.224 kg, Magnum 33.04 g (ilk tahmin), Citra whirlpool 86.96 g, gypsum 8.70 g, dekstroz 0.435 kg |
| 4.2 | yuvarlama | Pale 7.22 kg, Magnum 33 g |
| 4.3 | oran modu | Pale 7.826 kg |
| 5.1 | Ca 38, Mg 8, Na 18, SO₄ 32, Cl 40, HCO₃ 110 mg/L | katyon 3.338, anyon 3.597 mEq/L, denge −%3.75, iyi |
| 5.2 | aynı | sertlik 127.8 mg/L CaCO₃ = 7.16 °dH = 12.77 °fH = 2.55 mEq/L |
| 5.3 | aynı | alkalinite 90.2 mg/L CaCO₃ = 5.05 °dH = 1.80 mEq/L |
| 6.1 | kazanda 49 g pellet | 0.343 L |
| 6.2 | dry hop 60 g pellet + 20 g cryo | 0.510 L |
| 6.3 | 2 kg meyve püresi | eklenen 1.905 L, tortuda 0.381 L |

## 8. Açık noktalar

- **Doğrulanacak varsayılanlar:** η değerleri, varsayılan τ ve debi, hop ve meyve emilimi. İlk kullanıcı verileriyle ve kaynak taramasıyla güncellenecek.
- **Fermantasyon tahmini için alternatif:** lojistik model, gecikme fazını da kapsar. Gerçek parti verisiyle (Tilt/iSpindel kayıtları) iki model karşılaştırılacak.
- **Soğuma süresi:** τ'nın hacimle doğrusal ölçeklenmesi kaba bir yaklaşım. Farklı hacimlerde 3–4 ölçümle kontrol edilecek.
