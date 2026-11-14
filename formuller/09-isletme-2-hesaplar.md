# İşletme 2. sürüm hesapları — spesifikasyon

`06-eksik-hesaplar.md` listesindeki **İşletme 2. sürüm** hesapları: etiket enerjisi, vergi ve beyan, spunding. Referans uygulama ve test vektörleri: `testler/09_isletme_2_tv.py` (`python3 -I testler/09_isletme_2_tv.py`, yalnız standart kütüphane).

İşaretler (07 ve 08 ile aynı):
- **[kesin]**: tanım veya fizik/kimya.
- **[mevzuat]**: yasa veya yönetmelikteki kural.
- **[endüstri]**: demleme literatüründe yerleşik yöntem.
- **[tasarım]**: bizim seçimimiz.
- **[doğrulanacak]**: varsayılan değer, kontrol edilecek.

**Önemli:** Vergi oranları ve etiket kuralları ülkeye göre değişir ve her yıl güncellenir. Motor yalnız **yöntemleri** içerir. Oranlar, bantlar, eşikler ve yuvarlama kuralları **veri tablosundadır**; kaynağı ve geçerlilik tarihiyle saklanır ve uygulama güncellemesi beklemeden yenilenir. Bu dosyadaki oranlar örnektir; beyanname için kullanılmadan önce resmi kaynaktan doğrulanmalıdır.

---

## 1. Etiket enerjisi (AB)

Etiket ekranı bu hesabı kullanır.

### 1.1 Dönüşüm katsayıları [mevzuat: AB 1169/2011, Ek XIV]

| Bileşen | kJ/g | kcal/g |
|---|---|---|
| Alkol (etanol) | 29 | 7 |
| Karbonhidrat (polioller hariç) | 17 | 4 |
| Protein | 17 | 4 |
| Yağ | 37 | 9 |
| Organik asit | 13 | 3 |
| Poliol | 10 | 2.4 |
| Lif | 8 | 2 |

Biradaki yağ, organik asit, poliol ve lif ihmal edilebilir düzeydedir. Laboratuvar değeri girilirse hesaba eklenir.

### 1.2 Bileşenler [kesin, tasarım]

```
alkol (g/100 mL)          = ABV · 0.78924                 (ölçülmüş ABV; etanol yoğunluğu, 20 °C)
                          = ABW · SG_son · 0.9982         (tahmini ABV; aşağıya bakın)
gerçek ekstrakt (g/100 mL) = RE (°P, g/100 g) · SG_son        (RE: 01 §6.1)
karbonhidrat (g/100 mL)   = gerçek ekstrakt − protein − (kül, girilmişse)
enerji                    = Σ bileşen · katsayı
```

- **ABV:** ölçülmüş değer varsa o kullanılır (etikette beyan edilen ABV), yoksa 01 §6.3'teki tahmin.
- **Alkol gramı, iki yol [tasarım; F1 kararı 13]:**
  - ölçülmüş ABV: `alkol (g/100 mL) = ABV · 0.78924` (hacim yüzdesi × etanol yoğunluğu; doğru yol);
  - tahmini ABV: `alkol (g/100 mL) = ABW · SG_son · 0.9982` (ABW: 01 §6.2'deki g/100 g, doğrudan). Eski yol (tahmini ABV 0.79661 ile türetilip 0.78924 ile gram'a çevrilir) pilsner örneğinde 4.056 g, yeni yol 4.086 g verir (%0.75, yaklaşık 1 kJ); vektör 1.1 ve 1.2 yeni yola göre.
  - `RE × SG` çarpımında su yoğunluğu (0.9982) yoktur (%0.2); karar kapsamı dışında, değişmedi.
- **Protein:** laboratuvar değeri yoksa varsayılan 0.4 g/100 mL [doğrulanacak; tahıl faturasına göre 0.2–0.6 arası değişir].

