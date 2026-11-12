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
- **Karar (T1.12 B9, B10):** Sabit nokta yinelemesi en çok **10 000 adım** sürer (normal girdide 100'ün çok altında yakınsar; çok büyük miktarda, ör. 1e5 oz, sınıra dayanır ve sonuç yakınsamamış kalır). Hata bayrağı API şekline bağlı olduğu için T1.16'ya bırakıldı. `Garetz_hop_weight` 200 adımda hedefe girmezse (hedef ulaşılamaz: tablo değeri 0 olan t ≤ 10 dk, A = 0, ya da aralığı aşan hedef) **0** döndürür; Tinseth ve Rager terslerindeki "ulaşılamaz → 0" ile aynıdır. Referans ve Rust aynı kuralı uygular.
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
- **Karar (T1.12 B1):** Soğuma doğrusu alt sınırsızdır (0 °F'ı 157. dk'da, 0 K'i `e = 670.31/1.344 = 498.74` dk'da geçer; sonrasında üs pozitife döner ve taşar). Kural: `τ ≥ 5` iken `(T_F + 459.67)·5/9 ≤ 0` olunca `f` limit değerini (0) alır ve toplam biter (döngüden çıkılır); ayrıca adım sayısı en çok 20 000 ile sınırlıdır (`D` sonsuz ya da 1e7 olsa da döngü ~4990 adımda biter, sonuç sonludur). `e < 498.74` için sonuç eski formülle birebir aynıdır (referans ve Rust aynı kuralı uygular). Vektörlerde 60..1e7 dk süreleri var.
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
   - Kenar durumu: BeerSmith'te form 4 dry hop ilk döngüde diziye alınıyor, ikinci döngüde atlanıyor ve indeksler kayıyor (yazılım hatası). **Karar (T1.12 B2):** bu kayma taklit edilmez; form 4 dry hop her iki döngüde de normal bir acılık hopu gibi işlenir (`I_i` hesaplanır, ölçeklenir, `F_adj = 1`, `U = 1`). Ölçeklenen tarifin IBU'su hedefe eşit çıkar (vektör: 1 oz %10 pellet 60 dk + 0.05 fl oz %50 form 4 dry hop). Dry hop olan diğer formlar (use = 1, form ≠ 4) iki döngüde de atlanır.
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
- `ScaleBitterness`'taki form 4 dry hop indeks kayması: taklit edilmiyor, §6.1'de karar var (T1.12 B2).
