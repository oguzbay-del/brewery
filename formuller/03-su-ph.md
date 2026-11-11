# 03 — Su Kimyası, Alkalinite, Mash pH ve Asitleme

Bu belge BeerSmith 4 (Linux) hesap motorunun su/pH bölümünü, kendi uygulamanızda yeniden yazabileceğiniz biçimde tarif eder. Formüller decompile edilmiş koddan çıkarıldı ve kendi ifademle yazıldı. Sabitler programdaki değerlerle birebir aynıdır. "Kaynak" satırı, ilgili davranışın hangi fonksiyonda bulunduğunu gösterir.

İşaretler:
- **[kesin]**: kodda doğrudan görüldü.
- **[belirsiz]**: çıkarım ya da yorum içerir; uygulamadan önce doğrulayın.
- **[tahmin]**: yayımlanmış model adı eşleştirmesi; kodda isim geçmiyor.

---

## 0. Birimler, iç temsil ve enum'lar

### 0.1 İç birimler [kesin]
| Büyüklük | İç birim | Not |
|---|---|---|
| Hacim (reçete, malzeme) | US fl oz | 1 US gal = 128 fl oz; 1 fl oz = 29.57352956 mL; dolayısıyla 1 gal = 3.78541178 L (`BConvertVolume`) |
| Tahıl ağırlığı | oz | kg'a `BConvertWeight(x, oz→kg)` ile çevrilir |
| İyon derişimi | ppm (= mg/L) | |
| Alkalinite, RA | ppm CaCO₃ cinsinden | 50 ppm CaCO₃ = 1 mEq/L |
| Mash kalınlığı `F_R_MASH_THICKNESS` | L/kg | Ana mash adımının `F_MH_THICKNESS` alanından alt-alan olarak gelir |
| Tahıl rengi `F_G_COLOR` | SRM (iç birim) **[belirsiz: iç birimin SRM olduğu varsayıldı; renk dönüşümü kodu 0x1d birimini kullanıyor]** | |
| Asit derişimi | % (ağırlıkça) | |

Birim kodları (`BConvertVolume`/`BConvertWeight`): 9 / 0x27 = fl oz, 0x0e = US gal, 0x0f = mL, 0x10 = L; ağırlıkta 7 = kg, 8 = lb, 9 = oz, 10 = g.

Misc (`F_M_UNITS`) → gram dönüşümü (`BSMisc::ToGrams`):
| kod | birim | gram karşılığı |
|---|---|---|
| 0 | mg | x / 1000 |
| 1 | g | x |
| 2 | oz | oz→g |
| 3 | lb | lb→g |
| 4 | kg | x · 1000 |
| 5 | mL | dönüşüm yok, değer olduğu gibi kalır **[belirsiz: mL değeri gram sayılıyor]**. Asitler bu birimle kaydedilir. |
| 6 | tsp | x · 0.8 · 4.93 (0.8 g/mL yoğunluk varsayımı) |
| 7 | tbsp | x · 0.8 · 14.78 |

### 0.2 Enum'lar [kesin]
**Reçete tipi `F_R_TYPE`:** 0 = Extract, 1 = Partial Mash, 2 = All Grain. Kodda "mash'li reçete" 1 veya 2 demektir.

**Misc kullanım `F_M_USE`:** 0 = Boil, 1 = Mash, 2 = Primary, 3 = Secondary, 4 = Bottling, 5 = Sparge.

**Misc tipi `F_M_TYPE`:** 5 = Water Agent.

**Su ajanı tipi `F_M_WA_TYPE`** (`BSMisc::WaterAgentType`, misc adına göre otomatik atanır):
| değer | ajan | tanınan adlar |
|---|---|---|
| 0 | tanınmadı / değil | |
| 1 | Gypsum (CaSO₄·2H₂O) | "Gypsum", "Gypsum (Calcium Sulfate)" + yerelleştirilmiş ad |
| 2 | Sofra tuzu (NaCl) | "Salt", "Table Salt", "NaCl" |
| 3 | Epsom (MgSO₄·7H₂O) | "Epsom Salt", "Epsom Salt (MgSO4)" |
| 4 | Kalsiyum klorür (CaCl₂, susuz) | "Calcium Chloride", "CaCl" |
| 5 | Kabartma tozu (NaHCO₃) | "Baking Soda" |
| 6 | Tebeşir (CaCO₃) | "Chalk" |
| 7 | Magnezyum klorür (MgCl₂·6H₂O) | "Magnesium Chloride", "MgCl2" |
| 8 | Söndürülmüş kireç (Ca(OH)₂) | "Pickling Lime" |
| 10 | Laktik asit | adında "Lactic Acid" geçen |
| 11 | Fosforik asit | adında "Phosphoric Acid" geçen |

İyon hesapları yalnızca 1–8 aralığını kullanır (`0 < tip < 9`). Asitler (10, 11) iyon profiline katılmaz, yalnızca pH hesabına girer.

**Tahıl tipi `F_G_TYPE`:** 0 = Grain, 1 = Extract, 2 = Sugar, 3 = Adjunct, 4 = Dry Extract, 5 = Fruit, 6 = Juice, 7 = Honey.

**Tahıl kullanımı `F_G_USE`:** 0 = Mash, 1 = Boil, …; pH hesabı yalnızca `F_G_USE < 1` (mash) olanları sayar.

**Asit seçimi `F_R_ACID` / `F_PH_ACID`:** 0 = None, 1 = Lactic, 2 = Phosphoric, 3 = Acid Malt.

**Asit modeli `F_R_ACID_MODEL`:** 0 = "MPH 3 Model", 1 = "BW Model" (kodda "BNW"). Reçetede ekipman profilinden gelir (`F_E_ACID_MODEL`). Liste varsayılanı **1 (BW)**. Bu son bilgi AddList'in son parametresinin varsayılan indeks olduğu varsayımına dayanır **[belirsiz]**.

---

## 1. Tuz iyon katkı matrisi

### 1.1 Tablo [kesin]
Global `a_matrix` (48 double, 8 sütun × 6 satır, sütun-öncelikli) ile `BSMisc::{Calcium,Magnesium,Sodium,Sulfate,Chloride,Bicarb}Func` içindeki sabitler birebir aynıdır.
**Birim: 1 US galon suya 1 g tuz eklendiğinde oluşan ppm (mg/L).** g/L tabanına geçmek için değeri 3.78541178 ile çarpın.

