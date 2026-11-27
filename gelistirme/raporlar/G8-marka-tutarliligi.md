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

### 3.4 Madde bazında kanıt — recipe-design

Sıra §3.3 envanterindeki dört bölüm ve 19 madde ile aynıdır. Durum: **Var (API)** = belirtilen kapsamın sunucu kodu mevcut; **Kısmi** = iddianın yalnız bir kısmı doğrulandı; **Sonra** = açık geliştirme kapsamı; **Belirsiz** = yeterli kanıt bulunmadı, mevcut özellik olarak kullanılmamalı. Kaynaklar: [recipes.rs](../../sunucu/crates/api/src/recipes.rs) (R), [recipe_org.rs](../../sunucu/crates/api/src/recipe_org.rs) (O), [API envanteri](../../sunucu/crates/api/CLAUDE.md) (A), [ISLER](../ISLER.md) (I; yol haritası kanıtıdır, canlı görev durumu değildir).

| Madde | Durum | Kanıt / eksik kapsam |
|---|---|---|
| 1. Boş reçete ve malzeme önerisi | Kısmi | R: recipe_create, NewRecipe, VersionInput; öneri algoritması kanıtlanmadı. I: T2.21. |
| 2. Stil şablonları | Belirsiz | I: T2.19 stil kılavuzu ve örnek veri; gerçek başlangıç şablonu ve uyarlama akışı kanıtlanmadı. |
| 3. Kopyalama/yeni sürüm | Kısmi | R: NewDraft.from_version_id ve recipe_version_create yeni sürüm sağlar; ayrı reçeteye kopyalama akışı doğrulanmadı. |
| 4. İçe aktarma biçimleri | Kısmi | A: T2.26, BeerXML, BeerJSON ve Brewfather JSON; .bsmx örnek bekliyor, Brewer's Friend uyumluluğu doğrulanmadı. |
| 5. Malzeme ve proses düzenleme | Var (API) | R: VersionInput, FermentableLine, HopLine, YeastLine, MiscLine, mash_snapshot, fermentation_snapshot, boil_time_min. İstemci ekranı ayrıca doğrulanmalı. |
| 6. Yüzdeler ve hedefleri dengede tutma | Kısmi | R: VersionInput hedef OG, IBU, SRM alanlarını içerir; otomatik dengeleme kanıtlanmadı. |
| 7. All-grain/ekstrakt dönüşümü, ikame, yuvarlama | Kısmi | R: VersionInput.kind; scale işlevindeki TODO küçük miktar yuvarlamasını Aşama 3'e bırakır. Otomatik dönüşüm ve ikame doğrulanmadı. |
| 8. Hacim/verim/ekipmana göre ölçekleme | Kısmi | R: ScaleBody.factor, batch_volume_l, calculate ve recipe_version_scale; verim ve ekipman bazlı ölçekleme ayrı doğrulanmalı. |
| 9. Uyarı ve tek adım düzeltme | Kısmi | R: constraint_error ve recipe_version_calculate; tek adım düzeltme önerisi kanıtlanmadı. |
| 10. BJCP/BA ve özel kılavuz | Kısmi | I: T2.19 stil kılavuzları; gerçek kılavuz verisi örnek seed'den ayrı karardır. Tam veri setleri doğrulanmadı. |
| 11. Stil aralığı karşılaştırma/eşleme | Kısmi | I: T2.19 eşleme uçları; aralık karşılaştırma ekranı doğrulanmadı. |
| 12. Reçete/sürüm karşılaştırma | Var (API) | R: recipe_version_compare, /recipe-versions/{id}/compare/{other_id}. Yan yana ekran ayrıca doğrulanmalı. |
| 13. Değişmez sürüm ve İşletme onayı | Kısmi / Sonra | R: recipe_version_publish, version_immutable; TODO: submit/review/reject Aşama 6. I: T2.21. |
| 14. Kilit ve parti sürümü | Var (API) | R: RecipePatch.is_locked, O: edit-session; batches.rs: immutable recipe_version_id. |
| 15. Arama/filtre/klasör/etiket | Var (API) | R: RecipeFilter, O: folders/tags, A: T2.27 global search. Parti organizasyonunun tümü ayrıca doğrulanmalı. |
| 16. Favoriler ve sütun seçimi | Kısmi | O: recipefavorite_put/delete; sütun kişiselleştirme doğrulanmadı. |
| 17. Üretim türleri ve yöntemler | Kısmi | R: VersionInput.kind extract, partial_mash, all_grain, cider, mead, wine, seltzer; mash_snapshot içinde `biab`. Tüm istemci akışları doğrulanmadı. |
| 18. Not, renk, görsel, özel alan | Kısmi | R: NewRecipe.notes ve RecipePatch.notes; A: T2.18 attachments. Renk ve özel alanlar doğrulanmadı. |
| 19. Arşiv ve geri yükleme | Kısmi | A: T2.37 trash, 30 gün; R: version_immutable. Ayrı arşiv davranışı doğrulanmadı. |

**Öneri:** reçete sayfasında sürüm, temel düzenleme ve organizasyon için kanıtlı kapsamı koru; şablon, otomatik dengeleme, tam biçim uyumluluğu ve İşletme onay sürecini mevcut özellik olarak sunma. Bu bölüm 19/256 katalog maddesinin ayrıntılı sınıflandırmasıdır.


### 3.5 Madde bazında kanıt — brew-day ve inventory-costing

Sıra §3.3 ile aynıdır. Kaynaklar: [API envanteri](../../sunucu/crates/api/CLAUDE.md) (A), [batches.rs](../../sunucu/crates/api/src/batches.rs) (B), [brew_day.rs](../../sunucu/crates/api/src/brew_day.rs) (D), [stock.rs](../../sunucu/crates/api/src/stock.rs) (S), [files.rs](../../sunucu/crates/api/src/files.rs) (F). Bu incelemede A yeniden okundu; B ve S için §3.2'deki doğrudan kod bulguları esas alındı. Belirsiz durum yokluk kanıtı değildir.

