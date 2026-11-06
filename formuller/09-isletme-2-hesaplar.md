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
alkol (g/100 mL)          = ABV · 0.78924                 (etanol yoğunluğu, 20 °C)
gerçek ekstrakt (g/100 mL) = RE (°P, g/100 g) · SG_son        (RE: 01 §6.1)
karbonhidrat (g/100 mL)   = gerçek ekstrakt − protein − (kül, girilmişse)
enerji                    = Σ bileşen · katsayı
```

- **ABV:** ölçülmüş değer varsa o kullanılır (etikette beyan edilen ABV), yoksa 01 §6.3'teki tahmin.
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
| `hacim` | oran × hacim | Türkiye ÖTV (maktu tutar; birim doğrulanacak), ABD federal (varil başına) |
| `saf_alkol` | oran × V × ABV/100 (litre saf alkol, LPA); ABV bandına göre oran; küçük üretici indirimi LPA başına | Birleşik Krallık |
| `hl_plato` | oran × hL × °P (°P tam sayıya kesilebilir) | Almanya, Avusturya |
| `plato_bandi` | °P bandına göre hL başına oran | İsviçre |
| `kademeli_hacim` | yıllık kümülatif hacme göre kademeli oran; yıl içindeki önceki çıkışlar hesaba katılır | ABD federal küçük bira fabrikası oranı |

**Ortak kurallar:**
- **Ülke birden çok yöntemi birleştiriyorsa** (ör. bant + kademe + indirim), yöntemler zincir olarak uygulanır. Zincirin sırası veri tablosundadır.
- **ABV kaynağı:** ölçülmüş (laboratuvar) değer; ölçüm yoksa tahmini, ama rapor bunu "tahmini" diye işaretler. Bazı ülkelerde beyan edilen ABV kullanılır; hangisi olduğu veri tablosundadır.
- **°P kaynağı:** ölçülmüş başlangıç ekstraktı (Stammwürze), yani OG'nin °P karşılığı.
- **Para birimi ve yuvarlama:** veri tablosundan; kuruş ve sent yuvarlaması satır bazında ya da toplamda.

### 2.3 Rapor çıktısı [tasarım]
- Dönem (ay veya çeyrek), ürün, ambalaj tipi (fıçı/şişe/kutu) ve vergi satırı bazında hacim, ABV, °P, LPA, oran ve tutar.
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
gereken (g/L)      = gereken vol · 1.977                   (1 vol CO₂ = 1.977 g/L, 0 °C 1 atm)
```

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
ayar basıncı = CO2_pressure(T_fermantasyon, hedef vol)   (psig; bar = psi · 0.0689476)
```

### 3.4 Uyarılar [tasarım]
- **Tank basınç sınırı:** ayar basıncı tank kaydındaki çalışma basıncını aşarsa engellenir. Önerilen çözüm: daha düşük hedef ya da daha soğuk fermantasyon.
- **FG belirsizliği:** kapama gravitesi beklenen FG'ye bağlıdır. Fermantasyon tahmini (07 §1) güven aralığı genişse "1–2 puan erken kapatma riski" gösterilir.
- **İzleme:** basınç sensörü varsa gerçek basınç ve sıcaklıktan anlık CO₂ hacmi hesaplanır (04 §3.5'in tersi, sayısal çözüm) ve grafikte gösterilir.

---

## 4. Test vektörleri

`python3 -I testler/09_isletme_2_tv.py` çıktısından. Vergi vektörleri **örnek oranlarla** yöntemleri test eder; gerçek oran değildir.

| No | Girdi | Beklenen |
|---|---|---|
| 1.1 | Pilsner OG 1.048, FG 1.009, protein 0.4 | ABV 5.14, alkol 4.06 g, gerçek ekstrakt 4.08 g, karbonhidrat 3.7 g; **187 kJ / 45 kcal** /100 mL |
| 1.2 | Citra Pale Ale OG 1.052, FG 1.011 | ABV 5.41; 204 kJ / 49 kcal |
| 1.3 | 1.1 ama ölçülen ABV %5.0 | alkol 3.95 g; 184 kJ / 44 kcal |
| 2.1 | 20 hL, %5.0, `saf_alkol` 21.78/LPA | 100 LPA → 2 178.00 |
| 2.2 | aynı, bantlar (<3.5: 9.27, <8.5: 21.78, diğer: 29.54), indirim 5.0/LPA | 1 678.00 |
| 2.3 | 20 hL, 12.6 °P, `hl_plato` 0.787, °P kesilir | 20 · 12 · 0.787 = 188.88 |
| 2.4 | 20 hL, 12 °P, `plato_bandi` (≤10: 16.88, ≤14: 25.32, diğer: 33.76) | 506.40 |
| 2.5 | 20 hL, `kademeli_hacim` (60 000 bbl'a kadar 3.50/bbl, sonra 16/bbl), yıl başı | 59.65 |
| 2.6 | aynı, bu yıl eşiğe 10 hL kalmış | 10 hL @3.50 + 10 hL @16 /bbl = 166.17 |
| 2.7 | 20 hL, `hacim` 12.49/L | 24 980.00 |
| 3.1 | hedef 2.5 vol, 12 °C, FG 1.010 | kalıntı 1.12 vol; 2.73 g/L gerekli; **kapama SG 1.0128** (2.8 puan önce); ayar 19.5 psi = 1.35 bar |
| 3.2 | hedef 2.6 vol, 18 °C, FG 1.012 | kalıntı 0.92 vol; 3.33 g/L; kapama SG 1.0154 (3.4 puan); 27.1 psi = 1.87 bar |

## 5. Açık noktalar

- **Türkiye ÖTV:** biranın güncel (Temmuz 2026 sonrası) maktu tutarı ve tutarın birimi Resmi Gazete'den alınacak. Beyanname dönemi ve verginin doğduğu an (ÖTV Kanunu) da tanımlanacak.
- **Birleşik Krallık:** küçük üretici indirimi bir formülle hesaplanır, sabit bir indirim değildir. gov.uk kılavuzundaki hesap ayrı bir yöntem olarak eklenecek.
- **Almanya:** kademeli oran tablosu ve °P'nin kesirli kısmının ele alınışı yasa metninden alınacak.
- **Etiket:** protein varsayılanı ve AB yuvarlama kılavuzu kontrol edilecek. Türkiye yönetmeliğiyle karşılaştırılacak.
- **Spunding:** 04'teki kalıntı CO₂ ve basınç polinomları topluluk uyumları [tahmin]. Basınçlı fermantasyonda ölçülen basınçla karşılaştırılacak.