| Tuz (WA tipi) | Ca²⁺ | Mg²⁺ | Na⁺ | SO₄²⁻ | Cl⁻ | HCO₃⁻ |
|---|---|---|---|---|---|---|
| 1 Gypsum CaSO₄·2H₂O | 61.478 | 0 | 0 | 147.359 | 0 | 0 |
| 2 NaCl | 0 | 0 | 103.891 | 0 | 160.212 | 0 |
| 3 Epsom MgSO₄·7H₂O | 0 | 26.043 | 0 | 102.933 | 0 | 0 |
| 4 CaCl₂ (susuz) | 95.371 | 0 | 0 | 0 | 168.731 | 0 |
| 5 NaHCO₃ | 0 | 0 | 72.275 | 0 | 0 | 191.827 |
| 6 CaCO₃ | 105.754 | 0 | 0 | 0 | 0 | 322.016 |
| 7 MgCl₂·6H₂O | 0 | 31.573 | 0 | 0 | 92.111 | 0 |
| 8 Ca(OH)₂ | 142.856 | 0 | 0 | 0 | 0 | 434.991 |

### 1.2 Molar kütle tutarlılığı
Beklenen değer şöyle hesaplanır: `faktör = 264.172 · (iyon kütlesi × stokiyometri) / tuzun molar kütlesi` (264.172 = 1000 mg / 3.78541 L).

| Tuz | M (g/mol) | Beklenen | Programdaki | Not |
|---|---|---|---|---|
| Gypsum dihidrat 172.17 | Ca 40.078 → 61.49; SO₄ 96.06 → 147.39 | 61.478 / 147.359 | ✓ dihidrat |
| NaCl 58.44 | Na 103.92; Cl 160.26 | 103.891 / 160.212 | ✓ |
| Epsom heptahidrat 246.47 | Mg 26.05; SO₄ 102.96 | 26.043 / 102.933 | ✓ heptahidrat |
| CaCl₂ **susuz** 110.98 | Ca 95.40; Cl 168.79 | 95.371 / 168.731 | ✓ susuz. Dihidrat (147.01) olsaydı 72.0 / 127.4 çıkardı. Uygulamada kullanıcıya hangi formun varsayıldığını gösterin. |
| NaHCO₃ 84.007 | Na 72.30; HCO₃ 191.87 | 72.275 / 191.827 | ✓ |
| CaCO₃ 100.09 | Ca 105.78; **2×HCO₃** 322.1 | 105.754 / 322.016 | ✓ CO₃²⁻, 2 HCO₃⁻ olarak sayılıyor (CO₂ ile çözünme varsayımı) |
| MgCl₂·6H₂O 203.30 | Mg 31.58; Cl 92.13 | 31.573 / 92.111 | ✓ hekzahidrat |
| Ca(OH)₂ 74.09 | Ca 142.9; **2×HCO₃** 435.1 | 142.856 / 434.991 | ✓ OH⁻, HCO₃⁻ eşdeğeri olarak sayılıyor |

Küçük farklar, kullanılan atom kütlelerinin yuvarlanmasından gelir. Uygulamada tablodaki sabitleri aynen kullanın.

---

## 2. Bir su ajanının iyon katkısı (ppm)

**Ne hesaplar:** Reçetedeki tek bir tuz malzemesinin, ait olduğu su hacminde oluşturduğu iyon artışı.
**Kaynak:** `BSMisc::CalciumFunc` … `BicarbFunc` (misc şemasında `F_M_CALCIUM` … `F_M_BICARB` hesaplı alanları); hacim ataması `BSRecipe::RecipeRecalculate` içinde yapılır.

```
g      = ToGrams(F_M_AMOUNT, F_M_UNITS)
V_gal  = F_M_TEMP_VOL / 128            (fl oz → gal)
ppm_i  = (g / V_gal) · A[tip][i]       (V_gal = 0 ise 0)
```

**Referans hacim `F_M_TEMP_VOL`** [kesin]. Reçete her yeniden hesaplandığında, ajanın kullanımına göre atanır.

Mash'li reçete (tip 1 veya 2):
| F_M_USE | hacim |
|---|---|
| 1 Mash | `F_R_MASH_TOT_WATER` (toplam mash suyu) |
| 5 Sparge | `F_R_SPARGE_VOL` |
| 0 Boil | `F_R_BOIL_VOL` |
| 2 Primary | `F_R_VOLUME` (batch hacmi) |
| diğer | `F_R_FINAL_VOL` |

Extract reçete: Boil veya Mash için `F_R_BOIL_VOL`, Primary için `F_R_VOLUME`, diğerleri için `F_R_FINAL_VOL`.

`F_R_IGNORE_WATER_AGENTS` = 1 ise (varsayılan 0) tüm tuz katkıları 0 sayılır, çünkü tuzların profile zaten dahil olduğu kabul edilir.

---

## 3. Su profilleri (mash, sparge, kaynatma, fermentör)

İyon indeksleri: Ca, Mg, Na, SO₄, Cl, HCO₃.

### 3.1 Temel (base) profil [kesin]
**Kaynak:** `WaterBaseIon`, `TotalWaterIon` (ve `WaterBicarbPPM` = HCO₃ için `TotalWaterIon`).
Reçetedeki tüm su malzemelerinin (M_WATER) miktarla ağırlıklı ortalamasıdır:
```
Base_i = Σ (F_W_<iyon> · F_W_AMOUNT) / Σ F_W_AMOUNT
```
Su malzemesi yoksa ya da toplam miktar 0 ise sonuç 0 olur (saf su). Alanlar: `F_R_B_CA … F_R_B_HCO3`.

**Su pH'ı** (`WaterPhFunc`): `F_W_PH` değerlerinin aynı biçimde miktar-ağırlıklı ortalaması. Su yoksa 7.0.

### 3.2 Kullanım bazlı ayarlanmış iyon [kesin]
**Kaynak:** `WaterIonAdjusted(recipe, iyon, use, includeBase)`, `GetWaterAgentContrib`, `GetBaseWaterIon`.
```
Adj(i, use, base?) = [base? ise Base_i, değilse 0] + Σ ppm_i(ajan)
```
Toplamdaki ajanlar `F_M_TYPE` = 5, `F_M_USE` = use ve 1 ≤ `F_M_WA_TYPE` ≤ 8 koşullarını sağlayanlardır.

