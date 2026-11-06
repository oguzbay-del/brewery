# 04 – Maya, Mead/Şarap ve Karbonasyon: BeerSmith 4 ve Brewtarget Karşılaştırması

**Kaynaklar**
- BS = BeerSmith 4 spesifikasyonu: `/home/user/formuller/04-maya-mead-karbonasyon.md` (bölüm numaraları §x.y ile verildi) ve `/home/user/formuller/testler/04_maya_karb_tv.py`.
- BT = Brewtarget kaynak kodu: `/home/user/src/brewtarget` (dosya:satır ile verildi).
- Sayısal karşılaştırma betiği: `/home/user/karsilastirma/testler/04_maya_karb_bt_vs_bs.py`. Çalıştırma: `python3 -I 04_maya_karb_bt_vs_bs.py [brewtarget_kökü]`. BT'nin USDA Brix tablosunu doğrudan `SucroseConversion.cpp` dosyasından okur. BS test vektörlerini bire bir üretir (70.71 %, 221.12 milyar, 3.424 L, 1.524 L, 117.34 g, 11.86 psi).

**Kısa sonuç:** Brewtarget'ın bu alandaki hesap motoru çok küçük. Yalnız iki araç var: `PitchDialog` (183 satır) ve `PrimingDialog` (100 satır). Mead/şarap aracı, fıçı basıncı hesabı ve tatlandırma aracı yok. Maya ve reçete modelinde canlılık hesabı da yok. Hücre hedefi formülü ve starter büyüme eğrisi iki programda pratikte aynı. Asıl farklar şunlar: canlılık modeli (BT doğrusal %0.7/gün, BS üstel %10/ay), stir plate çarpanı, kuru maya hücre yoğunluğu ve priming şekerinin stokiyometrisi.

---

## 1. Hesap hesap karşılaştırma

### 1.1 Maya

