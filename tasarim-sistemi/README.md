Demleme; ev biracıları ve küçük bira fabrikaları için iPad, iPhone ve web'de çalışan (Apple Watch ileride opsiyonel) bir tarif ve üretim uygulaması. Görsel dil Apple'ın kendi uygulamalarını izler: sistem yazısı, sistem renkleri, standart bileşenler ve tek bir vurgu rengi, mayşe amberi.

## İçerik ve dil

- Arayüz metni Türkçe ve İngilizce (ABD/İngiltere/Avustralya yazım farklarıyla). Kullanıcıya "sen" diye hitap et ("İlk tarifini sihirbazla oluştur").
- Kısa, etken cümle. Düğme fiille başlar ve ne olacağını söyler: "Demlemeyi başlat", "Arşivle", "Değişiklikleri sil". "Tamam" ve "Gönder" gibi genel sözcük kullanma.
- Hata ve uyarı metni ne olduğunu ve ne yapılacağını söyler, özür dilemez: "Gecikmiş: FV2 SG okuması dünden beri yok."
- Bira dünyasının terimleri olduğu gibi kalır: OG, FG, IBU, ABV, SG, °P, EBC/SRM, dry hop, cold crash, keg, cask, CIP, BBT, FV.
- Sayılar bölge ayarına göre: TR ve AB'de ondalık virgül ("19,2 °C"), UK/US/AU'da nokta. SG her zaman üç basamak ("1.048"). Birim değerden bir boşlukla ayrılır.
- Tahmin her zaman aralıkla verilir ("3–4 gün · ±0.002"); veri yetmezse sayı yerine nedeni yazılır.
- Emoji kullanma; durumları ikon ve kelimeyle anlat.

## Renk

- Zemin `bg`, üzerindeki gruplar ve kartlar `card`; düğme ve ray zemini `fill`. Birincil metin `label`, ikincil `label2`, yalnız süs için `label3`.
- Vurgu tek renk, iki tonda: yazı ve ikonda koyu amber `tint`, geniş alanlarda parlak amber `tint-fill` ve üzerinde her zaman `tint-ink`. Parlak amberin üstüne beyaz yazma.
- `tint-fill` kullanılan yerler: ana düğme, demleme sayaç halkası, `BrewReturnBar`, seçili kenar çubuğu, grafik çubukları, takvim şeritleri. Ekranda tek ana düğme olur.
- Durum renkleri yalnız durum için: `green` tamam, `yellow` uyarı (açık temada turuncu), `red` hata ve acil, `blue` bilgi ve sistem eylemi. Hiçbiri marka rengi değil; durum her zaman bir kelime veya ikonla birlikte.
- Seçili satır `tint-soft`. Ayırıcı ve ince kenar `sep`.
- `frame` yalnız konseptteki cihaz çerçevesi ve kapak içindir.

## Temalar ve erişilebilirlik

- Üç tema: Açık, Koyu ve Yüksek karşıtlık. Varsayılan "Sistem": cihazın açık/koyu ayarını izler; seçim cihaz başına. iOS "Karşıtlığı Artır" açıkken Yüksek karşıtlık teması kullanılır.
- Açık ve koyu temada okunması gereken her metin en az 4.5:1 karşıtlıktadır. Bunun için açık temada `label2` Apple'ın %60 değeri yerine %75 opaklıkta, `tint`, `green`, `yellow` ve `red` bir ton koyudur. Yüksek karşıtlık teması `label2`, `tint`, `tint-fill`, `tint-ink` ve `sep` değerlerini daha da koyulaştırır. `label3` yalnız süs ve pasif öge içindir (pasif öge karşıtlık şartından muaftır). Koyu temada yüksek karşıtlık (`contrast-dark`, koyu + Karşıtlığı Artır) aynı 5 token için daha parlak değer verir: `bg`/`card` üzerinde `label2` 12.6/10.7:1, `tint` 13.2/10.7:1, `sep` 3.9:1, `tint-fill` `bg` üzerinde 13.2:1, `tint-ink` `tint-fill` üzerinde 13.2:1.
- Odak halkası `blue`, 3 px, 2 px dışarıda.
- Durum hiçbir zaman yalnız renkle anlatılmaz: alarm işaretleri renk + şekil + numara, uyarı kutuları ikon + metin taşır.
- Yazı boyutu iOS Dinamik Yazı ayarını izler; tablodaki `t-` stilleri varsayılan (Large) boyutlardır. "Hareketi Azalt" açıkken geçiş animasyonları kapanır.
- Dokunma alanı en az `tap-min` (44 px).

## Tipografi

