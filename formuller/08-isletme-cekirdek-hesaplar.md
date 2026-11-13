# İşletme çekirdek hesapları — spesifikasyon

`06-eksik-hesaplar.md` listesindeki **İşletme çekirdek** hesapları. Bunlar BeerSmith'ten çıkarılmadı; demleme endüstrisinin yerleşik tanımları ve bizim tasarım kararlarımız. Referans uygulama ve test vektörleri: `testler/08_isletme_cekirdek_tv.py` (`python3 -I testler/08_isletme_cekirdek_tv.py`, yalnız standart kütüphane).

**İşaretler:**
- **[kesin]**: tanım veya fizik/kimya.
- **[endüstri]**: demleme literatüründe yerleşik yöntem.
- **[tasarım]**: bizim seçimimiz.
- **[doğrulanacak]**: varsayılan değer, kontrol edilecek.

**Birimler:** L, hL, kg, °P, SG. Para birimi soyut (çalışma alanının para birimi). °P → SG dönüşümü 01 §1.2'deki `ToSG`.

**Karar (T1.12 C2, C6): ondalık tip ve yuvarlama.** Stok miktarı ve para `Decimal`dir (referans: Python `decimal`; Rust: `rust_decimal`). Miktar yuvarlanmaz, tam ondalık toplanır ve çıkarılır; 0.1 + 0.2 kg lot ile 0.3 kg ihtiyaç tam biter (toz satır ve hayali eksik oluşmaz). Para: her maliyet kalemi 2 ondalığa **ROUND_HALF_UP** (yarım yukarı) yuvarlanır; toplam = **yuvarlanmış kalemlerin toplamı**; litre maliyeti = toplam / ambalajlanan litre, 3 ondalığa ROUND_HALF_UP. Ambalajlanan hacim ya da miktar ≤ 0 ise hata. Birim fiyat (lot, kargo dahil) yuvarlanmaz. Fiziksel büyüklükler (L, kg, kWh, °P) `f64`.

**Karar (T1.12 C9): ham değer.** Harmanlama, malt analizi ve renk gösterim için yuvarlama yapmaz (özellikle tarife giren verim ve potansiyel); yuvarlama sunumdadır. §6 tablosu gösterim yuvarlamasıyla yazılmıştır.

**Karar (T1.12 C1, C7): sürekli eşikler** `x < eşik − 1e-9` biçiminde (00-OKU-BENI): diastatik güç, canlılık, gün sayısı, harmanlama hedef aralığı.

---

## 1. Maliyet

Raporlar, stok ve parti özeti bu hesabı kullanır. Ev modunda da açılabilir; orada lot yoktur, malzemenin son alış fiyatı kullanılır.

### 1.1 Lot birim fiyatı [tasarım]

```
birim fiyat = (fatura birim fiyatı · miktar + kargo, gümrük, nakliye payı) / miktar
```

- Bir faturadaki ortak kargo bedeli, kalemlere **tutar oranında** dağıtılır.
- Fiyat mal kabulde lota yazılır ve sonradan değişmez. Düzeltme gerekirse yeni bir stok hareketi olarak kaydedilir, denetim kaydına geçer.

### 1.2 FIFO tüketim [endüstri]
- Malzeme ihtiyacı lotlardan **kabul sırasına göre** düşülür.
- Parti başlatmada belirli bir lot seçilmişse önce o lot kullanılır. Örnek: Citra L-2411, alfa değeri tarife göre ayarlanmış lot.
- **Yuvarlama (C2, C6):** her tüketim satırının maliyeti `miktar · birim fiyat` 2 ondalığa ROUND_HALF_UP; toplam maliyet satır maliyetlerinin toplamıdır (ham çarpımların toplamı değil). Örnek: 1.5 kg · 1.25 = 1.875 → satırda 1.88.
- Stok yetmezse eksik miktar ayrıca raporlanır. Eksik kısmın maliyeti, malzemenin **son alış fiyatıyla** tahmin edilir ve "tahmini" diye işaretlenir.
- **Plan maliyeti:** parti başlatmada, o anki stokla FIFO'ya göre öngörülür.
- **Gerçek maliyet:** stok hareket defterindeki gerçek tüketimden hesaplanır. İkisi parti özetinde yan yana gösterilir.