### 3.3 Türetilen profiller [kesin]
| Profil | Alan | Formül |
|---|---|---|
| Mash suyu | `F_R_W_CA … F_R_W_HCO3` (`WaterIon`) | `Mash_i = Adj(i, use = 1, base = evet)` |
| Sparge suyu | `F_R_S_PROFILE` (`GetSpargeWaterField`) | `Sparge_i = Adj(i, use = 5, base = evet)` |
| Mash + sparge karışımı | `GetTotalWaterIon` | Mash'li reçete ve `F_R_SPARGE_VOL` > 0 ise `(V_m·Mash_i + V_s·Sparge_i)/(V_m + V_s)`; aksi halde `Mash_i`. Burada V_m = `F_R_MASH_TOT_WATER`, V_s = `F_R_SPARGE_VOL`. |
| Kaynatma girişi | `GetTotalBoilIon` | `Boil_i = Tot_i · k_top + Adj(i, use = 0, base = hayır)`. `F_R_TOP_UP_KETTLE` ≠ 0 ise `k_top = (V_avail + V_s)/(V_avail + V_s + V_topK)`, değilse 1. V_avail = `F_R_MASH_AVAIL`, V_topK = `F_R_TOP_UP_KETTLE`. |
| Fermentöre giriş | `GetBeerWaterIon` | `Beer_i = Boil_i · S_boil + Adj(i, use = 2, base = hayır)` |

### 3.4 Kaynatma derişim faktörü `S_boil` [kesin]
**Kaynak:** `BoilIonScaleFactor`.
Girdiler: Vb = `F_R_BOIL_VOL`, Vp = `F_R_POST_BOIL_VOL`, Vc = `F_R_COOL_LOSS`, Vt = `F_R_TRUB_LOSS`, Vf = `F_R_TOP_UP` (fermentöre eklenen su).
```
Vb > 0, Vp > 0 ve F_R_HAS_BOIL_ING > 0 ise:
  a = Vp − Vc
  b = a − Vt
  S_boil = ( (Vb / a) · b + Vf ) / (b + Vf)
aksi halde S_boil = 1
```
Yorum: kaynatmadan çıkan wort (derişim faktörü Vb/a) ile fermentör ilave suyunun (faktör 1, yani o da aynı su profilinde varsayılıyor) hacim-ağırlıklı ortalaması.
**[belirsiz]** Faktör yalnızca `F_R_HAS_BOIL_ING` (reçetede kaynatma malzemesi var mı) doğruysa uygulanıyor. Bu koşulun amacı belli değil.

---

## 4. Alkalinite, sertlik, RA, SO₄/Cl, renk aralığı

Aynı formüller üç yerde var: reçete (`BSRecipe::*`, mash profili `F_R_W_*` üzerinden), su malzemesi (`BSWater::*`, `F_W_*`) ve su profili aracı (`BSTools::*`, `F_WP_TOT_*`).

### 4.1 Alkalinite [kesin]
`Alk (ppm CaCO₃) = HCO₃ · 50 / 61`
Kaynak: `AlkalinityFunc` (×3). HCO₃'ün tamamının alkalinite sayıldığı basit dönüşüm; karbonat ve pH etkisi yok.

### 4.2 Etkin sertlik (Kolbach) [kesin]
`H_eff (ppm CaCO₃) = Ca / 1.4 + Mg / 1.7`
Kaynak: `HardnessFunc` (×3). Kolbach'ın "Ca/3.5 + Mg/7" (CaCO₃ cinsinden) ifadesinin ppm iyon cinsinden yazılmış hâli (2.497/3.5 ≈ 1/1.4; 4.118/7 ≈ 1/1.7).

### 4.3 Kalıntı alkalinite (RA) [kesin], Kolbach RA
`RA = Alk − H_eff`  (ppm CaCO₃)
Reçetede `F_R_RESID_ALK` alanı **mash suyu profilinden** (`F_R_W_*`, yani base + mash tuzları) hesaplanır. Sparge tuzları ve asitler RA'yı etkilemez. Kaynak: `ResidAlkFunc` (×3).

### 4.4 Sülfat/Klorür oranı ve sınıfı [kesin]
`F_R_SULFATE_CHLORIDE = SO₄ / Cl` (mash profili). Kaynak: `SulfiteChlorideRatio` (adı yanlış yazılmış), `SulfateBalRatio` (×2).
**[belirsiz]** Derleyicinin kaldırdığı "ulaşılamaz bloklar" büyük olasılıkla Cl = 0 için bir koruma. Cl = 0 olduğunda 0 döndürmeyi önerilir.

Sınıflama (`SulfateBalFunc` ×3, eşikler hepsinde aynı):
| oran r | etiket |
|---|---|
| r < 0 veya r > 18 | Out of Range |
| 0 ≤ r < 0.4 | Extremely Malty |
| 0.4 ≤ r < 0.6 | Very Malty |
| 0.6 ≤ r < 0.8 | Malty |
| 0.8 ≤ r < 1.5 | Balanced |
| 1.5 ≤ r < 2.01 | Slightly Bitter |
| 2.01 ≤ r < 6.01 | Bitter |
| 6.01 ≤ r < 8.01 | Very Bitter |
| 8.01 ≤ r ≤ 18 | Extremely Bitter |

"Medium Bitterness" metni kaynaklarda tanımlı ama kodda hiç kullanılmıyor.

### 4.5 RA'ya göre önerilen renk aralığı [kesin], Palmer nomogramı (tahmin)
Kaynak: `BSRecipe::ColorRange` (`BSWater`/`BSTools` sürümleri de var).
```
SRM_min = max(0, 5.2 + 0.082 · RA)
SRM_max = max(0, (RA + 122.4) / 12.2)
```
Sonuç kullanıcının renk birimine çevrilir ve "min-max birim" metni olarak gösterilir.

---

## 5. Mash pH modeli