#### brew-day (17 madde)

| Madde | Durum | Kanıt / eksik kapsam |
|---|---|---|
| 1. iPhone tam ekran ve iPad/web adım listesi | Kısmi | A T2.22: adım API'si; cihaz ve ekran akışları doğrulanmadı. |
| 2. Zamanlayıcı, başlat/bitir, gerçek zamanlar | Kısmi | A T2.22: adım ve gerçekleşen değerler; zamanlayıcı ve tüm zaman düzenleme davranışı doğrulanmadı. |
| 3. Sonraki adım, hedef, ölçüm | Kısmi | A T2.22: adım, ölçüm ve özet; otomatik rehberlik doğrulanmadı. |
| 4. Live Activity, Dynamic Island, zamana duyarlı bildirimler | Belirsiz | A T2.24: bildirim modülü; Apple sistem entegrasyonları için ayrı iOS kanıtı gerekli. |
| 5. Eş zamanlı partiler | Kısmi | A T2.22: parti uçları; paralel kullanıcı akışı doğrulanmadı. |
| 6. Gerçek miktar, zaman, sıcaklık ve hacim | Kısmi | A T2.22: gerçek miktar ve ölçüm; tüm alanların bir arada kaydı doğrulanmadı. |
| 7. Adım düzenleme ve malzeme ikamesi | Kısmi | A T2.22: adım ve sapma kayıtları; ekleme/atlama/sıralama ve ikame kapsamı doğrulanmadı. |
| 8. Sapma ve gerçek OG/IBU/ABV yeniden hesabı | Kısmi / Sonra | B: §3.2, gerçek ABV ölçümden gelir, motor hesabı açık TODO; A T2.22 sapma. |
| 9. Hedef karşılaştırması ve yorum | Kısmi | A T2.22: özet ve sapma; bağlamsal yorum kanıtlanmadı. |
| 10. Gerçekleşenden yeni reçete taslağı | Belirsiz | A T2.21 taslak ve T2.22 gerçekleşen değer ayrı işlevler; birleşik akış doğrulanmadı. |
| 11. A4/US Letter üretim formu | Belirsiz | A T2.22: imzalı demleme paketi; basılabilir form ve sayfa ölçüsü kanıtlanmadı. |
| 12. Basılı form QR bağlantısı | Belirsiz | A T2.22 paket; §3.2 kap QR kodu farklı işlevdir. |
| 13. Numaralı sırada kâğıt ölçümü | Kısmi | A T2.22: istemci UUID ile tekrar güvenli ölçüm; kâğıt sırası akışı doğrulanmadı. |
| 14. Apple tarama ve çıkarılan değer karşılaştırması | Kısmi | A T2.18/F: taranmış belge uçları; OCR ve Apple ekranı doğrulanmadı. |
| 15. Zorunlu tarama ve saklama politikası | Belirsiz | A T2.18: belge saklama uçları; zorunluluk ve saklama kuralı kanıtlanmadı. |
| 16. Ev modunda form özelleştirme | Belirsiz | A T2.22 paket; özel alan ve yerleşim kanıtı yok. |
| 17. Hatırlatıcıdan hızlı ölçüm | Kısmi | A T2.22 ölçüm ve T2.24 bildirim; sistem kısayolu ve birleşik akış doğrulanmadı. |

#### inventory-costing (14 madde)

| Madde | Durum | Kanıt / eksik kapsam |
|---|---|---|
| 1. Altı malzeme grubu | Kısmi | A T2.21 reçete ve T2.27 kütüphane malzemesi araması; tüm alt türler doğrulanmadı. |
| 2. Ayrıntılı özellikler ve özel malzeme | Kısmi | A T2.27 malzeme araması ve T2.37 geri yükleme; ayrıntılı alanlar doğrulanmadı. |
| 3. İkame ve stoğa duyarlı planlama | Kısmi | A T2.23 reçeteden stok düşülerek alışveriş listesi; otomatik ikame kanıtlanmadı. |
| 4. Malzemeye özgü birim ve bölgesel gösterim | Kısmi | S: kg/L ve paket adedi sınırı (§3.2); bölgesel UI doğrulanmadı. |
| 5. Özel malt analizi ve COA | Belirsiz | A T2.18 belge uçları; COA ve malt analizi iş akışı kanıtlanmadı. |
| 6. Ev stoku, minimum seviye, alışveriş | Kısmi | A T2.23 stok toplamı ve alışveriş listesi; minimum seviye doğrulanmadı. |
| 7. Lot, tedarikçi, rezervasyon ve FIFO değerleme | Kısmi | A T2.23 lot ve en eski lottan düşüm; rezervasyon ve FIFO değerleme doğrulanmadı. |
| 8. Mal kabul, barkod ve sayım | Kısmi | A T2.23 receive/count; barkod ekranı doğrulanmadı. |
| 9. Hareket defteri | Kısmi | A T2.23 receive/consume/adjust/count; transfer işlemi doğrulanmadı. |
| 10. Üretim lot kullanılabilirliği | Kısmi | A T2.23 stok toplamı ve parti düşümü; önceden lot kullanılabilirlik ekranı doğrulanmadı. |
| 11. Ev reçete maliyeti | Belirsiz | S maliyet alanları (§3.1); hesap ve ekran kanıtı gerekli. |
| 12. İşletme FIFO parti/litre maliyeti | Belirsiz | A T2.23 FIFO tüketim, FIFO maliyetleme sonucu değildir. |
| 13. Şişe ve servis maliyeti | Belirsiz | A T2.23 stok ve §3.2 kap türleri; paket başına maliyet hesabı doğrulanmadı. |
| 14. FIFO fiyatı ve miktarından tahmin | Belirsiz | A T2.23 en eski lottan düşüm; fiyat bazlı tahmin ayrıca kanıtlanmalı. |