### 1.3 Yuvarlama ve beyan [mevzuat, doğrulanacak]
- Enerji kJ ve kcal olarak, **tam sayıya** yuvarlanarak; besin değerleri 0.1 g'a yuvarlanır. AB Komisyonu'nun tolerans ve yuvarlama kılavuzu kontrol edilecek.
- **Hangi beyan:** %1.2'nin üstündeki içeceklerde besin değeri beyanı zorunlu değildir; yalnız enerji beyan edilebilir. Hangi beyanın zorunlu olduğu ülkeye göre etiket şablonunda tutulur.
  - Türkiye Gıda Etiketleme ve Tüketiciyi Bilgilendirme Yönetmeliği AB ile büyük ölçüde uyumlu, ama ayrıca kontrol edilecek.
  - ABD (TTB) kalori beyanı farklı kurallara bağlıdır; 01 §7'deki 12 oz porsiyonlu hesap orada kullanılır.

---

## 2. Vergi ve beyan

Mevzuat raporu bu hesabı kullanır.

### 2.1 Ne zaman vergilenir [tasarım, mevzuata göre veri]
- Verginin doğduğu an ülkeye göre değişir: ambalajlama, antrepodan çıkış ya da satış. Bu an **veri tablosunda** tanımlanır.
- Motor, ambalaj ve çıkış kayıtlarından ilgili dönemin vergilenecek hacmini toplar.
- İmha edilen, iade gelen ya da laboratuvara giden hacim ayrı satırlarda raporlanır. Vergiden düşülüp düşülmeyeceği yine veri tablosundaki kurala bağlıdır.

### 2.2 Yöntemler [tasarım; her biri bir mevzuat tipini karşılar]

| Yöntem | Formül | Kullanan örnek ülke |
|---|---|---|
| `hacim` | oran × hL (oran birimi **hL başına**; litre başına oran × 100 olarak girilir) | Türkiye ÖTV (maktu tutar; birim doğrulanacak), ABD federal (varil başına) |
| `saf_alkol` | oran × V × ABV/100 (litre saf alkol, LPA); ABV bandına göre oran; küçük üretici indirimi LPA başına | Birleşik Krallık |
| `hl_plato` | oran × hL × °P (°P tam sayıya kesilebilir) | Almanya, Avusturya |
| `plato_bandi` | °P bandına göre hL başına oran | İsviçre |
| `kademeli_hacim` | yıllık kümülatif hacme göre kademeli oran; yıl içindeki önceki çıkışlar hesaba katılır | ABD federal küçük bira fabrikası oranı |

**Ortak kurallar:**
- **Ülke birden çok yöntemi birleştiriyorsa** (ör. bant + kademe + indirim), yöntemler zincir olarak uygulanır. Zincirin sırası veri tablosundadır.
- **ABV kaynağı:** ölçülmüş (laboratuvar) değer; ölçüm yoksa tahmini, ama rapor bunu "tahmini" diye işaretler. Bazı ülkelerde beyan edilen ABV kullanılır; hangisi olduğu veri tablosundadır.
- **°P kaynağı:** ölçülmüş başlangıç ekstraktı (Stammwürze), yani OG'nin °P karşılığı.
- **Para birimi ve yuvarlama:** veri tablosundan; kuruş ve sent yuvarlaması satır bazında ya da toplamda (varsayılan mod: yarım yukarı).
- **Ondalık tip:** tutarlar, oranlar, hacim, ABV ve °P ondalık tiple (Decimal) hesaplanır ve karşılaştırılır; ikili kayan nokta kullanılmaz (`round(2.675, 2) = 2.67` tuzağı; bant ve `floor(°P)` sınırında yanlış banda düşme).
- **Girdi yuvarlama kuralı (veri tablosu, yöntem başına):** ABV ve °P'nin vergi için kaç haneye (`input_dp`) ve hangi yönde (yukarı, aşağı/kesme, en yakın) yuvarlanacağı. **Varsayılan "yuvarlama yok"**: satır bunu açıkça yazar (`input_dp = None`; 10.05 °P, ≤10.0 bandının dışında kalır). **Kuralı tanımsız satırda motor hata verir** (ABV ya da °P kullanan yöntemler: `saf_alkol`, `hl_plato`, `plato_bandi`; `ValueError`, vektör 2.13); sessiz varsayılan yoktur. İngiltere'de ABV'nin bir ondalığa aşağı kesildiğini hatırlıyorum, emin değilim **[doğrulanacak]** (F1 kararı 4).
- **Satır durumu:** her ülke satırı `doğrulanacak` başlar. Yalnız resmi kaynak (HMRC/gov.uk, BierStG, TTB, Resmi Gazete) ve kaynak tarihi birlikte kaydedilince `doğrulandı` olur; resmi olmayan kaynaklar (haber, hesaplama siteleri) satırı doğrulanmış yapmaz (vektör 2.14). Bu dosyadaki örnek tablonun hiçbir satırı `doğrulandı` değildir.