### 5.1 Tahıl (grist) pH'ı [kesin]
**Kaynak:** `GetEstGristPH` → `F_R_GRIST_PH`.
Girdiler: mash tahıl toplam ağırlığı W = `F_R_MASH_GRAIN_WEIGHT`, her tahılın ağırlığı w_j ve rengi c_j (SRM).
Toplama yalnızca `F_G_USE` = 0 (Mash) ve `F_G_TYPE` ∈ {0 Grain, 3 Adjunct} olan tahıllar girer.
```
g(c) = c − 1.9            , c < 14
     = 0.453 · c + 5.0    , 14 ≤ c < 200
     = 42.0               , c ≥ 200
S = Σ (w_j / W) · g(c_j)
pH_grist = 5.72 − 0.0337 · S
W ≤ 0 ise pH_grist = 7.0
```
Ayrıntılar:
- Paydada **tüm** mash tahıllarının ağırlığı (`F_R_MASH_GRAIN_WEIGHT`) var. Sayılmayan tipler (şeker vb.) S'ye 0 katar, yani pH 5.72 olan bir baz malt gibi davranır.
- Asit malt (yaklaşık 3 SRM) burada sıradan açık malt gibi sayılır. Asitliği ayrıca 5.4'teki terimle eklenir.
- Kristal ve kavrulmuş ayrımı renk eşikleriyle yapılır: < 14 SRM baz malt, 14–200 kristal/karamel (renkle doğrusal artan asitlik), ≥ 200 kavrulmuş (asitlik sabit 42, kristal eğrisine göre daha az asidik).
- Model adı: kodda geçmiyor. Renk tabanlı ampirik grist pH yaklaşımı, Troester'in "kristal malt asitliği renkle artar, kavrulmuş daha az" bulgusuyla uyumlu **[tahmin]**.

### 5.2 RA → pH eğimi [kesin]
**Kaynak:** `MashRASlope(model, t)`. Girdi t = `F_R_MASH_THICKNESS` (L/kg). Birim: pH / (mEq/L).
```
model 0 (MPH 3):  m = 0.037 + 0.014 · t
model 1 (BW):     m = t · 0.47930613 · 0.17 / 1.5
```
0.47930613, L/kg değerini qt/lb'ye çeviren katsayıdır (1 qt/lb = 2.0864 L/kg). Dolayısıyla BW için `m = 0.17 · t[qt/lb] / 1.5`.
Model 0'da t dönüştürülmeden L/kg olarak kullanılır **[kesin]**. Model adları: "MPH 3" ve "BW (Bru'n Water)" arayüz metinlerinden alındı. Katsayıların literatürdeki kaynağı **[belirsiz]**.

**[doğrulanacak]** Eğim `m` ve mash kalınlığı `t` örtük bir tampon kapasitesi verir: `B = t / m` (mEq/(kg·pH)). MPH için t = 3'te B ≈ 38; BW için kalınlıktan bağımsız B ≈ 18.4, yani bildiğim literatür aralığının (baz maltlar için kabaca 30–50) yaklaşık yarısı. Sonuç: T3'te BW asit önerisi MPH'nin yarısı (1.77 mL / 3.52 mL). Hangi modelin varsayılan olacağı karar listesinde (F1-kararlar); ölçülmüş mash pH verisiyle karşılaştırılana kadar bu iki modelin farkı kullanıcıya gösterilmelidir.

**Kenar durumlar:** `t ≤ 0` ya da `W ≤ 0` iken eğim `m = 0` olur ve `ΔpH / m` tanımsızdır; motor bu durumda sıfır değil "mash kalınlığı/tahıl ağırlığı eksik" hatası verir.

### 5.3 Tahmini mash pH'ı (asitsiz) [kesin]
**Kaynak:** `GetEstMashPH` → `F_R_EST_MASH_PH`.
```
pH_est = pH_grist + m · RA / 50
```
RA ppm CaCO₃ cinsindendir; /50 ile mEq/L'ye çevrilir. Kolbach RA'nın doğrusal pH kaymasına dönüştürülmesi (Troester/Kolbach tipi) **[tahmin]**.

### 5.4 Reçetedeki asitlerin RA eşdeğeri [kesin]
**Kaynak:** `MiscAcidRA(recipe, mash)`. Döndürdüğü değer mEq/L'dir ve negatiftir (asitlik).
Ortak büyüklükler: W_kg = mash tahılı (kg), t = mash kalınlığı (L/kg), mash suyu hacmi = W_kg · t (L).
Taranan malzemeler `FilterMashAgents` filtresinden geçenlerdir: WA tipi 10/11 olan misc'ler ve `F_G_IS_ACID_MALT` olan tahıllar.

**(a) Sıvı asit** (misc, WA = 10 laktik veya 11 fosforik). mash = doğru ise `F_M_USE` = 1, yanlış ise `F_M_USE` = 5 olanlar alınır.
```
p    = F_M_ACID_PCT / 100
V_L  = asit hacmi (L)                       (ToFluidOz → L)
Laktik:   MWk = 0.09009 ; ρ = 1 + 0.237·p
Fosforik: MWk = 0.098   ; ρ = 1 + 0.49·p + 0.365·p²
f    = model 1 ise AcidFraction(laktik→1, fosforik→2, pH = 5.4), değilse 1
ΔmEq/L = − p · 1000 · ρ · V_L · f / (MWk · W_kg · t)
```
p·ρ·V_L·1000 gram saf asittir. MWk'ya bölmek mmol verir, f ile çarpınca mEq, mash suyu hacmine bölünce mEq/L elde edilir.

**(b) Asit malt** (yalnızca mash = doğru, grain ve `F_G_IS_ACID_MALT`):
```
P     = F_G_ACID_PCT / 100
Wa_kg = asit malt ağırlığı (kg)
f     = model 1 ise AcidFraction(3, 5.4) (pK 3.86, laktik), değilse 1
ΔmEq/L = − P · 1000 · Wa_kg · f / (W_kg · t · 0.09009)
```
Kodda W ile (mash suyu + sparge) hacmi de okunuyor ama kullanılmıyor.

**Asit malt tanıma** (`BSGrain::IsAcidMalt`): `F_G_TYPE` = 0 (Grain) ve adında "Acid Malt" ya da "Acidulated" (veya yerelleştirilmiş karşılığı) geçmesi.

### 5.5 Asitli (ayarlanmış) mash pH'ı [kesin]
**Kaynak:** `AdjustedMashPH` → `F_R_ADJ_MASH_PH`.
```
pH_adj = pH_grist + m · ( RA/50 + MiscAcidRA(mash = doğru) )
```

---

## 6. Asit miktarı hesapları

