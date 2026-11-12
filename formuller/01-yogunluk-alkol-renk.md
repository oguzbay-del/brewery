# 01 — Yoğunluk, Verim, Alkol, Renk ve Ölçekleme Formülleri

Kaynak: BeerSmith 4 (Linux) ikili dosyasından decompile edilmiş hesap motoru. Bu belge, kendi hesap motorunu yazacak geliştirici için **uygulanabilir bir referans spesifikasyonudur**. Formüller kodun davranışını yeniden ifade eder; kod kopyalanmamıştır. Her bölümün sonunda izlenebilirlik için kaynak fonksiyon adı verilmiştir.

İşaretler:
- **[doğrulandı]**: decompile ve gerektiğinde makine kodu (objdump) ile kontrol edildi.
- **[çıkarım]**: koddan dolaylı olarak çıkarıldı (ör. birim kodlarının adı).
- **[belirsiz]**: tam okunamadı ya da yorumlanması tartışmalı.
- **[tahmin]**: yayımlanmış formül adıyla eşleştirme konusunda tahmin.

---

## 0. Genel kurallar, birimler ve enum'lar

### 0.1 Dahili birimler

Program tüm hesapları İngiliz/ABD birimleriyle yapar ve yalnızca gösterirken çevirir. Bu birimler alan şemalarındaki (`BSchema::SetUnits`) dahili birim kodlarından ve formüllerdeki sabitlerden doğrulandı:

| Büyüklük | Dahili birim | Kanıt |
|---|---|---|
| Hacim | **ABD sıvı onsu (fl oz)**; `V/128` = galon | Tüm reçete hacimleri `/128` ile kullanılıyor; gravite ayarlama aracında varsayılan hacim 640 (= 5 gal) |
| Ağırlık (fermentable) | **ons (oz, avoirdupois)**; `W/16` = libre | `F_G_AMOUNT` varsayılanı 16.0 (=1 lb), formüllerde `/16` |
| Sıcaklık | **°F** | 139/144/160/153.5 °F sabitleri; `BConvertTemperature(x,2,1)` = °F→°C |
| Yoğunluk | **SG** (ör. 1.050) | Gravite alanlarının dahili birim kodu 3 |
| Renk | **SRM** | `F_G_COLOR` dahili birimi 0x1D(=29), `O_UNITS_COLOR` varsayılanı 29 |
| Refraktometre okuması | **°Brix** | `F_RT_READING` dahili birimi 0x1F(=31) |
| Verim / yield / attenuasyon | **%** (0–100) | |

**Birim kodları** (`BConvertHydrometer`, `BConvertColor`, `BConvertRefractometer`, `BConvertTemperature` içinde) **[çıkarım]**:

| Kod | Anlam |
|---|---|
| 1 | °C |
| 2 | °F |
| 3 | SG (özgül ağırlık) |
| 4 | °Plato |
| 0x1D = 29 | SRM |
| 0x1E = 30 | EBC |
| 0x1F = 31 | °Brix |
| 0x20 = 32 | Kırılma indisi (RI, nD) |
| 0x28 = 40 | Kelvin (BConvertTemperature: K = °C + 273.15) |
| 0x2F = 47 | °Lovibond |

### 0.2 Enum'lar (combo sırası makine kodundan doğrulandı)

**Fermentable tipi `F_G_TYPE`** (`GrainTypes` dizisi) **[doğrulandı]**:

| Değer | Tip | Hesap sınıfı |
|---|---|---|
| 0 | Grain (Tahıl/malt) | "mash'lenebilir": verim (efficiency) uygulanır |
| 1 | Extract (sıvı malt özütü) | "sabit verimli": verim uygulanmaz |
| 2 | Sugar (Şeker) | sabit verimli |
| 3 | Adjunct (Katkı) | mash'lenebilir |
| 4 | Dry Extract (kuru malt özütü) | sabit verimli |
| 5 | Fruit (Meyve) | sabit verimli |
| 6 | Juice (Meyve suyu) | sabit verimli |
| 7 | Honey (Bal) | sabit verimli |
| 8 | Acid (Asit) | kodda tip 0/3 gibi işlenir (pratikte yield ≈ 0) |

Aşağıda **"sabit verimli tipler" = {1,2,4,5,6,7}**, **"mash'lenebilir tipler" = {0,3,8}** olarak anılır.

**Fermentable kullanım aşaması `F_G_USE`** (`GrainUses`) **[doğrulandı]**:

| Değer | Aşama |
|---|---|
| 0 | Mash |
| 1 | Steep (demleme/ıslatma) |
| 2 | Boil (kaynatma) |
| 3 | Whirlpool (kaynatma sonrası) |
| 4 | Primary (birincil fermantasyon) |
| 5 | Secondary |
| 6 | Tertiary |
| 7 | Bottling (şişeleme) |
| 8 | Sparge (yıkama) |

**Reçete tipi `F_R_TYPE`** (`RecipeTypes`) **[doğrulandı]**: 0 = Extract, 1 = Partial Mash, 2 = All Grain, 3 = Cider, 4 = Mead, 5 = Wine, 6 = Seltzer.

**Refraktometre hesap tipi `F_RT_TYPE`**: 0 = Mayalanmamış wort, 1 = Mayalanan wort (OG biliniyor), 2 = Bitmiş biranın OG'si (FG hidrometreyle biliniyor).

**Gravite ayarlama özüt tipi `F_GT_TYPE`**: 0 = Dry Malt Extract, 1 = Liquid Malt Extract.

**Seyreltme tipi `F_DT_TYPE`**: 0 = Su, 1 = Wort ya da bira.

### 0.3 Yardımcı tanımlar

- **PPG** (points per pound per gallon): `PPG = yield% × 46 / 100`. 46, sukrozun (yield %100) PPG'sidir.
- **Gravite puanı**: `P = (SG − 1) × 1000`.
- **Hacim (galon)**: `gal = V_floz / 128`.

---

## 1. Temel dönüşümler

### 1.1 SG → °Plato — `ToPlato`
- Girdi: SG. Çıktı: °P.
- `°P = 135.997·SG³ − 630.272·SG² + 1111.14·SG − 616.868`
- Sınırlama ya da koruma yok.
- Bilinen formül: ASBC/"Lincoln" kübik polinomu (yaygın homebrew formülü) **[tahmin: adı]**.

### 1.2 °Plato → SG — `ToSG`
- `SG = 1 + °P / (258.6 − (°P / 258.6) · 227.1)`
- Bilinen formül: Brewers Friend/"Hackbarth" tipi ters dönüşüm **[tahmin]**.

### 1.3 `BConvertHydrometer(x, from, to)` — yoğunluk birimi çevirici
1. `from == to` ise x olduğu gibi döner.
2. Ölçek düzeltmesi: `from == 3` (SG) iken x kullanıcı tarafından "puan" olarak girilmişse normalize edilir:
   - x > 880 → x/1000; aksi halde x > 88 → x/100; aksi halde x > 8.8 → x/10. (Ör. 1050 → 1.050.)
