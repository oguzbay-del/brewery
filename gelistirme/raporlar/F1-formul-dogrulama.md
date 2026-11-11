# F1: Hesap motoru formül spesifikasyonlarının bağımsız doğrulaması

Tarih: 11 Kasım 2026. Kapsam: `formuller/11`, `10`, `03` ve `09` (yalnız vergi yöntemleri ve AB etiket enerjisi); ortak tanımlar için `01` ve `04`.

## Özet

- Toplam **22 bulgu**: **2 kritik**, **7 önemli**, **13 küçük**.
- İki kritik bulgunun ikisi de fizik hatası değil, **ikili kayan nokta karşılaştırması**: durgunluk kuralı (10 §1.5) bir puanlık farkların yarısında çalışmıyor; lagering haftası (11 §1.5) her 8'in katı OG'de bir hafta fazla çıkıyor ve test vektörü bu hatayı "beklenen" diye kaydetmiş.
- Önemli bulgular: FG referansında tetiğin hiç gelmediği ölü bölge, invert şekerin kuru madde varsayımı, fıçı kafa boşluğunun kondisyon sıcaklığında hesaplanması ve havalı fıçıda gösterilen basınç, kısmi kaynatmada su formülünün işareti, BW mash pH eğiminin fiziksel tampon kapasitesiyle uyuşmaması, vergi motorunda kayan nokta ve uç durumlar, no-chill modelinde 60 °C kesmesi.
- Fizik, kimya ve birimler genel olarak doğru: stokiyometri, molar kütleler, ideal gaz hesabı, Balling katsayıları, iyon matrisi, alkalinite/RA, asit ayrışma kesirleri, karbonat dengesi, AB enerji katsayıları ve bütün test vektörleri bağımsız hesapla tutuyor (lagering haftası dışında).
- Yöntem: depodaki Python dosyaları içe aktarılmadı. Bağımsız hesaplar `/tmp/claude-0/-home-user-brewery/9e16fef0-a8b7-5145-928c-0a9718edc1b6/scratchpad/f1/` altında (`a11.py`, `a10.py`, `a03.py`, `a09.py`), `python3 -I` ile çalıştırıldı. Aşağıdaki sayılar bu çıktılardan.
- Formül dosyaları ve test betikleri değiştirilmedi.

---

## Kritik bulgular (yanlış sonuç üretir)