### 1.3 Su ve enerji [tasarım]

```
su        = toplam su (L) / 1000 · m³ fiyatı                    (toplam su: 05 §5.8)
elektrik  = kWh · kWh fiyatı
  kWh     = Σ ısınma ısısı (kJ) / η / 3600  +  kaynatma gücü (kW) · kaynatma süresi (saat)
gaz       = Σ ısı (kJ) / (η · Hu) · m³ fiyatı
```

- Isınma ısısı 07 §3.1'deki formülle hesaplanır: strike suyu, adım ısıtmaları, kaynamaya ısınma.
- **Hu (alt ısıl değer):** doğal gazda 34 500 kJ/m³ [doğrulanacak]. Dağıtım şirketinin faturasındaki değer girilebilir.
- Soğutma suyu ayrıca girilirse su kalemine eklenir. Daldırma soğutucuda debi × süre.
- Enerji ve su varsayılan olarak **kapalı**; işletme isterse açar. Yalnız malzeme maliyeti de anlamlı bir rapordur.

### 1.4 Ambalaj ve toplam [tasarım]

```
ambalaj   = Σ ambalaj birimi sayısı · birim maliyeti     (kutu/şişe + kapak + etiket + koli; fıçıda yalnız sarf)
toplam    = malzeme + su + enerji + ambalaj
litre     = toplam / ambalajlanan hacim
birim     = toplam / ambalaj birimi sayısı
```

- **Yuvarlama (C6):** malzeme, su, enerji, ambalaj kalemleri ayrı ayrı 2 ondalığa ROUND_HALF_UP; `toplam` bu dört yuvarlanmış kalemin toplamı; `litre = toplam / ambalajlanan hacim` 3 ondalığa ROUND_HALF_UP. Ambalajlanan hacim ≤ 0 ise hata.
- Litre maliyeti **ambalajlanan hacme** bölünür. Bu yüzden fire (trub, transfer, dolum kaybı) maliyete doğal olarak yansır.
- Rapor ayrıca "kayıp maliyeti" gösterir: `(şıra hacmi − ambalajlanan hacim) · şıra litre maliyeti`.
- İşçilik ve genel gider ERP kapsamında olduğu için **yoktur**. İstenirse "parti başı sabit gider" diye tek bir elle girilen kalem eklenebilir.

---

## 2. Harmanlama

Tank hareketi ekranındaki "Birleştir" işlemi bu hesabı kullanır. Bir partiyi suyla seyreltmek de iki bileşenli harmanlamadır; bileşenlerden biri su olur (ABV 0, IBU 0, renk 0).

### 2.1 Karışım değerleri

| Değer | Kural | İşaret |
|---|---|---|
| Hacim | `V = Σ Vᵢ` (transfer kayıpları önce düşülür) | [kesin] |
| ABV | `Σ Vᵢ·ABVᵢ / V` | [yaklaşık] alkol–su karışımındaki hacim büzülmesi ihmal edilir (bu seviyelerde < %0.1) |
| IBU | `Σ Vᵢ·IBUᵢ / V` | [kesin] IBU derişimdir (≈ mg/L izo-alfa) |
| Renk (SRM, EBC) | `Σ Vᵢ·Rᵢ / V` | [kesin] soğurmaya dayalıdır, derişimle doğrusal |
| CO₂ (hacim) | `Σ Vᵢ·CO₂ᵢ / V` | [kesin] çözünmüş miktar; denge basıncı karışım sıcaklığından yeniden hesaplanır (04 §3.5) |
| Ekstrakt (°P) | `Σ mᵢ·Pᵢ / Σ mᵢ`, `mᵢ = Vᵢ · SG(Pᵢ)` | [kesin] °P kütle yüzdesidir, kütle ağırlıklı |
| Görünür FG (SG) | karışımın ekstraktından `ToSG` ile | [yaklaşık] alkol yoğunluğu farkı ihmal |

### 2.2 Hedefe göre oran [kesin]
İki bileşenle hedef değere ulaşmak için gereken ikinci bileşen hacmi:

```
V₂ = V₁ · (x₁ − x_hedef) / (x_hedef − x₂)        yalnız x_hedef, x₁ ile x₂ arasındaysa
```