### 6.1 Asit pKa tablosu ve ayrışma kesri [kesin]
**Kaynak:** `GetAcidPKs`, `AcidFraction(tip, pH)`.

| kod | asit | pK1 | pK2 | pK3 |
|---|---|---|---|---|
| varsayılan (0, 1 laktik, 3 asit malt, …) | laktik | 3.86 | (20) | (20) |
| 2 | fosforik | 2.12 | 7.20 | 12.44 |
| 4 | sitrik | 3.14 | 4.77 | 6.39 |
| 5 | asetik | 4.75 | (20) | (20) |
| 6 | HCl | −7.0 | (20) | (20) |
| 7 | sülfürik | −1.0 | 1.92 | (20) |
| 8 | tartarik | 2.98 | 4.34 | (20) |

pK = 20, "o proton yok" anlamına gelir (katkısı pratikte 0).
```
r_k = 10^(pH − pK_k)
D   = 1 + r1 + r1·r2 + r1·r2·r3
q(pH) = (r1 + 2·r1·r2 + 3·r1·r2·r3) / D      (molekül başına verilen ortalama proton = mEq/mmol)
```
Örnek: laktik q(5.4) = 0.97197, fosforik q(5.4) = 1.01508.

### 6.2 Litre başına asit (mash) [kesin]
**Kaynak:** `PHAcidPerLiter(mEq/L, asit, c%, model)`.
```
asit = 0              → 0
asit ∈ {1, 3} (laktik veya asit malt): MW = 90.09 ; ρ = 1 + 0.237·c/100
asit = 2 (fosforik):   MW = 98.0  ; ρ = 1 + 0.49·c/100 + 0.375·(c/100)²
f = model 1 ise AcidFraction(asit, 5.4), değilse 1
asit = 3 (asit malt):  miktar = mEq · 100 · MW / (c · f · 1000)        [g malt / L mash suyu]
diğer:                 miktar = mEq · 100 · MW / (c · f · ρ · 1000)    [mL çözelti / L mash suyu]
```
**Tutarsızlık [kesin]:** fosforik yoğunluğunun karesel katsayısı burada 0.375, `MiscAcidRA` ve sparge hesabında ise 0.365. Sonuç olarak "önerilen miktarı ekle, sonra ayarlanmış pH'ı hesapla" döngüsü fosforikte tam olarak hedefe oturmaz (fark binde birler düzeyinde). Birebir uyum istiyorsanız iki katsayıyı da koruyun.

### 6.3 Mash asidi [kesin]
**Kaynak:** `BSRecipe::PHAcidAmount(recipe, sparge = yanlış, tahminden)` (araç sürümü: `BSTools::PHAcidAmount`).
```
ΔpH = (tahminden ? F_R_EST_MASH_PH : F_R_MASH_PH) − F_R_TARGET_PH
Şu durumlarda sonuç 0: asit = 0, ΔpH < 0, ya da (asit = 3 ve sparge)
mEq/L = ΔpH / m(model, t)
miktar = PHAcidPerLiter(mEq/L, asit, F_R_ACID_CONC, model) · W_kg · t
```
Çıktı: laktik/fosforik için mL, asit malt için gram. Asit malt seçiliyse derişim alanı (`F_R_ACID_CONC`) **maltın asit yüzdesi** olarak yorumlanır. Varsayılan 88 olduğundan asit malt için kullanıcı bunu yaklaşık 2–3 yapmalıdır.

**Asitsiz tahmin ile ölçüm modları** (`DoPHAdjust`):
- "Tahminden": ölçülen mash pH'ı geçici olarak `F_R_EST_MASH_PH` yapılır, mevcut asitler (`RemoveAcids`) silinir, sonra yeni asit eklenir.
- "Ölçümden": `F_R_MASH_PH` kullanılır ve mevcut asitler silinmez; ek asit hesaplanır.
- Sıvı asit seçiliyse mash asidi `F_M_USE` = 1 olarak eklenir. Sparge hacmi > 0 ise sparge asidi (6.5) de `F_M_USE` = 5 olarak eklenir. Asitler `F_M_UNITS` = 5 (mL) ve `F_M_ACID_PCT` = `F_R_ACID_CONC` ile kaydedilir.

### 6.4 Asit malt miktarı (yinelemeli) [kesin]
**Kaynak:** `PhAcidMaltAmt`, `AddAcidMalt`.
Asit malt eklemek hem tahıl ağırlığını hem grist pH'ını değiştirdiği için yineleme gerekir.
```
reçetenin kopyası alınır, tüm asitler silinir; toplam = 0
en fazla 8 tur:
    F_R_MASH_PH = pH_adj (o anki kopyada)
    g = PHAcidAmount(ölçümden modu, mash)          // gram
    kopyaya g gram asit malt eklenir (yeni bir "Acid Malt" kaydı)
    toplam += g; yeniden hesapla
    pH_adj − hedef < 0.005 ise dur
sonuç = toplam (g)
```
Yeni asit malt kaydının varsayılanları: renk 3.0, potansiyel 1.027, nem %4, protein %6, maks %10, DP 0, `F_G_USE` = Mash, `F_G_ACID_PCT` = `F_R_ACID_CONC`.

### 6.5 Sparge suyu asidi [kesin]
**Kaynak:** `PHAcidAmount(…, sparge = doğru, …)`, `SpargeAcidConcentration`, `BNWSpargeAcidification`.

Girdiler:
- HCO₃ = base su ortalaması (`WaterBicarbPPM`; tuzlar dahil değil)
- pH_w = su pH'ı (`WaterPhFunc`)
- pH_t = `F_R_TARGET_PH` (mash hedefiyle aynı alan)
- Sparge hacmi: `V_sp [L] = (F_R_MASH_TOT_WATER + F_R_SPARGE_VOL)[L] − W_kg · t`. Bu, toplam sudan ana mash adımının suyunun çıkarılmış hâlidir; doğrudan `F_R_SPARGE_VOL` kullanılmaz **[kesin; amaç belirsiz]**.

