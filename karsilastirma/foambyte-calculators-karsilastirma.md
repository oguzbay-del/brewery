# Foambyte Calculators ile Demleme repo karşılaştırması

> Bu belge, FoamByte'un `https://foambyte.com/calculators/` sayfasındaki hesap araçlarının içeriklerini ile bu repo'daki plan ve formül spesifikasyonlarını eşleştirir. Amaç, kapsam benzerliğini ortaya çıkarmak ve ürünün şu anki durumunu netleştirmektir. 
> 
> Not: Bu repo, README ve CLAUDE.md'ye göre henüz gerçek uygulama kodu değil; plan, tasarım, formül ve test vektörleri aşamasındadır. Bu nedenle "özellik var" ya da "uygulama hazır" gibi ifadeler yerine, "kapsam karşılığı var" ve "uygulama aşaması henüz gelmedi" ayrımı yapılmalıdır.

## Özet

Foambyte calculators sayfası, bira üretimi için temel hesap araçları sunar:
- Gravity / ABV / OG-FG hesapları
- Mash ve hacim hesapları
- IBU / bitterness ve renk hesapları
- Su kimyası ve pH
- Maya canlılığı ve starter hesapları
- Karbonasyon / priming
- Hydrometer ve refraktometer düzeltmeleri
- Birim dönüşümleri
- Yardımcı araçlar: boil-off, dilution, decoction, extract, no-chill vb.

Bu repo içinde buna karşılık gelen başlıca dosyalar şunlardır:
- `formuller/01-yogunluk-alkol-renk.md`
- `formuller/02-hop-ibu.md`
- `formuller/03-su-ph.md`
- `formuller/04-maya-mead-karbonasyon.md`
- `formuller/05-mash-hacimler.md`
- `formuller/10-elle-giris-ekstrakt-nochill.md`
- `formuller/11-tetikleyici-fici-cask.md`

Bu, kapsam olarak Foambyte calculators'ın ana kategorilerini yakalayan bir tasarım olduğunu gösterir. Ancak repo şu anda ürüne ait çalışan kod değil; formül ve test tasarımı seviyesindedir.

## Foambyte calculators kategorileri -> repo eşlemesi

### 1) Gravity / ABV / OG-FG hesapları
Foambyte sayfasında öne çıkan alan: yoğunluk, alkol oranı, gerçek ekstrakt, attenuation ve düzeltme.

Repo karşılığı:
- `formuller/01-yogunluk-alkol-renk.md`
- `karsilastirma/01-yogunluk-alkol-renk-karsilastirma.md`

Yorum:
- Bu alan repo içinde en güçlü karşılıkların başında gelir.
- OG/FG/ABV ve verim hesapları formül olarak detaylı yazılmıştır.
- Bu, Foambyte'nun calculators sayfasındaki ana kullanım senaryolarına doğrudan benzer.

### 2) IBU / bitterness / hop hesapları
Foambyte sayfasında öne çıkan alan: aroma ve acılık hesapları, hop katkısı, IBU ve hop sonu etkileri.

Repo karşılığı:
- `formuller/02-hop-ibu.md`
- `karsilastirma/02-hop-ibu-karsilastirma.md`

Yorum:
- IBU hesapları yoğun biçimde işlenmiş.
- Tinseth, Rager, Garetz, whirlpool, rakım etkisi, hop stand, geç ekstrakt segmentleri gibi konular detaylı şekilde ele alınmış.
- Foambyte calculators'daki bitterness araçlarının mantığıyla çok uyumlu bir tasarım görünümü var.

### 3) Su kimyası / pH / mash adjustments
Foambyte calculators sayfasında öne çıkan alan: su profili, pH, asitleme ve mash tarafı.

Repo karşılığı:
- `formuller/03-su-ph.md`
- `karsilastirma/03-su-ph-karsilastirma.md`

Yorum:
- Repo burada çok güçlü.
- Sulfate/chloride, alkalinite, mash pH, laktik/fosforik asit, asit malt, profil eşleştirme gibi hesaplar yer alıyor.
- Bu, Foambyte calculators karşılaştırmasında en kritik alanlardan biri.