**Karar (T1.16 C): satır durumu kodu.** Motor `RowStatus::{Unverified, Verified}` döndürür; "doğrulanacak" / "doğrulandı" metinleri gösterimdir (vektör metni enum eşlemesiyle karşılaştırır).
- **Bant sınırı türü (veri tablosu):** üst sınırın banda dahil olup olmadığı. Varsayılan: `saf_alkol` dışlayıcı (`ABV < üst`), `plato_bandi` kapsayıcı (`°P ≤ üst`); her satırda açıkça yazılır.
- **Karar (T1.12 D1): oranlar ve birim çarpanları sonlu ondalıktır.** Oran tablosundaki her oran ve birim çarpanı en çok 15 anlamlı basamaklı sonlu bir ondalıktır ve çarpımla uygulanır; bölmeyle türetilmez (`3.50 / 1.17347765` sonsuz ondalık verir, 28 basamakta kesilir; Python ve Rust son basamakta ayrışır). ABD bira varili = **1.17347765 hL** (31 ABD galonu, kesin). Varil başına oran, tabloya hL başına sonlu ondalık olarak yazılır (ör. $3.50/bbl → 2.9826 $/hL, $16/bbl → 13.6347 $/hL; yuvarlama tablo kaydında belgelenir). Çarpımlar 28 basamağı aşmadığından Rust sonucu Python'la ölçeğiyle birlikte aynıdır.
- **Karar (T1.12 D8): bant ve kademe tabloları doğrulanır.** Üst sınırlar kesin artan, açık uç (`None`) yalnız sonda, oran ≥ 0 olmalıdır; aksi hâlde motor hata verir (Rust `Error::InvalidBands`, referans `ValueError`), sessizce yanlış vergi hesaplamaz. Boş liste "bant yok" demektir (geçerli).
- **Karar (T1.12 D5): sonuç metni düz gösterimdedir** (bilimsel gösterim yok) ve **eksi sıfır içermez** (`-0.00` yerine `0.00`; referans sonuca `+ 0` uygular). Toplama ve çıkarmada sonucun ölçeği işlenenlerin büyük ölçeğidir (`4 − 0.00 = 4.00`; T1.12 D4).
- **Karar (T1.12 D2/D13):** `spund` sonlu olmayan girdi ya da FG ≤ 0'da hata verir (`Error::InvalidInput`). Tank basınç sınırı karşılaştırması `1e-9` psi mutlak toleranslıdır (`psi > sınır + 1e-9`); sınıra eşit ayar engellenmez.
- **Uç durumlar:** son bant/kademe açık uçlu olmalıdır (üst sınır `None`); aksi hâlde bant bulunamazsa ya da son kademenin üstünde hacim kalırsa motor hata verir, sessizce 0 vergi hesaplamaz. İndirim oranı oranı aşamaz (`max(oran − indirim, 0)`); satır vergisi hiçbir zaman negatif olmaz.

