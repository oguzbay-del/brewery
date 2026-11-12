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
- `SG_preboil = 1 + (OG − 1) × (V_batch + V_trub) / V_preboil` (V_preboil > 0 ise). Soğuma ve top-up'ı dikkate almaz.

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
- `V_water = V × (SG − 1) / (SG_target − 1) − V`, negatifse 0.

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

## Kararlar (T1.12, T1.15)

- Mash'te su yokken (başlangıç hacmi ≤ 0) yapılan her dekoksiyon adımı, ilk adım da dahil, `no_water` işaretini taşır.
