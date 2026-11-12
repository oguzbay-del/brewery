# Bira Demleme Hesap Motoru — Formül Spesifikasyonu

Kaynak: BeerSmith 4.0.22 (Linux `.deb`) — hesap fonksiyonları Ghidra ile decompile edilip incelendi; Mac/Windows sürümleriyle aynı kaynak koddan derlenmiş (veri dosyaları birebir aynı). Formüller kendi ifademizle, uygulanabilir spesifikasyon olarak yazıldı.

## Dosyalar

| Dosya | İçerik |
|---|---|
| `01-yogunluk-alkol-renk.md` | Fermentable puanları, OG/FG, sanal OG, verim, mash sıcaklığına göre FG düzeltmesi, ABV/ABW (Balling), gerçek ekstrakt, attenuasyon, kalori/karbonhidrat, UK vergi ABV, renk (Morey), renk/gravite ayarlama, refraktometre, hidrometre düzeltmesi, birim dönüşümleri |
| `02-hop-ibu.md` | Tinseth, Rager, Garetz; tarif IBU motoru (geç ekstrakt SG segmentleri), whirlpool ve kaynatma sonrası katkı, rakım/kaynama sıcaklığı etkisi, form/kullanım düzeltmeleri, AAU, acılık ayarlama, hop oranları, hop yaşlanması |
| `03-su-ph.md` | Tuz–iyon katkı matrisi, mash/sparge/kaynatma/fermentör su profilleri, alkalinite, RA (Kolbach), SO₄/Cl, mash pH modeli (MPH ve BW), laktik/fosforik/asit malt hesapları, sparge asitleme, NNLS ile profil eşleştirme |
| `04-maya-mead-karbonasyon.md` | Maya canlılığı, pitch rate hedefleri, starter büyüme modeli (stir plate, 2 kademe), kuru maya, mead besinleri (TOSNA/TiOSNA, YAN), şarap sülfit/sorbat, tatlandırma, priming şekeri, fıçı basıncı |
| `05-mash-hacimler.md` | Strike/infüzyon/dekoksiyon ısı dengesi, BIAB, batch/fly sparge, tahıl emilimi ve hacmi, kaynatma kaybı, soğuma büzülmesi, tam hacim zinciri, parti-gyle, seyreltme |
| `testler/` | Her bölüm için Python referans uygulaması ve test vektörleri (`python3 testler/02_hop_ibu.py` gibi çalışır) |
| `HESAP-MOTORU-TAMAMI.md` | Beş bölümün tek dosyada birleşimi |

## Önemli genel notlar

- **Dahili birimler:** BeerSmith içeride İngiliz birimleri kullanıyor — hacim fl oz (`/128` = galon), ağırlık oz, sıcaklık °F, gravite SG, renk SRM. Kendi uygulamanızda metrik iç birim seçebilirsiniz; dönüşüm sabitleri dokümanlarda.
- **"Belirsiz" işaretli kısımlar:** Her bölümün sonunda listelendi. Bunlar ya koddan kesin çıkarılamadı ya da programda hata gibi görünen davranışlar (ör. bazı sabitlerin iki yönde farklı yazılması). Birebir uyum mu, doğru formül mü istediğinize göre karar verin.
- **Test vektörleri** formüllerin Python uygulamasından hesaplandı; BeerSmith'in kendi ekran çıktılarıyla karşılaştırılmadı. Kendi uygulamanızı yazmadan önce BeerSmith'te birkaç tarifi açıp sonuçları karşılaştırmanız önerilir.
- Sistem tasarımında not edilen yaklaşım (önceki analizden): alan/ayar tanımları veri tablosunda, yerel+bulut senkron her kayıtta (ID, değişiklik zamanı, senkron durumu), raporlar şablonla.


---

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
- Mash nesnesi ya da adım listesi geçersizse best = Tref kalır ve düzeltme 0 olur.
- Bu yöntemin yayımlanmış bir kaynağı bilinmiyor; BeerSmith'e özgü doğrusal bir model **[tahmin]**.
- **[belirsiz]**: Atlama koşulu `prev` değişkeninin önceki değerine bakar (decompile ve makine kodu bunu gösteriyor). Tasarım niyeti tam olarak anlaşılamadı ama davranış yukarıdaki gibidir.

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
- `CalcABV(OG, FG, &abw)` (3 parametreli sürüm) aynıdır ama **OE = max(OE, AE) kırpması yapmaz** ve ABW'yi de döndürür.

| Fonksiyon | OG | FG |
|---|---|---|
| `RecipeABVFunc` (tahmini ABV) | `F_R_VIRTUAL_OG` | `F_R_FG` |
| `RecipeABVMeasuredFunc` | `F_R_OG_MEASURED` | `F_R_FG_MEASURED` |
| `AttenuationABV` / `AttenuationABW` (araç) | `F_AT_START_GRAVITY` | `F_AT_FINAL_GRAVITY` |

### 6.4 Görünür attenuasyon — `ApparentAttenuation(OG, FG)`
- `AA(%) = (OE − AE)/OE × 100` (OE = max(OE, AE)). OE = 0 için koruma yoktur (OG = 1.000 → sıfıra bölme).
- `EstApparentAttenFunc`: (`F_R_VIRTUAL_OG`, `F_R_FG`); `ApparentAttenFunc`: ölçülen OG/FG; `AttenuationApparent`: araç alanları.

### 6.5 Gerçek attenuasyon — `RealAttenuation(OG, FG)`
- `RA(%) = (1 − RE/OE) × 100`
- `RealAttenFunc`: ölçülen OG/FG; `AttenuationReal`: araç alanları.

### 6.6 Araçtaki ekstraktlar
- `OriginalExtract` = ToPlato(F_AT_START_GRAVITY), `ApparentExtract` = ToPlato(F_AT_FINAL_GRAVITY), `CalcRealExtract` = RealExtract(start, final).

### 6.7 İngiltere vergi ABV'si — `UKTaxABVFunc`
Girdi: ölçülen OG ve FG. `d = (OG − FG) × 1000`. `ABV = d × k`, k aşağıdaki tablodan (eşik **aşılırsa**, yani `d > eşik`):

| d > | k |
|---|---|
| (varsayılan) | 0.125 |
| 6.95 | 0.126 |
| 10.45 | 0.127 |
| 17.25 | 0.128 |
| 26.15 | 0.129 |
| 36.05 | 0.130 |
| 46.55 | 0.131 |
| 57.15 | 0.132 |
| 67.95 | 0.133 |
| 78.85 | 0.134 |
| 89.75 | 0.135 |

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
- Ara değerler de hesaplanır ama dönmez: ABV ≈ ABW·FG/0.794, AA, RA ve kaloriler (OG = max(OG, FG) ile).
- Bilinen formül: RI–FG'den ABW ve RE bağıntıları **Louis K. Bonham** ("The Use of Handheld Refractometers by Homebrewers", Zymurgy) ile uyumlu **[tahmin]**.

**ABV/ABW** — `RefractometerABV` / `RefractometerABW`:
- Tip 0 → 0.
- Tip 1 → CalcABV/CalcABW(OG = F_RT_GRAVITY, FG = düzeltilmiş).
- Tip 2 → CalcABV/CalcABW(OG = düzeltilmiş, FG = F_RT_GRAVITY).

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
| SRM 8.819 → EBC | 17.374 |

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


---

# 02 — Şerbetçiotu (Hop) ve IBU Hesap Motoru

Kaynak: BeerSmith 4 (Linux) decompile edilmiş fonksiyonları. Bu doküman uygulanabilir bir formül spesifikasyonudur; kod kopyası değildir. Her bölümün sonunda kaynak fonksiyon adı verilmiştir.

Emin olunmayan noktalar **[belirsiz]**, yayımlanmış formül eşleşmesi tahmin ise **[tahmin]** diye işaretlidir.

---

## 0. Dahili birimler ve sabitler

| Büyüklük | Dahili birim | Not |
|---|---|---|
| Hop miktarı `F_H_AMOUNT` | **oz (ağırlık)**. Form 3/4 (ekstrakt) için hacim olarak gösterilir (ml); dahilde **fl oz** olarak saklandığı varsayılır | `HopIBUAdjust` form 3 ve 4 için birim kodu 0xF (= ml) kullanıyor; `BConvertVolume`'da 9 = fl oz, 0xF = ml, 0x10 = L |
| Alfa `F_H_ALPHA` | **yüzde** (5.0 = %5) | |
| Hacimler (`F_R_VOLUME`, `F_R_TRUB_LOSS`, `F_R_BOIL_VOL`, `F_HB_VOL`) | **fl oz** (US). Galona çevirmek için /128 | |
| Süre `F_H_BOIL_TIME` | dakika (whirlpool hopu için demleme/steep süresi) | |
| Sıcaklık `F_H_WHIRLPOOL_TEMP`, `F_HA_TEMP` | **°F** dahili; hesaptan önce °C'ye çevrilir (BConvertTemperature birim 1 = °C, 2 = °F) | |
| Rakım `F_R_ALTITUDE` | **ft** | |
| Fermentable miktarı `F_G_AMOUNT` | oz (lb için /16) | |

**IBU dönüşüm sabiti:** `K = 74.62686567164178 = 100/1.34`. IBU = U · alfa% · oz · K / galon. Bu, klasik 0.7489·(1/0.01) = mg/L dönüşümünün BeerSmith'teki hâli (1 oz/gal = 7489 mg/L → 74.89; BeerSmith 1.34 bölenini kullanıyor, yani ≈74.63). Tarif yolunda aynı ifade `W·U·A·100·128 / (V_floz·1.34)` biçiminde yazılır; ikisi matematiksel olarak eşdeğerdir.

`GaretzUtilization`'da ayrıca **74.9** sabiti var (aşağıda).

---

## 1. Enum değerleri

### Hop formu `F_H_FORM`
| Değer | Ad | Davranış |
|---|---|---|
| 0 | Pellet | `O_HOP_PELLET_ADJ` uygulanır |
| 1 | Plug | `O_HOP_PLUG_ADJ` |
| 2 | Leaf (yaprak) | `O_HOP_LEAF_ADJ` |
| 3 | Hop Extract (CO2) | Normal hop gibi kaynatılır. Form düzeltmesi yok. Miktar hacim olarak gösterilir |
| 4 | Extract (Isomerized), yani izomerize ekstrakt | Kullanım oranı her zaman 1.0. Ekipman ve form düzeltmeleri yok. Dry hop olarak eklense bile IBU verir (bkz. §6) |