- Uygulamada Apple sistem yazısı: SF Pro (`sys`), rakamlar için SF Pro Rounded (`round`), kod ve lot için SF Mono (`mono`). Bu yazılar web'e gömülmez; web'de aynı yığın sistem yazısına düşer (macOS'ta SF, Windows'ta Segoe UI).
- Metin stilleri iOS Dinamik Yazı ölçeğidir: `t-large-title` sekme kökündeki büyük başlık, `t-headline` gezinme çubuğu başlığı, `t-body` hücre metni, `t-caption-1` hücre alt satırı.
- Rakamlar hizalanan her yerde eş genişlikli (`font-variant-numeric: tabular-nums`): değer sütunları, sayaçlar, tablolar.
- `t-timer` yalnız demleme sayacında, `t-operator-value` yalnız operatör görünümünde, `t-metric` özet kartlarında.

## Boşluk, köşe ve gölge

- 4 px tabanlı ölçek: `space-1` (4) ile `space-8` (32). Ekran kenarı ve hücre iç boşluğu `space-4`.
- Köşeler: ikon kutucuğu `radius-xs`, segment rayı ve küçük düğme `radius-sm`, liste, düğme ve uyarı kutusu `radius-md`, özet kartı ve büyük düğme `radius-lg`, anahtar `radius-pill`.
- Gölge yalnız iki yerde: segmentteki seçili parça (`shadow-seg`) ve anahtar topuzu (`shadow-knob`). Kartlar gölgesizdir; ayrım zemin farkıyla (`bg` / `card`) yapılır.

## Düzen ve gezinme

- iPad ve web: kenar çubuğu + içerik + gerektiğinde Inspector (Numbers gibi). iPhone: en fazla 5 sekme (`TabBar`), büyük başlık, seçiciler sheet olarak.
- Her ekran dört gezinme tipinden birindedir: sekme kökü, ileri itme (geri düğmesi önceki ekranın adını taşır), sheet (Vazgeç / Kaydet), tam ekran (demleme günü, sihirbaz; çıkış sol üstte). Sheet üstüne sheet açılmaz.
- Süren demleme her ekranda `BrewReturnBar` ile görünür.
- Web'de her ekranın kendi adresi vardır; üç seviyeden derin ekranlarda üstte iz yolu.
- Uygulamada sistem bileşenleri kullanılır (SwiftUI `TabView`, `NavigationSplitView`, `List`, `Toggle`, `Picker`); buradaki bileşenler web uygulaması ve konsept için aynı görünümü tanımlar. iOS 26'nın cam (Liquid Glass) görünümü yalnız gezinme katmanında ve sistemden gelir; içerikte ve operatör görünümünde cam kullanılmaz.

## Giriş düğmeleri ve iç araçlar

- "Apple ile devam et" ve "Google ile devam et" düğmeleri kendimiz çizilmez: Apple'da `SignInWithAppleButton`, web'de Apple'ın JS düğmesi; Google için Google'ın marka kurallarındaki düğme. İkisi aynı boyda, alt alta; Apple üstte. Yedek "E-postayla bağlantı al" düz metin bağlantısıdır.
- Destek paneli (iç kullanım) aynı token ve bileşenleri kullanır; ayırt etmek için adres çubuğunda ve başlıkta koyu "İÇ KULLANIM" etiketi bulunur. Müşteri ekranlarında bu etiket kullanılmaz.

## Operatör görünümü

- Üretim panosu, Bugün, tank detayı ve demleme günü için cihaz başına açılan görünüm (ISA-101). Normal durum gri ve renksizdir: `hp-bg`, `hp-tile`, `hp-ink`, `hp-ink2`.
- Renk yalnız sapmada: `AlarmMark` 1 acil (`hp-a1`, kare), 2 uyarı (`hp-a2`, eşkenar dörtgen), 3 bilgi (`hp-a3`, daire). Yanıp sönme yalnız onaylanmamış acil alarmda.
- Değerlerin yanında küçük ölçek: ray `hp-track`, normal aralık `hp-band`.
- Bildirim öncelikleri aynı üç seviyeyi kullanır.

## Baskı

- Yazdırılan formlar (brew sheet, tank turu, etiket) temadan bağımsızdır: her zaman beyaz kağıt üzerinde siyah. Kağıt boyutu bölgeye göre (US Letter, diğerleri A4). Formdaki QR kod taramayı doğru partiye bağlar.

## İkonlar ve logo

- Uygulamada Apple SF Symbols kullanılır; çizgi kalınlığı metin ağırlığını izler. Konsept ve buradaki önizlemeler SF Symbols'e benzeyen basit çizgi ikonlarla çizilmiştir; web uygulamasında SF Symbols lisansı web'e izin vermediği için ayrı bir ikon seti seçilmesi gerekir.
- Logo henüz yok; ürün adı düz yazıyla yazılır. Logo hazır olunca `assets/Logos/` grubuna eklenecek.