### 2.2a Yöntem zinciri, yürürlük ve yıl başından hacim [tasarım, T2.8]

Motor oran tablosu içermez; veri tablosu satırlarını **girdi** olarak alır. Girdi modeli DB satırlarıyla bire birdir (`tax_rate_tables` + `tax_rate_bands`, adr/0007 T2.7 notu 6): `rate_class`, `chain_position`, yürürlük `[başlangıç, bitiş)`, `method`, oran, bantlar, kademeler, bant sınırı türü, girdi yuvarlaması, indirim (`discount_rate` + `discount_conditions`).

- **Tablo seçimi (yürürlük tarihi):** motor adayların içinden `rate_class` eşleşen ve dönemin **ilk gününde** yürürlükte olanları alır (`başlangıç ≤ gün < bitiş`; bitiş günü dahil değildir, yürürlük günü yeni sürümdür). Dönemin **son gününde** seçilen küme farklıysa (bir tablo dönem içinde başlıyor ya da bitiyor) motor **hata** verir: dönem bölünür, sessizce tek oran uygulanmaz. Seçim boşsa, `chain_position` < 1 ya da aynı konumda iki satır yürürlükteyse, yürürlük aralığı boşsa ya da tarih biçimi `YYYY-AA-GG` değilse hata. Aday sırası önemsizdir.
- **Zincir:** seçilen satırlar `chain_position` artan sırayla adımlardır. **Karar (T2.8): adımlar toplanır.** Her adım kendi yöntemiyle (§2.2; bant, kademe ya da tek oran) bir **bileşen vergisi** hesaplar; satır vergisi bileşenlerin toplamıdır (yuvarlanmamış). Sıra çıktı sırasını ve raporu belirler; bileşenler birbirinin girdisi değildir. Para yuvarlaması bütün zincir için tektir: seçilen satırlar para birimi, vergi doğuş anı, düşülecek türler ve para yuvarlama kuralında (hane, mod, satır/toplam) **aynı** olmalıdır, değilse servis hata verir. Satır "doğrulandı" yalnız seçilen **her** satır doğrulandıysa olur.
- **İndirim ve koşulları:** indirim yalnız `saf_alkol` adımında vardır (LPA başına tutar; oranı aşamaz, §2.2). `discount_conditions` ile koşullanabilir; verilen **her** koşul sağlanmalıdır, biri sağlanmazsa indirim uygulanmaz ve adım `indirim_uygulandi = false` döner: `abv_alti` (girdi yuvarlaması sonrası ABV **<** sınır) ve `ytd_hl_alti` (satır başındaki `ytd_hl` **<** sınır, hL). Koşul tanımlı ama girdisi yoksa (ABV) ya da indirim `saf_alkol` dışı bir yöntemdeyse ya da negatifse hata. İndirim satırı **bölmez**: eşik satır başındaki `ytd_hl` ile karşılaştırılır (kademeler ise satırı böler).
- **`ytd_hl` (yıl başından hacim):** aynı oran sınıfında, yılın başından dönemin başına dek **vergilenen** hacim, hL (≥ 0). Çağıran (api) ambalaj ve transfer/çıkış kayıtlarından toplar; tabloda tutulmaz. Dönemin satırları verilen sırayla işlenir ve her satırın vergilenecek hL'si sonrakinin başlangıcına eklenir: `ytd_{i+1} = ytd_i + litre_i / 100`. Kademe ve `ytd_hl_alti` satır başındaki değeri okur. Satır sırası hangi ürünün ucuz kademeye düştüğünü belirler; sıra çağıranındır. Sıfır hacimli satır `ytd`'yi değiştirmez.
- **Birim eşlemesi (DB → motor):** `input_rounding` `none` → yuvarlama yok, `up` → `ROUND_UP`, `down` → `ROUND_DOWN`, `nearest` → `ROUND_HALF_UP` (ABV ve °P ≥ 0 olduğundan `up` = `ROUND_CEILING`); `rounding_mode` `half_up`, `half_even`, `up`, `down` → `ROUND_HALF_UP`, `ROUND_HALF_EVEN`, `ROUND_UP`, `ROUND_DOWN`; `bounds_inclusive` → bant sınırı türü; `base_rate` → tek oran; `tax_rate_bands.upper_bound` boş → açık uç.
- **Kapsam dışı (açık):** yüzde indirimi (İsviçre: her 1 000 hL için %1), bir adımın çıktısını sonrakine veren (art arda) bileşim, üretim hacmine bağlı UK küçük üretici formülü. Bunlar gerçek oranlar doğrulanınca ayrı yöntem olarak eklenir (§5).