**Karar (T1.12 C7):** "arasında" kesin ve toleranslıdır: `en küçük + 1e-9 < x_hedef < en büyük − 1e-9`; uçta ya da dışında sonuç yoktur ("hedef aralık dışı").

- `x` herhangi bir doğrusal değer olabilir: ABV, IBU, renk.
- Birden fazla değer aynı anda hedeflenirse hepsi iki bileşenle tutmayabilir. Ekran bir değeri hedef seçtirir, diğerlerinin sonucunu gösterir.

### 2.3 İzlenebilirlik
- Karışım yeni bir parti kodu almaz.
- Ana parti, kaynak partilerin hacimlerini taşır. Lot zinciri (izlenebilirlik raporu) her kaynağın hammadde lotlarını korur.

---

## 3. Hasat edilen mayayı tekrar kullanma

Maya yönetimi ekranı bu hesabı kullanır.

### 3.1 Gereken hücre [kesin, 04 §1.8]

```
N (milyar) = pitch oranı (milyon hücre/mL/°P) · V (L) · °P
```

### 3.2 Hücre sayımı: Neubauer hemositometre [kesin]
Merkez büyük kare 1 mm² ve 0.1 mm derinlikte, yani 10⁻⁴ mL; 25 küçük kareye bölünür.

```
hücre/mL = (sayılan hücre / sayılan küçük kare) · 25 · 10⁴ · seyreltme
canlılık = boyanmamış / (boyanmamış + boyanmış)            (metilen mavisi veya metilen menekşesi)
```

Bu bir **laboratuvar** yöntemidir. Ekranda sayım girişi; ara değerler ve sonuç aynı anda gösterilir.

### 3.3 Slurry hacmi [tasarım]

```
slurry (mL) = N · 10⁹ / (hücre/mL · canlılık)
```

- **Sayım yoksa** varsayılan etkin yoğunluk 1.0 × 10⁹ hücre/mL kullanılır. Bu değer maya dışı tortuyu da kapsar [doğrulanacak].
  - Hasat yöntemi ve tortu oranı sonucu birkaç kat değiştirebilir. Ekranda "tahmini, sayım önerilir" uyarısı çıkar.
- **Canlılık ölçülmemişse** hasattan bu yana geçen süreyle 04 §1.4'teki slurry yaşlanma modelinden tahmin edilir.
  - **Karar (T1.12 C13):** tahmini canlılığı hesaplayıp işleve veren çağıran katmandır (04 §1.4). Bu bölümdeki uyarı işlevi canlılık verilmediğinde (`None`) canlılık uyarısı üretmez.

### 3.4 Uyarılar [tasarım]

| Koşul | Uyarı |
|---|---|
| Nesil > ayardaki sınır (varsayılan 8) | "yeni kültür önerilir" |
| Hasattan bu yana > 14 gün | "canlılık düşük olabilir, sayım önerilir" |
| Canlılık < %80 | "starter veya taze maya" |
| Farklı stil ya da maya suşu geçmişi | bilgi: önceki partiler ve nesil zinciri gösterilir |

Sürekli eşikler tolerans payıyla: hasat günü `> 14 + 1e-9`, canlılık `< 0.80 − 1e-9`. Nesil sayıdır, tam karşılaştırılır.

---

## 4. Profesyonel verim

Raporlar ve ekipman kalibrasyon önerisi bu hesabı kullanır.

### 4.1 Kazandaki ekstrakt [endüstri]

```
ekstrakt (kg) = V_sıcak · 0.96 · SG(°P) · °P / 100
```

- `V_sıcak`: kaynatma sonunda, sıcakken ölçülen hacim.
- 0.96: yaklaşık 100 °C'den 20 °C'ye hacim büzülmesi; 05'teki %4 soğuma büzülmesiyle aynı.
- Hacim 20 °C'de ölçülmüşse 0.96 kullanılmaz.

### 4.2 Brewhouse verimi [endüstri]

```
brewhouse verimi (%) = 100 · ekstrakt (kg) / malt (kg)
laboratuvara oran (%) = brewhouse verimi / laboratuvar ekstraktı (ince öğütme, olduğu gibi)
```

- Profesyonel üretimde kullanılan tanım budur (Almanca adıyla Sudhausausbeute).
- Ev demlemesindeki "verim" farklı bir sayıdır: tarifteki potansiyele oran (01 §4). İkisi raporda ayrı gösterilir.