**Öneri:** üretim adımları, gerçek ölçümler, stok hareketleri ve alışveriş listesi için kanıtlı API kapsamını ayır. Otomatik hesap, Apple entegrasyonu, basılı/optik form ve FIFO maliyetleme iddialarını bağımsız kanıt olmadan mevcut özellik diye sunma. Toplam 50/256 katalog maddesi madde bazında sınıflandırıldı; kalan 206 açık.


### 3.6 Madde bazında kanıt — fermentation-monitoring, team-notifications, connected-devices, imports-exports-tools

Sıra §3.3 ile aynıdır. Kaynaklar: [API envanteri](../../sunucu/crates/api/CLAUDE.md) (A), [notifications.rs](../../sunucu/crates/api/src/notifications.rs) (N), [devices.rs](../../sunucu/crates/api/src/devices.rs) (D), [import_export.rs](../../sunucu/crates/api/src/import_export.rs) (I). N, D ve I ilk tip tanımları ve modül açıklamaları doğrudan okundu. API kanıtı cihaz testi veya yayımlanmış hizmet kanıtı değildir.

#### fermentation-monitoring (19 madde)

| Madde | Durum | Kanıt / eksik kapsam |
|---|---|---|
| 1. Yedi aşamalı zaman çizelgesi | Kısmi | A T2.22, §3.2 STAGES: gerçek yedi değer pazarlama listesinden farklı; cancelled var, ayrı paketlemeye hazır aşaması yok. |
| 2. Eş zamanlı parti ve fermantör | Kısmi | A T2.22 parti API; fermantör eş zamanlı atama kanıtlanmadı. D tank eşlemesi TODO. |
| 3. Ölçüm, not, fotoğraf, olay | Kısmi | A T2.22 ölçüm, T2.18 ek dosya ve T2.15 SSE; birleşik zaman çizelgesi doğrulanmadı. |
| 4. Plan ve gerçek özet | Var (API) | A T2.22 summary, adım ve sapma; ekran ayrıca doğrulanmalı. |
| 5. Tadım notu ve reçeteye geri besleme | Belirsiz | A T2.21 reçete ve T2.22 parti modülleri; tadım ve geri besleme kanıtı yok. |
| 6. Elle ve cihazla SG/sıcaklık/pH | Kısmi | D modül başlığı cihaz okumalarını parti ölçümüne dönüştürür; A T2.22 elle ölçüm; üç ölçümün tüm adaptörlerde desteği doğrulanmadı. |
| 7. Kaynak etiketli grafik | Belirsiz | D cihaz eşleme ve A T2.22 ölçüm; grafik, kâğıt kaynağı ve etiket doğrulanmadı. |
| 8. Fermantasyon ilerleme vurguları | Belirsiz | A T2.22 aşama/özet; otomatik ilerleme vurgusu kanıtı yok. |
| 9. Kuru şerbetçiotu/soğuk çöktürme/paketleme önerisi | Belirsiz | A T2.22 takvim ve geriye planlama; öneri algoritması doğrulanmadı. |
| 10. Sensörsüz elle ölçüm hatırlatıcıları | Kısmi | N Notification.kind reminder, D eşleme; sensörsüz durum tetikleyicisi doğrulanmadı. |
| 11. Sabit SG ve hedef dışı sıcaklık kuralı | Kısmi | N AlertRule.condition ölçüm, op, value, cooldown; birkaç günlük sabitlik hesabı kanıtlanmadı. |
| 12. Üç öncelik, sessiz saat, gruplama, günlük özet | Kısmi | N Notification.priority 1..3, Preferences quiet_start/end, daily_summary; gruplama doğrulanmadı. |
| 13. Onay, ertele, yok say, geçmiş | Var (API) | N ActionBody approve/postpone/ignore; InboxFilter ve durumlar; UI doğrulanmadı. |
| 14. Vardiya yönlendirme/yükseltme | Belirsiz | N caller inbox; vardiya devri ve sonraki sorumlu algoritması kanıtlanmadı. |
| 15. Hatırlatıcıdan hızlı SG/sıcaklık girişi | Kısmi | N reminder ve A T2.22 ölçüm; doğrudan giriş ekranı doğrulanmadı. |
| 16. Ev ve tank widget | Belirsiz | D last_data_at; Apple widget kanıtı yok. |
| 17. Aynı reçeteden parti karşılaştırması | Belirsiz | A T2.21 sürüm karşılaştırması ve T2.22 parti özeti; parti karşılaştırma ve tadım puanı kanıtı yok. |
| 18. Şişe/keg olgunlaşma ve karbonasyon | Belirsiz | §3.2 containers.rs kap türü; karbonasyon ilerlemesi kanıtlanmadı. |
| 19. Tadım sonucundan taslak | Belirsiz | A T2.21 taslak; tadım verisinden taslak oluşturma kanıtlanmadı. |

#### team-notifications (14 madde)

