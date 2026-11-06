# 02 — Hop ve IBU: BeerSmith 4 ve Brewtarget karşılaştırması

**Kaynaklar**
- BeerSmith: `/home/user/formuller/02-hop-ibu.md` (aşağıda "BS §n" diye geçiyor), referans betik `/home/user/formuller/testler/02_hop_ibu.py`
- Brewtarget: `/home/user/src/brewtarget/src/...` (aşağıda "BT dosya:satır" diye geçiyor). Ana dosyalar `measurement/IbuMethods.cpp`, `model/Recipe.cpp` (`recalcIBU` 1249–1290, `boilAdditionTiming` 2917–3038, `ibuFromHopAddition` 3040–3177)
- Sayısal karşılaştırma betiği: `/home/user/karsilastirma/testler/02_hop_ibu_karsilastirma.py`. Çalıştırma: `python3 -I 02_hop_ibu_karsilastirma.py`. Bağımlılığı yok, çalıştığı doğrulandı.

---

## 1. Hesap hesap karşılaştırma

| Hesap | BeerSmith | Brewtarget | Fark | Hangisi daha doğru / güncel |
|---|---|---|---|---|
| **Tinseth kullanım oranı** | `1.65·0.000125^(SG−1) · (1−e^(−0.04t))/4.15` (BS §2.1) | Aynı formül (IbuMethods.cpp:87–89) | Yok | Eşit. İkisi de Tinseth 1997'yi birebir kullanıyor. |
| **mg/L dönüşüm sabiti** | `K = 100/1.34 = 74.627` (oz·%/gal → IBU) (BS §0) | `AA·g·1000/L`, SI birimlerinde tam değer; US karşılığı 74.89 (IbuMethods.cpp:142) | BT her yerde **+%0.35** | BT daha doğru (1 oz/gal = 7489 mg/L). BS'nin 1.34 böleni tarihsel bir yuvarlama. |
| **Rager** | 5 dakikalık **basamak tablosu**. `n=int(t+0.5)`, 60 dk'da %30 (BS §2.2) | Sürekli **tanh** uyumu: `U = (18.11 + 13.86·tanh((t−31.32)/18.17))/100` (IbuMethods.cpp:153) | 60 dk'da 0.300 / 0.3084 (+%2.8). 30 dk'da 0.153 / 0.171 (+%12). Basamak sınırlarında sıçrama yok | Rager'in 1990 makalesi bir tablo veriyor. Tanh yaygın kullanılan bir eğri uyumu. BT'nin sürekli olması UI açısından daha iyi; ama "orijinal" olan BS'nin tablosu. Bilimsel olarak ikisi de eski. |
| **Rager gravity düzeltmesi** | `GA=(SG−1.050)/0.2`, SG>1.050 ise (BS §2.2) | Aynısı (IbuMethods.cpp:155) | Yok | Eşit. |
| **Garetz** | Var; iteratif, HF/CF/GF ile, TF=1.02 sabit (BS §2.3) | **Yok**. `// TODO: Add Garetz and Daniel methods` (IbuMethods.h:66) | — | Garetz güncel değil. Ölçülen IBU'yu sistematik olarak düşük tahmin ettiği bilinir. |
| **Noonan** | Yok | 7. derece polinom × hacim(5 gal) × SG basamak faktörü (IbuMethods.cpp:163–189) | — | Eski ve çok az kullanılan bir model. Yeni bir uygulamada gerekmez. |
| **mIBU (Hosom 2015)** | Ayrı bir seçenek olarak yok. Benzer işi **PostBoilUtil** ("Estimate Boil Hop Util in Whirlpool", `F_E_WHIRLPOOL_CARRY`) yapıyor: Tinseth türevi × Arrhenius, **doğrusal** soğuma `T_F=210.64−1.344·e`, adım 0.1 dk (BS §3.5) | Formül seçeneği `mIBU` (IbuMethods.cpp:197–252). Tinseth türevi × Arrhenius, **üstel** soğuma `T_K = 53.70·e^(−b·t)+319.55`, `b = 0.0002925·A_eff/V + 0.00538` (kazan iç çapı ve ağız çapından hesaplanır), adım 0.001 dk | Çekirdek aynı (2.39e11·e^(−9773/T) ve **τ<5 ⇒ f=1 istisnası ikisinde de var**, IbuMethods.cpp:220). Soğuma eğrisi farklı: 15 dk'da BT 80.7 °C, BS 88.0 °C. Birleştirme farklı: BS `1−Π(1−s)`, BT `U_kaynama + U_soğuma` (toplama). | BT, Hosom'un sonraki (kazan geometrisine bağlı) soğuma modelini kullanıyor; bu daha güncel. BS'nin doğrusal eğrisi büyük olasılıkla Hosom'un ilk yazısındaki tek bir kazana ait uyum **[tahmin]**. |
| **Whirlpool / hop stand hopu** | `U_T(SG, t_steep) · Ftemp(T_wp)`. Sıcaklığı kullanıcı girer ve sabit kabul edilir; seçili denklemden bağımsız olarak hep Tinseth kullanılır (BS §3.3–3.4, §5.2-3) | Kaynatma sonrası adımdaki ekleme: Tinseth/Rager/Noonan seçiliyse **0 IBU** (`timeInBoil=0`; RecipeCalcTests.cpp:324–336 bunu doğruluyor). mIBU seçiliyse alev kesiminden itibaren soğuma eğrisiyle hesaplanır; `postBoilDelay` = alev kesiminden eklemeye kadar geçen süre, `coolTime` = kalan sıcak süre (Recipe.cpp:3132–3139). **BoilStep sıcaklığı kullanılmıyor**; sıcaklık sadece süreden ve kazan geometrisinden türetiliyor | Büyük fark (bkz. §2) | Fiziksel olarak BT mIBU'nun yaklaşımı (zamanla soğuma) daha gerçekçi. Ama kullanıcı "80 °C'de tut" diyemiyor. BS'nin sabit sıcaklık modeli, sıcaklığı kontrol edilen hop stand için doğru, pasif soğumada ise yanlış. |
| **Kaynatma hoplarının soğumadaki katkısı** | Opsiyonel (`F_E_WHIRLPOOL_CARRY`), süre ekipmandan `F_E_WHIRLPOOL_TIME` (BS §3.5, §5.2-c) | Sadece mIBU'da, **her zaman açık**. Süre: soğutma adımının uzunluğu; adım yoksa varsayılan **30 dk** (Boil.h:124, Boil.cpp:134–142, Recipe.cpp:3092–3094) | mIBU seçili ve soğutma adımı tanımlı değilse, 60 dk'lık hop sessizce +%3.2 IBU alır | Varsayılan 30 dk sıcak bekleme tartışmalı; kullanıcı fark etmeyebilir. |
| **Rakım / kaynama noktası** | `T_b = 100 − ft·0.3048·0.00324` °C; bütün kaynatma, FWH ve mash hoplarına `Ftemp(T_b)` uygulanıyor (BS §3.2). 5000 ft'te ×0.71 | `Equipment::boilingPoint_c` var (Equipment.cpp:318, varsayılan 100) ama **IBU'da kullanılmıyor**. Yalnızca MashWizard.cpp:151 ve MashDesigner.cpp:203 okuyor. Rakım alanı yok | 5000 ft'te BT +%41 | Yönü BS'de doğru: düşük kaynama sıcaklığı izomerizasyonu yavaşlatır. Ancak 5000 ft'te tam Arrhenius çarpanı (×0.71), pratikte rapor edilen etkiden büyük görünüyor **[emin değilim]**. |
| **IBU hacmi** | `V + trub` (soğumuş kaynatma sonu hacmi) (BS §5.3) | `m_batchSize_l` (fermentöre giden hacim; Recipe.cpp:3106). Parametre açıklamasında Tinseth'in "post-boil volume is correct" sözü alıntılanıyor (IbuMethods.h), ama kodda batch size kullanılıyor | Trub/chiller kaybı varsa BT yüksek çıkar: 0.5 gal kayıpta +%10.4 | **BS daha doğru.** IBU bir konsantrasyon (mg/L); kazanda kalan kayıp konsantrasyonu değiştirmez. |
| **IBU için SG** | Kaynatma öncesi SG. Geç ekstrakt varsa **zaman segmentleri** halinde parçalı SG (BS §5.1) | `m_boilGrav`: kaynatma öncesi SG, geç eklemeler hariç, tek değer (Recipe.cpp:1209–1243, 3109). Yorumlar birbiriyle çelişiyor: 3071–3075 "OG kullan", 3107–3108 "ortalama gravity" diyor, kod ise pre-boil kullanıyor | Geç ekstraktta BT, hopları tüm kaynatma boyunca düşük SG'de sayar: örnekte +%11.5 | Tinseth "kaynatma ortalaması" öneriyor. İkisi de pre-boil kullanıyor, bu da hafif yüksek IBU verir. Geç ekstraktı modelleyen BS daha doğru. |
| **Form düzeltmesi** | Pellet ×1.10, plug ×1.02, leaf ×1.00; seçenekle değiştirilebilir veya kapatılabilir (`O_HOP_TYPE_ADJ`). CO2 ekstraktta düzeltme yok. İzomerize ekstraktta kullanım oranı 1.0 (BS §4) | Pellet ×1.10, plug ×1.02, diğerleri ×1.0; **sabit kodlanmış** (Recipe.cpp:3156–3168). Extract/WetLeaf/Powder formları için özel durum yok | Pellet/plug için aynı | Eşit. ×1.10 pellet katsayısı literatürde tartışmalı (Hosom: kaynatmada etki daha çok non-IAA kaynaklı). |
| **Ekipman kullanım oranı** | `F_E_HOP_UTIL` % (BS §4) | `Equipment::hopUtilization_pct`, varsayılan 100 (Equipment.h:252; Recipe.cpp:3077–3080, 3171) | Aynı mantık | Eşit. |
| **First wort hop** | ×1.10 (`O_HOP_FIRST_WORT_ADJ`=+10). Extract tarifte uygulanmıyor. Süre = hopun süresi; kaynatmadan uzunsa ek segment (BS §4, §5.2) | ×1.1 (`firstWortHopAdjustment`, OptionDialog.cpp:520). Süre = tüm kaynatma süresi. FWH, "kaynatmanın 1. adımı (mash sonundan kaynamaya rampa) başında eklenen hop" olarak çıkarsanıyor (RecipeAdditionHop.cpp:262+) | Sayısal olarak aynı | Eşit. FWH bonusunun gerçek olup olmadığı tartışmalı. |
| **Mash hop** | ×0.20 (`O_HOP_MASH_ADJ` = −80) (BS §4) | `mashHopAdjustment` varsayılanı **0**, yani IBU vermiyor; 0'dan büyük yapılırsa kaynatma süresiyle hesaplanıyor (Recipe.cpp:3047–3050, 3141–3142) | BS 3.79 IBU, BT 0 | Ölçümler mash hopundan çok az IBU geldiğini gösteriyor; ikisi de savunulabilir. |
| **Dry hop** | 0 IBU (izomerize ekstrakt hariç) (BS §4) | Fermentation/Packaging aşaması: 0 IBU (Recipe.cpp:3143–3144) | Aynı | Eşit. İkisinde de dry hopun ölçülen IBU'ya katkısı (non-IAA, polifenol) yok. |
| **İzomerize ekstrakt** | Kullanım oranı 1, `W(fl oz)·A·74.63/gal` (BS §5.3) | Özel durum yok. Hacimle girilmişse hacim ağırlık sayılıyor, `qCritical "PROBABLY WRONG"` uyarısı veriliyor (Recipe.cpp:3066–3070) | BT −%76 (örnekte) | BS daha doğru. |
| **Hopped extract (fermentable IBU·gal/lb)** | `lb·IBUgal/lb / ((V+T)/128)` (BS §5.4) | `ibuGalPerLb · (kg/L) / 8.34538` (Recipe.cpp:1271) | **BT hatası**: kg/L → lb/gal için 8.345 ile *çarpmak* gerekirken bölüyor, sonuç 69.6 kat küçük | BS doğru. |
| **Hop yaşlanması** | `HopAlpha`: HSI, sıcaklık (27 °F'ta yarılanma) ve depolama faktörü (BS §10) | `Hop::sixMonthAlphaLoss_pct` alanı var ama "we do not use this in any calculations" (Hop.h:276–292) | BT'de yok | BS (Garetz modeli). |
| **Hoplar arası etkileşim** | Yok; her hop bağımsız toplanıyor (BS §5.4) | Yok; `recalcIBU` hop başına topluyor (Recipe.cpp:1259–1264) | Aynı | İkisinde de doygunluk (çözünürlük sınırı) modeli yok. SMPH'de var. |

---

## 2. Sayısal karşılaştırma

Ortak girdiler: 1 oz (28.35 g), %5 AA, SG 1.050, 5 gal (18.93 L), yaprak hop, ekipman kullanım oranı %100, deniz seviyesi. BT mIBU'da kazan çapı belirtilmemişse fallback değer kullanılıyor: iç çap ve ağız çapı 45 cm (IbuMethods.cpp:245–246), b = 0.02996/dk.

| Senaryo | BeerSmith IBU | Brewtarget IBU | Fark | Not |
|---|---|---|---|---|
| 1 Tinseth 60 dk | 17.2137 | 17.2748 | +0.35% | Sadece K sabiti farkı |
| 2 Rager 60 dk | 22.3881 | 23.0954 | +3.16% | Tablo / tanh |
| 3 Rager 60 dk SG 1.070 | 20.3528 | 20.9958 | +3.16% | |
| 3b Rager 20 dk | 7.5373 | 7.8205 | +3.76% | |
| 4 Garetz 60 dk | 14.6009 | — | | BT'de yok |
| 5 Noonan 60 dk | — | 21.2500 | | BS'de yok |
| 6 mIBU, soğuma 0 dk | 17.2137 | 17.2748 | +0.35% | mIBU, Tinseth'e indirgeniyor |
| 7 60 dk + 20 dk sıcak bekleme (BS carry / BT mIBU) | 17.6732 | 17.7409 | +0.38% | Kaynatma hopunda modeller neredeyse aynı |
| 7b 60 dk, BT mIBU varsayılan soğuma 30 dk | 17.2137 (carry kapalı) | 17.7725 | +3.25% | BT'nin gizli varsayılanı |
| **8 0 dk (alev kesimi) + 20 dk soğuma** | **7.1159** | **5.8595** | −17.7% | BT daha hızlı soğuyor (üstel eğri) |
| 8b 0 dk, sadece Tinseth | 0 | 0 | | |
| 8c (8) τ<5 istisnası kaldırılmış BT mIBU | 7.1159 | 5.1376 | −27.8% | İstisnanın etkisi ~0.7 IBU |
| **9 Whirlpool 80 °C 20 dk, BT Tinseth/Rager/Noonan** | **2.3872** | **0.0000** | −100% | BT kaynatma sonrası eklemeye IBU vermiyor |
| 9b Aynısı, BT mIBU; ekleme, wort 80 °C'ye inince (alev kesiminden 15.7 dk sonra) | 2.3872 | 4.1959 | +75.8% | τ<5 istisnası ilk 5 dk'yı 100 °C gibi sayıyor |
| 9c BT mIBU, hop alev kesiminde eklenip 20 dk bekletiliyor | 2.3872 | 5.8595 | +145% | BS'de sıcaklık sabit 80 °C |
| 9d (9b) τ<5 istisnası kaldırılmış | 2.3872 | 1.4124 | −40.8% | Gecikmeli whirlpool için fiziksel olarak tutarlı değer |
| 10 Rakım 5000 ft, Tinseth 60 | 12.2274 | 17.2748 | +41.3% | BT rakımı kullanmıyor |
| 11 Pellet, Tinseth 60 | 18.9351 | 19.0023 | +0.35% | |
| 12 FWH pellet 60 | 20.8286 | 20.9025 | +0.35% | |
| 13 Mash hop pellet 60 (varsayılan ayarlar) | 3.7870 | 0.0000 | −100% | BT varsayılanı 0 |
| 14 5 gal parti + 0.5 gal trub | 15.6489 | 17.2748 | +10.4% | IBU hacmi farkı |
| 15 Geç ekstrakt (1.040, 15 dk'da 1.050'ye çıkıyor) | 16.9433 | 18.8992 | +11.5% | BT segment kullanmıyor |
| 16 İzomerize ekstrakt 0.1 fl oz, %50 | 74.6269 | 18.0206 | −75.9% | BT hacmi ağırlık gibi işliyor |
| 17 Hopped extract 1 lb, 10 IBU·gal/lb | 2.0000 | 0.0287 | −98.6% | BT birim hatası |
| 17b (17) BT, çarpma ile düzeltilmiş | 2.0000 | 2.0000 | 0% | Hatanın kaynağı doğrulandı |

Kullanım oranının süreye göre değişimi (SG 1.050):

| dk | Tinseth (ikisi aynı) | BS Rager (tablo) | BT Rager (tanh) | BS Garetz | BT Noonan (U eşdeğeri) |
|---|---|---|---|---|---|
| 5 | 0.0460 | 0.0500 | 0.0570 | 0.0000 | 0.0467 |
| 10 | 0.0836 | 0.0600 | 0.0667 | 0.0000 | 0.0665 |
| 15 | 0.1145 | 0.0800 | 0.0819 | 0.0195 | 0.0835 |
| 20 | 0.1397 | 0.1010 | 0.1044 | 0.0483 | 0.1001 |
| 30 | 0.1773 | 0.1530 | 0.1710 | 0.1047 | 0.1602 |
| 45 | 0.2117 | 0.2690 | 0.2694 | 0.1683 | 0.2671 |
| 60 | 0.2307 | 0.3000 | 0.3084 | 0.1949 | 0.2837 |
| 90 | 0.2467 | 0.3000 | 0.3193 | 0.2125 | 0.3004 |

Soğuma eğrileri (alev kesiminden sonra):

| t (dk) | 0 | 5 | 10 | 15 | 20 | 30 |
|---|---|---|---|---|---|---|
| BT (45 cm kazan, 18.93 L) °C | 100.1 | 92.6 | 86.2 | 80.7 | 75.9 | 68.3 |
| BS (doğrusal) °C | 99.2 | 95.5 | 91.8 | 88.0 | 84.3 | 76.8 |

80 °C'ye inme süresi: BT'de 15.7 dk, BS'de 25.8 dk.

---

## 3. Özellik karşılaştırması

| Özellik | BeerSmith | Brewtarget |
|---|---|---|
| Tinseth / Rager | Var / Var (tablo) | Var / Var (tanh) |
| Garetz | Var | Yok (TODO) |
| Noonan | Yok | Var |
| mIBU (formül seçeneği olarak) | Yok; carry seçeneği eşdeğer işi görüyor | Var (kazan geometrisi parametreleriyle) |
| SMPH | Yok | Yok (kodda iskeleti yorum satırı olarak duruyor, IbuMethods.cpp:254–264) |
| Üç denklemi aynı anda gösterme | Var (`RecipeIBUTinseth/Rager/Garetz`, BS §5) | Yok; tek global formül (IbuMethods.cpp:301, 322–331) |
| Sabit sıcaklıklı whirlpool (kullanıcı °C girer) | Var | Yok |
| Zamanla soğuyan whirlpool / hop stand | Kaynatma hopları için (carry); whirlpool hopları için yok | mIBU'da, kaynatma adımları üzerinden (BoilStep) |
| Rakım / kaynama noktası etkisi | Var | Alan var, IBU'da kullanılmıyor |
| Geç ekstrakt SG segmentleri | Var | Yok (sadece pre-boil SG'den çıkarılıyor) |
| Form ve FWH/mash düzeltmelerinin ayarlanabilmesi | Hepsi seçenek (`O_HOP_*`) | FWH ve mash ayarlanabiliyor, form sabit kodlanmış |
| İzomerize ekstrakt | Var | Yok |
| Hopped extract (fermentable) | Var | Var ama hatalı (×1/69.6) |
| Hop yaşlanması (HSI) | Var (araç) | Alan var, kullanılmıyor |
| Hedef IBU'ya ölçekleme, hop başına IBU düzenleme | Var (BS §6) | Yok |
| Bağımsız "Hop Bitterness" aracı | Var (BS §7) | Yok |
| AAU/HBU, hop oranı g/L | Var (BS §7.1, §8) | Yok |
| IBU/GU | (bu bölümde yok) | Var (MainWindow.cpp:1650–1657) |

---

## 4. Hatalar ve tuhaflıklar

**BS spesifikasyonunda işaretlenen noktalar ve Brewtarget'ın bunlara karşılığı**

| BS tuhaflığı | Brewtarget'ta durum | Yorum |
|---|---|---|
| `PostBoilUtil`'de `τ<5 ⇒ f=1` istisnası (BS §3.5, §13) | **Birebir aynısı var:** `(time_minutes < 5.0) ? 1.0 : ...`, yorumu "The 1.0 case accounts for nonIAA components" (IbuMethods.cpp:218–220) | BeerSmith'e özgü bir hata değil; iki program da muhtemelen Hosom'un mIBU referans uygulamasından almış. Ama mutlak "kaynama yaşı" τ'ya bağlı olduğu için alev kesimi ve whirlpool hoplarında ilk 5 dakikayı 100 °C gibi sayıyor. BT'de whirlpool eklemelerinde her zaman `timeInBoil=0` olduğundan etki **daha büyük**: senaryo 9b'de 1.41 → 4.20 IBU. |
| 100 °C süreksizliği (`Ftemp(99.99)=1.0085`) (BS §3.1) | BT'de Ftemp clamp yok. mIBU'da T(0)=100.1 °C, f≈1.008; τ<5 istisnası nedeniyle pratikte görünmüyor | Arrhenius ifadesini 100 °C'ye normalize etmek gerekir: `k(T)/k(373.15)`. |
| `O_WHIRLPOOL_UTIL` kullanılmıyor (BS §4) | BT'de karşılığı yok | — |
| Garetz'te GF alt sınırı yok; tarif yolunda HF kayboluyor (BS §2.3) | BT'de Garetz yok | — |
| `ScaleBitterness` indeks kayması (BS §6.1) | BT'de böyle bir araç yok | — |
| Whirlpool hopu her zaman Tinseth, sabit sıcaklık (BS §3.3) | BT'de whirlpool IBU'su yalnızca mIBU'da var | Rager veya Noonan seçen BT kullanıcısı whirlpool IBU'sunu tamamen kaybediyor. |

**Brewtarget'a özgü hatalar ve tuhaflıklar**
1. **Hopped extract birim hatası** (Recipe.cpp:1271): `/ 8.34538` yerine `* 8.34538` olmalı. Sonuç 69.6 kat küçük çıkıyor (senaryo 17/17b).
2. **IBU hacmi batch size** (Recipe.cpp:3106). Kendi parametre açıklamasıyla (IbuMethods.h, Tinseth'in "post boil volume" alıntısı) çelişiyor. Kayıp hacmi kadar IBU'yu şişiriyor.
3. **Gravity yorumları birbiriyle ve kodla çelişiyor**: Recipe.cpp:3071–3075 "OG kullan", 3107–3108 "ortalama" diyor; kod pre-boil `m_boilGrav` kullanıyor.
4. **mIBU varsayılan 30 dk soğuma**: soğutma adımı yoksa `Boil::default_coolTime_mins = 30` sessizce uygulanıyor (Boil.h:124). 60 dk'lık hopa +%3.2 ekliyor.
5. **mIBU kazan çapı fallback'i 45/45 cm**: ekipmanda boşsa yalnızca qWarning veriliyor (IbuMethods.cpp:236–246). Ağız çapı = iç çap kabulü, tamamen açık bir kazan anlamına geliyor.
6. **`boilingPoint_c` IBU'ya girmiyor**. Kaynatma ve whirlpool adımlarının sıcaklıkları da mIBU'da kullanılmıyor; sıcaklık yalnızca süreden türetiliyor.
7. **Hacimle girilen hop** (ekstrakt) ağırlık gibi işleniyor. Kod bunu kendisi de "PROBABLY WRONG" diye logluyor (Recipe.cpp:3066–3070).
8. Mash hopları varsayılan ayarda 0 IBU veriyor. BS ile ciddi bir varsayılan farkı.
9. mIBU'da birleştirme toplama ile yapılıyor (`U_boil + U_post`, IbuMethods.cpp:249). BS'nin olasılıksal birleştirmesinden farkı ihmal edilebilir (<%0.1).

**BS'ye özgü olanlar (spesifikasyondan)**
- K = 74.63 (yaklaşık %0.35 düşük).
- Rager basamak tablosunda 5 dakikalık sıçramalar var.
- Rakım çarpanı tam Arrhenius ile uygulanıyor (5000 ft'te ×0.71). Bu muhtemelen etkiyi abartıyor **[emin değilim]**.

---

## 5. Kendi uygulaman için öneri

1. **Varsayılan model Tinseth olsun.** Formül iki programda da aynı. Sabit olarak BS'nin 1/1.34'ünü değil, tam SI dönüşümünü (`AA·g·1000/L`) kullan.
2. **Geç hop ve whirlpool için mIBU'yu temel al**, BT'nin kazan geometrisine bağlı üstel soğuma eğrisiyle: Hosom'un güncel modeli, BS'nin doğrusal eğrisinden daha genel. Şunları ekle:
   - Arrhenius'u `k(T)/k(T_kaynama)` olarak normalize et (100 °C süreksizliği olmasın).
   - `τ<5 ⇒ 1` istisnasını **mutlak süreye değil, alev kesiminden geçen süreye** bağla ya da tamamen kaldır. Mevcut hâli whirlpool IBU'sunu yaklaşık 3 kat şişiriyor (9b/9d).
   - Sıcaklığı kontrol edilen hop stand için BS gibi "sabit T" seçeneği sun: `U_T(t)·k(T)/k(100)`. BT bunu sunmuyor.
   - Kazan sonrası soğuma süresi açıkça kullanıcıdan alınsın; gizli 30 dk varsayılanı kullanma.
3. **Hacim:** kaynatma sonu, soğumuş hacim (= fermentör + trub/chiller kaybı), BS gibi. **SG:** kaynatma ortalaması (pre-boil ile post-boil'in ortalaması; Tinseth'in önerisi). Geç ekstrakt varsa BS gibi segmentli hesapla.
4. **Rakım:** kaynama noktasını ekipmandan al ve Arrhenius ile uygula, ama bunu açılıp kapanabilir bir seçenek yap (etkinin büyüklüğü belirsiz).
5. **Düzeltmeler** ayarlanabilir seçenek olsun (BS gibi): pellet 1.10, plug 1.02, FWH 1.10, mash 0.2 (veya 0). İzomerize ekstraktta kullanım oranı 1 olsun. Hopped extract: `lb·IBUgal/lb / gal`; BT'deki birim hatasını tekrarlama.
6. **İkincil modeller:** Rager (sürekli tanh hâli) karşılaştırma için yeter. Garetz ve Noonan gereksiz. İleride SMPH (non-IAA + doygunluk + dry hop katkısı) güncel hedef olabilir. "Üç modeli yan yana göster" (BS) kullanıcı için faydalı.
7. **Hop yaşlanması:** BS'nin HSI modelini (birinci dereceden bozunma, 27 °F'ta yarılanma, depolama faktörü) ekle. BT'de alan var ama hesapta kullanılmıyor.
