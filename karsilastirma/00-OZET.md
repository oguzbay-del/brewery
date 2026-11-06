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
