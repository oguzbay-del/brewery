# Gravite tetikleyicisi, fıçıda priming, cask: spesifikasyon

Plandaki "Eksik hesaplar" tablosunda yazılmamış kalan üç hesap:

| Hesap | Aşama |
|---|---|
| Fermantasyon adımına gravite tetikleyicisi (diacetyl rest, FG sabit, lagering) | Ev MVP |
| Fıçıda priming miktarı | Ev MVP |
| Cask (İngiltere): hedef CO₂, priming, durultucu, kondisyon süresi | İşletme çekirdek |

Referans uygulama ve test vektörleri: `testler/11_tetikleyici_fici_cask_tv.py` (`python3 -I testler/11_tetikleyici_fici_cask_tv.py`).

**Karşılaştırma ilkesi** (00-OKU-BENI.md › Önemli genel notlar): bu dosyadaki bütün gravite eşikleri (Δ, `FG_ref + Δ`, 4 puan, 1 puan) tam sayı birimde karşılaştırılır; 1 birim = 0.00001 SG (0.01 puan), `u(SG) = round(SG × 100000)`. Sürekli büyüklüklerde (eğim) açık tolerans 1e-9 kullanılır; lagering süresi gibi puan sayan hesaplar önce tam sayı puana çevrilir. Gerekçe: `(1.048 − 1) × 1000 = 48.00000000000004` ve `1.014 − 1.013 > 0.001` ikili kayan noktada.

İşaretler: **[kesin]** tanım, stokiyometri veya fizik; **[tasarım]** bizim seçtiğimiz yöntem; **[doğrulanacak]** varsayılan değer.

Birimler: °C, L, g, saat; SG; basınç bar (gösterge). Kalıntı CO₂ ve denge basıncı 04 §3.2 ve §3.5'teki formüllerdir.

---

## 1. Fermantasyon adımına gravite tetikleyicisi

Fermantasyon profilindeki bir adım süre yerine gravite koşuluyla başlayabilir: "FG'ye 4 puan kala diacetyl rest'e geç", "FG sabitlenince cold crash". Tetiklenen adım Bugün'e görev, bildirim kutusuna hatırlatıcı olarak düşer; kullanıcı onaylamadan kontrolcüye komut gitmez (plan: Cihaz entegrasyonu ilkeleri).

