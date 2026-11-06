# 01 — Yoğunluk, Verim, Alkol, Attenuasyon, Kalori, Renk, Ölçüm Araçları: BeerSmith 4 ↔ Brewtarget

Kaynaklar:
- **BS** = BeerSmith 4 spesifikasyonu `/home/user/formuller/01-yogunluk-alkol-renk.md` (bölüm numaraları "BS §x").
- **BT** = Brewtarget kaynağı `/home/user/src/brewtarget` (yollar `src/` altından, `dosya:satır`).
- Sayısal karşılaştırma betiği: `/home/user/karsilastirma/testler/01_yogunluk_karsilastirma.py` (`python3 -I 01_yogunluk_karsilastirma.py`; Brewtarget'ın USDA sukroz tablosunu `src/measurement/SucroseConversion.cpp` dosyasından çalışma anında okur).

---

## 0. Özet: iki motorun felsefesi

| | BeerSmith | Brewtarget |
|---|---|---|
| Ekstrakt modeli | **PPG/puan** toplamı: `lb × yield×46/100 × verim / gal` (BS §2.1, §3.2) | **Sukroz kütlesi → °Plato → SG**: `kg × yield × (1−nem) × verim` ile sukroz eşdeğeri, sonra `P = şeker/(şeker+su)`, su = `V − şeker/1.587`; SG kübik polinomun kökünden (`Recipe.cpp:1040-1117`, `Algorithms.cpp:377-388`, `262-274`) |
| Dahili birimler | ABD (oz, fl oz, °F, SRM) | SI (kg, L, °C); renk °L olarak saklanır |
| FG modeli | Malzeme bazında: tahıl × (1−att), şeker **−0.231**, meyve/bal **−0.05**, mash sıcaklığı düzeltmesi, maya alkol toleransı (BS §2.2, §5) | Toplam puan × (1−att); yalnız "fermente olmayan oran" ayrılır. Mash sıcaklığı, şeker etkisi ve tolerans **yok** (`Recipe.cpp:1119-1150`) |
| ABV | **Balling** (ABW → ABV) (BS §6.2-6.3) | **HMRC tablo yöntemi** (`(OG−FG)×1000 × k`), tablo dışında "Hall/Daniels" yedek formülü (`Algorithms.cpp:435-531`) |
| Aşama mantığı | OG = fermantasyona kadar (phase ≤ 4), sanal OG = tertiary'ye kadar, FG = hepsi (BS §3.3-3.4, §5.1) | OG/FG **tüm** eklemeleri sayar; kaynatma gravitesi geç eklemeleri hariç tutar (`Recipe.cpp:1209-1243`) |

---

## 1. Hesap hesap karşılaştırma tablosu

| Hesap | BeerSmith (BS §) | Brewtarget (dosya:satır) | Fark | Hangisi daha doğru / güncel |
|---|---|---|---|---|
| SG → °P | `135.997·SG³ − 630.272·SG² + 1111.14·SG − 616.868` (§1.1) | Aynı kübik (`Algorithms.cpp:49-54`, `262-264`) | Yok | Eşit (ASBC kübik uyumu). |
| °P → SG | Kapalı form `1 + P/(258.6 − (P/258.6)·227.1)` (§1.2) | Aynı kübiğin **sayısal kökü** (secant, 0.900–1.150) (`Algorithms.cpp:231-248, 266-274`) | 30 °P'ye kadar ≤ 0.0001 SG | BT ileri/geri dönüşümde **tutarlı**: SG→P→SG aynı SG'ye döner. BS'de iki ayrı uyum eğrisi var, gidiş-dönüş küçük hata bırakır. Pratikte eşit. |
| Brix ↔ SG | Brix→SG: 258.6/**258.2**/227.1 formülü; SG→Brix: ayrı kübik (§1.3) | **USDA sukroz tablosu** (~800 nokta) + doğrusal interpolasyon (`Algorithms.cpp:276-375`, `SucroseConversion.cpp`) | ≤ 0.001 °Bx / 0.00001 SG | BT ölçülmüş veriye dayandığı için kavramsal olarak daha doğru. Fark ölçüm hassasiyetinin altında. |
| Brix → RI | `1.33302 + 0.001427193·B + 5.791157e-6·B²` (§1.4) | Aynı (`Algorithms.cpp:424-428`) | Yok | Eşit. |
| SRM → EBC | `×1.97` (§1.5) | `×25/12.7 = ×1.9685` (`measurement/Unit.cpp:704`) | −0.08 % | Eşit sayılır. 1.97 ASBC/EBC standardı. |
| °L ↔ SRM | `SRM = 1.3564·°L − 0.76` (§1.5) | `SRM = 1.3546·°L − 0.76` (`Unit.cpp:692-707`) | 30 °L'de −0.14 % | **BT doğru**: literatürde katsayı 1.3546. BS'de rakamların yeri değişmiş. |
| Yield → potansiyel | `1 + yield·0.046/100`, nem hesaba katılmaz (§1.6) | `kg × fineGrindYield × (1−nem)`. Yoksa coarse+diff, o da yoksa potansiyelden ters hesap (`Fermentable.cpp:399-421`, `RecipeAdditionFermentable.cpp:137-153`) | Nem girilirse BT %3-5 daha az ekstrakt verir | BT'de veri modeli daha zengin (DBFG, CGDB, nem). BS'deki "yield" zaten as-is kabul edilir. |
| OG puanı, mash tahılı | `lb·PPG·eff` (§2.1) | `kg·yield·eff` → Plato (`Recipe.cpp:1093-1099`) | Aynı reçetede +0.01 % (12.3 vektörü 1.0609/1.0610) | Eşit. |
| OG, **steep** tahılı | Ayrı `O_STEEP_EFFICIENCY` = **%15** (brewhouse verimi kullanılmaz) (§2.1) | Tip **Grain** ve aşama ≠ Mash ise yield × **0.60**, **ayrıca brewhouse verimi** de uygulanır (`RecipeAdditionFermentable.cpp:148-151`) | %72 verimde BT %43.2, BS %15 etkin verim: **+188 %** puan | İkisi de kaba. Literatür (Palmer, *How to Brew*) steep için yaklaşık %50–60 *uzantı/yield* oranı veriyor. Bu BT'ye yakın, BS'nin %15'i düşük kalıyor. Kristal/karamel maltlarda (zaten şekerleşmiş) BT daha gerçekçi. BS'nin artısı ayarın kullanıcıya açık olması. |
| OG, şeker/özüt/meyve/bal | Sabit verim, eff yok (tip 1,2,4,5,6,7) (§2.1) | `isExistingSugar()` = Sugar, Extract, Dry_Extract, Fruit, Juice, Honey: eff yok (`Fermentable.cpp:431-447`, `Recipe.cpp:2855-2895`) | Aynı küme | Eşit. |
| Hangi aşamalar OG'ye girer | phase ≤ 4 (Mash…Primary) + Sparge. Secondary/Tertiary yalnız sanal OG'de, Bottling hiçbirinde (§3.3-3.4) | **Tüm** fermentable eklemeleri OG'ye girer, aşama filtresi yok (`Recipe.cpp:2855-2895`) | Kuru şerbetçiotu gibi geç (secondary) şeker eklemelerinde BT OG'yi şişirir | **BS daha doğru**: "OG", mayalanma başındaki ölçülebilir gravitedir. Sanal OG kavramı ABV için doğru yaklaşım. |
| OG hacmi | `F_R_VOLUME` (fermentöre giden). Extract reçetede pre-trub hacmi (§3.3) | `batchSize + kettleTrubChillerLoss` (`Recipe.cpp:796-804, 922`). Mevcut şekerler ayrıca `(postBoil−trub)/postBoil` ile azaltılır (`Recipe.cpp:1069-1091`) | 0.5 gal trub ile OG 1.0609 → **1.0553** (−0.52 % SG, ~−9 puan) | **BS mantıklı**: brewhouse verimi tanımı gereği "fermentöre kadar tüm kayıpları" içerir, BT'nin kendi yorumu da öyle (`Recipe.h:307-311`). BT trub hacmini paydaya ekleyince kaybı **iki kez** sayıyor (mevcut şekerler için üç kez). BT'nin ilgili birim testi devre dışı (`unitTests/RecipeCalcTests.cpp:172-174` "Would be good to fix and reinstate"). |
| Kaynatma öncesi gravite | Mash+steep+boil+sparge, geç özütler hariç, `(V+trub)/Vboil` ile seyreltme (§3.5-3.6) | `eff·nişasta − geç nişasta + mevcut − geç mevcut` / `preBoilSize` (`Recipe.cpp:1209-1243`) | Formülasyon farkı | BT'de dikkat: geç eklenen **nişasta** için `eff·tümü − geç` yazılmış, yani geç tahıla eff uygulanmadan düşülüyor (küçük tutarsızlık). BS ayrıca IBU için zaman profilli SG dizisi üretiyor (geç özüt). BT bunu yapmıyor. |
| Mash sonrası gravite / mash verimi | `GetEstPostMashOG`, `GetEfficiency(mash_only)` (§3.5, §4) | Doğrudan karşılığı yok. BrewLog'da "kettle'a verim" = ölçülen puan·hacim / **beklenen** puan·hacim (`model/BrewLog.cpp:465-488`) | Kavram farkı | BS doğru (teorik maksimuma oran). BT'deki değer "plana göre %" oluyor, gerçek mash verimi değil. |
| Ölçülen brewhouse verimi | Yalnız tahıl potansiyeli paydada. Şeker puanları paydan düşülür (§4.1) | Şeker **dahil** tüm potansiyel paydada. Ölçülen ekstrakt kütlesi = `P/100·V·SG` (`BrewLog.cpp:502-534`) | Spes. 12.6 vektörü: 62.23 % → **66.34 %** | **BS'nin tanımı sektörde yaygın olan** (verim tahıl için anlamlı). BT şeker oranı yüksek reçetede verimi şişirir. Kütle tabanlı ölçü (BT) ise daha titiz. İdeali ikisinin birleşimi: kütle bazlı + şeker düşülmüş. |
| Attenuasyon kaynağı | **İlk** (secondary olmayan) maya, `(min+max)/2`. Maya yoksa 73 % (§5.1) | Maya eklemeleri arasında **en yüksek**. Ekleme bazlı değer, yoksa `(min+max)/2`, yoksa 75 % (`Recipe.cpp:1119-1135`, `Yeast.cpp:354-359`, `Yeast.h:125`). Maya yoksa att = **0** (FG = OG) | BS 73, BT 0 (maya yok) | BT'nin "en yüksek" seçimi çok mayalı (ör. Brett) reçetelerde mantıklı. Maya yokken FG=OG göstermesi de dürüst. |
| Mash sıcaklığı etkisi | `att += −1.25·(T−153.5 °F)`, T 144–160 °F'a kırpılır (§5.1 adım 2) | **Yok** | 152 °F'ta +1.875 puan att | Kaba ama yönü doğru (düşük mash = daha fermente olabilir wort; Fix & Fix, Briggs). BS'nin varlığı artı, ancak doğrusal ve kalibrasyonu belirsiz. |
| Şeker / meyve FG etkisi | Şeker puanı × **−0.231**, meyve/meyve suyu/bal × **−0.05** (§2.2) | Diğer ekstrakt gibi `(1−att)` (`Recipe.cpp:1137-1150`) | 12.3 vektörü FG: BS 1.0102, BT **1.0153**. ABV 6.72 → 5.95 % (−11 %) | **BS daha gerçekçi**: sukroz ~%100 fermente olur, oluşan etanol yoğunluğu düşürür. −0.231 değeri literatürle (Balling: 1 °P şeker → ~−0.2…−0.25 °P görünür ekstrakt) uyumlu. BT şekerli reçetelerde FG'yi sistematik olarak yüksek, ABV'yi düşük tahmin ediyor. |
| Fermente olmayan malzeme | `F_G_NOT_FERMENTABLE` bayrağı: tam puan FG'ye (§2.2) | `fermentableFraction_pct` (0–100, kısmi oran mümkün). Nişasta kaynaklıya da uygulanır (`Recipe.cpp:2869-2893`, `1093-1117`) | Laktoz testinde FG +0.5 %. ABV'yi BT etkilemiyor (fermentable-only OG/FG) | **BT daha esnek** (kısmi fermente olabilirlik, ör. %70 fermente olabilen dekstrin şurubu). |
| Alkol toleransı sınırı | ABV > tol ise FG bisection ile yükseltilir (§5.1 adım 4, §5.2) | Yok | OG 1.100 / att %90: BS FG 1.0259 (ABV 10.0), BT 1.0100 (ABV 12.15) | BS'nin özelliği yüksek OG'de gerçekçi. Ancak "tolerans" verisi mayalarda güvenilmez. Seçenek olarak sunulmalı. |
| ABV | Balling: `ABW = (OE−RE)/(2.0665−0.010665·OE)`, `ABV = ABW·FG/0.79661` (§6.2-6.3) | HMRC: `ABV = round1(ΔSG·1000) × k(Δ)`. Δ > 100.7 puanda yedek `76.08(OG−FG)/(1.775−OG)·FG/0.794` (`Algorithms.cpp:92-104, 435-531`) | Normal birada BT −1.4…−0.9 % (göreli). OG 1.120/FG 1.010'da BT **+8.4 %** (yedek formüle düşüyor) | **Balling (BS) daha geniş aralıkta doğru.** HMRC tablosu 0.8–13.6 % aralığında küçük biralar için vergi yöntemi (resmî ama ±0.1–0.2 hassasiyette). Tablo dışında BT'nin yedek formülü yüksek gravitede abartıyor (kendi kodundaki notta bile yaklaşık deniyor). |
| UK vergi ABV | Ayrı fonksiyon, aynı HMRC tablosu, Δ yuvarlanmaz, 0.135 tablo dışında da devam (§6.7) | Ana ABV bu (Δ 0.1'e yuvarlanır) | Tablo içinde birebir aynı. Δ > 100.7'de BS 0.135 ile devam, BT yedek formül | Tablo içi eşit. |
| Görünür attenuasyon | Plato bazlı `(OE−AE)/OE` (§6.4) | SG puanı bazlı `(OG−FG)/(OG−1)` (`BrewLog.cpp:585-598`) | BT +0.9…+1.8 puan yüksek | Plato bazlı (BS) tanım olarak daha doğru (ASBC "apparent degree of fermentation"). Puan bazlı yaygın homebrew yaklaşımı. |
| Gerçek attenuasyon / RE | `RE = 0.1808·OE + 0.8192·AE`, `RA = 1 − RE/OE` (§6.1, 6.5) | RE yalnız kalori içinde, RA yok (`Recipe.cpp:1320-1326`) | — | BS daha kapsamlı. |
| Kalori | 12 oz başına: `1881.22·FG(OG−FG)/(1.775−OG) + 3550·FG·(0.1808·OG+0.8192·FG−1.0004)` (SG ile), sanal OG kullanılır (§7) | kcal/L: `(6.9·ABW + 4·(RE−0.1))·FG·10`, Plato bazlı (hbd.org/ensmingr). 33 cl, 12 oz, pint seçenekleri (`Recipe.cpp:1297-1350`, `2672-2681`) | 1.050/1.010: −0.9 %. 1.120/1.010: −7.3 % | İkisi de aynı ASBC kökenli yaklaşım. BS formülü Plato yerine SG'yi doğrudan karıştırıyor (yaklaşık), yüksek gravitede sapma büyüyor. BT'nin Plato bazlı hali tanıma daha sadık. Ayrıca BS alkol/karbonhidrat ayrımı ve gram karbonhidrat veriyor. |
| Renk — MCU | `Σ lb·renk / gal`, gal = parti + trub, renk "SRM" alanı doğrudan °L gibi kullanılıyor (§8.1) | `Σ kg·°L·8.3454 / L`, L = parti + trub (`Recipe.cpp:989-1030`). Renk °L olarak saklanır, BeerJSON'dan SRM gelirse °L'ye çevrilir (`Fermentable.cpp:186-196, 351-355, 457-462`) | Renkler °L girilince birebir aynı (8.819). SRM olarak girilip °L'ye çevrilince −13 % | **BT kavramsal olarak doğru**: Morey MCU'su °L ile tanımlı. BS'nin "SRM'yi °L gibi kullanma" alışkanlığı açık renkte önemsiz, koyu maltlarda 60 SRM ≈ 44.9 °L farkı yaratır. |
| Renk — SRM formülü | Yalnız Morey `1.4922·MCU^0.6859` (§8.1) | Morey (varsayılan), **Daniels** `0.2·MCU+8.4`, **Mosher** `0.3·MCU+4.7` seçilebilir (`measurement/ColorMethods.cpp:40-52, 96-103`) | Morey birebir. Daniels/Mosher alternatif | Morey güncel standart. Daniels/Mosher yalnız MCU > ~10'da anlamlı (doğrusal, düşük MCU'da 8.4/4.7 tabanı hatalı). |
| Sıvı fermentable rengi | Miktar her zaman oz | Hacimle girilen fermentable'ın rengi/IBU'su **atlanıyor** (uyarı loglanır) (`Recipe.cpp:1008-1019`, `1268-1277`) | — | BT'de eksik. |
| Hidrometre sıcaklık düzeltmesi | °C polinomu, `SG + SG·[…]` (§9.1) | °F oran polinomu `SG·p(T)/p(Tc)` (`Algorithms.cpp:533-570`) | 77 °F: aynı. 100 °F: +0.00005. 150 °F: +0.0005 | Pratikte eşdeğer. İkisi de su yoğunluğu–sıcaklık eğrisinden. 150 °F gibi uç değerde sapma büyüyor ama orada iki model de güvenilmez. |
| Refraktometre, mayalanmamış wort | `ToSG((B − sıfır)/WCF)`, WCF 1.02, sıfır sapması ayarlı (§9.2 tip 0) | WCF ve sıfır **yok**. Okuma doğrudan °P kabul edilir (`tools/RefractoDialog.cpp:57-98`) | 12 °Bx: 1.04739 vs 1.04837 (~1 puan) | **BS doğru**: WCF (genelde 1.02–1.04) kalibrasyonu literatürde standart (Brewer's Friend, Terrill). |
| Refraktometre, mayalanan wort (FG) | Terrill kübik. OB = ToPlato(OG), WCF uygulanmaz (§9.2 tip 1) | Aynı Terrill kübik ("primetab"), OB = SG_toPlato(OG), WCF yok (`Algorithms.cpp:410-422`) | **Birebir** (1.01061) | Eşit. İkisi de Terrill'in WCF ile bölme önerisini atlıyor. WCF=1.04 ile FG 1.01027 olurdu (~0.3 puan). |
| Refraktometre ABV | Balling(OG, FG_düz) (§9.2) | BYO formülü `(277.8851 − 277.4·SG + 0.9956·B + 0.00523·B² + 0.000013·B³)·SG/0.79` (`Algorithms.cpp:394-400`) | +4.3 % | Balling (BS) tutarlı. BT aracı reçetedeki ABV (HMRC) ile de tutarsız. |
| Refraktometre ABW / RE | Bonham: `…RI·(937.8135·RI − 1805.1288)`, RE (§9.2 tip 2) | Aynı formül ama sabit **1805.1228** (`Algorithms.cpp:407`) | ABW +0.008 (≈ +0.2 %) | BS doğru. BT'de yazım hatası (kaynak primetab: 1805.1288). |
| Refraktometre, bitmiş biradan OG (FG hidrometreyle) | Var (tip 2) | Yok | — | BS'ye özgü özellik. |
| Gravite yükseltme (özüt ekleme) | `ΔP·gal / PPG`, DME 44, LME 36 (§9.3) | `kg = ΔGP·L·0.1/46` (OG düzeltme aracı) (`tools/OgCorrectionTool.cpp:279-302`) | 10 puan, 5 gal: BS DME 18.18 oz. BT 14.51 oz. Sukrozun doğru değeri 17.39 oz | **BT'de birim hatası**: 46 puan "1 lb/gal" içindir. SI'de 1 kg/L ≈ 384 puan, dolayısıyla kg = ΔGP·L/384 olmalı. BT /460 kullanıyor ve **%16.5 az** öneriyor. |
| Su ile seyreltme | `V·(G−1)/(H−1) − V` (§9.4) | Kütle/Plato bazlı: `100·şeker/P_hedef − şeker − su` (`OgCorrectionTool.cpp:186-263`). Kaynatma buharlaşmasını da modelliyor | 1.000 vs 1.007 gal | İkisi de kabul edilebilir. BT daha fiziksel (puanlar tam doğrusal değil). |
| Seyreltme/karıştırma aracı | Su ya da ikinci wort ile karıştırma (§9.5) | Ayrı araç yok | — | BS'ye özgü. |
| Tahmini OG (kaynatma öncesinden) | — | `BrewLog::calculateOg` (`BrewLog.cpp:490-500`) | — | BT'ye özgü (brew-day aracı). |
| Ölçekleme (OG hedefi) | Tüm fermentable'lar `P_yeni/P_eski` ile, ardından IBU korunur (§3.7) | Yok | — | BS'ye özgü. |
| Ölçekleme (hacim / verim) | Spesifikasyonda bu bölümde yok (BS "Scale Size" var, `beersmith4-ozellik-analizi.md:29`) | Hacim oranı × (tahıllar için) eski/yeni verim oranı. Hoplar yalnız hacimle. Mash adım hacimleri sıfırlanır (`tools/ScaleRecipeTool.cpp:75-137`) | — | BT basit ve doğru yönde. Ancak hop kullanımı gravite ile değiştiği için IBU yeniden dengelenmiyor. Mayalar ölçeklenmiyor. |
| Renk ayarlama (hedef SRM'ye) | Sezgisel iteratif algoritma (§8.3) | Yok | — | BS'ye özgü. |

---

## 2. Sayısal cross-check (betik çıktısı)

Betik: `/home/user/karsilastirma/testler/01_yogunluk_karsilastirma.py`. BS sütunu spesifikasyon test vektörlerini yeniden üretir. Değerler `01_yogunluk_tv.py` ile aynı: 5.276 / 1.0609 / 1.0102 / 8.819 / 1.05200 / 1.04739 / 1.01061 / 18.182 / 62.23. BT sütunu C++ formüllerinin Python'a taşınmış hâlidir. Fark = (BT−BS)/BS.

### 2.1 Dönüşümler
| Hesap | BeerSmith | Brewtarget | Fark |
|---|---|---|---|
| SG→°P 1.050 | 12.3876 | 12.3876 | 0.00 % |
| °P→SG 12 | 1.04838 | 1.04837 | −0.00 % |
| °P→SG 30 | 1.12917 | 1.12925 | +0.01 % |
| Brix→SG 12 | 1.04838 | 1.04837 (USDA tablo) | −0.00 % |
| SG→Brix 1.050 | 12.3870 | 12.3881 | +0.01 % |
| Brix→RI 10 | 1.347871 | 1.347871 | 0 |
| SRM 8.819→EBC | 17.373 | 17.360 | −0.08 % |
| 30 °L→SRM | 39.932 | 39.878 | −0.14 % |

### 2.2 Alkol / attenuasyon / kalori
| OG / FG | ABV BS (Balling) | ABV BT (HMRC) | Fark | AA BS (°P) | AA BT (puan) | kcal/12oz BS | kcal/12oz BT | Fark |
|---|---|---|---|---|---|---|---|---|
| 1.050/1.010 | 5.276 | 5.200 | −1.45 % | 79.33 | 80.00 | 165.2 | 163.7 | −0.9 % |
| 1.065/1.012 | 7.042 | 6.943 | −1.40 % | 80.71 | 81.54 | 218.2 | 213.3 | −2.2 % |
| 1.080/1.016 | 8.566 | 8.448 | −1.38 % | 78.90 | 80.00 | 274.0 | 264.2 | −3.6 % |
| 1.100/1.020 | 10.812 | 10.720 | −0.85 % | 78.62 | 80.00 | 350.8 | 331.9 | −5.4 % |
| 1.120/1.010 | 14.987 | **16.253** (yedek formül) | **+8.44 %** | 90.87 | 91.67 | 424.8 | 393.7 | −7.3 % |

BS'nin UK vergi ABV'si ile BT'nin HMRC sonucu ilk dört satırda **birebir** aynı (5.200 / 6.943 / 8.448 / 10.720). Son satırda BS 14.850 verirken BT tablo dışına çıkıp 16.253 veriyor.

### 2.3 Reçete OG/FG (BS §12.3 vektörü: 10 lb %80 + 1 lb %72 mash, 1 lb şeker boil, 5.5 gal, %72, maya 73/77, mash 152 °F)
| Senaryo | BS | BT | Fark |
|---|---|---|---|
| OG | 1.0609 | 1.0610 | +0.01 % |
| FG | 1.0102 | 1.0153 | +0.50 % (≈ 5 puan) |
| FG (BS mash düzeltmesi kapalı) | 1.0112 | 1.0153 | +0.40 % |
| ABV | 6.717 | 5.954 | **−11.4 %** |
| Kullanılan attenuasyon | 76.875 | 75.0 | — |
| 1 lb crystal **steep** edilince OG | 1.0574 | 1.0593 | +0.17 % |
| Steep tahılının puan katkısı (pts/gal) | 0.903 | 2.601 | **+188 %** |
| Trub kaybı 0.5 gal ile OG | 1.0609 | **1.0553** | −0.52 % |
| Extract (6 lb DME) + steep: OG / FG | 1.0534 / 1.0134 | 1.0555 / 1.0139 | +0.19 % / +0.05 % |
| +1 lb laktoz: OG / FG / ABV | 1.0672 / 1.0166 / 6.742 | 1.0674 / 1.0216 / 5.954 | — / +0.5 % / −11.7 % |
| OG 1.100, att %90: FG / ABV (BS tol = 10) | 1.0259 / 10.02 | 1.0100 / 12.15 | +21 % ABV |

FG farkının kaynağı: BS şekeri −0.231 katsayısıyla FG'den **düşürür** (−1.93 puan), mash sıcaklığıyla attenuasyonu yükseltir (+1.9 puan att). BT şekeri tahıl gibi %25 kalıntıyla bırakır (+2.1 puan).

### 2.4 Renk
| Senaryo | BS | BT | Fark |
|---|---|---|---|
| 10 lb 2 + 1 lb 60, 6 gal, renk °L olarak | 8.819 | 8.819 | 0 |
| Aynı, renkler SRM olarak girilmiş (BT °L'ye çevirir) | 8.819 | 7.667 | −13.1 % |
| BT Daniels / Mosher seçilirse | 8.819 | 11.067 / 8.700 | +25.5 % / −1.4 % |

### 2.5 Araçlar
| Hesap | BS | BT | Fark |
|---|---|---|---|
| Hidrometre 1.050 @77 °F (cal 60 °F) | 1.05200 | 1.05200 | 0 |
| Hidrometre 1.050 @100 °F | 1.05609 | 1.05614 | +0.005 % |
| Hidrometre 1.050 @150 °F | 1.06958 | 1.07004 | +0.04 % |
| Refrakto mayalanmamış 12 °Bx | 1.04739 (WCF 1.02) | 1.04837 (WCF yok) | +0.09 % (~1 puan) |
| Refrakto FG (OG 1.050, 6.5 °Bx) | 1.01061 | 1.01061 | 0 |
| Refrakto ABV | 5.196 (Balling) | 5.421 (BYO) | +4.3 % |
| Refrakto ABW (Bonham) | 4.0873 | 4.0953 | +0.2 % (1805.1228 yazım hatası) |
| 1.040→1.050, 5 gal ekstrakt (oz) | 18.18 (DME) / 17.39 (sukroz, 46 ppg) | 14.51 | −20 % / **−16.5 %** |
| 1.060→1.050, 5 gal su (gal) | 1.000 | 1.007 | +0.7 % |
| Brewhouse verimi (§12.6) | 62.23 % | 66.34 % | +6.6 % |

---

## 3. Özellik karşılaştırması (yalnız birinde olanlar)

**Yalnız BeerSmith'te:**
- Ayarlanabilir steep verimi (`O_STEEP_EFFICIENCY`).
- Sanal OG (secondary/tertiary eklemeleri ABV ve kaloriye dahil).
- Mash sıcaklığına bağlı FG düzeltmesi (sıcaklık, eğim, aç/kapa).
- Şeker ve meyve/bal için negatif FG katsayıları.
- Maya alkol toleransı ile FG sınırlama.
- Gerçek ekstrakt ve gerçek attenuasyon (reçete ve araç).
- Tahmini ve ölçülen mash verimi.
- Kaynatma için zaman profilli SG dizisi (geç özütün IBU'ya etkisi).
- Refraktometre: WCF, sıfır sapması, FG'den OG bulma (tip 2).
- DME/LME ayrımlı gravite yükseltme, seyreltme/karıştırma aracı.
- OG hedefine göre ölçekleme (IBU korunur), hedef SRM'ye renk ayarlama.
- UK vergi ABV'si ayrı alan.
- Kaloride alkol/karbonhidrat ayrımı ve gram karbonhidrat.

**Yalnız Brewtarget'ta:**
- Daniels ve Mosher renk formülü seçeneği.
- Gerçek °L↔SRM ayrımı (fermentable rengi °L saklanır).
- USDA tablo bazlı Brix↔SG.
- Kısmi `fermentableFraction_pct` (kısmen fermente olabilen malzeme).
- Nem (moisture), DBFG/CGDB/coarse-fine farkı ile verim.
- Birden çok maya varken en yüksek attenuasyonu seçme, ekleme bazlı attenuasyon.
- OG düzeltme aracı (kaynatma öncesi ölçümden su ekleme / kaynatma süresi / şeker önerisi; buharlaşma modelli).
- BrewLog'da kaynatma öncesinden tahmini OG.
- Hacim + verim oranıyla ölçekleme.
- 33 cl / pint / litre kalori birimleri.
- Hacimle girilen fermentable kavramı (ancak renk/IBU hesabında desteklenmiyor).

---

## 4. Hatalar ve tuhaflıklar

### 4.1 BeerSmith spesifikasyonunda işaretlenenler ve Brewtarget'taki karşılığı
| BS tuhaflığı (BS §) | Brewtarget ne yapıyor | Yorum |
|---|---|---|
| °L→SRM katsayısı 1.3564 (§1.5) | 1.3546 (`Unit.cpp:701-707`) | BS'deki yazım hatasını doğruluyor. Kendi uygulamada 1.3546 kullanılmalı. |
| Brix→SG'de 258.2 (§1.3) | Formülü kullanmıyor, tablo kullanıyor. Kod yorumunda aynı 258.2'li formülü "BYO/Brewer's Friend" kaynaklı olarak anıyor (`Algorithms.cpp:348-350`) | 258.2 BS'ye özgü değil, yayımlanmış formülün kendisinde var. Etkisi ihmal edilebilir. |
| FG toplamında aşama filtresi yok, Bottling şekeri FG'yi düşürür (§5.1 adım 3) | BT'de OG'de de aşama filtresi yok, şişeleme şekeri OG **ve** FG'ye girer | BT daha kötü durumda: priming şekeri OG'yi şişirir. İkisinde de "Bottling" (karbonasyon) eklemeleri hariç tutulmalı. |
| Tolerans kontrolünde sanal OG yerine `F_R_OG` (§5.1 adım 4) | Tolerans yok | — |
| Mash adım atlama koşulu `prev` (§5.1, §11.2) | Mash sıcaklığı modeli yok | Kendi uygulamada daha temiz bir kural önerilir: "sakkarifikasyon adımı" = 60–72 °C aralığında süre ağırlıklı ortalama sıcaklık. |
| IterateColor'daki %25 kuralı yanlış alanı karşılaştırıyor (§8.3) | Renk ayarlama yok | — |
| Extract reçetede pre-trub hacmi kullanımı (§3.3) | BT reçete tipine göre ayrım yapmıyor | BS mantığı: extract reçetede su eklenip toplam hacim hedeflenir. |
| Sıfıra bölme korumaları eksik (AA, su ile seyreltme) (§6.4, §9.4) | BT `abvFromOgAndFg` içinde `Q_ASSERT(og >= fg)` (yalnız debug derlemesinde). BrewLog'da OG > 1 kontrolü var | İkisinde de giriş doğrulaması zayıf. |

### 4.2 Brewtarget'ta bulunan hatalar / tuhaflıklar
1. **Trub kaybı iki kez sayılıyor** (`Recipe.cpp:922` + `1069-1093`): OG paydası `batch + trub`. Mevcut şekerler ek olarak `(postBoil−trub)/postBoil` ile azaltılıyor. Brewhouse verimi zaten tüm kayıpları içerdiği için 0.5 gal trubda ~9 puan düşük OG çıkıyor. İlgili test devre dışı (`RecipeCalcTests.cpp:172-174`).
2. **Şeker ve meyve attenuasyonla bırakılıyor** (`Recipe.cpp:1137-1150`): şekerli reçetelerde FG yüksek, ABV düşük (test: −11 %).
3. **ABV tablo dışında yedek formüle düşüyor** (`Algorithms.cpp:506-513`): Δ > 100.7 puanda (ör. barleywine, imperial stout) ABV %8 abartılıyor. Ayrıca HMRC tablosunun 0.1 puana yuvarlama ve "kesme" kuralı vergi amaçlıdır, tahmin için gereksiz.
4. **OG düzeltme aracında şeker miktarı %16.5 eksik** (`OgCorrectionTool.cpp:279-302`): "1 kg/L = 46 puan" varsayımı yanlış. Doğrusu ≈ 384 puan/(kg/L), formülde /460 yerine /384 olmalı.
5. **Bonham ABW sabiti yazım hatası** 1805.1228 (`Algorithms.cpp:407`), doğrusu 1805.1288.
6. **Refraktometre aracı ABV için reçeteden farklı formül** (BYO) kullanıyor, sonuçlar tutarsız (+4 %). WCF ayarı yok. `lineEdit_op->setQuantity(inputOG)` (`RefractoDialog.cpp:86`) OP alanına OG yazıyor gibi görünüyor (alan dönüşüm yapıyorsa zararsız) **[belirsiz]**.
7. **Steep kuralı yalnız `Type::Grain` için** (`RecipeAdditionFermentable.cpp:148`): mash dışındaki `Other_Adjunct` tam verimle sayılır. Ayrıca 0.60 ile brewhouse verimi çarpılıyor, bu da kaynakta açıklanmayan bir birleşim.
8. **Kaynatma gravitesinde** geç eklenen tahılın kütlesi verim uygulanmadan düşülüyor (`Recipe.cpp:1214-1217`). Küçük tutarsızlık.
9. **Kalori** `m_og`/`m_fg` (fermente olmayanlar dahil) ile, **ABV** ise yalnız fermente olabilen OG/FG ile hesaplanıyor. Laktozlu birada kalori/ABV tutarlı ama ilk bakışta kafa karıştırıcı.
10. **Ölçülen verim** şekeri paydaya katıyor (`BrewLog.cpp:518-530`). "Kettle'a verim" teorik maksimuma değil **beklenen** değere oran (`BrewLog.cpp:465-488`).
11. Hacimle girilen fermentable'lar renk ve IBU hesabında **atlanıyor** (`Recipe.cpp:1008-1019`).
12. Maya yokken attenuasyon 0, yani FG = OG (`Recipe.cpp:1121-1135`). Bu belgelenmiş ama BS'deki 73 % varsayılanından farklı.

---

## 5. Kendi uygulama için öneri

1. **Ekstrakt modeli:** Kütle → °P → SG (BT yaklaşımı) fiziksel olarak daha doğru ve SI dostu. Ancak puan/PPG modeliyle sonuç aynı çıkıyor (+0.01 %). Basitlik ve BeerSmith uyumu için **PPG puan modeli** de yeterli. Önerim: dahili olarak sukroz eşdeğeri kütle kullan, °P↔SG'yi tek polinom + sayısal ters ile hesapla (gidiş-dönüş tutarlı).
2. **Verim uygulama kuralları:** BS'nin tip/aşama matrisi doğru temel: mash/sparge tahılına brewhouse verimi, şeker/özüt/meyve/bal için verim yok. **Steep** için ayrı, kullanıcı ayarlı bir verim kullan. Varsayılanı %15 yerine kristal/karamel için ~%50–60 (BT'ye yakın), kavrulmamış baz malt steep'i için düşük bir değer olsun. Verim zaten fermentöre kadar kayıpları içerdiği için **hacim payda = fermentöre giden hacim** olmalı (BT'nin trub çift sayımından kaçın).
3. **Aşamalar:** OG = fermantasyon başına kadar. **Sanal OG** (secondary/tertiary eklemeleri dahil) ABV ve kalori için kullanılsın. Şişeleme/priming eklemeleri OG ve FG'den **hariç** tutulsun (BS'nin FG tuhaflığını kopyalama).
4. **FG modeli:** BS'nin malzeme bazlı modeli + BT'nin kısmi fermente olabilirlik oranı:
   - tahıl: `(1−att)`;
   - şeker: ~−0.23, meyve/bal: ~−0.05 (veya malzeme bazlı "fermentability %" üzerinden genelleştir);
   - laktoz/maltodekstrin: fermentableFraction ile;
   - mash sıcaklığı düzeltmesi **açılıp kapatılabilir** seçenek olarak (BS katsayıları varsayılan);
   - maya toleransı sınırı opsiyonel.
   - Çok mayalı reçetede "en yüksek attenuasyon" (BT) ya da "birincil maya" seçeneği.
5. **ABV:** Varsayılan **Balling** (BS formülü, ABW·FG/0.79661). Seçenek olarak "basit (OG−FG)·131.25", "Hall/Daniels" ve **HMRC (UK vergi)** sun. HMRC'yi tablo dışında 0.135 ile sürdür ya da uyarı ver, BT'deki yedek formüle düşme.
6. **Attenuasyon / ekstrakt:** Görünür attenuasyonu Plato bazlı hesapla. Gerçek ekstrakt ve gerçek attenuasyonu da göster (BS).
7. **Kalori:** Plato bazlı ASBC formülü (BT biçimi): `(6.9·ABW + 4·(RE−0.1))·FG·10` kcal/L. Alkol/karbonhidrat ayrımı ve birim seçenekleri (12 oz, 33 cl, 100 ml) sun.
8. **Renk:** Fermentable rengini °L sakla (ya da SRM saklayıp MCU için °L'ye çevir; katsayı **1.3546**). Morey varsayılan, Daniels/Mosher seçenek. EBC = SRM × 1.97. Hacim = parti + trub (iki program da aynı).
9. **Araçlar:**
   - refraktometre: WCF (varsayılan 1.02–1.04) + sıfır sapması (BS); FG için Terrill kübik (WCF'yi her iki okumaya uygulama seçeneğiyle); FG'den OG (Bonham, sabit 1805.1288);
   - hidrometre: herhangi biri (ikisi eşdeğer);
   - gravite yükseltme: PPG'ler doğru (sukroz 46, DME ~44, LME ~36). BT'nin /46·0.1 hatasını kopyalama;
   - seyreltme: kütle bazlı (BT) ya da puan bazlı, fark < %1;
   - ölçülen verim: kütle bazlı ölçüm + sabit verimli malzemelerin katkısı düşülmüş (BS tanımı), ayrıca mash verimi.
10. **Ölçekleme:** Hacim + verim ölçekleme (BT) ve OG hedefine ölçekleme (BS). Her ikisinden sonra IBU'yu yeniden dengele.
