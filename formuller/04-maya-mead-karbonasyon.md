# 04 – Maya, Mead/Şarap ve Karbonasyon Hesap Motoru (BeerSmith 4, Linux)

Bu belge BeerSmith 4'ün maya (canlılık, hücre sayısı, starter), mead ve şarap (TOSNA besin programı, YAN, sülfit, sorbat, tatlandırma) ve karbonasyon (şeker, basınç) hesaplarını uygulanabilir formüller olarak tanımlar. Kaynak, decompile edilmiş fonksiyonlar ve gerektiğinde x86 assembly'dir. Bütün sabitler ikilideki `double` değerleriyle karşılaştırılıp doğrulandı.

İşaretler: **[kesin]** kodda doğrudan görüldü. **[belirsiz]** davranış kodda görülüyor ama amacı ya da bağlamı tam net değil. **[tahmin]** yayımlanmış model adı ya da yorum bize ait.

---

## 0. Ortak altyapı

### 0.1 İç birimler (BeerSmith'in "base" birimleri)
BeerSmith alanları US birimlerinde saklar. Dönüşümler `BConvertVolume`, `BConvertWeight` ve `BConvertHydrometer` ile yapılır:

| Kod | Birim | Not |
|---|---|---|
| hacim 9 / 0x27 | US fl oz | 1 fl oz = 29.57352956 ml; 128 fl oz = 1 gal |
| hacim 0xE | US gal | |
| hacim 0xF | ml | |
| hacim 0x10 | L | |
| ağırlık 9 | oz | 1 lb = 0.4535924 kg, 1 oz = 28.349525 g |
| ağırlık 10 | g | |
| ağırlık 7 | kg | |
| sıcaklık (base) | °F | Karbonasyon fonksiyonları °F ile çalışır |
| basınç 5 | psi | |
| karbonasyon 0x30 / 0x31 | vol CO2 / g/L | bkz. §3.7 |
| hidrometre 3 / 4 / 0x1F | SG / °Plato / °Brix | |

Uygulamada girdiler SI olabilir, ama formüller aşağıda belirtilen birimlerle beslenmelidir.

### 0.2 SG → °Plato (`ToPlato`) [kesin]
```
P(SG) = 135.997·SG³ − 630.272·SG² + 1111.14·SG − 616.868
```
Ters dönüşüm `ToSG(P) = P / (258.6 − (P/258.6)·227.1) + 1`. Brix→SG varyantında ikinci 258.6 yerine **258.2** kullanılıyor; bu kodda gerçekten böyle.

---

## 1. Maya

### 1.1 Enum'lar [kesin]
| Alan | Değerler |
|---|---|
| `F_Y_FORM` (malzeme) | 0 Liquid, 1 Dry, 2 Slant, 3 Culture. Viabilite hesabında 3 = "slurry" gibi davranır |
| `F_YC_YEAST_FORM` (starter aracı) | 0 Liquid, 1 Dry, 2 Slurry |
| `F_YC_TYPE` (starter aracı) | 0 Ale or Cider, 1 Lager, 2 Hybrid/Wheat, 3 Mead or Wine |
| `F_Y_TYPE` | 0 Ale, 1 Lager, 2 Wine, 3 Champagne, 4 Wheat |
| `F_R_TYPE` (reçete) | 0 Extract, 1 Partial Mash, 2 All Grain, 3 Cider, 4 Mead, 5 Wine, 6 Seltzer (7 kodda "şarap tipi" kabul ediliyor ama listede görünmüyor) |
| `F_R_ALE_TYPE` | 0 Ale, 1 Lager, 2 Hybrid (mixed) |
| `F_Y_USE_PKGS` | 0 = miktar hacim/ağırlık; 1 = paket sayısı (birim 0x29 "pkgs") |

### 1.2 Ayarlar (Option, varsayılan) [kesin]
| Option | Varsayılan | Anlam |
|---|---|---|
| O_Y_DRY_VIABILITY | 98 % | Yeni kuru mayanın canlılığı |
| O_Y_LIQUID_VIABILITY | 97 % | Yeni sıvı maya |
| O_Y_SLURRY_VIABILITY | 96 % | Yeni slurry |
| O_Y_DRY_AGING | 1.66 %/ay | Kuru maya kaybı |
| O_Y_LIQUID_AGING | 10 %/ay | Sıvı maya kaybı |
| O_Y_SLURRY_AGING | 25 %/ay | Slurry kaybı |
| O_Y_ALE_PITCH | 0.75 M hücre/ml·°P | Ale hedefi |
| O_Y_LAGER_PITCH | 1.5 | Lager hedefi |
| O_Y_MIX_PITCH | 1.0 | Hybrid hedefi |
| O_Y_WINE_PITCH | 0.5 | Şarap/mead hedefi |
| O_Y_GROWTH_STARTER | 4 | Bir starter'da izin verilen azami büyüme katsayısı |
| O_Y_STIR_PCT | 45 % | Stir plate büyüme artışı |
| O_Y_DRY_CELLS_PER_GRAM | 18 milyar/g | Kuru maya hücre yoğunluğu (yalnız "paket" modunda kullanılır, bkz. 1.10) |
| O_Y_DRY_HYDRATE | 25 ml/g | Hidrasyon suyu |
| O_Y_GOFERM | 0.05 g/ml | Hidrasyon suyu başına GoFerm (25 × 0.05 = 1.25 g GoFerm / g maya) |
| O_Y_SLURRY_CONC (2.4 milyar/L), O_Y_SLURRY_NON_YEAST (15 %), O_Y_STIR_PLATE_FACTOR (2.66) | – | İncelenen hesaplarda kullanılmıyor (eski modelden kalma) [belirsiz] |