(Sıra enums.txt'deki S_HF_PELLET/PLUG/LEAF ile koddaki 0/1/2 kontrollerinden çıkarıldı. 3 ve 4 değerleri S_HF_EXTRACT_CO2 ve S_HF_EXTRACT_ISO'dan geliyor. Form 4'ün izomerize ekstrakt olduğu, "kaynatma gerektirmez" notu ve %100 kullanım davranışıyla tutarlı.)

### Hop kullanımı `F_H_USE`
| Değer | Ad |
|---|---|
| 0 | Boil (kaynatma) |
| 1 | Dry Hop: IBU = 0 (form 4 hariç, §6) |
| 2 | Mash |
| 3 | First Wort (FWH) |
| 4 | Steep/Whirlpool ("Aroma") |

### Hop tipi `F_H_TYPE`
0 = Bittering, 1 = Aroma, 2 = Both. Bu alan IBU hesabında **kullanılmıyor**.

### Hop denklemi `O_HOP_EQUATION` (Option, varsayılan `0`)
0 = Tinseth, 1 = Rager, 2 = Garetz. Fonksiyona verilen denklem parametresi 0..2 dışındaysa (ör. −1) global ayar okunur.

### Tarif tipi `F_R_TYPE`
0 = Extract, 1 = Partial Mash, 2 = All Grain. Sadece "Extract mi?" kontrolü yapılıyor (§4).

### Hop yaşlandırma depolama tipi `F_HA_STORAGE`
0 = Poly/Plastik torba → faktör 1.0; 1 = Oksijen bariyerli torba/kavanoz → 0.75; 2 = Vakumlu folyo → 0.5.

### Fermentable kullanımı `F_G_USE` (geç ekstrakt SG profili için, §5.1)
Kodda {0, 1, 2, 8} kazan içi (mash/steep/boil/sparge), 3 ise kaynatma sonrası (whirlpool) olarak ele alınıyor. **[belirsiz]** Tam sıra muhtemelen 0 = Mash, 1 = Steep, 2 = Boil, 3 = Whirlpool, 4.. = fermantasyon aşamaları, 8 = Sparge. Bu, fermentable spesifikasyonundan (diğer proje) doğrulanmalı.

---

## 2. Temel kullanım oranı (utilization) modelleri

Tüm fonksiyonlarda girdiler: `SG` (özgül ağırlık, ör. 1.050) ve `t` (dakika). Çıktı, oran olarak kullanım (0..1).

### 2.1 Tinseth
- Gravity faktörü: `Fg(SG) = 1.65 · 0.000125^(SG − 1)`
- Zaman faktörü: `Ft(t) = (1 − e^(−0.04·t)) / 4.15`
- `U_T(SG, t) = Fg(SG) · Ft(t)`
- Tek hop IBU (araç yolu): `IBU = U_T · A · W · K / V_gal`
- Koşul: `t ≤ 0`, `W ≤ 0` veya `A < 0` ise IBU = 0. `V = 0` ise IBU = 0.
- Ters fonksiyon (gerekli miktar): `W = IBU · V_gal / (U_T · A · K)`
- Yayımlanmış ad: **Glenn Tinseth (1997)**, sabitlerle birebir aynı.
- Kaynak: `tinseth_hop_gravity_adjustment`, `tinseth_hop_boil_time_factor`, `tinseth_hop_utilization`, `tinseth_hop_ibu`, `tinseth_hop_ibu_from_weight`, `tinseth_hop_weight(_from_ibu)`, `TinsethUtil`.

### 2.2 Rager
- Dakika önce yuvarlanır: `n = int(t + 0.5)` (sıfıra doğru kesme).
- Tablo `R(n)`:

| n | R |
|---|---|
| 0 | 0 |
| n ≤ 5 (negatif dahil) | 0.05 |
| 6–10 | 0.06 |
| 11–15 | 0.08 |
| 16–20 | 0.101 |
| 21–25 | 0.121 |
| 26–30 | 0.153 |
| 31–35 | 0.188 |
| 36–40 | 0.228 |
| 41–45 | 0.269 |
| 46–50 | 0.281 |
| 51–55 | 0.291 |
| ≥ 56 | 0.30 |

- Gravity düzeltmesi: `GA = (SG − 1.050)/0.2` (SG > 1.050 ise), aksi hâlde 0.
- `U_R(SG, t) = R(n) / (1 + GA)`
- IBU = `R(n) · A · W · K / ((1 + GA) · V_gal)`. Ön koşullar Tinseth ile aynı.
- Ters: `W = (1 + GA) · V_gal · IBU / (R · A · K)`
- Yayımlanmış ad: **Jackie Rager (Zymurgy 1990)**. Tablo, Rager'in yüzde tablosunun (5, 6, 8, 10.1, 12.1, 15.3, 18.8, 22.8, 26.9, 28.1, 29.1, 30) 5 dakikalık basamak hâli.
- Kaynak: `rager_hop_util`, `rager_hop_gravity_adjustment`, `rager_hop_ibu`, `rager_hop_ibu_from_weight`, `rager_hop_weight(_from_ibu)`, `RagerUtil`.

### 2.3 Garetz (iteratif)
Parametreler (`Garetz_hop_ibu(W, A, t, SG, V, V_boil)`):
- `W`: miktar (oz), `A`: alfa (%), `t`: dakika, `SG`: kaynatma gravity'si.
- `V`: **son hacim** (galon). `V_boil`: **kaynatma hacmi** (galon). Bu ikisi Garetz'in "concentration factor" CF = V/V_boil oranını verir. (Görevde sorulan param_5/param_6 bunlardır; yükseklik faktörü değildir.)
- Tablo `GaretzUtil` (yüzde, 15 eleman): `0, 0, 2, 5, 8, 11, 14, 16, 18, 19, 20, 21, 22, 23, 23`
- İndeks: `i = trunc((trunc(t) − 1) / 5)`, üst sınır 14. C tamsayı bölmesi sıfıra doğru keser, yani 0 < t < 1 için i = 0. Örnekler: t = 60 → i = 11 → %21; t = 15 → i = 2 → %2; t = 10 → %0.
- Hesap:
  - `r = V / V_boil`
  - `CG = (SG − 1)·r + 1` (kaynatma gravity'si)
  - `GF = (CG − 1.050)/0.2 + 1`. **Alt sınır uygulanmıyor**: SG < 1.050'de GF < 1, bu da IBU'yu artırır. Garetz'in orijinalinden farklı olabilir.
  - `TF = 1.02` sabit. Garetz'in sıcaklık/rakım faktörü; burada sabitlenmiş. Rakım ayrıca §3.2'de işleniyor.
  - Başlangıç: `IBU_prev = 1`, `IBU = 50`.
  - Döngü, `|IBU_prev − IBU| > 0.05` iken sürer:
    - `HF = r · IBU / 260 + 1` (hopping rate factor)
    - `IBU_prev = IBU`
    - `IBU = U%[i] · A · W · 0.749 / (V · GF · HF · TF)`
- 0.749 sabiti, U% ve A% yüzde olarak girildiği için `7489/10000`'dir: 1 oz/gal = 7489 mg/L.
- Ön koşullar: `t ≤ 0`, `W ≤ 0` veya `A < 0` ise 0. Kodda V = 0 için de 0 dönen bir kontrol var (decompile'da erişilemez görünüyor).
- **Kullanım oranı biçimi** (`GaretzUtilization(SG, t)`): `Garetz_hop_ibu(1, 1, t, SG, 1, 1) / 74.9`. W = 1 oz, A = %1, V = 1 gal ve oran 1 olduğu için IBU çok küçüktür ve HF ≈ 1 olur. Sonuç yaklaşık `U%/100 / (GF · 1.02)`. **Önemli:** tarif yolunda Garetz'in hopping-rate (HF) etkisi pratikte kaybolur. Araç yolunda (§7) gerçek miktar kullanıldığı için HF etkilidir.
- Ters (`Garetz_hop_weight`): hedef IBU için W, ikiye bölme (bisection) ile bulunur. Aralık [0, 1e6], başlangıç W = 500, |IBU − hedef| ≤ 0.05 olunca durur. Bu fonksiyon V_boil = V ile çağrılır.
- Yayımlanmış ad: **Mark Garetz, "Using Hops" (1994)**. Yaklaşık uyumlu; tablo ve sabit TF basitleştirilmiş **[tahmin]**.
- Kaynak: `Garetz_hop_ibu`, `Garetz_hop_weight`, `GaretzUtilization`, global `GaretzUtil`.

---

## 3. Sıcaklık ve rakım etkisi

### 3.1 Sıcaklık faktörü (whirlpool ve kaynama altı)
- `Ftemp(T°C) = 2.39·10¹¹ · e^(−9773 / (T + 273.15))`, **yalnızca T < 100 °C ise**. T ≥ 100 ise `Ftemp = 1`.
- Not: formül 100 °C'de ≈1.0064 verir. 100'ün hemen altında küçük bir süreksizlik oluşur (99.99 °C → 1.0085).
- Yayımlanmış ad: **Malowicki & Shellhammer (2005)** Arrhenius izomerizasyon hız oranı (k(T)/k(100 °C)).

### 3.2 Rakım → kaynama noktası (`BoilTemp`)
- `T_boil(°C) = 100 − alt_ft · 0.3048 · 0.00324` (yaklaşık her 308.6 m için 1 °C). Fonksiyon sonucu °F olarak döndürür, `HopUtil` tekrar °C'ye çevirir.
- Girdi: `F_R_ALTITUDE` (ft). Rakım ≤ 0 ise T ≥ 100 olur, faktör 1.
- Etki: tarif yolunda **tüm kaynatma/FWH/mash hoplarının** kullanım oranı `Ftemp(T_boil)` ile çarpılır. Kaynatma denklemi Tinseth, Rager ya da Garetz olabilir. Örnek: 5000 ft → 95.06 °C → ×0.7103.
- Kaynak: `BoilTemp`, `HopUtil`.

### 3.3 `HopUtil(SG, t, denklem, whirlpool_mu, T_°F)`: segment kullanım oranı
- `whirlpool_mu = true` ise: `U = U_T(SG, t) · Ftemp(T°C)`. **Her zaman Tinseth**; seçili denklem dikkate alınmaz.
- Değilse: `U = U_denklem(SG, t) · Ftemp(T°C)`. Burada T, kaynama sıcaklığıdır (§3.2).

### 3.4 Whirlpool hop IBU (araç yolu; `whirlpool_hop_ibu`, `whirlpool_hop_util`)
- `U_W = U_T(SG, t_steep) · Ftemp(T_whirlpool°C)`
- `IBU = U_W · A · W · K / V_gal`. V = 0 ise 0.
- Sıcaklık sabit kabul edilir. Soğuma whirlpool hopunun kendisi için modellenmez.

### 3.5 Kaynatma hoplarının whirlpool'daki ek katkısı (`PostBoilUtil(SG, t0, D)`)
- Ne hesaplar: kaynatma bittikten sonra, whirlpool boyunca kazanda kalan kaynatma hoplarının ek kullanım oranını. Sayısal integral olarak hesaplanır.
- Girdiler: `SG` (kaynatma sonu gravity'si, §5.1'deki son eleman), `t0` = hopun kaynatma süresi (dk), `D` = toplam whirlpool süresi (dk, `F_E_WHIRLPOOL_TIME`, tamsayı).
- Algoritma (sol Riemann toplamı, adım 0.1 dk):
  ```
  S = 0; τ = t0
  τ < t0 + D olduğu sürece:
      e      = τ − t0                       (alev kesildikten beri geçen süre)
      T_F    = 210.64 − 1.344·e             (°F; 99.24 °C'den 0.747 °C/dk soğuma)
      f      = 2.39e11 · exp(−9773 / ((T_F + 459.67)·5/9))
      τ < 5.0 ise f = 1                     (DİKKAT: e değil τ, yani mutlak hop süresi)
      S     += Fg(SG) · 0.04 · e^(−0.04·τ) / 4.15 · f · 0.1
      τ     += 0.1
  sonuç S
  ```
- Yorum: Tinseth zaman eğrisinin türevi, hopun "kaynama yaşı" τ'dan devam eder ve soğuyan wort'un Arrhenius faktörüyle ağırlıklandırılır. `τ < 5` istisnası nedeniyle 5 dk'dan kısa kaynatılan (ör. alev kesimi = 0 dk) hoplar ilk dakikalarda tam faktör alır. Bu davranış amaçlı görünmüyor ama birebir uygulanmalı.
- Kayan nokta birikimi nedeniyle adım sayısı 10·D veya 10·D+1 olabilir. Birebir eşleşme için 0.1'i aynı şekilde biriktirin.
- **Her zaman Tinseth** tabanlıdır (Rager/Garetz seçili olsa bile).
- Ayar: ekipmanda `F_E_WHIRLPOOL_CARRY` ("Estimate Boil Hop Util in Whirlpool") ≠ 0 ise uygulanır.
- Yayımlanmış ad: yok. BeerSmith'e özgü, Malowicki tabanlı bir soğuma modeli **[tahmin]**.
- Kaynak: `PostBoilUtil`, `HopTotalUtil`.

---

## 4. Hop düzeltme çarpanı (`HopAdjustments(hop, util%, extract_tarif_mi)`)

Çıktı, IBU'yu çarpan tek bir faktör: `F_adj`.

1. Form = 4 (izomerize ekstrakt) ise **1.0** döner. Ekipman kullanım oranı da uygulanmaz.
2. Use = 1 (Dry Hop) ise **0** döner.
3. Aksi hâlde `m = 1` ile başlanır ve sırayla şu çarpanlar uygulanır:
   - Use = 3 (FWH) **ve tarif Extract değilse**: `m ×= (100 + O_HOP_FIRST_WORT_ADJ)/100`. Varsayılan **+10.0** → ×1.10.
   - `O_HOP_TYPE_ADJ` ≠ 0 (varsayılan **1**) ise, forma göre tek bir çarpan:
     - Leaf (2): `(100 + O_HOP_LEAF_ADJ)/100`, varsayılan **0.0** → ×1.00
     - Plug (1): `(100 + O_HOP_PLUG_ADJ)/100`, varsayılan **2.0** → ×1.02
     - Pellet (0): `(100 + O_HOP_PELLET_ADJ)/100`, varsayılan **10.0** → ×1.10
     - Form 3 (CO2 ekstrakt): düzeltme yok
   - Use = 2 (Mash) **ve tarif Extract değilse**: `m ×= (100 + O_HOP_MASH_ADJ)/100`. Varsayılan **−80.0** → ×0.20.
4. Sonuç: `F_adj = (util% / 100) · m`.

`util%` girdisi:
- Tarif yolunda ekipmanın `F_E_HOP_UTIL` değeridir ("Large Batch Hop Util"). 20 galondan küçük partilerde %100 olmalı. Varsayılan değer Option tablosunda yok **[belirsiz]**, pratikte 100.
- Araç yolunda `F_HB_HOP_UTIL` kullanılır.

`extract_tarif_mi`:
- Tarif yolunda `F_R_TYPE == 0` (Extract) kontrolüdür. Extract tariflerde FWH ve mash hopları düzeltmesiz, düz kaynatma hopu gibi sayılır.
- Araç yolunda her zaman false.

Whirlpool hopları (use 4) da form ve ekipman düzeltmesini alır.

Kaynak: `HopAdjustments` (iki kopya, aynı mantık).

Not: `O_WHIRLPOOL_UTIL` (varsayılan 50.0, "Whirlpool utilization compared to a regular boil") incelenen hiçbir IBU fonksiyonunda **kullanılmıyor**. BeerSmith 2/3'ten kalan eski bir ayar olduğu düşünülüyor **[belirsiz]**. Whirlpool artık §3.4'teki sıcaklık modeliyle hesaplanıyor.

---

## 5. Tarif düzeyinde IBU (asıl motor)

Çağrı zinciri:
- `RecipeIBUFunc` → `NewGetEstIBU`
- `NewGetEstIBU` → her hop için `NewHopIBU` → `HopTotalUtil` → `HopUtil` / `PostBoilUtil`

`RecipeIBUTinseth`, `RecipeIBURager` ve `RecipeIBUGaretz`, `O_HOP_EQUATION`'ı geçici olarak 0, 1 ya da 2 yapıp `NewGetEstIBU`'yu çağırır, sonra ayarı geri yükler. Böylece üç denklemin sonucu da ayrı ayrı gösterilebilir.

### 5.1 Kaynatma boyunca gravity profili (`GetLateExtractSGs(tarif, V_boil)`)
Amaç, geç eklenen ekstraktları hesaba katan, zamana göre parçalı sabit bir SG listesi üretmektir. Liste `[(SG_k, t_k)]` biçimindedir; t azalan sırada gider ve son elemanın `t = 0` olduğu garanti edilir.

**Karar (T1.16 B): tek uygulama.** Profil 01 §3.6'da tanımlıdır ve motorda tek fonksiyondur (`density::late_extract_profile`); `hops` onu çağırır. Önceki iki tanım (burada ve 01'de) üç noktada ayrışıyordu, 01'inki seçildi: kazan dışı kullanımdaki geç ekstrakt profile girmez, kullanımı Whirlpool olan geç ekstrakt `t = 0` girdisine eklenir, son girdi her zaman `t = 0`'dır. Fermentable puanı 01 §2.1 `GetOGPoints` ile (`F_R_EFFICIENCY` ve `O_STEEP_EFFICIENCY` reçeteden); 02 dışarıdan `points` almaz. Geç ekstrakt: tipi ∈ {1, 2, 4, 5, 6, 7} ve `F_G_LATE_EXTRACT` > 0. `SG(P) = 1 + P · r / (V/128) / 1000`, `r = (V + T) / V_boil`; V ≤ 0 ise SG = 1.

Kaynak: `GetLateExtractSGs` (ek decompile), `OGPointsPhase`.

### 5.2 Hop başına toplam kullanım oranı (`HopTotalUtil(hop, tarif, SG[], t[], denklem)`)
Girdi: hop süresi `th = F_H_BOIL_TIME`. Kaynama sıcaklığı `T_b` §3.2'deki gibi `F_R_ALTITUDE`'dan bulunur.

1. Form 4 → **U = 1.0**.
2. `W ≤ 0` veya use = 1 (dry) → **U = 0**.
3. **Use = 4 (Whirlpool):**
   - k, `t[k] = 0` olan son elemandır.
   - `U = U_T(SG[k], th) · Ftemp(F_H_WHIRLPOOL_TEMP → °C)`
   - Tek segmenttir; carry eklenmez. Bu dal decompile'da bozuktu, assembly'den doğrulandı.
4. **Diğer (boil, FWH, mash):** segment listesi `s[]` oluşturulur.
   - (a) `t[0] < th` ise (hop süresi tarifin kaynatma süresinden uzunsa, ör. FWH veya mash hopu):
     `s ← HopUtil(SG[0], th) − HopUtil(SG[0], t[0])`
   - (b) Her `i` için (`t[i] ≠ 0` olduğu sürece), `t[i+1] ≤ th` ise:
     `başla = min(t[i], th)`, `s ← HopUtil(SG[i], başla) − HopUtil(SG[i], t[i+1])`
     Yani hop eklendikten sonraki her gravity aralığında, o aralığın SG'si ile kümülatif kullanım farkı alınır.
   - (c) Ekipman `F_E_WHIRLPOOL_CARRY` ≠ 0 ise:
     `s ← PostBoilUtil(SG[son], th, F_E_WHIRLPOOL_TIME)`
5. Birleştirme (olasılıksal):
   ```
   U = Σ_k s_k · Π_{j<k} (1 − s_j)  =  1 − Π_k (1 − s_k)
   ```
   Tek segmentte `U = s_0` olur. Geç ekstrakt yoksa ve carry kapalıysa `U = U_denklem(SG_preboil, th) · Ftemp(T_b)`.

`HopUtil` her segmentte rakım faktörünü uygular. Tinseth için fark `U(a) − U(b) = Fg·(e^(−0.04b) − e^(−0.04a))/4.15` olur. Rager'de basamak farkı, Garetz'de iteratif kullanım farkı alınır.

Kenar durumu: kaynatma hopu süresi 0 ise (alev kesimi) segment katkısı 0 olur. Carry açıksa yalnızca `PostBoilUtil` katkısı kalır.

Kaynak: `BSRecipe::HopTotalUtil` (decompile + assembly).

### 5.3 Hop IBU'su (`NewHopIBU`)
```
V_ibu  = F_R_VOLUME + F_R_TRUB_LOSS      (fl oz; ≤ 0 ise 0.01)
U      = HopTotalUtil(...)
F_adj  = HopAdjustments(hop, F_E_HOP_UTIL, F_R_TYPE == 0)
IBU_h  = F_adj · W · U · A · 100 · 128 / (V_ibu · 1.34)
       = F_adj · U · A · W · K / (V_ibu/128)
```
- **IBU'nun hacmi:** parti hacmi (fermentöre giden) + trub/kayıp hacmi. Bu, soğutulmuş kaynatma sonu hacmine karşılık gelir. Kaynatma öncesi hacim ya da sadece fermentör hacmi kullanılmaz.
- Form 4: `IBU = W · A · 74.627 / V_gal`. Burada W hacimdir (fl oz) ve yoğunluk 1 kabul edilir **[belirsiz]**.

Kaynak: `BSRecipe::NewHopIBU`.

### 5.4 Tarif toplam IBU (`NewGetEstIBU`)
- `IBU_tarif = Σ_hop IBU_h + Σ_hopped_extract IBU_e`
- Hopped extract (fermentable): `F_G_TYPE == 1` (Extract) ve `F_G_IBU_GAL_PER_LB > 0` ise:
  `IBU_e = (F_G_AMOUNT/16 · F_G_IBU_GAL_PER_LB) / max((V + T)/128, 0.0001)` (IBU·gal/lb · lb / gal)
- Her hop için yan çıktılar:
  - `F_H_IBU_CONTRIB = IBU_h`
  - `F_H_PERCENT = W·100 / F_R_TOT_HOPS`
- Hop sırası ya da fazı sonucu etkilemez. Her hop bağımsız hesaplanıp toplanır. Hoplar arası doygunluk modeli yoktur.
- Kaynak: `BSRecipe::NewGetEstIBU`, `RecipeIBUFunc`, `RecipeIBUTinseth`, `RecipeIBURager`, `RecipeIBUGaretz`.

---

## 6. Acılık ayarlama araçları

### 6.1 Hedef IBU'ya ölçekleme (`ScaleBitterness(tarif, hedef)`, `DoAdjustBitterness`)
`DoAdjustBitterness`, hedef alanını (`F_R_DESIRED_IBU`) mevcut `F_R_IBU` ile doldurur, diyaloğu açar ve ardından `ScaleBitterness`'ı çağırır.

1. `R = F_R_IBU`.
   - `|hedef − R| ≈ 0` ise hiçbir şey yapılmaz (başarılı).
   - R ≈ 0 veya hedef ≈ 0 ise başarısız döner.
2. `F_R_VOLUME` 0 ise 0.0001 yapılır.
3. Her hop (dry hoplar hariç; form 4 dry hop dahil) için `I_i = NewHopIBU`. Negatif ya da ≈0 ise 0 alınır. Hopped extract IBU toplamı `E` (§5.4).
4. `E < R` ise `R' = R − E` olur, aksi hâlde R değişmez. `T' = max(hedef − E, 0)`.
5. Her dry olmayan ve `I_i ≠ 0` hop için yeni miktar:
   `W_yeni = (I_i / F_adj_i) · (T'/R') · (V+T) · 1.34 / (A · U_i · 100 · 128)`
   - Pratikte `W_yeni = W_eski · T'/R'`: bütün acılık hopları aynı oranla ölçeklenir. Hopped extract sabit kalır.
   - `A·U` ≤ 0 olan hop atlanır.
   - Kenar durumu: form 4 dry hop ilk döngüde diziye alınıyor, ikinci döngüde atlanıyor. Bu durumda dizi indeksleri kayar (yazılım hatası) **[belirsiz]**.
- Kaynak: `BSRecipe::ScaleBitterness`, `DoAdjustBitterness`.

### 6.2 Hop başına IBU düzenleme (`HopIBUAdjust`, `TotHopAdjIBU`)
- `IBU_contrib > 0` olan her hop listelenir (en fazla 40). Kullanıcı her hop için yeni IBU girer.
- Yeni miktar: `W_yeni = (W_eski / IBU_eski) · IBU_yeni`. Doğrusal kabul edilir; Garetz'de tam doğru değildir.
- Gösterim birimi: form 3 ve 4 için ml, diğerleri için `O_UNITS_SMALL_WEIGHT`.
- `TotHopAdjIBU`, diyalogda girilen IBU'ların toplamını gösterir.

---

## 7. Hop Bitterness aracı (Tools; `HopIBUFunc` → `CalcIBU`)
Tarif motorundan **daha basit** bir yoldur. Girdiler:
- `F_HB_OG`: kaynatma gravity'si (tek SG)
- `F_HB_VOL`: parti hacmi, fl oz
- `F_HB_FORMULA`: denklem
- `F_HB_HOP_UTIL`: %

`F_HB_BOIL_VOL` IBU hesabında kullanılmıyor.

Her hop için `CalcIBU`:
1. Form 4: `IBU = W · A · 100 · 128 / (V_floz · 1.34)`.
2. `W ≤ 0`, use = 1 (dry) ya da `V ≤ 0` ise 0.
3. Use = 4: `whirlpool_hop_ibu(W, A, t, SG, V_gal, T_whirlpool°C)` (§3.4).
4. Değilse denkleme göre `tinseth_hop_ibu` / `rager_hop_ibu` / `Garetz_hop_ibu(W, A, t, SG, V_gal, V_gal)`. Garetz'de r = 1 olduğu için HF gerçek IBU ile hesaplanır.
5. Sonuç × `HopAdjustments(hop, F_HB_HOP_UTIL, false)`. Araçta tarif Extract sayılmaz; FWH ve mash düzeltmeleri her zaman uygulanır.

Araçta **olmayanlar:** rakım düzeltmesi, geç ekstrakt segmentleri, whirlpool carry, hopped extract, trub hacmi.

Çıktılar:
- `F_HB_IBU = Σ IBU`
- `F_H_IBU_CONTRIB`
- `F_H_PERCENT = W·100/ΣW`
- global `aau = Σ CalcAAU`

### 7.1 AAU / HBU (`CalcAAU`)
- `AAU = W(oz) · A(%)`, yalnızca use ∈ {0 Boil, 3 FWH} **ve** `t ≥ 15 dk` ise; aksi hâlde 0.
- Aynı mantık `BSRecipe::CalcAAU`'da da var (tarif AAU'su).
- Yayımlanmış ad: **Alpha Acid Units / Homebrew Bittering Units**. 15 dk eşiği BeerSmith'e özgü.

---

## 8. Hop oranları (`RecipeHopRate(tarif, faz)`)
- `rate = (Σ W_faz → gram) / (F_R_VOLUME → litre)`, yani **g/L**. Hacim yalnızca parti hacmidir, trub dahil değildir. Hacim 0 ise 0.
- `RecipeTotHopsPhase(faz)`: use = faz olan hopların toplam miktarı. Faz = 0 (boil) ise FWH (use 3) da eklenir.
- `RecipeBoilHopRate` = faz 0 (boil + FWH).
- `RecipeWhirlpoolHopRate` = faz 4.
- Diğer iki çağıran (0x845222, 0x84528e) muhtemelen dry hop (1) ve/veya toplam **[belirsiz]**.
- Dönüşümler: oz → g ×28.349525 (= 0.4535924/16 × 1000, 04 ile tek sabit; **Karar (T1.16 B):** önceki 28.349523125 bırakıldı, fark 6.6e-8 göreli); fl oz → L ×29.57352956/1000. Sabitler motorda `units`'tedir.

## 9. Toplam whirlpool süresi (`TotWhirlpoolTime`)
- `max(F_R_WHIRLPOOL_TIME, max_i int(t_i + 0.48))`. Burada i, use = 4 olan hoplar ve use = 6 (Whirlpool) olan misc kalemleridir.
- Tamsayı dakika döner. Zamanlayıcı ve uyarılar için kullanılır (S_WARN_WHIRLPOOL_TIME); IBU'ya girmez. IBU carry hesabı ekipmanın `F_E_WHIRLPOOL_TIME` alanını kullanır.

## 10. Hop yaşlanması (`HopAgeAlphaFunc` → `HopAlpha`)
- Girdiler:
  - `A0 = F_HA_ALPHA` (%)
  - `HSI = F_HA_HSI` (6 ayda 68 °F'ta kaybedilen alfa %; en fazla 99)
  - `T = F_HA_TEMP` (°F)
  - `gün = F_HA_AGE(ay) · 30`
  - `SF` = depolama faktörü (1.0 / 0.75 / 0.5, §1)
- Formül:
  ```
  k   = −ln(1 − HSI/100) / 180          (1/gün)
  TF  = 0.5^((68 − T)/27)               (her 27 °F soğutma hızı yarıya indirir)
  A   = A0 · exp(−k · TF · gün · SF)
  ```
- Kontrol: HSI = 30, 180 gün, 68 °F, SF = 1 → `A = 0.70 · A0`.
- Yayımlanmış ad: **Garetz (1994) hop storage index modeli** (birinci dereceden bozunma + sıcaklık faktörü; Nickerson & Likens HSI). Sabit 27 °F yarılanma ve depolama faktörleri **[tahmin]** Garetz tablolarından.
- Çıktı `F_HA_ADJ_ALPHA` (%).

---

## 11. Uygulama özeti (sözde kod)

```python
K = 100/1.34

def recipe_ibu(recipe, equip, opts):
    prof = late_extract_sg_profile(recipe)            # 01 §3.6 (tek uygulama) -> [(sg,t)], son t=0
    Tb   = 100 - recipe.altitude_ft*0.3048*0.00324    # °C
    V    = max(recipe.volume_floz + recipe.trub_floz, 0.01) / 128   # gal
    ibu  = 0
    for h in recipe.hops:
        U   = hop_total_util(h, prof, Tb, equip, opts.equation)     # §5.2
        adj = hop_adjust(h, equip.hop_util_pct, recipe.type == 0, opts)  # §4
        h.ibu = adj * U * h.alpha * h.amount_oz * K / V
        ibu += h.ibu
    for g in recipe.fermentables:
        if g.type == 1 and g.ibu_gal_per_lb > 0:
            ibu += g.amount_oz/16 * g.ibu_gal_per_lb / max(V, 0.0001)
    return ibu
```

---

## 12. Test vektörleri (Python ile hesaplandı)

Ortak girdiler (aksi belirtilmedikçe): W = 1 oz, A = %5, t = 60 dk, SG = 1.050, V = 5 gal. Deniz seviyesi, düzeltme çarpanı 1 (araç yolu, leaf/düzeltme yok).

| # | Durum | Kullanım oranı | IBU |
|---|---|---|---|
| 1 | Tinseth | 0.230664 | **17.2137** |
| 2 | Rager | 0.300000 | **22.3881** |
| 3 | Rager, SG 1.070 (GA = 0.1) | 0.272727 | 20.3528 |
| 4 | Garetz, araç yolu (V = V_boil = 5 gal, gerçek HF) | — | **14.6009** |
| 5 | Garetz, tarif yolu (`GaretzUtilization`) | 0.194939 | 14.5477 |
| 6 | Garetz, tarif yolu, SG 1.040 (GF < 1, alt sınır yok) | 0.204657 | — |
| 7 | Whirlpool 80 °C, 20 dk (Ftemp = 0.228993) | 0.031989 | **2.3872** |
| 8 | Rakım 5000 ft → 95.0622 °C (203.112 °F), Ftemp = 0.710328, Tinseth 60 dk | 0.163848 | 12.2274 |
| 9 | `PostBoilUtil(1.050, 60, 20)` | 0.008003 | — |
| 10 | 60 dk hop + whirlpool carry 20 dk: 1 − (1 − 0.230664)(1 − 0.008003) | 0.236821 | 17.6732 |
| 11 | 0 dk (alev kesimi) hop, carry 20 dk: `PostBoilUtil(1.050, 0, 20)` | 0.095353 | 7.1159 |
| 12 | Tarif, pellet (×1.10), Tinseth, F_E_HOP_UTIL = 100 | — | 18.9351 |
| 13 | Tarif, FWH pellet (×1.10 × 1.10), All Grain | — | 20.8286 |
| 14 | Tarif, mash pellet (×1.10 × 0.20), All Grain | — | 3.7870 |
| 15 | İzomerize ekstrakt (form 4): 0.1 fl oz, %50, 5 gal | 1.0 | 74.6269 |
| 16 | Geç ekstrakt: SG 1.040 (60→15 dk) + 1.050 (15→0 dk), 60 dk hop | 0.227040 | — (tek SG 1.040 olsaydı 0.252354) |
| 17 | `Ftemp(100)` / `Ftemp(99.99)` | 1.000000 / 1.008524 | — |
| 18 | HopAlpha: %5, HSI 30, 6 ay, 68 °F, poly torba | — | alfa **3.5000** |
| 19 | HopAlpha: %5, HSI 30, 6 ay, 40 °F, vakum (0.5) | — | alfa 4.5838 |

Kullanım oranı / süre tablosu (SG 1.050, Garetz = tarif yolu `GaretzUtilization`):

| dk | Tinseth | Rager | Garetz |
|---|---|---|---|
| 5 | 0.0460 | 0.0500 | 0.0000 |
| 10 | 0.0836 | 0.0600 | 0.0000 |
| 15 | 0.1145 | 0.0800 | 0.0195 |
| 20 | 0.1397 | 0.1010 | 0.0483 |
| 30 | 0.1773 | 0.1530 | 0.1047 |
| 45 | 0.2117 | 0.2690 | 0.1683 |
| 60 | 0.2307 | 0.3000 | 0.1949 |
| 90 | 0.2467 | 0.3000 | 0.2125 |

Test betiği: `/tmp/claude-0/-home-user/9e16fef0-a8b7-5145-928c-0a9718edc1b6/scratchpad/hop/ibu.py`. Bu bir referans uygulamadır; `PostBoilUtil` döngüsü kayan nokta birikimi dahil birebir taklit edildi.

---

## 13. Açık noktalar / belirsizlikler
- `GetOGPoints`'in tam birimi ve `F_G_USE` / `F_G_TYPE` enum sırası (fermentable spesifikasyonunda doğrulanmalı).
- Form 3/4 miktarının dahili birimi (fl oz varsayıldı) ve izomerize ekstraktta yoğunluk = 1 varsayımı.
- `F_E_HOP_UTIL` varsayılan değeri (100 varsayıldı).
- `O_WHIRLPOOL_UTIL` görünüşte kullanılmıyor. Diğer modüllerde (ör. eski dosya içe aktarma) kullanılıp kullanılmadığı kontrol edilmedi.
- `PostBoilUtil`'deki `τ < 5` istisnası ve 100 °C süreksizliği kodda böyle. Yazılım hatası olabilir ama birebir uyumluluk için korunmalı.
- `ScaleBitterness`'taki form 4 dry hop indeks kayması.


---

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

**Asit modeli `F_R_ACID_MODEL`:** 0 = "MPH 3 Model", 1 = "BW Model" (kodda "BNW"). Reçetede ekipman profilinden gelir (`F_E_ACID_MODEL`). BeerSmith liste varsayılanı **1 (BW)** (AddList'in son parametresinin varsayılan indeks olduğu varsayımına dayanır **[belirsiz]**). **Bizim motorun varsayılanı 0 (MPH)**; BW seçenek olarak kalır ve "doğrulanacak" işaretlidir (F1 kararı 3, bkz. §5.2 ve §6.8).

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
- **Bizim model (F1 kararı 12a):** `g(c) = min(c − 1.9, 0.453·c + 5.0)` (c < 200), c ≥ 200 için 42.0. İki doğrunun kesişimi 6.9/0.547 = 12.614 SRM'dedir; fonksiyon 14 SRM'de sürekli olur (13.99 → 11.337, 14 → 11.342; BeerSmith: 12.09 → 11.342). 12.614 SRM altındaki ve 14 SRM üstündeki maltlarda sonuç BeerSmith ile aynıdır. 200 SRM'deki sıçrama (95.6 → 42.0) kavrulmuş malt için bilinçli bir model seçimi olarak korunur [doğrulanacak]. BeerSmith uyumu modunda eski fonksiyon kullanılır.
- Model adı: kodda geçmiyor. Renk tabanlı ampirik grist pH yaklaşımı, Troester'in "kristal malt asitliği renkle artar, kavrulmuş daha az" bulgusuyla uyumlu **[tahmin]**.

### 5.2 RA → pH eğimi [kesin]
**Kaynak:** `MashRASlope(model, t)`. Girdi t = `F_R_MASH_THICKNESS` (L/kg). Birim: pH / (mEq/L).
```
model 0 (MPH 3):  m = 0.037 + 0.014 · t
model 1 (BW):     m = t · 0.47930613 · 0.17 / 1.5
```
0.47930613, L/kg değerini qt/lb'ye çeviren katsayıdır (1 qt/lb = 2.0864 L/kg). Dolayısıyla BW için `m = 0.17 · t[qt/lb] / 1.5`.
Model 0'da t dönüştürülmeden L/kg olarak kullanılır **[kesin]**. Model adları: "MPH 3" ve "BW (Bru'n Water)" arayüz metinlerinden alındı. Katsayıların literatürdeki kaynağı **[belirsiz]**.

**[doğrulanacak]** Eğim `m` ve mash kalınlığı `t` örtük bir tampon kapasitesi verir: `B = t / m` (mEq/(kg·pH)). MPH için t = 3'te B ≈ 38; BW için kalınlıktan bağımsız B ≈ 18.4, yani bildiğim literatür aralığının (baz maltlar için kabaca 30–50) yaklaşık yarısı. Sonuç: T3'te BW asit önerisi MPH'nin yarısı (1.77 mL / 3.52 mL). **Karar (F1 kararı 3):** MPH varsayılan model, BW seçenek ve "doğrulanacak". BW seçilince ekran `uyarilar` alanına "BW modeli doğrulanacak: MPH'ye göre yaklaşık yarı asit önerir" notunu yazar. Kesin varsayılan, altın testteki ölçülmüş mash pH ile (ALTIN-TEST senaryo 12 ve 13) verilecek.

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
f    = model 1 ise AcidFraction(laktik→1, fosforik→2, pH = 5.4), değilse 1      (BeerSmith uyumu)
       bizim model: f = AcidFraction(asit, hedef pH), iki modelde de (§6.8)
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
f = model 1 ise AcidFraction(asit, 5.4), değilse 1       (BeerSmith uyumu; bizim model: AcidFraction(asit, hedef pH), §6.8)
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
**Bizim model:** pH_t sparge hedefidir (§6.8); model 0 sonucu `AcidFraction(asit, pH_t)` ile bölünür.

**Kenar durum (model 1):** su pH'ı 4.3 ise (alkalinite son noktası) C_T paydası 0 olur; `pH_w ≤ 4.3` için hesap yapılmaz ("sparge suyu zaten asidik, asit gerekmez"). `pH_w ≤ pH_t` ise asit 0 (aşağıdaki "sıfır ya da negatif" kuralı).
Kural 6.3'teki gibidir: ΔpH (mash pH − hedef) < 0 ya da asit = 0 ise sparge sonucu da 0 olur. Bu koşul sparge suyuna değil mash pH'ına bakar **[kesin]**.

### 6.6 Görüntüleme [kesin]
`AcidAmountString`: sıvı asit için `mL` ve hesap 48 tbsp'den küçükse "(x tbsp)" eklenir (1 tbsp = 1/256 gal). Asit malt kullanıcının ağırlık biriminde gösterilir. Asit yoksa " - ".

### 6.7 pH aracı varsayılanları (BSTools Mash pH Tool) [kesin; BeerSmith değerleri]
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

Araçta mash kalınlığı `t = V_mash[L] / W[kg]` ve sparge hacmi doğrudan `F_PH_SPARGE_VOL` olarak alınır. Formüller reçete sürümüyle aynıdır. Bizim uygulamada `F_PH_MODEL` varsayılanı 0 (MPH), ayrı sparge hedefi alanı vardır (§6.8).

### 6.8 Bizim model: düzeltmeler ve uyarılar [tasarım; F1 kararı 3 ve 12]
BeerSmith uyumu için §5–§6 aynen korunur (karşılaştırma ekranı ve altın test). Kendi modelimiz (varsayılan) şu farklarla çalışır:
- **Varsayılan model MPH (0).** BW (1) seçenektir ve "doğrulanacak" işaretlidir (§5.2).
- **Sürekli grist fonksiyonu (12a):** §5.1'deki `min(c − 1.9, 0.453·c + 5.0)`.
- **Ayrışma kesri hedef pH'ta (12b):** mash asidinde (`MiscAcidRA`, `PHAcidPerLiter`) `f = AcidFraction(asit, hedef pH)`, **iki modelde de** (BeerSmith'te yalnız model 1 ve sabit 5.4). Sparge asidinde model 0 sonucu `AcidFraction(asit, sparge hedefi)`'ne bölünür (laktik asidin %97 ayrışması hesaba katılır, BeerSmith'e göre %3 daha fazla asit). Asit malt için aynı kural (pK 3.86). Model 1 sparge zaten `AcidFraction`'a bölüyordu.
- **Ayrı sparge hedefi (12c):** yeni alan `F_R_SPARGE_TARGET_PH`, varsayılan **6.0** [doğrulanacak]. §6.5'teki `pH_t` bu alandan gelir; mash hedefi (`F_R_TARGET_PH`) yalnız mash asidini belirler. BeerSmith uyumu modunda ikisi aynı alandır. Sparge asidinin ΔpH koşulu (§6.5, son paragraf) değişmedi.
- **Tebeşir uyarısı (12d):** model değişmez (tebeşir tam çözünür varsayımı §1.2). Reçetede tebeşir (WA tipi 6) miktarı > 0 ise ekran `uyarilar` alanına "çözünürlük düşük, asitli mash olmadan etkisi abartılı olabilir (tebeşir)" yazar.
- **Uyarı alanı:** hesap çıktısına `uyarilar: [...]` (metin listesi) eklenir; hesap uyarıyla durmaz. Referans uygulama: `uyarilar(model, tuz_gram)` (test betiği 03).

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
| `F_E_ACID_MODEL` / `F_R_ACID_MODEL` | BeerSmith: 1 (BW) **[belirsiz]**; bizim motor: **0 (MPH)** | model (§6.8) |
| `F_R_SPARGE_TARGET_PH` (yeni, bizim) | 6.0 [doğrulanacak] | sparge asidinin hedef pH'ı (§6.8) |
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
10. Grist fonksiyonu `g(c)` 14 SRM'de süreksizdir (13.99 SRM → 12.09, 14 SRM → 11.34). BeerSmith uyumu için korunur; kendi modelimizde sürekli yapıldı (§5.1, §6.8; F1 kararı 12a).
11. Mash asidinde ayrışma kesri hedef pH yerine sabit 5.4'te alınır; model 0 sparge hesabı ise laktik asidin %97 ayrışmasını hesaba katmaz (yaklaşık %3 eksik asit). Sparge hedefi mash hedefiyle aynı alandır: 5.4 için 2.86 mL, yaygın 6.0 hedefi için 2.15 mL (T4 koşullarında). Tebeşirin tam çözündüğü varsayımı alkaliniteyi olduğundan büyük gösterebilir. Kendi modelimizde ayrışma hedef pH'ta alınır, sparge hedefi ayrıdır ve tebeşir için ekran uyarısı verilir (§6.8; F1 kararı 12b, 12c, 12d).

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

