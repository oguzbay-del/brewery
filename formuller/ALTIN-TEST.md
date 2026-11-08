# Altın test şablonu

Bu, planlama listesinin **2. işi**. `plan/gorusme.md` içindeki karar tablosu dolmadan burası doldurulmaz. Python dosyaları (`formuller/testler/`) altın test değildir: sayılar formülün kendi çıktısıdır, BeerSmith ekranıyla veya hidrometreyle karşılaştırılmamıştır (`formuller/00-OKU-BENI.md`).

Burada BeerSmith sayıları ve gerçek demleme sayıları boştur. Uydurma sonuç yazılmaz.

## Ne zaman geçti sayılır

Aynı tarif BeerSmith 4'te açılır. Ekrandaki sayı "BeerSmith" sütununa, bizim referans betiğinin sayısı "Python" sütununa yazılır. İkisi de boşsa satır beklemede kalır.

Geçme payı öneridir; ilk üç tarif görülünce `plan/gorusme.md` soru 5 ile sıkılaştırılır veya gevşetilir. \[doğrulanacak\]

| Sayı | Önerilen pay |
| --- | --- |
| OG ve FG | ±0,001 SG |
| IBU | ±1 veya %3, büyük olan |
| Renk | ±1 EBC |
| Hacim (strike, sparge, kaynatma sonu) | ±0,1 L |
| ABV | ±0,1 yüzde puan |
| Mash pH | Bu sette karar yok. MPH ve BW yan yana yazılır; hangisinin kalacağı görüşmedeki su satırına bakılır |

Bir satır payın dışındaysa formül düzeltilir, pay genişletilerek geçilmiş sayılmaz.

Gerçek demleme ölçümü ayrı sütundur. Varsa yazılır (hidrometre, düzeltme sıcaklığıyla). Yoksa satır "ekran karşılaştırması" olarak kalır; ölçüm uydurulmaz.

## Nasıl doldurulur

1. Görüşmede "Ev MVP" kalan senaryolar açık, "yok" denenler bu tabloda Ev MVP dışı işaretlenir.
2. BeerSmith'te tarifi kur. Birim metrik görünsün. Ekran görüntüsü repoya konmaz; sayılar tabloya yazılır.
3. Aynı girdiyle ilgili `formuller/testler/` betiği çalıştırılır. Betik bu tarifi içermiyorsa önce betiğe vektör eklenir, sonra sayı buraya geçilir.
4. Fark = Python − BeerSmith. Pay dışındaysa not sütununa hangi girdi şüpheli (verim, hop formu, kayıp).

## Tarifler

On altı senaryo. Hepsi Ev MVP adayıdır; görüşme "yok" derse sağdaki işaret değişir. Malzeme adı örnek olsun diye durur, zorunlu marka değildir. Miktar, BeerSmith'e kurulurken yazılır; şimdilik boş.

| No | Senaryo | Neden | Ev MVP | OG | FG | IBU | EBC | Hacim L | pH MPH | pH BW | ABV | Ölçüm | Fark notu |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | Tek adım ale, yaklaşık 20 L, pale + az crystal, 60 dk hop | Temel OG, FG, renk, IBU | aday | | | | | | | | | | |
| 2 | Stout, kavrulmuş malt payı yüksek | Morey renk, koyu uç | aday | | | | | | | | | | |
| 3 | OG 1.080 ve üstü | FG, alkol, verim | aday | | | | | | | | | | |
| 4 | BIAB, kazanda kayıp farklı | Hacim zinciri | aday | | | | | | | | | | |
| 5 | Batch sparge, iki su | Strike ve sparge hacmi | aday | | | | | | | | | | |
| 6 | Ekstrakt + steep | LME/DME puanı, tahıl değil | aday | | | | | | | | | | |
| 7 | Whirlpool ve geç hop ağır | IBU segmenti | aday | | | | | | | | | | |
| 8 | 60 + 15 + 0 dk, pelet | Kullanım ve form düzeltmesi | aday | | | | | | | | | | |
| 9 | Aynı tarif, hop yaprak | Form çarpanı | aday | | | | | | | | | | |
| 10 | Yüksek rakım | Kaynama sıcaklığı ve IBU | aday | | | | | | | | | | |
| 11 | Aynı tarif, mash 65 °C ve 68 °C | FG sıcaklık düzeltmesi. İki satır gibi doldurulur | aday | | | | | | | | | | |
| 12 | Yumuşak su, asit ekli | MPH ve BW yan yana. Model seçimi bu satırda yapılmaz | aday | | | | | | | | | | |
| 13 | Sert, karbonatlı su | Aynı pH kararı, ikinci uç | aday | | | | | | | | | | |
| 14 | No-chill | Geç IBU ayarı | aday | | | | | | | | | | |
| 15 | 20 L tarif 100 L'ye ölçek | Ölçek çarpanı. İşletme ekranı değil | aday | | | | | | | | | | |
| 16 | Görüşmeden gönüllü bir gerçek parti | Ekran tahmini ve hidrometre. Kişi kodu (E1), ad yok | aday | | | | | | | | | | |

pH sütunları yalnız 12 ve 13 için doldurulur. Diğerlerinde boş kalır.

## Bitti sayılması

- Görüşme tablosunda "yok" denen senaryo burada "Ev MVP dışı" yazıyor.
- Kalan satırların OG, FG, IBU, EBC ve hacim hücrelerinde BeerSmith ve Python sayısının ikisi de var.
- Pay dışındaki her satırın notu var ve formül işi kapanmadan "geçti" denmiyor.
- 12 ve 13'te MPH ve BW'nin ikisi de yazılı; hangisinin motor varsayılanı olacağı görüşme kararında.

Ondan sonra sıradaki plan işi formül ADR'si ve veri modeli API'sidir. Rust motoru bu tablo geçmeden yazılmaz.