### 1.3 Yaş (gün ve ay) – `AgeDays`, `YeastAge` [kesin]
- `AgeDays` (malzeme): `gün = (referans tarih − F_Y_PKG_DATE)` tam gün. Referans tarih, maya reçetedeyse (`F_Y_IN_RECIPE ≠ 0`) `F_Y_BREW_DATE`, değilse bugündür.
- `BSTools::YeastAge` (starter aracı): `ay = gün/30`, burada gün = bugün − `F_YC_DATE`. Tarih gelecekteyse ya da gün < 1 ise 0 döner.
- `BSRecipe::YeastAge` (alan adı "Avg Yeast Age", birim ay): ikincil fermantasyona eklenmeyen her mayanın `F_Y_AGE_DAYS` değeri **toplanır** ve 30'a bölünür. Kodda ortalama alınmıyor, toplam veriliyor. Birden fazla maya varsa sonuç "ortalama" değildir [kesin, olası hata].

### 1.4 Canlılık (viability) – `ViabilityPctFunc`, `YeastViability` [kesin]
```
V(%) = V0 · ((100 − r)/100) ^ (gün/30)
V = max(V, 5.0)        (yalnız gün > 0 ise uygulanır; gün ≤ 0 ise V = V0)
```
- `V0`, forma göre ilgili `O_Y_*_VIABILITY` seçeneğidir. Malzemede Dry = 1, Culture(3) = slurry, diğerleri liquid. Araçta Dry = 1, Slurry = 2, diğerleri liquid.
- `r` (%/ay):
  - Malzemede `F_Y_AGE_RATE`. Bu alan < 0 ise (eski veya ithal veri) `YeastImportHook` doldurur: Dry → 1.66, Culture → 25, Laboratuvar = "White Labs" olan sıvı → **5**, diğer sıvılar → 10.
  - Araçta `F_YC_AGE_RATE` (varsayılan 10). Form değişince `O_Y_DRY_AGING`, `O_Y_LIQUID_AGING` veya `O_Y_SLURRY_AGING` ile yeniden doldurulur (`RefreshYeastStarter`).
- Araçta `F_YC_CALC_VIABILITY = 0` ise kullanıcının girdiği `F_YC_VIABILITY` aynen kullanılır.
- Model: aylık sabit oranlı üstel kayıp (bileşik). White Labs/Wyeast'in "ayda yaklaşık %20" doğrusal kuralının yerine geçiyor [tahmin: BeerSmith'e özgü].

`ViableCellsFunc` (malzeme): `canlı hücre = F_Y_TOT_CELLS · V/100` (milyar).
`BSTools::ViableCells`: `F_YC_CELLS_AVAIL · F_YC_VIABILITY/100`.

### 1.5 Toplam hücre (malzeme) – `YeastTotalCells` [kesin]
Sonuç milyar hücredir.
- Paket modu: `F_Y_CELLS (milyar/paket) × F_Y_NEW_AMOUNT (paket)`.
- Hacim/ağırlık modu:
  - Dry (1): `gram × 10`. Kod `(g/75)·100·7.5` yazıyor; bu 10 milyar/g'a eşit. Yani **O_Y_DRY_CELLS_PER_GRAM (18) burada kullanılmıyor**.
  - Liquid/Slant/Culture (0, 2, 3): `ml × F_Y_CELLS_ML` (milyar/ml, varsayılan 1.0).
  - form ≥ 4: 0.

Starter aracındaki karşılığı `YeastCellsAvail`: Dry için `F_YC_DRY_AMT(g) × 10`, diğerleri için `F_YC_WET_AMT(ml) × F_YC_CELLS_ML` (varsayılan 1.0).

### 1.6 Reçete düzeyindeki toplamlar [kesin]
Hepsi, reçetedeki **ikincile eklenmeyen** (`F_Y_ADD_TO_SECONDARY = 0`) mayalar üzerinden hesaplanır.
- `YeastCellsNoStarter` (`F_R_CELLS_NO_STARTER`): `F_R_CALC_VIABILITY = 0` ise Σ`F_Y_TOT_CELLS`, değilse Σ`F_Y_VIABLE_CELLS`.
- `YeastPackCount`: Σ`F_Y_NEW_AMOUNT`.
- `YeastAvgPackSize`: `cells_no_starter / Σamount`. Cells ≤ 0 ise 100; Σamount = 0 ise 100; ingredient listesi geçersizse 0.
- `YeastPackViability`: mayaların `F_Y_VIABILITY_PCT` ortalaması. Maya yoksa 98.
- `YeastTotGrams`: Dry mayalar için Σ gram. Paket modunda bir paket **11 g** sayılır. Hacim modunda oz → g çevrilir.
- `YeastForm`, `YeastUsePkgs`, `YeastCellsMl`, `YeastDryPitchRate`: ilk birincil mayanın ilgili alanı. Maya yoksa sırasıyla 0, 1, 1.0 ve 75 g/hL.