| Madde | Durum | Kanıt / eksik kapsam |
|---|---|---|
| 1. Kurulumda mod ve sonradan değiştirme | Belirsiz | A T2.11 workspace create; mod değiştirme ve kurulum ekranı doğrulanmadı. |
| 2. Ev modunda İşletme alanlarının gizlenmesi | Belirsiz | A T2.13 izin; mod bazlı görünüm kanıtı değil. |
| 3. Ev reçete/parti iş birliği | Kısmi | A T2.13 Home sharing/davet; her iki kayıt türünün ekranı doğrulanmadı. |
| 4. Mod değişiminde kayıtları koruma | Belirsiz | A T2.11/T2.13; mod geçişinde veri koruma testi gerekli. |
| 5. Süreli davet ve çoklu rol | Kısmi | A T2.13 invitations, roles, assignments; davet süresi ve çoklu rol UI doğrulanmadı. |
| 6. Altı hazır rol | Kısmi | A T2.13 role endpoints; altı rolün seed ve izinleri doğrulanmadı. |
| 7. Özel/kopyalanan rol, izin grubu, kap kapsamı | Kısmi | A T2.13 roles ve assignments; kopyalama, gruplama ve kap/alan kapsamı doğrulanmadı. |
| 8. Kişi bazında izin özeti ve açılış ekranı | Kısmi | A T2.13 izin API; birleşik izin özeti ve role özgü ekran doğrulanmadı. |
| 9. Paylaşılan iPad PIN/geçiş/kilit | Kısmi | A T2.12 auth/shared_device.rs, auth/pin.rs, members/switch/lock; otomatik kilit ve iPad akışı doğrulanmadı. |
| 10. Gelen kutusu, push ve e-posta özeti | Var (API) | N Notification, Preferences push_enabled/email_enabled/daily_summary; teslim testi ve UI açık. |
| 11. Öncelik, sessiz saat, gruplama, geçmiş | Kısmi | N priority, Preferences, InboxFilter; gruplama kanıtı yok. |
| 12. Onayla, ertele, yok say, çöz ve olay kaydı | Kısmi | N ActionBody üç işlem; resolved durum alanı var, kullanıcı çözme işlemi ve olay kaydı ayrıca doğrulanmalı. |
| 13. Vardiya yönlendirme ve devir | Belirsiz | N caller inbox; vardiya akışı kanıtlanmadı. |
| 14. Reçete ve kalite onay kuyruğu | Kısmi / Sonra | A T2.21 publish; §3.4 reçete inceleme TODO Aşama 6; kalite onay kuyruğu doğrulanmadı. |

#### connected-devices (16 madde)

| Madde | Durum | Kanıt / eksik kapsam |
|---|---|---|
| 1. Tilt/iSpindel/RAPT adaptör adayları | Kısmi | D KINDS ispindel, rapt, tilt, other; gerçek donanım uyumluluk testi açık. Metin koşullu. |
| 2. TiltBridge/Pitch webhook | Kısmi | D ingest/{key} ve Tilt bridge modül açıklaması; Pitch ve gerçek bağlantı doğrulanmadı. |
| 3. Yenilenebilir cihaz anahtarı | Var (API) | D DeviceWithKey, rotate; yalnız SHA-256 özeti saklanır. |
| 4. Cihaz-parti eşleme ve kalibrasyon | Var (API) | D Device.batch_id/calibration, NewDevice; istemci doğrulanmadı. |
| 5. Bağlantı ve okuma durumu | Kısmi | D last_data_at; bağlantı göstergesi UI doğrulanmadı. |
| 6. Yoğunluk/pH/DO/basınç | Belirsiz | D cihaz okumalarını parti ölçümüne dönüştürür; tüm dört veri türü kanıtlanmadı. |
| 7. Tank ataması ve bağlantı durumu | Sonra / Kısmi | D başlık TODO vessel_id; last_data_at mevcut. |
| 8. Profesyonel API/CSV alımı | Belirsiz | D ingest veri yolu; profesyonel CSV bağlayıcı kanıtı yok. |
| 9. Modbus TCP/OPC UA/MQTT | Belirsiz | D HTTP ingest; üç protokol ve endüstriyel ağ geçidi kanıtlanmadı. |
| 10. Kontrol komutları kapsam dışı | Kapsam dışı | §3.3 ürün sınırı, D kayıt/eşleme/okuma yolu; kontrol işlevi iddiası değildir. |
| 11. Cihaz tamponlama ve yeniden gönderme | Belirsiz | D son okuma zamanı ve A T2.22 tekrar güvenli istemci UUID; cihaz tamponu kanıtlanmadı. |
| 12. Kaynak etiketi ve kalibrasyon bağlamı | Kısmi | D calibration ve batch_id; ölçüm ekranındaki kaynak etiketi doğrulanmadı. |
| 13. Üreticiden bağımsız adaptör | Kısmi | D KINDS içinde other ve ortak ingest adresi; her adaptörün tasarımı doğrulanmadı. |
| 14. Eğilim ve eşik uyarıları | Kısmi | D okumaları parti ölçümü, N AlertRule condition; trend ekranı doğrulanmadı. |
| 15. İşletme API ve webhook yol haritası | Belirsiz | D özel cihaz ingest; genel üretim webhook'u, yayın ve güvenlik onayı kanıtlanmadı. |
| 16. Salt okunur ağ geçidi ve kontrol dışı | Kapsam dışı / Belirsiz | §3.3 kontrol ürün kapsamı dışında; D salt veri toplama yolu; endüstriyel ağ geçidi henüz kanıtlanmadı. |

#### imports-exports-tools (18 madde)