### 4.3 Dönüşüm ve süzme verimi [endüstri: Braukaiser yaklaşımı]

```
ilk şıra maks (°P) = 100 · E / (E + W)
  E = malt (kg) · laboratuvar ekstraktı olduğu gibi (%) / 100
  W = mash suyu (L) + malt (kg) · nem (%) / 100
dönüşüm verimi = ölçülen ilk şıra °P / ilk şıra maks
süzme verimi   = (brewhouse verimi / laboratuvar ekstraktı) / dönüşüm verimi
```

- **Karar (T1.17):** motor `lab_ratio` (§4.2, kesir) ve `lauter_eff` (süzme verimi, kesir) hesaplar. Bölenlerden biri (laboratuvar ekstraktı ya da dönüşüm verimi) 0 ise hata verir (00-OKU-BENI hata kuralı). Motor BIAB'de de hesaplar; gösterimi gizlemek istemcinin işidir, bu yüzden yanıtta süzme alanı isteğe bağlıdır (istemci "sparge yok" bilgisini verirse sunucu alanı boş bırakır).
- **Ne için:** düşük verimin nedenini ayırır.
  - Dönüşüm düşükse sorun mash'te: öğütme, sıcaklık, süre, pH.
  - Süzme düşükse sorun lauterde.
- **Ölçüm:** ilk şıra yaklaşık 10 dakika recirculation sonrasında alınır. Ekranda ölçüm alanı bu tanımla açıklanır.
- **Kapsam:** BIAB ve sparge'sız mash'te ilk şıra = tüm şıra olduğu için süzme verimi anlamlı değildir; gösterilmez.

---

## 5. Malt analizinden ekstrakt potansiyeli

Kendi malzemen ekranı (malt) bu hesabı kullanır.

### 5.1 Analiz değerleri [endüstri]
Malt teknik föyündeki üç değer:
- **FGDB:** ince öğütme, kuru baz ekstrakt %.
- **Nem:** %.
- **İnce–kaba farkı:** genelde 1–2 puan.

```
FGAI = FGDB · (1 − nem/100)                  ince, olduğu gibi (laboratuvar ekstraktı; §4 bunu kullanır)
CGDB = FGDB − ince–kaba farkı                kaba, kuru baz
CGAI = CGDB · (1 − nem/100)                  kaba, olduğu gibi
```

### 5.2 Tarifte kullanılan değer [tasarım]
- **Karar (T1.12 C9):** FGAI, CGDB, CGAI, potansiyel ve PPG yuvarlanmadan döner; sabitler (0.04621, 0.046, 0.4621) bu yüzden vektörle sabitlenir.
- Tarif hesaplarındaki "yield" (01 §1.6) **CGAI** kabul edilir. Gerçek öğütmeye en yakın değer budur.
- Föyde ince–kaba farkı yoksa varsayılan 1.5 puan [doğrulanacak].

```
potansiyel SG = 1 + yield · 0.04621 / 100      (sakaroz = 46.21 PPG) [kesin]
PPG           = yield · 0.4621
```

01 §1.6'daki BeerSmith formülü 0.046 kullanıyor. Biz 0.04621'i öneriyoruz; ikisi arasındaki fark 4. ondalık basamakta. Bu karar 01 dosyasının "belirsiz" listesine de not edildi.

### 5.3 Renk [kesin]
- Föy rengi genelde EBC'dir: `SRM = EBC / 1.97`, `°L = (SRM + 0.76) / 1.3546`.
- 1.3546 plan kararıdır; BeerSmith 1.3564 kullanıyor (01 §1.5 notu).

### 5.4 Diğer analiz alanları
Diastatik güç (°WK veya °L), protein, Kolbach indeksi, β-glukan, friabilite. Hesapta kullanılmazlar, saklanır.
- **Diastatik güç:** tarif editöründe "bu tahıl faturası kendi kendini dönüştürür mü" kontrolüne girer. Tahıl faturası ağırlıklı ortalama < 35 °L ise uyarı verilir [doğrulanacak eşik].
  - **Karar (T1.12 C1):** ağırlıklı ortalama (ağırlıklar toplamıyla bölünmüş `Σ kg·°L`; toplamlar Python `sum` ile aynı telafili toplamla) ve uyarı ⇔ `ortalama < 35 − 1e-9`. Tam 35 °L uyarı vermez. Boş ya da sıfır ağırlıklı fatura: ortalama 0, uyarı yok.

