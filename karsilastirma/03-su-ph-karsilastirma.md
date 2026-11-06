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