| Madde | Durum | Kanıt / eksik kapsam |
|---|---|---|
| 1. BeerXML ve BeerJSON içe/dışa aktarım | Kısmi | I modül başlığı iki içe aktarım biçimi; dışa aktarımın bu biçimleri doğrulanmadı. |
| 2. BeerSmith .bsmx | Sonra | I TODO T2.26 örnek dosya gerekli. |
| 3. Brewfather/Brewer's Friend | Kısmi | I Brewfather JSON backup içe aktarımı; Brewer's Friend ve diğer dışa aktarım biçimleri doğrulanmadı. |
| 4. Malzeme/ekipman/stil/birim eşleme | Kısmi | I apply ekipman oluşturur/eşler, malzemeyi adla eşler, mapping_decisions saklar; stil/birim kapsamı ayrıca doğrulanmalı. |
| 5. Kaydetmeden önce hesap farkı inceleme | Kısmi | I preview kaynak değerlerini verir, apply taslak yaratır ve kaynak/motor değerlerini yan yana döndürür; farkların kayıttan önce sunulması doğrulanmadı. |
| 6. Refraktometre ve yoğunluk araçları | Belirsiz | A T2.16 motor bağlantısı; özel araç uçları ve UI doğrulanmadı. |
| 7. Şişe/keg/cask karbonasyon ve priming | Belirsiz | A T2.16 ve §3.2 kap türleri; hesaplayıcı kanıtı açık. |
| 8. Seyreltme, hacim, birim dönüşümü | Belirsiz | A T2.16 motor bağlantısı; özel yöntem ve ekran doğrulanmadı. |
| 9. Ekstrakt, kısmi kaynatma ve dönüşüm | Belirsiz | A T2.21 reçete ve T2.16 motor; özel araç kanıtı açık. |
| 10. Dekoksiyon/parti-gyle ve Ev modu | Belirsiz | A T2.16 motor; özel araç ve hak kapsamı doğrulanmadı. |
| 11. Ev no-chill ve karbonasyon takibi | Belirsiz | A T2.16 ve §3.2 kap türleri; hesap ve takip ekranı açık. |
| 12. Kontrol cihazına reçete aktarımı | Belirsiz | D ingest veri toplama içindir; kontrol cihazı reçete aktarımı doğrulanmadı. |
| 13. Üretim/tank/fermantasyon/paketleme formu | Belirsiz | A T2.22 imzalı paket ve takvim; dört basılı form kanıtlanmadı. |
| 14. Ev özel form alanı ve yerleşim | Belirsiz | A T2.22; özel form yapılandırması doğrulanmadı. |
| 15. Reçete/parti/ekipman/su/malzeme/cihaz/destek export | Kısmi | I recipe, batch, workspace export job; tüm veri türlerinin worker çıktısı doğrulanmadı. |
| 16. CSV/JSON ve dosyaları arşivde indirme | Kısmi | I export job ve presigned link; A T2.18 dosya uçları; arşiv içeriği ve biçimleri doğrulanmadı. |
| 17. Yarışma formu ve özel alanlar | Belirsiz | A T2.21 reçete ve T2.22 parti; form/özel alanlar kanıtlanmadı. |
| 18. Su raporu içe aktarımı | Belirsiz | I reçete biçimleri; laboratuvar su raporu ayrıştırıcısı doğrulanmadı. |

**Öncelikli öneriler:** cihaz sayfası için gerçek donanım testlerini aday listesinden ayır; tank eşlemesini mevcut özellik sayma. İçe aktarma farklarının apply sonrasında döndüğü kaynak kodla uyumlu ifadeler kullan. Vardiya, Apple widget ve otomatik önerileri kanıt olmadan mevcut kapsamda sunma. Toplam 117/256 katalog maddesi madde bazında sınıflandırıldı; kalan 139 açık.


### 3.7 Madde bazında kanıt — hesaplar, su, ekipman ve planlama

Sıra §3.3 ile aynıdır. Kaynak: [ISLER](../ISLER.md) (Y), [sunucu çalışma alanı](../../sunucu/CLAUDE.md), [engine.rs](../../sunucu/crates/api/src/engine.rs) (E), [API envanteri](../../sunucu/crates/api/CLAUDE.md) (A). Y, uygulama kodunu bulmaya yarayan tarihsel teknik kayıt; canlı görev durumu veya üretim yayını kanıtı değildir. Y'nin bitmiş motor satırı yalnız motor kapsamını destekler; istemci akışı için ayrı kod/test gereklidir.

#### brewing-calculations (19)

| Madde | Durum | Kanıt / eksik kapsam |
|---|---|---|
| 1. OG/FG, attenuasyon, ABV | Kısmi | Y T1.11 tarif özeti ve T1.16 ortak tarif modeli; E motor bağlantısı. Bütün çıktı alanları ve ekran doğrulanmadı. |
| 2. Tinseth, geç ekleme, whirlpool, hop-stand | Kısmi | Y T1.14 §02 tarif IBU ve acılık; varsayılan yöntem ve tüm ekleme davranışları doğrulanmadı. |
| 3. Seçilebilir renk modeli | Belirsiz | Y T1.11/T2.20 hesap ayarları; renk yöntemi seçimi ve UI kanıtı açık. |
| 4. Mayşe sıcaklığı ve maya toleransı | Belirsiz | Y T1.4 maya ve T1.11 mash; düzeltmenin birleşik tahminde kullanımı doğrulanmadı. |
| 5. Parti/kazan/tortu/hacim zinciri | Kısmi | Y T1.11 mash hacimleri, T1.14 §05 hacim/su; tüm kayıp zinciri UI doğrulanmadı. |
| 6. Şerbetçiotu yaşlanması ve alfa asit | Belirsiz | Y T1.14 §02 acılık ayarı; yaşlanma yöntemi ayrı kanıt gerektirir. |
| 7. Starter ve aşılama | Kısmi | Y T1.4 maya ve T1.11 maya servisi; starter seçeneklerinin istemci desteği açık. |
| 8. Karbonasyon, priming, üst boşluk/basınç | Kısmi | Y T1.4/T1.11 karbonasyon ve T1.8 spunding; üst boşluk ve sınır UI doğrulanmadı. |
| 9. Mead/şarap YAN | Kısmi | Y T1.4 mead/maya; YAN düzeltmesinin şarap kapsamı doğrulanmadı. |
| 10. Ekstrakt/kısmi kaynatma/seyreltme/no-chill | Belirsiz | Y T1.14 §02, §05 ve T2.21 reçete türleri; özel araçlar tek tek doğrulanmadı. |
| 11. Nasıl hesaplandı görünümü | Kısmi | Y T2.16 ara değerler; E Calculated.output; görünümün kendisi doğrulanmadı. |
| 12. Motor sürüm damgası | Var (API) | E Calculated.engine_version, CalcResult; Y T2.16. |
| 13. SI değerleri, birim ve bölgesel gösterim | Kısmi | Y T1.16, T2.17 ve T2.38 ortak birim kataloğu; tüm UI gösterimleri doğrulanmadı. |
| 14. Referans vektörleri ve altın reçete | Kısmi | Y T1.14/T1.17 parite ve uç testleri; her sürüm için altın reçete CI sonucu ayrıca gerekli. |
| 15. No-chill, yağ oranları, BU:GU, alternatif yöntemler | Kısmi | Y T1.14 §02 hop oranı ve IBU; diğer yöntemlerin tek tek kanıtı açık. |
| 16. Meyve, pH sıcaklık, toplam asitlik | Belirsiz | Y T1.17 su pH eksikleri ve T2.21 cider; üç araç doğrulanmadı. |
| 17. Ekşi/karma fermentasyon ve fıçı olgunlaştırma | Belirsiz | Y T1.11 maya ve T2.20 fermentasyon profili; özel modeller doğrulanmadı. |
| 18. Keg hat dengesi | Belirsiz | Y T1.4 karbonasyon; hat dengesi ayrı hesap. |
| 19. Yöntem seçimi ve gelişmiş varsayılanlar | Kısmi | Y T2.20 workspace/user calculation options; tüm seçenekler ve ekran açık. |

