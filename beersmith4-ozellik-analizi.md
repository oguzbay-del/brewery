# BeerSmith 4.0.22 — Özellik ve İşleyiş Analizi

Kaynak: `BeerSmith_4.0.22.dmg` (macOS, Intel + Apple Silicon). Analiz, paketteki **kaynak dosyalarından** (menü/ayar/alan tanım veritabanı `Resources.sqlite`, veri şeması `BeerSmith.sqlite`, rapor şablonları) yapıldı. Çalıştırılabilir dosya decompile edilmedi; formüllerin iç uygulaması bu yüzden bilinmiyor, yalnızca hangi seçeneklerin ve girdilerin olduğu biliniyor.

---

## 1. Mimari (dışarıdan gözlenen)

- **Masaüstü uygulaması**, tek binary; tüm UI metinleri, menüler, ayarlar ve alan açıklamaları **SQLite tablosunda** (`Resources.sqlite`: 523 menü, 219 ayar, 1593 alan, 1175 metin). Dil dosyası da ayrı bir SQLite ile değiştiriliyor → çeviri kolaylığı.
- **Veri modeli**: her varlık için bir tablo (`M_RECIPE`, `M_HOPS`, `M_GRAIN`, `M_YEAST`, `M_MISC`, `M_WATER`, `M_MASH`, `M_STYLE`, `M_EQUIPMENT`, `M_CARB`, `M_AGE` (fermantasyon), `M_SHOP`, `M_INVENTORY`, `M_NOTES`, `M_FOLDER`).
  - Her tablonun bir **`M_CLOUD_*` ikizi** var → yerel/bulut ayrımı tablo düzeyinde.
  - Her kayıtta ortak alanlar: `_PERMID_` (kalıcı ID), `_MOD_` (değişiklik), `_CLOUDID_`, `_CLOUD_STATE_` (senkron durumu), `_EXTRA_`.
  - **Araçların son girdileri bile tabloda saklanıyor** (`M_TOOL_*`, 30+ tablo).
- **Raporlar şablon tabanlı**: HTML şablonlar (`Brewsheet.htm`, `MeadBrewsheet.htm`, `CiderBrewSheet.htm`, `ExtractBrewsheet.htm`, `Competition.htm`, `Shop.htm`…) içinde `$EST_OG`, `$MASH_STEPS`, `$YEAST_STARTERS` gibi değişkenler. Kullanıcı kendi rapor şablonunu ekleyip dışa aktarabiliyor.
- **BeerXML dışa aktarma** da aynı şablon sistemiyle (`Recipe.xml`, `Hops.xml`… `$NAME`, `$BATCH_SIZE`…).
- Dosya uzantısı: `.bsmx`.

**Kendi uygulamanız için çıkarım:** UI metinlerini/alan tanımlarını veri olarak tutmak, yerel+bulut senkronunu her kayıtta ID + değişiklik zamanı + senkron durumu ile yönetmek ve raporları şablonla üretmek iyi kanıtlanmış bir yaklaşım.

---

## 2. Özellik envanteri

### 2.1 Tarif tasarımı
- İçecek türleri: **bira (all-grain, extract, partial mash), mead, şarap, cider**.
- Sekmeler: Design, Mash, Water, Yeast Starter, Fermentation, Session, Notes, Vols (detaylı su hacimleri).
- **Tersine ayar araçları**: *Adjust Gravity* (hedef OG), *Adjust Color*, *Adjust Bitterness* (hedef IBU), *Grain Pct* (yüzdeyle tahıl), *Hop IBUs* (hop başına IBU hedefi).
- **Convert Recipe**: all-grain ↔ extract dönüşümü (varsayılan ekstre/baz malt ayarı, "mashable'ları da dönüştür").
- **Scale Size**: boyut, verim veya ekipmana göre ölçekleme + ölçekleme sonrası yeniden dengeleme.
- **Substitute**: malzemeyi başka bir malzemeyle değiştirme; **Round Up Amounts**.
- Miktar/süre artır-azalt düğmeleri (hop, tahıl, misc için ayarlanabilir adım).
- **Check Recipe**: tarifin eksiksiz ve tutarlı olup olmadığını denetleme.
- **Lock/Unlock**, sürüm numarası, **Recipe Archive** (değişiklik geçmişi; ekleme/silme/düzenleme/taşıma ayrı ayrı arşivlenebiliyor, kurtarma).
- **Özel alanlar** (custom fields) ve tarif görseli, içecek rengi seçimi.
- **Maliyet**: malzeme fiyatları, tarif toplam maliyeti, envanterden fiyat güncelleme, para birimi ayarı.
- Kalori (tahmini ve ölçülen), UK vergi ABV tablosu.

