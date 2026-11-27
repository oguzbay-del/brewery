# G8 — Marka ve ürün tutarlılığı

27 Kasım 2026 — Taslak inceleme. Referans: [ADR-0012](../../adr/0012-urun-markasi.md), onaylı logo [#148](https://github.com/oguzbay-del/brewery/pull/148), [G5 #429](https://github.com/oguzbay-del/brewery/pull/429) ve [G6 #431](https://github.com/oguzbay-del/brewery/pull/431) (ikisi de birleştirilmiş).

## 1. Uyumsuzluklar ve öneriler

| Bulgu | Kaynak | Öneri |
|---|---|---|
| iOS görünen adı GRISTORA, ancak uygulama bundle kimliği hâlâ `app.demleme.Demleme`. ADR-0012 §4 hedefi `com.gristora.app`. | [ios/project.yml](../../ios/project.yml), ADR-0012 §4 | İlk TestFlight yüklemesinden önce uygulama ve test hedeflerinin kimliklerini ADR ile eşleştir; imzalama, Apple giriş ve universal link etkilerini Claude ile doğrula. İç proje ve hedef adlarının değişmesi gerekmez. |

## 2. Doğrulanmış ortak öğeler

| Öğe | Web uygulaması | iOS | Pazarlama |
|---|---|---|---|
| Görünen ad | [index.html](../../web/index.html): GRISTORA; [manifest](../../web/public/manifest.webmanifest): name ve short_name GRISTORA | [project.yml](../../ios/project.yml): INFOPLIST_KEY_CFBundleDisplayName GRISTORA | [BRAND-ASSETS.md](../../marketing-site/BRAND-ASSETS.md): GRISTORA |
| Favicon | [favicon.svg](../../web/public/favicon.svg) | AppIcon ayarı var; varyantların cihazda görsel doğrulaması açık | [favicon.svg](../../marketing-site/public/favicon.svg); web uygulamasındaki SVG içeriğiyle birebir aynı |
| Logo | #429: başlık sembolü ve duyarlı yazı | #431: açık, koyu ve renklendirilmiş ikon, launch screen; cihaz testi henüz doğrulanmadı | [BRAND-ASSETS.md](../../marketing-site/BRAND-ASSETS.md): #148 onaylı varlıklar; gerçek rota kullanımı ayrıca incelenecek |

## 3. Ürün iddiaları

İddiaların rota bazlı envanteri ve her birinin uygulama/API veya ISLER satırı ile eşlemesi henüz tamamlanmadı. Bu taslak ürünün yayınlanabilirliğini veya özelliklerin mevcut olduğunu onaylamaz.

İnceleme devamında EN/TR rotaları, özellik metinleri, ekran görüntüleri, sunucu uçları ve gerçek kullanıcı akışları kontrol edilecek. Pazarlama metni veya uygulama kodu bu rapor kapsamında değiştirilmez.

### 3.1 İlk özellik kataloğu karşılaştırması

Kaynak: [yayındaki özellik sayfalarının veri dosyası](../../marketing-site/src/data/features.ts). Uygulama tarafındaki ilk kanıt: [API modül envanteri](../../sunucu/crates/api/CLAUDE.md). API kanıtı, web veya iOS ekranının tamamlandığı veya ürünün yayında olduğu anlamına gelmez. Aşağıdaki satırlar grup düzeyinde ilk eşlemedir; kalan madde ve rota denetimi açık.

| Özellik rotası | Kodda belgelenen karşılık | Açık iddialar ve öneri |
|---|---|---|
| recipe-design | T2.21: [recipes.rs](../../sunucu/crates/api/src/recipes.rs), [recipe_org.rs](../../sunucu/crates/api/src/recipe_org.rs): sürüm, yayınlama, ölçekleme önizlemesi, karşılaştırma, klasör, etiket, favori ve yumuşak kilit | Başlangıç şablonları, ekstrakt dönüşümü, tüm stil rehberleri, özel alanlar ayrı doğrulanmalı. T2.26 BeerXML/BeerJSON/Brewfather JSON aktarımı var; BSMX örnek dosya bekliyor; diğer formatlara uyumluluk iddiası yok. |
| brewing-calculations | T2.16: [engine.rs](../../sunucu/crates/api/src/engine.rs): sunucu motoruna gRPC, sürüm damgası ve hesap sonucu kaydı | Her özel hesap modeli, yöntem seçimi, sonuç açıklaması ve referans doğrulaması ayrı kontrol edilmeli. |
| water-mash-yeast | Motor bağlantısı T2.16; bu alanın her işlevi için doğrudan kanıt henüz okunmadı | Su raporu, tuz/asit, mayşeleme ve maya hazırlığı maddelerini tek tek doğrula. |
| brew-day | T2.22: [batches.rs](../../sunucu/crates/api/src/batches.rs), [brew_day.rs](../../sunucu/crates/api/src/brew_day.rs): onaylı tariften parti, adımlar, ölçüm, sapma, imzalı paket | Live Activity, zamanlayıcı, baskı, QR, belge tarama ve OCR kullanıcı akışları henüz doğrulanmadı. T2.18 [files.rs](../../sunucu/crates/api/src/files.rs) yalnız belge saklama uçlarının kanıtı. |
| fermentation-monitoring | T2.22 parti aşamaları ve ölçümleri; T2.25 [devices.rs](../../sunucu/crates/api/src/devices.rs); T2.24 [notifications.rs](../../sunucu/crates/api/src/notifications.rs) | Grafikler, kaynak etiketleri, öneriler, widget, vardiya ve tadım akışları ayrı doğrulanmalı. |
| equipment-calibration | T2.37 [trash.rs](../../sunucu/crates/api/src/trash.rs) ekipman geri yükleme desteğini belgeler; profil ve kalibrasyon ayrıntıları henüz doğrulanmadı | Tank basıncı, doluluk, temizlik ve kalibrasyon özellikleri için doğrudan kanıt gerekli. |
| planning-scheduling | T2.22: takvim ve geriye planlama | Isıtma/soğutma tahmini, kapasite optimizasyonu, kişiselleştirilmiş üretim panosu henüz doğrulanmadı. |
| inventory-costing | T2.23 [stock.rs](../../sunucu/crates/api/src/stock.rs): lot, hareket defteri, giriş/tüketim/sayım, eski lottan düşüm, alışveriş listesi | FIFO tüketim tek başına FIFO maliyetleme kanıtı değildir. Barkod, COA, maliyet, rezervasyon ayrı kontrol edilmeli. |
| production-management | T2.22 parti/adım/ölçüm; T2.23 stok hareketleri | Tank transfer/bölme/birleştirme, geri çağırma, temizlik uyarısı ve vardiya panosu ayrı incelenmeli. ERP kapsam dışı ifadesi korunmalı. |
| quality-laboratory | Henüz doğrudan kalite modülü kod kanıtı bu denetimde okunmadı | Kalite limitleri, laboratuvar, serbest bırakma, HACCP ve çift onay için kanıt bulunmadan mevcut özellik olarak tanıtma. |
| packaging-compliance | T2.22 parti aşamaları; paketleme ve CIP ayrıntısı henüz doğrulanmadı | Vergi/ülke uyumu iddiaları için mevzuat ve uygulama kanıtı gerekir. |
| team-notifications | T2.13 rol/üye/davet/Home paylaşımı; T2.12 ortak cihaz/PIN; T2.24 bildirim kutusu, tercihler ve uyarı kuralları | Mod geçişi, rol kapsamı, vardiya ve onay kuyrukları ayrı doğrulanmalı. |
| connected-devices | T2.25 cihaz kaydı, parti eşleme, anahtar yenileme; T2.24 uyarılar | Belirli marka ve protokol bağlantıları doğrulanmadı; salt okunur ve geleceğe dönük ifadeler korunmalı. |
| imports-exports-tools | T2.26 [import_export.rs](../../sunucu/crates/api/src/import_export.rs): üç doğrulanmış aktarım biçimi, aktarımı uygulama, dışa aktarma işi ve indirme bağlantısı | BSMX açık; diğer araç ve biçimler tek tek doğrulanmalı. |

| platform-security-data | T2.11 Apple/Google/e-posta oturumu, T2.12 passkey ve ortak cihaz, T2.13 izinler, T2.16 motor, T2.27 arama, T2.28 [subscription.rs](../../sunucu/crates/api/src/subscription.rs), T2.29–30 destek: [API envanteri](../../sunucu/crates/api/CLAUDE.md) | Tam özellikli iPad/web, offline kuyruk, Siri, widget, yedekleme ve tüm dışa aktarma/silme akışları kullanıcı uçtan uca testleriyle doğrulanmalı. Platform kapsam dışı listesi mevcut özellik iddiası değildir. |
| analytics-reports | T2.22 [batches.rs](../../sunucu/crates/api/src/batches.rs) plan/gerçek ölçüm kaydı; T2.23 [stock.rs](../../sunucu/crates/api/src/stock.rs) maliyet alanları; K2 [containers.rs](../../sunucu/crates/api/src/containers.rs) kap, dolum ve geri çağırma | Trend, verim, litre maliyeti, tank kullanım ve üretim KPI'ları için doğrudan rapor/ekran kanıtı henüz yok. |

### 3.2 Doğrudan kaynak kodundan doğrulanan sınırlar

| Pazarlama iddiası | Kod bulgusu | Öneri |
|---|---|---|
| Brew day: gerçek OG, IBU ve ABV yeniden hesaplanır | [batches.rs](../../sunucu/crates/api/src/batches.rs) başındaki TODO: gerçek ABV şu an ölçümden gelir; OG/FG ile motor hesabı bekliyor. | Otomatik gerçek ABV hesabını mevcut özellik olarak sunma. OG ve IBU için ayrıca kanıt gerekli. |
| Fermentasyon: paketlemeye hazır, paketlendi ve kapandı ayrı aşamalar | Aynı dosyada STAGES: planned, brewing, fermenting, conditioning, packaging, completed, cancelled. | Pazarlama aşamalarını gerçek geçişlerle hizala; UI eşlemesini ayrıca kontrol et. |
| Stok: tüm maya stok birimleri otomatik düşülür | [stock.rs](../../sunucu/crates/api/src/stock.rs) TODO: kg/L desteklenir; paket adedi için gerçek parti kaydında miktar gerekli. | Adet bazında otomatik düşüm iddiasını doğrulamadan kullanma. |
| Paketleme: şişe, kutu, keg, cask ve lot izleme | [containers.rs](../../sunucu/crates/api/src/containers.rs) dört paket türü, dolum lotu, durum, QR, etiket ve geri çağırma API'sini doğrular. PDF etiket metni ASCII ile sınırlı; SSE istemci işi açık. | Özelliği bu kapsamla sınırla; tüm dillerde etiket ve canlı senkronizasyon vaat etme. |
| Abonelik yönetimi | [subscription.rs](../../sunucu/crates/api/src/subscription.rs) erişim kaynağı, plan, koltuk, dönem ve deneme sonunu döndürür; ücretli yazma engeli var, okuma ve export açık kalır. | API ile ödeme ekranını ayrı değerlendir; #427 henüz açık. |

16 özellik grubunun tamamı ilk eşlemede yer aldı. Her alt iddia için ayrıntılı kanıt ve EN/TR rota karşılaştırması halen açık.

**Öncelikli metin riski:** hesap motoru, reçete ve üretim sayfalarındaki kesin şimdiki zaman ifadeleri, bir API modülünün varlığıyla tüm alt özelliklerin çalıştığı izlenimini verebilir. Mevcut işlevi yalnız doğrulanan maddeye bağla; doğrulanmayan iddiaları yayın kapsamından çıkar. Bu rapor henüz tüm özellik maddelerini kapsamıyor.

### 3.3 İngilizce ve Türkçe özellik sayfaları

Kaynak: [EN veri](../../marketing-site/src/data/features.ts), [TR veri](../../marketing-site/src/data/features.tr.ts), [EN şablon](../../marketing-site/src/pages/features/[slug].astro), [TR şablon](../../marketing-site/src/pages/tr/features/[slug].astro). İki veri dosyasının 16 slug'ı ve bölüm sırası eşleşiyor. Bu yapısal denetim çeviri doğruluğu veya ürün işlevi kanıtı değildir.

**Yüksek öncelikli tutarsızlık:** EN detay şablonu özetin önüne `Planned scope:` koyuyor, açıklama metnini `Planned product direction:` ile başlatıyor ve `Roadmap stages` etiketini gösteriyor. TR şablon özeti ve meta açıklamayı doğrudan gösteriyor, eşdeğer aşama etiketini göstermiyor. Dil değiştiğinde aynı ürün iddiasının olgunluğu farklı anlaşılabilir. Her iki dilde de doğrulanmamış iddiaları yayın kapsamından çıkararak tutarlılık sağla; kaldırılmış olgunluk etiketlerini kullanıcı kararı olmadan geri getirme.

| Rota | EN bölüm/madde | TR bölüm/madde |
|---|---:|---:|
| recipe-design | 4/19 | 4/19 |
| brewing-calculations | 4/19 | 4/19 |
| water-mash-yeast | 3/15 | 3/15 |
| brew-day | 3/17 | 3/17 |
| fermentation-monitoring | 4/19 | 4/19 |
| equipment-calibration | 2/9 | 2/9 |
| planning-scheduling | 3/14 | 3/14 |
| inventory-costing | 3/14 | 3/14 |
| production-management | 3/17 | 3/17 |
| quality-laboratory | 3/11 | 3/11 |
| packaging-compliance | 3/15 | 3/15 |
| team-notifications | 3/14 | 3/14 |
| connected-devices | 3/16 | 3/16 |
| imports-exports-tools | 3/18 | 3/18 |
| platform-security-data | 6/31 | 6/31 |
| analytics-reports | 2/8 | 2/8 |
| Toplam | 52/256 | 52/256 |

Madde envanteri (TR, kaynak yukarıdaki veri dosyasıdır). Her madde için grup düzeyindeki ilk kanıt §3.1 ve doğrudan doğrulanan sınırlar §3.2 geçerlidir. Aşağıdaki liste tek başına hiçbir maddenin tamamlandığını göstermez. Tamamlanma durumu ve ISLER satırı tek tek doğrulanacak.

#### recipe-design

**Size uygun yoldan başlayın**

- Stil ve hedef odaklı malzeme önerileriyle boş bir reçeteden başlayın
- Stil tabanlı başlangıç şablonlarını kullanın ve ekipmanınıza ve parti hacminize uyarlayın
- Mevcut bir reçeteyi kopyalayın ya da bilinçli olarak yeni bir sürüm oluşturun
- BeerXML, BeerJSON, BeerSmith .bsmx, Brewfather ve Brewer's Friend için olası içe aktarma yolları; biçim doğrulaması ve malzeme, ekipman, stil ve birim eşleştirmesi sonrasında değerlendirilebilir.

**Yinelemeye uygun bir reçete düzenleyicisi**

- Fermente edilebilir malzemeleri, şerbetçiotunu, mayayı, çeşitli malzemeleri, mayşeyi, kaynatmayı ve fermantasyon ayrıntılarını düzenleyin
- Seçili değerleri dengede tutarak tahıl yüzdelerini ve hedef OG, IBU ya da rengi ayarlayın
- Tam tahıl reçetelerini ekstrakta, ekstrakt reçetelerini tekrar tam tahıla çevirin; malzemeleri ikame edin ve miktarları yuvarlayın
- Kayıpları yeniden hesaplayarak parti hacmine, demhane verimine ya da ekipmana göre ölçekleyin
- Reçete kontrollerini, uyarıları ve tek adımlık düzeltme önerilerini inceleyin

**Stiller, sürümler ve iş birliği**

- BJCP 2021, BJCP bal likörü/elma şarabı ve Brewers Association stil rehberleri; özel rehber desteğiyle
- Stil aralıklarını karşılaştırın ve bir reçeteyi rehberler arasında eşleştirin
- İki reçeteyi ya da sürümü yan yana karşılaştırın
- Yayımlanmış sürümleri değiştirilemez biçimde saklayın; işletme reçeteleri taslak, inceleme ve onay aşamalarından geçer
- Reçeteyi yanlışlıkla yapılan düzenlemelere karşı kilitleyin ve her partinin kullandığı sürümü koruyun

**Reçete kütüphanesi ve desteklenen üretim türleri**

- Reçeteleri ve partileri arama, filtre, klasör ve etiketlerle düzenleyin
- Favorileri sabitleyin ve reçete listelerinde hangi sütunların görüneceğini seçin
- Tam tahıl, ekstrakt, kısmi mayşe ve BIAB yaklaşımları dahil bira, bal likörü, elma şarabı ve seltzer reçetelerini destekleyin
- Reçete notlarını, renk seçimlerini, görselleri ve özel alanları saklayın
- Yayımlanmış sürümlerin üzerine sessizce yazmadan reçete geçmişini arşivleyin ve geri yükleyin

#### brewing-calculations

**Temel reçete hesaplamaları**

- Başlangıç ve son yoğunluk (OG / FG), görünür attenuasyon ve hacimce alkol (ABV)
- Varsayılan olarak Tinseth ile acılık (IBU); geç eklemeler, whirlpool ve hop-stand davranışı dahil
- Seçilebilir hesaplama yaklaşımıyla renk tahminleri
- Fermantasyon tahminlerinde mayşe sıcaklığı ve maya toleransı düzeltmeleri
- Parti, kazan, tortu ve hacim zinciri hesaplamaları

**Gelişmiş üretim modelleri**

- Sağlanan girdilerle şerbetçiotu yaşlanması ve alfa asit düzeltmeleri
- Starter büyümesi ve maya aşılama hesaplamaları
- Şişeler ve kegler için karbonasyon ve priming; keg üst boşluğu ve basınç sınırları dahil
- Düzeltilmiş YAN yönetimiyle bal likörü ve şarap fermantasyonu hesaplamaları
- Ekstrakt, kısmi kaynatma, seyreltme ve no-chill hesaplama araçları

**Açıklanabilir sonuçlar**

- Temel sonuçlar için “Nasıl hesaplandı” görünümü
- Kaydedilen her sonuçla birlikte saklanan hesap motoru sürümü
- Bölgesel gösterim ve birim dönüşümüyle tutarlı, SI tabanlı iç değerler
- Her motor sürümü için referans test vektörleri ve altın reçete doğrulaması

**Uzman yöntemler**

- No-chill IBU, şerbetçiotu yağı oranları, BU:GU ile alternatif IBU ve renk yöntemleri
- Elma şarabı ve şarap için meyve eklemeleri, pH sıcaklık düzeltmesi ve toplam asitlik ayarı
- Ekşi ve karma fermantasyon hedefleri ile fıçıda olgunlaştırma hesaplamaları
- Hortum çapı, uzunluğu ve servis basıncı için keg hat dengesi araçları
- Hesaplama ayarlarında yöntem seçimi ve gelişmiş varsayılanlar

#### water-mash-yeast

**Su profilleri**

- Kaynak su ve hedef su profillerini kaydedin
- Laboratuvar su raporunu girin ve iyon konsantrasyonlarını yönetin
- Tuz eklemelerini ve asit ayarlarını hesaplayın
- İyon dengesini, sertliği ve bölgesel sertlik birimlerini inceleyin
- Su profillerini karşılaştırın ve bir reçetede kullanılan raporu saklayın

**Mayşe planlaması**

- Dinlenmeleri, sıcaklıkları, süreleri ve infüzyon adımlarını içeren mayşe programı
- Seçilen modelle mayşe pH tahmini
- Mayşe hacmi ve sıcaklık adımları için dekoksiyon hesaplayıcısı
- Mayşe suyu, yıkama suyu ve hacim hesaplamaları
- BIAB'a özgü ekipman ve emilim yönetimi

**Maya hazırlığı**

- Maya seçimi ve yaş bilgileri
- İki aşamalı starterlar dahil starter planlaması
- İşletme modunda hektolitre başına kuru maya aşılama rehberliği
- Fermantasyon profilleri ve hedef koşullar
- İşletme çekirdeğinde hasat ve nesil takibi; gelişmiş ev modunda maya yönetimi etkinleştirilebilir

#### brew-day

**Rehberli demleme günü zaman çizelgesi**

- Tam ekran adım adım iPhone akışı ve ayrıntılı iPad/web adım listesi
- İsteğe bağlı zamanlayıcılar, başlat/bitir denetimleri ve düzenlenebilir gerçek zamanlar
- Sonraki adım rehberliği, hedef değerler ve ölçüm girişi
- Desteklenen cihazlarda Apple Live Activity, Dynamic Island ve zamana duyarlı bildirimler
- Birden fazla parti aynı anda yürütülebilir

**Plan ve gerçekleşen**

- Gerçek miktarları, zamanı, sıcaklığı ve hacmi kaydedin
- Adım ekleyin, atlayın, yeniden sıralayın ya da düzeltin; malzeme ikamelerini ve eklemelerini kaydedin
- Sapmaları işaretleyin ve çevrimiçiyken gerçek OG, IBU ve ABV değerlerini yeniden hesaplayın
- Ölçülen değerleri hedeflerle karşılaştırın ve bağlama uygun yorum alın
- İsteğe bağlı olarak gerçekleşen sonucu yeni bir reçete taslağına dönüştürün

**Kâğıtla uyumlu akış**

- Planlanan ve gerçekleşen alanları içeren, yazdırılabilir A4 veya US Letter üretim formları
- QR kodu basılı formu partisine bağlar
- Kâğıt üzerindeki ölçümleri aynı numaralı sırayla girin
- Üretim formlarını Apple belge tarama ile tarayın ve çıkarılan değerleri yan yana inceleyin
- İşletme çalışma alanları taranmış kayıtları zorunlu kılabilir ve saklama kuralları uygulayabilir
- Ev modunda üretim formu bölümlerini ve alanlarını özelleştirin
- Hatırlatıcılardan ve desteklenen sistem kısayollarından hızlı ölçüm girişi kullanın

#### fermentation-monitoring

**Eksiksiz bir parti zaman çizelgesi**

- Planlanan, üretilen, fermente olan, olgunlaşan, paketlemeye hazır, paketlenmiş ve kapatılmış aşamalarını izleyin
- Birden fazla partiyi ve fermantörü eş zamanlı yürütün
- Ölçümleri, notları, fotoğrafları ve olayları parti zaman çizelgesinde tutun
- Parti özetinde plan ve gerçek değerleri karşılaştırın
- Tadım notlarını kaydedin ve gelecekteki reçete çalışmalarına yön vermek için kullanın

**Ölçümler ve içgörüler**

- Cihazlardan ya da elle girişle alınan yoğunluk, sıcaklık ve pH okumaları
- Cihaz, elle ve kâğıt okumaları için ölçüm kaynağı etiketli grafikler
- Fermantasyon ilerleme vurguları
- Kuru şerbetçiotu ekleme, soğuk çöktürme ve paketleme zamanlaması önerileri
- Sensör bağlı olmadığında elle okuma hatırlatıcıları

**Kurallar ve uyarılar**

- Yoğunluğun birkaç gün sabit kalması ya da sıcaklığın hedef dışına çıkması gibi kurallar
- Üç öncelik düzeyi, sessiz saatler, gruplanmış uyarılar ve günlük özetler
- Onaylama, erteleme, yok sayma ve geçmiş içeren tek bir bildirim gelen kutusu
- İşletmede vardiya yönlendirmesi ve bir sonraki sorumlu kişiye yükseltme
- Elle okuma hatırlatıcısından doğrudan hızlı SG ve sıcaklık girişi
- Fermantasyon durumunu bir bakışta göstermek için ev ve tank odaklı widget'lar

**Parti karşılaştırması ve olgunlaştırma**

- Aynı reçeteden yapılan partileri OG, FG, verim ve tadım puanına göre karşılaştırın
- Ev modunda şişe ve keg olgunlaştırmasının hedef karbonasyona doğru ilerlemesini izleyin
- Ev modunda tadım geri bildirimini bir reçete taslağına dönüştürün

#### equipment-calibration

**Ekipman profilleri**

- İlk profil için rehberli ekipman kurulumu
- Mayşe tankı, kazan, fermantör ve paketleme kabı ayrıntıları
- Hacim kayıpları, tahıl emilimi, buharlaşma, ölü hacim ve soğutma büzülmesi
- İşletme çalışma alanları için birden fazla demhane ve kap profili
- Ekipmana özgü reçete ölçekleme ve verim varsayımları

**Kap hazırlığı**

- Tank kapasitesi, basınç sınırı ve mevcut doluluk
- Temizlik durumu ve son temizlik kaydı
- Üretim takvimindeki kap kullanılabilirliği
- Ekipman kalibrasyon kayıtları

#### planning-scheduling

**Takvim ve geriye dönük planlama**

- Demleme günü, fermantasyon adımları, kuru şerbetçiotu ekleme, soğutma ve paketleme tarihlerini planlayın
- Fermantasyon profillerinden hatırlatıcılar zamanlayın
- Isıtma ve soğutma süresini güç, hacim ve soğutma ekipmanından tahmin edin
- Zamanlama tahminlerini iyileştirmek için geçmiş ekipman performansını kullanın
- Kapasite ve doluluk çakışması uyarılarıyla işletme tank takvimi

**Paralel partiler**

- Partileri aşamaya ve yaklaşan göreve göre görün
- Tüm etkin partilerdeki görevleri içeren tek bir “Bugün” listesi
- Haftalık tank takvimi ve günlük tank turu çıktısı
- Bir partiyi atamadan önce tank çakışmalarını belirleyin
- İşletmede tank boşaltma önerileri ve kapasite optimizasyonu

**Esnek görünümler**

- Düzenlenebilir pano kartları ve sıralaması
- Role özgü varsayılan görünümler
- Yalnızca etkin fermantasyonlar gibi kayıtlı görünümler
- Operatör odaklı Bugün görünümü ve duvara monte ekran modu

#### inventory-costing

**Malzeme kütüphanesi**

- Malt, fermente edilebilir malzemeler, şerbetçiotu, maya, su ve çeşitli malzemeler
- Ayrıntılı malzeme özellikleri ve özel malzemeler
- Malzeme ikameleri ve stoğa duyarlı reçete planlaması
- Malzemeye özgü birimler ve bölgesel gösterim
- İşletme modunda özel malt analizi ve analiz sertifikası (COA)

**Stok ve mal kabul**

- Ev stok miktarları, asgari seviyeler ve alışveriş listesi
- İşletmede stok lotları, tedarikçi bilgileri, rezervasyonlar ve FIFO değerlemesi
- Mal kabul, barkod desteği ve stok sayımları
- Girişler, kullanım, transferler ve düzeltmeler için hareket defteri
- Üretim için lot kullanılabilirliği

**Reçete ve parti maliyetleri**

- Ev biracıları için isteğe bağlı reçete maliyeti
- FIFO lot fiyatlarıyla işletmede parti ve litre başına maliyet
- Ambalaj boyutuna göre şişe ve servis maliyeti
- FIFO lot fiyatlarına ve kaydedilen miktarlara dayanan maliyet tahminleri

#### production-management

**Üretim panosu ve operatör görünümü**

- Parti aşamasına ve tanka göre düzenlenmiş üretim panosu
- Operatörler için Bugün görev listesi
- Role dayalı pano ve kayıtlı görünümler
- Üretim sahasındaki iPad'ler için operatör ekranı
- Günlük tank turları ve yazdırılabilir haftalık tank takvimi

**Tank ve parti hareketleri**

- Doluluğu, durumu, basınç sınırını ve temizlik durumunu içeren tank ayrıntısı
- Hacim ve kayıp kayıtlarıyla kaplar arası transfer
- Bir partiyi birden fazla tanka bölme
- Hesaplanan harman özellikleriyle partileri birleştirme
- Kapasite çakışması kontrolleri ve tank çizelgeleme

**Üretim kayıtları**

- Operatör kimliği, adım tamamlama ve gerçek ölçümler
- Sapma kayıtları ve düzeltici kararlar
- Malzemelerden paketlemeye partinin izlenebilirliği; lottan partiye geri çağırma raporlarıyla
- Birden fazla operatör ve eş zamanlı etkin partiler
- Kapatılmış üretim kayıtlarındaki değişiklikler için denetim izi
- Geçerli temizlik kaydı olmayan bir tank atandığında uyarılar
- Açık uyarıların ve üretim notlarının vardiya devri

#### quality-laboratory

**Ürün spesifikasyonları**

- ABV, OG, FG, IBU, renk ve pH için reçeteye bağlı spesifikasyonlar
- Çözünmüş oksijen, bulanıklık, diasetil ve mikrobiyoloji parametreleri
- Aralık dışı uyarılar ve sapma kayıtları
- Partiye ve reçete sürümüne bağlı kalite incelemesi

**Laboratuvar ve duyusal kayıtlar**

- pH, çözünmüş oksijen, CO₂, ABV ve mikrobiyoloji için laboratuvar sonuçları
- Tadım notları, duyusal puan ve tanımlayıcı nitelikler
- Geçmiş sonuçların sessizce düzenlenmesi yerine düzeltme kayıtları
- Duyusal panel ve üçgen test akışları

**Serbest bırakma ve süreç kontrolü**

- Gerekçeleriyle kalite bekletme, ret ve serbest bırakma kararları
- İşletme akışlarında paketlemeden önce serbest bırakma onayı
- HACCP kontrol noktaları ve çift imza

#### packaging-compliance

**Paketleme çalışmaları ve kaplar**

- Yapılandırılabilir boyutlarla şişe, kutu, keg ve UK cask kap türleri
- Paketleme miktarları, litreler, kayıplar, paketleme verimi ve lot kodları
- Tek bir parti içinde birden fazla kap türü
- Şişe priming, keg basınçlı karbonasyon, BBT'den kutulama ve cask olgunlaştırma rehberliği
- Paketleme raporlaması için ambalaj malzemesi ve boş ağırlık

**Temizlik ve mahzen süreci**

- CIP kimyasalı, konsantrasyon, sıcaklık, süre ve doğrulama
- Tank temizlik durumu ve son temizlenme kaydı
- Geçerli temizlik kaydı olmayan bir tank atandığında uyarılar
- Spunding ve zorlamalı karbonasyon planlaması
- UK pazarı için cask olgunlaştırma ve durultucu (finings) hesaplamaları

**Etiketler ve mevzuat raporları**

- Ülkeye özgü vergi hesaplama yöntemleri ve raporlama şablonları
- ABD TTB, Birleşik Krallık HMRC, Avustralya ATO, Almanya Zoll ve Türkiye raporlama kapsamı
- Yargı bölgesine göre ABV veya Plato tabanlı vergi hesaplamaları
- Gereklilikler doğrulandığında ABV, lot kodu, alerjenler ve besin değerleri gibi etiket bilgileri
- Depozito iadesi ve ambalaj/EPR raporları

#### team-notifications

**Ev ve işletme çalışma alanları**

- İlk kurulumda bir mod seçin ve daha sonra çalışma alanı ayarlarından değiştirin
- Ev modu gelişmiş işletme alanlarını görünümden uzak tutar
- Ev modunda bir reçete veya parti üzerinde iş birliği için isteğe bağlı davetler
- Mod değiştirmek reçeteleri ve geçmişi korur; gizlenen işletme kayıtları silinmez

**Ekip, roller ve paylaşılan cihazlar**

- Süreli davetlerle kullanıcıları davet edin ve bir ya da daha fazla rol atayın
- Sahip, usta demci, operatör, kalite, depo ve salt okunur denetçi rolleri
- Özel roller, kopyalanan roller, izin grupları ve isteğe bağlı kap/alan kapsamı
- Kişi bazında izin özeti ve role özgü açılış ekranları
- Kişisel PIN, hızlı kullanıcı geçişi ve otomatik kilitle paylaşılan üretim iPad'i

**Bildirimler ve onaylar**

- Uygulama içi gelen kutusu, anlık bildirimler ve isteğe bağlı günlük e-posta özeti
- Üç öncelik düzeyi, sessiz saatler, gruplanmış uyarılar ve bildirim geçmişi
- Bildirimleri bir olay kaydıyla onaylayın, erteleyin, yok sayın ya da çözün
- İşletmede vardiya yönlendirmesi, yükseltme ve vardiya devri
- Reçete onayı, kalite serbest bırakma ve diğer role dayalı onay kuyrukları

#### connected-devices

**Evde bira yapım cihazları**

- Olası adaptör adayları arasında Tilt hidrometreler, iSpindel ve RAPT bulunabilir; her biri uygulama ve doğrulama gerektirir.
- TiltBridge veya Pitch gibi köprüler için webhook tabanlı veri yolu, uç nokta uygulaması ve güvenlik incelemesinden sonra değerlendirilebilir.
- Tasarlanan yaklaşım, yenilenebilir ve cihaza özgü anahtarlar içerebilir.
- Planlanan cihaz-parti eşleştirmesi ve kalibrasyon kayıtları.
- Tasarlanan akış, bağlantı ve okuma durumunu gösterebilir.

**Profesyonel sensörler**

- Planlanan ölçüm bağlamı yoğunluk, pH, çözünmüş oksijen ve basınç verilerini içerebilir.
- Planlanan tank ataması ve bağlantı durumu bağlamı.
- Profesyonel sensörler için API veya CSV ile veri alımı, bağlayıcının uygulanması ve doğrulanmasına bağlı olası bir yol olarak ele alınır.
- Modbus TCP, OPC UA ve MQTT gibi protokoller için salt okunur endüstriyel ağ geçidi bağlantısı, uygulama ve güvenlik incelemesine bağlı planlanan bir yöndür.
- Ekipmana kontrol komutu göndermek ürün kapsamının dışındadır.

**Veri kullanımı ve güvenilirlik**

- Cihaz veya ağ geçidinde okumaların tamponlanıp bağlantı kurulunca iletilmesi planlanan bir davranıştır.
- Planlanan kayıtlar, ölçümlerin yanında kaynak etiketlerini ve kalibrasyon bağlamını gösterebilir.
- Cihaz adaptörlerinin tek bir üreticiden bağımsız tasarlanması hedeflenir.
- Okumalar, fermantasyon eğilimleri ve yapılandırılmış eşik uyarıları için kullanılabilir.
- İşletme yol haritasında API ve üretim verisi webhook'ları değerlendirilebilir; ev biracılığı cihaz webhook'ları ayrı bir tasarım yoludur. Herkese açık bir uç nokta için uygulama, güvenlik incelemesi ve yayın onayı gerekir.
- Planlanan endüstriyel ağ geçidi yaklaşımı salt okunurdur; ekipman komutu ve kontrolü kapsam dışındadır.

#### imports-exports-tools

**Reçete taşınabilirliği**

- Biçim doğrulamasına bağlı olası BeerXML ve BeerJSON içe/dışa aktarma yolları.
- Biçim doğrulamasına bağlı olası BeerSmith .bsmx içe aktarma yolu.
- Brewfather ve Brewer's Friend reçete dışa aktarımları, doğrulanacak olası yollar olarak değerlendirilebilir.
- Planlanan içe aktarma eşleştirmesi malzeme, ekipman, stil ve birimlerin değerlendirilmesini kapsayabilir.
- Planlanan inceleme adımı, kaydetmeden önce kaynak ile GRISTORA hesaplama farklarını gösterebilir.

**Odaklı bira yapım araçları**

- Refraktometre düzeltmesi ve yoğunluk araçları
- Şişeler, kegler ve caskler için karbonasyon ve priming hesaplayıcıları
- Seyreltme, hacim ve birim dönüşümü
- Kısmi kaynatma, tamamlama suyu ve LME/DME/bal dönüşümleri için ekstrakt bira yapım araçları
- Dekoksiyon ve parti-gyle araçları; parti-gyle Ev modunda kullanılabilir
- Ev modunda no-chill IBU ayarı ve şişe/keg karbonasyon takibi
- Ev modunda seçili bira kontrol cihazlarına reçete aktarımı, uygulama ve doğrulamaya bağlı olası bir yol olarak değerlendirilebilir.

**Raporlar ve özel kayıtlar**

- Üretim formları, tank turları, fermantasyon ve paketleme formları
- Ev modunda özel üretim formu alanları ve yerleşimleri
- Planlanan dışa aktarma kapsamı reçete, parti, ekipman, su profili, malzeme, cihaz ve destek verilerini içerebilir.
- CSV ve JSON dışa aktarımları ile taranmış belge ve fotoğrafların indirilebilir arşivlerde sunulması, uygulama ve doğrulamaya bağlı olası bir kapsamdır.
- Ev modunda yarışma formu çıktısı ve özel reçete/parti alanları
- Ev modunda laboratuvar su raporlarının içe aktarılması, biçim doğrulamasına bağlı olası bir yol olarak değerlendirilebilir.

#### platform-security-data

**GRISTORA nerede çalışır**

- iPad: tam özellikli yerel SwiftUI uygulaması
- iPhone: demleme günü, ölçüm, fermantasyon, stok ve alışveriş akışları; reçeteler salt okunur görünür, reçete tasarımı ve planlama iPad ve web'de
- Web: Mac ve PC tarayıcıları için tam özellikli React ve TypeScript deneyimi
- Apple Watch desteklenmez
- Android ve ayrı Mac/Windows masaüstü uygulamaları kapsam dışındadır

**Tek motor, tek doğruluk kaynağı**

- Rust hesap motoru sunucu tarafında çalışır; hesaplama kodu istemcilere gönderilmez
- Web ve Apple uygulamaları arasında ortak API ve veri modeli
- Önce çevrimiçi çalışma; çevrimdışı zamanlayıcılar ve ölçüm girişi için sınırlı, indirilen bir demleme günü paketi
- Genel çevrimdışı senkronizasyon yoktur; kuyruğa alınan demleme günü ölçümleri bağlantı geri geldiğinde yüklenir
- Sürümlü hesaplama sonuçları ve değiştirilemez reçete/parti referansları

**Güvenlik ve veri sahipliği**

- Apple veya Google ile giriş; yedek olarak tek kullanımlık e-posta bağlantıları; şifre tabanlı kayıt yoktur
- İşletme yöneticileri için passkey'ler, oturum yönetimi ve uzaktan oturum kapatma
- Çalışma alanı ayrımı, sunucu tarafı izinler ve yalnızca eklemeli denetim kayıtları
- Şifreli depolama, korumalı dosya erişimi ve müşteri onayıyla süreli destek erişimi
- Planlanan veri taşınabilirliği kapsamı reçete, parti, ölçüm, stok, ekipman ve dosyaların dışa aktarılmasını ve hesap silme talebi akışını içerebilir.

**Destek ve hizmet**

- Aranabilir yardım ve destek talebi gelen kutusu
- Müşterinin görebildiği talep geçmişi ve yanıtlar
- Gerekçe ve denetim geçmişiyle isteğe bağlı, süreli destek erişimi
- Müşteri uygulamasından ayrılmış dahili destek çalışma alanı
- Sistem durumu ve destek tanılamaları

**Çalışma alanı kurulumu, tercihler ve abonelik**

- Rehberli ilk açılış: Ev ya da İşletme modunu, veri bölgesini ve ekipman profilini seçin
- Reçeteler, partiler, malzemeler ve stok genelinde genel arama
- Bölgesel ön ayarlar, kategoriye özgü birimler, yerel sayı/tarih biçimleri ve İngilizce/Türkçe dil desteği
- Tema, görünüm, bildirim, ekipman, hesaplama ve stil rehberi ayarları
- Abonelik ve hak yönetimi
- Hesap bağlama, oturum yönetimi, uzaktan oturum kapatma ve planlanan hesap dışa aktarma/silme akışları

**Apple ve web deneyimi**

- iOS Live Activities, Dynamic Island, widget'lar, zamana duyarlı bildirimler ve belge tarama
- Apple deneyiminin parçası olarak Siri ve Kısayollar entegrasyonu
- Apple Watch desteklenmez; Android ve ayrı masaüstü uygulamaları kapsam dışındadır
- Face ID veya PIN ile uygulama kilidi, paylaşılan cihaz PIN'i ve yönetici passkey'leri
- Şifreli yedeklemeler, veri taşınabilirliği ve müşterinin görebildiği destek erişim geçmişi

#### analytics-reports

**Parti ve reçeteden öğrenme**

- Plan ve gerçekleşen parti özeti ile tadım notları
- Aynı reçeteden yapılan partileri karşılaştırın
- Ev modunda yoğunluk, verim ve tadım puanı eğilimleri

**Bira üretimi raporlaması**

- Demhane, dönüşüm ve süzme verimi
- Demleme gününden paketlemeye hacim kaybı
- FIFO malzeme maliyeti ve litre başına maliyet
- Tank kullanımı, parti çıktısı ve üretim panosu metrikleri
- Malzeme lotundan paketleme çalışmasına lot izlenebilirliği ve geri çağırma raporu