---

## 6. Test vektörleri

`python3 -I testler/08_isletme_cekirdek_tv.py` çıktısından.

| No | Girdi | Beklenen |
|---|---|---|
| 1.1 | Pale 4.5 kg; lotlar L-A 3.0 kg @1.10, L-B 25 kg @1.25 | L-A 3.0 kg (3.30) + L-B 1.5 kg (1.875 → 1.88) = 5.18; eksik 0 |
| 1.2 | Crystal 0.30 kg, stok 0.20 kg @52.0 | stoktan 10.40; eksik 0.10 kg (son alış fiyatıyla tahmini) |
| 1.3 | 25 kg × 1.20 + kargo 4.00 | 1.36 / kg |
| 1.4 | 15 L 15→72 °C + 28 L 76→100 °C (8 kg kazan, η 0.90) + 70 dk 2.5 kW kaynatma | 4.99 kWh |
| 1.5 | aynı ısınma ısısı, gaz, η 0.40, Hu 34 500 | 0.487 m³ |
| 1.6 | malzeme 18.40; su 35 L @30/m³; enerji 4.99 kWh @2.6; 60 kutu @0.35; ambalajlanan 20 L | su 1.05, enerji 12.97, ambalaj 21.00, toplam 53.42, litre 2.671 |
| 2.1 | 960 L (ABV 5.4, IBU 38, SRM 7, CO₂ 1.0, 2.8 °P) + 400 L (4.6, 22, 4, 1.0, 2.4 °P) | 1360 L, ABV 5.165, IBU 33.29, SRM 6.12, CO₂ 1.0, 2.682 °P |
| 2.2 | 960 L %5.4 → %5.2, %4.6'lık bira ile | 320 L |
| 2.3 | 960 L %5.4 → %5.0, su ile | 76.8 L |
| 3.1 | 10 hL, 12 °P, ale 0.75 | 9000 milyar hücre |
| 3.2 | 5 küçük karede 230 hücre, 1:100 seyreltme; 212 boyanmamış, 18 boyanmış | 1.150 × 10⁹ hücre/mL; canlılık 0.922 |
| 3.3 | 3.1 + 3.2 | 8.49 L slurry |
| 3.4 | sayım yok, 1.0 × 10⁹ hücre/mL, canlılık 0.90 | 10.0 L (tahmini) |
| 4.1 | 12 °P, 1040 L sıcak | SG 1.0484; ekstrakt 125.6 kg |
| 4.2 | 170 kg malt; FGDB 81.5, nem 4.3 | brewhouse verimi %73.88; laboratuvar (FGAI) %78.0; oran %94.7 |
| 4.3 | mash suyu 520 L; ölçülen ilk şıra 19.6 °P | ilk şıra maks 20.09 °P; dönüşüm %97.5; süzme %97.1 |
| 5.1 | FGDB 81.5, nem 4.3, ince–kaba 1.5 | FGAI 78.0, CGDB 80.0, CGAI (yield) 76.56; potansiyel 1.0354 (BeerSmith sabitiyle 1.0352); PPG 35.38 |
| 5.2 | 3.5 EBC | 1.78 SRM, 1.87 °L |

## 7. Açık noktalar

- **Doğrulanacak varsayılanlar:** doğal gaz Hu, varsayılan slurry yoğunluğu, ince–kaba farkı, diastatik güç eşiği.
- **Yield tanımı:** BeerSmith'in "yield" alanı içe aktarılan tariflerde hangi bazda (ince/kaba, kuru/olduğu gibi)? BeerXML'de `YIELD` ince öğütme kuru bazdır ve `COARSE_FINE_DIFF` ile `MOISTURE` ayrı alanlardır. İçe aktarmada bu alanlar varsa §5 ile CGAI'ye çevrilmeli. Gerçek dosyalarla kontrol edilecek.
- **Harmanlama:** ABV'de hacim büzülmesi ihmal edildi. Yüksek alkollü karışımlarda (barley wine ile harmanlama) etkisi kontrol edilecek.