### 2.2 Hop ve IBU
- Formül seçimi: **Tinseth, Rager, Garetz**.
- Kullanım düzeltmeleri: pellet, leaf, plug, **first wort**, **mash hop**.
- **Whirlpool/steep**: hop başına ortalama whirlpool sıcaklığı, ekipmanda toplam whirlpool süresi, "kaynatma hoplarının whirlpool'daki kalan katkısını hesapla", steep/whirlpool kullanım faktörü.
- Dry hop: başlangıç günü ve faz.
- **Hop Age** aracı: depolama koşulu + HSI'ya göre alfa kaybı.

### 2.3 Mash ve su
- Hazır mash profilleri: tek/çift infüzyon, sıcaklık (step) mash, 1-2-3'lü dekoksiyon, **BIAB**, **RIMS/HERMS**, batch sparge, mash-out'suz.
- BIAB'a özel tahıl emilimi, full-boil mash hacmi.
- Tahıl/kap ısı kapasitesi, tun ağırlığı, deadspace, ekipmana göre ayar.
- **Mash sıcaklığına göre FG düzeltmesi** (eğim ve medyan sıcaklık ayarlanabilir).
- **Su**: profil eşleştirme (hedef profil seç → tuz eklemelerini en iyi uyumla hesapla), seyreltme suyu, mash/sparge ayrı tuz eklemeleri.
- **Mash pH**: tahmini pH'tan veya **pH metre ölçümünden** asit hesabı; asit malt veya laktik/fosforik asit ekleme; asit modeli seçimi; sparge suyu asitlendirme.
- Alkalinite/bikarbonat girdisi.

### 2.4 Maya
- **Canlılık (viability)**: üretim tarihi → demleme tarihi; kuru/sıvı/slurry için başlangıç canlılığı ve aylık kayıp oranı ayarlanabilir.
- **Starter hesabı**: stir plate, ikinci kademe starter, starter OG/hacmi, hücre/ml, kuru maya hidrasyon suyu, GoFerm.
- Şarap/mead için ayrı pitch rate.

### 2.5 Mead / şarap / cider
- Bal, meyve, meyve suyu fermentable türleri (Brix/SG girişi).
- **Mead Nutrients** aracı (yöntem seçimi, Fermaid-K toplamı, besin takvimi, "besinleri malzemelere ekle").
- **Sulfite** aracı: pH'a göre önerilen serbest SO2, metabisülfit miktarı.
- **Backsweeten** aracı.
- Ayrı brewsheet şablonları ve stil kılavuzları (BJCP 2015 mead/cider kategorileri dahil).

### 2.6 Demleme günü (Session)
- **Brew sheet**: adım adım talimatlar (mash/sparge adımları, asit metni, su hazırlama, maya starter, kademeli fermantasyon, şişeleme).
- Özel adım ekleme.
- **Zamanlayıcılar**: mash/steep, kaynatma, whirlpool; ayrı alarm, özel alarm sesi.
- **Session sekmesi**: ölçülen değerler (mash sonrası SG, kaynatma öncesi hacim/SG, OG, FG, primary/secondary sonrası SG, şişeleme hacmi, mash pH) → **mash verimi, brewhouse verimi, gerçek ABV, attenuation, gerçek ekstrakt** hesapları.
- "New Session" diyaloğu: Brew Log klasörüne kopya, tarihi güncelle, ölçümleri sıfırla, sürümü artır.
- Ölçüm alanlarında **açılır refraktometre**.

### 2.7 Fermantasyon takibi
- Fermantasyon profilleri: primary/secondary/tertiary/bulk/aging, her birinde başlangıç-bitiş sıcaklığı ve gün.
- **Fermentation Data**: okuma ekleme, grafik, CSV içe/dışa aktarma.
- **Tilt hidrometre** entegrasyonu (Google Sheets bağlantısı veya CSV, filtre aralığı).
- Karbonasyon profilleri (şişe/fıçı, priming şekeri, fıçı priming faktörü).