### 2.3 Rapor çıktısı [tasarım]
- Dönem (ay veya çeyrek), ürün, ambalaj tipi (fıçı/şişe/kutu) ve vergi satırı bazında hacim, ABV, °P, LPA, oran ve tutar.
- **Karar (T1.17): LPA ve uygulanan oran motordan gelir.** `tax_line_detail` vergiyle birlikte LPA'yı (yalnız `saf_alkol`; diğer yöntemlerde boş) ve satıra uygulanan oranı döndürür. `saf_alkol` oranı indirim sonrası net orandır (`vergi = LPA × oran`); `hacim` ve `hl_plato` oranı 0'a kenetlenir; `plato_bandi` bant oranıdır; `kademeli_hacim` birden çok oran uyguladığı için oran boştur. Rapor satırı bu iki değeri olduğu gibi gösterir, istemci yeniden hesaplamaz.
- **CSV/PDF:** resmi beyanname formatını birebir üretmek ilk sürümde hedef değil. Rapor, beyannameye girilecek sayıları verir.

### 2.4 Örnek oran tablosu [doğrulanacak; arama tarihi 31 Ekim 2026]

| Ülke | Yöntem | Değer (örnek) | Kaynak ve not |
|---|---|---|---|
| Türkiye | `hacim` | Ocak 2026: 12,49 TL (bira) | Haber ve hesaplama sitelerinden. **Birim doğrulanacak** (litre başına mı, litre × % alkol başına mı). Temmuz 2026'da alkollü içkilere 6 aylık ÜFE (%16,09) uygulandı; biranın yeni tutarı resmi kaynakta bulunamadı. Resmi Gazete'deki Cumhurbaşkanı Kararı ve Bakanlık tablosundan alınacak. |
| Birleşik Krallık | `saf_alkol` + bant + küçük üretici indirimi + fıçı (draught) indirimi | 1 Şub 2026: %3.4 fıçı bira tam oran £8.58/LPA; küçük üretici indirimiyle £6.59 | gov.uk. Küçük üretici indirimi %8.5 ABV altında ve yıllık 4 500 hL saf alkol üretim sınırında; indirim formülü üretim hacmine bağlı, gov.uk kılavuzundaki hesap birebir uygulanmalı. 1 Şub 2027 için RPI (%4.55) artışı önerisi var. |
| Almanya | `hl_plato` + küçük bira fabrikası kademesi | genel oran 0.787 €/hL/°P | BierStG §2. Küçük fabrikalar için 1 000 hL adımlarla azalan kademeli oranlar var; tablo yasa metninden alınacak. °P'nin kesirli kısmının nasıl ele alındığı yasa metninden doğrulanacak. |
| Avusturya | `hl_plato` | 2 €/hL/°P | BierStG 2022 §3; küçük fabrika indirimleri ayrıca. |
| İsviçre | `plato_bandi` + küçük fabrika indirimi | ≤10.0 °P 16.88; 10.1–14.0 °P 25.32; ≥14.1 °P 33.76 CHF/hL | BAZG. 55 000 hL altındaki fabrikalarda her 1 000 hL için %1 indirim (en fazla %40). |
| ABD (federal) | `kademeli_hacim` | ilk 60 000 varil $3.50/bbl, 2 milyon varile kadar $16/bbl, genel oran $18/bbl | 27 CFR 25.152 ve TTB. Tablo 2021 arşivinden; güncelliği TTB sitesinden doğrulanacak. Eyalet vergileri ayrı. |