**Model 0 (MPH): kapalı karbonat sistemi yük dengesi** [kesin] (A. J. deLange yaklaşımı **[tahmin]**). Sonuç: gerekli asit, mol(eq)/L.
```
K1 = 4.4435e-7   (pK1 ≈ 6.352)
K2 = 4.667e-11   (pK2 ≈ 10.331),  K1·K2 = 2.07378145e-17
c  = HCO₃ / 61016            (mol/L)
[H]w = 10^−pH_w ; [OH]w = 10^(pH_w−14) ; [H]t = 10^−pH_t ; [OH]t = 10^(pH_t−14)
[CO₃]w = K2 · c / [H]w
C_T = c · ([H]w² + K1·[H]w + K1K2) / (K1·[H]w)                (toplam karbonat, mol/L)
Z_t = C_T · (K1·[H]t + 2·K1K2) / ([H]t² + K1·[H]t + K1K2)      (pH_t'deki karbonat yükü)
Asit (mol/L) = ([H]t − [H]w) + ([OH]w − [OH]t) + (c + 2·[CO₃]w) − Z_t
```
Kod aynı ifadeyi 10^pH çarpanlarıyla yazar (Z_t'deki x = 10^pH_t biçimi); yukarıdaki form matematiksel olarak eşdeğerdir.

**Model 1 (BW): Bru'n Water tipi alkalinite titrasyonu** [kesin]
```
α0(pH) = 1 / (1 + r1 + r1·r2),  α2(pH) = r1·r2 / (1 + r1 + r1·r2)
          r1 = 10^(pH − 6.38),  r2 = 10^(pH − 10.373)
Alk_mEq = (HCO₃ · 50/61) / 50
C_T (mmol/L) = Alk_mEq / [ (α0(4.3) − α0(pH_w)) + (α2(pH_w) − α2(4.3)) ]
ΔZ = (α2(pH_w) − α2(pH_t)) + (α0(pH_t) − α0(pH_w))
Asit (mol/L) = C_T · ΔZ / AcidFraction(F_R_ACID, pH_t) / 1000
```
Burada alkalinite pH 4.3 son noktasına göre tanımlıdır (10^−2.08 = 10^(4.3−6.38), 10^−6.073 = 10^(4.3−10.373)). [H⁺]/[OH⁻] terimleri yoktur.

**Hacme çevirme** (iki model için ortak):
```
c = max(F_R_ACID_CONC, 1) / 100
Laktik:   mL = (0.09009 / c / (1 + 0.237·c))              · Asit(mol/L) · V_sp · 1000
Fosforik: mL = (0.098   / c / (1 + 0.49·c + 0.365·c²))   · Asit(mol/L) · V_sp · 1000
Asit malt (3) veya diğer: 0
Sonuç: laktik/fosforik sıfır ya da negatifse 0.
```
**Kenar durum (model 1):** su pH'ı 4.3 ise (alkalinite son noktası) C_T paydası 0 olur; `pH_w ≤ 4.3` için hesap yapılmaz ("sparge suyu zaten asidik, asit gerekmez"). `pH_w ≤ pH_t` ise asit 0 (aşağıdaki "sıfır ya da negatif" kuralı).
Kural 6.3'teki gibidir: ΔpH (mash pH − hedef) < 0 ya da asit = 0 ise sparge sonucu da 0 olur. Bu koşul sparge suyuna değil mash pH'ına bakar **[kesin]**.

### 6.6 Görüntüleme [kesin]
`AcidAmountString`: sıvı asit için `mL` ve hesap 48 tbsp'den küçükse "(x tbsp)" eklenir (1 tbsp = 1/256 gal). Asit malt kullanıcının ağırlık biriminde gösterilir. Asit yoksa " - ".

### 6.7 pH aracı varsayılanları (BSTools Mash pH Tool) [kesin]
| Alan | Varsayılan |
|---|---|
| `F_PH_PH_MEAS` ölçülen mash pH | 5.6 |
| `F_PH_TARGET` | 5.3 |
| `F_PH_VOLUME` mash hacmi | 384 fl oz (3 gal) |
| `F_PH_SPARGE_VOL` | 384 fl oz |
| `F_PH_GRAIN_WT` | 160 oz (10 lb) |
| `F_PH_BICARB` | 0 ppm |
| `F_PH_SPARGE` sparge suyu pH'ı | 7.5 |
| `F_PH_CONCENTRATION` | %88 |
| `F_PH_MODEL` | 1 (BW) |

Araçta mash kalınlığı `t = V_mash[L] / W[kg]` ve sparge hacmi doğrudan `F_PH_SPARGE_VOL` olarak alınır. Formüller reçete sürümüyle aynıdır.

---

## 7. Su profili eşleştirme (en iyi tuz eklemeleri)

**Kaynak:** `CalcWaterMatch`, `CalcIdealWaterProfile`, `nnls` (+ `g1`, `h12`), `UpdateWaterSalts`, `SetWaterAddition`, `CalcBestWaterAdditions`, `BestFitWaterProfile`, `UpdateSpargeSalts`, `BSTools::WPCalcAdditions`.

### 7.1 Amaç fonksiyonu [kesin]
Hedef profil T = (`F_R_TGT_CA … F_R_TGT_HCO3`, varsayılan hepsi 0).
```
Mash'li reçetede s = 1.
Extract reçetede ve F_R_HAS_BOIL_ING ≠ 0 ise s = S_boil (bkz. 3.4).
b = T − s · Base                      (6 iyonluk vektör, ppm; negatif bileşen olabilir)
min ‖ A·x − b ‖₂   koşul: x ≥ 0
```
- A, 6×n boyutlu matristir (bölüm 1.1 tablosunun sütunları). x, g/US gal cinsinden tuz derişimleridir.
- **Ağırlık yok.** Altı iyonun hepsi ppm cinsinden eşit ağırlıklıdır. Bu yüzden büyük değerli iyonlar (SO₄, HCO₃) uyumu daha çok belirler.
- Çözücü: Lawson–Hanson NNLS (en fazla 3n iterasyon, standart `nnls.c`; `g1` Givens dönüşümünü, `h12` Householder dönüşümünü kurar).
- **Hariç tutulan tuzlar** sütun olarak çıkarılır, sonra sonuçta 0'a sabitlenir:
  - `F_R_EXCLUDE_CHALK` (sütun 6, tebeşir)
  - `F_R_EXCLUDE_MGCL` (sütun 7, MgCl₂)
  - `F_R_EXCLUDE_LIME` (sütun 8, kireç)
  Üçünün de varsayılanı 0'dır. Gypsum, NaCl, Epsom, CaCl₂ ve NaHCO₃ her zaman katılır.
- Çözüm tek olmayabilir. A·x (sonuç profili) tektir ama gram dağılımı NNLS uygulamasına göre değişebilir. Birebir aynı sonuç için Lawson–Hanson'ı aynı sütun sırasıyla uygulayın.

### 7.2 Gram'a çevirme ve reçeteye ekleme [kesin]
```
V_mash_gal = F_R_MASH_TOT_WATER (gal)      ; mash'siz reçetede F_R_VOLUME
V_sp_gal   = F_R_SPARGE_VOL (gal)          ; mash'siz reçetede 0
F_R_COMBINE_SALTS = 1 ise (ve sparge > 0):  V_mash_gal += V_sp_gal, sparge eklemesi yapılmaz
gram_mash[k]   = x_k · V_mash_gal
gram_sparge[k] = x_k · V_sp_gal            (mash'li reçete, sparge > 0, birleştirme kapalı)
```
Mash ve sparge suyuna **aynı derişim** uygulanır, yani her iki su da hedef profile getirilir.
- `SetWaterAddition`: 0.01 g'dan az miktarlar reçeteden silinir. Yeni kayıtlar `F_M_TYPE` = 5, `F_M_UNITS` = 1 (g), `F_M_USE` = 1 (mash), 5 (sparge) ya da 2 (mash'siz reçete) olarak eklenir.
- `F_R_HOLD_SPARGE_SALTS` = 1 ise sparge tuzlarının kullanımı 0 (Boil) olarak işaretlenir. Böylece kaynatmaya eklenmiş sayılırlar ve sparge yerine boil profiline girerler. 0'a geri alınırsa tekrar 5 olurlar.
- Metin çıktısı (`CalcBestWaterAdditions`): her tuz için `%4.1f g`.