### 1.1 Referans FG [tasarım]
- Varsayılan `FG_ref = FG_tarif` (maya attenüasyonu ve mash sıcaklığı düzeltmesiyle, 01).
- Fermantasyon tahmini gösteriliyorsa (07 §1.4) ve `FG∞ > FG_tarif + Δ − 0.001` ya da `FG∞ < FG_tarif − (Δ − 0.001)` ise `FG_ref = FG∞` (tam sayı birimde: `u(FG∞) > u(FG_tarif) + u(Δ) − 100` ya da `< u(FG_tarif) − (u(Δ) − 100)`). Maya beklenenden farklı attenüe ediyorsa tetik tarifin FG'sine göre ya hiç gelmez ya da erken gelir. Ekranda "tahmine göre" notu çıkar.
- **Her iki yön de Δ'ya bağlıdır (simetrik; F1 kararı 14).** Yukarı yön: `FG∞`, `FG_tarif + Δ` ile `FG_tarif + 0.004` arasındaysa tetik eşiği asimptotun altında kalır ve hiç gelmezdi (ale: FG_tarif 1.010, FG∞ 1.013, Δ 0.002). Aşağı yön: eskiden sabit 0.004 idi; artık Δ'ya göre (ale: FG_tarif 1.010, FG∞ 1.0085, Δ 0.002 → tahmin; eski kuralda tarif kalırdı, tetik gerçek FG'nin 1.5 puan üstünde, yani erken gelirdi; vektör 1m).

### 1.2 "Puan kala" koşulu [tasarım]
```
tetik: SG ≤ FG_ref + Δ
```
- **Cihaz verisinde:** saatlik medyan (07 §1.1) art arda **3 noktada** koşulu sağlamalı. Tek bir gürültülü okuma tetiklemez. Tetik zamanı, üç noktanın ilki.
- **Elle okumada:** tek okuma yeter; görev "tekrar ölçerek doğrula" notuyla gelir.
- Varsayılan Δ: lager diacetyl rest **0.004**, ale **0.002** [doğrulanacak]. Profilde değiştirilebilir. Ekranda yüzde karşılığı da yazar: `p = (OG − SG)/(OG − FG_ref)`.

### 1.3 Tetik zamanı tahmini [tasarım]
07 §1 modeli oturmuşsa (`SG(t) = FG∞ + A·e^(−k·t)`):
```
t_tetik = ln(A / (FG_ref + Δ − FG∞)) / k        (FG_ref + Δ > FG∞ ise)
```
Uç durumlar (referans uygulama `(durum, saat)` döndürür):
- `FG_ref + Δ ≤ FG∞` → **"tetik erişilemez"**: eğri eşiğe hiç inmez. Ekranda "Bu fermantasyon hızıyla eşiğe inilmeyecek görünüyor; eşiği yükselt ya da elle karar ver" yazılır. Tahmin gösterilirken FG_ref = FG∞ olduğundan bu durum yalnız tahminin kapalı olduğu ya da Δ ≤ 0 girildiği hâlde oluşur.
- `A ≤ FG_ref + Δ − FG∞` → eşik zaten sağlanıyor (tetik geçmiş), `t_tetik = 0`.
Takvim ve Bugün "tahmini diacetyl rest: Perşembe öğleden sonra" gösterir.

**Az okumalı (elle) partide (Karar T1.17)** 10 §1.2 modeli kullanılır: `SG(t) = FG + (OG − FG)·e^(−k·(t − L))`, `FG = FG_tarif` (10 §1: FG tariften alınır), `k` ve `L` 10 §1.3 oturtmasından (`manual::few_forecast` çıktısı: yuvarlanmış `k`, 4 hane) ve gecikme fazı. Eşik `θ = FG_ref + Δ`; elle partide tahmin gösterilmediğinden `FG_ref = FG_tarif`'tir (§1.1). Hepsi tam sayı birimde (`u`):
```
d = u(FG_ref) + u(Δ) − u(FG)           (θ'nın modelin asimptotundan yüksekliği)
d ≤ 0                  → "tetik erişilemez"
u(OG) − u(FG) ≤ d      → "tetik geçmiş" (t = 0; OG zaten eşikte)
k < 0 hata, k = 0 hata (sıfıra bölme)
t_tetik = L + ln( (u(OG) − u(FG)) / d ) / k      (saat, parti başlangıcından)
```
Gün çözünürlüğü (10 §1.6 ile aynı biçim, saat verilmez): parti başlangıcından `(floor(t_tetik/24), ceil(t_tetik/24))`; son okumadan itibaren kalan `(max(0, floor((t_tetik − t_son)/24)), max(0, ceil((t_tetik − t_son)/24)))`. Etiket "yaklaşık". FG ± σ aralığı tetik için hesaplanmaz (tek `k`); belirsizlik yalnızca gün yuvarlamasıyla gösterilir. Hesap yalnız `few_forecast` tahmin verdiğinde (durum "Estimate": `k > 0`, yeterli veri, durgunluk yok) yapılır; diğer durumlarda tetik tahmini yoktur.

### 1.4 "FG sabit" koşulu [tasarım]
- **Cihaz:** son 48 saatin saatlik medyanlarına doğrusal eğim; en az 24 nokta. `|eğim| × 24 ≤ 0.0005` (günde yarım puandan az) ise sabit.
- **Elle:** 10 §1.5 durgunluk kuralı (48 saat arayla iki okuma, fark ≤ 0.001, tam sayı birimde).
- Eğim karşılaştırması `≤ 0.0005 + 1e-9` (tolerans). Son 48 saatteki bütün noktalar aynı zamandaysa eğim tanımsızdır, "veri yetersiz" döner.
- Cold crash ve "ambalaja hazır" önerisi bu koşulu kullanır.

### 1.5 Sıcaklık rampası ve lagering [tasarım]
```
rampa süresi (saat) = |T_hedef − T_başlangıç| / hız × 24
```
- Varsayılan hız: lager soğutma **2 °C/gün** [doğrulanacak]; diacetyl rest'e ısınma ve cold crash "olabildiğince hızlı" (ekipman profilindeki soğutma hızı).
- **Lagering süresi önerisi:** her 8 OG puanı için 1 hafta, en az 2 en çok 12 hafta [doğrulanacak; yaygın kaba kural]. Önce OG tam sayı puana çevrilir (`round((OG − 1) × 100000)` 0.01 puan biriminde), sonra bölünür ve **yukarı yuvarlanır** (`ceil(puan / 8)`: OG 1.048 → 6 hafta, 1.050 → 7 hafta). Yalnız öneri; kullanıcı profilde değiştirir.

---

## 2. Fıçıda priming

BeerSmith ve Brewtarget fıçı için ayrı bir kural uygulamıyor (04 §3.3: `CarbKegFactor = 0.5` tanımlı ama kullanılmıyor). "Fıçıya şişenin yarısı kadar şeker" önerisi topluluk alışkanlığı. Fiziksel olarak fıçı daha az değil, biraz **daha fazla** şeker ister, çünkü kafa boşluğundaki gaz da basınçlanır. Uygulama iki modu ayrı ayrı sunar.

### 2.1 Biradaki CO₂ [kesin]
Planda seçilen yöntem: Brewtarget stokiyometrisi + BeerSmith kalıntı CO₂'si.
```
n_bira (mol) = max(0, hedef_vol − kalıntı_vol(T_fermantasyon_max)) × V_bira / 22.414
```
**İki ayrı CO₂ verimi varsayımı bilinçli olarak korunur [F1 kararı 10]:** priming şekeri tamamen fermente olur ve maya büyümesi ihmal edilir, bu yüzden şekerde **kuramsal** verim kullanılır (glukoz 0.4886, sakaroz 0.5143 g CO₂/g). §3.2'deki kalan ekstraktta ise maya biyokütlesi de oluştuğundan **Balling** oranı kullanılır (0.9565/2.0665 = 0.4629 g CO₂/g). İki varsayım %5–10 farklıdır; fark kasıtlıdır, tek varsayıma indirilmez.
**1 vol CO₂ = 1.9635 g/L** (= 44.01/22.414, ideal gaz; 09 ve 10 ile tek sabit, F1 kararı 9).

### 2.2 Kafa boşluğundaki CO₂ ve hesap sıcaklığı [kesin fizik; F1 kararı 2]
Denge basıncı 04 §3.5 polinomundan. Tablo saf CO₂ kafa boşluğu varsayar, dolayısıyla CO₂ kısmi basıncı = gösterge + 1 atm.

**Şeker, servis sıcaklığındaki dengeye göre hesaplanır** (toplam CO₂ korunur): fıçı kondisyonda ılık, serviste soğuktur; soğuyunca boşluktaki CO₂'nin bir kısmı biraya geçer. Şeker, soğuk serviste biranın tam hedef vol'ünde olacağı miktardır:
```
n_boşluk,servis = P_abs(hedef_vol, T_servis) × V_boşluk / (R × T_servis,K) − n_başlangıç
n_başlangıç     = 1 atm × V_boşluk / (R × T_kondisyon,K)   (fıçı CO₂ ile süpürüldüyse; havayla ise 0)
n_şeker_CO₂     = n_bira + max(0, n_boşluk,servis)
```
(Eski tanım şekeri kondisyon sıcaklığındaki boşluk dengesiyle kuruyordu ve servis sıcaklığında hedefin üstüne çıkıyordu: 2b koşullarında +0.09 vol. `T_servis = T_kondisyon` verilirse eski sonuç aynen çıkar, vektör 2h.)

**Tepe basıncı (kondisyon sıcaklığında):** aynı toplam CO₂, kondisyon sıcaklığında yeniden dengeye getirilir:
```
n_toplam = kalıntı_vol(T_ferm)·V_bira/22.414 + n_şeker_CO₂ + n_başlangıç
v_tepe:   v·V_bira/22.414 + P_abs(v, T_kond)·V_boşluk/(R·T_kond,K) = n_toplam      (v için sayısal çözüm, bisection)
tepe_basinci_bar = (P_abs(v_tepe, T_kond) − 1 atm)/10⁵      ; havayla dolu fıçıda toplam gösterge + 1.013 bar
```
Ekran tepe basıncını ayrıca gösterir ve fıçı kaydındaki çalışma/güvenlik basıncıyla (`limit_bar`) karşılaştırır; aşarsa **`uyarilar`** alanına "tepe basıncı fıçı sınırını aşıyor: X bar > Y bar (kondisyon T °C)" yazılır. Hesap uyarıyla durmaz.

### 2.3 Şeker [kesin stokiyometri]
| Şeker | Mol CO₂ / mol şeker | Molar kütle |
|---|---|---|
| Dekstroz monohidrat (mısır şekeri) | 2 | 198.17 g/mol |
| Sakaroz | 4 | 342.30 g/mol |
| İnvert şeker | 4 | 360.31 g/mol |

`şeker (g) = (n_bira + n_boşluk) / (mol CO₂ / mol şeker) × M / kuru_madde`

`kuru_madde` (0–1): ürünün kuru madde oranı. **Her şeker ve şurup malzemesinde bulunan bir alandır** ve değeri malzeme veritabanından gelir (şurup ve bloklarda, ör. invert şeker, 1'den küçüktür; %80 kuru madde için aynı CO₂ 1/0.8 = 1.25 kat ürün ister, vektör 3g). **Alan boşsa** ekran uyarır ve hesap **durmaz**: 1.0 ile hesaplanır ve **`uyarilar`** alanına "kuru madde bilinmiyor: 1.0 varsayıldı (şurup ve bloklarda şeker miktarı eksik çıkabilir)" yazılır (vektör 2g, 3h) [F1 kararı 1; tipik invert şurup değerleri %70–85, ürüne göre, doğrulanacak]. Saf kuru şekerde (dekstroz, sakaroz) 1.0 veritabanından gelir, uyarı çıkmaz.

Karşılaştırma notu: aynı girdi (2.4 vol, 20 °C, 19 L) için 04 §5.3 BeerSmith yöntemi 117.3 g (susuz glukoza denk sabit), bu belge 129.2 g (dekstroz monohidrat, stokiyometrik) verir; fark %10'dur ve karşılaştırma ekranında not edilir. Bizim değer stokiyometrik olarak doğrudur.

### 2.4 Modlar [tasarım]
- **Tam doğal karbonasyon:** §2.1 + §2.2. 19 L birada 2 L boşluk, 4 °C servis için ≈ %5 fazla şeker (süpürülmüş fıçı; kondisyon sıcaklığında kurulsaydı %11).
- **Gösterilen basınçlar:** `servis_basinci_bar`, hedef vol için servis sıcaklığındaki CO₂ tablosu basıncıdır (örnek 0.74 bar). `tepe_basinci_bar`, kondisyon sıcaklığındaki CO₂ (kısmi) gösterge basıncıdır. Havayla dolu (süpürülmemiş) fıçıda kafa boşluğundaki 1 atm hava ayrıca basınç yaptığı için gerçek fıçı gösterge basıncı `tepe_toplam_gosterge_bar = tepe_basinci_bar + 1.013` olur (örnek: 1.71 → 2.72 bar); fıçı ve emniyet vanası sınırı bu değerle karşılaştırılır (§2.2 uyarısı). Süpürülmüş fıçıda ve boşluk 0 iken ikisi aynıdır.
- **Kısmi + CO₂ ile tamamla:** tam miktarın **1/3**'ü [doğrulanacak]. Kapak contasını oturtur ve oksijeni süpürür; servis sıcaklığında hedef vol için gereken CO₂ basıncı (04 §3.5; `co2_tamamlama_bar`) ayrıca gösterilir. Daha az tortu, daha hızlı servis.
- Kafa boşluğu ekipman profilindeki fıçı hacminden (Kaplar) ve doldurulan litreden gelir.

---

## 3. Cask (İngiltere)

Cask ale, fıçıda ikinci fermantasyonla düşük karbonasyona getirilir ve el pompasıyla servis edilir. Karbonasyon mahzende (cellar) spile ile ayarlanır.

### 3.1 Hedef CO₂ [doğrulanacak]
Varsayılan **1.1 vol**, aralık 0.9–1.4. Mahzen sıcaklığı 11–13 °C.

### 3.2 Kalan ekstrakttan gelen CO₂ [kesin: Balling]
Cask çoğu zaman FG'ye 1–2 puan kala doldurulur; kalan ekstrakt CO₂ üretir (Balling oranı, §2.1'deki gerekçeyle).
```
ΔAE (°P)   = Plato(SG_dolum) − Plato(FG_ref)
ΔRE (g/L)  = 0.8192 × ΔAE × 10 × SG_dolum            (gerçek ekstrakt; Balling RE = 0.1808·OE + 0.8192·AE)
CO₂ (g/L)  = ΔRE × 0.9565 / 2.0665                    (Balling: 2.0665 g ekstrakt → 0.9565 g CO₂)
CO₂ (vol)  = CO₂ (g/L) / 1.9635
```
Bir puan kalan ekstrakt yaklaşık **0.5 vol** verir; iki puan hedefin üstüne çıkar.

### 3.3 Priming [kesin stokiyometri]
```
eksik_vol = hedef − kalıntı_vol(T_fermantasyon_max) − CO₂_ekstrakttan
şeker = §2.3 ile (varsayılan invert şeker; kuru madde boşsa §2.3 uyarısı)
```
- `eksik_vol < 0` ise şeker önerilmez; "fazla CO₂: mahzende spile ile boşaltılır" notu çıkar.
- Kafa boşluğu cask'ta küçük ve servis basınçsız olduğu için §2.2 uygulanmaz.

### 3.4 Durultucu (finings) [doğrulanacak]
```
doz (mL) = V (L) × oran (mL/L)        varsayılan isinglass 2 mL/L
```
Doz ürüne ve biraya göre çok değişir. Ekran, ölçüm silindirinde durultucu testi yapılmasını önerir; test sonucu girilirse oran o biranın tarifine kaydedilir.

### 3.5 Kondisyon süresi [tasarım, doğrulanacak]
```
gün = 3 × 2^((12 − T_mahzen) / 10)
```
12 °C'de 3 gün. Süre dolunca "berraklık ve CO₂'yi kontrol et (spile)" görevi Bugün'e düşer. Ambalaj kaydında cask satırı bu tarihle "servise hazır" olur.

---

## 4. Test vektörleri

| # | Girdi | Beklenen |
|---|---|---|
| 1a | SG = 1.012 + 0.036·e^(−0.03t), Δ 0.004, saatlik | tetik 74. saat |
| 1b | Aynı eğri, model tahmini | 73.2 saat |
| 1c | 13. saatte tek nokta eşik altında, 14'te üstünde, 15–17 altında | tetik 15. saat (13 tetiklemez) |
| 1d | FG tarif 1.012, tahmin 1.018 gösteriliyor (Δ 0.004) | FG_ref 1.018 (tahmin) |
| 1m | Aşağı yön Δ'ya bağlı: FG tarif 1.010, tahmin FG∞ 1.0085, Δ 0.002; FG tarif 1.012, FG∞ 1.0085, Δ 0.004; FG tarif 1.012, FG∞ 1.0095, Δ 0.004 | FG_ref 1.0085 (tahmin); 1.0085 (tahmin); 1.012 (tarif) |
| 1i | Ölü bölge: FG tarif 1.010, tahmin FG∞ 1.013, Δ 0.002 (ale) | FG_ref 1.013 (tahmin) |
| 1j | FG_ref 1.010, FG∞ 1.013, Δ 0.002, A 0.037, k 0.03 | tetik erişilemez |
| 1k | FG_ref 1.012 = FG∞, A 0.003, Δ 0.004 | tetik zaten geçmiş (t = 0) |
| 1e | Son 48 saat eğimi −0.00024/gün | sabit |
| 1f | Son 48 saat eğimi −0.0012/gün | sabit değil |
| 1g | 15 → 2 °C, 2 °C/gün | 156 saat |
| 1h | OG 1.048; 1.072 | 6 hafta; 9 hafta |
| 1l | OG 1.050; 1.056; 1.080; 1.0485 | 7; 7; 10; 7 hafta (yukarı yuvarlama) |
| 2a | 2.4 vol, fermantasyon 20 °C, kondisyon 20 °C, servis 4 °C, 19 L, boşluk 0, kuru madde 1.0 | dekstroz 129.2 g (şişeyle aynı); servis 0.74 bar, tepe 1.80 bar |
| 2b | Aynı, 2 L boşluk, CO₂ ile süpürülmüş | 136.1 g (+%5.3); servis 0.74 bar; tepe 2.32 vol, 1.71 bar |
| 2c | Aynı, havayla | 144.3 g (+%11.7); tepe CO₂ basıncı 1.71 bar, toplam gösterge **2.72 bar** |
| 2d | Kısmi 1/3, servis 4 °C | 45.4 g; CO₂ ile 0.74 bar; tepe 0.64 bar |
| 2e | 2b, sakaroz | 117.5 g (şişe eşdeğeri 111.6 g) |
| 2g | 2b ama kuru madde bilinmiyor (`dry = None`) | 136.1 g (1.0 ile); `uyarilar`: "kuru madde bilinmiyor" |
| 2h | 2b ama servis 20 °C (= kondisyon): eski tanım | 143.8 g (+%11.3), tepe 1.80 bar |
| 2i | Kondisyon 25 °C, havayla, fıçı sınırı 2.5 bar | 144.3 g; tepe 2.29 vol, CO₂ 2.02 bar, toplam gösterge 3.04 bar; `uyarilar`: "tepe basıncı fıçı sınırını aşıyor: 3.04 bar > 2.50 bar" |
| 3a | Firkin 40.9 L, hedef 1.1 vol, fermantasyon 19 °C, FG'de dolum | kalıntı 0.89 vol, eksik 0.21 vol, invert 34.9 g (0.85 g/L) |
| 3b | 1 puan kala dolum | ekstrakttan 0.50 vol, şeker yok, fazla |
| 3c | 2 puan kala dolum | ekstrakttan 0.99 vol, şeker yok, fazla |
| 3g | 3a ama invert şeker %80 kuru madde | 43.7 g (1.07 g/L) |
| 3h | 3a ama kuru madde bilinmiyor | 34.9 g (1.0 ile); `uyarilar`: "kuru madde bilinmiyor" |
| 3e | Durultucu 2 mL/L, firkin | 82 mL |
| 3f | Mahzen 12 °C; 8 °C | 3.0 gün; 4.0 gün |

## 5. Açık noktalar
- Şeker servis sıcaklığındaki dengeye göre kurulur (§2.2, F1 kararı 2); servis sıcaklığı kullanıcı girdisidir (varsayılan 4 °C). Kondisyonun servisten sıcak olduğu ve fıçının serviste soğutulduğu varsayılır. Soğuk kondisyonda (T_kond < T_servis) tepe basıncı servis basıncından düşük çıkar, uyarı gerekmez.
- Havalı fıçıda hava kısmı basıncı, sıcaklık değişiminde sabit 1 atm alınır (sıcaklık ve hava miktarı etkisi ihmal) [tasarım].
- Kalıntı vol polinomu ile basınç polinomu tam tutarlı değil: kalıntı vol'de (0.8615, 20 °C) basınç 0 olmalıyken polinom 1.76 psi (0.12 bar) verir; hedef kalıntıya yakınken kafa boşluğu payı tam sıfır çıkmaz. Etkisi küçük.
- Kafa payı yüzdesi (`kafa_payi_%`) hedef ≤ kalıntı iken (şişe eşdeğeri 0) tanımsızdır, "—" gösterilir.
- **Doğrulanacak:** Δ varsayılanları, lager soğutma hızı, lagering kuralı, kısmi priming oranı (1/3), cask hedef vol, isinglass oranı, cask kondisyon süresi.
- Cask değerleri İngiltere'deki bir veya iki üreticiyle ve Cask Marque yönergeleriyle kontrol edilecek.
- 04 §3.5 basınç polinomunun 0 °C altında ve 3.5 vol üstünde güvenilirliği bilinmiyor; fıçı hesabında bu aralık dışı uyarı verir.

**Karar (T1.12 D6, D7, D13, D2, D9):**
- **Uyarılar kod + sayısal alandır; metin istemcide üretilir** (ADR-0004, Rust `PrimingWarning`, referans sözlük `{"kod": …}`; değerler yuvarlanmaz): `kuru_madde_bilinmiyor`; `polinom_sicaklik_alti {nokta: kondisyon|servis, T_C}` (T < 0 °C); `polinom_vol_ustu {nokta: hedef|tepe, vol}` (vol > 3.5 + 1e-9); `tepe_vol_doymus {sinir_vol: 10}` (toplam CO₂ 10 vol'un tuttuğundan fazla: `equilibrium_vol` 10'da doyar, değer güvenilmez); `tepe_basinci_siniri_asildi {tepe_bar, limit_bar, kondisyon_C}` (`tepe_toplam_gosterge_bar > limit + 1e-9`). Sıra bu listedeki sıradır. Cask yalnız `kuru_madde_bilinmiyor` üretir.
- **Doğrulama:** `trigger_index` `consecutive ≥ 1` ister (0 hata); `lager_weeks` `pts_per_week` < 0 ya da > 10⁶ için hata, 0 için `ZeroDivision`; sonlu olmayan ya da |x| > 10⁶ girdi, negatif hacim, kuru madde [0, 1] dışı, `part` (0, 1] dışı, `k < 0` hata verir (Rust `Error::InvalidInput`, referans `ValueError`).