Yukarıdaki tablo ve asit malt değerleri **BeerSmith uyumu** modudur. **Bizim model** (§6.8, ayrışma hedef pH 5.4'te, iki modelde de):

| | Model 0 (MPH, varsayılan) | Model 1 (BW) |
|---|---|---|
| Laktik %88, mL/L | 0.246440 | 0.120469 |
| **Laktik %88, toplam (14.7 L)** | **3.623 mL** | **1.771 mL** |
| Asit malt %3, 1. tur | 128.43 g (pH 5.4073) | 62.78 g (pH 5.4036, durur) |
| Asit malt, 2. tur | +4.29 g → **toplam 132.72 g**, pH_adj 5.4002 | |

### T4: Sparge asidi
HCO₃ = 150 ppm, su pH 7.5, hedef 5.4, laktik %88. Toplam su 30 L, W = 4.9 kg, t = 3 → V_sp = 30 − 14.7 = **15.3 L**.
- Model 0 (karbonat dengesi): 0.00220511 mol/L → **2.858 mL**
- Model 1 (BW): 0.00229262 mol/L → **2.971 mL**

Bu değerler BeerSmith uyumu modudur (hedef 5.4). **T4b, bizim model** (model 0 `AcidFraction`'a bölünür; ayrı sparge hedefi):
| sparge hedefi | Model 0 (MPH) | Model 1 (BW) |
|---|---|---|
| 5.4 | 0.00226871 mol/L → **2.940 mL** | 0.00229262 mol/L → **2.971 mL** |
| 6.0 (varsayılan) | 0.00166758 mol/L → **2.161 mL** | 0.00170903 mol/L → **2.215 mL** |

### T9: Grist sürekliliği (bizim model, §5.1)
- `g(13.99)`: BeerSmith 12.090, bizim **11.337**; `g(14)`: 11.342 (iki modelde de); `g(12.614)` = 10.714.
- Tek malt 13 SRM (W = 1 kg): grist pH BeerSmith 5.3459, bizim **5.3530**.

### T10: Varsayılan model ve uyarılar (§6.8)
- Varsayılan model 0 (MPH); 5 g gypsum, tebeşir yok → `uyarilar = []`.
- BW (1) seçili ve 2 g tebeşir → `uyarilar` iki madde: "BW modeli doğrulanacak…" ve "çözünürlük düşük, asitli mash olmadan etkisi abartılı olabilir (tebeşir)".

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


---

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
| ağırlık 9 | oz | 1 lb = 0.4535924 kg, 1 oz = 28.349525 g (motorda tek sabit `units::OZ_G`; 02 §8 de bunu kullanır, T1.16 B) |
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

**Bizim motor (F1 kararı 9):** BeerSmith uyumu notudur; yeni bölümlerde (09, 10, 11) ve motorda tek sabit **1.9635 g/L/vol** (= 44.01/22.414, ideal gaz) kullanılır. 1.9642 ve 1.96042 yalnız BeerSmith karşılaştırma ekranında geçerlidir.

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


---

# 05 — Mash, Ekipman ve Hacim Hesapları (BeerSmith 4 hesap motoru spesifikasyonu)

Bu doküman BeerSmith 4 (Linux) ikili dosyasından decompile edilen fonksiyonlardan çıkarılmış formülleri **kendi ifademizle** verir. Her bölümde kaynak fonksiyon adı belirtilmiştir. "belirsiz" = koddan tam doğrulanamadı; "tahmin" = yayımlanmış formül adı eşleştirmesi bizim yorumumuzdur.

---

## 0. Genel kurallar, iç birimler ve sabitler

### 0.1 İç (veritabanı) birimleri
Motor tüm alanları sabit iç birimlerde saklar; ısı hesapları için önce metriğe çevirir, sonra geri döner.

| Büyüklük | İç birim | Birim kodu (BConvert*) | Not |
|---|---|---|---|
| Hacim | ABD sıvı onsu (fl oz) | 9 / 0x27 | 1 fl oz = 29.57352956 mL; 128 fl oz = 1 gal |
| Ağırlık | ons (oz, avoirdupois) | 9 | 1 lb = 16 oz; 1 lb = **0.4535924 kg** (kodda kullanılan değer) |
| Sıcaklık | °F | 2 | °C kodu = 1, K kodu = 0x28 |
| Uzunluk (rakım) | ft | — | BoilTemp içinde ×0.3048 ile m'ye çevriliyor |

Diğer birim kodları: 7 = kg, 8 = lb, 10 = g, 0x0b = cup (1/16 gal), 0x0c = pint (1/8 gal), 0x0d = quart (1/4 gal), 0x0e = gal, 0x0f = mL, 0x10 = L.

Faydalı türetilmiş dönüşümler:
- 1 fl oz/oz → L/kg: ×(0.02957352956 / (0.4535924/16)) = **×1.0431755**
- 1 fl oz/oz → qt/lb: **×0.5** (araçlarda "/2.0" olarak görünür)
- Mash kalınlığı fonksiyonu (GetMainMashThickness) fl oz/oz → L/kg için sabit **1.043192807606429** kullanır (yukarıdakinden 4. basamakta farklı; kod sabiti olduğu gibi alın).

### 0.2 Global ayarlar (Option tablosu)
| Option | Ad (Option_Name) | Varsayılan | Birim / anlam |
|---|---|---|---|
| O_GRAIN_VOL | Adv.GrainVolume | **0.652** | 1 kg tahılın mash içinde kapladığı hacim, L/kg |
| O_MALT_SPECIFIC_HEAT | Adv.MaltHeatCapacity | **0.380** | Malt özgül ısısı, cal/(g·°C) |
| O_ADV_GRAIN_ABSORB | Adv.Grain.Absorbtion | **0.96** | Tahıl emilimi, fl oz su / oz tahıl (= **1.0014 L/kg** = 0.12 gal/lb) |
| O_ADV_BIAB_ABSORB | Adv.BIAB.Absorb | **0.586** | BIAB emilimi, fl oz/oz (= **0.6113 L/kg**, normalin ~%61'i) |
| O_ADV_STEEP_WATER_GRAIN | Adv.Steep.WG | **4.0** | Demleme (steep) su/tahıl oranı, qt/lb |
| O_MASH_FG_ADJ_TEMP | Recipe.Mash.FGTemp | 153.5 | °F — "ana" mash adımını seçmek için referans |
| O_UNITS_MASH_VOL | Units.MashVol | 13 (qt) | Görüntüleme birimi; 15 (mL) veya 16 (L) seçiliyse su/tahıl L/kg ile, aksi halde qt/lb ile gösterilir |

Suyun özgül ısısı 1 cal/(g·°C) ve yoğunluğu 1 kg/L kabul edilir (formüllerde su hacmi L cinsinden doğrudan "ısı kapasitesi" olarak kullanılır).

### 0.3 Şema varsayılanları (yeni kayıt açıldığında)
Ekipman (InitEquipSchema): tun hacmi 640 fl oz (5 gal = 18.93 L), tun ağırlığı 160 oz (10 lb = 4.536 kg), tun özgül ısısı **0.3** cal/g°C, tun deadspace 0, tun addition 0, deadspace ayarı kapalı, otomatik kaynatma hacmi (F_E_CALC_BOIL) **açık**, kaynatma hacmi 768 fl oz (6 gal), kaynatma süresi 60 dk, boil off 64 fl oz (0.5 gal), boil off saatlik oran bayrağı (F_E_BOIL_RATE_FLAG) **açık**, trub kaybı 0, soğuma büzülmesi **%4**, kazan top-up 0, batch hacmi 640 fl oz, fermentör kaybı 0, fermentör top-up 0, rakım 0.

Mash profili (InitMashSchema): tahıl ağırlığı tabanı 160 oz, tahıl sıcaklığı 72 °F, kaynama sıcaklığı **212 °F** (sabit varsayılan; rakımdan otomatik hesaplanmıyor — bkz. §6.1), tun sıcaklığı 72 °F, sparge 168 °F, pH 5.2, batch sparge kapalı, batch doldurma yüzdesi **%95**, eşit batch **açık**, önce boşalt **kapalı**, ekipmana göre ayarla (F_MH_EQUIP_ADJUST) **açık**, BIAB kapalı, BIAB hacim tabanı 640 fl oz.

Mash adımı (InitMashStepSchema): tip = Infusion, infüzyon 0, adım sıcaklığı 154 °F, adım süresi 40 dk, yükselme süresi 4 dk.

### 0.4 Enum değerleri
| Alan | Değerler |
|---|---|
| F_MS_TYPE (mash adımı tipi) | 0 = Infusion, 1 = Decoction, 2 = Temperature (doğrudan ısıtma) |
| F_R_TYPE (reçete tipi) | 0 = Extract, 1 = Partial Mash, 2 = All Grain, >2 = mead/şarap/cider vb. |
| F_G_USE (fermente edilebilir kullanım yeri) — koddan görülenler | 0 = Mash, 1 = Steep, 2 = Boil, 4 = Primary, 5 = Secondary, 8 = Sparge (3/6/7: whirlpool/tertiary/bottling — sıra **belirsiz**) |
| F_G_TYPE (koddan) | 0 = Grain, 3 = Adjunct (bu ikisi "mash'lenebilir" sayılır); 1, 2, 4 = Sugar/Extract/Dry Extract (geç ekleme yapılabilenler; tam eşleme **belirsiz**) |
| F_SI_CALC (strike/infüzyon aracı) | 0 = İlk strike, 1 = Infüzyon adımı |
| F_DT_TYPE (seyreltme aracı) | 0 = Su, 1 = Wort/bira |
| F_WN_BREW_TYPE (su ihtiyacı aracı) | 0 = Extract, ≠0 = mash'li (all grain / partial) |
| Bool alanlar | F_MH_BIAB, F_MH_BATCH, F_MH_BATCH_EVEN, F_MH_BATCH_DRAIN, F_MH_EQUIP_ADJUST, F_E_CALC_BOIL, F_E_BOIL_RATE_FLAG, F_E_TUN_ADJ_DEADSPACE: 0 = kapalı, 1 = açık |

---

## 1. Tahıl hacmi ve tahıl emilimi

### 1.1 Tahılın kapladığı hacim — `GrainVol`, `BSMash::MashVol`
- **Ne:** Kuru tahılın mash içinde kapladığı (su yerine geçen) hacim.
- **Girdi:** W = tahıl ağırlığı (kg). **Çıktı:** V_grain (L).
- **Formül:** `V_grain = O_GRAIN_VOL × W` (varsayılan 0.652 L/kg).
- `MashVol(Vsu, W) = Vsu + O_GRAIN_VOL × W` → toplam mash hacmi.

### 1.2 Tahıl emilimi — `BSRecipe::RecipeGrainAbsorb` (F_R_GRAIN_ABSORB)
- **Ne:** Mash tahıllarının tuttuğu, geri alınamayan su.
- **Girdi:** W_mash = F_R_MASH_GRAIN_WEIGHT (mash profilinin tahıl ağırlığı = mash'lenen tahıllar, bkz. §2.6).
- **Formül:**
  - BIAB değilse: `A = O_ADV_GRAIN_ABSORB × W_mash` (iç birimde fl oz/oz × oz → fl oz; metrik: **1.0014 L/kg × kg**)
  - F_R_BIAB açıksa: `A = O_ADV_BIAB_ABSORB × W_mash` (**0.6113 L/kg**)

### 1.3 Sparge tahıllarının emilimi — `SpargeAbsorb`, `TotSpargeGrains` (F_R_SPARGE_ABSORB)
- **Ne:** "Sparge" aşamasında eklenen tahılların (F_G_USE = 8 **ve** F_G_TYPE ∈ {0, 3}) tuttuğu su.
- **Formül:** `A_sp = (BIAB ? O_ADV_BIAB_ABSORB : O_ADV_GRAIN_ABSORB) × ΣW_sparge`.

### 1.4 Mash tun hacim ihtiyacı — `MashTunVolFunc` (F_MH_TUN_VOL_NEEDED), `RecipeTunVolNeeded`
- `V_tun_needed(mash) = MashTotWater + O_GRAIN_VOL × W_mash`
- Reçete düzeyi: `V_tun_needed(recipe) = V_tun_needed(mash) + O_GRAIN_VOL × ΣW_sparge` (sparge tahılları da tuna girer).

### 1.5 Toplam mash suyu — `MashTotWaterFunc` (F_MH_TOT_WATER → F_R_MASH_TOT_WATER)
- `MashTotWater = Σ (F_MS_TOT_INFUSION)` — **dekoksiyon tipindeki (1) adımlar hariç** tüm adımlar.
- `F_MS_TOT_INFUSION = F_MS_INFUSION + F_MS_TUN_ADDITION` (`MashStepTotInfusionFunc`).

### 1.6 Mash'te kullanılabilir su — `MashAvailWaterFunc`
- `V_avail = MashTotWater − A − F_R_TUN_DEADSPACE` (ilk süzmede alınabilecek wort).

---

## 2. Isı dengesi formülleri (strike, infüzyon, dekoksiyon)

Tüm ısı formülleri aynı iç dönüşümü yapar: sıcaklıklar °F→°C, hacimler fl oz→L, ağırlıklar oz→kg; sonuç geri °F / fl oz'a çevrilir. Aşağıda **metrik** (°C, L, kg) yazılmıştır.

Ortak semboller:
- W = tahıl ağırlığı (kg), c_m = O_MALT_SPECIFIC_HEAT (0.38)
- M_t = tun kütlesi (kg), c_t = tun özgül ısısı (cal/g°C), V_t = tun hacmi (L)
- F_MH_EQUIP_ADJUST = 0 ise motor **M_t = 0** kullanır (tun ısısı yok sayılır).

### 2.0 Etkin tun kütlesi (kısmi doluluk düzeltmesi) — tüm ısı fonksiyonlarında ortak
- **Ne:** Tun yalnızca mash ile temas eden kısmı oranında ısı alır varsayımı.
- **Formül:** `V_cap = 0.8 × V_t`. Eğer `V_cap > 0` **ve** `V_mash < V_cap` ise `M_t,eff = M_t × V_mash / V_cap`, aksi halde `M_t,eff = M_t`.
- V_mash her fonksiyonda farklı tanımlanır (aşağıda belirtildi). Sabit **0.8** kodda sabittir.
- Yayımlanmış ad: yok (BeerSmith'e özgü; tahmin).

### 2.1 İlk strike suyu sıcaklığı — `MashInTemp`
- **Ne:** Kuru tahıla eklenecek ilk suyun sıcaklığı (mash-in).
- **Girdiler:** V_w = eklenen su (L), T_s = hedef adım sıcaklığı (°C), W, T_g = tahıl sıcaklığı (°C), M_t, c_t, V_t, T_t = tun ön sıcaklığı (°C).
- **Çıktı:** T_strike (°C).
- **V_mash** = O_GRAIN_VOL·W + V_w.
- **Formül:**
  `T_strike = T_s + [ (T_s − T_t)·c_t·M_t,eff + c_m·W·(T_s − T_g) ] / V_w`
- **Kenar durumu:** V_w = 0 ise hesap yapılmaz (alan değişmez / tanımsız).
- **Yayımlanmış formül:** Palmer/Daniels strike denklemi `Tw = (0.41/r)(T2 − T1) + T2` (metrik, r = L/kg) ile aynıdır; BeerSmith 0.41 yerine c_m = 0.38 kullanır ve tun terimini ekler. r = V_w/W yazılırsa tun'suz hali birebir Palmer formülüdür.

### 2.2 Ara infüzyon (sıcak su ekleyerek adım yükseltme) sıcaklığı — `InfuseTemp`
- **Ne:** Mevcut mash'i T1'den T2'ye çıkarmak için V_a litre suyun hangi sıcaklıkta olması gerektiği.
- **Girdiler:** V_a = eklenecek su (L), T2 = hedef (°C), V_w = mash'teki mevcut su (L), T1 = mevcut mash sıcaklığı (°C; önceki adımın adım sıcaklığı), W, M_t, c_t, V_t.
- **V_mash** = O_GRAIN_VOL·W + V_a + V_w.
- **Formül:** `T_inf = T2 + (T2 − T1)·(c_t·M_t,eff + c_m·W + V_w) / V_a`
- **Kenar:** V_a ≤ 0 ise hesap yapılmaz. Sonuç > 100 °C (212 °F) ise mash adımı hata mesajı üretir: "Infusion temperature above boiling. Add more water!" (`MashStepError`).
- **Yayımlanmış formül:** Palmer infüzyon denklemi `Wa = (T2 − T1)(0.41G + Wm)/(Tw − T2)` sıcaklık için çözülmüş hali; BeerSmith c_m=0.38 + tun terimi.

### 2.3 Belirli sıcaklıktaki sudan ne kadar eklenmeli — `InfuseAmount` (Mash Adjust aracı)
- **Girdiler:** T_inf = eklenecek suyun sıcaklığı, T2, V_w, T1, W, M_t, c_t, V_t.
- **V_mash** = O_GRAIN_VOL·W + V_w (eklenen su dahil değil).
- **Formül:** `V_a = (T2 − T1)·(c_t·M_t,eff + c_m·W + V_w) / (T_inf − T2)`
- **Kenar:** T2, T1 ile T_inf arasında **kesin** olarak değilse (T2 ≤ min veya ≥ max) sonuç 0.
- Mash Adjust aracı (`MashAdjustVolFunc`) V_t olarak `1.5 × F_MA_VOLUME` geçirir; araçtaki su/tahıl oranı = `(F_MA_VOLUME + V_a)/F_MA_GRAIN` (fl oz/oz) ÷ 2 → qt/lb.

### 2.4 Dekoksiyon hacmi — `DecoctVol`
- **Ne:** T1'deki mash'ten ne kadarının çekilip kaynatılması (T_b) ve geri karıştırılınca T2'ye ulaşılması gerektiği.
- **Girdiler:** T2 = hedef, V_w = mash'teki su (L), T1 = mevcut sıcaklık, W, M_t, c_t, V_t, T_b = kaynama sıcaklığı (F_MH_BOIL_TEMP, varsayılan 100 °C).
- **V_mash** = O_GRAIN_VOL·W + V_w.
- **Formül:**
  `f = [(T2 − T1)/(T_b − T1)] × [(c_t·M_t,eff + c_m·W + V_w) / (c_m·W + V_w)]`, `f = min(f, 1)`
  `V_decoct = V_mash × f` (L mash, tahıl+su)
- **Kenarlar:** T1 ≥ T_b → 0. W = 0 ve V_w = 0 → 0. f en fazla 1 (tüm mash). Mash'te su yoksa (start vol = 0) adım hata verir: "Decoction Error: Must have some water in mash to Decoct it!".
- **Yayımlanmış formül:** Palmer dekoksiyon yaklaşımı `V_d = V_t (T2 − T1)/(T_b − T1)`; BeerSmith tun ısı kapasitesi oranıyla çarpar (tahmin: kendi uzantısı).
- Dekoksiyon aracı (`DecoctionMashVolFunc`) V_t yerine `1.2 × F_D_START_VOL` geçirir; su/tahıl = `F_D_START_VOL / F_D_GRAIN / 2` (qt/lb).

### 2.5 Mash adımlarının toplu yeniden hesabı — `BSMash::RecalculateMash` (BIAB değilse)
Adımlar sırayla (i = 0..n−1) işlenir. `V_cum` = o ana kadar eklenmiş su (başlangıç 0).

Her adım için:
1. Adıma kopyalanır: F_MS_GRAIN_WEIGHT = F_MH_GRAIN_WEIGHT, F_MS_START_VOL = V_cum, tun HC/hacim/sıcaklık, tahıl sıcaklığı; tun kütlesi (EQUIP_ADJUST kapalıysa 0).
2. `V_add = F_MS_INFUSION`. **Yalnızca i = 0 ve F_MH_TUN_DEADSPACE ≠ 0 ise:** `V_add += F_MH_TUN_DEADSPACE` ve F_MS_TUN_ADDITION = F_MH_TUN_DEADSPACE; diğer adımlarda TUN_ADDITION = 0.
3. Tipe göre:
   - **Infusion (0):**
     - i = 0: START_TEMP = 0; `INFUSION_TEMP = MashInTemp(V_add, T_step, W, T_grain, M_t, c_t, V_t, T_tun)`.
     - i > 0: START_TEMP = önceki adımın T_step; `INFUSION_TEMP = InfuseTemp(V_add, T_step, V_cum, T_prev, W, M_t, c_t, V_t)`.
     - DECOCTION_AMT = 0.
   - **Decoction (1):**
     - i = 0: ilk adım dekoksiyon olamaz; strike gibi hesaplanır (`MashInTemp`), DECOCTION_AMT = 0.
     - i > 0: `DECOCTION_AMT = DecoctVol(T_step, V_cum, T_prev, W, M_t, c_t, V_t, T_boil)`, INFUSION_TEMP = 0.
   - **Temperature (2):** bu fonksiyonda sıcaklık hesaplanmaz; tekil güncellemede (`UpdateInfusionTemp`) `INFUSION_TEMP = T_step` atanır.
4. Tip ≠ Decoction ise `V_cum += V_add`.

`UpdateInfusionTemp` (tek adım düzenleme ekranı): Infusion tipinde START_TEMP = 0 ise ilk adım kabul edilir → `MashInTemp(INFUSION + TUN_ADDITION, …, T_tun)`; değilse `InfuseTemp(INFUSION, T_step, START_VOL, START_TEMP, …)`. Diğer tiplerde INFUSION_TEMP = T_step.

### 2.6 Mash tahıl ağırlığı ve profil ölçekleme — `RecipeRecalculate`, `ScaleFromGrainWeight`
- **Mash tahıl ağırlığı** W_mash = Σ F_G_AMOUNT, koşul: F_G_USE < 1 (Mash) **ve** F_G_TYPE ∈ {0, 3}.
- Reçete yeniden hesabında (yalnızca F_R_TYPE ∈ {1, 2}) mash profiline kopyalanır: tun kütlesi, tun HC, tun hacmi (ekipmandan), ve
  `F_MH_TUN_DEADSPACE = F_R_TUN_ADDITION + (F_R_TUN_ADJ_DEADSPACE ? F_R_TUN_DEADSPACE : 0)`.
  BIAB ise `F_MH_BIAB_VOL = F_R_BIAB_MASH_VOL`.
- **Ölçekleme** (`ScaleFromGrainWeight(W_yeni)`): BIAB değilse ve eski taban W_eski > 0, W_yeni > 0, W_eski ≠ W_yeni ise her adımda (infüzyon > 0) `INFUSION ← W_yeni × INFUSION / W_eski` (su/tahıl oranı korunur). Sonra F_MH_GRAIN_WEIGHT = W_yeni ve mash yeniden hesaplanır.

### 2.7 Adım su/tahıl oranı — `MashStepWaterGrainFunc`, `ScaleWaterGrain`
- Görüntüleme: Decoction adımında `r = START_VOL / W`, diğerlerinde `r = (START_VOL + INFUSION) / W` (iç birim fl oz/oz); gösterim birimi O_UNITS_MASH_VOL mL/L ise L/kg, aksi halde qt/lb.
- Kullanıcı oran girerse: `INFUSION ← r × W − START_VOL` (iç birimde); yalnızca |değişim| > 1 fl oz ise yazılır.

### 2.8 Ana mash kalınlığı — `GetMainMashThickness` (F_MH_THICKNESS, L/kg)
- Grain weight > 0 olan adımlar arasından adım sıcaklığı O_MASH_FG_ADJ_TEMP'e (153.5 °F) **en yakın** olan seçilir (fark < 40 °F şartı; eşitlikte ilk bulunan kalır).
- Seçilen adımda: Decoction ise `r = START_VOL/W`, değilse `r = (START_VOL + TOT_INFUSION)/W`.
- Bulunamazsa: `r = F_MH_TOT_WATER / F_MH_GRAIN_WEIGHT` (W = 0 ise 0).
- Sonuç: `r × 1.043192807606429` (fl oz/oz → L/kg).

---

## 3. BIAB (tam hacim, sparge'sız mash)

### 3.1 BIAB toplam mash suyu — `RecipeBIABMashVol` (F_R_BIAB_MASH_VOL)
- **Formül:** `V_BIAB = V_preboil + F_R_TUN_DEADSPACE − F_R_TOP_UP_KETTLE + A_BIAB`
  - V_preboil = F_R_BOIL_VOL (ekipmandan), A_BIAB = O_ADV_BIAB_ABSORB × W_mash (§1.2).
  - Not: BIAB'de deadspace her zaman eklenir (F_R_TUN_ADJ_DEADSPACE bayrağına bakılmaz) ve F_E_TUN_ADDITION kullanılmaz.

### 3.2 BIAB adım dağıtımı — `BSMash::RecalculateBIAB`
- Adımlar **sondan başa** işlenir. `V_rem = F_MH_BIAB_VOL`.
- Her adım k için TUN_ADDITION = 0. k > 0 ve tip ≠ Decoction ise `V_rem −= INFUSION_k`.
- k = 0: `INFUSION_0 = V_rem` (kalan tüm su ilk adıma), START_VOL = 0.
- k > 0: `START_VOL_k = V_rem` (o adımdan önce mash'te bulunan su).
- Sıcaklık/dekoksiyon hesapları §2.5 ile aynı (ilk adım `MashInTemp(INFUSION_0, …)`, sonrakiler `InfuseTemp` / `DecoctVol`).
- BIAB'de profil ölçekleme infüzyonları değiştirmez (§2.6).

---

## 4. Sparge

### 4.1 Sparge suyu — `RecipeSpargeVolFunc` (F_R_SPARGE_VOL)
- **Formül:**
  `V_sparge = V_preboil + A + A_sp + F_E_TUN_DEADSPACE − F_E_TOP_UP_KETTLE − MashTotWater`
  - A = mash tahıl emilimi (§1.2), A_sp = sparge tahılı emilimi (§1.3), MashTotWater = §1.5 (deadspace/tun addition ilaveleri dahil).
- Sonuç negatif olabilir; batch hesabında 0'a kırpılır.
- BIAB'de MashTotWater = V_BIAB olduğundan sonuç ≈ A_sp (sparge tahılı yoksa 0).
- Mantık: Kazana girmesi gereken = pre-boil; mash'ten çıkamayan = emilim + deadspace; kazana doğrudan eklenen top-up düşülür.
- **F_E_TUN_ADDITION ("Recoverable Mash Deadspace")** yalnızca ilk adıma eklenir (MashTotWater'ı büyütür) ve kayıp sayılmaz → sparge suyunu aynı miktarda azaltır; toplam su değişmez.

### 4.2 Batch sparge planı — `GetBatchSpargeSize(i)`, `SpargeStep`
`SpargeStep` metin oluşturur: reçete Extract veya BIAB ise "If steeping, remove grains, and prepare to boil wort" (sparge yok). Batch sparge kapalıysa (F_R_BATCH_SPARGE = F_MH_BATCH = 0) fly sparge: "V_sparge suyu T_sparge'da" metni. Batch açıksa i = 0..49 için `GetBatchSpargeSize` çağrılır; sonuç ≤ 0 (ve "drain" bayrağı yok) olunca döngü biter.

Tanımlar (hepsi aynı hacim biriminde):
- S = max(V_sparge, 0)
- A_t = A + A_sp
- V_g = O_GRAIN_VOL × F_MH_GRAIN_WEIGHT
- P = F_MH_BATCH_PCT (varsayılan %95), V_tun = F_R_TUN_VOL (ekipman F_E_MASH_VOL)
- **Cap** (bir batch'te süzülebilecek azami hacim) = `P·V_tun/100 − V_g − A_t`; Cap < 0 ise `Cap = P·V_tun/100`
- **M** (ilk süzmede çıkacak wort) = `F_MH_TOT_WATER − A_t`
- **R** (ilk süzmeden önce tun'a eklenebilecek boşluk) = `Cap − M`
- E = F_MH_BATCH_EVEN, D = F_MH_BATCH_DRAIN

Adım i (i = 0 "ilk süzmeden önce ekleme / ilk süzme", i ≥ 1 sparge batch'leri):

**(a) "Boşalt" durumu:** i = 0 ve **değil** (R > 0 ve D = 0) → sonuç = M, `drain = true` (metin: "mash tun'u boşalt"). D = 1 ise i = 0 daima budur.

**(b) Eşit batch (E = 1 ve D = 0):**
- n = 2; `size = (S + M)/2` (eğer R ≥ 0 veya i < 1), aksi halde `size = S`.
- `while Cap < size and n < 10:` n += 1; `size = (S+M)/n` (R ≥ 0 veya i < 1) aksi halde `S/(n−1)`.
- i = 0: `size − M ≥ 0` ise sonuç `size − M` (ilk süzmeden önce eklenecek su); değilse sonuç M, drain = true.
- i ≥ 1: `int(n + 0.3) ≤ i` ise 0; değilse `size`.
- Yorum: toplam wort (M + S) n eşit parçaya bölünür; her parça ≤ Cap.

**(c) Doldurarak batch (E = 0) veya drain+even (D = 1 ve E = 1):**
- R = max(R, 0).
- i = 0: sonuç `min(S, R)` (tun'u kapasiteye kadar doldur).
- i ≥ 1: D = 0 ve E = 0 ise `S ← S − R`.
  - E = 0: `(i−1)·Cap ≤ S` ise sonuç `min(Cap, S − (i−1)·Cap)`; değilse 0. (Her batch Cap kadar, sonuncusu kalan.)
  - E = 1 (yalnız D = 1 ile ulaşılır): `k = int(S/Cap + 0.99)` (Cap ≤ 0 ise Cap = 1); sonuç `S/k`, i > k ise 0.

Kenar: döngü üst sınırı 50 adım; eşit batch sayısı en fazla 10.

---

## 5. Kaynatma, soğuma, kayıplar ve hacim zinciri

### 5.1 Toplam boil-off — `BSEquip::TotBoilOffFunc` (F_E_TOT_BOIL_OFF → F_R_BOIL_OFF)
- F_E_BOIL_RATE_FLAG = 0: `BO = F_E_BOIL_OFF` (toplam hacim)
- F_E_BOIL_RATE_FLAG = 1 (varsayılan): `BO = F_E_BOIL_OFF × (F_E_BOIL_TIME / 60)` (saatlik oran × saat)

### 5.2 Pre-boil (kaynatma başı) hacmi — `BSEquip::EquipRecalculate` (F_E_BOIL_VOL → F_R_BOIL_VOL)
- Yalnızca F_E_CALC_BOIL = 1 ise hesaplanır; aksi halde kullanıcı girişi.
- c = min(F_E_COOL_PCT, 99) (yüzde)
- **Formül:** `V_preboil = BO + (V_batch − V_topup_ferm + V_trub) / (1 − c/100)`
  - V_batch = F_E_BATCH_VOL (fermentöre giren), V_topup_ferm = F_E_TOP_UP, V_trub = F_E_TRUB_LOSS.
- Not: Kazan top-up (F_E_TOP_UP_KETTLE) bu formüle girmez; o, mash/sparge suyundan düşülür (§3.1, §4.1).

### 5.3 Post-boil hacmi — `BSEquip::PostBoilVolFunc`, `BSRecipe::PostBoilVolFunc`
- `V_postboil = V_preboil − BO` (sıcak, soğuma öncesi).

### 5.4 Soğuma büzülmesi — `EquipCoolLossFunc` (F_E_COOL_LOSS)
- `V_cool = (c/100) × V_postboil`. Varsayılan c = %4 (suyun 100 °C → 20 °C arası ~%4 hacim kontraksiyonu; bilinen pratik değer).

### 5.5 Pre-trub hacmi — `PreTrubLossVolFunc` (F_E_PRE_TRUB_LOSS)
- `V_pretrub = V_batch + V_trub`.

### 5.6 Buharlaşma oranı — `EquipEvapRate` (F_E_NEW_EVAP_RATE, %/saat)
- `evap% = BO × 100 / (V_preboil' × t/60)`, V_preboil' = V_preboil (≤ 0 ise 1 alınır), t = F_E_BOIL_TIME (dk).

### 5.7 Şişeleme hacmi — `EquipBottleVolFunc`, `BSRecipe::FinalVolFunc`
- Ekipman: `V_bottle = V_batch − F_E_FERMENTER_LOSS`.
- Reçete: starter dahil değilse aynı; F_R_INCLUDE_STARTER = 1 ise `V_bottle = V_batch + V_starter (+ V_ikinci_starter, eğer F_R_USE_SECOND_STARTER) − F_R_FERMENTER_LOSS`.

### 5.8 Toplam su — `BSRecipe::TotalWaterFunc`
- F_R_TYPE ∈ {1, 2} (Partial/All Grain): `V_total = MashTotWater + V_sparge + V_topup_kettle + V_topup_ferm`
- Extract ve kaynatma malzemesi yok (F_R_HAS_BOIL_ING = 0): `V_total = V_batch`
- Extract (kaynatmalı): `V_total = V_preboil + V_topup_ferm`

### 5.9 Steep (demleme) ve extract yardımcıları
- `SteepVolFunc` (F_R_STEEP_VOL): `V_steep = min( V_preboil − V_boilgrains , O_ADV_STEEP_WATER_GRAIN × W_steep[lb] × 32 fl oz/qt )`; iç birimde `4.0 × (oz/16) × 32`. Metrik: 4 qt/lb = **8.345 L/kg**.
- `BoilGrainWeight` / `BoilGrainVolume`: F_G_USE = 2 olan fermente edilebilirlerin F_G_AMOUNT / F_G_VOLUME toplamı (`TotalGrainUseField`). SteepGrainVolume (use = 1), PrimaryGrainVolume (use = 4), SecondaryGrainVolume (use = 5) aynı mantık. F_G_VOLUME ("çözünmüş hacim") başka modülde hesaplanır (**belirsiz**, bu dokümanın kapsamı dışında).
- `BoilWaterAdd`: `V_boil_water = V_preboil − V_steep − V_boilgrains` (steep sonrası kazana eklenecek su).
- `PrimaryWaterAdd`: (F_R_TYPE ≤ 2 veya kaynatma malzemesi varsa) `V = V_batch − V_postboil − V_primarygrains − V_topup_ferm`; aksi halde post-boil terimi 0.

### 5.10 Pre-boil gravite tahmini — `GetEstPostMashOG` (ilişkili)
- `SG_preboil = 1 + (OG − 1) × (V_batch + V_trub) / V_preboil` (V_preboil > 0 ise). Soğuma ve top-up'ı dikkate almaz. Tek uygulama: `mash::preboil_sg`; 01 §3.5 onu çağırır (T1.16 B).

### 5.11 Hacim zinciri — akış şeması

```
                          ┌──────────────────────────────────────────────┐
  MASH SUYU (MashTotWater)│ Σ adım infüzyonları (dekoksiyon hariç)        │
  = strike + step         │ + ilk adıma: tun_addition (+ deadspace*)     │
    infüzyonları          └──────────────────────────────────────────────┘
        │       (*F_R_TUN_ADJ_DEADSPACE açıksa)
        │  − tahıl emilimi A = 0.96 fl oz/oz (≈1.0 L/kg) × W_mash
        │  − tun deadspace (F_E_TUN_DEADSPACE)
        ▼
  İLK SÜZME (M)  +  SPARGE SUYU (V_sparge)  − sparge tahıl emilimi A_sp
        │            (V_sparge = V_preboil + A + A_sp + deadspace − topup_kettle − MashTotWater)
        │  + kazan top-up (F_E_TOP_UP_KETTLE)
        ▼
  PRE-BOIL  V_preboil = BO + (V_batch − topup_ferm + V_trub)/(1 − c/100)
        │  − boil-off BO  (= oran × süre/60   veya sabit hacim)
        ▼
  POST-BOIL (sıcak)  V_postboil = V_preboil − BO
        │  − soğuma büzülmesi  c% × V_postboil   (varsayılan %4)
        ▼
  SOĞUK WORT (kazanda) = V_postboil × (1 − c/100)
        │  − trub/chiller kaybı V_trub
        │  + fermentör top-up suyu
        ▼
  FERMENTÖR  V_batch (F_E_BATCH_VOL = F_R_VOLUME)
        │  − fermentör/transfer kaybı (F_E_FERMENTER_LOSS)
        │  + yeast starter (opsiyonel)
        ▼
  ŞİŞELEME / FIÇI  V_bottle

  TOPLAM SU = MashTotWater + V_sparge + topup_kettle + topup_ferm
  BIAB:      MashTotWater = V_BIAB = V_preboil + deadspace − topup_kettle + 0.586 fl oz/oz × W (≈0.611 L/kg)
             V_sparge ≈ A_sp (normalde 0)
```

### 5.12 "Water Needed" aracı (WN*) — bağımsız basit model
Bu araç reçete motorundan **ayrıdır** ve soğuma kaybını farklı tabanda hesaplar:
- `WNCoolLoss`: `V_cool = (V_ferm − V_topup) × (c/100) / (1 − c/100)` (trub hariç taban — ekipman formülünde trub dahildir; fark bilinçli/bilinmiyor → **belirsiz**)
- `WNBoilVol`: `V_boil = (V_ferm − V_topup) + V_trub + V_cool + V_boiloff` (boil-off burada girdi hacmi)
- `WNBoilPct`: `boil% = V_boiloff × 100 / V_boil`
- `WNGrainAbsorb`: `A = O_ADV_GRAIN_ABSORB × W` (BIAB ayrımı yok)
- `WNSpargeWater`: `V_sparge = V_boil + A − V_topup_kettle + V_lauter_deadspace − V_mash`
- `WNTotalWaterVol`: Extract (0): `V_boil + V_topup`; diğer: `V_sparge + V_mash + V_topup_kettle + V_topup`
- `WNBottleVol`: `V_ferm − V_loss_ferm`

---

## 6. Kaynatma araçları ve sıcaklık

### 6.1 Rakıma göre kaynama sıcaklığı — `BoilTemp`
- **Girdi:** h = rakım (ft, F_R_ALTITUDE ← F_E_ALTITUDE). **Çıktı:** °C (iç olarak °F'ye çevrilir).
- **Formül:** `T_b = 100 − h × 0.3048 × 0.00324` → kilometre başına **3.24 °C** düşüş (doğrusal).
- Kullanım: yalnızca hop kullanım hesabında (`HopTotalUtil`). Mash dekoksiyonunun kullandığı F_MH_BOIL_TEMP (varsayılan 212 °F) bu fonksiyondan **beslenmiyor** (koddan görülen bağlantı yok; **belirsiz**).
- Yayımlanmış: barometrik yaklaşımın doğrusal hali (~1 °C/300 m); tahmin.

### 6.2 Boil-off aracı — `BoilOffFunc`, `BoilOffVolRateFunc`, `BoilCoolLossFunc`, `BoilEndVolFunc`, `BoilEndGravityFunc`
- V0 = F_B_START_VOL, e = F_B_EVAP_RATE (%/saat), t = F_B_BOIL_TIME (dk), c = F_B_COOL_LOSS_RATE (%), SG0 = F_B_START_GRAVITY
- Saatlik buharlaşma: `V_rate = e × V0 / 100`
- Toplam boil-off: `BO = t × e × V0 / 6000`
- Soğuma kaybı: `V_cool = (c/100) × (V0 − BO)`
- Son hacim: `V_end = V0 − BO − V_cool`
- Son gravite: `SG_end = 1 + (SG0 − 1) × V0 / V_end`; V_end = 0 ise 1.0. (Not: soğuma büzülmesi de konsantre eder kabulü.)

---

## 7. Parti-gyle — `PartiFrac2`, `PartiVol1/2`, `PartiOG1/2`
- **Girdiler:** OG = F_PG_OG (bölünmemiş partinin tahmini OG'si), V = F_PG_VOL, f1 = F_PG_FRAC1 (% ilk süzme).
- `f2 = 100 − f1`
- `V1 = V × f1/100`, `V2 = V × f2/100`
- `k = 0.5^(3·f1/100)`
- `OG1 = 1 + (OG − 1) × (1 − 0.5·k)`
- `OG2 = 1 + (OG − 1) × 0.5·k`
- **Not:** Bu ampirik model toplam şeker puanlarını korumaz (f1·(OG1−1) + f2·(OG2−1) ≠ 100·(OG−1); örn. %50/%50'de ortalama puan OG puanının yarısı). OG girdisinin anlamı veya modelin kaynağı **belirsiz**; formülü olduğu gibi uygulayın. Yayımlanmış ad: bilinmiyor (tahmin: BeerSmith'e özgü üstel azalma yaklaşımı).

---

## 8. Seyreltme ve gravite ayarı

### 8.1 Seyreltme — `DilutionTotVolFunc`, `DilutionFinalGravityFunc`
- `V_tot = V0 + V_add`
- V_add < 0 ise: `SG = SG0` (değişmez).
- Su ile (F_DT_TYPE = 0): `SG = 1 + (SG0 − 1) × V0 / V_tot`
- Wort/bira ile (1): `SG = 1 + [(SG0 − 1)·V0 + (SG_add − 1)·V_add] / V_tot`
- Yayımlanmış ad: gravite puanı kütle dengesi (standart karışım formülü).

### 8.2 Hedef graviteye seyreltmek için su — `GravityAdjWaterVol`
- `V_water = V × (SG − 1) / (SG_target − 1) − V`, negatifse 0. Tek uygulama: 01 §9.4 (`density::gravity_adjust_water`, birimsiz hacim); 05 kopya taşımaz (T1.16 B).

---

## 9. Çeşitli: misc malzeme hacmi — `BSMisc::ToFluidOz`
Misc malzemenin miktarını (F_M_AMOUNT, birim F_M_UNITS) fl oz'a çevirir; ağırlıklar 1 g = 1 mL kabul edilir:
| F_M_UNITS | Dönüşüm | Muhtemel birim (tahmin) |
|---|---|---|
| 0 | x/1000 g → mL | mg |
| 1 | x g → mL | g |
| 2 | değişmez | oz (fl oz kabul) |
| 3 | lb → g → mL | lb |
| 4 | x·1000 → mL | kg |
| 5 | mL | mL |
| 6 | x·4.93 mL | tsp |
| 7 | x·14.78 mL | tbsp |
| 8 | 1/16 gal | cup |
| 9 | 1/8 gal | pint |
| 10 | 1/4 gal | quart |
| 11 | L | L |
| 12 | gal | gal |

---

## 10. Uygulama için referans Python (kendi ifademizle) ve test vektörleri

```python
FLOZ_L = 29.57352956/1000.0           # 1 fl oz -> L
LB_KG  = 0.4535924; OZ_KG = LB_KG/16
GRAIN_VOL = 0.652                       # O_GRAIN_VOL (L/kg)
C_MALT    = 0.380                       # O_MALT_SPECIFIC_HEAT
ABS_STD   = 0.96*FLOZ_L/OZ_KG           # O_ADV_GRAIN_ABSORB  -> 1.0014 L/kg
ABS_BIAB  = 0.586*FLOZ_L/OZ_KG          # O_ADV_BIAB_ABSORB   -> 0.6113 L/kg

def tun_eff(m_tun, v_mash, v_tun):
    cap = 0.8*v_tun
    return m_tun*v_mash/cap if (cap > 0 and v_mash < cap) else m_tun

def strike_temp(Vw, T_step, W, T_grain, m_tun=0, c_tun=0, V_tun=0, T_tun=None):
    if Vw == 0: return None
    T_tun = T_grain if T_tun is None else T_tun
    me = tun_eff(m_tun, GRAIN_VOL*W + Vw, V_tun)
    return T_step + ((T_step-T_tun)*c_tun*me + C_MALT*W*(T_step-T_grain))/Vw

def infusion_temp(Va, T2, Vw, T1, W, m_tun=0, c_tun=0, V_tun=0):
    if Va <= 0: return None
    me = tun_eff(m_tun, GRAIN_VOL*W + Va + Vw, V_tun)
    return T2 + (T2-T1)*(c_tun*me + C_MALT*W + Vw)/Va

def infusion_amount(Tinf, T2, Vw, T1, W, m_tun=0, c_tun=0, V_tun=0):
    if not (min(T1,Tinf) < T2 < max(T1,Tinf)): return 0.0
    me = tun_eff(m_tun, GRAIN_VOL*W + Vw, V_tun)
    return (T2-T1)*(c_tun*me + C_MALT*W + Vw)/(Tinf-T2)

def decoct_vol(T2, Vw, T1, W, m_tun=0, c_tun=0, V_tun=0, Tb=100.0):
    if not T1 < Tb or (W == 0 and Vw == 0): return 0.0
    Vm = GRAIN_VOL*W + Vw
    me = tun_eff(m_tun, Vm, V_tun)
    f = (T2-T1)/(Tb-T1) * (c_tun*me + C_MALT*W + Vw)/(C_MALT*W + Vw)
    return Vm*min(f, 1.0)

def boil_temp_C(alt_ft): return 100.0 - alt_ft*0.3048*0.00324

def equip_boil_vol(batch, topup, trub, cool_pct, boil_off_total):
    c = min(cool_pct, 99.0)
    return boil_off_total + (batch - topup + trub)/(1 - c/100)

def batch_sparge(i, S, absorb, W, tun_vol, pct, mash_water, even, drain):
    """GetBatchSpargeSize -> (hacim, drain_bayragi)"""
    S = max(S, 0.0)
    cap = pct*tun_vol/100 - GRAIN_VOL*W - absorb
    if cap < 0: cap = pct*tun_vol/100
    M = mash_water - absorb
    R = cap - M
    if not (i != 0 or (R > 0 and not drain)):
        return M, True
    if even and not drain:
        n = 2.0
        size = (S+M)/2 if (R >= 0 or i < 1) else S
        while cap < size and n < 10:
            n += 1
            size = (S+M)/n if (R >= 0 or i < 1) else S/(n-1)
        if i == 0:
            return (size-M, False) if size-M >= 0 else (M, True)
        return (0.0 if int(n+0.3) <= i else size), False
    R = max(R, 0.0)
    if i == 0: return min(S, R), False
    if not drain and not even: S -= R
    if not even:
        return (min(cap, S-(i-1)*cap) if (i-1)*cap <= S else 0.0), False
    if cap <= 0: cap = 1.0
    n = int(S/cap + 0.99)
    return (0.0 if n < i else S/n), False

def parti(OG, vol, f1):
    k = 0.5**(3*f1/100)
    return dict(vol1=vol*f1/100, vol2=vol*(100-f1)/100,
                og1=1+(OG-1)*(1-0.5*k), og2=1+(OG-1)*0.5*k)
```

### Test vektörleri (yukarıdaki kodla hesaplandı; °C, L, kg)

| # | Senaryo | Girdi | Beklenen çıktı |
|---|---|---|---|
| TV1 | Strike, tun yok (EQUIP_ADJUST=0) | W=5 kg, V_w=15 L (3 L/kg), T_g=20, T_s=66 | **T_strike = 71.827 °C** |
| TV2 | Strike, tun dahil | + M_t=4 kg, c_t=0.12, V_t=40 L, T_t=20 (V_mash=18.26 < 32 → M_eff=2.2825) | **72.667 °C** |
| TV2b | Strike, varsayılan tun | M_t=4.536 kg, c_t=0.3, V_t=18.93 L (V_mash ≥ 0.8·V_t → tam kütle) | **76.000 °C** |
| TV3 | Ara infüzyon miktarı | 66→72 °C, 100 °C su, V_w=15, W=5 | **V_a = 3.621 L** (geri kontrol: InfuseTemp → 100.0 °C) |
| TV3b | Ara infüzyon sıcaklığı | V_a=4 L, 66→72, V_w=15, W=5 | **T_inf = 97.350 °C** |
| TV4 | Dekoksiyon | 66→72, V_w=15, W=5, T_b=100 | **3.222 L mash**; 50→66 için **5.843 L** |
| TV5 | Kaynama sıcaklığı | 1000 m (3280.84 ft) | **96.760 °C** |
| TV6 | Ekipman zinciri | V_batch=23, top-up=0, trub=1, cool=%4, boil-off 4 L/saat × 60 dk | **V_preboil = 29.000 L**, post-boil 25.000, soğuma kaybı 1.000, soğuk wort 24.0 → −1 trub → 23 |
| TV7 | All-grain toplam su (TV6 + mash) | W=5 kg, emilim 1.0014 L/kg → A=5.007; deadspace 1 L (ADJ açık → ilk adım 15+1=16 L); top-up kettle 0 | **V_sparge = 19.007 L**, **toplam su = 35.007 L**, tun hacim ihtiyacı 19.26 L |
| TV8 | BIAB | V_preboil=29, deadspace 1, W=5, emilim 0.6113 L/kg | **V_BIAB = 33.057 L**; strike (66 °C hedef, 20 °C tahıl, tun yok) **68.644 °C** |
| TV9a | Batch sparge, eşit (E=1,D=0) | TV7 değerleri, V_tun=40 L, %95 | Cap=29.73, M=10.993, R=18.74 → i0: **4.007 L ekle** (ilk süzmeden önce), i1: **15.000 L** |
| TV9b | Batch sparge, doldurarak (E=0,D=0) | aynı | i0: **18.740 L ekle**, i1: **0.267 L** |
| TV9c | Önce boşalt + eşit (D=1,E=1) | aynı | i0: **DRAIN 10.993 L**, i1: **19.007 L** |
| TV9d | Küçük tun 25 L, doldurarak | aynı | Cap=15.49 → i0: **4.490 L**, i1: **14.517 L** |
| TV10 | Parti-gyle %60/%40 | OG=1.060, V=40 L | V1=**24.0 L**, V2=**16.0 L**, OG1=**1.0514**, OG2=**1.0086** (k=0.28717) |
| TV11 | Seyreltme | 20 L @1.060 + 5 L su / + 5 L @1.040 wort | **1.048** / **1.056** |
| TV12 | Pre-boil SG tahmini | OG 1.050, batch 23, trub 1, preboil 29 | **1.0414** |
| TV13 | Boil aracı | V0=25 L, %10/saat, 60 dk, %4 soğuma, SG0=1.050 | BO=**2.5**, soğuma **0.9**, V_end=**21.6**, SG_end=**1.0579** |
| TV14 | Water Needed aracı | V_ferm=23, top-up 0, trub 1, cool %4, boil-off 4, mash 15, lauter deadspace 1, W=5 | V_cool=**0.958**, V_boil=**28.958**, sparge=**19.966**, toplam=**34.966** |
| TV15 | Steep oranı | 4 qt/lb | **8.345 L/kg** |
| — | Emilim sabitleri | 0.96 / 0.586 fl oz/oz | **1.0014 / 0.6113 L/kg** |

---

## 11. Belirsizlikler / dikkat edilecekler
1. **F_MH_BOIL_TEMP** (dekoksiyon kaynama sıcaklığı) rakımdan otomatik türetilmiyor gibi görünüyor (varsayılan 212 °F); `BoilTemp` yalnızca hop hesabında. — belirsiz.
2. **Ekipman ve WN aracı soğuma tabanı farkı:** ekipman (batch − topup + trub)/(1 − c), WN aracı (ferm − topup)·c/(1 − c) + trub. Kendi uygulamanızda ekipman formülünü esas alın.
3. **Parti-gyle** modeli puan korumuyor (§7) — belirsiz.
4. **F_G_USE / F_G_TYPE** enum sıraları kısmen koddan çıkarıldı (§0.4).
5. `GetMainMashThickness` sabiti 1.043192807606429, birim dönüşümünden (1.0431755) hafifçe farklı — kod sabitini kullanın.
6. Recipe hacim alanları (F_R_BOIL_VOL, F_R_TOP_UP, F_R_TRUB_LOSS, F_R_COOL_PCT, F_R_TUN_DEADSPACE, F_R_TUN_ADDITION, F_R_TOP_UP_KETTLE, F_R_TUN_VOL/MASS/HC, F_R_FERMENTER_LOSS, F_R_VOLUME) ekipman alanlarına **alt alan (sub-field) bağlantısıdır** (`InitRecipeSchema`); F_R_BOIL_OFF ← F_E_TOT_BOIL_OFF, F_R_MASH_TOT_WATER ← F_MH_TOT_WATER, F_R_MASH_GRAIN_WEIGHT ← F_MH_GRAIN_WEIGHT.
7. "Temperature" (2) tipi adımın infüzyonu varsa mash suyuna eklenir ama ısı hesabı yapılmaz (doğrudan ısıtma varsayımı).