#### water-mash-yeast (15)

| Madde | Durum | Kanıt / eksik kapsam |
|---|---|---|
| 1. Kaynak ve hedef su profili | Kısmi | Y T2.20 su profili; kaynak/hedef ayrımı ve ekran doğrulanmadı. |
| 2. Su analizi ve iyonlar | Kısmi | Y T2.20 water analysis report, T1.14 §03 SO₄/Cl; tüm iyon alanları doğrulanmadı. |
| 3. Tuz ve asit hesabı | Kısmi | Y T1.11 su servisi, T1.17 çok asitli pH; tuz önerisi ve UI ayrıca gerekli. |
| 4. İyon dengesi, sertlik ve birimler | Kısmi | Y T1.14 §03, T2.17 birim kataloğu; sertlik yöntemleri doğrulanmadı. |
| 5. Profil karşılaştırma ve reçete raporu | Belirsiz | Y T2.20 profil/rapor ve T2.21 reçete; karşılaştırma ve snapshot akışı açık. |
| 6. Dinlenme/süre/sıcaklık/infüzyon | Kısmi | Y T2.20 mash profile, T1.11 mash; alan ve UI kapsamı açık. |
| 7. Seçilen modelle mash pH | Kısmi | Y T1.17 çok asitli pH, T1.14 §03; model seçimi doğrulanmadı. |
| 8. Dekoksiyon | Belirsiz | Y T1.11 mash hacimleri; dekoksiyon hesabı ayrıca doğrulanmalı. |
| 9. Mayşe/yıkama suyu/hacim | Kısmi | Y T1.14 §05 toplam su ve hacim, T1.11 mash; istemci akışı açık. |
| 10. BIAB ekipman/emilim | Kısmi | Y T1.14 §05 BIAB/emilim, T2.20 equipment; özel UI doğrulanmadı. |
| 11. Maya seçimi/yaşı | Kısmi | Y T2.19 malzeme, T1.4 maya; yaş girişinin etkisi doğrulanmadı. |
| 12. İki aşamalı starter | Belirsiz | Y T1.4/T1.11 maya; iki aşamalı davranış kanıtı açık. |
| 13. İşletme kuru maya hl rehberliği | Belirsiz | Y T1.4 maya; işletme doz rehberi doğrulanmadı. |
| 14. Fermantasyon profili/hedef | Kısmi | Y T2.20 fermentation profile, T2.22 timers/targets; ekran doğrulanmadı. |
| 15. Hasat/nesil ve Ev modülü | Sonra / Belirsiz | Y T2.7 YeastGeneration tablosu, uçlar Aşama 6; Ev modülü ve akış kanıtı yok. |

#### equipment-calibration (9)

| Madde | Durum | Kanıt / eksik kapsam |
|---|---|---|
| 1. Rehberli ilk ekipman | Kısmi | Y T2.20 Equipment wizard; istemci ekranı doğrulanmadı. |
| 2. Mayşe tankı/kazan/fermantör/paket kabı | Kısmi | Y T2.20 equipment/vessels, K2 ve T2.59 package types; tüm profil alanları açık. |
| 3. Kayıp/emilim/buharlaşma/ölü hacim/büzülme | Kısmi | Y T1.14 §05 kayıplar ve hacim, T2.20 equipment; her alanın UI desteği açık. |
| 4. Çoklu demhane/kap profilleri | Kısmi | Y T2.20 equipment/vessels; İşletme sınırları ve çoklu akış açık. |
| 5. Ekipmana göre ölçek ve verim | Kısmi | Y T2.40 verim alanları, T2.21 scaling; tüm ölçekleme davranışı doğrulanmadı. |
| 6. Tank kapasite/basınç/doluluk | Kısmi / Sonra | Y T2.20 vessels ve T2.7 VesselOccupancy (Aşama 6); basınç/doluluk akışı açık. |
| 7. Tank temizliği ve son kayıt | Sonra | Y T2.7 CleaningRecord, Aşama 6 uçları; K2 kap statüsü tank temizliği değildir. |
| 8. Takvimde kap kullanılabilirliği | Belirsiz | Y T2.22 calendar, T2.7 occupancy; birleşik görünüm doğrulanmadı. |
| 9. Kalibrasyon kayıtları | Kısmi | Y T2.20 calibration proposal ve ayar geçmişi; özel kayıt ve ekran doğrulanmadı. |

#### planning-scheduling (14)