| Hesap | BeerSmith | Brewtarget | Fark | Hangisi daha doğru / güncel |
|---|---|---|---|---|
| **Canlılık modeli** | Üstel: `V = V0·((100−r)/100)^(gün/30)`, taban %5. V0: kuru 98, sıvı 97, slurry 96. r: kuru 1.66, sıvı 10 (White Labs 5), slurry 25 %/ay (§1.2, §1.4) | Doğrusal: `V = 97 − 0.7·gün`. QSpinBox tamsayıya keser, aralık 1–100 (`src/tools/PitchDialog.cpp:182`, `ui/pitchDialog.ui:209-214`). Forma göre ayrım yok, tek bir eğri var | 90 günde BS %70.7, BT %34. 180 günde BS %51.6, BT %1 (§2) | BT'nin %0.7/gün kuralı (≈%21/ay) eski Mr Malty / *Yeast* kitabı dönemi sıvı maya kuralıdır. Kuru mayaya uygulandığında açıkça yanlış olur. BS'nin üstel ve forma bağlı modeli fiziksel olarak daha makul: kayıp yavaşlar, sıfıra inmez. Oranların doğruluğu ise üreticiye ve paketlemeye göre değişiyor; literatür tek bir değer vermiyor (**emin değilim**). Modern sıvı paketler (ör. PurePitch) muhtemelen %10/ay'a daha yakın |
| Canlılık girişi | Malzemede yaş (`F_Y_PKG_DATE`), araçta tarih veya elle giriş (§1.3–1.4) | Yalnız araçta: üretim tarihi veya elle %. Tarih bugünden ileri olamaz (`PitchDialog.cpp:41`). İpucu metni "Best By date less three months" (`ui/pitchDialog.ui:180`). Yeast/RecipeAdditionYeast modelinde canlılık ya da paket tarihi yok. `StockPurchase` yalnız `dateBestBefore` tutuyor (`src/model/StockPurchase.h:130`) | BT reçeteye bağlı değil | BS daha eksiksiz |
| **Hücre hedefi** | `N = pitch·V(L)·P(OG)` milyar. Reçete tipine göre ön ayar: ale 0.75, lager 1.5, hybrid 1.0, mead/şarap 0.5 (§1.7–1.8) | Aynı formül: `cells = rate·1e6·V_L·1e3·P` (`PitchDialog.cpp:107`). Oran bir kaydırıcıyla 0.75–2.0 arasında seçilir (`:97`). Tip ön ayarı yok, yalnız ipucu metni var: "ale 0.75-1, lager 1.5-2" (`ui/pitchDialog.ui:115`). Hacim reçetenin `finalVolume_l`, OG de `og` değerinden alınır (`src/MainWindow.cpp:2497-2498`) | 20 L 1.060 için ikisinde de 221.12 milyar (fark %0) | Eşit. İkisi de Fix/White'ın 0.75/1.5 M/ml·°P kuralını kullanıyor. BT 0.75'in altına inemez, yani mead/şarap için 0.5 seçilemez. Plato polinomu ikisinde aynı (`src/Algorithms.cpp:52-54`, BS §0.2) |
| Paket sayısı (starter yok) | `N / ortalama_paket` (paket başına hücre ayrı bir alan, varsayılan 100 milyar). Yukarı yuvarlama `+0.99` ile yapılır (§1.12) | `cells / (viab·1e9)`. Paket başına 100 milyar sabit kodlu, alan yok (`PitchDialog.cpp:108`) | 6.50 / 3.13 paket. Fark yalnız canlılıktan geliyor | BS daha esnek; PurePitch, Imperial gibi farklı paket boyutları girilebiliyor |
| **Starter büyüme modeli** | Güç yasası: `I = 247.69·g^−2.1791`, `g = (247.69/I)^(1/2.1781)`. 1–6 aralığına kırpılır (§1.9) | `I = (12.522/g)^2.18` (`PitchDialog.cpp:132`). 12.522^2.18 = 247.0, yani **aynı White (*Yeast*, 2010) eğrisi** | Aynı I için büyüme katsayısı farkı %0.1–0.2 (§2). Aynı mevcut hücreyle starter hacmi stir yokken %0.3 farklı | Model aynı. BT'de büyüme kırpılmıyor ve `g < 1` durumu kontrol edilmiyor (bkz. §4). İkisi de Kai Troester'in (Braukaiser) "büyüme ∝ ekstrakt" stir plate modelini kullanmıyor |
| **Stir plate / havalandırma** | Stir: büyüme ×1.45 (`O_Y_STIR_PCT` 45 %). Hacim önerisinde `r/1.45` alınır, bu da hacmi 1.45^2.179 = **2.25×** küçültür (§1.9–1.10). `O_Y_STIR_PLATE_FACTOR = 2.66` seçeneği var ama kullanılmıyor (§1.2) | Hacim bölücüsü: yok 1.0, "O2 at start" **1.33**, stir plate **2.66** (`PitchDialog.cpp:115-125, 133`) | Stir plate ile BT %15 daha küçük starter öneriyor (1.291 L / 1.524 L, aynı hücreyle) | BS'nin kullanılmayan 2.66 seçeneği ile BT'nin 2.66 değeri aynı. Muhtemelen ikisinin de kaynağı eski Mr Malty hesaplayıcısı [tahmin]. Hangi değerin doğru olduğu belirsiz. Braukaiser deneyleri stir plate'te büyümenin inokülasyondan çok ekstraktla sınırlı olduğunu gösteriyor, dolayısıyla sabit çarpan kaba bir yaklaşım. BT'nin O2 seçeneği (1.33) BS'de yok |
| Mevcut hücre (starter girişi) | Malzemedeki canlı hücre (paket × hücre × canlılık) (§1.6) | `paket_sayısı × viab` (milyar). Paket sayısı 1–floor(gerekli paket) arasında (`PitchDialog.cpp:112, 128`) | – | BS daha esnek (slurry ml × hücre/ml gibi girişleri de kabul ediyor) |
| İki kademe starter | Var: her kademe √r (§1.11) | Yok | – | BS |
| Starter DME | `DME = (OG−1)·1000·V_gal/45` lb, 45 ppg, varsayılan OG 1.036 (§1.14) | Yok | – | BS |
| **Kuru maya** | Hacim modu: `g = oran(g/hL)·V/100`, varsayılan 75 g/hL, hücre olarak 10 milyar/g. Paket modu: `N/18` (§1.5, §1.13). Hidrasyon 25 ml/g, GoFerm 1.25 g/g | `dry_g = cells / 20e9` → **20 milyar/g** (`PitchDialog.cpp:109`). Hidrasyon ve GoFerm yok | 20 L 1.060: BS 15.0 g (g/hL) veya 12.28 g (paket modu), BT 11.06 g (−%26 / −%10) | Üreticiler hücre sayısı yerine g/hL veriyor (ör. ale 50–80, lager 80–120 g/hL; garanti edilen canlı hücre en az 5–6 milyar/g). Bu nedenle BS'nin g/hL yaklaşımı üretici talimatına daha yakın. BT'nin 20 milyar/g değeri White'ın kitabından geliyor ve iyimser. Laboratuvar sayımları 7–20 milyar/g arasında değişiyor (**emin değilim**) |
| Ortalama attenüasyon | `(min+max)/2` (§1.15) | Aynı; ikisi de yoksa 75 % (`src/model/Yeast.cpp:354-359`) | – | Eşit |
| Reçete düzeyi maya alanları | Pitch, starter, paket ve kuru gram reçetede hesaplanıyor (§1.6–1.13) | `RecipeAdditionYeast` içinde `timesCultured` ve `cellCountBillions` var (`src/model/RecipeAdditionYeast.h:35-36`), ama hiçbir hesapta kullanılmıyor (yalnız tablo ve serileştirme) | – | BS |