3. Dönüşümler:
   - SG → Plato: `ToPlato`.
   - Plato → SG: `ToSG`.
   - **SG → Brix**: `Bx = ((182.4601·SG − 775.6821)·SG + 1262.7794)·SG − 669.5622` (ayrı bir kübik polinom).
   - **Brix → SG**: `SG = 1 + Bx / (258.6 − (Bx / 258.2)·227.1)`. Dikkat: iç paydada **258.2** var (ToSG'de 258.6). Sonuç ToSG'ye çok yakındır (12 °Bx'te 5. basamakta bile fark yok). Birebir uyum isteniyorsa bu sabit korunmalı.
   - Plato ↔ Brix: SG üzerinden zincirlenir (Plato→SG→Brix, Brix→SG→Plato).

Kaynak: `BConvertHydrometer`, `ToSG`, `ToPlato`.

### 1.4 Refraktometre birimleri — `BConvertRefractometer`
- **Brix → RI**: `RI = 1.33302 + 0.001427193·Bx + 5.791157e-6·Bx²`
- **RI → Brix** (ikinci derece denklemin kökü): `Bx = (√(2.036879859249e-6 − (1.33302 − RI)·2.3164628e-5) − 0.001427193) / 1.1582314e-5`
- SG ↔ Brix: `BConvertHydrometer` ile; SG ↔ RI: Brix üzerinden.

### 1.5 Renk birimleri — `BConvertColor`
- SRM → EBC: `EBC = SRM × 1.97`; EBC → SRM: `SRM = EBC / 1.97`.
- °L → SRM: `SRM = 1.3564·°L − 0.76`; hedef EBC ise sonuç ayrıca ×1.97.
- SRM → °L: `°L = (SRM + 0.76) / 1.3564`; EBC → °L: `°L = (EBC/1.97 + 0.76) / 1.3564`.
- Not: Literatürdeki yaygın katsayı **1.3546**'dır, programda **1.3564** kullanılıyor (rakamların yeri değişmiş görünüyor). Birebir uyum için 1.3564 kullanın.

### 1.6 Yield ↔ Potansiyel — `PotentialFromYield`, `YieldFromPotential`, `BConvertGrainYield`
- `Potansiyel_SG = 1 + yield% × 0.046 / 100` (ör. %80 → 1.0368).
- `yield% = ((SG − 1)·1000 / 46)·100`, sonra **[0, 100]** aralığına kırpılır.
- `BConvertGrainYield(x, from, to)`: `from == 3` (SG) ise YieldFromPotential, aksi halde PotentialFromYield.

### 1.7 Sıcaklık — `BConvertTemperature`
- °F → °C: `(F − 32) × 5/9`; °C → °F: `C × 9/5 + 32`; Kelvin: °C ± 273.15.

---

## 2. Fermentable'ın gravite katkısı (puanlar)

### 2.1 OG puanı — `BSGrain::GetOGPoints(grain, eff)`
Girdiler: `A` = miktar (oz), `Y` = yield (%), tip, kullanım (`F_G_USE`), `eff` = reçete brewhouse verimi `F_R_EFFICIENCY` (%).
Çıktı: "puan·galon" (lb × PPG × verim oranı; galon başına puan elde etmek için hacme bölünür).

```
PPG = Y × 46 / 100
eğer tip ∈ {1,2,4,5,6,7}:            # sabit verimli (özüt, şeker, meyve, bal…)
    P = (A/16) × PPG
değilse (tip 0,3,8):                   # mash'lenebilir
    eğer kullanım ∈ {0 (Mash), 8 (Sparge)}:
        P = (A/16) × PPG × eff/100
    değilse:                            # Steep, Boil, Whirlpool, Primary… aşamasında tahıl
        P = (A/16) × PPG × O_STEEP_EFFICIENCY/100
```
- Ayar: `O_STEEP_EFFICIENCY` ("Recipe.Steep.Efficiency"), varsayılan **15.0 %**.
- Not: Steep (1) kullanımındaki tahıl, All Grain reçetede bile brewhouse verimi değil **steep verimi** alır.

Kaynak: `BSGrain::GetOGPoints` **[doğrulandı]**.

### 2.2 FG puanı — `BSGrain::GetFGPoints(grain, eff, atten)`
Girdiler: 2.1'dekiler ve `atten` = görünür attenuasyon (%).
Çıktı: fermantasyon sonrası kalan "puan·galon".

```
f = 1 − atten/100                       # varsayılan kalan oran
eğer tip ∈ {1 (Extract), 4 (Dry Extract)}:
    P = (A/16) × PPG                     # verimsiz
eğer tip == 2 (Sugar):
    P = (A/16) × PPG ;  f = −0.231
eğer tip ∈ {5,6,7} (Fruit/Juice/Honey):
    P = (A/16) × PPG ;  f = −0.05
değilse (0,3,8): P, 2.1'deki gibi (mash/sparge → eff, diğer → steep verimi)
eğer F_G_NOT_FERMENTABLE == 0:  P = P × f
# F_G_NOT_FERMENTABLE == 1 (ör. laktoz) ise P olduğu gibi kalır (tamamı FG'ye geçer)
```
- **Negatif katsayılar:** Şeker tamamen fermente olur ve oluşan alkol yoğunluğu suyun altına çeker; bu yüzden şeker FG'yi **düşürür** (−0.231 × puan). Meyve, meyve suyu ve bal için −0.05. Katsayılar sabit kodlanmıştır **[doğrulandı: −0.231, −0.05 ikili dosyadan okundu]**.
- "Fermente edilemez" bayrağı açık bir şeker de tam pozitif puanla FG'ye eklenir.

Kaynak: `BSGrain::GetFGPoints` **[doğrulandı, objdump ile dallar kontrol edildi]**.

---

## 3. Orijinal gravite (OG) ve türevleri

### 3.1 Aşamaya göre OG puanı toplamı — `OGPointsPhase(recipe, phase)`
```
Σ GetOGPoints(g, F_R_EFFICIENCY)   şu fermentable'lar için:
    kullanım ≤ phase   VEYA   kullanım == 8 (Sparge)
```

### 3.2 OG tahmini — `EstimateOG(recipe, V_floz, phase)`
```
V ≤ 0 ise   OG = 1.0
değilse     OG = 1 + OGPointsPhase(phase) / (V/128) / 1000
```

### 3.3 Reçete OG'si — `RecipeOGFunc`
- `V = F_R_VOLUME` (fermentöre giden parti hacmi, fl oz).
- **Reçete tipi 0 (Extract)** ise `V = Ekipman.F_E_PRE_TRUB_LOSS` ("trub kaybından önceki hacim, özüt hesaplarında kullanılır").
- `OG = EstimateOG(V, phase = 4)` → Mash, Steep, Boil, Whirlpool, Primary ve Sparge aşamaları dahil; Secondary, Tertiary ve Bottling hariç.
- Bilinen yöntem: standart "PPG × lb × verim / gal" yaklaşımı.

### 3.4 Sanal OG (tertiary'ye kadar) — `RecipeVirtualOGFunc` → `F_R_VIRTUAL_OG`
- 3.3 ile aynıdır, yalnız `phase = 6`: Secondary ve Tertiary eklemeleri de OG'ye sayılır. Bottling (7) hariçtir.
- **ABV, kalori ve tahmini görünür attenuasyon bu sanal OG ile hesaplanır.**

### 3.5 Mash sonrası gravite tahmini — `GetEstPostMashOG` → `F_R_OG_MASH`
```
P0  = (EstimateOG(F_R_VOLUME, phase=0) − 1) × 1000      # yalnız Mash + Sparge
eğer F_R_BOIL_VOL > 0:
    P0 = P0 × (Ekipman.F_E_TRUB_LOSS + F_R_VOLUME) / F_R_BOIL_VOL
OG_mash = 1 + P0/1000
```
Mantık: Fermentör hacmine göre hesaplanan puanlar, kaynatma sonu hacmine (parti + trub kaybı) ve oradan kaynatma öncesi hacme seyreltilir.
**Karar (T1.16 B):** seyreltme formülü tek yerde yazılır: 05 §5.10 (`mash::preboil_sg`, `1 + (OG − 1)·(V_parti + V_trub)/V_kaynatma`); `density::est_post_mash_og` onu çağırır (`F_R_BOIL_VOL ≤ 0` ise seyreltme yapılmaz, OG_mash = EstimateOG).

### 3.6 Kaynatma öncesi gravite ve geç özüt profili — `GetLateExtractSGs` / `RecipeOGBoilFunc` → `F_R_OG_BOIL`
Girdi: `Vb` = kaynatma hacmi (`F_R_BOIL_VOL`; ≤0 ise alan değeri, o da ≤0.1 ise 0.1).
```
r  = (F_R_VOLUME + F_R_TRUB_LOSS) / Vb                  # seyreltme oranı
P  = Σ GetOGPoints(g, eff)   kullanım ∈ {0,1,2,8} olan ve
                             "geç özüt" OLMAYAN fermentable'lar için
     (geç özüt = tip ∈ {1,2,4,5,6,7} VE F_G_LATE_EXTRACT > 0 dk)
SG[0] = 1 + P·r/(F_R_VOLUME/128)/1000   (F_R_VOLUME ≤ 0 ise 1.0);  t[0] = F_R_BOIL_TIME
```
- `RecipeOGBoilFunc` yalnız `SG[0]`'ı döndürür: **kaynatma başlangıç gravitesi**.
- Profilin devamı, şerbetçiotu kullanımı (IBU) için kullanılır: geç özütler `F_G_LATE_EXTRACT` (dk) azalan sırada (eşitlikte isme göre) eklenir. Her birinde `P += (A/16)·PPG` (verimsiz) ve `SG[k] = 1 + P·r/(V/128)/1000`, `t[k] = min(late_time, boil_time)`. Zamanı bir öncekiyle 0.1 dk'dan az farklı olan kayıt yeni girdi açmaz, önceki girdinin SG'sini günceller. Son girdi Whirlpool (kullanım 3) fermentable'larını da ekler ve `t = 0` alır.
- Dizi kapasitesi 128 girdidir (yığın tamponu).
- **Karar (T1.16 B): tek uygulama.** Aynı profil 02 §5.1'de de kullanılır (`density::late_extract_profile`; `hops` onu çağırır, kendi kopyası yoktur). Önceki iki tanım üç noktada ayrışıyordu; 01'inki seçildi: (1) geç ekstrakt yalnız kazan kullanımlarında (0, 1, 2, 8) sayılır, kazan dışı kullanımdaki (Primary, Secondary, Bottling) geç ekstrakt profile girmez; (2) kullanımı Whirlpool (3) olan fermentable geç ekstrakt sayılmaz, `t = 0` girdisine eklenir; (3) son girdi her zaman `t = 0`'dır: öncesindeki girdi 0.1 dk'dan yakınsa onun yerine geçer (SG güncellenir, zaman 0 olur). Fermentable puanı `GetOGPoints` ile (§2.1) hesaplanır; 02 artık dışarıdan `points` almaz.

### 3.7 Gravite ölçekleme (OG hedefine göre) — `ScaleGravity(recipe, OG_hedef)` / `DoAdjustGravity`
```
P_eski = (F_R_OG − 1)·1000 ;  P_yeni = (OG_hedef − 1)·1000
P_eski > 0 ve P_yeni > 0 ise:
    her fermentable için  A ← A × P_yeni / P_eski
    ardından ScaleBitterness(recipe, IBU_eski)   # IBU'yu korumak için şerbetçiotları yeniden ölçeklenir (bkz. IBU belgesi)
P_yeni ≤ 0 ise hata mesajı; P_eski ≤ 0 ise hiçbir şey yapılmaz.
```
- Tüm fermentable'lar **aynı oranla** ölçeklenir, buna aşama ve tip ayrımı yapılmaz (Secondary/Bottling dahil). Sonra reçete yeniden hesaplanır.
- `DoAdjustGravity`, kullanıcıdan `F_R_DESIRED_OG` alır; değer değişmediyse ölçekleme yapmaz.

### 3.8 Mash profili ölçekleme — `BSMash::ScaleFromGrainWeight(mash, W_yeni)`
- `W_yeni ≤ 0`, eski `F_MH_GRAIN_WEIGHT ≤ 0` ya da ikisi eşitse yalnız `F_MH_GRAIN_WEIGHT = W_yeni` atanır.
- BIAB değilse (`F_MH_BIAB == 0`), infüzyon hacmi > 0 olan her adım için `F_MS_INFUSION ← W_yeni × (F_MS_INFUSION / W_eski)` (su/tahıl oranı korunur).
- BIAB ise yalnız ağırlık atanır; su hacimleri başka yerde hesaplanır.

### 3.9 Kullanıma göre alan toplamı — `TotalGrainUseField(recipe, use, field)`
- Kullanımı `use` olan tüm fermentable'ların `field` değerleri toplanır (ör. miktar).

---

## 4. Verim (efficiency)

### 4.1 Genel verim fonksiyonu — `GetEfficiency(recipe, OG, V_floz, mash_only)`
Çıktı: %.
```
V  ≤ 0 ise V = 0.01 ;  OG ≤ 0 ise OG = F_R_OG
F_R_TYPE ∉ {1 (Partial Mash), 2 (All Grain)} ise → 0
Dahil edilen fermentable'lar:
    mash_only = true :  kullanım ∈ {0 Mash, 1 Steep, 8 Sparge}
    mash_only = false:  kullanım ∈ {0,1,2,3,4, 8}
Bunlar için:
    sabit verimli tipler {1,2,4,5,6,7}:   S += (A/16)·PPG      # verim uygulanmaz
    diğer tipler (0,3,8):                  M += (A/16)·PPG      # teorik %100 potansiyel
M_g = M / (V/128)
Pm  = (OG − 1)·1000
mash_only = false ise  Pm = Pm − S/(V/128)                     # sabit kaynaklı puanlar düşülür
Pm < 0 veya M_g ≤ 0 ise → 0
Verim = Pm / M_g × 100
```
- Not: mash_only modunda şeker/özüt puanları **düşülmez** (çünkü `F_R_OG_MASH` tanım gereği özüt/şeker eklemeden önceki gravitedir).
- Tahıllar burada steep/brewhouse ayrımı yapılmadan %100 potansiyelle sayılır.

### 4.2 Kullanımlar
| Fonksiyon | OG girdisi | Hacim girdisi | mash_only | Anlam |
|---|---|---|---|---|
| `RecipeEstMashEfficiency` | `F_R_OG_MASH` (tahmini) | `F_R_BOIL_VOL` | true | Tahmini mash verimi |
| `RecipeActMashEfficiency` | `F_R_OG_MASH_MEASURED` | `F_R_BOIL_VOL_MEASURED` | true | Ölçülen mash verimi |
| `RecipeActualEfficiency` | `F_R_OG_MEASURED` | `F_R_VOLUME_MEASURED` | false | Ölçülen brewhouse verimi |

Referans ve Rust'ta üç sarmalayıcı da vardır (`recipe_est_mash_efficiency(r)`, `recipe_act_mash_efficiency(r, og_mash_measured, boil_vol_measured)`, `recipe_actual_efficiency(r, og_measured, volume_measured)`); ölçülen değerler reçete modelinde olmadığından çağıran verir. Tahmini mash verimi `F_R_OG_MASH` için §3.5 `est_post_mash_og(r)`'yi kullanır.
---

## 5. Final gravite (FG)

### 5.1 Reçete FG'si — `RecipeFGFunc`
**Adım 1: Attenuasyon.**
- Malzeme listesindeki **ilk** maya kaydı alınır (`F_Y_ADD_TO_SECONDARY == 0` olanlar arasından; ilk geçerli maya bulununca döngü biter).
- `atten = (F_Y_MIN_ATTENUATION + F_Y_MAX_ATTENUATION)/2` (`AvgAttenuation` ile aynı), `tol = F_Y_TOLERANCE` (alkol toleransı, %ABV).
- Maya yoksa `atten = 73.0`, `tol = 10.0`.

**Adım 2: Mash sıcaklığı düzeltmesi** (yalnız `F_R_TYPE == 2`, All Grain):
```
Tref = O_MASH_FG_ADJ_TEMP (153.5 °F)
best = Tref ; bestDiff = 40 ; prev = 0
mash adımları sırayla (t = F_MS_STEP_TEMP, °F):
    d = |t − Tref|
    d < bestDiff ise:
        (prev < Tref  VE  prev ≥ 139  VE  t > Tref) ise  → bu adımı atla
        değilse  best = t ; bestDiff = d ; prev = t
O_MASH_FG_ADJUST ≠ 0 ise:
    best = clamp(best, 144, 160)          # °F
    atten = atten + O_MASH_FG_ADJ_SLOPE × (best − Tref)
```
- Anlamı: Tref'e en yakın adım (±40 °F içinde) "sakkarifikasyon sıcaklığı" kabul edilir. Tref'in altında ve ≥139 °F'ta bir adım zaten seçildiyse sonraki daha sıcak adımlar (ör. mash-out) yok sayılır.
- Ayarlar: `O_MASH_FG_ADJ_TEMP` = **153.5** (°F), `O_MASH_FG_ADJ_SLOPE` = **−1.25** (% attenuasyon / °F), `O_MASH_FG_ADJUST` = **1** (açık).
- Örnek: 148 °F adımı → atten += −1.25 × (148 − 153.5) = **+6.875** puan.
- Mash nesnesi ya da adım listesi geçersizse best = Tref kalır; kırpma yine uygulanır, yani düzeltme yalnız `Tref` 144–160 °F içindeyse 0'dır (Tref = 140, adım yok → best = 144, atten −1.25·4 = −5 değişir). Referans bu sırayı izler.
- Bu yöntemin yayımlanmış bir kaynağı bilinmiyor; BeerSmith'e özgü doğrusal bir model **[tahmin]**.
- **[belirsiz]**: Atlama koşulu `prev` değişkeninin önceki değerine bakar (decompile ve makine kodu bunu gösteriyor). Tasarım niyeti tam olarak anlaşılamadı ama davranış yukarıdaki gibidir.

- Trub kaybı: §3.5/§3.6 `Ekipman.F_E_TRUB_LOSS`, §3.6/§8.1 `F_R_TRUB_LOSS` der; referans ve Rust tek alan taşır (reçetenin ekipman kopyası ile aynı değer kabul edildi **[seçim]**).

**Adım 3: FG puanları.**
```
FGP = Σ GetFGPoints(g, F_R_EFFICIENCY, atten)   # TÜM fermentable'lar, aşama filtresi YOK
V   = F_R_VOLUME   (Extract reçetede Ekipman.F_E_PRE_TRUB_LOSS)
FG  = V ≤ 0 ise 1.0, değilse 1 + FGP/(V/128)/1000
```
- Dikkat: OG'nin aksine FG toplamında **aşama filtresi yoktur**. Secondary, Tertiary ve hatta Bottling (7) aşamasındaki şeker de FG'ye katkı yapar (şeker için negatif). Sanal OG ise Bottling'i içermez. Birebir uyum isteniyorsa bu asimetri korunmalı.

**Adım 4: Maya alkol toleransı sınırı.**
```
abv = CalcABV(F_R_OG, FG)          # dikkat: burada sanal OG değil F_R_OG
abv > tol ve tol > 0 ise:  FG = FGFromTolerance(F_R_OG, FG, tol)
```

Kaynak: `RecipeFGFunc`, `GetTotFGPoints`, `FGPoints`, `BSGrain::GetFGPoints`.

### 5.2 Toleranstan FG — `FGFromTolerance(OG, FG0, tol)`
İkiye bölme (bisection) ile, ABV'yi `tol`'e eşitleyen FG aranır:
```
lo = FG0 ; hi = 1.18
50 kez tekrar (i = 0..49):
    mid = (lo + hi)/2
    a = CalcABV(OG, mid)
    |a − tol| < 0.04 ise → mid döndür
    a > tol ise lo = mid      # alkol fazla → FG yükselt
    a < tol ise hi = mid
döngü biterse son mid döndürülür
```
Kaynak: `FGFromTolerance` **[doğrulandı: dönüş değeri makine kodundan]**.

---

## 6. Alkol, ekstrakt ve attenuasyon

Ortak tanımlar: `OE = ToPlato(OG)`, `AE = ToPlato(FG)`. Aksi belirtilmedikçe `OE = max(OE, AE)` kırpması uygulanır.

### 6.1 Gerçek ekstrakt — `RealExtract(OG, FG)`
- `RE = 0.1808·OE + 0.8192·AE` (°P)
- Bilinen formül: **Balling** gerçek ekstrakt bağıntısı.

### 6.2 Ağırlıkça alkol — `CalcABW(OG, FG)`
- `ABW(%) = (OE − RE) / (2.0665 − 0.010665·OE)`
- Bilinen formül: **Balling** (Hall/"Balling formula") **[tahmin: adı]**.

### 6.3 Hacimce alkol — `CalcABV(OG, FG)`
- `ABV(%) = ABW × FG / 0.79661`
- Bilinen formül: Balling ABW'sinden yoğunluk oranıyla ABV'ye geçiş. 0.79661 etanolün özgül ağırlığıdır. Yaygın "(OG−FG)×131.25" basit formülü **kullanılmıyor**.
- `CalcABV(OG, FG, &abw)` (3 parametreli sürüm) aynıdır ama **OE = max(OE, AE) kırpması yapmaz** ve ABW'yi de döndürür: `OE = ToPlato(OG)`, `AE = ToPlato(FG)`, `RE = 0.1808·OE + 0.8192·AE`, `ABW = (OE − RE)/(2.0665 − 0.010665·OE)`, `ABV = ABW × FG / 0.79661`. OG < FG iken ABW ve ABV negatif çıkar (kırpmalı sürümde 0). Referans: `calc_abv3`, Rust: `abv_abw_unclamped`.

| Fonksiyon | OG | FG |
|---|---|---|
| `RecipeABVFunc` (tahmini ABV) | `F_R_VIRTUAL_OG` | `F_R_FG` |
| `RecipeABVMeasuredFunc` | `F_R_OG_MEASURED` | `F_R_FG_MEASURED` |
| `AttenuationABV` / `AttenuationABW` (araç) | `F_AT_START_GRAVITY` | `F_AT_FINAL_GRAVITY` |

### 6.4 Görünür attenuasyon — `ApparentAttenuation(OG, FG)`
- `AA(%) = (OE − AE)/OE × 100` (OE = max(OE, AE)). OE = 0 için koruma yoktur. `ToPlato(1.000) = −0.003` olduğundan payda sıfırı SG ≈ 1.0000117'de bulur (OG = 1.000'de sonsuz değil, anlamsız bir sonuç: `AA(1.0000117, 1.0) ≈ 11893.6 %`); SG < 1.0000117 için Plato negatiftir. Hata sözleşmesi (OG ≤ FG ya da OG ≈ 1) çağıran katmanda belirlenir (T1.16, C2).
- `EstApparentAttenFunc`: (`F_R_VIRTUAL_OG`, `F_R_FG`); `ApparentAttenFunc`: ölçülen OG/FG; `AttenuationApparent`: araç alanları.

### 6.5 Gerçek attenuasyon — `RealAttenuation(OG, FG)`
- `RA(%) = (1 − RE/OE) × 100`
- `RealAttenFunc`: ölçülen OG/FG; `AttenuationReal`: araç alanları.

### 6.6 Araçtaki ekstraktlar
- `OriginalExtract` = ToPlato(F_AT_START_GRAVITY), `ApparentExtract` = ToPlato(F_AT_FINAL_GRAVITY), `CalcRealExtract` = RealExtract(start, final).

### 6.7 İngiltere vergi ABV'si — `UKTaxABVFunc`
Girdi: ölçülen OG ve FG. Eşikler **tam sayı gravite biriminde** karşılaştırılır (`00-OKU-BENI.md`, kayan nokta kuralı): `u(SG) = round(SG × 100000)` (yarımda çifte), `du = u(OG) − u(FG)` (1 birim = 0.01 puan), `d = du / 100` (puan). `ABV = d × k`; k aşağıdaki tablodan, eşik **aşılırsa** (`du > eşik × 100`, kesin `>`):

| d > | du > | k |
|---|---|---|
| (varsayılan) | | 0.125 |
| 6.95 | 695 | 0.126 |
| 10.45 | 1045 | 0.127 |
| 17.25 | 1725 | 0.128 |
| 26.15 | 2615 | 0.129 |
| 36.05 | 3605 | 0.130 |
| 46.55 | 4655 | 0.131 |
| 57.15 | 5715 | 0.132 |
| 67.95 | 6795 | 0.133 |
| 78.85 | 7885 | 0.134 |
| 89.75 | 8975 | 0.135 |

**Karar (T1.12 C1):** BeerSmith `d = (OG − FG) × 1000` değerini f64 olarak `d > 6.95` vb. ile karşılaştırır. Ondalıkta tam eşik olan farklarda (ör. OG 1.00695, FG 1.000: `d = 6.950000000000012`) ikili hata yüzünden bir üst katsayı seçilir; T1.12 denemesinde tam eşik olan 1210 (OG, FG) çiftinin 583'ünde böyle oldu. Bu bir vergi rakamı olduğundan BeerSmith'in f64 davranışı **bilerek bırakıldı**: eşik tam sayı birimde, kesin `>` ile karşılaştırılır ve tam eşikte alt katsayı kalır (HMRC tablosunun yazılış biçimi). Etkisi yalnız eşiğin birkaç birimi içindedir; Aşama 5 altın testinde BeerSmith ekranıyla farklı çıkabilecek yer burasıdır.

Bilinen yöntem: HM Revenue & Customs (UK Excise Notice 226) gravite kaybı tablosu **[tahmin: adı]**.

### 6.8 Ortalama maya attenuasyonu — `BSYeast::AvgAttenuation`
- `(F_Y_MIN_ATTENUATION + F_Y_MAX_ATTENUATION)/2`

---

## 7. Kalori ve karbonhidrat (12 ABD sıvı onsu = 355 ml porsiyon başına)

### 7.1 Alkol kalorisi — `alc_calories_per_12_oz(OG, FG)`
- `kcal_alk = 1881.22 × FG × (OG − FG) / (1.775 − OG)`

### 7.2 Ekstrakt (karbonhidrat) kalorisi — `ext_calories_per_12_oz(OG, FG)`
- `kcal_ekst = 3550.0 × FG × ((0.1808·OG + 0.8192·FG) − 1.0004)`
- Bilinen formül: ASBC tabanlı, homebrew literatüründe yaygın "12 oz kalori" formülleri **[tahmin: kaynak]**. Bu fonksiyon SG'leri doğrudan karıştırır, Plato'ya çevirmez.

### 7.3 Kullanımlar (hepsinde önce `OG = max(OG, FG)`)
| Fonksiyon | OG | FG | Çıktı |
|---|---|---|---|
| `RecipeCaloriesEstFunc` | VIRTUAL_OG | F_R_FG | alk + ekst |
| `RecipeCaloriesEstAlcohol` | VIRTUAL_OG | F_R_FG | alk |
| `RecipeCaloriesEstCarbs` | VIRTUAL_OG | F_R_FG | ekst |
| `RecipeCaloriesMeasuredFunc` | OG_MEASURED | FG_MEASURED | alk + ekst |
| `AttenuationTotCal` / `AttenuationAlcCal` | araç | araç | toplam / alk |

- `RecipeCarbohydratesEst` = `F_R_CALS_CARBS / 4` (gram; 4 kcal/g).
- `BSTools::CarbCalories` = `F_AT_TOT_CALORIES − F_AT_ALC_CALORIES`; `CarbGrams` = `F_AT_CARB_CALORIES / 4`.

---

## 8. Renk

### 8.1 Reçete rengi — `RecipeColorFunc` (Morey)
```
MCU_toplam = Σ (A_oz/16) × renk_SRM          # TÜM fermentable'lar (aşama filtresi yok)
V_gal      = (F_R_VOLUME + F_R_TRUB_LOSS)/128
V_gal ≤ 0 ise MCU = 0, değilse MCU = MCU_toplam / V_gal
SRM = 1.4922 × MCU^0.6859
```
- Bilinen formül: **Morey** (1.4922 ve 0.6859). Fermentable rengi SRM olarak saklanır ve doğrudan Lovibond yerine kullanılır.
- Hacim, fermentör hacmi + trub kaybıdır (yani kaynatma sonu hacmi). Extract reçetelerde de aynıdır.

### 8.2 Ters Morey
- `MCU = (0.67015 × SRM)^1.45794` (0.67015 = 1/1.4922, 1.45794 = 1/0.6859).

### 8.3 Renk ayarlama (hedef SRM'ye göre bileşim değiştirme) — `DoAdjustColor` → `GetColorAdjRange` → `AdjustColor` → `IterateColor`
Bu kısım sezgisel (heuristik) bir arama algoritmasıdır. Birebir yeniden üretmek zordur; aşağıda davranış özetlenmiştir.

**DoAdjustColor**: Yalnız bira tiplerinde (`F_R_TYPE < 3`) çalışır; diğer tiplerde renk ikonu seçici açılır. Kullanıcıdan `F_R_DESIRED_COLOR` alınır. Önce `GetColorAdjRange` ile ulaşılabilir aralık bulunur; hedef bu aralığın dışındaysa uyarı verir. İçindeyse `AdjustColor(hedef)` çalıştırılır.
Ön koşullar referansta ve Rust'ta `do_adjust_color` ile sarılıdır: `F_R_TYPE ≥ 3` → `not_applicable` (reçete değişmez); hedef `[min, max]` dışında (uçlar dahil içeride) → `out_of_range` (reçete değişmez); aksi halde `adjusted` ve `AdjustColor` sonucu. Ham `adjust_color` bu korumaları taşımaz **[seçim: uçlar aralığa dahil]**.

**GetColorAdjRange**: Reçetenin kopyası üzerinde `AdjustColor(1.0 SRM)` ve `AdjustColor(150.0 SRM)` çalıştırılır. Ortaya çıkan renkler [min, max] aralığıdır. min ≠ max ise ayar mümkündür.

**AdjustColor(hedef)**:
```
IBU0 = F_R_IBU
en fazla 40 iterasyon:
    GetColorFlexibility(kopya, hedef) başarısızsa → dur
    önceki = kopya.renk
    yeni   = IterateColor(kopya2, hedef)
    |yeni − önceki| < 0.005  → dur (yakınsadı)
    |yeni − hedef|  < 0.049  → dur (hedefe ulaşıldı)
ScaleBitterness(recipe, IBU0)                 # IBU korunur
başarı = |renk − hedef| < 0.049
```

**IterateColor(recipe, Tt)** (tek iterasyon):
```
OG0 = F_R_OG ; C = F_R_COLOR ; (lo, hi) = GetColorFlexibility(...)
Vg  = (F_R_VOLUME + F_R_TRUB_LOSS)/128
Δ   = (0.67015·Tt)^1.45794·Vg − (0.67015·C)^1.45794·Vg     # gereken MCU·gal değişimi
Mc  = (0.67015·C)^1.45794·Vg
W   = tüm fermentable miktarlarının toplamı (oz)
Döngü 1, her fermentable i (m_i = A_i·c_i/16):
    "renk verici" ise  [(Tt ≤ c_i ve lo < c_i) veya hi ≤ c_i]:
        A_i ← (m_i + (m_i/Mc)·Δ)·16/c_i            (c_i > 0 ise)
        A_i ← clamp(A_i, 0, Amax_i)
            Amax_i = (F_G_MAX_IN_BATCH + 15)% × W
        ΔA_i = yeni − eski ; ΔP += ΔA_i'nin gravite eşdeğeri:
            tip ∈ {1,2,4,5,6,7}: (Y/100)·ΔA/16
            reçete Extract (0):   (Y/100)·ΔA·15/1600          # 15 sabit kodlu
            diğer:                (Y/100)·ΔA·eff/1600
    değilse "baz" grubu: Wb += A_i
Döngü 2 (Wb > 0 ve ΔP ≠ 0 ise), baz fermentable'lar için
(c_i < hi, (c_i ≤ Tt veya c_i ≤ lo), Y ≠ 0):
    e_i = aynı birimde gravite eşdeğeri
    e_i ≤ ΔP ise A_i = 0
    değilse A_i ← A_i·(1 − A_i·(ΔP/e_i)/Wb)
    A_i ← clamp(A_i, 0, Amax_i)   ; Partial Mash ve tip 0 ise Amax_i ≥ 0.25·W
ScaleGravity(recipe, OG0)        # OG eski değerine döndürülür
dönüş: yeni F_R_COLOR
```
- **[belirsiz / olası hata]**: Döngü 1'deki "%25 alt sınırı" koşulu `F_G_TYPE` yerine kaydın nesne tipini (`GetType()`) 0 ile karşılaştırıyor; bu yüzden pratikte hiç uygulanmıyor. Döngü 2'de doğru alan kullanılıyor.
- Extract reçetede sabit **15** kullanılıyor (`O_STEEP_EFFICIENCY` okunmuyor).
- **[seçim]** Referansın spesifikasyonun sustuğu yerlerdeki seçimleri (her biri için `parite/01_yogunluk.py`'de vaka var): döngü 1'de `c > 0` yanında `Mc > 0` koşulu aranır; döngü 2'de gravite eşdeğeri `e ≤ 0` olan baz malzeme atlanır (`e ≤ ΔP` ise A = 0 kuralı yalnız `e > 0` için); döngü 2 yalnız döngü 1'de "baz" sayılanlar üzerinde döner; "koyu" = rengi hedefin üstünde olan; `F_R_TOT_GRAINS` tüm miktarların toplamıdır (§11.5).

**GetColorFlexibility(recipe, hedef) → (lo, hi, ok)** **[yaklaşık]**:
- `F_R_TOT_GRAINS ≤ 0` veya `F_R_VOLUME ≤ 0` ise ok = false, lo = hi = mevcut renk.
- Aksi halde lo = 1000, hi = mevcut renk ile başlanır. Her fermentable için:
  - Koyulaştırırken (hedef > mevcut) rengi ≥ hedef olan ve payı zaten `≥ (MaxInBatch + 5)%` olan atlanır.
  - Açarken miktarı 0 olan koyu fermentable atlanır.
  - `s = min(2·pay, 1)` alınır, `pay = A/TOT_GRAINS`. Varsayımsal renk `SRM' = 1.4922·[((1−s)·C·TOT + c_i·s·TOT)/16 / (F_R_VOLUME/128)]^0.6859` hesaplanır. Burada C, SRM cinsinden mevcut renktir ve MCU gibi kullanılır (birim karışımı koddaki gibidir).
  - lo = min(lo, SRM'), hi = max(hi, SRM').
- lo > hi ise lo = hi = 0 ve ok = false; lo == hi ise ok = false; aksi halde ok = true.

---

## 9. Ölçüm araçları

### 9.1 Hidrometre sıcaklık düzeltmesi — `HydroAdjustFunc`
Girdiler: `SG_ölç`, ölçüm sıcaklığı `T` (alan °F), kalibrasyon sıcaklığı `Tc` (alan °F). İkisi de önce °C'ye çevrilir.
```
SG_düz = SG_ölç + SG_ölç × [ 1.62860587e-5·(Tc − T)
                           − 5.84994855e-6·(Tc² − T²)
                           + 1.53243635e-8·(Tc³ − T³) ]      (T, Tc °C)
```
- Su yoğunluğu–sıcaklık polinomundan türetilmiş bir oran düzeltmesi gibi görünüyor. Yaygın °F'lık "1.313454 − 0.132674·T …" formülü **değil** **[tahmin]**.

### 9.2 Refraktometre — `RefractometerGravityFunc` → `F_RT_CORRECTED_GRAVITY`
Ayarlar: `O_REF_BRIX_ZERO` = **0.0** (suda okunan sıfır sapması), `O_REF_BRIX_CORRECTION` = **1.02** (wort düzeltme faktörü, WCF).
`B = F_RT_READING − O_REF_BRIX_ZERO` (°Bx).

**Tip 0 — Mayalanmamış wort:**
- `SG = ToSG(B / WCF)`. WCF yalnız bu tipte uygulanır.

**Tip 1 — Mayalanan wort (OG biliniyor, `F_RT_GRAVITY` = OG):**
```
OB = ToPlato(OG)          # °P, WCF uygulanmaz
FG = 1.001843 − 0.002318474·OB − 7.775e-6·OB² − 3.4e-8·OB³
           + 0.00574·B + 3.344e-5·B² + 8.6e-8·B³
```
- Bilinen formül: **Sean Terrill** kübik düzeltmesi (2011 sürümü) **[tahmin: eşleşme güçlü]**. Dikkat: Terrill normalde OB ve FB'nin WCF'ye bölünmesini önerir. Bu kod **bölmüyor** ve OB'yi Brix yerine Plato olarak hesaplıyor (`BConvertHydrometer(OG,3,4)`) **[doğrulandı: objdump]**.

**Tip 2 — Bitmiş biranın OG'si (`F_RT_GRAVITY` = hidrometreyle ölçülen FG):**
```
RI   = 1.33302 + 0.001427193·B + 5.791157e-6·B²
ABW% = 1017.5596 − 277.4·FG + RI·(937.8135·RI − 1805.1288)       # (kod /100 sonra ×100 yapar)
RE   = 194.5935 + 129.8·FG + RI·(410.8815·RI − 790.8732)          # °P
OE   = (RE + 2.0665·ABW%) / (1 + 0.010665·ABW%)                    # Balling ters çözüm
OG   = ToSG(OE)
```
- Ara değerler de hesaplanır ama dönmez: ABV ≈ ABW·FG/**0.794** (0.79661 değil), AA, RA ve kaloriler (OG = max(OG, FG) ile). Referans/Rust'ta yalnız ABV ara değeri `refractometer_finished_abv` olarak açıktır; AA, RA ve kaloriler §6 / §7 fonksiyonlarıyla aynı girdiden hesaplanır.
- Bilinen formül: RI–FG'den ABW ve RE bağıntıları **Louis K. Bonham** ("The Use of Handheld Refractometers by Homebrewers", Zymurgy) ile uyumlu **[tahmin]**.

**ABV/ABW** — `RefractometerABV` / `RefractometerABW`:
- Tip 0 → 0.
- Tip 1 → CalcABV/CalcABW(OG = F_RT_GRAVITY, FG = düzeltilmiş).
- Tip 2 → CalcABV/CalcABW(OG = düzeltilmiş, FG = F_RT_GRAVITY).
- İki çağrı (2 parametreli, kırpmalı `CalcABV`/`CalcABW`) referansta `refractometer_abv` / `refractometer_abw` (tip, `F_RT_GRAVITY`, okuma, sıfır) olarak vardır; tip 1 WCF kullanmaz, tip 0 için WCF gerekmez (sonuç 0).

### 9.3 Gravite yükseltme (özüt ekleme) — `GravityAdjustFunc` → `F_GT_WEIGHT`
```
ΔP = (Hedef − Mevcut)·1000 ;  ΔP ≤ 0 ise 0
V  = max(F_GT_VOLUME, 0.01)  (fl oz)
oz = ΔP·(V/128)·16 / PPG ,   PPG = 44 (Tip 0, DME) | 36 (Tip 1, LME)
```

### 9.4 Gravite düşürme (su ile seyreltme) — `GravityAdjWaterVol` → `F_GT_WATER_VOL`
- `V_su = V·(Mevcut − 1)/(Hedef − 1) − V`, negatifse 0 (fl oz). Hedef = 1.000 için koruma yoktur (sıfıra bölme; motorda hata).
- **Karar (T1.16 B):** 05 §8.2 aynı formüldür; tek uygulama `density::gravity_adjust_water` (hacim birimi çağıranındır: 01'de fl oz, 05'te L), `mash` kendi kopyasını taşımaz.

### 9.5 Seyreltme aracı — `DilutionFinalGravityFunc`, `DilutionTotVolFunc`
```
Vt = V0 + Va                                   (DilutionTotVolFunc)
Va < 0 ise SG = SG0
Tip 0 (su):           SG = 1 + (SG0 − 1)·V0/Vt
Tip 1 (wort/bira):    SG = 1 + (SG0 − 1)·V0/Vt + (SGa − 1)·Va/Vt
```
- Vt = 0 için koruma yoktur.
- Uygulama burada tekrarlanmaz: aynı hesap 05 §8.1'dedir (`mash::dilution`, metrik birim). 01 referansında karşılığı yoktur (ADR-0003 madde 4: kopyalanmaz). Karar (T1.16 B): tek uygulama `mash::dilution`; 01'in seyreltme aracı onu çağırır (birim dönüşümü çağıranda).

---

## 10. Kullanılan ayarlar (Option) özeti

| Option | Ad | Varsayılan | Kullanıldığı yer |
|---|---|---|---|
| O_STEEP_EFFICIENCY | Recipe.Steep.Efficiency | 15.0 % | GetOGPoints, GetFGPoints |
| O_MASH_FG_ADJ_TEMP | Recipe.Mash.FGTemp | 153.5 °F | RecipeFGFunc |
| O_MASH_FG_ADJ_SLOPE | Recipe.Mash.FGSlope | −1.25 %/°F | RecipeFGFunc |
| O_MASH_FG_ADJUST | Recipe.Mash.Adjust | 1 (açık) | RecipeFGFunc |
| O_REF_BRIX_ZERO | Refractometer.Brix.Zero | 0.0 °Bx | RefractometerGravityFunc |
| O_REF_BRIX_CORRECTION | Refractometer.Brix.Correction | 1.02 | RefractometerGravityFunc (yalnız tip 0) |
| O_UNITS_COLOR | Units.Color | 29 (SRM) | gösterim |

Sabit kodlu değerler: maya yoksa atten 73 %, tolerans 10 %; FG arama üst sınırı 1.18, tolerans 0.04 %ABV, en fazla 50 iterasyon; şeker FG katsayısı −0.231; meyve/meyve suyu/bal −0.05; renk ayarında en fazla 40 iterasyon, eşikler 0.005 ve 0.049 SRM, ayar aralığı 1–150 SRM.

---

## 11. Belirsiz kalanlar / dikkat edilecekler

1. **Birim kodu adları** (3 = SG, 4 = Plato, 31 = Brix, 32 = RI, 29 = SRM, 30 = EBC, 47 = °L) formüllerden çıkarıldı. Birim tablosu kaynakta görülmedi **[çıkarım]**.
2. **Mash FG düzeltmesindeki adım atlama koşulu** (bkz. 5.1) makine kodundan doğrulandı, ama tasarım niyeti belirsiz.
3. **GetColorFlexibility** koşulları karmaşık bir decompile çıktısından okundu; ayrıntılar yaklaşıktır. Renk ayarlama algoritmasının birebir kopyası gerekiyorsa ek doğrulama önerilir.
4. **IterateColor** döngü 1'deki %25 kuralı büyük olasılıkla bir hata (yanlış alan karşılaştırması).
5. `F_R_TOT_GRAINS` hesaplayan fonksiyon incelenmedi. Muhtemelen tüm fermentable miktarlarının toplamıdır (oz) **[belirsiz]**.
6. FG toplamında aşama filtresi olmaması (Bottling şekeri FG'yi düşürür) ve Brix→SG'deki 258.2 sabiti büyük olasılıkla kasıtsız. Yine de birebir uyum için korunmalı.
7. Yayımlanmış formül adları (Balling, Terrill, Bonham, HMRC, Morey) sabitlerin eşleşmesine dayanır. Morey ve Balling kesin, diğerleri **[tahmin]**.

---

- **Yield → potansiyel sabiti:** BeerSmith 0.046 kullanıyor (§1.6); sakarozun kesin değeri 46.21 PPG (0.04621). Kendi uygulamada 0.04621 önerilir; fark 4. ondalıkta (bkz. 08 §5.2).

## 12. Test vektörleri

Değerler bu belgedeki formüllerle Python'da hesaplanmıştır (double hassasiyet, yuvarlanmış).

### 12.1 Dönüşümler
| Girdi | Sonuç |
|---|---|
| ToPlato(1.050) | 12.3876 °P |
| ToSG(12.0 °P) | 1.04838 |
| Brix→SG(12.0 °Bx) (258.2'li) | 1.04838 |
| SG→Brix(1.050) | 12.3870 °Bx |
| Brix→RI(10.0) | 1.347871 |
| RI→Brix(1.347871) | 10.0000 |
| SRM 8.81906 (yuvarlanmamış, §12.5) → EBC | 17.374 |
| SRM 8.819 (yuvarlanmış) → EBC | 17.373 |

### 12.2 Alkol / attenuasyon / kalori
| OG | FG | ABW % | ABV % | AA % | RA % | RE °P | kcal alk | kcal ekst | kcal toplam | UK vergi ABV % |
|---|---|---|---|---|---|---|---|---|---|---|
| 1.050 | 1.010 | 4.162 | 5.276 | 79.33 | 64.99 | 4.337 | 104.8 | 60.4 | 165.2 | 5.200 |
| 1.065 | 1.012 | 5.543 | 7.042 | 80.71 | 66.11 | 5.389 | 142.1 | 76.1 | 218.2 | 6.943 |
| 1.080 | 1.016 | 6.717 | 8.566 | 78.90 | 64.64 | 6.836 | 176.0 | 98.0 | 274.0 | 8.448 |

### 12.3 Reçete OG/FG örneği (All Grain)
Girdiler: Parti hacmi 5.5 gal (704 fl oz), brewhouse verimi %72. Malzemeler: 160 oz (10 lb) Grain yield %80 (Mash); 16 oz Grain yield %72 (Mash); 16 oz Sugar yield %100 (Boil). Maya attenuasyonu min 73 / max 77 (ort. 75), tolerans 10. Tek mash adımı 152 °F; ayarlar varsayılan.
- OG puanları = 160/16·36.8·0.72 + 16/16·33.12·0.72 + 16/16·46 = **334.806** → **OG = 1.0609**
- atten = 75 + (−1.25)·(152 − 153.5) = **76.875 %**
- FG puanları = (264.96 + 23.846)·(1 − 0.76875) + 46·(−0.231) = **56.160** → **FG = 1.0102**
- ABV(1.0609, 1.0102) = **6.717 %** (< 10, tolerans devreye girmez)

### 12.4 Tolerans sınırı
- FGFromTolerance(OG = 1.100, FG0 = 1.010, tol = 10) → **FG ≈ 1.0259** (ABV ≈ 10.016 %)

### 12.5 Renk (Morey)
- 10 lb × 2 SRM + 1 lb × 60 SRM; F_R_VOLUME + F_R_TRUB_LOSS = 6.0 gal → MCU = 13.333 → **SRM = 8.819** (EBC 17.374)

### 12.6 Araçlar
| Hesap | Girdi | Sonuç |
|---|---|---|
| Hidrometre düzeltmesi | SG 1.050, T = 77 °F, Tc = 60 °F | 1.05200 |
| Refraktometre tip 0 | 12.0 °Bx, sıfır 0, WCF 1.02 | 1.04739 |
| Refraktometre tip 1 | OG 1.050, 6.5 °Bx | FG 1.01061 (ABV 5.196 %) |
| Refraktometre tip 2 | FG 1.010, 6.5 °Bx | OG 1.05137 (ABW 4.257 %, RE 4.491 °P, OE 12.711 °P) |
| Gravite yükseltme | 1.040 → 1.050, 640 fl oz, DME | 18.182 oz |
| Gravite yükseltme | aynı, LME | 22.222 oz |
| Su ile seyreltme | 1.060 → 1.050, 640 fl oz | 128.0 fl oz (1.0 gal) |
| Seyreltme aracı (su) | 5 gal @1.060 + 1 gal su | 1.05000 |
| Seyreltme aracı (wort) | 5 gal @1.060 + 1 gal @1.030 | 1.05500 |
| Brewhouse verimi | 160 oz grain %80 + 16 oz sugar (Boil), ölçülen OG 1.050, 5.5 gal, All Grain | 62.23 % |

---

## 13. Kaynak fonksiyon dizini

ToSG, ToPlato, BConvertHydrometer, BConvertRefractometer, BConvertColor, BConvertGrainYield, PotentialFromYield, YieldFromPotential, BConvertTemperature, BSGrain::GetOGPoints, BSGrain::GetFGPoints, OGPointsPhase, EstimateOG, RecipeOGFunc, RecipeVirtualOGFunc, GetEstPostMashOG, GetLateExtractSGs, RecipeOGBoilFunc, ScaleGravity, DoAdjustGravity, BSMash::ScaleFromGrainWeight, TotalGrainUseField, GetEfficiency, RecipeEstMashEfficiency, RecipeActMashEfficiency, RecipeActualEfficiency, RecipeFGFunc, GetTotFGPoints, FGPoints, FGFromTolerance, RealExtract, CalcABW, CalcABV (2 sürüm), ApparentAttenuation, RealAttenuation, ApparentAttenFunc, EstApparentAttenFunc, RealAttenFunc, CalcRealExtract, OriginalExtract, ApparentExtract, Attenuation{ABV,ABW,Apparent,Real,AlcCal,TotCal}, UKTaxABVFunc, BSYeast::AvgAttenuation, alc_calories_per_12_oz, ext_calories_per_12_oz, RecipeCalories{EstFunc,EstAlcohol,EstCarbs,MeasuredFunc}, RecipeCarbohydratesEst, CarbCalories, CarbGrams, RecipeColorFunc, DoAdjustColor, GetColorAdjRange, AdjustColor, IterateColor, GetColorFlexibility, GetTotalGrainWeight, HydroAdjustFunc, RefractometerGravityFunc, RefractometerABV, RefractometerABW, GravityAdjustFunc, GravityAdjWaterVol, DilutionFinalGravityFunc, DilutionTotVolFunc.
