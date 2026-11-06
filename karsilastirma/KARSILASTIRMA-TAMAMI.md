# BeerSmith 4 ↔ Brewtarget — Karşılaştırma Özeti

BeerSmith tarafı: binary'den çıkarılan formül spesifikasyonu (`formuller/`) ve özellik analizi. Brewtarget tarafı: GitHub kaynak kodu (commit `62d8229`). Her hesap alanında Brewtarget formülleri Python'a taşındı ve BeerSmith test vektörleriyle aynı girdilerde çalıştırıldı (`testler/`, hepsi `python3 -I <betik>` ile çalışır). Brewtarget'ta bulunan hatalar kod okunarak ve Python portuyla tespit edildi; Brewtarget programı derlenip çalıştırılarak doğrulanmadı.

## Dosyalar

| Dosya | Konu |
|---|---|
| `01-yogunluk-alkol-renk-karsilastirma.md` | OG/FG, verim, ABV, renk, refraktometre, kalori |
| `02-hop-ibu-karsilastirma.md` | Tinseth/Rager/Garetz/Noonan/mIBU, whirlpool, rakım, IBU hacmi |
| `03-su-ph-karsilastirma.md` | Tuz iyonları, RA, mash pH, asitleme, profil eşleştirme |
| `04-maya-mead-karbonasyon-karsilastirma.md` | Canlılık, starter, kuru maya, priming, fıçı basıncı, mead/şarap |
| `05-mash-hacimler-karsilastirma.md` | Strike/infüzyon/dekoksiyon, BIAB, sparge, hacim zinciri |
| `06-ozellik-veri-modeli-karsilastirma.md` | Özellik matrisi, veri modeli, mimari, standartlar, MVP önerisi |

## Sayısal karşılaştırma — öne çıkanlar