### 2.8 Envanter, alışveriş, takvim
- **Envanter**: malzeme bazında stok; tariften stoğa ekle/stoktan düş; malzeme seçerken "yalnızca stoktakileri göster" filtresi.
- **Alışveriş listesi**: tarifteki malzemeleri sepete ekle, sipariş eşiği, birim fiyat, toplam maliyet.
- **Takvim**: demleme tarihleri; klasöre/demleyene göre filtreleme.
- **Notes**: başlangıç/bitiş tarihi ve öncelikli notlar (görev listesi gibi).

### 2.9 Bağımsız araçlar (Tools)
Alkol ve attenuation (kalori dahil) · Hop Bitterness · Boil Off · Carbonation · Decoction Volume · Dilution · Adjust Gravity (ekstre ile) · Hop Age · Hop Rate · Hydrometer Adjust · Infusion/Strike · Mash Adjust · Mash pH · Mead Nutrients · **Parti-Gyle** · Refractometer (kalibrasyonlu) · Sulfite · Backsweeten · Water Needed · Water Profile · Weight-to-Volume · Yeast Starter · birim dönüştürücüler (sıcaklık, hacim, ağırlık, basınç, SG/Plato/Brix, renk, karbonasyon).

### 2.10 Bulut ve paylaşım
- **Cloud veya Local depolama modu** (ilk açılışta seçiliyor).
- Tüm malzeme/profil tipleri için bulut ikizleri; seçili öğeleri bulutla değiş tokuş; otomatik senkron aralığı; proxy desteği.
- Tarifleri **paylaşımlı/özel** yapma, paylaşım ID'si ve URL'si, **puanlama ve yorum**, tarifi yorumları kaybetmeden güncelleme.
- **Cloud Search**: tür (bira/şarap/cider/partial mash), sıralama, sayfalama.

### 2.11 Diğer
- **Add-on** sistemi: hop, maya, tahıl, misc, su, stil, ekipman, mash, karbonasyon, fermantasyon paketleri.
- **Equipment Wizard** (adım adım ekipman profili oluşturma) ve karşılama ekranı / başlangıç rehberleri.
- Yarışma raporu (kulüp, adres, telefon), özel rapor şablonları, CSV rapor.
- Yedekleme: otomatik kaydetme, otomatik yedekten kurtarma, **ZIP tam yedek/geri yükleme**, BeerSmith 1.4/2/3'ten içe aktarma.
- Arayüz: sekmeli/pencereli görünüm, önizleme paneli (alt/sağ), favoriler, yer imleri çubuğu, açık/koyu tema, özelleştirilebilir sütunlar, alan yanında **gösterge topları** (stile uygunluk).
- 21 günlük deneme, lisans aktivasyonu/deaktivasyonu.

---

## 3. Brewtarget ile karşılaştırma: BeerSmith'te olup Brewtarget'ta olmayanlar

| Alan | Eksik özellik |
|---|---|
| Tarif | Hedef OG/renk/IBU'ya ayarlama, yüzdeyle tahıl, all-grain↔extract, malzeme ikamesi, Check Recipe, özel alanlar, tarif maliyeti |
| Hop | Garetz; whirlpool sıcaklığına bağlı kullanım; kaynatma hoplarının whirlpool katkısı; hop yaşlanması |
| Mash | Mash sıcaklığına göre FG düzeltmesi; pH ölçümünden asit hesabı; dekoksiyon hacmi aracı |
| Maya | Tarihe bağlı canlılık; tarife bağlı starter (2 kademe, stir plate) |
| Mead/şarap/cider | Tamamı (besin, sülfit, tatlandırma, stiller, brewsheet) |
| Fermantasyon | Okuma grafiği, CSV, **Tilt entegrasyonu** |
| Planlama | Alışveriş listesi, takvim, not/görev listesi |
| Araçlar | Parti-gyle, dilution, boil off, water needed, weight-to-volume, kalori |
| Ekosistem | Bulut senkron, paylaşım/puanlama, add-on, equipment wizard |

---

## 4. Kendi uygulamanız için öneri: MVP kapsamı