### 1.7 Hedef pitch oranı – `RecommendedLiquidPitch`, `GetPitchMinMax`, `PitchRange` [kesin]
**Reçete:**
```
rec = O_Y_WINE_PITCH          eğer F_R_TYPE ∈ {4,5,6,7}  (Mead, Wine, Seltzer…)
    = O_Y_ALE_PITCH            F_R_ALE_TYPE = 0
    = O_Y_LAGER_PITCH          F_R_ALE_TYPE = 1
    = O_Y_MIX_PITCH            F_R_ALE_TYPE = 2
    = 0.75                     diğer
min = rec − 0.4 ;  max = rec + 0.4
```
Cider (3) bira gibi ele alınır ve şarap oranını almaz [kesin].
`F_R_PITCH_RATE`, `F_R_ALE_TYPE` değiştiğinde `F_R_REC_PITCH_LIQUID` değerine sıfırlanır. Kuru maya için `F_R_DRY_PITCH_RATE`, `F_R_REC_DRY_RATE` değerine sıfırlanır (`UpdateYeastPitch`).

**Starter aracı** (`F_YC_TYPE`'a göre min/max; `F_YC_PITCH_RATE` varsayılanı her durumda 0.75):

| Tip | merkez | aralık |
|---|---|---|
| 0 Ale/Cider | O_Y_ALE_PITCH (0.75) | ±0.30 → 0.45–1.05 |
| 1 Lager | O_Y_LAGER_PITCH (1.5) | ±0.25 → 1.25–1.75 |
| 2 Hybrid | O_Y_MIX_PITCH (1.0) | ±0.40 → 0.60–1.40 |
| 3 Mead/Wine | O_Y_WINE_PITCH (0.5) | ±0.25 → 0.25–0.75 |

`PitchRange` metni "min-max" biçiminde, 2 ondalık.

**Kuru maya (g/hL)** – `DryMinPitchRate`, `DryMaxPitchRate`:
```
min = max(5, floor(rec_dry/4)) ;  max = 2·rec_dry
```
`rec_dry` = `F_YC_REC_DRY_RATE` (araç varsayılanı 75 g/hL) veya `F_R_REC_DRY_RATE` (mayanın `F_Y_DRY_PITCH_RATE` değeri, yoksa 75). Metin 1 ondalık.

### 1.8 Gerekli hücre – `YeastCellsNeeded`, `TotYeastCellsNeeded` [kesin]
Sıvı/slurry (ve paket modundaki kuru):
```
N (milyar) = pitch (M/ml·°P) × V (L) × P(OG)
```
Kuru maya, hacim/ağırlık modunda: reçetede `F_R_YEAST_FORM = 1` ve `F_R_YEAST_USE_PKGS ≠ 1`, araçta `F_YC_YEAST_FORM = 1`.
```
N = V(L) × 7.5 × (dry_rate / rec_dry_rate)      (her iki oran da min 0.01'e kırpılır)
```
Bu, lab önerisinde (ör. 75 g/hL) litre başına 7.5 milyar, yani gram başına 10 milyar varsayımına karşılık gelir.

`EstCellsML` (M hücre/ml) = `N × 1000 / V_ml` (V_ml ≥ 0.1).

### 1.9 Starter büyüme modeli – `InnocRate`, `YeastGrowthRate` [kesin]
İnokülasyon oranı `I` (M hücre/ml = milyar/L) ile büyüme katsayısı `g` (son hücre / başlangıç hücresi) arasında bir güç yasası kullanılır.
```
InnocRate(g)  = exp(5.512221 − 2.1791038·ln g)  = 247.69 · g^(−2.1791038)
GrowthRate(I) = exp((ln I − 5.512221) / (−2.1781038))
   stir plate varsa:  g ← g · (1 + O_Y_STIR_PCT/100)       (varsayılan ×1.45)
   g ← min(g, 6.0) ;  g ← max(g, 1.0)
```
- İki fonksiyondaki üs aynı değil: biri −2.1791038, diğeri −2.1781038. Kodda böyle; muhtemelen bir yazım hatası. Fark ihmal edilebilir, ama bire bir uyumluluk için iki sabit de korunmalıdır.
- Model adı: Chris White ve Jamil Zainasheff'in *Yeast* kitabındaki "inoculation rate vs. growth" verisine güç-yasası eğri uydurması gibi görünüyor [tahmin]. Kai Troester (Braukaiser) modelindeki ekstrakt bazlı doğrusal büyüme **kullanılmıyor**. Stir plate etkisi sabit +%45 çarpanıdır.

**Starter sonrası hücre** (`YeastCellsStarter`):
```
I1 = canlı_hücre(milyar) / starter_L         (starter_L ≤ 0 ise I1 = 0 → büyüme yok)
hücre1 = canlı_hücre × GrowthRate(I1, stir)
```
Reçetede taban `F_R_CELLS_NO_STARTER`, araçta `F_YC_VIABLE_CELLS`.

**İkinci kademe** (`YeastCellsSecStarter`):
```
I2 = hücre1 / ikinci_starter_L ;  hücre2 = hücre1 × GrowthRate(I2, stir)
```
Reçetede ikinci kademe için de `F_R_STIR_PLATE` kullanılıyor; `F_R_SS_STIR_PLATE` hesapta okunmuyor [kesin, olası hata].

### 1.10 Önerilen starter hacmi (tek kademe) – `NewCalcYeastStarter`, `RecStarterSize` [kesin]
```
girdi: N (gerekli, milyar), A (mevcut, milyar), stir
eğer N ≤ A → 0
r = N / A ;  stir ise r ← r / 1.45
eğer r ≤ 1 → 0
V_starter (L) = A / InnocRate(r)        (sonra fl oz'a çevrilir)
```
Reçetede A = `F_R_CELLS_NO_STARTER`, araçta A = `F_YC_VIABLE_CELLS`.
Not: büyüme sınırı (6 veya O_Y_GROWTH_STARTER) bu formülde uygulanmıyor. Çok yüksek r değerlerinde öneri, `GrowthRate`'in 6 ile kırptığı bölgeye düşebilir; uygulama hücre sonucunu yine `GrowthRate` ile hesaplar.

### 1.11 İki kademeli starter – `YeastRec2Stage`, `YeastRecStage1/2` [kesin]
```
A ← max(A, 0.1)
koşul: araçta A ≤ N, reçetede A < N  (sağlanmazsa iki hacim de 0)
r = N/A ;  s = 1 (stir yok) veya 1 + O_Y_STIR_PCT/100
stir ise r ← r / s²
g = √r
  (yalnız reçetede: g = min(√r, O_Y_GROWTH_STARTER) ; varsayılan 4)
V1 (L) = A / InnocRate(g)                 → YeastRecStage1
V2 (L) = A·g·s / InnocRate(g)             → YeastRecStage2
```
Büyüme iki kademeye eşit bölünür (her kademe √r). İkinci kademenin başlangıç hücresi, birinci kademenin sonucudur.

### 1.12 Paket sayıları – `YeastPacksNoStarter`, `YeastPacksStarter`, `CalcYeastPacksNeeded` [kesin]
- Starter yoksa: `pkt = N < pkt_boyu ? 1 : int(N/pkt_boyu + 0.99)`. Burada pkt_boyu = `YeastAvgPackSize` (≤ 0 ise 1).
- Starter varsa:
  ```
  G = O_Y_GROWTH_STARTER (≤0 ise 1.2) ;  stir ise G ← G / 1.45
  pkt = N < pkt_boyu ? 1 : int(N / (G·pkt_boyu) + 0.99)
  sonuç = int(pkt) + 0.1
  ```
  Stir plate burada izin verilen büyümeyi **küçültüyor**, dolayısıyla daha fazla paket istiyor. `+0.1` eklemesi de kodda var (muhtemelen gösterim için yuvarlama). [belirsiz: niyet]
- `CalcYeastStarterSize(r, pkt_boyu, pkt) = pkt·pkt_boyu / InnocRate(r)` (r > 1 ise, aksi halde 0). Bu eski API; güncel arayüz §1.10'u kullanıyor.

### 1.13 Kuru maya – `YeastDryGrams`, `YeastPacksDry`, `YeastDry5`, `YeastDryHydrate`, `YeastGoferm` [kesin]
```
gram = dry_rate (g/hL) × V(L) / 100                 (hacim/ağırlık modu, araç ve reçete)
gram = N / O_Y_DRY_CELLS_PER_GRAM (18; ≤0 ise 0.1)  (reçete, paket modu)
11.5 g paket sayısı = gram / 11.5      (F_R_DRY_YEAST_PACKS, yuvarlama yok)
5 g paket sayısı    = gram / 5
hidrasyon suyu (ml) = YeastTotGrams × O_Y_DRY_HYDRATE (25)
GoFerm (g)          = YeastTotGrams × 25 × O_Y_GOFERM (0.05) = 1.25 g/g maya
```
`YeastStarterText`:
- Kuru maya: "Hydrate yeast with %s warm water and add optional %3.1f g of GoFerm".
- Starter boyutu 0: "No yeast starter used".
- Tek kademe: "Create a yeast starter with %s of wort (%s dry malt extract)".
- İki kademe: "Prepare first starter: %s with %s dry extract, then decant and create second starter: %s with %s DME".

### 1.14 Starter DME miktarı – `DMEStarter` [kesin]
```
DME (oz) = ((OG − 1)·1000 · V_floz/128) / 45 · 16
```
DME verimi 45 ppg (puan·gal/lb) alınır. Starter OG varsayılanı 1.036.
Örnek (SI): DME (g) ≈ (OG−1)·1000 · V_L / 3.785 / 45 · 453.59.

### 1.15 Diğer
- `AvgAttenuation = (F_Y_MIN_ATTENUATION + F_Y_MAX_ATTENUATION)/2`.
- `YeastConvertAmount`: hedef birim "pkgs" (0x29) değilse normal birim dönüşümü yapılır.
- `YeastAmountUnitsFunc`/`YeastUnitsFunc`: paket modunda "pkgs". Değilse kuru maya için O_UNITS_DRY_YEAST (varsayılan 10 = g), sıvı için O_UNITS_WET_YEAST (15 = ml).
- `YeastImportHook`: `F_Y_TOLERANCE == 10` ise Wine tipi 15, Champagne tipi 18 yapılır. Paket modunda `F_Y_NEW_AMOUNT ← F_Y_AMOUNT` senkronlanır.

---

## 2. Mead ve şarap

### 2.1 Enum'lar ve varsayılanlar [kesin]
- `F_MT_METHOD` / `F_R_MEAD_METHOD`: **0 = "TOSNA 2/3 (Fermaid-O)"**, **1 = "TiOSNA (Fermaid-K)"**. Kodda 2 değeri de işleniyor (YAN katsayısı 212.3, muhtemelen DAP) ama listede yok [belirsiz].
- `F_MT_YEAST_FACTOR` / `F_R_YEAST_FACTOR`: 0 Low N (0.75), 1 Medium N (0.90), 2 High N (1.25).
- Mead aracı varsayılanları: OG 1.090, hacim 640 fl oz (5 gal), maya 5 g, bal yüzdesi 100 %.
- Şarap aracı varsayılanları: pH 3.4, hacim 640 fl oz, ölçülen serbest SO2 0, hedef SO2 30 ppm.

### 2.2 Mead maya miktarı – `MeadPitchRate`, `MeadYeastGrams` [kesin]
```
P = ToPlato(OG)
oran (g/L) = 0.25                         P ≤ 24
           = 0.25 + 0.0625·(P − 24)       24 < P < 28
           = 0.5                          P ≥ 28
gram = V(L) × oran
```
Kod hücre/gram ile çarpıp aynı değere bölüyor; ikisi birbirini götürür. Model: Lallemand'ın 25 g/hL (<24 Brix) ve 50 g/hL önerisi, arada doğrusal geçiş [tahmin].

### 2.3 Hidrasyon – `MeadGoferm`, `MeadHydrate`, `MeadHydrate1` [kesin]
```
su (ml)    = maya_g × O_Y_DRY_HYDRATE (25)
GoFerm (g) = maya_g × 25 × O_Y_GOFERM (0.05)   = 1.25 × maya_g
```
Metin: "Mix %3.1f g of GoFerm with %3.1f ml warm water then add yeast and slowly mix in must until pitch temperature is reached".

### 2.4 Besin programı (TOSNA / TiOSNA) – `MeadTotFermO/K`, `MeadFermO/K`, `MeadNutrients` [kesin]
```
k   = {Low 0.75, Med 0.90, High 1.25}
YAN_hedef (ppm) = P(OG) × 10 × k × (bal% / 100)     (bal% yalnız araçta; reçetede 1)
V_gal = hacim (US gal)
Fermaid-O toplam (g) = YAN_hedef × V_gal / 50     (yalnız method 0; aksi halde 0)
Fermaid-K toplam (g) = YAN_hedef × V_gal / 100    (yalnız method 1; aksi halde 0)
Tek ekleme = toplam / 4
```
Program metni: "Add four individual additions of %3.1f g each at 24, 48, 72 hours and 7 days or 1/3 sugar break."
- Model: TOSNA 2.0 ("Tailored Organic Staggered Nutrient Addition", meadmaking topluluğu). YAN gereksinimi Brix·10·N-faktör ile hesaplanır, Fermaid-O 1 g/gal ≈ 50 ppm etkin YAN sayılır. TiOSNA, Fermaid-K karşılığıdır [tahmin: sabitlerin kaynağı].
- Kod °Brix yerine °Plato kullanıyor; pratikte farkı küçük [kesin].
- Reçetedeki sürüm hacim olarak `F_R_VOLUME`, OG olarak `F_R_OG` alır.
- `MeadPctHoney` (reçete): bal tipi (`F_G_TYPE == 7`, alt tip < 7) fermente edilebilirlerin OG puanı / birincil faz OG puanı × 100.

### 2.5 YAN – `YAN`, `YANEquivalent`, `MeadYANAdded`, `MeadYANEquivalent`, `NutrientNeeded` [kesin]
Besin başına "1 g/L → ppm YAN" katsayısı c:

| method | besin | c |
|---|---|---|
| 0 | Fermaid-O | 40 |
| 1 | Fermaid-K | 100 |
| 2 | (DAP, tahmin) | 212.3 |
| diğer | – | 100 |

```
YAN (ppm, mg/L)   = c × gram / V(L)          (V = 0 ise c döner)
Eşdeğer YAN       = YAN × 4  (yalnız Fermaid-O, method 0; organik azot ≈ 4× etkin)
NutrientNeeded(ppm, method, V_L) = ppm × V_L / c   (gram; yardımcı fonksiyon, çağıranı yok)
```
- Araç `gram` olarak **toplam** Fermaid'i alır (`F_MT_TOT_FERMO/K`).
- Reçete sürümü (`BSRecipe::MeadYANAdded`) ise **tek eklemeyi** (`F_R_FERMO/K` = toplam/4) okuyor. Bu yüzden reçetedeki YAN, araçtakinin dörtte biri çıkar [kesin, olası hata; kendi uygulamanızda toplamı kullanmanız önerilir].

### 2.6 Sülfit – `WineSulfiteRecommended`, `WineSulfiteNeeded` [kesin]
```
Önerilen serbest SO2 (ppm) = exp(2.21828·pH − 4.27809)
K-metabisülfit (g) = V_gal × 3.785 × (hedef − ölçülen_serbest) / 576
```
- 576, şu ifadenin kısaltmasıdır: `mg → g (1000) × K-meta'nın %57.6 SO2 verimi`.
- Hedef < ölçülen ise sonuç negatif çıkar; kodda kırpma yok.
- Öneri eğrisi, 0.8 ppm moleküler SO2 tablosunun (pKa ≈ 1.81 Henderson–Hasselbalch) üstel regresyonudur [tahmin].
- Reçetede pH için `F_R_FINAL_PH`, hacim için `F_R_FINAL_VOL` kullanılır.

pH → önerilen serbest SO2 tablosu (formülden hesaplandı):

| pH | 3.0 | 3.2 | 3.4 | 3.5 | 3.6 | 3.8 | 4.0 |
|---|---|---|---|---|---|---|---|
| ppm | 10.8 | 16.8 | 26.2 | 32.7 | 40.8 | 63.5 | 99.0 |

### 2.7 Sorbat – `WineSorbateNeeded` [kesin]
```
Potasyum sorbat (g) = V_gal × 1.25        (≈ 0.33 g/L)
```

### 2.8 Tatlandırma (backsweeten) aracı – `RefreshBacksweeten`, `BackAgentVol`, `BackAgentWeight` [kesin]
Tatlandırıcı (`F_BT_AGENT`) seçildiğinde `F_BT_AGENT_OG` varsayılanı yazılır (birim °Brix / % şeker):

| enum | tatlandırıcı | potansiyel |
|---|---|---|
| 0 | Honey | 82.5 |
| 1 | Apple Juice Concentrate | 70 |
| 2 | Juice | 15 |
| 3 | Dry Malt Extract | 75 |
| 4 | Table Sugar | 98 |
| 5 / diğer | Other | 15 |

```
SG_a (hacim için) = ToSG(potansiyel)                  (Plato formülü)
eğer hedef_SG == SG_a → 0
V_a (L) = V_içecek(L) × (SG_içecek − SG_hedef) / (SG_hedef − SG_a)
Ağırlık (kg) = BrixToSG(potansiyel) × V_a(L)          (Brix formülü, 258.2 varyantı)
```
Bu, gravite puanlarının doğrusal karışımıdır. Varsayılanlar: hacim 640 fl oz, içecek SG 1.015, hedef 1.025. Hacim hesabında Plato→SG, ağırlık hesabında Brix→SG kullanıldığı için iki sonuç arasında küçük bir tutarsızlık oluşur [kesin].

---

## 3. Karbonasyon

### 3.1 Enum'lar ve varsayılanlar [kesin]
- `F_C_TYPE` / `F_R_CARB_TYPE`: 0 Bottle, 1 Keg (zorla, basınç), 2 Keg with priming agent.
- Karbonasyon aracı varsayılanları: 2.3 vol, 68 °F, 640 fl oz. Birimler: hacim fl oz, sıcaklık °F, basınç psi, şeker oz.
- O_UNITS_CARB = 48 (0x30 = vol); 0x31 = g/L.

### 3.2 Kalıntı (dissolved) CO2 [kesin]
Şeker formülünün içinde bulunur; fermantasyon ya da şişeleme sıcaklığına bağlıdır:
```
T = max(T_°F, 32)
CO2_kalıntı (vol) = 3.0378 − 0.050062·T + 0.00026555·T²
```
Tahmini model: yaygın homebrew kalıntı-CO2 eğrisi, ASBC çözünürlük verisine kuadratik uyum [tahmin].
Kullanılan T değeri, araçta `F_CAR_TEMP`, reçetede karbonasyon profilinin `F_C_TEMPERATURE` alanıdır. Doğru kullanımda bu, biranın ulaştığı en yüksek sıcaklıktır (fermantasyon sonu); BeerSmith alanı "Bottling or Keg Storage Temperature" olarak etiketlemiş.

### 3.3 Priming şekeri – `SugarNeeded` ve araç fonksiyonları [kesin]
```
mısır şekeri (g) = 15.195 × V_gal × (hedef_vol − CO2_kalıntı(T))
DME (g)          = mısır × 1.538
sonuç oz'a çevrilir (÷ 28.349525)
```
- `CarbCornSugarFunc`: mısır şekeri (dekstroz monohidrat).
- `CarbDMEFunc`: DME (×1.538).
- `CarbTableSugarFunc`: sofra şekeri (sakaroz) = mısır / 1.099.
- `CarbKegCornFunc`: **mısır şekeriyle tamamen aynı** hesap. Fıçı faktörü uygulanmıyor [kesin]. Eski BeerSmith sürümleri fıçı için yaklaşık yarım miktar öneriyordu; `CarbKegFactor` sabiti 0.5 döndürüyor ama hiçbir hesapta kullanılmıyor, yalnız `F_R_CARB_KEG_FACTOR` alanında gösteriliyor [belirsiz / olası hata].
- Kenar durumlar: hedef vol < kalıntı ise sonuç **negatif** çıkar; kodda kırpma yok. T < 32 °F ise 32 alınır.
- Model: yaygın priming formülü (15.195 g·gal⁻¹·vol⁻¹ dekstroz). Kaynak muhtemelen Tinseth/Fix'e dayanan homebrew hesap makineleri [tahmin].

### 3.4 Reçetedeki karbonasyon miktarı – `CarbAmountText(Vol)`, `CarbEquivFunc` [kesin]
- `F_C_TYPE == 1` (Keg): basınç = `CO2_pressure(F_R_CARB_VOLS, F_C_TEMPERATURE)`, kullanıcının basınç birimine çevrilir.
- Diğer tipler (0 ve 2): şeker = `SugarNeeded(F_R_CARB_VOLS, F_C_TEMPERATURE, V_gal, mısır)`. `F_C_CARB_RATE ≠ 0` ise şeker `/ (F_C_CARB_RATE/100)` ile bölünür. `F_C_CARB_RATE` = priming etkinliği, "mısır şekerine göre %".
- Hacim, normal hesapta `F_R_FINAL_VOL`, ölçülmüş hacim alanında (`F_R_CARB_AMOUNT_ACTUAL`) `F_R_FINAL_VOL_MEASURED` olur.
- `CarbEquivFunc` = `100 / max(F_R_CARB_RATE, 1)` ("Priming Sugar Equiv" katsayısı).
- Tip 2 için de fıçı faktörü yok; şişe ile aynı şeker hesaplanır.
- `CarbUsedString`: "Bottle with …", "Keg with …" + miktar + primer adı. Vol veya oran 0 ise "None".

### 3.5 Zorla karbonasyon basıncı – `CO2_pressure`, `CarbPressureFunc` [kesin]
```
T = max(T_°F, 32) ;  V = hedef vol
P (psi) = −16.6999 − 0.0101059·T + 0.00116512·T² + 0.173354·T·V + 4.24267·V − 0.0684226·V²
P = max(P, 0)
```
Model: Zahm & Nagel / ASBC karbonasyon tablosuna 2 değişkenli polinom uyumu (topluluk hesap makinelerinde yaygın) [tahmin]. Gösterge basıncıdır (psig).

### 3.6 Kalori/karbonhidrat yardımcıları [kesin]
`CarbCalories = F_AT_TOT_CALORIES − F_AT_ALC_CALORIES` ; `CarbGrams = F_AT_CARB_CALORIES / 4` (4 kcal/g).

### 3.7 Birim dönüşümü – `BConvertCarbonation` [kesin]
```
vol → g/L : × 1.9642
g/L → vol : ÷ 1.96042
```
İki sabit birbirinin tersi değil: gidiş-dönüş yaklaşık %0.2 kayar. Kodda böyle; uyumluluk gerekiyorsa korunmalı. Fiziksel değer yaklaşık 1.96–1.98 g/L'dir.

---

## 4. Uygulama notları / bilinen tuhaflıklar (özet)
1. Büyüme modelindeki iki üs farklı: −2.1791038 ve −2.1781038 (§1.9).
2. Kuru maya hücre sayısı çoğu yerde 10 milyar/g varsayar. O_Y_DRY_CELLS_PER_GRAM (18) yalnız reçetenin "paket modu" gram hesabında kullanılır (§1.5, §1.8, §1.13).
3. Reçete "Avg Yeast Age" ortalama değil toplamdır (§1.3).
4. Reçete ikinci kademe stir plate için birinci kademenin bayrağını kullanır (§1.9).
5. Paket hesabında stir plate büyüme sınırını düşürür (§1.12).
6. Reçetedeki mead YAN'ı tek eklemeden hesaplanır, yani 1/4 çıkar (§2.5).
7. Fıçı + priming hesabında 0.5 faktörü uygulanmaz (§3.3).
8. Şeker ve sülfit hesaplarında negatif sonuç kırpılmaz.
9. Viabilite tabanı %5; büyüme katsayısı 1–6 aralığına kırpılır.

---

## 5. Test vektörleri (Python ile hesaplandı)

Bu değerler `/tmp/claude-0/-home-user/9e16fef0-a8b7-5145-928c-0a9718edc1b6/scratchpad/n4/tv.py` betiğiyle üretildi; betik yukarıdaki formüllerin bire bir uygulamasıdır. Sıcaklıklar formüle °F olarak verildi (20 °C = 68 °F, 4 °C = 39.2 °F). 19 L = 5.0193 gal.

### 5.1 Canlılık (90 gün ≈ 3 ay)
| Form | V0 | r (%/ay) | V(90 g) | 100 milyar/paket → canlı |
|---|---|---|---|---|
| Sıvı (genel) | 97 | 10 | **70.71 %** | 70.71 milyar |
| Sıvı, White Labs | 97 | 5 | 83.17 % | 83.17 milyar |
| Kuru | 98 | 1.66 | 93.20 % | – |
| Slurry/Culture | 96 | 25 | 40.50 % | – |

### 5.2 20 L, OG 1.060 ale (pitch 0.75 M/ml·°P)
- P(1.060) = 14.7414 °P → **gerekli hücre = 221.12 milyar**. Reçete aralığı 0.35–1.15 olur; araç aralığı 0.45–1.05 → 132.7–309.6 milyar. EstCellsML = 11.06 M/ml.
- Mevcut hücre 70.71 milyar (3 aylık tek paket):

| Senaryo | Tek kademe starter | İki kademe V1 / V2 | DME (1.036, tek kademe) |
|---|---|---|---|
| Stir yok | **3.424 L** (115.8 fl oz) | 0.989 L / 1.748 L | 11.58 oz = 328.2 g |
| Stir plate | **1.524 L** (51.5 fl oz) | 0.440 L / 0.778 L | 5.15 oz = 146.1 g |

  Kontrol: 1.524 L ile I = 46.41 M/ml. GrowthRate × 1.45 = 3.128, sonuç 221.2 milyar ✓.
- Taze paket (97 milyar): stir yok 2.359 L, stir plate 1.050 L.
- Kuru maya, 20 L, 75 g/hL: 15.0 g → 150 milyar hücre (araç), 1.30 adet 11.5 g paket, 375 ml hidrasyon suyu, 18.75 g GoFerm.

### 5.3 Priming: 2.4 vol, 20 °C (68 °F), 19 L
- Kalıntı CO2 = 0.8615 vol.
- **Mısır şekeri = 117.34 g** (4.139 oz).
- Sofra şekeri = 106.77 g.
- DME = 180.47 g.
- "Keg with corn sugar" = 117.34 g (BS4 davranışı, §3.3).

### 5.4 Zorla karbonasyon: 4 °C (39.2 °F), 2.5 vol
- **P = 11.86 psi** (81.8 kPa gösterge).
- Kontrol: 38 °F / 2.5 vol → 11.25 psi; 68 °F / 2.5 vol → 27.65 psi.

### 5.5 Şarap: pH 3.5
- **Önerilen serbest SO2 = 32.65 ppm.**
- 19 L, ölçülen 0 ppm → K-metabisülfit = 1.077 g.
- Sorbat = 6.27 g.

### 5.6 Mead: OG 1.100, 19 L, %100 bal, orta-azot maya
- P = 23.769.
- Fermaid-O toplam 21.47 g (4 × 5.37 g); YAN 45.21 ppm, eşdeğer 180.8 ppm.
- Fermaid-K toplam 10.74 g (4 × 2.68 g); YAN 56.51 ppm.
- Maya: 0.25 g/L → 4.75 g. OG 1.120 için P = 28.06 → 0.5 g/L.
- 5 g maya için GoFerm 6.25 g, su 125 ml.

### 5.7 Tatlandırma: 19 L, SG 0.998 → 1.010, bal (82.5)
- ToSG(82.5) = 1.44319 → **bal hacmi = 0.526 L**.
- Ağırlık = 1.44346 × 0.526 = 0.760 kg.

### 5.8 Birim
2.4 vol → 4.714 g/L. Geri dönüşte 4.714 g/L → 2.4046 vol (asimetrik sabitler).

---

## 6. Kaynak fonksiyonlar
- **Maya:** `BSYeast::AgeDays`, `ViabilityPctFunc`, `ViableCellsFunc`, `YeastTotalCells`, `YeastImportHook`, `AvgAttenuation`, `YeastConvertAmount`, `YeastUnitsFunc`, `YeastAmountUnitsFunc`.
- **Starter aracı:** `BSTools::YeastViability`, `YeastAge`, `ViableCells`, `YeastCellsAvail`, `TotYeastCellsNeeded`, `YeastCellsNeeded`, `EstCellsML`, `GetPitchMinMax`, `Min/MaxPitchRate`, `PitchRange`, `DryMin/MaxPitchRate`, `DryPitchRange`, `YeastDryGrams`, `InnocRate`, `YeastGrowthRate`, `YeastInocRate`, `YeastSecInocRate`, `YeastCellsStarter`, `YeastCellsSecStarter`, `NewCalcYeastStarter`, `RecStarterSize`, `YeastRec2Stage`, `YeastRecStage1/2`, `CalcYeastStarter*`, `CalcYeastPacksNeeded`, `RecipeStartDMEFunc`, `RecipeSecStartDMEFunc`, `RefreshYeastStarter`, `InitYeastStarter`.
- **Reçete:** `BSRecipe::RecommendedLiquidPitch`, `GetPitchMinMax`, `UpdateYeastPitch`, `YeastCellsNeeded`, `EstCellsML`, `YeastCellsNoStarter`, `YeastCellsStarter`, `YeastCellsSecStarter`, `YeastInocRate`, `YeastSecInocRate`, `RecStarterSize`, `YeastRec2Stage`, `YeastPacksNoStarter`, `YeastPacksStarter`, `YeastPackCount`, `YeastAvgPackSize`, `YeastPackViability`, `YeastAge`, `YeastTotGrams`, `YeastDryGrams`, `YeastPacksDry`, `YeastDry5`, `YeastDryHydrate`, `YeastGoferm`, `YeastStarterText`, `YeastForm`, `YeastUsePkgs`, `YeastCellsMl`, `YeastDryPitchRate`; `DMEStarter`.
- **Mead/şarap:** `MeadPitchRate`, `YAN`, `YANEquivalent`, `NutrientNeeded`; `BSTools::MeadYeastGrams`, `MeadGoferm`, `MeadHydrate`, `MeadHydrate1`, `MeadTotFermO/K`, `MeadFermO/K`, `MeadYANAdded`, `MeadYANEquivalent`, `MeadNutrients`, `InitMeadToolSchema`; `BSRecipe::MeadTotFermO/K`, `MeadFermO/K`, `MeadYANAdded`, `MeadYANEquivalent`, `MeadNutrients`, `MeadPctHoney`; `Wine{SulfiteRecommended,SulfiteNeeded,SorbateNeeded}` (BSTools ve BSRecipe); `InitWineTool`; `InitBacksweetenTool`, `RefreshBacksweeten`, `BackAgentVol`, `BackAgentWeight`.
- **Karbonasyon:** `SugarNeeded`, `CO2_pressure`, `BConvertCarbonation`; `BSTools::Carb{CornSugar,TableSugar,DME,KegCorn,Pressure}Func`, `CarbCalories`, `CarbGrams`, `InitCarbonationSchema`; `BSRecipe::CarbAmountText`, `CarbAmountTextVol`, `CarbAmountMeasuredText`, `CarbEquivFunc`, `CarbKegFactor`, `CarbUsedString`, `CarbActualString`.
- **Yardımcı:** `ToPlato`, `ToSG`, `BConvertHydrometer`, `BConvertVolume`, `BConvertWeight`.