### 1.2 Mead / şarap

| Hesap | BeerSmith | Brewtarget |
|---|---|---|
| Mead maya miktarı (Plato'ya göre 0.25–0.5 g/L) | Var (§2.2) | **Yok** |
| GoFerm / hidrasyon | Var (§2.3) | Yok |
| TOSNA / TiOSNA besin programı, YAN | Var (§2.4–2.5) | Yok |
| Önerilen serbest SO2 `exp(2.21828·pH − 4.27809)`, K-metabisülfit | Var (§2.6) | Yok |
| Potasyum sorbat 1.25 g/gal | Var (§2.7) | Yok |
| Tatlandırma (backsweeten) aracı | Var (§2.8) | Yok |
| Mead/şarap reçete tipi | Var (F_R_TYPE 4/5) | Yalnız enum ve veri düzeyinde var: `Recipe::Type::Mead/Wine` (`src/model/Recipe.h:262-273`), `Fermentable::Type::Honey/Juice/Fruit` (`src/model/Fermentable.h:157-165`), `Yeast::Type::Wine/Champagne` (`src/model/Yeast.h:133-150`). Bunlara bağlı bir hesap yok. Varsayılan veride yalnız "Campden Tablet" ve "Yeast Nutrient" misc kayıtları bulunuyor (`data/DefaultContent001-OriginalDefaultData.xml`) |
| Brix → SG | Formül: `B/(258.6 − (B/258.2)·227.1) + 1` (§0.2, §2.8) | USDA 1970 sakaroz tablosu (0–80 Brix, 801 satır) üzerinde doğrusal interpolasyon (`src/measurement/SucroseConversion.cpp`, `src/Algorithms.cpp:331-372`). **80 Brix'in üstü 1.41471'e kırpılır**, yalnızca uyarı verir (`src/Algorithms.cpp:132-137`) |

Brix dönüşümü karşılaştırması (tatlandırma aracının girdisi):

| Brix | BS SG | BT SG (USDA) | Fark (puan) |
|---|---|---|---|
| 15 | 1.0610 | 1.0610 | ≈0 |
| 70 | 1.3550 | 1.3500 | +5 (BS yüksek) |
| 80 | 1.4250 | 1.4147 | +10 |
| 82.5 (bal) | 1.4432 | 1.4147 (kırpılmış) | +28 |

Bal örneği (19 L, 0.998 → 1.010): BS **0.526 L**, BT tablosu kırpılmış SG ile 0.563 L. USDA eğiminden (~0.0066/Bx) dışdeğerle bulunan SG ≈ 1.431 ile 0.542 L çıkıyor. Buna göre BS yaklaşık %3 az, kırpılmış BT %4 fazla. 0–80 Brix aralığında USDA tablosu daha doğru. Bal gibi aralık dışı değerler için BT kırpma yapıyor, ki bu bir hata.

### 1.3 Karbonasyon

| Hesap | BeerSmith | Brewtarget | Fark | Hangisi daha doğru / güncel |
|---|---|---|---|---|
| **Kalıntı CO2** | `3.0378 − 0.050062·T + 0.00026555·T²`, T °F, en az 32 (§3.2) | `1.57·0.97^T`, T °C (`src/tools/PrimingDialog.cpp:66`) | 20 °C'de −%0.9, 10 °C'de −%3.4, 4 °C'de −%6.3, 0 °C'de −%8.1 (BT düşük) | BS'nin kuadratiği Zahm & Nagel / ASBC tablosundaki 0 psig değerleriyle örtüşüyor: 32 °F ≈ 1.70, 68 °F ≈ 0.86 vol. Soğukta BS daha doğru |
| Sıcaklığın anlamı | Alan adı "Bottling or Keg Storage Temperature" (§3.2) | Alan adı "Beer Temperature", ipucu "Temp of the beer" (`ui/primingDialog.ui:60,82`) | – | Fiziksel olarak doğru değer, fermantasyondan sonra biranın ulaştığı en yüksek sıcaklıktır. İki programın etiketi de yanıltıcı |
| **Priming şekeri (mısır şekeri)** | `15.195 g/gal/vol` = 4.0145 g/L/vol (§3.3) | Stokiyometri: `(vol − kalıntı)·V/22.4` mol CO2, glukoz 2 CO2 verir. Monohidrat 198 g/mol → 4.420 g/L/vol (`PrimingDialog.cpp:67-80`) | 2.4 vol, 20 °C, 19 L: BS 117.34 g, BT 129.84 g (**+%10.7**). Kalıntı aynı alınsa bile +%10.1 | BS sabiti, glukozun **susuz** formuna ve %100 CO2 verimine karşılık geliyor (1.9642 × 180/88 = 4.018). BT'nin susuz glukoz sonucu 118.04 g, BS'den yalnız +%0.6 farklı. Gerçek mısır şekeri dekstroz monohidrattır, dolayısıyla teoride BT'nin monohidrat sonucu daha doğru. Pratikte maya şekerin tamamını CO2'ye çevirmez (Balling'e göre ~%95), bu da gerçek ihtiyacı biraz daha artırır. Ancak BS'nin formülü topluluk hesaplayıcılarında yaygın ve sahada kabul görüyor. %10 fark tadımda fark edilebilecek sınıra yakın (**kesin hüküm vermiyorum**) |
| Sofra şekeri | `mısır / 1.099` (§3.3) | `mol/4 · 342` (`:84-86`) | 106.77 g / 112.14 g (+%5.0) | BS'de bir iç tutarsızlık var. Taban değer susuz glukoza göre ayarlanmış, ama 1.099 ≈ 198/180 oranı mısır şekerini monohidrat sayıyor. Stokiyometriye göre sakaroz, susuz glukozun 0.95 katı olmalı; BS 0.91 kullanıyor. Bu açıdan BT tutarlı |
| DME | `mısır × 1.538` (= ~%65 fermente olabilir) (§3.3) | `glukoz/0.60` (= %60) (`:87-89`) | 180.47 g / 196.73 g (+%9.0) | İkisi de kaba bir varsayım. DME'nin fermente olabilirliği %55–65 civarında ve partiye göre değişiyor |
| Glukoz (susuz) | Yok | Var (`:81-83`) | – | BT |
| Fıçıya priming (yarım miktar) | `CarbKegFactor = 0.5` sabiti var ama hiçbir hesapta uygulanmıyor (§3.3) | `kegPrimingFactor` alanı yalnız saklanıyor (BeerXML uyumu için; `src/model/Recipe.h:387`, `Recipe.cpp:2482-2483`), hiçbir hesapta kullanılmıyor | – | İkisinde de fiilen yok |
| Priming etkinliği / şeker eşdeğeri | `F_C_CARB_RATE` ile bölme var (§3.4) | `primingSugarEquiv` alanı var (`Recipe.h:385`), hesapta kullanılmıyor | – | BS |
| Reçetede karbonasyon | `carb_vols` ve sıcaklık reçetede; araç reçete hacmini kullanıyor (§3.4) | `carbonation_vols`, `carbonationTemp_c` ve `forcedCarbonation` reçetede saklanıyor (`Recipe.h:366-383`). PrimingDialog reçeteden veri **almıyor** (`MainWindow.cpp:358`, yalnız oluşturulur) | – | BS |
| **Fıçı basıncı** | `P(psi) = −16.6999 − 0.0101059T + 0.00116512T² + 0.173354TV + 4.24267V − 0.0684226V²` (§3.5). 4 °C ve 2.5 vol için **11.86 psi** | **Yok.** Kodda basınç hesabı bulunmuyor | – | BS. Değer Zahm & Nagel tablosuyla uyumlu (39 °F, 2.5 vol ≈ 11.5–12 psig) |
| vol ↔ g/L | Gidiş ×1.9642, dönüş ÷1.96042; asimetrik (§3.7) | Tek sabit 1.96, simetrik (`src/measurement/Unit.cpp:759-766`) | %0.2 | BT tutarlı. Fiziksel değer 0 °C, 1 atm için 1.977 g/L. 1.96 bir bira literatürü değeri (BYO). Ancak BT birim etiketini **"mg/L"** olarak yazmış (`Unit.cpp:766`), oysa çarpan g/L'ye karşılık geliyor. Bu bir hata |

---

## 2. Sayısal cross-check (betik çıktısı)

`/home/user/karsilastirma/testler/04_maya_karb_bt_vs_bs.py` çıktısından seçilmiş satırlar:

| Test | BeerSmith | Brewtarget | Fark (BT−BS)/BS |
|---|---|---|---|
| Canlılık, sıvı paket, 90 gün (~3 ay) | 70.71 % | 34 % | −51.9 % |
| Canlılık, sıvı White Labs (5 %/ay), 90 gün | 83.17 % | 34 % | −59.1 % |
| Canlılık, sıvı, 30 gün | 87.30 % | 76 % | −12.9 % |
| Canlılık, sıvı, 180 gün | 51.55 % | 1 % | −98.1 % |
| Canlılık, kuru, 90 gün | 93.20 % | (ayrım yok: 34 %) | – |
| Gerekli hücre, 20 L 1.060 ale, 0.75 | 221.12 milyar | 221.12 milyar | 0.0 % |
| Gerekli paket (starter yok) | 3.13 | 6.50 | +108 % |
| Starter, stir yok (her biri kendi canlılığıyla) | 3.424 L | 8.151 L | +138 % |
| Starter, stir plate (kendi canlılığıyla) | 1.524 L | 3.064 L | +101 % |
| Starter, O2 at start (yalnız BT) | – | 6.129 L | – |
| Starter, stir yok, **aynı** A = 70.71 milyar | 3.424 L | 3.435 L | +0.3 % |
| Starter, stir plate, **aynı** A = 70.71 milyar | 1.524 L | 1.291 L | −15.2 % |
| Starter, taze paket (97 milyar), stir yok / stir | 2.359 / 1.050 L | 2.366 / 0.889 L | +0.3 / −15.3 % |
| Büyüme katsayısı, I = 10 / 25 / 100 M/ml | 4.365 / 2.866 / 1.517 | 4.355 / 2.860 / 1.514 | −0.1…−0.2 % |
| Kuru maya, g | 15.0 (75 g/hL) veya 12.28 (18 mlr/g) | 11.06 (20 mlr/g) | −26 % / −10 % |
| Kalıntı CO2, 20 °C | 0.8615 vol | 0.8538 vol | −0.9 % |
| Kalıntı CO2, 4 °C | 1.483 vol | 1.390 vol | −6.3 % |
| Priming, 2.4 vol, 20 °C, 19 L: mısır şekeri | 117.34 g | 129.84 g (monohidrat) | +10.7 % |
| aynı: susuz glukoz | – | 118.04 g | (BS "mısır" değerine göre +0.6 %) |
| aynı: sofra şekeri | 106.77 g | 112.14 g | +5.0 % |
| aynı: DME | 180.47 g | 196.73 g | +9.0 % |
| Fıçı basıncı, 4 °C, 2.5 vol | 11.86 psi (81.8 kPa) | yok | – |
| 2.4 vol → g/L | 4.714 | 4.704 | −0.2 % |
| Bal hacmi (tatlandırma, 82.5 Bx) | 0.526 L | 0.563 L (kırpılmış SG ile; BT'de araç yok) | +7 % |

Yorum: Starter hacmindeki büyük farkların **tamamı canlılık modelinden** geliyor. Aynı canlı hücre sayısı verildiğinde iki program stir yokken %0.3 içinde aynı sonucu veriyor. Stir plate ile kalan %15'lik fark, 2.66 ile 2.25 çarpanlarının farkıdır.

---

## 3. Özellik karşılaştırması

**Yalnız BeerSmith'te olanlar:** forma göre ayrı canlılık ve yaşlanma oranları (üreticiye özel oran dahil); reçete tipine göre pitch ön ayarları ve min–max aralıkları, mead/şarap için 0.5; paket başına hücre alanı ve hücre/ml (slurry); iki kademeli starter; starter DME miktarı; g/hL tabanlı kuru maya, 11.5 g ve 5 g paket sayısı, hidrasyon suyu, GoFerm; reçeteye entegre maya hesabı ve metin talimatları; mead maya ve GoFerm hesabı, TOSNA/TiOSNA, YAN; SO2 önerisi, K-metabisülfit, sorbat; tatlandırma aracı; fıçı basıncı; karbonasyon tipleri (şişe, fıçı, priming'li fıçı); priming etkinliği; reçete hacmiyle bağlantılı karbonasyon.

**Yalnız Brewtarget'ta olanlar:** "O2 at start" havalandırma seçeneği (×1.33); susuz glukoz şeker türü; stokiyometrik (açık formüllü) priming; 0–80 Brix aralığında USDA tablo interpolasyonlu Brix↔SG; geniş maya ve kültür tipi listesi (Kveik, Brett, Lacto, Pedio, MixedCulture vb., `src/model/Yeast.h:133-150`), ancak hesaplarda kullanılmıyor.

**İkisinde de olmayanlar:** Braukaiser (Kai) stir plate büyüme modeli; sıcaklık ve basınç için Henry yasasına dayalı tam model; gerçek (ölçülmüş) şeker fermente olabilirlik değeri; başlık boşluğu (headspace) etkisi.

---

## 4. Hatalar ve tuhaflıklar

### Brewtarget
1. **Canlılık tek ve doğrusal:** `97 − 0.7·gün`, tamsayıya kesilir, en düşük %1 (`PitchDialog.cpp:182`, `ui/pitchDialog.ui:209-214`). Kuru mayaya da aynı eğri uygulanır; 4 aylık kuru maya %13 görünür. Kalıntı CO2 hesabı da tamsayı yüzdeye dayandığı için kaba sonuç verir.
2. **Büyüme kırpılmıyor:** `growthRate = gerekli/mevcut` için üst sınır yok (BS 6 ile sınırlar, §1.9). Ayrıca `growth < 1` olduğunda (hücre zaten yeterliyse) yine pozitif bir starter hacmi hesaplanır (`PitchDialog.cpp:131-133`). Testte 1 paket ve %34 canlılıkla g = 6.5 çıktı; White eğrisinin geçerli aralığının dışına düşüldü ve 8.15 L önerildi.
3. Paket sayısı kutusu, "starter olmadan gereken paket" değerinin tabanıyla sınırlanıyor (`:112`). Gerekli paket 1.9 ise en fazla 1 paket girilebilir; .ui'deki en çok 5 sınırı da (`ui/pitchDialog.ui:284-288`) koddan eziliyor.
4. Kaydırıcı 0.75–2.0 aralığında; mead/şarap için 0.5 seçilemiyor (`:97`).
5. Kuru mayada 20 milyar/g kullanılıyor ve canlılık uygulanmıyor (`:109`).
6. Priming aracı reçeteye bağlı değil. `kegPrimingFactor`, `primingSugarEquiv`, `carbonationTemp_c` alanları saklanıyor ama hiçbir hesapta kullanılmıyor (`Recipe.cpp:2472-2483`). BS'deki "0.5 fıçı faktörü kullanılmıyor" tuhaflığının (§3.3) aynısı BT'de de var; muhtemelen ikisi de BeerXML alanının kalıntısı.
7. `carbonationGramsPerLiter` biriminin etiketi "mg/L", çarpanı ise g/L (`Unit.cpp:766`).
8. BrixToSG 80 Brix'te kırpılıyor (`Algorithms.cpp:132-137`). Bal (~82 Bx) ve sakaroz şurubu için yanlış sonuç verir.
9. Sıcaklık negatif olduğunda kalıntı CO2 artmaya devam eder (0.97^T), alt sınır yok. Hedef < kalıntı olduğunda negatif şeker çıkar; BS'de de kırpma yok (§4.8).
10. `cellCountBillions` alan tipi `OrdinalNumeral` (`RecipeAdditionYeast.cpp:58`), oysa bir sayım değeri için ordinal tip anlamsız. Bu alan hiçbir hesapta kullanılmıyor.

### BeerSmith tuhaflıklarının (spes. §4) Brewtarget karşısındaki durumu
| BS tuhaflığı | Brewtarget'ta karşılığı |
|---|---|
| Büyüme üslerinin farklı olması (−2.1791 / −2.1781) (§1.9) | BT tek üs kullanıyor (2.18); eğri aynı. Fark ihmal edilebilir (%0.2). Kendi uygulamanızda tek sabit kullanın |
| Kuru maya için 10 milyar/g ve 18 milyar/g'ın birlikte kullanılması (§1.5, §1.13) | BT tutarlı biçimde 20 milyar/g kullanıyor. Değerler arasındaki dağınıklık, literatürdeki belirsizliği yansıtıyor |
| "Avg Yeast Age" değerinin toplam olması (§1.3) | BT'de reçete düzeyinde yaş yok |
| İkinci kademe stir bayrağı (§1.9) | BT'de iki kademe yok |
| Paket hesabında stir'ın sınırı düşürmesi (§1.12) | BT'de böyle bir paket hesabı yok. BS'nin davranışı ters yönde görünüyor; hata olması muhtemel |
| Reçetedeki YAN değerinin 1/4 çıkması (§2.5) | BT'de mead hesabı yok |
| Fıçı faktörü 0.5'in uygulanmaması (§3.3) | BT'de de alan var ama uygulanmıyor. Aynı kalıntı |
| Negatif şekerin kırpılmaması (§4.8) | BT'de de kırpılmıyor |
| Asimetrik vol↔g/L sabitleri (§3.7) | BT simetrik (1.96). Asimetri BS'ye özgü bir hata |
| Hacimde Plato→SG, ağırlıkta Brix→SG kullanılması (§2.8) | BT tek bir tablo kullanıyor ama 80 Bx'te kırpıyor |
| Ek bulgu: BS'nin "mısır şekeri" sabiti aslında susuz glukoza karşılık geliyor, sofra şekeri oranı ise monohidrat varsayıyor | BT her şeker türünü ayrı ve tutarlı biçimde stokiyometrik hesaplıyor |

---

## 5. Kendi uygulama için öneri

1. **Hücre hedefi:** `N = oran·V·°P` (iki programda ortak). Reçete veya maya tipine göre ön ayar sunun: ale 0.75, lager 1.5, hybrid 1.0, mead/şarap 0.5 (BS). Oran 0.25–2.0 aralığında kullanıcıya açık olsun.
2. **Canlılık:** BS'nin üstel modelini kullanın: `V0·(1−r)^(gün/30)`, form ve üreticiye göre düzenlenebilir r (kuru ~1.7, sıvı 10, slurry 25 %/ay) ve %5 taban. BT'nin doğrusal %0.7/gün kuralını almayın. Ölçülmüş canlılık (metilen mavisi) girişi de mümkün olsun.
3. **Starter:** White güç yasasını tek sabitle kullanın: `I = 247.7·g^−2.18`. Büyümeyi 1–6 aralığında kırpın ve gerekli ≤ mevcut olduğunda 0 döndürün. Stir plate için iki model sunun: (a) basit çarpan (BS 1.45 büyüme; havalandırma için BT'deki O2 seçeneği eklenebilir), (b) Braukaiser modeli (stir plate'te büyüme ≈ 1.4 milyar hücre / g ekstrakt; bu sabiti literatürden doğrulayın). İki kademe ve DME hesabını BS'den alın.
4. **Kuru maya:** Önce g/hL ile hesaplayın (üretici talimatı, varsayılan ale 75, lager 120 g/hL gibi; düzenlenebilir). Hücre/g değeri tek ve ayarlanabilir bir sabit olsun (varsayılan 10–15 milyar/g); BS'deki 10/18 karmaşasını tekrarlamayın.
5. **Priming:** Stokiyometrik yaklaşımı (BT) ve açık şeker listesini kullanın: dekstroz monohidrat, susuz glukoz, sakaroz, DME, bal ve kullanıcı tanımlı şekerler. Her şeker için "CO2 verimi / fermente olabilirlik" alanı olsun. Kalıntı CO2 için BS'nin kuadratiğini (Zahm/ASBC ile uyumlu) kullanın; sıcaklık alanını "fermantasyon sonrası en yüksek bira sıcaklığı" olarak etiketleyin. BS ile uyumluluk gerekiyorsa "BeerSmith uyumlu (15.195)" seçeneği eklenebilir; iki yaklaşım arasındaki fark ~%10.
6. **Fıçı basıncı:** BS polinomunu kullanın (BT'de karşılığı yok). Sonucu gösterge basıncı olarak verin ve rakım düzeltmesi seçeneği ekleyin.
7. **vol↔g/L:** Tek ve simetrik bir sabit kullanın (1.96 veya 1.977).
8. **Mead/şarap:** BT'de karşılığı olmadığı için BS modellerini alın: TOSNA/TiOSNA (YAN toplam Fermaid üzerinden, BS'deki 1/4 hatası düzeltilmiş olarak), SO2 eğrisi, sorbat. Tatlandırma için Brix→SG'de USDA tablosunu (0–80 Bx) kullanın; 80 Bx'in üstünde tablonun son eğimiyle dışdeğerleyin ya da BS formülüne geçin, kırpmayın.