| Madde | Durum | Kanıt / eksik kapsam |
|---|---|---|
| 1. Demleme/fermantasyon/dry-hop/soğutma/paket tarihleri | Kısmi | Y T2.22 calendar/backward planning; tüm olay türleri doğrulanmadı. |
| 2. Profilden hatırlatıcı | Kısmi | Y T2.20 fermentation profile, T2.24 Reminder; otomatik bağ doğrulanmadı. |
| 3. Güç/hacim/ekipmandan ısıtma/soğutma süresi | Belirsiz | Y T1.14 §05 soğuma ve T2.20 equipment; süre tahmini kanıtı açık. |
| 4. Geçmiş ekipman performansından tahmin | Belirsiz | Y T2.20 ayar geçmişi ve T2.22 gerçek değer; öğrenme algoritması doğrulanmadı. |
| 5. Tank kapasite/çakışma uyarısı | Sonra / Belirsiz | Y T2.7 VesselOccupancy ve T2.22 calendar; Aşama 6 uçları, uyarı kanıtı yok. |
| 6. Aşama ve yaklaşan göreve göre parti | Kısmi | Y T2.22 stage/steps/calendar; filtreli ekran doğrulanmadı. |
| 7. Tüm etkin parti Bugün listesi | Belirsiz | Y T2.22 steps/calendar; birleştirilmiş görev UI açık. |
| 8. Haftalık tank takvimi/günlük tur çıktısı | Belirsiz | Y T2.22 calendar, T2.7 occupancy; çıktı kanıtı yok. |
| 9. Atama öncesi tank çakışması | Sonra / Belirsiz | Y T2.7 occupancy Aşama 6; kontrol kanıtlanmadı. |
| 10. Tank boşaltma önerisi/optimizasyon | Belirsiz | Y T2.7 occupancy; optimizasyon algoritması kanıtı yok. |
| 11. Düzenlenebilir pano kartları | Belirsiz | Y T2.20 user settings; pano düzeni doğrulanmadı. |
| 12. Role özgü varsayılan görünüm | Belirsiz | Y T2.13 roles; varsayılan UI ayrı kanıt. |
| 13. Kayıtlı görünümler | Belirsiz | Y T2.20 settings ve T2.22 batches; kayıtlı filtre doğrulanmadı. |
| 14. Operatör Bugün/duvar ekranı | Belirsiz | Y T2.12 paylaşılan cihaz, T2.22 steps; ekran kanıtı yok. |

### 3.8 Madde bazında kanıt — production, quality, packaging ve analytics

Kaynak: Y, A ve [containers.rs](../../sunucu/crates/api/src/containers.rs) (C, modül başlığı ve veri tipleri okundu). T2.7 veri modeli tamamlanmış olsa da satır açıkça İşletme uçlarını Aşama 6'ya bırakır; tablo varlığı kullanılabilir ürün özelliği değildir. K2 kap uçları bunun belirli bir alt kümesidir.

#### production-management (17)

| Madde | Durum | Kanıt / eksik kapsam |
|---|---|---|
| 1. Aşama/tank üretim panosu | Kısmi | Y T2.22 stage; T2.7 occupancy Aşama 6; pano doğrulanmadı. |
| 2. Operatör Bugün listesi | Belirsiz | Y T2.22 steps; birleşik ekran açık. |
| 3. Role dayalı pano/kayıtlı görünüm | Belirsiz | Y T2.13 roles, T2.20 settings; UI kanıtı yok. |
| 4. Üretim iPad operatör ekranı | Belirsiz | Y T2.12 shared device; operatör ekranı doğrulanmadı. |
| 5. Tank turu/haftalık çıktı | Belirsiz | Y T2.22 calendar; çıktı ve tur iş akışı açık. |
| 6. Tank doluluk/durum/basınç/temizlik | Sonra / Kısmi | Y T2.7 occupancy/cleaning, T2.20 vessels; tamamı için Aşama 6 iş akışı gerekli. |
| 7. Kap transferi/hacim/kayıp | Sonra | Y T2.7 Transfer, uçlar Aşama 6. |
| 8. Partiyi tanklara bölme | Sonra / Belirsiz | Y T2.7 Transfer/Occupancy; bölme uçları doğrulanmadı. |
| 9. Partileri harman hesabıyla birleştirme | Sonra / Belirsiz | Y T2.7 Transfer; harman uygulama akışı doğrulanmadı. |
| 10. Kapasite çakışma ve tank çizelgesi | Sonra / Belirsiz | Y T2.7 Occupancy ve T2.22 calendar; birleşik kontrol açık. |
| 11. Operatör, adım, ölçüm | Kısmi | Y T2.22 steps/measurement; operatör alanı ve ekran ayrıca doğrulanmalı. |
| 12. Sapma ve düzeltici karar | Kısmi | Y T2.22 Deviation; karar akışı doğrulanmadı. |
| 13. Malzemeden pakete izlenebilirlik/recall | Kısmi | C last_lot_id ve last_fill_line_id, Y K2 recall, T2.23 lot; bütün zincir doğrulanmadı. |
| 14. Çoklu operatör ve aktif parti | Kısmi | Y T2.13 roles, T2.22 batches; eş zamanlı operasyon ekranı açık. |
| 15. Kapalı kayıt değişikliklerinde audit | Kısmi | Y T2.5 AuditLog; C ContainerEvent append-only ve audit; kapalı üretim kaydının tamamı doğrulanmadı. |
| 16. Temizlik kaydı olmayan tank uyarısı | Sonra | Y T2.7 CleaningRecord Aşama 6; uyarı akışı açık. |
| 17. Vardiya devri | Belirsiz | Y T2.24 notifications; vardiya devri kanıtı yok. |

#### quality-laboratory (11)