### 4) Maya canlılığı / starter / karbonasyon
Foambyte calculators sayfasında öne çıkan alan: yeast viability, starter size, priming, carbonation.

Repo karşılığı:
- `formuller/04-maya-mead-karbonasyon.md`
- `karsilastirma/04-maya-mead-karbonasyon-karsilastirma.md`

Yorum:
- Bu bölümde maya canlılığı, pitch rate, starter, priming ve fıçı basıncı işlenmiş.
- Foambyte calculators'ın yeast/carbonation çıkarımlarına benziyor.
- Ayrıca mead ve cider için ayrı hesaplar da eklenmiş; bu, daha geniş bir özellik setini gösterir.

### 5) Mash / volume / decoction / sparge
Foambyte calculators sayfasında öne çıkan alan: mash hesapları, brewday hacim, decoction, boil-off, dilution.

Repo karşılığı:
- `formuller/05-mash-hacimler.md`
- `karsilastirma/05-mash-hacimler-karsilastirma.md`

Yorum:
- Bu alan Foambyte calculators ile doğrudan benzerlik gösterir.
- Batch sparge, strike temperature, decoction, boil-off, soğuma büzülmesi gibi hesaplar detaylı.

### 6) Yardımcı hesaplar / extract / no-chill / cask
Foambyte calculators sayfası çoğunlukla yardımcı araçlar listeler; repo da bunu dokümante ediyor.

Repo karşılığı:
- `formuller/10-elle-giris-ekstrakt-nochill.md`
- `formuller/11-tetikleyici-fici-cask.md`

Yorum:
- Bu dosyalar, Foambyte kalkülatörlerindeki pratik araçların repo içindeki teorik versiyonlarını çağrıştırıyor.
- Özellikle no-chill, extract aracı, cask priming ve fermantasyon tetikleyicileri çok benzer işlevleri tarif ediyor.

## Repo'nin gerçek durumu

`README.md` ve `CLAUDE.md` açıkça diyor:
- uygulama kodu yazılmıyor
- çalışma, plan / tasarım / formül / araştırma aşamasında
- gerçek üretim uygulaması henüz yok

Bu nedenle Foambyte calculators sayfası ile karşılaştırma yapılırken şu ayrım çok önemlidir:

- Kapsam olarak benzerlik: Yüksek
- Uygulama olgunluğu: Henüz mevcut değil
- Gerçek ürün fonksiyonelliği: Yok

Yani bu repo, Foambyte'un calculator seti gibi bir ürünün kodu değil; buna benzer işlevleri planlayan ve formülleştiren bir tasarım/dokümantasyon reposu.

## Sonuç

Foambyte calculators sayfası ve bu repo arasındaki en doğru değerlendirme şudur:

- Repo, Foambyte'un hesap kategorilerini büyük ölçüde tasarım ve algoritma düzeyinde yakalıyor.
- Ev biracılığı / küçük üretim için hesap aracı setinin ana yapısını düşünsel olarak içeriyor.
- Ancak bu repo, çalışan bir uygulama ya da canlı ürün değil; plan ve formül spesifikasyonu aşamasında.

Dolayısıyla en doğru cümle:

"Foambyte calculators ile benzer hesap alanları repo içinde detaylı şekilde planlanmış olsa da, bu repo şu an uygulama kodu değil; hesap ve ürün tasarımı düzeyinde bir hazırlık deposudur."

## Dizin ve referanslar

- `formuller/01-yogunluk-alkol-renk.md`
- `formuller/02-hop-ibu.md`
- `formuller/03-su-ph.md`
- `formuller/04-maya-mead-karbonasyon.md`
- `formuller/05-mash-hacimler.md`
- `formuller/10-elle-giris-ekstrakt-nochill.md`
- `formuller/11-tetikleyici-fici-cask.md`
- `README.md`
- `CLAUDE.md`
- `https://foambyte.com/calculators/`
