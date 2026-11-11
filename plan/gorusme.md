# Görüşme ve Ev MVP kilidi

Bu, planlama listesinin **1. işi**. Altın test (`formuller/ALTIN-TEST.md`) bu dosyadaki karar tablosu dolmadan doldurulmaz. Görüşme ve altın test bitmeden hesap motoru ve ekran kodu ana dala yazılmaz (teknik deneme atılabilir prototiptir).

Görüşmeleri yapan kişi notu buraya işler. Bu dosyada görüşme yapılmış gibi sonuç yoktur.

## Kimle

Beş ile on ev biracısı, iki ile üç küçük işletme. Tarih ve kişi yazılmadan satır "yapıldı" sayılmaz.

Ev biracısı karması, hepsi birden:

- En az ikisi bugün BeerSmith, Brewfather veya Brewer's Friend kullanıyor.
- En az ikisi tarifi kağıtta, notta veya tabloda tutuyor.
- En az biri demleme gününü telefonda geçiriyor.
- En az biri son bir yılda 20 litrenin üstünde parti kaynatmış.

İşletme: 1 hL ile 30 bbl arası, ilk hedef pazar ABD veya İngiltere. Bu görüşme Ev MVP'ye özellik eklemek için değil, "Ev'e koyma" listesini doğrulamak için. ERP sorulmaz (satış, sipariş, fatura, keg takibi).

## Oturum

40–60 dakika. Kayıt için izin sorulur. Tarif dosyası bu oturumda istenmez; altın test ayrı ve gönüllüdür.

Not biçimi, kişi başına:

| Alan | Yaz |
| --- | --- |
| Kod | E1, E2, … ev; İ1, İ2, … işletme. Ad soyad repoya yazılmaz |
| Tarih | |
| Pazar | ABD / İngiltere / başka |
| Bugünkü aracı | BeerSmith, Brewfather, kağıt, başka |
| Parti büyüklüğü | tipik litre veya bbl |
| Tek cümle | Son demlemede nerede tıkandı |

## Ev biracısına sorulacaklar

Her sorunun kilitleyeceği karar yanında. Cevap "evet" diye yönlendirilmez.

| # | Soru | Kilitleyeceği karar |
| --- | --- | --- |
| 1 | Son demlemeyi baştan sona anlat. Nerede durdun, neyi başka yerde hesapladın? | Sihirbaz mı editör mü önde |
| 2 | Tarif bugün nerede duruyor? Taşırken ne bozuluyor? | İçe aktarma (BeerXML, BeerJSON, .bsmx) Ev MVP'de zorunlu mu |
| 3 | Demleme günü telefon mu, kağıt mı, ikisi mi? Sayaç kullandın mı? | Sayaç isteğe bağlı kalsın mı |
| 4 | İnternet kesilince hangi iş yarıda kalırsa uygulamayı bırakırsın? | Demleme paketinin sınırı (ölçüm girişi yeter mi, yeniden hesap da mı) |
| 5 | OG, IBU, renk, hacim, mash pH. Hangisi yanlışsa tarifi uygulamada tutmazsın? | Altın testin geçme payı hangi sayıda sıkı |
| 6 | Aşağıdaki "doğrulanacak" satırlarını tek tek: Ev'de ilk günden olsun mu, sonra mı, hiç mi? | Karar tablosu |
| 7 | Ücretsiz bir sürüm olsaydı içinde ne olmalıydı? Hangisi için para verirsin? | Ücretsiz katmanın sınırı (deneme süresi planda karar: 1 ay tam deneme) |

## İşletmeye sorulacaklar

Kısa. Üç soru.

1. Bugün üretimi nerede izliyorsun (pano, tabela, başka yazılım)?
2. Bir ev uygulamasını işletmede açmanı engelleyen üç şey nedir?
3. Çok kullanıcı, lot, CIP, mevzuat raporu: hangisi ilk ay, hangisi yıl sonra?

İşletme cevapları Ev MVP kapsamını büyütmez. "İlk ay" deneni İşletme çekirdeğine not düş, Ev satırına taşıma.

## Doğrulanacak kapsam

Plandaki Ev MVP. Görüşme bitmeden sağ sütun boş kalır. Beş ev notu yokken "MVP kilitli" yazılmaz.

| Özellik | Plandaki yer | Görüşme sonucu | Karar: Ev MVP / sonra / yok | Kim (kod) |
| --- | --- | --- | --- | --- |
| Tarif sihirbazı önde, editör sade | Mod ayrımı | | | |
| Demleme paketinin sınırı: yalnız ölçüm girişi mi, gerçek değerle yeniden hesap da mı (paket kendisi sabit karar) | Platformlar | | | |
| Sayaç isteğe bağlı, kağıt ve tarama | Demleme günü | | | |
| BeerXML, BeerJSON, .bsmx içe aktarma | Planlama işleri | | | |
| Su ve pH (malt bazlı grist pH; MPH/BW eğimi seçenek, varsayılan altın testle) | Hesap motoru | | | |
| Ekipman kalibrasyonu | Ek özellikler, Ev MVP | | | |
| Otomatik fermantasyon uyarısı | Ek özellikler, Ev MVP | | | |
| Tarif karşılaştırma | Ek özellikler, Ev MVP | | | |
| Geriye planlama | Ek özellikler, Ev MVP | | | |
| Dekoksiyon aracı | Ek özellikler, Ev MVP | | | |
| Kalori | Ek özellikler, Ev MVP | | | |
| Ekstrakt araçları | Ek özellikler, Ev MVP | | | |
| Parti fotoğrafı | Ek özellikler, Ev MVP | | | |
| Favoriler | Ek özellikler, Ev MVP | | | |
| Şişe başına maliyet | Ek özellikler, Ev MVP | | | |
| Mead / cider | Mod ayrımı, açık | | | |
| Çok kullanıcı, roller, denetim kaydı | Ev'de kapalı | | | |
| Mevzuat raporu | Ev'de kapalı | | | |

## Bitti sayılması

- En az beş ev ve iki işletme notu dosyada.
- Üstteki tablonun her satırında karar ve kod var.
- "Sonra" ve "yok" denen satırlar `formuller/ALTIN-TEST.md` içinde ilgili senaryoyu "Ev MVP dışı" diye işaretler.

Ondan sonra 2. iş: altın test.
