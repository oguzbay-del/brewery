# FoamByte sitesi geneli ↔ Demleme repo: ek sayfa karşılaştırması

> **Claude için:** Bu rapor, daha önce incelenen `/features/`, `/calculators/` ve `/faqs/` sayfalarına ek olarak FoamByte ana sayfası, fiyatlandırma, ürün alt sayfaları ve duyuru/blog içeriklerini Demleme'nin ürün planıyla karşılaştırır. Her özellik için “planlı” ile “uygulanmış” ayrımını koruyun. Depo henüz çalışan ürün kodu içermiyor.
>
> **Araştırma sınırı:** FoamByte sayfalarının erişilebilen arama sonuçları ve sayfa özetleri kullanıldı; sayfanın tamamı ve uygulama içindeki canlı akışlar bağımsız olarak test edilmedi. Bazı FoamByte fiyat/limit özetleri birbiriyle tutarsız olduğundan, doğrulanmamış limit veya fiyatları kesin bilgi olarak kullanmayın. Yayımlamadan önce ilgili sayfaları doğrudan kontrol edin.

**İnceleme tarihi:** 7 Kasım 2026  
**Repo:** [`oguzbay-del/brewery`](https://github.com/oguzbay-del/brewery)  
**Karşılaştırılan ek FoamByte sayfaları:** [Ana sayfa](https://foambyte.com/), [Pricing](https://foambyte.com/pricing/), [Recipe Studio](https://foambyte.com/features/recipe-studio/), [Brewing Profiles](https://foambyte.com/features/brewing-profiles/), [Batch Tracking](https://foambyte.com/features/batch-tracking/), [Recipe Studio duyurusu](https://foambyte.com/blog/recipe-studio-is-now-live-built-with-brewers-not-behind-closed-doors/), [Batch Tracking duyurusu](https://foambyte.com/blog/batch-tracking-is-now-available-in-foambyte-free-brewing-software-plan/), [IoT duyurusu](https://foambyte.com/blog/foambyte-working-on-iot-device-integration-for-real-time-fermentation-tracking/).

## Yönetici özeti

FoamByte'ın kamuya açık sitesi, çalışır durumda olan Recipe Studio ve ücretsiz katmanda sunulan bazı özellikleri; ayrıca geliştirilmekte olan tank görünümü, envanter ve IoT entegrasyonlarını anlatıyor. Demleme repo'sunda tarif düzenleme, profiller, batch/fermantasyon, stok ve cihaz okumaları için geniş bir **ürün planı** var. Öne çıkan fark, FoamByte'ın belli özellikleri ürün olarak sunması; Demleme'nin ise hâlâ plan/tasarım/formül hazırlığında olmasıdır.

Plan kapsamı bakımından Demleme; ev ve küçük işletme modları, detaylı üretim akışları, kalite, lot izlenebilirliği, roller, iOS ve web ile FoamByte'ın yalın küçük üretici odağından daha kapsamlı bir hedef tarif ediyor. Bunun bedeli yüksek ilk sürüm kapsamı ve geliştirme/işletme karmaşıklığıdır.

**Önemli tutarsızlık:** Arama sonuçlarında FoamByte ücretsiz plan sınırları için farklı rakamlar çıktı (örneğin 100 tarif/5 batch ve başka bir sonuçta 20 tarif/2 batch). Bu rapor rakamları doğrulanmış kabul etmiyor; limitler fiyat sayfasından doğrudan teyit edilmeden ürün kararına veya pazarlama metnine taşınmamalı.

## Sayfa bazında karşılaştırma

| FoamByte sayfası / iddiası | Demleme reposundaki karşılık | Değerlendirme |
|---|---|---|
| **Ana sayfa — ev biracıları ve nano/küçük bira üreticileri için yazılım; tarif, hesaplama, parti yönetimi; sade kullanım ve erişilebilir fiyat vurgusu** | Plan aynı şekilde ev biracıları ve küçük bira fabrikalarını hedefliyor; Ev/İşletme modları, tarif/hesap/üretim kapsamı var. | **Hedef kitle ve ana problem örtüşüyor.** Demleme'nin daha geniş işletme vizyonu “daha sade” olacağı anlamına gelmiyor; sade Ev modu ile geniş İşletme kapsamı arasında ürün sınırı korunmalı. |
| **Recipe Studio — tarif oluşturma/yönetme ve hesapları tarif akışına katma** | Tarif sihirbazı ve editörü; hedef OG/IBU/renk, ölçekleme, malzeme kütüphanesi, stil kılavuzu, BeerXML/BeerJSON ve bazı diğer formatlardan içe aktarma planı var. | **Planlı karşılık güçlü.** FoamByte'ın gerçek editör akışıyla kullanılabilirlik/eşdeğerlik kıyaslaması yapılamaz; Demleme kodlanmış değil. |
| **Recipe Studio — geniş malzeme kataloğu ve BJCP stil bilgisi (site/duyuru özetlerinde belirtiliyor)** | Malzeme kütüphanesi ve BJCP/BA/stil kılavuzları planlanıyor; özel malzeme ve lota özgü değerler de düşünülmüş. | **Kapsam planlı; içerik ve lisans doğrulaması gerekli.** Malzeme sayısı, veri kalitesi, güncelleme yöntemi ve lisanslar repo'da FoamByte ile kıyaslanacak sayısal katalog olarak belirlenmemiş. |
| **Recipe Studio — AI tarif oluşturucu “yakında” şeklinde duyuru/arama özetlerinde geçiyor** | Planın analitik modeller bölümünde doğal dille kullanıcının kendi verisine soru sorma fikri var; ilk sürüm için karar gerekli. Bu, tarif üreten AI aracıyla aynı özellik değil. | **Açık boşluk.** Demleme'de onaylanmış AI tarif üreticisi planı tespit edilmedi. İstenirse ürün kararı olarak ayrıca ele alınmalı; reçete doğruluğu, açıklanabilirlik ve kullanıcı onayı gerektirir. |
| **Brewing Profiles — brewhouse, su ve karbonasyon profilleri** | Ekipman, mash, kaynatma, fermantasyon, karbonasyon profilleri; hacim ve kayıp parametreleri; ekipman kalibrasyon önerileri planlanmış. | **Planlı karşılık güçlü.** Demleme'nin kalibrasyon asistanı hedefi ek farklılaşma olabilir, ama uygulama/test ve kullanıcı verisi henüz yok. |
| **Batch Tracking — brew day'den karbonasyona batch aşamaları, manuel SG/sıcaklık/pH okumaları, eklemeler ve fermantasyon grafikleri** | Parti durumları, plan/gerçekleşen, adım ve ölçümler, manuel/cihaz/kâğıt kaynakları, uyarılar, fermantasyon/ambalaj akışları planlanmış. | **Planlı karşılık güçlü ve kapsamı geniş.** FoamByte'ın batch takibi ürün olarak duyurulmuş; Demleme'de henüz çalışan batch akışı yok. |
| **Batch Tracking — ücretsiz planda sınırlı sayıda batch takibi duyurusu** | Demleme planında batch yeteneklerinin Ev/İşletme modlarına göre kapsamı var, ancak özelliklerin fiyat katmanlarına dağılımı kesin değil. | **Ürün kararı açık.** Batch limitleri ve ücretsiz/ücretli özellik kapıları belirlenmeli. FoamByte'ın güncel limitleri de sayfadan teyit edilmeli. |
| **Tank View — tank görünürlüğü, parti ve elle kalite okuma; sayfada test/geliştirme durumu belirtiliyor** | İşletme üretim panosu, tank görünümü/takvimi, tank atama ve hareketleri, kalite, CIP, operatör ekranı planlanmış. | **Planlı karşılık geniş; ürün olgunluğu kıyası yok.** FoamByte özelliği de kaynak sayfasında testte/gelişmekte olarak anlatılıyorsa bu durumu canlı ve tamamlanmış işlev saymamak gerekir. |
| **Inventory — stok, tüketim ve batch ile ilişki; sayfa/FAQ'da mevcut veya geliştirme kapsamı farklı biçimde sunulabiliyor** | Evde basit stok ve alışveriş listesi; işletmede lot, tedarikçi, rezervasyon, FIFO ve izlenebilirlik planlanmış. | **Planlı karşılık geniş.** FoamByte'ın envanter özelliğinin mevcut mu yol haritasında mı olduğu sayfalar arasında teyit edilmeli; stok lokasyonu/lot davranışı ayrıntı düzeyinde kıyaslanmadı. |
| **IoT fermentation tracking — gerçek zamanlı sensör okumaları, TILT ile başlama/entegrasyon geliştirme duyurusu** | Tilt, iSpindel ve RAPT okumaları; webhook, cihaz anahtarı, tank eşleştirme ve kalibrasyon planlanmış. Profesyonel sensör/ağ geçidi daha ileri fazda. | **Planlı karşılık var; entegrasyon kapsamı ve fazı farklı olabilir.** Cihaz desteği, bağlantı yöntemi, gecikme, kopma ve veri biriktirme davranışları uygulama prototipiyle doğrulanmalı. |
| **Platform — web tarayıcısı mevcut, mobil uygulama geliştirme aşamasında olarak tanımlanıyor** | Tam özellikli web ve native iOS/iPadOS hedefleniyor; Apple Watch sonraki opsiyonel fikir. | **Planlı karşılık var, farklı platform önceliği.** Linux/Chromebook tarayıcı desteği Demleme planında açık bir uyumluluk hedefi olarak yazılmamış; tarayıcı uyumluluk matrisi eklenmeli. |
| **Fiyatlandırma — ücretsiz ve ücretli kademeler, kullanım limitleri/özellik ayrımı** | Bir aylık tam deneme ve abonelik ekranı düşünülmüş; deneme sonrası verinin korunması planlanıyor. Fiyat ve ücretsiz katman sınırları açık karar. | **Kısmi karşılık; en önemli ürün boşluğu.** Ücretsiz plan içeriği, tarif/profil/batch limitleri, ekip kullanıcısı ve dışa aktarma kuralları kararlaştırılmalı. |
| **Ürün geliştirme yaklaşımı — erken kullanıcı geri bildirimiyle ürün geliştirme** | Plan, kullanıcı şikâyetleri ve ürün araştırmasına atıf yapıyor; ancak beta/erken erişim programının kapsamı, kullanıcı sayısı ve karar mekanizması belirgin değil. | **Kısmi karşılık.** Kullanıcı doğrulaması yapılmış gibi ifade edilmemeli; araştırma varsayımları ile gerçek beta bulguları ayrılmalı. |
| **Topluluk / tarif paylaşımı yönelimi** | Demleme planında herkese açık tarif paylaşımı ve topluluk özellikleri kapsam dışı bırakılmış; sınırlı çalışma alanı/kişiler arası davetler var. | **Bilinçli ürün farkı.** FoamByte'ın topluluk vizyonu ile eşleşme hedeflenmiyor. Bunu eksik özellikten ziyade açık kapsam kararı olarak tutun. |

## Fiyat ve limitler: doğrulama notu

Arama sonuçlarının bir kısmı FoamByte için ücretsiz planda tarif/profil/batch limitleri verirken diğer sonuçlar farklı sayılar döndürdü. Özetler ayrıca ücretli plan ayrıntıları konusunda tutarlı değildi. Bu nedenle:

- Bu raporda kesin FoamByte limiti veya fiyatı yazılmamıştır.
- `foambyte.com/pricing/` sayfasının güncel tablosu ve mümkünse kayıtlı hesaptaki plan arayüzü doğrudan kontrol edilmeden rakam karşılaştırması yapılmamalıdır.
- Demleme tarafında da “ücretsiz”, “ücretli”, “deneme sonrası salt okunur” gibi kararlar özellik matrisiyle tutarlı hale getirilmelidir.
- Ürün karşılaştırmasında bir sayfanın güncel metni, blog duyurusu ve eski tanıtım içeriği çelişirse tarih ve güncel ürün ekranı esas alınmalıdır.

## Repo tarafında eksik veya karar gerektiren konular

1. **Ücretsiz/ücretli katman matrisi:** tarif, profil, hesap makinesi, batch, envanter, kullanıcı/koltuk, dışa aktarma, geçmiş ve destek hangi planda?
2. **AI özellik sınırı:** doğal dille veri sorgulama fikri, AI tarif oluşturma değildir. Bu iki kullanım senaryosu ayrı karar ve risk değerlendirmesi gerektirir.
3. **Tarayıcı desteği:** Web hedefinde iPad/PC/Mac var; Linux ve Chromebook için destek/QA hedefi belirlenmeli.
4. **Cihaz entegrasyonu:** hobi cihazlarının tam listesi, TILT dahil her modelin bağlantı yolu ve MVP fazı netleştirilmeli.
5. **Katalog ve stil verisi:** malzeme sayısı/kalitesi, içe aktarma güncelleme sıklığı, kaynak ve lisanslar doğrulanmalı.
6. **Kullanıcı doğrulaması:** prototip testleri ve beta geri bildirimi için ölçülebilir bir plan tanımlanmalı.
7. **Kapsam sadeleştirme:** FoamByte'ın küçük üreticiye yönelik yalın konumlandırması karşısında Demleme'nin işletme/kalite/mevzuat kapsamı daha ağır. Ev MVP ile İşletme özelliklerini ayrı sürüm hedeflerinde tutmak önemli.

## Sonuç

FoamByte sitesinin daha geniş sayfa kümesi incelendiğinde, Demleme planında tarif, profil, parti/fermantasyon, tank, envanter ve cihaz okuma alanlarında kapsam karşılığı görülüyor. Buna karşılık FoamByte bazı işlevleri hâlihazırda ürün olarak sunduğunu duyururken Demleme repo'su hâlâ plan ve tasarım deposu. En belirgin plan boşlukları fiyat/katman matrisi, AI tarif üretimi kararı, açık platform uyumluluğu ve doğrulanmış katalog/cihaz kapsamıdır. Herhangi bir özellik paritesi iddiası kurulmadan önce gerçek uygulama geliştirilip test edilmelidir.

## Kaynaklar

- [FoamByte ana sayfa](https://foambyte.com/)
- [FoamByte fiyatlandırma](https://foambyte.com/pricing/)
- [FoamByte Recipe Studio](https://foambyte.com/features/recipe-studio/)
- [FoamByte Brewing Profiles](https://foambyte.com/features/brewing-profiles/)
- [FoamByte Batch Tracking](https://foambyte.com/features/batch-tracking/)
- [FoamByte Recipe Studio duyurusu](https://foambyte.com/blog/recipe-studio-is-now-live-built-with-brewers-not-behind-closed-doors/)
- [FoamByte ücretsiz planda batch tracking duyurusu](https://foambyte.com/blog/batch-tracking-is-now-available-in-foambyte-free-brewing-software-plan/)
- [FoamByte IoT entegrasyonu duyurusu](https://foambyte.com/blog/foambyte-working-on-iot-device-integration-for-real-time-fermentation-tracking/)
- [Demleme README](https://github.com/oguzbay-del/brewery/blob/main/README.md)
- [Demleme ürün planı](https://github.com/oguzbay-del/brewery/blob/main/plan/demleme-plani.md)
- [Claude çalışma kuralları](https://github.com/oguzbay-del/brewery/blob/main/CLAUDE.md)