### 7.3 "Best fit" profil gösterimi [kesin]
`BestFitWaterProfile`: `Profil_i = s·Base_i + Σ_k x_k · A[k][i]` (ppm), hacimden bağımsızdır.

### 7.4 Su profili aracı (WPCalcAdditions) [kesin]
Base suyu ve seyreltme suyu hacimle karıştırılır: `Mix_i = (V_b·B_i + V_d·D_i)/(V_b + V_d)`. Ardından `b = T − Mix` ile aynı NNLS çözülür.
**Tuhaflık:** gram değerleri `x · V_b` (yalnızca base hacmi, gal) ile hesaplanır, toplam hacimle değil **[kesin; muhtemelen hata]**.

---

## 8. İlgili alan varsayılanları [kesin]

| Alan | Varsayılan | Açıklama |
|---|---|---|
| `F_R_MASH_PH` | 5.2 | ölçülen mash pH |
| `F_R_TARGET_PH` | 5.2 | hedef pH (mash ve sparge) |
| `F_R_ACID` | 1 (Lactic) **[belirsiz]** | asit seçimi |
| `F_R_ACID_CONC` | %88 | asit derişimi; asit malt seçiliyse maltın asit % değeri |
| `F_E_ACID_MODEL` / `F_R_ACID_MODEL` | 1 (BW) **[belirsiz]** | model |
| `F_R_RUNOFF_PH` | 6.0 | yalnızca kayıt |
| `F_R_IGNORE_WATER_AGENTS` | 0 | |
| `F_R_HOLD_SPARGE_SALTS` | 0 | |
| `F_R_EXCLUDE_CHALK / LIME / MGCL` | 0 | |
| `F_R_COMBINE_SALTS` | 0 | |
| `F_R_AUTO_MATCH_WATER`, `F_R_AUTO_MATCH_PH` | 0 | otomatik eşleştirme bayrakları |
| `F_R_TGT_*` | 0 | |
| `F_G_ACID_PCT`, `F_M_ACID_PCT` | 0 | asit malt/asit eklenirken `F_R_ACID_CONC` kopyalanır |

Resources.sqlite `Option` tablosunda su/pH ile ilgili global bir ayar yok. Yalnızca `Units.Water` = 10 (g) var. Tüm parametreler reçete ve ekipman alanlarındadır.

---

## 9. Belirsizlikler ve bilinen tuhaflıklar
1. Fosforik yoğunluk katsayısı: `PHAcidPerLiter` içinde 0.375, diğer yerlerde 0.365 (6.2).
2. Sparge asidi hacmi `F_R_SPARGE_VOL` değil, "toplam su − W·t" (6.5). Sparge asidinin hesaplanıp hesaplanmayacağı mash ΔpH'a bağlı.
3. Sparge asidinde HCO₃ ve pH yalnızca base sudan alınır. Sparge tuzları (örneğin kabartma tozu) dikkate alınmaz.
4. `S_boil` yalnızca `F_R_HAS_BOIL_ING` > 0 olduğunda uygulanır. Fermentör ilave suyunun profili kaynatma girişi suyuyla aynı varsayılır.
5. `ToGrams`, mL birimini çevirmeden gram sayar (yalnızca su tuzu katkısını etkiler; asitler iyon hesabına girmez).
6. Su profili aracı gramları yalnızca base hacmiyle ölçekler (7.4).
7. Model adlarının literatürdeki karşılıkları (MPH 3, BW) ve katsayıların kaynağı kodda yok. Grist modeli **[tahmin]**.
8. SO₄/Cl için Cl = 0 koruması derleyicinin kaldırdığı blokta; döndürülen değer bilinmiyor.
9. Tahıl rengi iç biriminin SRM olduğu varsayıldı.
10. Grist fonksiyonu `g(c)` 14 SRM'de süreksizdir (13.99 SRM → 12.09, 14 SRM → 11.34). Birebir uyum için korunur; kendi modelimizde sürekli yapılması karar listesinde (F1-kararlar).
11. Mash asidinde ayrışma kesri hedef pH yerine sabit 5.4'te alınır; model 0 sparge hesabı ise laktik asidin %97 ayrışmasını hesaba katmaz (yaklaşık %3 eksik asit). Sparge hedefi mash hedefiyle aynı alandır: 5.4 için 2.86 mL, yaygın 6.0 hedefi için 2.15 mL (T4 koşullarında). Tebeşirin tam çözündüğü varsayımı alkaliniteyi olduğundan büyük gösterebilir.

---

## 10. Test vektörleri (Python ile hesaplandı)

Bu değerler bölüm 1–7'deki formüllerin doğrudan Python uygulamasıyla üretildi (`1 gal = 3.785411784 L`).

