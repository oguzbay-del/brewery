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