| Madde | Durum | Kanıt / eksik kapsam |
|---|---|---|
| 1. ABV/OG/FG/IBU/renk/pH spesifikasyonu | Sonra | Y T2.7 QualitySpec, Aşama 6 uçları; tüm alanlar ve UI açık. |
| 2. DO/bulanıklık/diasetil/mikrobiyoloji | Sonra / Belirsiz | Y T2.7 QualitySpec/LabResult; parametre seti doğrulanmadı. |
| 3. Aralık dışı uyarı/sapma | Kısmi / Sonra | Y T2.24 AlertRule ve T2.22 Deviation; kalite modülüne bağ Aşama 6. |
| 4. Parti/reçete sürümü kalite inceleme | Sonra | Y T2.7 QualitySpec/Release; inceleme uçları açık. |
| 5. pH/DO/CO₂/ABV/mikrobiyoloji sonuçları | Sonra | Y T2.7 LabResult; tam ölçüm seti ve UI açık. |
| 6. Tadım/duyusal puan | Belirsiz | Y T2.7 LabResult; tadım kayıt işlevi doğrulanmadı. |
| 7. Geçmiş sonucu düzeltme kaydı | Belirsiz | Y T2.5 AuditLog ve T2.7 LabResult; değişmez düzeltme akışı açık. |
| 8. Duyusal panel/üçgen testi | Belirsiz | Y T2.7 kalite veri modeli; özel panel/üçgen akışı doğrulanmadı. |
| 9. Bekletme/ret/serbest bırakma ve gerekçe | Sonra | Y T2.7 Release, Aşama 6; karar uçları açık. |
| 10. Paketlemeden önce release | Sonra | Y T2.7 PackagingRun/Release; zorunlu iş akışı doğrulanmadı. |
| 11. HACCP/çift imza | Belirsiz | Y T2.7 kalite/temizlik modeli; HACCP/çift imza kanıtı yok. |

#### packaging-compliance (15)

| Madde | Durum | Kanıt / eksik kapsam |
|---|---|---|
| 1. Şişe/kutu/keg/cask, özel boyut | Var (API) | C Container.package_kind/volume_l; Y T2.59 paket tipi GET/POST. |
| 2. Miktar/litre/kayıp/verim/lot | Kısmi / Sonra | C volume_l, last_lot_id, last_fill_line_id; Y T2.7 PackagingRun Aşama 6; tam verim/kayıp akışı açık. |
| 3. Partide birden fazla kap türü | Kısmi | C kap türleri ve fill line; tek partide birleşik paketleme ekranı doğrulanmadı. |
| 4. Priming/keg/BBT/cask rehberi | Kısmi | Y T1.4 karbonasyon, T1.8 spunding; BBT/cask rehber akışı açık. |
| 5. Ambalaj malzemesi/dara ağırlığı | Belirsiz | C package type ve volume; rapor alanları doğrulanmadı. |
| 6. CIP kimyasalı/konsantrasyon/sıcaklık/süre/doğrulama | Sonra | Y T2.7 CleaningRecord Aşama 6; alan ve akış açık. |
| 7. Tank temizlik durumu ve son kayıt | Sonra | Y T2.7 CleaningRecord; C kap status clean/dirty ayrı kapsam. |
| 8. Temiz olmayan tank atama uyarısı | Sonra | Y T2.7 CleaningRecord/Occupancy; uyarı doğrulanmadı. |
| 9. Spunding/zorlamalı karbonasyon planı | Kısmi | Y T1.8/T1.17 spunding, T1.4 karbonasyon; planlama ekranı açık. |
| 10. UK cask/finings hesabı | Belirsiz | C cask kap türü; Y T1.4 karbonasyon; finings ve UK özel yöntem doğrulanmadı. |
| 11. Ülkeye özgü vergi/rapor şablonu | Kısmi | Y T1.8/T2.8 vergi yöntemi, T2.7 veri tabloları; ülke şablonları doğrulanmadı. |
| 12. TTB/HMRC/ATO/Zoll/TR rapor kapsamı | Belirsiz | Y T2.8 genel yöntem; beş yargı alanı için güncel veri ve rapor onayı gerekli. |
| 13. ABV/Plato vergi | Kısmi | Y T2.8 yöntem zinciri ve T1.17 LPA; her ülke için Plato/ABV girdisi ve oran doğrulanmadı. |
| 14. ABV/lot/alerjen/besin etiketi | Kısmi | C QR/seri/ölçü etiketi, ASCII sınırı; Y T1.8 etiket enerjisi; zorunlu mevzuat alanları doğrulanmadı. |
| 15. Depozito/EPR | Belirsiz | Y T2.7 PackagingRun/vergi tabloları; EPR/depozito raporu kanıtı yok. |

#### analytics-reports (8)

| Madde | Durum | Kanıt / eksik kapsam |
|---|---|---|
| 1. Plan/gerçek ve tadım notu | Kısmi | Y T2.22 summary; tadım notu açık. |
| 2. Aynı reçeteden partileri karşılaştırma | Belirsiz | Y T2.21 recipe/version compare, T2.22 summary; batch compare ayrı. |
| 3. Yoğunluk/verim/tadım eğilimi | Belirsiz | Y T2.22 measurement ve T2.40 efficiency; trend ekranı açık. |
| 4. Demhane/dönüşüm/süzme verimi | Kısmi | Y T2.38/T2.40 verim ayrımı ve T1.17 süzme hesabı; rapor ekranı açık. |
| 5. Demlemeden paketlemeye hacim kaybı | Belirsiz | Y T2.22 actual, T2.7 PackagingRun Aşama 6; zincir raporu açık. |
| 6. FIFO maliyet/litre | Belirsiz | Y T2.23 lot/movement; FIFO tüketim maliyet raporu değildir. |
| 7. Tank kullanım/çıktı/pano KPI | Sonra / Belirsiz | Y T2.7 Occupancy/PackagingRun, Aşama 6; KPI raporu açık. |
| 8. Malzeme lotu–paketleme/recall | Kısmi | C last_lot_id/last_fill_line_id; Y K2 recall, T2.23 stock; bütün malzeme zinciri açık. |

**Öneri:** İşletme kalite, tank, CIP ve paketleme raporlaması için veri modelini işleyen API ve ekran kanıtı ile ayır. Vergi hesap motorunun bulunması, hiçbir ülke için mevzuat uyumluluğunun onayı değildir. Pazarlamada mevzuat raporu veya HACCP uyumluluğu vaat etme.

Bu iki bölüm 108 yeni maddeyi sınıflandırır. Toplam 225/256 katalog maddesi incelendi. Kalan 31 madde platform-security-data; ayrıca özellik dışı sayfalar, doğrudan motor/istemci kanıtı ve raporun sonuç bölümü açık.