### K1. Durgunluk kuralı bir puanlık farkların yarısında tetiklenmiyor
- **Yer:** `10-elle-giris-ekstrakt-nochill.md` §1.5; `testler/10_elle_ekstrakt_nochill_tv.py` satır 39 (`abs(g1 - g2) <= 0.001`). `11` §1.4 "Elle" maddesi aynı kurala atıf yapıyor, yani cold crash ve "ambalaja hazır" önerisi de etkilenir.
- **Sorun:** SG değerleri ikili kayan noktada tam temsil edilmiyor. "Fark ≤ 0.001" karşılaştırması, tam bir puan farkı olan okuma çiftlerinin yarısında yanlış çıkıyor.
- **Kanıt:** `abs(1.014 − 1.013) = 0.001000000000000112` → `<= 0.001` **yanlış**. 1.000–1.100 arasındaki 100 ardışık puan çiftinin **50'sinde** koşul sağlanmıyor (1.001/1.002, 1.003/1.004, …, 1.013/1.014). Örnek: 168. saat 1.014, 216. saat 1.013 → kural "bitmiş görünüyor" demeli, hiçbir şey demiyor. Mevcut test vektörleri (1d, 1e) farkı tam 0 olan okumalar kullandığı için hatayı yakalamıyor.
- **Düzeltme:** Bütün SG eşik karşılaştırmaları tam sayı puan üzerinden yapılmalı (`round(SG × 1000)` ya da onda bir puan; Rust'ta ondalık tip veya tam sayı). Spesifikasyona genel kural olarak yazılmalı. Test vektörüne 1.014 → 1.013 çifti eklenmeli.

### K2. Lagering süresi her 8'in katı OG'de bir hafta fazla; test vektörü yanlış değeri kaydetmiş
- **Yer:** `11-tetikleyici-fici-cask.md` §1.5 ve §4 satır 1h; `testler/11_tetikleyici_fici_cask_tv.py` satır 53–55.
- **Sorun:** Kural "her 8 OG puanı için 1 hafta". Kod `ceil((og − 1) × 1000 / 8)` kullanıyor; `(1.048 − 1) × 1000 = 48.00000000000004` olduğu için `ceil` 7 veriyor.
- **Kanıt:**

  | OG | Kurala göre (puan/8) | Koddaki ve tablodaki |
  |---|---|---|
  | 1.048 | 6 | **7** |
  | 1.072 | 9 | **10** |
  | 1.040 / 1.056 / 1.064 / 1.080 | 5 / 7 / 8 / 10 | 6 / 8 / 9 / 11 |

- **Düzeltme:** Önce `round((OG − 1) × 1000)` ile tam sayı puana çevir, sonra böl. Metin yuvarlama yönünü de söylemiyor (yukarı mı, en yakına mı); yazılmalı. Tablo 1h "6 hafta; 9 hafta" olmalı. Etkisi sınırlı (yalnız öneri), ama Rust motoru bu vektörü geçmek için hatayı kopyalamak zorunda kalır.

---

## Önemli bulgular (uç durumda yanlış veya belirsiz)

### Ö1. FG referansında ölü bölge: tetik hiç gelmeyebilir
- **Yer:** `11` §1.1 ve §1.2.
- **Sorun:** `FG_ref`, tahmine ancak `|FG∞ − FG_tarif| > 0.004` ise geçiyor. Ale varsayılanı `Δ = 0.002`. `FG∞`, `FG_tarif + Δ` ile `FG_tarif + 0.004` arasındaysa eşik (`FG_tarif + Δ`) eğrinin asimptotunun altında kalır ve tetik hiç gelmez.
- **Kanıt:** FG_tarif 1.010, FG∞ 1.013, Δ 0.002 → fark 0.003, referans tarifte kalır; eşik 1.012, SG 1.013'ün altına inmez. Lager'de (Δ 0.004) FG∞ = FG_tarif + 0.0035 ise tetik, asimptota yarım puan kala, yani çok geç gelir. §1.3 formülü bu durumda `None` döner; ekranda ne yazacağı tanımsız.
- **Düzeltme:** Geçiş eşiği sabit 0.004 yerine Δ'ya bağlanmalı: `FG∞ > FG_tarif + Δ − 0.001` ise `FG_ref = FG∞`. Ayrıca "tetik erişilemez" durumu için açık bir çıktı tanımlanmalı.

### Ö2. İnvert şeker (ve şuruplar) %100 kuru madde sayılıyor
- **Yer:** `11` §2.3 tablosu, §3.3 (cask varsayılanı invert şeker).
- **Sorun:** 360.31 g/mol iki heksozun kuru kütlesi; stokiyometri doğru. Ama biracıların kullandığı invert şeker şurup ya da bloktur ve su içerir (hatırladığım tipik değer %70–85 kuru madde; kesin değer ürüne göre, emin değilim). Spesifikasyonda kuru madde alanı yok.
- **Kanıt:** 3a vektörü 34.9 g. %80 kuru maddeli üründe aynı CO₂ için 34.9 / 0.80 = **43.7 g** gerekir; mevcut hesap %20 eksik doz verir.
- **Düzeltme:** Şeker tablosuna "kuru madde %" (malzeme veritabanından, varsayılanı doğrulanacak) eklenmeli; `şeker = mol / n × M / kuru_madde`. Koddaki `eff` parametresi buna uygun ama metinde yok ve kullanılmıyor.

### Ö3. Fıçı kafa boşluğu kondisyon sıcaklığında hesaplanıyor; havalı fıçıda gösterilen basınç 1 atm eksik
- **Yer:** `11` §2.2, §2.4; kod `head_mol`, `keg_priming`.
- **Sorun 1:** Şeker, kondisyon sıcaklığında (20 °C) birada tam hedef vol olacak biçimde hesaplanıyor. Fıçı servis sıcaklığına soğutulunca kafa boşluğundaki CO₂'nin bir kısmı biraya geçer ve bira hedefin üstüne çıkar.
- **Kanıt:** 2b koşulları (2.4 vol, 19 L, 2 L boşluk). Toplam CO₂ korunumu ve aynı basınç polinomuyla 4 °C'de denge çözülünce bira **2.49 vol** çıkıyor (+0.09 vol). Kafa boşluğu büyüdükçe fark artar; yarı dolu fıçıda (9.5 L boşluk) kafa payı %54.
- **Sorun 2:** Havayla dolu (süpürülmemiş) fıçıda mol hesabı doğru (CO₂ kısmi basıncı = tablo + 1 atm, başlangıç 0). Ancak çıktı `denge_basinci_bar` yine 1.80 bar gösteriyor. Gerçek gösterge basıncı hava da içinde olduğu için 1.80 + 1.01 = **2.81 bar**. Fıçı ve emniyet vanası sınırı açısından önemli.
- **Düzeltme:** Kafa boşluğu dengesi servis sıcaklığında kurulmalı (ya da iki sıcaklık da gösterilmeli; kondisyon sıcaklığındaki tepe basınç güvenlik için ayrıca verilmeli). Havalı durumda gösterge basıncına 1 atm eklenmeli.

### Ö4. Kısmi kaynatmada eklenecek su formülünün işareti ters
- **Yer:** `10` §2.5: `eklenecek su = V_son − V_k − kayıplar`; kod satır 98 kayıpları hiç almıyor.
- **Sorun:** Kazanda kalan kayıp (trub) fermentöre gitmez; açığı su kapatır. Doğrusu `su = V_son − (V_k − kayıp) = V_son − V_k + kayıp`. `V_k` kaynatma öncesi hacimse buharlaşma için de işaret artıdır. Eksi işaretin doğru olduğu bir yorum bulamadım. Kayıpla birlikte şeker de gittiği için kazan gravitesi de `puan_son · V_son / (V_k − kayıp)` olmalı.
- **Kanıt:** V_son 20, V_k 10, kayıp 1 L → metne göre 9 L su (fermentörde 18 L olur); doğrusu 11 L.
- **Düzeltme:** Formül düzeltilmeli, `V_k`'nin kaynatma sonu hacmi olduğu yazılmalı, kayıplı bir test vektörü eklenmeli.

### Ö5. BW mash pH eğimi, maltın bilinen tampon kapasitesinin yarısına karşılık geliyor
- **Yer:** `03-su-ph.md` §5.2, §6.3; T3 tablosu. Varsayılan model 1 (BW) diye not edilmiş.
- **Sorun:** Eğim `m` (pH / (mEq/L)) ve mash kalınlığı `t` (L/kg), örtük bir tampon kapasitesi verir: `B = t / m` (mEq / (kg · pH)). MPH için t = 3'te B = **38.0**; BW için kalınlıktan bağımsız B = **18.4**. Bildiğim literatür aralığı baz maltlar için kabaca 30–50 mEq/(kg·pH) (Troester, deLange); Kolbach'ın 0.03 pH/°dH kuralı da 0.084 pH/(mEq/L) eder, MPH ile uyumlu. BW eğimi bu aralığın yaklaşık yarısında.
- **Kanıt:** T3 (4.9 kg, 5.6234 → 5.40). Bağımsız hesap: ΔpH 0.2234 × 4.9 kg × B(35–45) = 38–49 mEq; %88 laktik 11.81 mEq/mL → **3.2–4.2 mL**. MPH 3.52 mL (aralıkta), BW **1.77 mL** (yarısı). Aynı nedenle BW, RA'nın pH etkisini iki kat büyük gösterir.
- **Belirsizlik:** Sabitler BeerSmith'ten çıkarım; çıkarımın doğruluğunu ve Bru'n Water'ın gerçek modelini buradan doğrulayamam. Söyleyebildiğim: iki model 2 kat farklı asit öneriyor ve BW fiziksel beklentinin dışında.
- **Düzeltme:** Varsayılan model bilinçli seçilmeli; ölçülmüş mash pH verisiyle (ya da yayımlanmış malt titrasyon verisiyle) karşılaştırılana kadar MPH varsayılan olmalı ve BW "doğrulanacak" işaretlenmeli.

### Ö6. Vergi motoru: kayan nokta, yuvarlama girdileri ve sessiz uç durumlar
- **Yer:** `09-isletme-2-hesaplar.md` §2.2; `testler/09_isletme_2_tv.py` satır 43–64.
- **Sorunlar:**
  1. **Para ve bant karşılaştırmaları ikili kayan noktada.** `floor(°P)` ve `plato <= üst` hesaplanmış bir °P ile çalışırsa sınırda yanlış banda düşer (K1 ve K2 ile aynı hata sınıfı). Örnek: `round(2.675, 2) = 2.67`.
  2. **Girdi hassasiyeti kuralı yok.** `plato_bandi` örneği (≤10.0; 10.1–14.0; ≥14.1) °P'nin önce 0.1'e yuvarlandığını varsayar; 10.05 için davranış tanımsız. `saf_alkol`de ABV'nin vergi için kaç haneye ve hangi yönde yuvarlanacağı parametre değil (İngiltere'de bir ondalığa aşağı kesme olduğunu hatırlıyorum; emin değilim, doğrulanacak). Bant üst sınırı `saf_alkol`de dışlayıcı (`<`), `plato_bandi`de kapsayıcı (`<=`); bu veri tablosunda açık alan olmalı.
  3. **Son kademenin üstü sessizce vergisiz.** `kademeli_hacim`de son kademenin üst sınırı aşılırsa kalan hacim için 0 vergi hesaplanıyor. Kanıt: tek kademe (100 hL'ye kadar 3.0), yıl içi 95 hL, yeni 20 hL → 15.0 vergi, **15 hL vergisiz**, hata yok.
  4. Bant bulunamazsa `next()` yakalanmamış hata veriyor; `indirim > oran` ise negatif vergi çıkıyor.
  5. `hacim` yönteminde oran birimi belirsiz: metin "oran × hacim", kod hL başına (`rate × hL`), TV 2.7 "12.49/L" için 1249.0 gönderiyor.
- **Düzeltme:** Tutarlar ve vergi girdileri ondalık tiple; her yöntem için veri tablosunda girdi yuvarlama kuralı (hane, yön), bant sınırı türü, oran birimi; son kademe açık uçlu olmalı ya da hata vermeli; negatif sonuç 0'a kırpılmalı.

### Ö7. No-chill modeli: 60 °C kesmesi eşdeğer sürenin %12–30'unu atıyor
- **Yer:** `10` §3.2 ("T < 60 °C katkısı ihmal"); kod satır 117–125 (yorum: "ftemp < 0.01").
- **Sorun:** `Ftemp(60 °C) = 0.0435`; yorumdaki 0.01 yanlış. Küp 60 °C altında saatlerce kaldığı için kuyruk ihmal edilebilir değil.
- **Kanıt (τ = 4 sa, ortam 20 °C, ince adımlı integral):**

  | T₀ | 60 °C'de kes | kesmeden | fark |
  |---|---|---|---|
  | 95 °C | 32.9 dk | 37.3 dk | +%13 |
  | 80 °C | 10.5 dk | 14.8 dk | +%42 (kesme sonucun %30'unu atıyor) |

- **Ek not:** Aynı `Ftemp` (9773 K) 02 §3'te BeerSmith sabiti olarak geçiyor. Malowicki ve Shellhammer'ın yayımladığı izomerizasyon sabiti `k₁ = 7.9·10¹¹·e^(−11858/T)`; bu aktivasyonla aynı örnek 25.3 dk verir (32.9 yerine). Yani model, τ'dan bağımsız olarak ±%25 belirsiz.
- **Düzeltme:** Kesme kaldırılmalı ya da 40 °C'ye çekilmeli; yorum düzeltilmeli. Model zaten varsayılan değil; ölçülmüş IBU ile kıyaslanmadan açılmaması kararı yerinde.

---

## Küçük bulgular (açıklama veya tutarlılık)

1. **CO₂ g/vol sabiti üç dosyada farklı.** `11` 1.9635 (44.01/22.414, ideal gaz), `09` §3.1 1.977 (gerçek gaz), `04` §3.7 1.9642 ve 1.96042. Aynı Balling hesabı cask ve spunding'de %0.7 farklı sonuç verir (TV 3.1: 2.733 yerine 2.715 g/L). Tek sabit seçilmeli.
2. **Priming verimi iki ayrı varsayım kullanıyor.** Şeker için kuramsal verim (glukoz 0.4886, sakaroz 0.5143 g CO₂/g), kalan ekstrakt için Balling (0.4629). Fark %5–10. Seçim yazılmalı.
3. **Aynı girdi için `11` ve `04` %10 farklı şeker veriyor.** 2.4 vol, 20 °C, 19 L: `11` 129.2 g (dekstroz monohidrat, stokiyometrik), `04` §5.3 BeerSmith 117.3 g (15.195 g/gal/vol susuz glukoza denk). Bizim değer stokiyometrik olarak doğru; karşılaştırma ekranında not edilmeli.
4. **`11` metin–kod farkları.** §1.3 metninde `A ≤ hedef` (tetik zaten geçmiş) durumu yok, kod `None` dönüyor. §2.1 metninde `max(0, …)` yok, kodda var. `kafa_payi_%` hedef ≤ kalıntı ise sıfıra böler. `is_stable` bütün noktalar aynı zamandaysa sıfıra böler.
5. **`fg_reference` sınırı.** `abs(1.016 − 1.012) > 0.004` ifadesi kayan noktada `True` veriyor. Fark tam 4 puan, kural ise "4 puandan büyükse" diyor; yine de tahmine geçiliyor (K1 ile aynı sınıf).
6. **Basınç polinomu ile kalıntı polinomu tam tutarlı değil.** Kalıntı vol'de (0.8615, 20 °C) basınç 0 olması gerekirken polinom 1.76 psi veriyor. Etkisi küçük (0.12 bar); hedef kalıntıya yakınken kafa boşluğu payı sıfır olmaz.
7. **`10` metin–kod farkları.** Kodda "hedefe ulaştı; 48 saat sonra tekrar ölç" durumu var, §1.4'te yok. Kod durgunluk kuralını veri yeterliliğinden önce ve gecikme öncesi okumaları da sayarak çalıştırıyor; metin sırayı söylemiyor. §3.2 metninde `T ≥ 100 °C → Ftemp = 1` kırpması yok (formül 100 °C'de 1.009 verir).
8. **No-chill integrali adım yöntemine bağlı.** Vektör 33.1 dk sol Riemann, 0.5 dk adımla üretilmiş; ince adımda 32.9 dk. Yöntem ve adım spesifikasyona yazılmalı ya da vektöre tolerans verilmeli.
9. **Az okumalı tahmin geç okumalara aşırı duyarlı.** Logaritmik uzayda orijinden geçen en küçük kareler, ağırlığı FG'ye en yakın (göreli hatası en büyük) okumaya veriyor. Örnek (OG 1.050, FG 1.012; 48 sa 1.030 ve 120 sa okuma): 1.014 / 1.015 / 1.016 → bitiş 174.7 / 198.4 / 219.8 saat. Bir puanlık okuma hatası ±22 saat. FG ± σ aralığı bunu büyük ölçüde örtüyor; yine de `SG − FG < 0.003` olan okumaların ağırlığı düşürülebilir.
10. **Kısmi kaynatma IBU varsayımı.** 2h örneğinde kazandaki alfa asit derişimi 560 mg/L; Tinseth'in doğrulandığı aralığın ve muhtemelen çözünürlük sınırının üstünde. Yöntem (kazan gravitesinde kullanım, son hacimde mg/L) tutarlı, ama yüksek hop yükünde sonuç iyimser olabilir; "yaklaşık" notu yerinde olur.
11. **`03` test vektörü T3 iç tutarsızlığı.** Tuz ppm'i 20 L su üzerinden hesaplanıp mash profiline eklenmiş, oysa mash suyu W·t = 14.7 L. Formülü etkilemez, ama örnek fiziksel olarak tutarsız.
12. **`03` uç durumlar ve varsayımlar.** `t = 0` ya da `W = 0` iken BW eğimi 0 ve `ΔpH / m` sıfıra böler; BW sparge modelinde su pH'ı 4.3 ise payda 0. Grist fonksiyonu 14 SRM'de süreksiz (13.99 → 12.09, 14 → 11.34). Mash asidinde ayrışma kesri hedef pH yerine sabit 5.4'te; model 0 sparge hesabı laktik asidin %97 ayrışmasını hesaba katmıyor (%3 eksik). Sparge hedefi mash hedefiyle aynı alan: 5.4 için 2.86 mL, yaygın 6.0 hedefi için 2.15 mL. Tebeşirin tam çözündüğü varsayımı alkaliniteyi olduğundan büyük gösterir.
13. **`09` etiket enerjisi küçük tutarsızlıklar.** Tahmini ABV 0.79661 ile türetilip gram'a 0.78924 ile çevriliyor: 4.056 g; doğrudan `ABW × SG × 0.9982` 4.086 g (%0.75, yaklaşık 1 kJ). `RE × SG`'de su yoğunluğu (0.9982) yok (%0.2). Tahmini değerde ABW'den doğrudan gitmek daha tutarlı; ölçülmüş ABV'de mevcut yol doğru.

---

## "Doğrulanacak" varsayılanlar için değerlendirme

| Varsayılan | Değer | Görüş |
|---|---|---|
| Δ lager / ale | 0.004 / 0.002 | Makul; yaygın pratik FG'ye 2–5 puan kala. Ö1'deki ölü bölgeye dikkat. |
| Lager soğutma hızı | 2 °C/gün | Makul (yaygın aralık 1–3 °C/gün). |
| Lagering kuralı | 8 puan başına 1 hafta | Bilinen kaba kuralla uyumlu. |
| Kısmi priming | 1/3 | Makul (yaygın öneri 1/3–1/2). |
| Cask hedef / mahzen | 1.1 vol / 11–13 °C | Bildiğim değerlerle uyumlu. |
| İsinglass | 2 mL/L | Emin değilim. Hatırladığım üretici aralığı kabaca varil başına 0.5–2 pint (yaklaşık 1.7–7 mL/L); 2 mL/L alt uçta. Üreticiden doğrulanmalı. |
| Cask kondisyon | 12 °C'de 3 gün | Makul ama alt uçta; pratikte 3–7 gün. |
| Gecikme fazı / σ | 12 sa / 0.003 | Makul. |
| LME / DME / bal | 78 / 95 / 75 % | PPG 35.9 / 43.7 / 34.5; yaygın 36 / 44 / 35 ile uyumlu. LME ve bal yoğunluğu 1.42 makul; DME yığın yoğunluğu için emin değilim. |
| Küp τ₂₀ | 4 saat | Kaba ısı dengesi (20 L, yüzey 0.44 m², h = 8–12 W/m²K) 4.4–6.6 saat veriyor. 5 saat daha savunulur; ölçümle doğrulanmalı. |
| Protein | 0.4 g/100 mL | Makul. |

---

## Doğru bulunanlar

**11 (tetikleyici, fıçı, cask)**
- Molar kütleler: dekstroz monohidrat 198.17, sakaroz 342.30, iki heksoz 360.31; CO₂ verimi 2 / 4 / 4 mol.
- `n_bira = Δvol × V / 22.414` ve 1 vol = 1.9635 g/L (ideal gaz, 0 °C, 1 atm).
- Kafa boşluğu: ideal gaz, mutlak basınç = gösterge + 1 atm; süpürülmüş fıçıda başlangıç 1 atm CO₂ çıkarılır, havalıda çıkarılmaz. Birimler (Pa, m³, K) doğru.
- Polinomların fizikle uyumu: 20 °C'de kalıntı 0.861 vol; basınç polinomundan türeyen çözünürlük 0.865 vol/atm; 0 °C 1.71, 12 °C 1.12 vol.
- Balling: 2.0665 g ekstrakt → 0.9565 g CO₂; RE = 0.1808·OE + 0.8192·AE, sabit OE'de ΔRE = 0.8192·ΔAE; °P → g/L için ×10×SG.
- Tetik zamanı formülü, eğim kuralı, rampa süresi, kondisyon süresi.
- Vektörler 1a–1g, 2a–2e, 3a–3f birebir: 74 sa; 73.24 sa; 156 sa; 129.22 / 143.84 / 152.08 / 47.95 / 124.23 g; 1.798 ve 0.744 bar; kalıntı 0.8875, invert 34.94 g; 0.496 ve 0.992 vol; 81.8 mL; 3.96 gün. Firkin 40.91 L.

**10 (elle giriş, ekstrakt, no-chill)**
- Oturtma formülü ve bitiş zamanı; 1a (k 0.02179, 210.8 sa, FG ± σ için 161.8–255.0 sa) ve 1f (k 0.02373, tek tek k oranı 2.15).
- 8.3454 katsayısı; PPG ve puan/(kg/L) değerleri; 2c–2g; 2f ABD birimleriyle çapraz kontrol (50.0 puan).
- Tinseth kullanım formülü; 2h 28.07 ve 44.00 IBU.
- Üst su sıcaklığı enerji dengesi (2i 10.0 °C).
- τ ∝ V^(1/3) ölçeklemesi (toplu ısı modeliyle tutarlı), 40 L için 5.04 sa.
- +20 dk kuralı: 60 → 80 dk +%5.5; 3c 32.29 → 53.62; 3d 60.72; 3e 45.58 IBU.

**03 (su ve pH)**
- İyon matrisi: 16 katsayının hepsi IUPAC molar kütlelerinden hesaplanan değerin %0.02–0.03 altında (sabit bir yuvarlama farkı). Hidrat biçimleri doğru; CaCO₃ ve Ca(OH)₂ için 2 × HCO₃ eşdeğerliği alkalinite açısından doğru.
- Alkalinite 50/61; Kolbach katsayıları 1/1.4 ve 1/1.7 (bağımsız 1.4015 ve 1.6999); RA.
- Ayrışma kesirleri: laktik 0.97197, fosforik 1.01508.
- Asit birim zinciri (mEq → g → mL), laktik yoğunluk %88'de 1.209; fosforik yoğunluk %10 / 75 / 85'te 1.053 / 1.573 / 1.680 (bildiğim değerlerle uyumlu; 0.375 katsayısı %85'te biraz daha yakın).
- Sparge model 0: karbonat sabitleri, toplam karbonat, yük dengesi; 0.00220511 mol/L, 2.858 mL. Model 1: 0.00229262 mol/L, 2.971 mL.
- T1–T6 vektörleri, asit malt yinelemesi (124.83 + 4.05 = 128.88 g; 62.78 g), NNLS sonucu (‖r‖ 18.8197, profil ve KKT koşulları).

**09 (vergi yöntemleri ve etiket enerjisi)**
- AB 1169/2011 Ek XIV katsayıları (alkol 29/7, karbonhidrat ve protein 17/4, yağ 37/9, organik asit 13/3, poliol 10/2.4, lif 8/2).
- TV 1.1–1.3: 186.97 kJ / 44.71 kcal; 203.51 / 48.64; 183.79 / 43.94 (yuvarlanmış değerler tabloyla aynı).
- LPA = V × ABV/100; hL·°P; bant; kademe mantığı (kademe sınırını aşan parti doğru bölünüyor). 1 bbl = 1.17348 hL.
- TV 2.1–2.7: 2178.00; 1678.00; 188.88; 506.40; 59.65; 166.17; 24 980.00.
- Spunding (kapsam dışı, hızlı kontrol): TV 3.1 kapama SG 1.0128; `11` §3.2'nin tersi olarak tutarlı.

---

## Kontrol edilemeyenler

- **Vergi oranları ve mevzuat ayrıntıları:** görev gereği doğrulanmadı. ABV ve °P yuvarlama kuralları için söylediklerim hafızadan; resmi kaynak gerekli.
- **`03`'ün BeerSmith ile birebirliği:** sabitler decompile çıkarımı; programla karşılaştıramadım. Yalnız iç tutarlılık ve kimya kontrol edildi. `03`'te tuz katkıları, alkalinite/RA, mash pH ve asit hesaplarına odaklanıldı; §3 profil zinciri, §7.2 gram dağıtımı ve §7.4 araç davranışı yalnız okundu.
- **Kalıntı CO₂ ve basınç polinomlarının kaynağı:** fiziksel değerlerle uyumlu, ama uyum verisini ve geçerlilik aralığını (0 °C altı, 3.5 vol üstü) doğrulayamadım.
- **No-chill modeli ve küp τ değeri:** ölçülmüş IBU ya da soğuma verisi yok; yalnız iç tutarlılık ve kaba ısı dengesi yapıldı.
- **Malt tampon kapasitesi aralığı (Ö5):** literatürden hatırladığım değer; ölçüm verisiyle doğrulanmalı.
- **İsinglass dozu, invert şeker kuru maddesi, DME yığın yoğunluğu:** ürün verisi gerekli.
- **`02` geç ekstrakt segmentleri ve `07` fermantasyon modeli:** `10` ve `11` bunlara atıf yapıyor; bu görevde incelenmedi.