Kaynaklar (arama 31 Ekim 2026):
- Türkiye: [KPMG Vergi](https://kpmgvergi.com/yayinlar/mali-bultenler/gumruk/otv-i-ve-otv-iii-sayili-listede-yer-alan-esyalara-iliskin-tutarlar-yeniden-belirlenmistir/3430), [PwC Türkiye bülteni](https://www.pwc.com.tr/tr/Hizmetlerimiz/vergi/dolayli-vergi/bultenler/kdv-%C3%B6tv-b%C3%BCltenleri/2026/kdv-otv-b%C3%BClteni-15.pdf), [AA](https://www.aa.com.tr/tr/ekonomi/akaryakit-sigara-ve-alkollu-iceceklerde-otv-artisi-sinirlandirildi/3786557).
- Birleşik Krallık: [gov.uk SPR kılavuzu](https://gov.uk/guidance/how-to-work-out-your-alcohol-duty-rates-if-youre-eligible-for-small-producer-relief), [gov.uk 2027 değişiklikleri](https://www.gov.uk/government/publications/changes-to-alcohol-duty-rates-from-1-february-2027/alcohol-duty-uprating-and-small-producer-relief-changes).
- Almanya, Avusturya, İsviçre: [gesetze-im-internet.de BierStG §2](https://www.gesetze-im-internet.de/bierstg_2009/__2.html), [jusline BierStG 2022 §3](https://www.jusline.at/gesetz/BierStG_2022/paragraf/3.pdf), [BAZG](https://www.bazg.admin.ch/dam/de/sd-web/uLGL5heLKF7x/biersteuer_d.pdf).
- ABD: [27 CFR 25.152](https://www.customsmobile.com/regulations/27/25.152), [Brewers Association](https://www.brewersassociation.org/?p=112442).

---

## 3. Spunding

Tank detayı ve araçlar bu hesabı kullanır. Fermantasyonun son birkaç puanında tank basınç vanasıyla kapatılır; kalan fermantasyonun ürettiği CO₂ birayı karbonatlar.

### 3.1 Gereken CO₂ [kesin, 04 ile]

```
kalıntı CO₂ (vol)  = 04 §3.2 formülü, fermantasyon sıcaklığında
gereken (vol)      = hedef vol − kalıntı
gereken (g/L)      = gereken vol · 1.9635                  (1 vol CO₂ = 44.01/22.414 = 1.9635 g/L, ideal gaz, 0 °C 1 atm)
```

**Tek sabit [F1 kararı 9]:** 09, 10 ve 11'in hepsi 1.9635 g/L/vol kullanır (eski 1.977 gerçek gaz değeriydi; kalıntı vol ile birlikte kullanılan ideal gaz sabiti seçildi). 04 §3.7'deki BeerSmith değerleri (1.9642 / 1.96042) yalnız "BeerSmith uyumu" notudur, motor 1.9635 kullanır.

### 3.2 Kapama gravitesi [endüstri: Balling]
2.0665 g ekstrakt → 1 g alkol + 0.9565 g CO₂ + 0.11 g maya.

```
fermente gerçek ekstrakt (g/L) = gereken g/L / (0.9565 / 2.0665)
ΔRE (°P)                       = bu değer / (10 · SG_son)
ΔAE (°P)                       = ΔRE / 0.8192                (01 §6.1'deki RE–AE bağıntısından)
kapama gravitesi               = ToSG(ToPlato(FG_beklenen) + ΔAE)
```

### 3.3 Vana ayarı [kesin, 04 §3.5]

```
ayar basıncı = CO2_pressure(T_fermantasyon, hedef vol)   (psig; bar = psi · 0.0689475729316836; 1 psi = 6894.757293168361 Pa tam değerdir, motorda tek sabit `units::PSI_BAR`)
```

### 3.4 Uyarılar [tasarım]
- **Tank basınç sınırı:** ayar basıncı tank kaydındaki çalışma basıncını aşarsa engellenir. Önerilen çözüm: daha düşük hedef ya da daha soğuk fermantasyon.
- **FG belirsizliği:** kapama gravitesi beklenen FG'ye bağlıdır. Fermantasyon tahmini (07 §1) güven aralığı genişse (aralık genişliği > 0.002 SG, tam sayı birimde 200; varsayılan, doğrulanacak) "1–2 puan erken kapatma riski" gösterilir.
- **İzleme:** basınç sensörü varsa gerçek basınç ve sıcaklıktan anlık CO₂ hacmi hesaplanır (04 §3.5'in tersi, sayısal çözüm) ve grafikte gösterilir.

---

## 4. Test vektörleri

`python3 -I testler/09_isletme_2_tv.py` çıktısından. Vergi vektörleri **örnek oranlarla** yöntemleri test eder; gerçek oran değildir.

| No | Girdi | Beklenen |
|---|---|---|
| 1.1 | Pilsner OG 1.048, FG 1.009, protein 0.4 | ABV 5.14, alkol 4.09 g (ABW yolu), gerçek ekstrakt 4.08 g, karbonhidrat 3.7 g; **188 kJ / 45 kcal** /100 mL |
| 1.2 | Citra Pale Ale OG 1.052, FG 1.011 | ABV 5.41; alkol 4.31 g; 204 kJ / 49 kcal |
| 1.3 | 1.1 ama ölçülen ABV %5.0 | alkol 3.95 g; 184 kJ / 44 kcal |
| 2.1 | 20 hL, %5.0, `saf_alkol` 21.78/LPA | 100 LPA → 2 178.00 |
| 2.2 | aynı, bantlar (<3.5: 9.27, <8.5: 21.78, diğer (üst sınır `None`): 29.54), indirim 5.0/LPA | 1 678.00 |
| 2.3 | 20 hL, 12.6 °P, `hl_plato` 0.787, °P kesilir | 20 · 12 · 0.787 = 188.88 |
| 2.4 | 20 hL, 12 °P, `plato_bandi` (≤10: 16.88, ≤14: 25.32, diğer (`None`): 33.76) | 506.40 |
| 2.5 | 20 hL, `kademeli_hacim` (60 000 bbl = 70 408.659 hL'ye kadar 2.9826/hL [≈ $3.50/bbl], sonra 13.6347/hL [≈ $16/bbl]), yıl başı | 59.65 |
| 2.6 | aynı, bu yıl eşiğe 10 hL kalmış | 10 hL @2.9826 + 10 hL @13.6347 = 166.17 |
| 2.7 | 20 hL, `hacim` 12.49/L (= 1249 PB/hL) | 24 980.00 |
| 2.8 | 20 hL, `plato_bandi` (2.4 bantları), 10.0 °P; 10.05 °P (`input_dp = None`); 10.05 °P ve `input_dp` = 1 (yarım yukarı) | 337.60 (kapsayıcı sınır); 506.40; 506.40 |
| 2.9 | 20 hL, ABV 3.5, `saf_alkol` bantları (2.2), dışlayıcı sınır | 70 LPA × 21.78 = 1 524.60 (ikinci bant) |
| 2.10 | 20 hL, %5.0, `saf_alkol` oran 4.0, indirim 5.0/LPA | 0.00 (negatif değil) |
| 2.12 | Girdi yuvarlama: 2.675, `input_dp` = 2, yarım yukarı | 2.68 (ikili kayan noktada 2.67 çıkardı) |
| 2.11 | `kademeli_hacim`, tek kademe 100 hL'ye kadar 3.0, yıl içi 95 hL, yeni 20 hL | hata (15 hL kademesiz; F1 kararı 5: son kademe açık uçlu girilir, kapalıysa aşım hata) |
| 2.13 | 20 hL, %5.0, `saf_alkol` oran 21.78, girdi yuvarlama kuralı yazılmamış | hata (kural tanımsız) |
| 2.13b | aynı, `input_dp = None` (açıkça yuvarlama yok) | 2 178.00 |
| 2.15 | Zincir (sentetik oran): 1 `saf_alkol` bantlı + indirim 5.0 (koşul: ABV < 8.5 ve `ytd_hl` < 100), 2 `kademeli_hacim` (100 hL: 3.0, 1000 hL: 5.0, sonra 7.5), 3 `hacim` 1.5; 20 hL, %5.0 | `ytd` 0: 1 678.00 + 60.00 + 30.00 = **1 768.00**; `ytd` 90 (kademe 90 → 110 hL bir satırda): 1 788.00; `ytd` 100: indirim yok (< sınır), 2 308.00; ABV 8.5: indirim yok, üçüncü bant |
| 2.16 | Aynı zincir, üç satır (10 hL, 10 hL, 30 hL %4.0), `ytd` 85 | satır başları 85 → 95.0 → 105.0 hL; vergiler 884.00, 894.00 (kademe geçişi), 2 808.60 (indirim bitti); toplam 4 586.60 |
| 2.17 | Yürürlük: `hacim` 1000.0 `[2026-01-01, 2026-07-01)`, 1200.0 `[2026-07-01, süresiz)`; 20 hL | 1 Haz–30 Haz: 20 000.00; **1 Tem**–31 Tem: 24 000.00; 15 Haz–15 Tem ve yürürlük öncesi dönem: hata |
| 2.14 | `row_status()`; `row_status("gov.uk SPR kılavuzu", "2026-10-31")` | doğrulanacak; doğrulandı |
| 3.1 | hedef 2.5 vol, 12 °C, FG 1.010 | kalıntı 1.12 vol; 2.71 g/L gerekli; **kapama SG 1.0128** (2.8 puan önce); ayar 19.5 psi = 1.35 bar |
| 3.2 | hedef 2.6 vol, 18 °C, FG 1.012 | kalıntı 0.92 vol; 3.31 g/L; kapama SG 1.0154 (3.4 puan); 27.1 psi = 1.87 bar |

## 5. Açık noktalar

- **Türkiye ÖTV:** biranın güncel (Temmuz 2026 sonrası) maktu tutarı ve tutarın birimi Resmi Gazete'den alınacak. Beyanname dönemi ve verginin doğduğu an (ÖTV Kanunu) da tanımlanacak.
- **Zincir (T2.8):** yüzde indirimi (İsviçre) ve art arda bileşim (§2.2a) gerçek oran tabloları doğrulanınca eklenir; vergi oranı tohumu (`tax_rate_tables` satırları) resmi kaynaktan, `unverified` olarak yazılır.
- **Birleşik Krallık:** küçük üretici indirimi bir formülle hesaplanır, sabit bir indirim değildir. gov.uk kılavuzundaki hesap ayrı bir yöntem olarak eklenecek.
- **Almanya:** kademeli oran tablosu ve °P'nin kesirli kısmının ele alınışı yasa metninden alınacak.
- **Etiket:** protein varsayılanı ve AB yuvarlama kılavuzu kontrol edilecek. Türkiye yönetmeliğiyle karşılaştırılacak.
- **Spunding:** 04'teki kalıntı CO₂ ve basınç polinomları topluluk uyumları [tahmin]. Basınçlı fermantasyonda ölçülen basınçla karşılaştırılacak.