| Hesap (aynı girdi) | BeerSmith | Brewtarget | Fark |
|---|---|---|---|
| Tinseth IBU (1 oz %5, 60 dk, 5 gal, 1.050) | 17.21 | 17.27 | +%0.35 (mg/L sabiti) |
| Flameout + 20 dk soğuma IBU | 7.12 | 5.86 | −%18 (soğuma eğrisi) |
| 80 °C whirlpool 20 dk IBU | 2.39 | 0 (Tinseth) / 4.20 (mIBU) | model farkı |
| 5000 ft rakımda IBU | rakım etkisi var | rakım yok | Brewtarget +%41 |
| Örnek tarif FG / ABV | 1.0102 / %6.72 | 1.0153 / %5.95 | ABV −%11 |
| OG 1.100, maya toleransı | ABV %10.0 (sınırlı) | %12.15 | tolerans yok |
| Steep tahıl puan katkısı | %15 verim | yield×0.60×verim | +%188 |
| 5 g gypsum + 3 g CaCl₂ / 20 L, Ca | 112.3 ppm | 127.8 ppm | +%14 (hidrat hatası) |
| Örnek fatura mash pH | 5.58 (MPH) / 5.53 (BW) | 5.31 | −0.2…−0.27 |
| 3 aylık sıvı maya canlılığı | %70.7 | %34 | doğrusal vs üstel |
| Starter (stir'sız), 20 L 1.060 | 3.42 L | 8.15 L | +%138 (canlılıktan) |
| Priming, 2.4 vol 20 °C 19 L | 117.3 g | 129.8 g | +%10.7 (monohidrat) |
| Strike, 5 kg 3 L/kg 20→66 °C | 71.83 °C | 72.13 °C | +%0.4 |
| Pre-boil (23 L fermentör) | 29 L | 28 L | soğuma büzülmesi yok |
| Toplam su / sparge | 35.007 / 19.007 L | aynı | %0 |

**Birebir aynı çıkanlar:** SG↔Plato, Morey renk, Terrill refraktometre FG, hidrometre düzeltmesi, Kolbach RA, hücre hedefi formülü, White starter eğrisi, Tinseth çekirdeği, toplam su/sparge.

## Genel değerlendirme

- **BeerSmith** formül kapsamında açıkça önde: FG modeli (mash sıcaklığı, şeker, tolerans), sanal OG, rakım, su eşleştirme ve asit hesapları, maya canlılığı, mead/şarap, fıçı basıncı, batch sparge. Kendi içinde küçük tutarsızlıkları var (1.3564 Lovibond katsayısı, fosforik 0.365/0.375, şeker sabitleri, kullanılmayan ayarlar).
- **Brewtarget** temel dönüşümlerde doğru ve BeerJSON/DotBeer, lot bazlı envanter, esnek profiller, PostgreSQL ve çok dil desteğinde önde; mIBU ve Noonan gibi ek IBU modelleri var. Ancak su/pH motorunda ciddi birim ve stokiyometri hataları, trub kaybının iki kez sayılması, hopped extract IBU hatası ve soğuma büzülmesi eksikliği var. **Su/pH için formül kaynağı olarak kullanılmamalı.**

## Kendi uygulama için birleşik öneri (kısa)

1. **Yoğunluk/ABV:** BeerSmith'in malzeme bazlı FG modeli + Brewtarget'ın kısmi fermente olabilirlik oranı; mash sıcaklığı ve tolerans açılıp kapatılabilir; ABV varsayılan Balling, HMRC ve basit formül seçenek; renk °L ve 1.3546.
2. **IBU:** Varsayılan Tinseth (tam SI sabiti); geç hop/whirlpool için mIBU (üstel soğuma) + sabit sıcaklıkta hop stand seçeneği; IBU hacmi parti+trub; geç ekstrakt segmentleri; rakım seçenek; hop yaşlanması.
3. **Su/pH:** BeerSmith iyon tablosu (molar kütleden, hidrat formları açık); malt bazlı grist pH + MPH/BW eğim seçeneği; pKa tabanlı asit hesabı; ağırlıklı NNLS eşleştirme.
4. **Maya/karbonasyon:** BeerSmith üstel canlılık; White eğrisi (kırpılmış) + isteğe bağlı Braukaiser; iki kademe starter; priming'de Brewtarget'ın stokiyometrisi + BeerSmith'in kalıntı CO₂ formülü; fıçı basıncı; mead/şarap BeerSmith formülleri (YAN hatası düzeltilmiş).
5. **Mash/hacim:** Tek ısı dengesi fonksiyonu; tahıl hacmi 0.652 L/kg; ekipman bazlı emilim + ayrı BIAB; %4 soğuma büzülmesi; rakımdan kaynama noktası; BeerSmith batch sparge algoritması.
6. **Veri modeli:** Kütüphaneye referans + hesapta kullanılan özelliklerin anlık kopyası; değişmez tarif sürümleri + ayrı Batch kaydı; UUID/updated_at/senkron durumu; hareket defteri envanter; iç birim SI; BeerJSON'a yakın model, BeerXML içe aktarma şart; cihaz entegrasyonu için adaptör katmanı.


---

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


---

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


---

# 03 — Su kimyası / mash pH: BeerSmith 4 ↔ Brewtarget karşılaştırması

**Kaynaklar**
- BeerSmith: `/home/user/formuller/03-su-ph.md` (bölüm numaraları "BS §x" olarak geçer), referans betik `formuller/testler/03_su_ph_tv.py`.
- Brewtarget 5.2.0 (`/home/user/src/brewtarget`, "BT" olarak geçer): `src/widgets/WaterAdjuster.cpp` (WA), `src/model/Misc.cpp`/`Misc.h`, `src/measurement/PhysicalConstants.h` (PC), `src/qtModels/tableModels/RecipeAdditionMiscTableModel.cpp` (RAMTM), `src/qtModels/tableModels/RecipeAdditionFermentableTableModel.cpp` (RAFTM), `src/model/Mash.cpp`, `src/model/Water.h`, `dev-doc/water.markdown`, `data/default_db.sqlite`.
- Karşılaştırma betiği: `/home/user/karsilastirma/testler/03_su_ph_karsilastirma.py` (çalıştırma: `python3 -I 03_su_ph_karsilastirma.py`). BT formülleri **hatalarıyla birlikte** ("as-is") taşındı. Ayrıca hataları düzeltilmiş bir "niyet edilen" varyant da hesaplanıyor. BS portu spec T3 değerlerini birebir üretiyor (betik §L: 5.6234/5.6253 pH, 3.521/1.771 mL).

**Kısa hüküm:** BeerSmith'in su/pH motoru basit ama tutarlı. Ampirik bir doğrusal model kullanıyor, kimyasal sabitleri doğru, asit ve sparge hesapları var, NNLS ile profil eşleştiriyor. Brewtarget'ın "Water Chemistry" aracı yalnızca ileri yönlü bir pH tahmini yapıyor. Asit miktarı çözücüsü, sparge asitleme ve profil eşleştirme yok. Pek çok birim ve stokiyometri hatası var, bazıları sonucu tamamen bozuyor. Örneğin 1 mL laktik asit pH'ı 78 birim düşürüyor ve RO %50 seçilince pH −6.7 çıkıyor. Brewtarget kendi uygulamanıza **formül kaynağı olarak alınmamalı**. Yalnızca molar kütle tablosu (PC:66-102, OH hatası hariç) ve tuzları açık hidrat formlarıyla enum'lama fikri alınmaya değer.

---

## 1. Hesap bazında karşılaştırma

| Hesap | BeerSmith (BS) | Brewtarget (BT) | Fark | Hangisi daha doğru |
|---|---|---|---|---|
| **Tuz tanıma** | Misc adına göre otomatik WA tipi ("Gypsum", "CaCl" …) (BS §0.2) | Açık `WaterAgentType` enum'u (Misc.h:167-183). BeerXML/JSON'dan gelen veya varsayılan içerikteki misc'lerde tip yok ve `Other` sayılıyor (Misc.cpp:266-278). `Other` ise hiç iyon katmaz. `default_db.sqlite`'taki "Gypsum", "Calcium Chloride" vb. kayıtlarda tip alanı yok. | BS isimle çalışır ama kırılgandır. BT açıktır ama varsayılan veriyle sıfır etki verir. | Açık enum (BT yaklaşımı) daha sağlam, ama varsayılan veri doldurulmalı. |
| **Desteklenen tuzlar** | 8 tuz: gypsum, NaCl, epsom, CaCl₂ (yalnız susuz), NaHCO₃, CaCO₃, MgCl₂·6H₂O, Ca(OH)₂ (BS §1.1) | 9 tuz: CaCl₂ susuz **ve** ·2H₂O, CaCO₃, CaSO₄·2H₂O, Ca(OH)₂, MgSO₄·7H₂O, MgCl₂·6H₂O, NaCl, NaHCO₃ + 4 asit (laktik, fosforik, sülfürik, HCl) (Misc.h:167-180) | BT'de CaCl₂ dihidrat da var. | BT'nin kapsamı daha iyi. CaCl₂ dihidrat piyasada yaygın. |
| **İyon katkı faktörleri** | Sabit `a_matrix` (ppm / 1 g / US gal). Stokiyometriyle ±0.05 % tutarlı, hidrat formları doğru (BS §1.2). | Molar kütleden hesaplanıyor (Misc.cpp:409-563, PC). Faktörlerin çoğu BS ile ±0.03 % aynı, **istisnalar şunlar:** gypsum-Ca **susuz CaSO₄** ile hesaplanıyor (Misc.cpp:414; +26.5 %, SO₄ ise dihidratla, yani tutarsız). MgCl₂ → Cl = 0 (Misc.cpp:444). Ca(OH)₂ → alkalinite 0 ve molar kütle hatalı (PC:84 `Hydroxide = H + 2·O`; +1.4 % Ca). CaCO₃ → HCO₃ değil CO₃ (Misc.cpp:458; 158.4 vs 322.0 HCO₃-eşdeğeri). | Betik §A: gypsum Ca +26.5 %, MgCl₂ Cl −100 %, kireç alkalinitesi −100 %, tebeşir alkalinitesi farklı temsil ediliyor. | **BS doğru.** BT'de 4 stokiyometri hatası var. |
| **Tuzun dağıtıldığı hacim** | Kullanım alanına göre: Mash → toplam mash suyu, Sparge → sparge hacmi, Boil → kaynatma hacmi … (BS §2) | Tüm tuzlar `totalMashWater_l` hacmine dağıtılıyor. Bu hacim infüzyon + **sparge** adımlarını içeriyor (WA:722-723, Mash.cpp:170-180). Mash ve sparge ayrımı yok. | BT, sparge'ı olan bir reçetede mash iyonlarını olduğundan düşük gösteriyor. | **BS.** |
| **Baz su karışımı / seyreltme** | Birden çok su malzemesinin miktarla ağırlıklı ortalaması (BS §3.1) | Baz su × (1 − RO oranı). Mash ve sparge için ayrı RO % (WA:67-88, 703-715). | Eşdeğer işlev, farklı arayüz. | Eşit. RO yüzdesi kullanıcı için pratik bir seçenek. |
| **Alkalinite** | `HCO₃·50/61` (BS §4.1) | pH hesabı **`Water::alkalinity_ppm` alanını** kullanıyor (WA:276-280), `bicarbonate_ppm` alanını kullanmıyor. Alan "CaCO₃" olarak işaretliyse ×1.22. Varsayılan 14 su profilinin hepsinde `alkalinity = 0`. | **BT'de bikarbonat pH hesabına hiç girmiyor** (kullanıcı ayrıca alkalinite girmedikçe). | **BS.** |
| **RA (Kolbach)** | `Alk − (Ca/1.4 + Mg/1.7)` ppm CaCO₃ (BS §4.3) | `HCO₃/61 − (2Ca/40.08)/3.5 − (2Mg/24.31)/7` mEq/L (WA:346-354) | Matematiksel olarak aynı. Betik §C: 81.354 vs 81.429 ppm (+0.09 %, yalnızca yuvarlama). RO > 0 olunca BT'de hata var (aşağıda). | Formül aynı. BS'nin uygulaması güvenilir. |
| **RA gösterimi, SO₄/Cl oranı, renk aralığı** | Var (BS §4.3-4.5) | Yok. Sadece hedef profilin ±%5 aralığında renklenen rakamlar ve radar grafiği var (WA:250-267). | | BS. |
| **Grist (saf su) pH'ı** | Malt bazında: `5.72 − 0.0337·Σ(w/W)·g(SRM)`. g, renk eşiklerine göre parçalı tanımlı (<14 / 14-200 / ≥200) (BS §5.1). | **Reçete düzeyinde**, Troester "Beer color to mash pH v2.0": `5.6 − (1/°P)·[0.21(1−r) + 0.06r]·SRM_bira` (WA:290-333). r = kristal/kavrulmuş renk oranı; DP < 1 Lintner olan malt kristal/kavrulmuş sayılıyor, >120 SRM için Lovibond 19'a sabitleniyor. | BT grist pH'ı OG'ye ve **bitmiş biranın rengine** (yani batch hacmine) bağlı. Betik §D: BS 5.622, BT 5.395 (−0.23). Koyu fatura (§N): BS 5.37, BT 5.02 (−0.35). | İkisi de ampirik. BS malt bazlı olduğu için hacimden bağımsız, bu yüzden fiziksel olarak daha doğru. BT'nin r hesabında da hata var (`=` / `+=`, aşağıda). Literatüre göre ideal model malt başına DI pH + tampon kapasitesi (Troester 2009, deLange); bu ikisinden hiçbiri değil. **Emin değilim** hangisinin ölçülen pH'a daha yakın olduğundan; ölçüm verisi yok. |
| **RA → pH eğimi** | MPH: `0.037 + 0.014·t` [L/kg]; BW: `0.17·t[qt/lb]/1.5` (BS §5.2); t = 3 L/kg için 0.079 / 0.163 pH/(mEq/L) | `ΔpH = RA·V_inf / t / 50 = RA·W_kg/50` (WA:354-358). Etkin eğim = W/50 (4.9 kg için 0.098). Yorumda "makale gristpH + RA·thickness/mEq diyor, çalıştıramadım" yazıyor (WA:355-357). | BT'de eğim **tahıl kütlesiyle orantılı**, yani boyutsal hata var. Betik §I: her şey ×2 alındığında BS pH'ı değişmiyor, BT'de 5.312 → 5.229 oluyor. | **BS.** Troester'in formülü RA·t/BC şeklinde (BC ≈ 35-50 mEq/(kg·pH)). |
| **Tuzların pH etkisi** | Tuzlar mash profiline girer, RA'yı değiştirir, eğimle pH'a yansır (BS §5.3) | Ayrı terim (WA:364-378): toplam mEq / t / 50. HCO₃ ve CO₃ **iki kez /61** bölünüyor (önce molar kütleye, sonra 61'e), bu yüzden NaHCO₃ ve CaCO₃'ün alkalinitesi yaklaşık 61 kat küçük çıkıyor. Ca katkısı da gypsum hatası yüzünden %26 fazla. | Betik §H: 3 g NaHCO₃ için BS +0.19/+0.40, BT **+0.004**. 3 g tebeşir için BS +0.23/+0.47, BT **−0.11** (tebeşir pH'ı *düşürüyor*, yön ters). | **BS.** |
| **Hazır asidin pH etkisi** | Laktik/fosforik: gram → mmol → (BW'de pKa'ya göre ayrışma kesri) → mEq/L → ×eğim (BS §5.4a) | Laktik: `totalAcid_kg` içinde **fazladan ×1000** var (RAMTM:244), birimi aslında mg. Sonra `1000·x/90/50/t` (WA:383-401). Fosforik: g → mmol/50/t (W ve V'den bağımsız). Sülfürik ve HCl enum'da var ama **hesaba girmiyor**. | Betik §F: 1 mL %88 laktik için BT ΔpH **78.2**, BS-BW 0.127. | **BS.** BT laktik hesabı kullanılamaz durumda. |
| **Asit malt** | `P%·kg·1000/90.09` mmol → mEq/L (BS §5.4b); ayrıca hem W'ya hem grist'e eklenir | `lacticAcidByWeight_pct · kg` (**/100 yok**, RAFTM:149), sonra 1000·x/90/50/t. Ayrıca `newTotals` sinyali yorum satırına alınmış, yani asit malt değişince pH yeniden hesaplanmıyor (WA:542-546). | Betik §G: 100 g %3 asit malt için BS 0.17/0.35, BT 0.024. | **BS.** |
| **Asit yoğunluğu** | Laktik `1+0.237p`; fosforik `1+0.49p+0.365p²` (bir yerde 0.375) (BS §5.4, §6.2) | Doğrusal: laktik `1+0.2·pct/88`, fosforik `1+0.685·pct/85` (RAMTM:226-249) | Betik §K: %88 laktik 1.2086 vs 1.2000; %10 fosforik 1.053 vs 1.081 (+2.7 %). Literatürde %10 H₃PO₄ ≈ 1.053, %85 ≈ 1.685, %88 laktik ≈ 1.21. | **BS** (seyreltik fosforikte belirgin olarak daha iyi). |
| **Asit ayrışması (pKa)** | MPH: 1 eq/mol. BW: Henderson-Hasselbalch kesri, pH 5.4'te laktik 0.972, fosforik 1.015 (BS §6.1). 8 asidin pKa tablosu var. | Laktik ve fosforik için 1 eq/mol sabit, MW 90 / 98. | BS BW modeli daha doğru. Fark ≈ %3. | **BS (BW).** |
| **Gereken asit miktarı (hedef pH)** | `PHAcidAmount`: ΔpH/eğim → mL ya da g asit malt; asit malt için 8 turluk yineleme (BS §6.2-6.4) | **Yok.** Sadece ileri yönlü pH gösterimi var. | | BS. |
| **Sparge suyu asitleme** | MPH: kapalı karbonat sistemi yük dengesi; BW: pH 4.3'e göre titrasyon (BS §6.5) | **Yok.** Su pH'ı alanı var (Water.h:197) ama hiçbir hesap kullanmıyor. | | BS. |
| **Profil eşleştirme** | Lawson-Hanson NNLS, 6 iyon eşit ağırlıklı, tebeşir/kireç/MgCl₂ hariç tutulabilir (BS §7) | **Yok.** Hedef profil yalnızca gösteriliyor (radar grafiği, ±%5 renk) (WA:233-267). | | BS. |
| **Kaynatma/fermentör iyon profili** | Var (BS §3.3-3.4) | Yok | | BS. |

## 2. Sayısal cross-check (betik çıktısı)

Ortak senaryo: T3 faturası (4.5 kg Pale 2 SRM, 0.3 kg C60, 0.1 kg Chocolate 350 SRM), W = 4.9 kg, t = 3 L/kg, mash suyu 14.7 L, baz su Ca50 Mg10 Na15 SO₄40 Cl30 HCO₃150. BT'nin grist modeli OG ve bira rengine ihtiyaç duyuyor. Bunlar için şu değerler varsayıldı: batch 20 L, %75 verim, PPG 37/34/28 → OG 1.0562 (13.85 °P), Morey 13.9 SRM. BT'de alkalinite alanı = 150 (HCO₃ olarak) girildi. "BT düzeltilmiş" varyantı, hatalar giderildiğinde Troester formülünün (RA·t/50, gypsum dihidrat, ×1000 yok, `+=`) vereceği değerdir.

| Test | BS MPH (m0) | BS BW (m1) | BT as-is | BT as-is − BS BW | BT düzeltilmiş |
|---|---|---|---|---|---|
| T1: 20 L'ye 5 g gypsum + 3 g CaCl₂: Ca ppm | 112.33 | 112.33 | 127.77 | **+13.7 %** | 112.37 |
| T1: SO₄ / Cl ppm | 139.45 / 95.81 | aynı | 139.48 / 95.83 | +0.02 % | aynı |
| T2: RA (ppm CaCO₃) | 81.354 | 81.354 | 81.429 | +0.09 % | — |
| Grist pH (saf su) | 5.6216 | 5.6216 | 5.3952 | −0.226 (−4.0 %) | 5.4373 |
| Mash pH, baz su + tuzlar (mash 14.7 L) | 5.5777 | 5.5310 | 5.3119 | −0.219 | 5.4043 |
| ↳ BT alkalinite alanı boş (varsayılan profil) | — | — | 5.0709 | −0.460 | — |
| 5.40 için %88 laktik (tuzlu senaryo) | 2.801 mL | 1.030 mL | 0 (zaten 5.31) | — | 0.089 mL |
| Mash pH, yalnız baz su | 5.7502 | 5.8868 | 5.5548 | −0.332 | 5.5351 |
| 5.40 için %88 laktik (tuzsuz) | **5.519 mL** | **3.827 mL** | **0.00198 mL** | −99.95 % | 2.822 mL |
| 1 mL %88 laktik → ΔpH | 0.063 | 0.127 | **78.2** | ×615 | 0.048 |
| 100 g asit malt %3 → ΔpH | 0.174 | 0.350 | 0.024 | −93 % | 0.133 |
| 3 g NaHCO₃ (saf su) → ΔpH | +0.192 | +0.396 | +0.004 | −99 % | — |
| 3 g CaCO₃ (saf su) → ΔpH | +0.230 | +0.474 | **−0.111** (yön ters) | — | — |
| Ölçek ×2 (tahıl, su, tuz) | 5.578 → 5.578 | 5.531 → 5.531 | 5.312 → **5.229** | hacme bağımlı | 5.404 → 5.404 |
| Mash RO %50 | — | — | **−6.70** | — | 5.355 |
| Koyu fatura (38 SRM) grist pH | 5.370 | 5.370 | 5.021 | −0.35 | 5.114 |

Yorum:
- İki BS modeli arasında, aynı girdiyle, gereken asit miktarı **yaklaşık 1.4-2.7 kat** fark ediyor (MPH ≫ BW). Bunun nedeni eğim farkı: t = 3'te 0.079 ile 0.163 arası.
- BT'nin hata içermeyen tek parçası Kolbach RA formülü. Hatalar düzeltildiğinde bile BT tahminleri BS-BW'ye göre yaklaşık −0.13 pH sapıyor, çünkü grist modeli farklı. Düzeltilmiş BT'nin eğimi t/50 = 0.06, yani BS-MPH'a (0.079) yakın.
- BS portunun doğruluğu: spec T3 (5.6234 / 5.6253 pH; 3.521 / 1.771 mL) birebir üretildi (betik §L). Bu tablodaki tuzlu senaryo, tuzlar 20 L yerine 14.7 L mash suyunda çözüldüğü için spec T3'ten farklı.

## 3. Özellik karşılaştırması

| Özellik | BS | BT |
|---|---|---|
| Hidrat formları açıkça seçilebiliyor (CaCl₂ susuz/dihidrat) | ✗ (yalnız susuz; kullanıcıya gösterilmiyor) | ✓ |
| Tuzu mash / sparge / boil / fermentöre ayrı ekleme | ✓ (6 kullanım tipi) | ✗ (hepsi toplam mash+sparge suyuna) |
| Kaynatma ve fermentör iyon profili, kaynatmayla derişme faktörü | ✓ | ✗ |
| RO / seyreltme | Su malzemesi karışımıyla | ✓ Mash ve sparge için ayrı RO % |
| Alkalinite ayrı girişi (CaCO₃ veya HCO₃ olarak) | ✗ (yalnız HCO₃) | ✓ (`alkalinityAsHCO3`), ancak bikarbonat alanıyla bağlantısı yok |
| RA, efektif sertlik, SO₄/Cl oranı ve etiket, RA'ya göre renk aralığı | ✓ | ✗ |
| İki mash pH modeli (MPH/BW) | ✓ | ✗ (tek ve hatalı Troester uyarlaması) |
| Asit miktarı çözücüsü (laktik, fosforik, asit malt yinelemeli) | ✓ | ✗ |
| Sparge suyu asitleme (karbonat dengesi / BW) | ✓ | ✗ |
| Ölçülen pH'tan düzeltme ("ölçümden" modu) | ✓ | ✗ |
| Sülfürik / HCl / sitrik asit | pKa tablosunda var, kullanıcıya açık değil | Enum'da var, hesapta yok |
| Hedef profile otomatik tuz eşleştirme (NNLS, tuz hariç tutma) | ✓ | ✗ |
| Grafik (radar) | ✗ | ✓ |
| pH tahmini bitmiş biranın rengine / OG'ye bağlı | ✗ (malt bazlı) | ✓ (bu bir dezavantaj) |

## 4. Hatalar ve tuhaflıklar

### 4.1 Brewtarget (kodda doğrulandı)
1. **Laktik asitte ×1000 fazladan çarpan** (RAMTM:244 `1000 * quantity * mult * density`, `mult = 1000`). 1 mL laktik pH'ı 78 birim düşürüyor.
2. **Asit malt % değeri /100 yapılmıyor** (RAFTM:149). Buna rağmen etki yine de küçük çıkıyor, çünkü `calculateAcidpH` mol cinsinden toplayıp 50·t'ye bölüyor (WA:394-400; mmol/L'ye çevrilmiyor).
3. **Gypsum Ca katkısı susuz CaSO₄ ile hesaplanıyor** (Misc.cpp:414), SO₄ ise dihidratla (Misc.cpp:546). Ca %26.5 fazla çıkıyor.
4. **MgCl₂·6H₂O**: Ca fonksiyonundaki `break` yüzünden `Q_UNREACHABLE()`'a düşüyor (Misc.cpp:427-431). Bu tanımsız davranış, debug derlemede çökme demek. Klorür katkısı 0 (Misc.cpp:444).
5. **`Hydroxide = H + 2·O`** (PC:84). Doğrusu H + O. `CalciumHydroxide = Ca + Hydroxide` da tek OH içeriyor (PC:99). Sonuç M = 73.08, doğrusu 74.09. Ayrıca Ca(OH)₂ hiç alkalinite katmıyor.
6. **CaCO₃ ve NaHCO₃ alkalinitesi iki kez /61 bölünüyor** (WA:370-376). Bu yüzden tebeşir pH'ı *düşürüyor*. CO₃ 1 eşdeğer sayılıyor, doğrusu 2.
7. **RA → pH dönüşümü boyutsal olarak hatalı**: `RA·V/t/50 = RA·W/50` (WA:354-358, 377). pH batch büyüklüğüne bağlı oluyor.
8. **RO hatası**: `1.0 - roWaterMash_pct()` ifadesinde değer 0-100 aralığında (WA:276). %50 RO → katsayı −49 → pH −6.7. Ca/Mg ise doğru oranı kullanıyor (`waterBase_fraction`) ve bu oran mash+sparge'a göre hesaplanıyor. Tutarsız.
9. **Bikarbonat pH hesabına girmiyor**: yalnızca `alkalinity_ppm` kullanılıyor ve varsayılan profillerde bu değer 0 (`data/default_db.sqlite`). Burton gibi HCO₃ = 300 olan su, pH için saf su gibi davranıyor.
10. **Grist pH'ında `=` yerine `+=` olmalı** (WA:314): yalnızca son kristal/kavrulmuş malt sayılıyor. Kavrulmuş maltlar Lovibond 19'a sabitleniyor, ama payda (`m_weightedColor_lovibond`) sabitlenmemiş renkle hesaplanıyor.
11. `setRecipe` içinde `m_totalGrains_kg` ve `m_weightedColor_lovibond` sıfırlanmadan biriktiriliyor (WA:600-642). Aynı reçete tekrar açılınca thickness yarıya iniyor ve oran kayıyor.
12. Tuz iyonları sparge dahil toplam suya bölünüyor (WA:722-723). Mash'e eklenen tuz mash'te olduğundan seyreltik görünüyor. pH terimi ise hacme hiç bölünmüyor.
13. Hacimli asitlerde "1 L = 1 kg" varsayımı (RAMTM:137-141): mL cinsinden girilen asit iyon hesaplarında kütle gibi sayılıyor. BS'deki `ToGrams` mL tuhaflığının aynısı (BS §9.5).
14. `dev-doc/water.markdown` artık eski. Ayrı `salt` tablosunu anlatıyor; kod Misc'e taşınmış, tablo boş.

### 4.2 BeerSmith "belirsiz/tuhaflık" maddeleri, Brewtarget ile karşılaştırmalı
| BS maddesi (§9) | Brewtarget'ta durum |
|---|---|
| 1. Fosforik yoğunluk katsayısı 0.375 / 0.365 tutarsız | BT tek, doğrusal bir yoğunluk kullanıyor (tutarlı ama seyreltikte %2.7 hatalı). Önerim: tek fonksiyon kullanın, 0.365 ölçümlere daha yakın (%85 → 1.680 vs gerçek ≈ 1.685; 0.375 → 1.687). Fark binde birler düzeyinde. |
| 2. Sparge asidi hacmi "toplam − W·t"; sparge asidi mash ΔpH'a bağlı | BT'de sparge asitleme yok. BS mantığı tuhaf: sparge suyu alkalinitesi mash pH'ından bağımsız olmalı; koşul sparge suyu pH'ı > hedef olmalı. |
| 3. Sparge asidinde sparge tuzları dikkate alınmıyor | BT de mash/sparge tuzlarını ayırmıyor. İkisi de eksik. |
| 4. `S_boil` yalnızca kaynatma malzemesi varsa uygulanıyor | BT'de kaynatma profili yok. |
| 5. `ToGrams` mL'yi gram sayıyor | BT aynı varsayımı yapıyor (4.1/13). |
| 6. Su profili aracında gram yalnız baz hacimle ölçekleniyor | BT'de eşdeğer araç yok. |
| 7. MPH/BW model kaynakları belirsiz | BT Troester v2.0'ı kaynak gösteriyor (WA:58-62) ama uygularken bozmuş. BS'nin eğimleri Troester'in (t/BC) yapısına benziyor; BW eğimi yaklaşık 2 kat dik. **Hangisinin ölçüme daha yakın olduğundan emin değilim.** Bru'n Water ve Troester'in yayımlanmış verisiyle doğrulanması gerekiyor. |
| 8. SO₄/Cl Cl = 0 koruması | BT oran hesaplamıyor. |
| 9. Tahıl rengi birimi SRM | BT'de kanonik birim de SRM (`color_srm`), Lovibond'a `(SRM+0.6)/1.35` ile çeviriliyor. |
| Ek: CaCO₃ = 2 HCO₃ (BS §1.2) | BT CO₃ olarak tutuyor ve hatalı kullanıyor. BS'nin "CO₂ ile çözünmüş, 2 HCO₃" varsayımı ev tipi pratikte (tebeşir mash'te kısmen çözünür) iyimser bir üst sınır. Gerçek etki genellikle daha az, ama BS'ninki tutarlı. |

## 5. Kendi uygulamanız için öneri

1. **İyon tablosu:** BS'nin `a_matrix` değerlerini ya da molar kütleden türetmeyi kullanın (BT'nin PC:66-102 tablosu OH düzeltilirse iyi). Tuzları **açık hidrat formuyla** enum'layın: CaCl₂ susuz ve dihidrat, gypsum dihidrat, epsom heptahidrat, MgCl₂ hekzahidrat. Bu BT'nin iyi fikri. Formu arayüzde gösterin. Ad eşleştirmeyi yalnızca içe aktarmada ipucu olarak kullanın.
2. **Tuz hacmi:** BS gibi kullanım bazlı olsun (mash / sparge / boil / fermentör). Mash ve sparge profillerini ayrı hesaplayın, birleşik profil ve kaynatma derişmesi de olsun (BS §3).
3. **RA:** Kolbach RA'yı kullanın (iki program aynı). Alkaliniteyi tek kaynaktan alın: HCO₃ veya "CaCO₃ olarak" girilsin ve birbirine çevrilsin. BT'deki çift alan karmaşasından kaçının.
4. **Mash pH modeli:** Kısa vadede BS'nin doğrusal yapısını alın (`pH = grist_pH + m(t)·(RA + asit)/50`). Grist pH malt bazlı ve hacimden bağımsız olsun (BS §5.1). Eğim modeli seçilebilir olsun (MPH / BW); varsayılan olarak, hangisinin doğrulandığını bilmediğim için ikisini birden gösterip bir "kalibrasyon" alanı eklemek mantıklı. Uzun vadede daha doğru model şu: **malt başına DI pH + tampon kapasitesiyle proton-yük dengesi** (Troester/deLange; Bru'n Water ve MME bunu kullanıyor). Su alkalinitesini hedef pH'a göre karbonat kimyasıyla (BS §6.5 MPH denklemiyle aynı), asitleri pKa kesriyle (BS §6.1) hesaplayıp ΣmEq = 0'ı ikiye bölme (bisection) ile çözün. **BT'nin "bira rengi/°P" modelini almayın**, çünkü sonucu batch hacmine bağlıyor.
5. **Asitler:** pKa tablosu ve ayrışma kesri (BS §6.1) ile tek bir yoğunluk fonksiyonu kullanın: laktik `1+0.237p`, fosforik `1+0.49p+0.365p²`. Laktik, fosforik, asit malt, sülfürik ve HCl'yi destekleyin; sitrik/asetik için pKa zaten var. Asit malt hem grist ağırlığına hem asit terimine girmeli. Kapalı çözüm yerine yinelemeli ya da bisection çözücü kullanın (BS §6.4).
6. **Sparge asitleme:** BS'nin MPH (karbonat dengesi) denklemini kullanın. Hacim olarak doğrudan sparge hacmini alın. Koşul olarak mash ΔpH'ı değil, sparge suyu pH'ını kontrol edin. Sparge tuzlarının alkalinitesini de katın.
7. **Profil eşleştirme:** NNLS'yi alın, ama **iyon başına ağırlık** ekleyin (örneğin 1/hedef ya da kullanıcının seçtiği önem). Eşit ağırlıkta SO₄ ve HCO₃ baskın çıkıyor (BS §7.1). Tuz hariç tutma ve mash+sparge birleştirme seçenekleri kalsın.
8. **Test:** Bu karşılaştırmadaki ölçek testi (her şey ×2 alındığında pH değişmemeli) ve tek tuz yön testleri (soda ve tebeşir pH'ı artırmalı, CaCl₂ ve gypsum düşürmeli) birim testi olarak eklenmeli. BT'deki hataların çoğu bu iki testle yakalanırdı.


---

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


---

# 05 — Mash ısı dengesi ve hacim zinciri: BeerSmith 4 ve Brewtarget karşılaştırması

Kaynaklar:
- **BS** = `/home/user/formuller/05-mash-hacimler.md` (§ numaraları) ve `/home/user/formuller/testler/05_mash_tv.py`
- **BT** = `/home/user/src/brewtarget` (dosya:satır)
- Sayısal cross-check: `/home/user/karsilastirma/testler/05_mash_bt_vs_bs.py` (`python3 -I` ile çalıştırıldı; BS portu spesifikasyondaki test vektörlerini assert ile doğruluyor)

Not: Brewtarget'ta `StrikeWaterDialog` yalnızca `src/tools/` altında bulunuyor. `dev-doc/` içinde mash ya da hacim konusunda bir doküman yok; oradaki dosyalar veritabanı, birimler ve su konularını kapsıyor.

---

## 1. Hesap hesap karşılaştırma

| Hesap | BeerSmith | Brewtarget | Fark | Hangisi daha doğru |
|---|---|---|---|---|
| **Strike suyu sıcaklığı** | `T = Ts + [(Ts−Tt)·c_t·M_t,eff + 0.38·W·(Ts−Tg)]/Vw` (§2.1). Tun dahil. Tun kısmen doluysa kütle `V_mash/(0.8·V_t)` oranında azaltılıyor (§2.0). `EQUIP_ADJUST=0` iken tun etkisi yok sayılıyor. | Strike sıcaklığı iki yerde, iki farklı malt özgül ısısıyla hesaplanıyor. (a) **MashWizard**: Kelvin cinsinden enerji dengesi `E_su+E_tahıl+E_tun = T_mash·ΣHC`, c_tahıl = **0.4** (`MashWizard.cpp:241-319`, `PhysicalConstants.h:141`). Tun ısı kapasitesi `Recipe::mashTunHeatCapacity_kcalC` ile alınıyor (`Recipe.cpp:3417-3428`). Kısmi doluluk düzeltmesi yok. `EQUIP_ADJUST` bilinçli olarak yok sayılıyor (`Recipe.h:711`). Mash kazanda yapılıyorsa tun HC = 0 (`Equipment.cpp:781-790`). (b) **StrikeWaterDialog**: Palmer formülü `(0.41/r)(T2−T1)+T2`, tun terimi yok (`StrikeWaterDialog.cpp:32-44`). | Formüller cebirsel olarak aynı enerji dengesi; Kelvin dönüşümü sonucu değiştirmiyor. Farkı iki şey yaratıyor: c_malt (0.38 / 0.40 / 0.41) ve BS'nin 0.8 doluluk sezgisi. | Fiziksel model ikisinde de aynı. 0.38 ile 0.41 arasındaki her değer literatürle uyumlu: Palmer 0.41 kullanıyor ve tun kaybını telafi etsin diye yuvarladığını söylüyor; kuru arpada ölçülen değerler yaklaşık 0.37–0.40. BS'nin 0.8 kuralı ampirik ve yayımlanmış bir kaynağı yok. BT'nin kazanda mash için tun HC'yi 0 alması (suyla birlikte ısınan kap) daha isabetli. |
| **Ara infüzyon (hacim)** | `Va = (T2−T1)(c_t·M_eff + 0.38W + Vw)/(T_inf−T2)` (§2.3) | MashWizard su sıcaklığını her zaman kaynama noktası kabul ediyor: `Va = (0.4W + tunHC + ΣV_önceki)(T2−T1)/(Tb−T2)` (`MashWizard.cpp:433-443`, 355). StrikeWaterDialog aynı hesabı 0.41 ile ve tunsuz yapıyor (`:49-59`). | Model aynı, yalnızca c_malt farklı. BT'de kullanıcı sıcaklık seçemiyor; su hep kaynar kabul ediliyor. | Eşdeğer. |
| **Ara infüzyon (sıcaklık)** | `T_inf = T2 + (T2−T1)(c_t·M_eff + 0.38W + Vw)/Va` (§2.2); 100 °C'yi aşarsa uyarı veriyor | MashDesigner `tempFromVolume_c`: `tw = m_MC·(tf−t1)/mw + tf`. `m_MC` = tahıl + eklenmiş su + (ilk adımdan sonra) tun (`MashDesigner.cpp:281-324`, `138-146`). Sonuç kaynama noktasıyla sınırlanıyor. | c_malt farkı dışında aynı. | Eşdeğer. |
| **Dekoksiyon** | `f = (T2−T1)/(Tb−T1)·(c_t·M_eff + c_mW + Vw)/(c_mW + Vw)`, `V = (0.652·W + Vw)·min(f,1)` (§2.4) | Oran BS ile aynı formülde: `ratio = (HC_wg + HC_tun)(T2−T1)/(HC_wg(Tb−T1))` (`MashWizard.cpp:412-414`; MashDesigner `:668-691` de aynı). Hacim ise `ratio·(Vw + W/0.963)` (`MashWizard.cpp:421-422`). | Oranlar eşit. Hacimdeki fark tahılın kapladığı hacimden geliyor: **0.652 L/kg** ile **1.038 L/kg**. Test vektörlerinde BT ~%10.6 daha fazla dekoksiyon hacmi veriyor. | Literatürde tahıl yer değiştirmesi ~0.65–0.70 L/kg (≈0.08 gal/lb; Palmer, Braukaiser), bu yüzden **BS daha doğru**. BT'nin 0.963 kg/L'lik "tahıl yoğunluğu" (`PhysicalConstants.h:104-105`, yazarın kendi ölçümü) kuru tahılın görünür hacmine yakın; ıslak mash'teki yer değiştirmeyi temsil etmiyor. |
| **Kaynama noktası / rakım** | `BoilTemp = 100 − h[m]·0.00324` (§6.1). Bu yalnızca hop tarafında kullanılıyor; dekoksiyon `F_MH_BOIL_TEMP` alanını kullanıyor (varsayılan 212 °F, rakımdan türetilmiyor; §11-1). | Rakım modeli yok. `Equipment::boilingPoint_c` kullanıcı tarafından giriliyor, varsayılan 100 (`Equipment.cpp:318`, `Equipment.h:421-426`). Bu değer dekoksiyonda, infüzyon suyunda ve üst sınır kontrollerinde kullanılıyor (`MashWizard.cpp:151`). | BS rakımı biliyor ama mash hesabına aktarmıyor; BT rakımı hiç bilmiyor ama elle girilen değeri mash'te tutarlı biçimde kullanıyor. | ~3.2–3.4 °C/km yaklaşımı deniz seviyesine yakın rakımlarda yeterli. İdeal olan, rakımdan hesaplanan değeri mash hesaplarına da aktarmak. |
| **Tahıl emilimi** | 0.96 fl oz/oz = **1.0014 L/kg**; BIAB için ayrı **0.6113 L/kg** (§0.2, §1.2). Sparge tahılları için de emilim hesaplanıyor (§1.3). | Ekipman alanı `grainAbsorption_LKg`; değer yoksa varsayılan **1.086** (`Equipment.h:254`), ekipman hiç yoksa **1.085** (`PhysicalConstants.h:113`). BIAB'a özel bir değer yok. Hazır all-in-one ekipmanlarında 0.8 kullanılıyor (`data/DefaultContent005-Fermentables-Recipes.beer`, Grainfather vb.). Sparge tahılı kavramı yok; yalnızca Mash aşamasındaki Grain tipi tahıllar sayılıyor (`Recipe.cpp:823-850`). | Varsayılanlarda ~%8 fark var. BIAB'da BT ~%78 daha yüksek emilim varsayıyor. | Palmer ~1.04 L/kg (0.5 qt/lb) veriyor; sıkılmış BIAB torbasında pratik değerler ~0.5–0.8 L/kg. **BS'nin BIAB ayrımı daha gerçekçi.** BT'deki değer kullanıcıya açık ve ekipman bazlı ayarlanabiliyor; bu esneklik iyi. |
| **Tahıl hacmi (tun doluluğu)** | `V = Vsu + 0.652·W` (§1.1, §1.4) | `W/0.963 + Vsu` (`MashDesigner.cpp:222-224`, `327-329`) | +%59 (L/kg bazında) | BS (bkz. dekoksiyon satırı) |
| **Strike / ölü hacim payı** | İlk adıma `Tun addition` (geri kazanılan) ve, ayar açıksa, tun deadspace (kayıp) ekleniyor (§2.5-2, §2.6) | `strikeWaterAllowance = foundationVolume + mashTunLoss` (kazanda mash'te mashTunLoss = 0) (`Equipment.cpp:800-802`, `Recipe.cpp:3403-3406`). BT yorumu BS'nin "Mash Tun Addition" alanının foundation ile aynı şey olduğunu açıkça söylüyor (`Equipment.h:408-413`). | Kavramlar aynı. BS'de deadspace'in strike'a eklenmesi bir bayrağa bağlı; BT'de her zaman ekleniyor. | Eşdeğer. BT'de "mash kalınlığı = tahıl yatağındaki su/tahıl" tanımı daha temiz. |
| **Toplam mash/sparge suyu** | `V_sparge = V_preboil + A + A_sp + deadspace − topup_kazan − MashTotWater` (§4.1) | `targetTotalMashVol = (preboil − topUpKettle − ekstraktHacmi) + abs·W + lauterLoss` (`Recipe.cpp:3344-3401`); `sparge = hedef − totalMashWater` (`MashWizard.cpp:519`) | Aynı girdilerle **birebir aynı** sonuç (TV7: 35.007 / 19.007 L). | Eşdeğer. BT ek olarak mash'e eklenen ekstrakt/şekerin hacmini de düşüyor. |
| **Batch sparge planı** | `GetBatchSpargeSize`: tun kapasitesi `P·V_tun − V_tahıl − A` (P varsayılan %95), ilk süzmeden önce su ekleme, eşit veya doldurarak dağıtım, "önce boşalt" seçeneği, en fazla 10 batch (§4.2) | Sparge hacmi kullanıcının seçtiği N'e eşit bölünüyor: `vol = sparge/N` (`MashWizard.cpp:552-559`). Kapasite kontrolü yok, süzmeler dengelenmiyor. | Algoritmalar köklü biçimde farklı. TV9a'da BS 4.007 L ekleyip ardından 15.0 L kullanıyor (iki eşit süzme); BT 2×9.50 L veriyor (süzmeler 10.99 / 9.50 / 9.50). | **BS çok daha olgun.** Eşit süzme hacimleri verimi artırır (Braukaiser'in batch sparge analizi). |
| **Sparge suyu sıcaklığı** | Kullanıcı girdisi (varsayılan 168 °F) | Hesaplanıyor: `Tw = (0.4W + abs·W + tunHC)/V_sparge·(T_sp − (T_son − 10)) + T_sp` (`MashWizard.cpp:531-543`; MashDesigner `:304-317`) | Yalnızca BT'de var. Ancak "son adım − 10 °C" bir sezgi ve formül batch sparge'da bile **tüm** sparge hacmini kullanıyor. | Fikir yararlı ama uygulaması kaba. |
| **Boil-off** | Saatlik oran × süre veya toplam hacim (bayrakla seçiliyor) (§5.1) | Yalnızca L/saat × süre (`Equipment.cpp:818-822`). BeerXML'den gelen %/saat değeri içe aktarırken L/saat'e çevriliyor (`Equipment.cpp:456-461`). | Fark küçük. | Eşdeğer. Hacim/saat daha doğru, çünkü buharlaşma yaklaşık olarak hacimden bağımsız. |
| **Pre-boil hacmi** | `BO + (V_batch − topup_ferm + V_trub)/(1 − c)`, c = %4 (§5.2) | `batch + trub − topup_fv + t/60·evap` (`Equipment.cpp:792-798`). Soğuma büzülmesi yok. | TV6: 29.0 ve 28.0 L (−%3.45) | **BS doğru.** Su 100 °C'den 20 °C'ye soğurken hacmi ~%4.2 küçülüyor (yoğunluk 0.9584 → 0.9982). Kazanda sıcak ölçüm yapan kullanıcı BT ile ~1 L eksik pre-boil'e hedeflenir. |
| **Post-boil → fermentör** | `V_post = V_pre − BO`; soğuma kaybı `c·V_post`; ardından trub düşülüyor, top-up ekleniyor (§5.3-5.5) | `final = post − trub + topUp` (`Recipe.cpp:913-929`) | Aynı 29 L pre-boil ile BT fermentöre 24 L, BS 23 L hesaplıyor. | BS |
| **Fermentör kaybı / şişeleme** | `V_bottle = V_batch − fermenter_loss (+starter)` (§5.7) | `fvLoss_l` alanı var ama hiçbir hacim hesabında kullanılmıyor (grep sonucu: yalnızca editörde ve serileştirmede geçiyor). | Yalnızca BS'de var. | BS |
| **BIAB** | Ayrı mod: `V_BIAB = V_pre + deadspace − topup_kazan + 0.6113·W` (§3.1). Adımlara sondan başa dağıtılıyor (§3.2). | Ayrı bir mod yok. "Mash tun hacmi = 0" ise mash kazanda kabul ediliyor (`Equipment.cpp:773-775`): tun HC = 0, mashTunLoss = 0. MashWizard'daki "No sparge" seçeneği yalnızca çok adımlı mash'te son adımı ayarlayarak toplam hacme ulaşıyor (`MashWizard.cpp:452-511`). | TV8 (29 L pre-boil): BS 33.06 L; BT 1.086 ile 34.43 L (+%4.2), BT 0.8 ile 33.00 L. Strike sıcaklıkları birbirine çok yakın (68.64 / 68.67 °C). | BS'nin BIAB'a özel emilimi daha doğru. BT'nin "kazanda mash = tun ısısı yok" kuralı fiziksel olarak daha doğru. |
| **Parti-gyle** | `OG1 = 1+(OG−1)(1−0.5k)`, `k = 0.5^(3f1)` (§7) | Yok | — | Ampirik bir model; şeker puanları korunmuyor (BS §7). |
| **Seyreltme / gravite ayarı** | Araçları var (§8) | Ayrı bir araç yok. BrewLog yalnızca ölçülen pre-boil gravitesinden OG tahmin ediyor: `nokta·V_pre/(V_pre − BO)` (`BrewLog.cpp:489-499`). | — | Standart kütle dengesi; tartışılacak bir nokta yok. |
| **Pre-boil SG** | `1 + (OG−1)(V_batch+V_trub)/V_pre` (§5.10) | Şeker kütlesi ve pre-boil hacminden °P üzerinden hesaplanıyor (`Recipe.cpp:1207-1243`). Bu bölüm yoğunluk alanına giriyor. | Yöntemler farklı. | BT'nin kütle bazlı yaklaşımı fiziksel olarak daha sağlam. |

## 2. Sayısal cross-check (girdiler BS test vektörleriyle aynı; birimler °C, L, kg)

Betik: `/home/user/karsilastirma/testler/05_mash_bt_vs_bs.py`

| TV | Hesap | BeerSmith | Brewtarget | Fark % | Not |
|---|---|---|---|---|---|
| TV1 | Strike, tunsuz (5 kg, 15 L, 20 °C → 66 °C) | 71.827 | 72.133 | +0.43 | MashWizard, c = 0.40 |
| TV1 | aynı | 71.827 | 72.287 | +0.64 | StrikeWaterDialog, c = 0.41 |
| TV2 | Strike, tun 4 kg, c = 0.12, 40 L | 72.667 | 73.605 | +1.29 | BT kısmi doluluk düzeltmesi yapmıyor |
| TV2b | Strike, BS varsayılan tun (4.536 kg, 0.3) | 76.000 | 76.306 | +0.40 | |
| TV2c | Strike, mash kazanda (BIAB/AIO) | 71.827 | 72.133 | +0.43 | BT'de tun HC = 0 |
| TV3 | 66 → 72 °C, 100 °C su (L) | 3.621 | 3.643 / 3.654 | +0.59 / +0.89 | Wizard / Dialog |
| TV3b | 4 L ile 66 → 72 °C için su sıcaklığı | 97.350 | 97.500 | +0.15 | MashDesigner |
| TV4 | Dekoksiyon 66 → 72 °C (L mash) | 3.222 | 3.563 | **+10.58** | Oranlar aynı (0.1765); fark yalnızca tahıl hacminden |
| TV4 | Dekoksiyon 50 → 66 °C | 5.843 | 6.461 | **+10.58** | |
| TV5 | Kaynama noktası, 1000 m | 96.760 | — | — | BT'de elle giriliyor (varsayılan 100) |
| TV6 | Pre-boil (23 L, trub 1 L, %4, 4 L/saat) | 29.000 | 28.000 | **−3.45** | BT'de soğuma büzülmesi yok |
| TV6 | Post-boil | 25.000 | 24.000 | −4.00 | |
| TV6 | 29 L pre-boil ile fermentöre giren | 23.000 | 24.000 | +4.35 | |
| TV7 | Toplam su, aynı girdiler | 35.007 | 35.007 | **0.00** | Yapı özdeş |
| TV7 | Sparge, aynı girdiler | 19.007 | 19.007 | 0.00 | |
| TV7 | Toplam su, her program kendi varsayılanlarıyla | 35.007 | 34.430 | −1.65 | BT: pre-boil 28 L, emilim 1.086 |
| TV7 | Sparge, kendi varsayılanlarıyla | 19.007 | 18.430 | −3.04 | |
| TV7 | Tun hacim ihtiyacı (16 L + 5 kg) | 19.260 | 21.192 | +10.03 | |
| TV8 | BIAB toplam su (pre-boil 29 L) | 33.057 | 34.430 | +4.15 | BT 1.086 L/kg |
| TV8 | aynı, BT 0.8 L/kg (Grainfather profili) | 33.057 | 33.000 | −0.17 | |
| TV8 | BIAB strike sıcaklığı | 68.644 | 68.672 | +0.04 | |
| TV9a | Eşit batch sparge | 4.007 ekle + 15.000 | 9.504 × 2 | — | Algoritmalar farklı |
| TV9 | Sparge suyu sıcaklığı (hedef 75.6 °C) | girdi | 83.26 | — | BT'deki −10 °C sezgisi |
| TV13 | Boil aracı, son hacim (25 L, 2.5 L/saat) | 21.600 | 22.500 | +4.17 | BT'de soğuma yok |
| TV13 | Son SG (SG0 = 1.050) | 1.0579 | 1.0556 | −0.22 | |
| — | Emilim (L/kg) | 1.0014 | 1.086 | +8.4 | |
| — | BIAB emilimi (L/kg) | 0.6113 | 1.086 | +77.6 | |
| — | Tahıl hacmi (L/kg) | 0.652 | 1.038 | +59.3 | |
| — | Malt özgül ısısı | 0.38 | 0.40 (0.41) | +5.3 | |

Özet: ısı dengesindeki sapmalar küçük (≤%1.3; sıcaklık olarak ≤ ~1 °C) ve yalnızca c_malt ile tun doluluk sezgisinden kaynaklanıyor. Büyük sapmalar üç yerde: **soğuma büzülmesi** (yaklaşık −1 L, %3.5–4), **tahıl hacmi** (dekoksiyonda +%10.6) ve **BIAB emilimi**.

## 3. Özellik farkları

**Yalnızca BeerSmith'te olanlar:**
- Soğuma büzülmesi (%), fermentör kaybı, şişeleme hacmi
- Rakımdan kaynama noktası hesabı
- BIAB'a özel emilim ve BIAB modu
- Sparge tahıllarının emilimi
- Tun kısmi doluluk düzeltmesi, `EQUIP_ADJUST` anahtarı
- Gelişmiş batch sparge (kapasite, eşit veya doldurarak dağıtım, önce boşalt)
- Parti-gyle, seyreltme ve gravite ayarı araçları, boil aracı, "Water Needed" aracı
- Steep su/tahıl oranı (4 qt/lb), boil-off'u oran ya da toplam hacim olarak girme seçeneği
- Tahıl miktarı değiştiğinde mash adımlarını ölçekleme (§2.6)

**Yalnızca Brewtarget'ta olanlar:**
- Sparge suyu sıcaklığının hesaplanması
- Interaktif MashDesigner (tun doluluk ve toplanan wort göstergeleri)
- Çok kaplı ekipman modeli (HLT, mash tun, lauter tun ayrı; lauter kaybı ayrı alan)
- "Mash tun hacmi = 0 → mash kazanda" kuralı
- Foundation hacmini mash kalınlığından ayırma
- Mash'e eklenen ekstrakt/şeker hacmini hesaba katma
- Ekipman bazında emilim değeri; hazır all-in-one profilleri (0.8 L/kg)

## 4. Hatalar ve tuhaflıklar

**Brewtarget (koddan okundu; çalıştırılarak doğrulanmadı):**
1. **MashWizard "No sparge" + tek adım:** Strike hacmi mevcut adım değeriyle aynı kalıyor, çünkü son adımı ayarlayan kod yalnızca `steps.size() > 1` olduğunda çalışıyor. Ardından `spargeWater ≥ 0.001` olursa, kullanıcı "no sparge" seçmiş olsa bile bir **Fly Sparge adımı ekleniyor** (`MashWizard.cpp:452`, `519-587`). Koddaki yorum da bu durumdan şüpheleniyor: "should never get here on nosparge" (`:521`). Sonuç: tek adımlı BIAB mash'te wizard hacmi tamamlamıyor, onun yerine sparge ekliyor.
2. **Ekipman yoksa finalVolume 0 oluyor:** else dalında `m_finalVolume_l = boil − 4.0` atanıyor, ancak hemen ardından `calculatedFinalVolume_l` (0.0) ile karşılaştırılıp üzerine yazılıyor (`Recipe.cpp:930-962`).
3. Malt özgül ısısı aynı programın içinde tutarsız: MashWizard ve MashDesigner 0.40 kullanırken StrikeWaterDialog 0.41 kullanıyor. Emilim varsayılanı da iki yerde farklı: 1.085 (`PhysicalConstants.h:113`) ve 1.086 (`Equipment.h:254`).
4. Soğuma büzülmesi hiç yok. Hacim zinciri kendi içinde tutarlı (pre-boil = batch + trub + evap), ama kullanıcı sıcak wort'u ölçtüğünde ~%4 sapma ortaya çıkıyor.
5. Tahıl hacmi 1.038 L/kg: dekoksiyon hacmini ve tun doluluğunu ~%10 fazla gösteriyor.
6. Batch sparge eşit bölünüyor; tun kapasitesi ve ilk süzme hacmi hesaba katılmıyor. Sparge sıcaklığı formülü batch başına hacim yerine tüm sparge hacmini kullanıyor.
7. `fvLoss_l` alanı tanımlı ama hiçbir hesapta kullanılmıyor.
8. `postMashAdditionVolume_l` yalnızca **Mash** aşamasında eklenen ekstrakt/şekeri sayıyor (`Recipe.cpp:3289-3297`); fonksiyonun adıyla çelişiyor. Kaynatmaya eklenen ekstrakt hacmi pre-boil hesabına girmiyor.
9. `boilingPoint_c` ekipmanın bir alanı. Koddaki TODO bunun yanlış yer olduğunu kabul ediyor (`Equipment.h:423`); rakım bilgisi yok.

**BeerSmith'te belirsiz veya tuhaf işaretlenenlerin Brewtarget'taki karşılığı:**
- §11-1 (dekoksiyondaki kaynama noktası rakımdan gelmiyor): BT de rakım kullanmıyor, ama tek bir elle girilen değeri tutarlı biçimde her yerde kullanıyor. BS'deki sorun tutarsızlık: rakım hop hesabında var, mash'te yok.
- §11-2 (ekipman ile Water Needed aracı arasındaki soğuma tabanı farkı): BT'de soğuma hiç olmadığından bu ikilik de yok. Ekipman formülü esas alınmalı; trub kaybı sıcak hacme dahil olduğu için soğuma tabanına girmesi fiziksel olarak doğru.
- §7 (parti-gyle şeker puanlarını korumuyor): BT'de karşılığı yok. Kendi uygulamada BS modeli değil, kütle dengesine dayalı bir model önerilir.
- §2.0 (0.8 doluluk sezgisi): BT tun kütlesini her zaman tam alıyor. Tun HC büyükse (TV2) aradaki fark ~0.9 °C.
- §2.5 (dekoksiyon tipindeki ilk adım strike gibi hesaplanıyor): BT bu durumda hata veriyor ("first step must be an infusion"; `MashWizard.cpp:178`, `MashDesigner.cpp:669`).

## 5. Kendi uygulama için öneri

1. **Isı dengesi:** Tek bir enerji dengesi fonksiyonu kullanılmalı: `T_strike = Ts + [c_m·W·(Ts−Tg) + HC_tun·(Ts−Tt)]/V_w`. Ara infüzyon ve dekoksiyon da aynı terimlerden türetilmeli. Varsayılan c_malt **0.38–0.40** olmalı ve ayarlanabilir kalmalı. Tun ısı kapasitesi ekipmandan gelmeli; mash kazanda yapılıyorsa 0 alınmalı (BT kuralı). Kısmi doluluk düzeltmesi isteğe bağlı olmalı, varsayılanda kapalı.
2. **Tahıl hacmi:** **0.65–0.67 L/kg** (BS). Dekoksiyon hacmi `(V_su + 0.652·W)·f` ile hesaplanmalı ve f'ye tun terimi dahil edilmeli. İkisinde de oran formülü aynı.
3. **Emilim:** Ekipman bazında ayarlanabilmeli. Varsayılanlar: klasik mash için ~1.0 L/kg, BIAB / sıkılmış torba için ~0.6–0.8 L/kg. Sparge tahıllarının emilimi de hesaplanmalı (BS).
4. **Hacim zinciri:** BS formülü alınmalı: `V_pre = BO_saatlik·t + (V_batch − topup_ferm + trub)/(1 − c)`, c ≈ %4 ve ayarlanabilir. Fermentör kaybı ve şişeleme hacmi eklenmeli. Toplam su için BS ve BT aynı sonucu veriyor (`V_pre − topup_kazan + abs·W + kayıp`). BT'nin foundation ile kayıp ayrımı ve mash'e eklenen ekstrakt hacmini düşmesi de alınmalı.
5. **Kaynama noktası:** Rakımdan hesaplanmalı, elle değiştirilebilmeli ve mash hesaplarına (dekoksiyon, kaynar su infüzyonu) da aktarılmalı.
6. **Sparge:** BS'nin batch sparge algoritması alınmalı (kapasite, eşit süzme, önce boşalt). BT'nin sparge suyu sıcaklığı hesabı, batch başına hacimle düzeltilerek ek özellik olarak sunulabilir.
7. **BIAB:** Ayrı bir mod olmalı: tam hacim, BIAB emilimi, tun HC = 0. BT'deki tek adımlı no-sparge hatası tekrarlanmamalı.
8. **Parti-gyle:** Şeker puanlarını koruyan bir model kullanılmalı. BS'nin üstel modeli istenirse yalnızca bir seçenek olarak sunulmalı.


---

# BeerSmith 4 ve Brewtarget: Özellik, Veri Modeli ve Mimari Karşılaştırması

**Amaç:** Kendi demleme uygulamanızda hangi özellikleri ve hangi veri modelini seçeceğinize karar vermek.
**Kapsam dışı:** Lisans konusu ve kod. Bu belge yalnızca inceleme sonucudur.

**İncelenen kaynaklar**

| Program | Sürüm | İncelenen kaynak |
|---|---|---|
| BeerSmith | 4.0.22 (macOS paketi) | `Resources.sqlite` (Menu/Option/Field/String), `BeerSmith.sqlite` (62 tablo), `DefRecipe.sqlite`, rapor ve BeerXML şablonları, `/home/user/beersmith4-ozellik-analizi.md`, `/home/user/formuller/*.md` |
| Brewtarget | 5.2.0 (+5.3.0 taslak), C++/Qt6 | `src/model`, `src/database/ObjectStoreTyped.cpp`, `DatabaseSchemaHelper.cpp` (şema sürümü 21), `src/serialization`, `src/undoRedo`, `src/unitTests`, `ui/*.ui`, `schemas/`, `translations/`, `CHANGES.markdown`, `dev-doc/` |

**Kanıt düzeyi:** BeerSmith'in çalıştırılabilir dosyası incelenmedi. Bu yüzden BeerSmith tarafındaki bilgiler menü, alan ve ayar tanımlarından ve veri şemasından çıkarıldı. Bir menü öğesinin var olması, özelliğin var olduğunu gösterir ama nasıl çalıştığını göstermez. Brewtarget tarafındaki bilgiler doğrudan kaynak koddan alındı.

**Simgeler:** ✅ var · ◐ kısmen var · ❌ yok · 🔜 Brewtarget'ın bir sonraki sürümü (5.3.0) için duyurulmuş

---

## 1. Özellik matrisi

### 1.1 İçecek türleri

| Özellik | BeerSmith | Brewtarget | Not |
|---|---|---|---|
| Bira: all-grain, extract, partial mash | ✅ | ✅ (`Recipe::Type`) | |
| Mead | ✅ Besin, sülfit ve tatlandırma araçları, ayrı brewsheet | ◐ Yalnızca tür olarak (`Mead`) | Brewtarget'ta mead'e özel hesap yok. |
| Şarap | ✅ | ◐ Tür olarak (`Wine`) | |
| Cider | ✅ Ayrı brewsheet (`CiderBrewSheet.htm`) | ◐ Tür olarak (`Cider`) | |
| Seltzer | ✅ (4.x sürüm notları) | ❌ (`Soda` / `Other` var) | |
| Kombucha, soda | ❌ | ◐ Tür olarak | Brewtarget bu türleri BeerJSON uyumu için ekledi. |
| Bal, meyve ve meyve suyu fermentable türleri | ✅ Brix/SG girişiyle | ✅ `Fruit`, `Juice`, `Honey` | |
| Maya dışı kültürler (Brett, Lacto, Kveik, bakteri) | ◐ Maya türü alanı üzerinden | ✅ `Yeast::Type` içinde | |

### 1.2 Tarif editörü ve tersine ayar araçları

| Özellik | BeerSmith | Brewtarget | Not |
|---|---|---|---|
| OG, FG, ABV, IBU, renk ve stile uygunluk göstergesi | ✅ Alan yanında gösterge topları | ✅ `StyleRangeWidget`, `IbuGuSlider` | |
| IBU formülleri | Tinseth, Rager, Garetz | Tinseth, Rager, Noonan | 5.3.0 için ek IBU yöntemleri duyuruldu. |
| Whirlpool sıcaklığına bağlı hop kullanımı | ✅ | ❌ | |
| Mash hop ve first wort düzeltmesi | ✅ | ✅ (Seçenekler → IBU Adjustments) | |
| Renk formülleri | `formuller/01` | Mosher, Daniels, Morey | |
| Hedef OG'ye göre ayarlama (Adjust Gravity) | ✅ Kaydırıcıyla | ❌ | |
| Hedef renge göre ayarlama (Adjust Color) | ✅ | ❌ | |
| Hedef IBU'ya göre ayarlama (Adjust Bitterness, hop başına IBU) | ✅ | ❌ | |
| Tahılı yüzdeyle girme (Grain Pct) | ✅ | ◐ Yüzde sütunu düzenlenebilir (5.0.4 düzeltmesi) | |
| All-grain ↔ extract dönüşümü (Convert Recipe) | ✅ | ❌ | |
| Ölçekleme (Scale) | ✅ Boyut, verim veya ekipmana göre, sonrasında yeniden dengeleme | ✅ Ekipman ve verime göre, sihirbazla | |
| Malzeme ikamesi (Substitute), miktarları yuvarlama | ✅ | ❌ | |
| Check Recipe (eksiklik ve tutarlılık denetimi) | ✅ | ❌ | |
| Tarif kilitleme | ✅ `F_R_LOCKED` | ✅ `locked` | |
| Özel alanlar (custom fields) | ✅ `M_CUSTOM_FIELDS` | ❌ | |
| Tarif görseli | ✅ | ❌ | |
| Tadım puanı ve tadım notu | ✅ | ✅ `tasteRating`, `tasteNotes` | |
| Kalori | ✅ Tahmini ve ölçülen | ✅ Üç porsiyon birimiyle | |
| Fermentasyonda eklenen şekerleri sayan sanal OG ("Est OG Through Tertiary") | ✅ | ◐ Eklemelerin `stage` alanı var; böyle bir "sanal OG" alanı yok | |
| Maya starter, canlılık ve pitch rate | ✅ Tarife bağlı: 2 kademe, stir plate, hücre/ml-°P | ◐ Ayrı pitch aracı: üretim tarihine göre canlılık, havalandırma | |
| Mash sıcaklığına göre FG düzeltmesi | ✅ | ❌ | |

### 1.3 Malzeme ve profil türleri

| Tür | BeerSmith tablosu | Brewtarget sınıfı | Not |
|---|---|---|---|
| Fermentable | `M_GRAIN` | `Fermentable` | |
| Hop | `M_HOPS` | `Hop` | Brewtarget'ta yağ bileşenleri ve alfa/beta min–max aralığı da var. |
| Maya | `M_YEAST` | `Yeast` | |
| Misc | `M_MISC` | `Misc` | Brewtarget 5.2'de tuzlar Misc'in alt kategorisi oldu (`WaterAgent`). |
| Su | `M_WATER` | `Water` | |
| Stil | `M_STYLE` (BJCP 2015 dahil) | `Style` (varsayılan içerik BJCP 2021) | |
| Ekipman | `M_EQUIPMENT` (34 alan) | `Equipment` (yaklaşık 53 alan) | Brewtarget ekipmanı kap kap modelliyor (BeerJSON'daki gibi HLT, mash tun, lauter, kettle, fermenter, packaging). |
| Mash profili | `M_MASH` (adımlar JSON içinde) | `Mash` + `MashStep` | |
| Kaynatma profili | ❌ (ekipman ve tarif alanları) | ✅ `Boil` + `BoilStep` | |
| Fermentasyon profili | `M_AGE` (sabit: primary, secondary, tertiary, bulk, aging) | `Fermentation` + `FermentationStep` (sınırsız adım) | |
| Karbonasyon profili | ✅ `M_CARB` | ❌ (alanlar tarifin içinde, ayrıca Priming aracı) | |

### 1.4 Demleme günü, session ve brew log

| Özellik | BeerSmith | Brewtarget | Not |
|---|---|---|---|
| Adım adım demleme talimatı | ✅ Şablonla üretiliyor (`Brewsheet.htm` vb.), özel adım eklenebiliyor | ✅ `Instruction` varlığı, `BrewDayScrollWidget`, `BrewDayFormatter` | |
| Ölçülen değerler (pre-boil, OG, FG, hacimler) | ✅ Tarifteki `F_R_*_MEASURED` ve `*_SET` sütunları | ✅ Ayrı `BrewLog` kaydında beklenen/ölçülen çiftleri | Bu fark mimari açıdan önemli (bkz. §2). |
| Verim, ABV ve attenuation hesabı | ✅ | ✅ `computed_*` alanları | |
| Birden çok demleme kaydı | ◐ "New Session" tarifi Brew Log klasörüne kopyalıyor | ✅ Bir tarife bağlı N adet `BrewLog` | |
| Batch numarası | ◐ Sürüm numarası artırılıyor | ✅ (5.1.0) | |
| Ölçüm alanında açılır refraktometre | ✅ | ❌ (ayrı araç) | |

### 1.5 Zamanlayıcılar

| Özellik | BeerSmith | Brewtarget | Not |
|---|---|---|---|
| Mash/steep, kaynatma ve whirlpool zamanlayıcıları | ✅ Üç ayrı sekme; ayarla, duraklat, sıfırla | ✅ Çoklu zamanlayıcı (`TimerMainDialog`, `TimerListDialog`, `BoilTimer`) | |
| Ekleme zamanında alarm | ✅ Özel alarm sesi | ✅ Sesli klipler ("aroma hops", "check boil" vb.) | |
| Zamanlayıcı durumunun tarifte saklanması | ✅ `F_R_MASH_TIMER`, `F_R_BOIL_TIMER` | ❌ | |

### 1.6 Fermentasyon takibi ve cihazlar

| Özellik | BeerSmith | Brewtarget | Not |
|---|---|---|---|
| Fermentasyon okumaları (tarih, SG, sıcaklık, pH) | ✅ `AgeData` (tarifin içinde JSON) | ❌ | |
| Okuma grafiği | ✅ | ❌ | |
| CSV içe/dışa aktarma | ✅ | ❌ | |
| Tilt | ✅ Google Sheets bağlantısı ve CSV (`F_R_TILT_LINK`, renk, filtre süresi) | ❌ | |
| iSpindel, RAPT, Plaato | ❌ | ❌ | Rakiplerin ikisinde de olmayan bir boşluk. |

### 1.7 Envanter, alışveriş ve maliyet

| Özellik | BeerSmith | Brewtarget | Not |
|---|---|---|---|
| Stok miktarı | ✅ Malzemedeki `F_*_INVENTORY` alanı ve ayrı `M_INVENTORY` | ✅ Lot bazlı: `StockPurchase*` + `StockUse*` | Brewtarget her alışı ayrı lot olarak tutuyor (tarih, tedarikçi, son kullanma, fiyat; hop için alfa, form ve yıl). |
| Tariften stoktan düşme | ✅ Çift düşmeye karşı uyarıyla | ✅ `StockUse` (sebep, tarih ve `brew_log_id` tutuluyor) | |
| "Yalnızca stokta olanları göster" | ✅ | ◐ Envanter sütunu, sıralama ve anlık görüntü | |
| Alışveriş listesi | ✅ `M_SHOP`, sipariş eşiği | 🔜 5.3.0 | |
| Fiyat ve tarif maliyeti | ✅ Birim fiyat, toplam maliyet, para birimi ayarları | ◐ Alış fiyatı var (`CurrencyAmount`), tarif maliyeti yok | |

### 1.8 Takvim ve notlar

| Özellik | BeerSmith | Brewtarget |
|---|---|---|
| Demleme takvimi | ✅ Klasöre veya demleyene göre filtrelenebiliyor | ❌ |
| Not ve görev listesi (öncelik, başlangıç/bitiş tarihi) | ✅ `M_NOTES` | ❌ (yalnızca tarif ve brew log notları) |

### 1.9 Hesap araçları

| Araç | BeerSmith | Brewtarget |
|---|---|---|
| Alcohol & Attenuation (kalori ve karbonhidrat dahil) | ✅ | ◐ Alcohol Percentage Tool (kalori yok) |
| Hop Bitterness (bağımsız IBU hesabı) | ✅ | ❌ |
| Hop Age (HSI ile alfa kaybı) | ✅ | ❌ (HSI alanı var, araç yok) |
| Hop Rate (g/l, oz/gal) | ✅ | ❌ |
| Boil Off | ✅ | ❌ |
| Carbonation / priming | ✅ | ✅ Priming Calculator (4 şeker türü) |
| Decoction Volume | ✅ | ◐ Mash Designer içinde |
| Dilution | ✅ | ❌ |
| Adjust Gravity | ✅ | ✅ Gravity (OG) Correction Tool |
| Hydrometer Adjust | ✅ | ✅ |
| Infusion / Strike | ✅ | ✅ Strike Water Calculator |
| Mash Adjust | ✅ | ◐ Mash Wizard / Mash Designer |
| Mash pH ve asit | ✅ Tahminden veya ölçümden | ◐ Water Adjuster (Kai Troester modeli, tuzlar, RO yüzdesi) |
| Water Profile (en uygun tuz eşleştirmesi) | ✅ | ◐ Elle tuz ekleme, hedef profil karşılaştırması |
| Water Needed | ✅ | ◐ Mash Wizard'ın toplam su hesabı |
| Mead Nutrients, Sulfite, Backsweeten | ✅ | ❌ |
| Parti-Gyle | ✅ | ❌ |
| Refractometer | ✅ Kalibrasyonlu | ✅ (OG/FG, ABV, ABW, gerçek ekstrakt) |
| Weight-to-Volume (fıçı boyutları) | ✅ | ❌ |
| Yeast Starter | ✅ | ◐ Pitch Rate Calculator |
| Birim dönüştürücüler | ✅ 7 ayrı araç | ✅ Tek araç (Unit Conversion) |
| Mash Wizard: no/fly/batch sparge | ◐ Hazır mash profilleriyle | ✅ |
| Equipment Wizard | ✅ | ❌ |

Not: BeerSmith her aracın son girdilerini kendi tablosunda saklıyor (29 adet `M_TOOL_*` tablosu). Brewtarget'ın araçları ise geçici; girdileri kaydetmiyor.

### 1.10 İçe ve dışa aktarma

| Format | BeerSmith | Brewtarget | Not |
|---|---|---|---|
| Kendi formatı | `.bsmx` (XML benzeri; içe ve dışa) | Kendi dosya formatı yok; DotBeer'i destekliyor | |
| BeerXML 1.0 | ✅ Şablonla dışa, içe aktarma | ✅ Kodla tanımlı eşleme (XSD doğrulamalı), uzantı etiketleri | Ayrıntılar §4'te. |
| BeerJSON 1.0 | ❌ Kaynaklarda hiç geçmiyor | ✅ İçe ve dışa | |
| DotBeer 1.0.0 (MPL 2.0 lisanslı açık şema) | ❌ | ✅ (5.2.0) | Referansla çalışan JSON formatı. |
| CSV | ✅ Rapor, fermentasyon verisi, Tilt | ❌ | |
| BBCode ve düz metin (panoya) | ❌ | ✅ | |
| Eski sürümlerden içe aktarma | ✅ BeerSmith 1.4, 2 ve 3 | ✅ Eski Brewtarget veritabanını yükseltme | |
| Belirli bir klasöre içe aktarma | ❌ | ✅ (5.2.0) | |

### 1.11 Raporlar ve yazdırma

| Özellik | BeerSmith | Brewtarget |
|---|---|---|
| Rapor üretimi | HTML ve metin şablonları, `$DEĞİŞKEN` yer tutucuları (213 ayrı değişken; brewsheet'te 46) | C++ ile HTML üretimi (`RecipeFormatter`, `BrewDayFormatter`, `StockFormatter`) |
| Kullanıcının rapor şablonu eklemesi | ✅ İçe ve dışa aktarma | ❌ |
| İçeceğe özel brewsheet | ✅ Bira, extract, mead, cider; yarışma, alışveriş | ❌ |
| Önizleme ve yazdırma | ✅ Sekme ve liste yazdırma | ✅ `PrintAndPreviewDialog` |
| Açık ve koyu CSS | ✅ `style.css`, `styledark.css` | ◐ Qt teması |

### 1.12 Bulut, senkron ve paylaşım

| Özellik | BeerSmith | Brewtarget |
|---|---|---|
| Bulut ve yerel depolama modu | ✅ İlk açılışta seçiliyor | ❌ |
| Senkron alanları | ✅ `_CLOUDID_`, `_CLOUD_STATE_`, `_MOD_` ve her türün `M_CLOUD_*` ikiz tablosu | ❌ |
| Çok cihazdan ortak kullanım | ✅ Bulut senkronuyla | ◐ Paylaşılan PostgreSQL sunucusu |
| Tarif paylaşma (özel/genel/listelenmemiş), puanlama, yorum | ✅ | ❌ |
| Bulut arama (sıralama, sayfalama) | ✅ | ❌ |
| Güncelleme denetimi | Lisans etkinleştirme | ✅ GitHub sürüm denetimi |

### 1.13 Yedekleme, geri alma, arşiv ve sürüm geçmişi

| Özellik | BeerSmith | Brewtarget |
|---|---|---|
| Geri al / yinele | ✅ Ayrıca "Undo Last" ile son malzeme değişikliği | ✅ `QUndoStack` tabanlı (`undoRedo/`) |
| Otomatik yedek ve geri yükleme | ✅ | ◐ Elle yedekleme ve geri yükleme (veritabanı dosyası) |
| ZIP ile tam yedek | ✅ Tablo seçerek geri yükleme | ❌ |
| Tarif arşivi | ✅ Ekleme, düzenleme, silme ve taşıma ayrı ayrı açılabiliyor; saklama süresi ve sayı sınırı; ayrı veritabanı | ❌ |
| Sürüm geçmişi | ◐ `F_R_VERSION` sayısı, Brew Log kopyası, "bump version" | ✅ Otomatik snapshot (`ancestor_id`), Ancestors diyaloğu |
| Silme | Kalıcı (arşive kopya alınabiliyor) | Yumuşak silme (`deleted` bayrağı) |

### 1.14 Birimler, dil ve platform

| Özellik | BeerSmith | Brewtarget |
|---|---|---|
| Birim sistemleri | Metric, US (English), Imperial hazır ayarları; her nicelik için ayrı birim ve hassasiyet (`Units.*`) | Imperial, US Customary, Metric, MetricAlternate; SRM/EBC/Lovibond; SG/Plato/Brix; alan bazında birim değiştirme |
| İç birim | İngiliz birimleri (fl oz, °F, oz) | SI (L, kg, °C) |
| Para birimi | ✅ Sembol, konum ve hassasiyet | ✅ `CurrencyAmount` |
| Dil desteği | Değiştirilebilir dil SQLite dosyası (paketle yalnızca İngilizce geliyor) | 29 Qt `.ts` dosyası (Türkçe dahil), çalışırken dil değiştirme |
| Platformlar | Windows, macOS (Intel + Apple Silicon), Linux (wxWidgets) | Windows, macOS, Linux (deb, rpm, AppImage, Flatpak) |
| Mobil ve web | Bu pakette yok | Yok |

### 1.15 Eklentiler ve arayüz kolaylıkları

| Özellik | BeerSmith | Brewtarget |
|---|---|---|
| Add-on (veri paketleri: hop, maya, tahıl, misc, su, stil, ekipman, mash, karbonasyon, fermentasyon) | ✅ | ❌ (varsayılan içerik veritabanına yükleniyor) |
| Kod eklentisi | ❌ | ❌ |
| Sekmeli ve pencereli görünüm, önizleme paneli | ✅ | ✅ Ağaç, sekmeler, katalog pencereleri |
| Favoriler ve yer imleri çubuğu | ✅ | ❌ |
| Gelişmiş arama (malzemeye göre) | ✅ | ◐ Ağaç filtresi |
| Özelleştirilebilir sütunlar | ✅ | ✅ Sütun gizleme |
| Klasörler | ✅ Tarifler için | ✅ Her tür için birinci sınıf varlık (5.1.0), sürükle-bırak |
| Aynı kayıtları birleştirme | ❌ | ✅ (4.2.0) |
| Bir profilin kaç tarifte kullanıldığı | ❌ | ✅ `numRecipesUsedIn` |
| Karşılama ekranı ve başlangıç rehberleri | ✅ | ❌ (PDF kılavuz var) |

---

## 2. Veri modeli karşılaştırması

### 2.1 Genel yaklaşım

| Konu | BeerSmith | Brewtarget |
|---|---|---|
| Model tipi | Belge yönelimli. Tarif tek bir geniş satır; alt nesneler JSON metni olarak gömülü. | İlişkisel ve normalize. Her alt nesne ayrı tablo; yabancı anahtarlarla bağlı. |
| Tablo sayısı | 62. Bunların 11'i ana varlık, 10'u bulut ikizi; ayrıca klasör, alışveriş, envanter ve tarif için bulut tabloları, 29 araç tablosu, arama ayarları ve not tablosu. | Şema v21. 11 `folder_for_*` tablosu, 13 varlık ve adım tablosu, 4 `*_in_recipe` tablosu, `brew_log`, `instruction`, 4 `*_stock_purchase`, 4 `*_stock_use` tablosu ve ayarlar. |
| Birincil anahtar | `_PERMID_` (INTEGER) | `id` (INTEGER) |
| Ortak alanlar | `_PERMID_`, `_MOD_` (epoch metni), `_CLOUDID_`, `_CLOUD_STATE_`, `_EXTRA_` | `id`, `name`, `deleted` |
| Yabancı anahtar ve index | Yok | Var |
| Sayıların saklanması | SQL sütunlarında doğal tip; JSON içinde 7 ondalıklı metin (`"640.0000000"`) | SI birimiyle `REAL` |
| Enum'lar | Tam sayı kodu (`F_H_USE = 0..`) | Metin eşlemesi (`EnumStringMapping`); BeerJSON enum değerleriyle aynı |

### 2.2 Alan sayıları (temsilî)

Brewtarget sayıları `ObjectStoreTyped.cpp`'den otomatik sayıldı ve ortak alanları (`id`, `name`, `deleted`) içeriyor. ±2 sapma olabilir.

| Varlık | BeerSmith | Brewtarget |
|---|---|---|
| Tarif | 112 (`M_RECIPE`; 7 gömülü JSON sütunu dahil) | ≈36 (`recipe`) + 33 (`brew_log`) + eklemeler |
| Hop | 26 | ≈33 (+ `hop_in_recipe` ≈13) |
| Fermentable | 33 | ≈38 (+ `fermentable_in_recipe` ≈13) |
| Maya | 38 | ≈24 (+ `yeast_in_recipe` ≈16) |
| Misc | 22 | ≈11 (+ `misc_in_recipe` ≈13) |
| Su | 26 | ≈20 |
| Stil | 29 | ≈29 |
| Ekipman | 34 | ≈53 |
| Mash ve adım | 26 + adım JSON'u (yaklaşık 18 alan) | 12 + 16 |
| Fermentasyon | 24 (`M_AGE`, sabit 5 aşama) | 6 + adım başına 15 |
| Alan tanımı metaverisi | 1593 `Field` satırı (etiket, açıklama, birim, ipucu) ve 219 `Option` satırı | 532 `Q_PROPERTY` (model sınıflarında) ve `TypeLookup` |

### 2.3 Tarif ile malzeme arasındaki bağ

**BeerSmith malzemeyi tarifin içine kopyalıyor.**

- `M_RECIPE.Ingredients` sütunu bir JSON dizisi tutuyor. Dizideki her öğe, kütüphane satırının tam kopyası. `_Schema_` kodu türü belirtiyor: 7406 tahıl, 7403 hop, 7421 misc, 7426 maya, 7423 su.
- Kopyada kütüphane kaydının ID'si yok. Bağ yalnızca isim üzerinden. "Update Prices" işleminin hata mesajı (`S_ERR_PRICE_MATCH`) da eşleştirmenin isimle yapıldığını gösteriyor.
- Ekipman, stil, mash (adımları dahil), karbonasyon, fermentasyon profili ve baz tahıl da tek tek JSON kopyası olarak tutuluyor (`F_R_EQUIPMENT`, `F_R_STYLE`, `F_R_MASH`…).
- Kütüphane tablosu ile tarifteki kopya aynı şemayı kullanıyor. Örneğin `M_HOPS` içinde `F_H_AMOUNT`, `F_H_BOIL_TIME`, `F_H_IN_RECIPE` ve `F_ORDER` gibi tarife özgü alanlar da var.

**Brewtarget malzemeye referans veriyor.**

- Bağ `RecipeAdditionHop`, `RecipeAdditionFermentable`, `RecipeAdditionMisc` ve `RecipeAdditionYeast` ara tablolarıyla kuruluyor (`hop_in_recipe` vb.).
- Bu tablolar `recipe_id` ve `hop_id` anahtarlarını, miktarı (`quantity`, `unit`) ve zamanlamayı tutuyor: `stage` (mash, boil, fermentation, packaging), `step`, `add_at_time_mins`, `add_at_gravity_sg`, `add_at_acidity_ph`, `duration_mins`.
- Malzeme verisi kopyalanmıyor. Kaynak koddaki yorum bunun bilinçli bir karar olduğunu söylüyor: BeerJSON'un `HopAdditionType` tipinin aksine, `Hop` sınıfındaki bilgi tekrarlanmıyor.
- Mash, kaynatma, fermentasyon, stil, ekipman ve su profilleri de paylaşılan kayıtlara yabancı anahtarla bağlı (`mash_id`, `boil_id`, `water_base`, `water_target` vb.).
- Su, bir "ekleme" olarak modellenmemiş. Tarifte bir baz su ve bir hedef su tutuluyor; mash ve sparge için RO yüzdesi de var.

**İki yaklaşımın sonuçları**

| | Kopya (BeerSmith) | Referans (Brewtarget) |
|---|---|---|
| Tarifin tarihsel doğruluğu | ✅ Kütüphanede alfa değeri değişse de eski tarif değişmez | ❌ Kütüphanedeki hop düzenlenince eski tarifler ve snapshot'lar da yeniden hesaplanır |
| Tarifin taşınabilirliği | ✅ Tarif kendi kendine yeterli; senkron ve dışa aktarma kolay | ◐ Dışa aktarırken bağlı kayıtlar toplanmalı (DotBeer referansları dosya içi ID'yle çözüyor) |
| Veri tekrarı | ❌ Aynı hop onlarca kez kopyalanıyor | ✅ Normalize |
| Sorgulama ("bu hop hangi tariflerde var?") | ❌ JSON taramak gerekiyor; bağ isimle | ✅ Basit bir join; `numRecipesUsedIn` |
| Envantere bağlama | ◐ İsim eşleştirmesiyle | ✅ Yabancı anahtar ve lot |
| Şema değişikliği | `_Schema_` sürüm koduyla esnek | Göç kodu gerekiyor (`DatabaseSchemaHelper`, v1'den v21'e) |

### 2.4 Tarif içindeki adımlar

| | BeerSmith | Brewtarget |
|---|---|---|
| Mash adımları | `M_MASH.steps` içinde JSON dizisi (`F_MS_TYPE`, sıcaklık, süre, rampa, infüzyon/dekoksiyon miktarı) | `mash_step` tablosu; sıralı, `mash_id` ile bağlı |
| Kaynatma adımları | Yok (kaynatma süresi ekipman ve tarifte) | `boil_step` tablosu (ön ısıtma, kaynatma, soğutma) |
| Fermentasyon | `M_AGE` içinde sabit sütunlar (`PRIM`, `SEC`, `TERT`, `BULK`, `AGE` için sıcaklık ve gün) | `fermentation_step` tablosu, sınırsız adım |
| Malzemenin hangi adıma eklendiği | Kullanım türüne göre (`F_H_USE`, `F_H_DRY_START`, `F_H_DRY_PHASE`) | `stage` + `step` + zaman / gravite / pH koşulu |
| Talimatlar | Rapor şablonunda üretiliyor (kalıcı değil) | `instruction` tablosu (kalıcı, tarife bağlı) |

### 2.5 Demleme kayıtları ve ölçümler

- **BeerSmith:** Ölçümler tarif satırındaki sütunlarda tutuluyor. Her ölçüm bir değer ve bir "girildi mi" bayrağından oluşuyor (`F_R_OG_MEASURED` + `F_R_OG_MEASURED_SET`). Birden fazla demleme kaydı için tarifin tamamı Brew Log klasörüne kopyalanıyor. Fermentasyon okumaları da tarifin içinde (`AgeData` JSON'u: `Date`, `F_AD_TEMP`, `F_AD_PH`, gravite ve kontrol bayrakları).
- **Brewtarget:** `brew_log` tablosu `recipe_id` ile tarife bağlı ve her ölçüm için beklenen/ölçülen/hesaplanan üçlüsünü tutuyor (`expected_original_gravity_sg`, `measured_original_gravity_sg`, `computed_efficiency_pct` vb.). Fermentasyon okumaları için bir tablo yok.

### 2.6 Profiller

Profillerin ikisi de kütüphanede yaşıyor. Fark tarife bağlanma biçiminde: BeerSmith seçilen profili tarife kopyalıyor, Brewtarget referans veriyor. Bunun bir sonucu: Brewtarget'ta bir mash profilini düzenlemek, onu kullanan bütün tarifleri etkiliyor. Bu yüzden editör kaç tarifin etkileneceğini gösteriyor (4.1.0, #948).

### 2.7 Envanter

- **BeerSmith:** İki yerde tutuluyor.
  1. Her malzeme satırında bir sayı alanı (`F_H_INVENTORY` vb.).
  2. Ayrı `M_INVENTORY` ve `M_SHOP` tabloları (`F_SH_NAME`, `F_SH_AMOUNT`, `F_SH_TYPE`, `F_SH_ING_ID`, fiyat ve sipariş eşiği). İki tablonun şeması aynı.

  Sürüm notlarına göre 4.x'te envanter malzeme tablolarından ayrıldı.
- **Brewtarget:** Bir hareket defteri gibi çalışıyor. `*_stock_purchase` her alışı tutuyor: miktar, sipariş, teslim ve son kullanma tarihleri, tedarikçi, fiyat ve hop'a özgü alfa, form ve yıl. `*_stock_use` her kullanımı tutuyor: sıra, tarih, sebep, miktar ve `brew_log_id`. Kalan stok bu hareketlerden hesaplanıyor. Lot bazlı FIFO, fiyat geçmişi ve demlemeyle izlenebilirlik bu sayede mümkün.

### 2.8 Klasörler

- **BeerSmith:** `M_FOLDER` (`F_F_NAME`, `F_F_PARENT`, `F_F_PRIVATE`). Tarif hem `F_R_PARENT` (klasör ID'si) hem de `F_R_FOLDER_NAME` (metin yolu, örneğin `/BeerSmith 3 Samples/`) tutuyor; yani bilgi tekrarlanıyor. Klasör yalnızca tarifler için var.
- **Brewtarget:** Her varlık türü için ayrı bir `folder_for_<tür>` tablosu var (`contained_in_folder_id` ile iç içe). Varlıkta da `contained_in_folder_id` sütunu var. Tabloların ayrı olması bilinçli: hop'un fermentable klasörüne konmasını şema düzeyinde engelliyor. BeerXML'e `FOLDER_PATH` uzantı etiketiyle yazılıyor.

### 2.9 Sürümleme

> **Düzeltme:** BeerSmith'teki `F_R_PARENT` alanı sürümle ilgili değil. `Field` tablosundaki tanımı "Parent folder id used for organizing recipes"; yani bir **klasör** referansı. Örnek veride bütün tariflerde değeri 1 ve bu, `M_FOLDER` tablosundaki tek klasörün ID'si.

**BeerSmith'in sürümleme mekanizmaları:**

1. `F_R_VERSION`: Elle girilen bir sayı. İsteğe bağlı olarak sürüm değişince "farklı isimle kaydet" önerisi çıkıyor (`FO_CONFIRM_VERSION_CHANGE`).
2. New Session diyaloğu: tarifin bir kopyasını Brew Log'a (yerel ya da bulut) alıyor ve sürümü artırıyor (`F_R_ARCHIVE`, `F_R_BUMP_VERSION`).
3. Recipe Archive: ekleme, düzenleme, silme ve taşıma işlemlerinin her biri için ayrı açılıp kapatılabiliyor. Ay ve adet sınırı var, arşivden kurtarma yapılabiliyor ve arşiv ayrı bir veritabanında tutuluyor.

Kopyalar arasında bir soy bağı (parent ID) tutulmuyor.

**Brewtarget'ın sürümleme mekanizması:**

- `recipe.ancestor_id` aynı tabloya işaret eden bir yabancı anahtar; kökte kaydın kendi ID'si.
- Tarif demlendikten sonra yapılan bir değişiklik otomatik snapshot alıyor (copy-on-write). Eski sürüm kilitleniyor ve ağaçta "subsidiary" olarak gizleniyor.
- Dal (branch) yok; doğrusal bir zincir. Elle fork alma, kilitleme ve ata atama (`AncestorDialog`) mümkün.
- Seçeneklerden kapatılabiliyor. Tarif silinince snapshot'ların da silinip silinmeyeceği seçilebiliyor.
- Zayıf nokta: snapshot, malzemeleri referansla tuttuğu için geçmiş tam olarak dondurulmuyor (bkz. §2.3).

### 2.10 Bulut alanları (yalnızca BeerSmith)

| Alan | Anlamı (alan tanımından ve kullanımdan) |
|---|---|
| `_PERMID_` | Yerel kalıcı ID (INTEGER PK) |
| `_CLOUDID_` | Sunucudaki karşılığın ID'si (0 = senkronize edilmemiş) |
| `_MOD_` | Son değişiklik zamanı (epoch, metin) |
| `_CLOUD_STATE_` | Senkron durumu |
| `_EXTRA_` | Ayrılmış alan |
| `M_CLOUD_*` | Buluttan inen kayıtların ayrı ikiz tabloları (aynı şema) |
| `F_C_PRIVATE`, `F_C_UNLISTED`, `F_C_RATING`, `F_C_COMMENTCOUNT` | Paylaşım ve topluluk alanları |
| `F_R_JSON` | "JSON version of the recipe – used for cloud sync". Sunucuyla alışveriş JSON ile, ama özel bir şemayla. |

Brewtarget'ta bunların hiçbirinin karşılığı yok. Kayıtların genel bir kimliği (UUID) ya da değişiklik zamanı tutulmuyor.

### 2.11 Güçlü ve zayıf yanlar

| | BeerSmith | Brewtarget |
|---|---|---|
| **Güçlü** | Tarif kendi kendine yeterli ve tarihsel olarak donmuş. Senkron alanları baştan tasarlanmış. Arşiv ayrıntılı. Araç girdileri kalıcı. Özel alanlar var. Fermentasyon okumaları var. | Normalize ve tutarlı. Eklemelerin zamanlaması BeerJSON'la uyumlu (stage, step, gravite, pH). Lot bazlı envanter. Profiller adım adım ve sınırsız. Soy zinciri var. Her tür için klasör. Yumuşak silme. Hem SQLite hem PostgreSQL. |
| **Zayıf** | 112 sütunlu "tanrı tablosu". JSON içinde sayılar metin. Kopyalar isimle bağlı. Yabancı anahtar ve index yok. Ölçümler tarifte; birden fazla demleme kaydı için tarifin tamamı kopyalanıyor. Fermentasyon 5 sabit aşama. İç birim İngiliz. `F_R_FOLDER_NAME` ile `F_R_PARENT` tekrar ediyor. | Kütüphanede yapılan bir düzenleme geçmiş tarifleri ve snapshot'ları değiştiriyor. Senkron ve UUID yok. Fermentasyon okuması yok. Maliyet modeli zayıf. Çok tablolu ve şablon ağırlıklı C++ kodu, değişiklik maliyetini artırıyor (21 şema göçü). |

---

## 3. Mimari

| Konu | BeerSmith | Brewtarget |
|---|---|---|
| Dil ve arayüz | C++ ve wxWidgets (Windows paketindeki lisans dosyasından) | C++20 ve Qt6 (Widgets, `.ui` dosyaları) |
| Depolama | Yerel SQLite (`BeerSmith.sqlite`), varsayılan tarif için `DefRecipe.sqlite`, ayrı arşiv veritabanı ve bulut sunucusu. Sürüm notlarına göre her kayıt anında commit ediliyor. | SQLite veya PostgreSQL (`QSQLITE` / `QPSQL`; seçeneklerde motor seçimi ve bağlantı testi). ObjectStore önbelleği; okumalar önbellekten, yazmalar anında. |
| Şema tanımı | Tablo başına sütunlar ve gömülü JSON'da `_Schema_` kodları | Kodda `TableDefinition` / `TableField` ile tanımlı; göçler `DatabaseSchemaHelper` içinde (v21) |
| Alan tanımları | **Veri olarak** tutuluyor: `Resources.sqlite.Field` her alan için etiket, açıklama, `Column_Name`, birim ve ipucu içeriyor; `Option` her ayar için varsayılan değer ve tipi; `Menu` her komut için metin, ikon, yardım ve ipucunu; `String` ise mesajları. Arayüz bu tablolardan üretiliyor. | **Kod olarak** tutuluyor: Qt property sistemi (`Q_PROPERTY`, `PropertyNames::*` sabitleri, `TypeLookup` ile birim ve nicelik tipi). Editörler `EditorBaseField` ile property'lere bağlanıyor. Etiketler `tr()` ile `localisedName_*` fonksiyonlarında. |
| Serileştirme | BeerXML ve `.bsmx` metin şablonuyla (`Recipe.xml` içinde `$NAME`); bulut için JSON | Bildirimsel eşleme tabloları: `XmlRecordDefinition` (BeerXML'de yaklaşık 466 alan eşlemesi), `JsonRecordDefinition` (BeerJSON'da yaklaşık 310, DotBeer'de yaklaşık 314); şema doğrulamalı (libxml2, Boost.JSON) |
| Raporlama | Şablon motoru: HTML ve TXT şablonları, `$VAR` ve `$STYLE.OG_RANGE` gibi noktalı yollar, CSS teması; kullanıcı kendi şablonunu ekleyebiliyor | Kodla üretilen HTML (Formatter sınıfları); özelleştirilemiyor |
| Çeviri | Bütün metinler SQLite'ta; dil dosyası alternatif bir SQLite olarak yükleniyor; teknik olmayan çevirmen için kolay | Qt Linguist `.ts` / `.qm`, 29 dil; çalışırken dil değişimi; çevirmen araç zinciri olgun |
| Undo/redo | Menüde var (`M_UNDO`, `M_REDO`, `M_R_UNDO`); klasör işlemleri için `F_F_OLD_ID` var; nasıl uygulandığı bilinmiyor | Komut deseni: `SimpleUndoableUpdate`, `UndoableAddOrRemove(List)`, `RelationalUndoableUpdate`; `QUndoStack` |
| Test kapsamı | Görünür bir test yok (kapalı kaynak) | QtTest, yaklaşık 2.900 satır, yaklaşık 33 test fonksiyonu: birim dönüşümü, all-grain hesabı, kaynatma zamanlaması, envanter, mash hacimleri, buharlaşma, ekipman hacimleri, BeerJSON gidiş-dönüş, BeerXML null değerler, DotBeer içe/dışa aktarma, klasör yolları. Hesap testleri sınırlı: tüm IBU ve renk yöntemleri test edilmiyor. |
| Dağıtım | NSIS (Windows), DMG (macOS); lisans etkinleştirme | CMake ve Meson; deb, rpm, AppImage, Flatpak, Windows, macOS |

**Çıkarım:** BeerSmith'in "alan tanımı veri olarak tutulur" yaklaşımı, çeviri, ipucu, birim etiketi, rapor değişkeni ve form üretimini tek bir kaynaktan besliyor. Brewtarget'ın property sistemi ise tip güvenliği ve serileştirme eşlemesi açısından güçlü. Yeni bir uygulamada iki yaklaşım birleştirilebilir: alan kataloğu (ID, tip, nicelik türü, iç birim, etiket anahtarı, BeerJSON yolu, BeerXML etiketi) tek bir bildirimsel şemada tutulur; tipler ve formlar bu şemadan üretilir.

---

## 4. Standartlarla uyum

### 4.1 Destek özeti

| Standart | BeerSmith | Brewtarget |
|---|---|---|
| BeerXML 1.0 | İçe ve dışa aktarma. Dışa aktarma, Reports klasöründeki `.xml` şablonlarıyla yapılıyor. Şablon bulunamazsa hata veriyor (`S_ERR_XML_TEMPLATE`). Bazı tablo türleri dışa aktarılamıyor (`S_ERR_XML_TYPE`). | İçe ve dışa aktarma; XSD (`schemas/beerxml/v1`). |
| BeerJSON 1.0 | **Yok.** Menü, metin ve alan tablolarında "json" yalnızca bir yerde geçiyor: bulut senkronu için olan `F_R_JSON`. | Var (`schemas/beerjson/1.0`). Veri modeli zaten BeerJSON'a göre yeniden yazılmış (enum'lar, eklemeler, boil ve fermentation profilleri). |
| DotBeer 1.0.0 | Yok | Var |

### 4.2 Alan eşlemesi (seçme)

| Kavram | BeerXML etiketi | BeerJSON | BeerSmith alanı | Brewtarget property'si |
|---|---|---|---|---|
| Hop alfa | `HOP/ALPHA` | `alpha_acid` | `F_H_ALPHA` | `Hop::alpha_pct` |
| Hop kullanım ve zamanlama | `USE`, `TIME` | `timing.use`, `timing.time`, `timing.duration` | `F_H_USE`, `F_H_BOIL_TIME`, `F_H_DRY_HOP_TIME`, `F_H_DRY_START` | `RecipeAddition::stage`, `step`, `addAtTime_mins`, `duration_mins` |
| Whirlpool sıcaklığı | – | `timing` + `temperature` (adım düzeyinde) | `F_H_WHIRLPOOL_TEMP` | – |
| HSI | `HSI` | `percent_lost` | `F_H_HSI` | `sixMonthAlphaLoss_pct` |
| Hop yağları | `MYRCENE` vb. (BeerJSON uzantısı) | `oil_content` | – | ✅ 13 yağ alanı |
| Fermentable verimi | `YIELD` | `yield.fine_grind` vb. | `F_G_YIELD` | `fineGrindYield_pct` vb. |
| Maya attenuation | `ATTENUATION` | `attenuation_range` | `F_Y_MIN_ATTENUATION`, `F_Y_MAX_ATTENUATION` | min/max |
| Mash adımı | `MASH_STEP` | `MashStepType` | `steps` JSON'u | `MashStep` |
| Kaynatma adımı | – | `BoilProcedureType` | – | `BoilStep` |
| Fermentasyon adımı | `PRIMARY_AGE`, `PRIMARY_TEMP`… (sabit) | `FermentationStepType[]` | `M_AGE` sütunları | `FermentationStep` |
| Ekipman | `EQUIPMENT` (tek nesne) | `equipment_items[]` (kap kap) | `M_EQUIPMENT` (tek nesne) | `Equipment` (kap bazlı alanlar) |
| Envanter | `INVENTORY` (Appendix A, metin) | `inventory` | `F_*_INVENTORY` | `StockPurchase` (BeerJSON'daki `inventory` yerine) |
| Ölçümler | `OG`, `FG` (gerçekleşen) | `BatchType` 1.0'da yok | `F_R_*_MEASURED` | `BrewLog` |
| Klasör | – | – | `F_R_FOLDER_NAME` | `FOLDER_PATH` uzantı etiketi |

**Dikkat edilecek noktalar:**

- BeerSmith'in BeerXML şablonları `DISPLAY_*` ve `INVENTORY` gibi Appendix A alanlarını yazıyor. Whirlpool sıcaklığı, kuru hop fazı, pitch rate, su hedefi ve fermentasyon okumaları gibi BeerSmith'e özgü alanlar BeerXML'e aktarılmıyor.
- Brewtarget bu `DISPLAY_*` etiketlerini okurken yok sayıyor (eşlemede `NULL_STR`). BeerXML'de karşılığı olmayan BeerJSON alanlarını ise uzantı etiketi olarak yazıyor.
- BeerXML'deki 3 aşamalı fermentasyon ile BeerJSON'daki N adımlı fermentasyon arasında gidiş-dönüş kayıpsız değil.
- BeerSmith'ten veri taşımak için tek ortak format BeerXML. Bu yüzden kendi uygulamanızda **BeerXML içe aktarma zorunlu**. Bunun yanında `.bsmx` içe aktarma da düşünülebilir; `.bsmx`, BeerXML benzeri XML ve kendi alan adlarını kullanıyor.

---

## 5. Kendi uygulamanız için öneriler

### 5.1 MVP kapsamı

| Öncelik | Özellik | Neden |
|---|---|---|
| **MVP** | Tarif editörü: bira türleri, OG/FG/ABV/IBU (Tinseth ve Rager; Garetz sonra), renk (Morey), stil göstergesi | İki programın da çekirdeği |
| **MVP** | Tersine ayar: hedef OG, hedef IBU, hedef renk ve yüzdeyle tahıl | BeerSmith'in en belirgin üstünlüğü; Brewtarget'ta yok |
| **MVP** | Ekipman profili ve hacim zinciri (`formuller/05`), mash profili ve strike hesabı | Doğru hacim olmadan hesaplar anlamsız |
| **MVP** | Brew day ekranı: talimat listesi, zamanlayıcılar (mash, kaynatma, whirlpool) ve ölçüm girişi ile verim hesabı (ayrı Batch kaydı olarak) | Mobil ve web için en değerli ekran |
| **MVP** | BeerXML içe ve dışa aktarma, BeerJSON içe ve dışa aktarma | Kullanıcıları taşımak ve gelecekle uyum |
| **MVP** | Temel araçlar: ABV, hidrometre sıcaklık düzeltmesi, refraktometre, priming/karbonasyon, birim dönüştürücü | Düşük maliyet, yüksek kullanım |
| **MVP** | Metrik ve İngiliz birimleri, i18n (TR ve EN) | Hedef kitle için |
| **MVP** | Undo/redo ve otomatik kayıt | İki programda da var; beklenen bir özellik |
| v1.1 | Su kimyası, tuz eşleştirme, mash pH (asit dahil; molar kütleyle doğru faktörler) | BeerSmith bile yıllarca hatalı faktör kullanmış; dikkatli ve testli yapılmalı |
| v1.1 | Maya: canlılık, starter, pitch rate | |
| v1.1 | Fermentasyon okumaları, grafik, CSV; Tilt, iSpindel ve RAPT köprüsü | BeerSmith yalnızca Tilt'i destekliyor, Brewtarget hiçbirini; bu bir farklılaşma alanı |
| v1.1 | Lot bazlı envanter (Brewtarget modeli), alışveriş listesi ve tarif maliyeti (BeerSmith modeli) | İki programın en iyi taraflarının birleşimi |
| v1.2 | Mead, cider ve seltzer modülü (besin, sülfit, backsweeten, sanal OG) | BeerSmith'te var, Brewtarget'ta yok |
| v1.2 | Bulut senkronu, paylaşım, puanlama ve yorum, topluluk araması | |
| v1.2 | Kullanıcı rapor şablonları, takvim ve görev listesi, parti-gyle, dilution, boil-off araçları | |

### 5.2 Önerilen veri modeli

İki programın en iyi fikirleri şöyle birleştirilebilir:

1. **Kütüphane malzemesi ile tarif eklemesi ayrı tutulsun, ama ekleme bir anlık görüntü (snapshot) taşısın.**
   - `ingredient` (kütüphane): `id` (UUID), tür, kimlik alanları ve varsayılan özellikler.
   - `recipe_addition`:
     - `recipe_id` ve `ingredient_id` (nullable referans; Brewtarget'taki gibi);
     - hesaplamayı etkileyen özelliklerin anlık kopyası (alfa, yield, renk, attenuation; BeerJSON `HopAdditionType` yaklaşımı ve BeerSmith'in tarihsel doğruluğu);
     - miktar ve birim;
     - zamanlama: `stage`, `step_id`, `time`, `duration`, `temperature`, `at_gravity`, `at_ph`.
   - Kütüphanede bir değişiklik yapıldığında "tariflerdeki kopyaları güncelle" seçeneği sunulsun.
2. **Profiller tarife kopyalanarak eklensin; adımlar ayrı tablolarda dursun.**
   - Mash, boil ve fermentation profilleri adım tablolarıyla tutulsun (Brewtarget, BeerJSON).
   - Tarife eklenen profil `source_profile_id` ile kopyalansın; böylece tarif kendi kendine yeterli olur.
3. **Demleme kaydı (Batch) tariften ayrı bir varlık olsun.** Tarif sürümünün dondurulmuş bir kopyasını ya da `recipe_version_id` referansını taşısın. Ölçümler beklenen/ölçülen çiftleri olarak tutulsun (Brewtarget `BrewLog`).
   - Fermentasyon okumaları ayrı bir `batch_reading` tablosunda tutulsun: zaman, SG, sıcaklık, pH, kaynak cihaz, ham veri (BeerSmith `AgeData`).
4. **Sürümleme:** `recipe_version` tablosu (`recipe_id`, `version_no`, `parent_version_id`, `created_at`, `snapshot_json`). Brewtarget'ın ata zinciri ile BeerSmith'in arşiv ayarları birleşebilir. Demleme yapılmış bir sürüm değişmez olsun.
5. **Senkron alanları her tabloda bulunsun:** `id` (UUID; BeerSmith'teki `_PERMID_` / `_CLOUDID_` ikiliği yerine tek ve global bir kimlik), `created_at`, `updated_at`, `deleted_at` (yumuşak silme), `version` / `etag`, `sync_state`. BeerSmith'teki gibi ayrı `M_CLOUD_*` ikiz tabloları kullanılmasın; sahiplik ve görünürlük alanlarıyla tek tablo yeterli.
6. **Envanter bir hareket defteri gibi tutulsun:** `stock_lot` (alış, fiyat, tarih, lot özellikleri) ve `stock_movement` (kullanım veya düzeltme, `batch_id`). Alışveriş listesi tariften ve eşiklerden türetilsin.
7. **Her tür için klasör ya da etiket** (Brewtarget); tek bir yol kaynağı tutulsun, metin yolu tekrarlanmasın.
8. **İç birimler SI olsun** (Brewtarget). Görüntüleme birimi kullanıcı tercihine bağlansın. Sayılar metin olarak tutulmasın.
9. **Alan kataloğu veri olarak tutulsun** (BeerSmith `Field` / `Option` fikri): her alan için ID, tip, nicelik türü, varsayılan değer, min/max, i18n anahtarı, BeerJSON yolu ve BeerXML etiketi. Formlar, doğrulama, rapor değişkenleri ve içe/dışa aktarma eşlemesi bu katalogdan üretilsin.
10. **Özel alanlar** (BeerSmith) için `custom_field_def` ve `custom_field_value` tabloları (ya da JSONB sütunu) kullanılsın.
11. **Araç girdileri** (BeerSmith `M_TOOL_*`) kullanıcı tercihleri altında tek bir JSON belgesinde tutulsun; 29 ayrı tabloya gerek yok.

### 5.3 Standartlar

- **Varsayılan iç model BeerJSON 1.0'a yakın tasarlansın.** Enum değerleri BeerJSON'dakilerle aynı olsun (Brewtarget de böyle yapıyor). Tarifin dışa aktarımı doğrudan BeerJSON olsun.
- **BeerXML 1.0 içe aktarma zorunlu** (BeerSmith ve eski araçlardan taşıma). Dışa aktarımda Appendix A'daki `DISPLAY_*` alanları da yazılsın; kendi alanlarınız uzantı etiketi olarak eklensin.
- **DotBeer** (referanslı JSON, MPL 2.0 lisanslı şema), yedekleme ve tam veritabanı dışa aktarımı için örnek alınabilir. Bu format malzemeleri ve stok alımlarını ID ile taşıyor.
- **Cihaz verisi:** Tilt (CSV ve Google Sheets), iSpindel (HTTP/JSON) ve RAPT API'si için bir adaptör katmanı kurulsun. Bu alanda iki programın da standardı yok.
- **Hesap motoru:** `/home/user/formuller/*.md` içindeki test vektörleriyle birim testleri yazılsın. Brewtarget'ın test yapısı (hesap, serileştirme gidiş-dönüş testleri ve birim dönüşümü) örnek alınabilir, ama kapsamı daha geniş tutulmalı: her IBU ve renk yöntemi, su kimyası ve hacim zinciri.