### T1: 20 L suya 5 g gypsum + 3 g CaCl₂ (susuz)
V = 20 / 3.785411784 = 5.28344 gal.
| iyon | katkı (ppm) |
|---|---|
| Ca | **112.33** (= (5·61.478 + 3·95.371)/5.28344) |
| Mg | 0 |
| Na | 0 |
| SO₄ | **139.45** |
| Cl | **95.81** |
| HCO₃ | 0 |

### T2: Alkalinite / sertlik / RA
Su: Ca 50, Mg 10, HCO₃ 150 ppm.
- Alk = 150·50/61 = **122.951** ppm CaCO₃
- H_eff = 50/1.4 + 10/1.7 = **41.597**
- RA = **81.354** ppm CaCO₃

### T3: Örnek tahıl faturası için mash pH ve asit
Not: aşağıdaki tuz katkısı T1'deki ppm değerleridir (20 L için hesaplanmış); mash suyu 14.7 L olduğundan aynı ppm'i 14.7 L'de veren miktar 5·14.7/20 = 3.675 g gypsum ve 2.205 g CaCl₂'dir. Örnekte ppm doğrudan mash profiline eklenmiştir; formül sonuçları bundan etkilenmez.

Tahıl (hepsi Grain, Mash): 4.5 kg Pale (2 SRM), 0.3 kg Crystal (60 SRM), 0.1 kg Chocolate (350 SRM). W = 4.9 kg, t = 3.0 L/kg (mash suyu 14.7 L).
- S = 4.5/4.9·0.1 + 0.3/4.9·(0.453·60+5) + 0.1/4.9·42 = **2.91918**
- pH_grist = 5.72 − 0.0337·S = **5.62162**

Mash suyu: Base (Ca 50, Mg 10, Na 15, SO₄ 40, Cl 30, HCO₃ 150) + T1'deki tuzlar.
- Profil: Ca 162.33, Mg 10, Na 15, SO₄ 179.45, Cl 125.81, HCO₃ 150
- RA = **1.1166** ppm CaCO₃
- SO₄/Cl = **1.426** ("Balanced")

| | Model 0 (MPH 3) | Model 1 (BW) |
|---|---|---|
| eğim m | 0.037 + 0.014·3 = **0.07900** | 3·0.47930613·0.17/1.5 = **0.162964** |
| pH_est | **5.6234** | **5.6253** |
| 5.4'e inmek için mEq/L | 2.82772 | 1.38226 |
| q(laktik, 5.4) | 1 | 0.971968 |
| Laktik %88, mL/L | 0.239530 | 0.120469 |
| **Laktik %88, toplam (14.7 L)** | **3.521 mL** | **1.771 mL** |
| Kontrol: bu asitle `AdjustedMashPH` | 5.4000 | 5.4000 |

Asit malt (%3, `F_R_ACID_CONC` = 3), yinelemeli `PhAcidMaltAmt` (eklenen malt 3 SRM, W'ya eklenir):
- Model 0: 1. tur 124.83 g (pH 5.4071), 2. tur +4.05 g → **toplam 128.88 g**, pH_adj 5.4002
- Model 1: 1. tur **62.78 g**, pH_adj 5.4036 (< 0.005 olduğu için durur)

### T4: Sparge asidi
HCO₃ = 150 ppm, su pH 7.5, hedef 5.4, laktik %88. Toplam su 30 L, W = 4.9 kg, t = 3 → V_sp = 30 − 14.7 = **15.3 L**.
- Model 0 (karbonat dengesi): 0.00220511 mol/L → **2.858 mL**
- Model 1 (BW): 0.00229262 mol/L → **2.971 mL**

### T5: Renk aralığı (RA = 1.1166)
SRM_min = **5.29**, SRM_max = **10.12**

### T6: Kaynatma derişim faktörü
Vb = 28, Vp = 24, soğuma kaybı 1, trub 1, ilave su 0 (L) → S_boil = (28/23)·22/22 = **1.21739**

### T7: Profil eşleştirme (NNLS)
Base: Ca 50, Mg 10, Na 15, SO₄ 40, Cl 30, HCO₃ 150. Hedef: Ca 100, Mg 15, Na 20, SO₄ 250, Cl 60, HCO₃ 150.
- Tüm tuzlar açıkken: ‖r‖ = **18.8197**. Sonuç profili (tek): Ca 108.4, Mg 28.9, Na 27.4, SO₄ 246.5, Cl 55.2, HCO₃ 150.0.
  Bir çözüm (scipy nnls, g/gal): gypsum 0.9506, NaCl 0.1189, epsom 0.6451, MgCl₂ 0.0670. 20 L için: 5.02 g, 0.63 g, 3.41 g, 0.35 g.
- Tebeşir, kireç ve MgCl₂ hariç: ‖r‖ = 18.8197, profil aynı. Gram dağılımı (20 L): gypsum 4.72, NaCl 0.63, epsom 3.84, CaCl₂ 0.19. Çözümün tek olmadığını gösterir.

Kontrol için profil vektörünü ve ‖r‖ değerini karşılaştırın. Gram dağılımı NNLS sürümüne göre değişebilir.

### Referans Python
```python
GAL_L = 128*29.57352956/1000
A = {'gypsum':[61.478,0,0,147.359,0,0],'nacl':[0,0,103.891,0,160.212,0],
     'epsom':[0,26.043,0,102.933,0,0],'cacl2':[95.371,0,0,0,168.731,0],
     'soda':[0,0,72.275,0,0,191.827],'chalk':[105.754,0,0,0,0,322.016],
     'mgcl2':[0,31.573,0,0,92.111,0],'lime':[142.856,0,0,0,0,434.991]}
def grist_ph(bill):                       # [(kg, SRM)]
    W=sum(w for w,_ in bill)
    g=lambda c: c-1.9 if c<14 else (0.453*c+5.0 if c<200 else 42.0)
    return 5.72-0.0337*sum(w/W*g(c) for w,c in bill)
def slope(model,t): return t*0.47930613*0.17/1.5 if model==1 else 0.037+0.014*t
def acid_frac(pk,ph):
    r=[10**(ph-k) for k in pk]+[0,0]; D=1+r[0]+r[0]*r[1]+r[0]*r[1]*r[2]
    return (r[0]+2*r[0]*r[1]+3*r[0]*r[1]*r[2])/D
```