1. **Çekirdek hesaplama motoru** (açık formüllerle, testli): OG/FG/ABV, Tinseth+Rager+Garetz IBU, whirlpool sıcaklık modeli, Morey renk, su kimyası ve mash pH, karbonasyon, starter + canlılık.
2. **Tarif editörü** + tersine ayar araçları (hedef OG/renk/IBU, yüzde) ve Check Recipe — BeerSmith'te var, Brewtarget'ta yok; kullanıcıların en çok kullandığı kolaylıklar.
3. **Session/brew day** ekranı: ölçümler, verim hesabı, zamanlayıcılar — mobilde çok değerli.
4. **Yerel + bulut senkron** (kayıt başına ID, değişiklik zamanı, senkron durumu) ve **web/mobil** öncelikli tasarım — BeerSmith masaüstü-öncelikli, burası farklılaşma alanı.
5. **BeerXML + BeerJSON** içe/dışa aktarma (BeerSmith kullanıcılarını taşımak için).
6. Sonra: envanter/alışveriş, Tilt ve diğer dijital hidrometreler (iSpindel, RAPT), mead/cider modülü, topluluk paylaşımı.

---

## 5. Windows paketinden ek bilgiler (`BeerSmith_4.0.22.exe`)

- Kurulum paketi NSIS ile yapılmış; arayüz kütüphanesi **wxWidgets** (lisans dosyasında wxWindows lisansı var), Visual C++ runtime ile geliyor.
- Veritabanları (`Resources.sqlite`, `BeerSmith.sqlite`, `DefRecipe.sqlite`) ve rapor şablonları Mac sürümüyle **birebir aynı**. Windows'a özel yalnızca `ReadMe.txt` (BeerSmith 4 sürüm notları) ve lisans metni.

### Sürüm notlarından gelen ek özellikler

**Tarif**
- **Seltzer** yeni içecek türü (bira, mead, cider, şarap yanında).
- Secondary/tertiary fermentable eklemeleri için **"Est OG Through Tertiary"** (sanal OG) → FG ve ABV doğru hesaplanıyor.
- Ayar diyaloglarında (renk, gravity vb.) **kaydırıcılar**.
- Whirlpool'a misc malzeme ekleme; hop oranı (g/l, lb/bbl, oz/gal) özel alan olarak.
- Yeni birim ayarları: sıvı/kuru maya birimleri, su/tahıl oranı, kalori, hop oranı, karbonasyon g/l.

**Maya**
- Maya miktarı gram veya ml olarak.
- Sıvı maya: hedef **hücre/ml-°Plato** veya **milyon hücre/ml** ile pitch rate ayarı.
- Kuru maya: üreticinin önerdiği pitch rate, ondan hücre/ml tahmini; kuru maya için de starter.
- Güncellenmiş üreme (propagation) modeli, stir plate verisi, iki kademeli starter dengesi.

**Su**
- Su hesabı **molar kütlelerden** yeniden yazılmış (önceki sürümde CaCl2 ve tebeşir faktörleri hatalıymış).
- Yeni tuzlar: **pickling lime** (Ca(OH)2), **MgCl2**; tebeşir/kireç hariç tutma.
- Mash ve sparge eklemelerini tek sette birleştirme.
- Su profilini **mash, kaynatma ve fermentöre giriş** için ayrı ayrı tahmin.
- pH ayarı iki düğmeye indirilmiş (tahmini pH'tan / ölçümden). Bir pH modeli adı geçiyor: **"BNW"**.

**Fermantasyon**
- Fermantasyon grafiğinde OG ve sıcaklığın yanında **pH** takibi.

**Envanter**
- Envanter malzeme tablolarından ayrılmış; alışveriş/envanter işlemlerinde **tek tek öğe seçimi**.
- Tarifin "envanterden düşülme tarihi" ve kısa sürede iki kez düşmeye karşı uyarı.

**Araçlar**
- Alkol aracı karbonhidrat ve kalori de veriyor; hacim aracında yaygın **fıçı boyutları**.

**Güvenilirlik**
- Her kayıtta anında commit, periyodik tam yedek, tablo bazında kurtarma; tarif arşivi ayrı veritabanında; şifreler şifreli.

### Kendi uygulamanız için ek çıkarımlar

1. **Su kimyasını baştan doğru kurun:** tuz faktörlerini molar kütlelerden hesaplayın ve testlerle doğrulayın. BeerSmith bile yıllarca hatalı faktör kullanmış.
2. **Sanal OG kavramı:** fermantasyon sırasında eklenen şekerleri (meyve, bal vb.) ABV'ye doğru yansıtmak için gerekli.
3. **Maya modeli:** pitch rate (hücre/ml-°P) + canlılık + üreme modeli üçlüsü profesyonel kullanıcılar için önemli.
4. **Seltzer** düşük eforlu, pazarda talep gören bir tür.
