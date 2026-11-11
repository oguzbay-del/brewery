# Demleme Uygulaması Ürün Planı

Oct 31, 2026 · @Oguz

## Özet ve kararlar

Bira üretimine odaklı, ev biracıları ve küçük işletmeler için iki modlu bir tarif ve üretim uygulaması yapılacak. Apple tarafı native SwiftUI, masaüstü web; hesap motoru tek ve ortak. Görsel ekran taslakları: [Demleme Arayüz Konsepti](https://claude.ai/artifact/BCEHj3wb6xj7eShvjTxKn6).

| Konu | Karar |
| --- | --- |
| Segmentler | Ev modu ve İşletme modu; tek uygulama, aynı veri modeli |
| Kapsam | Tarif, demleme, fermantasyon, üretim, envanter, maliyet, mevzuat raporları |
| Kapsam dışı | ERP katmanı: satış, sipariş, dağıtım, fıçı takibi, CRM, muhasebe. Topluluk ve herkese açık tarif paylaşımı (kullanıcılar mevcut platformları kullanır; bizde içe ve dışa aktarma var) |
| Platformlar | iPad (tam), iPhone (demleme günü odaklı), web (Mac ve PC, tam). Apple Watch ileride opsiyonel |
| Desteklenmeyen | Android, ayrı Mac ve Windows uygulaması; Apple Watch şimdilik yok (tasarımı konseptte duruyor, ileride opsiyonel eklenebilir) |
| Mimari | Apple: native SwiftUI. Web: React + TypeScript. Hesap motoru: Rust, yalnız sunucuda, ayrı servis; istemciler tek API ile kullanır. Altı katman, Kubernetes üzerinde, bulut sağlayıcıdan bağımsız |
| Tasarım | Apple tasarım ilkeleri (HIG); her ekran benzer işi yapan bir Apple uygulamasından fikir alıyor |
| Stil kılavuzları | BJCP 2021, BJCP mead/cider ve Brewers Association gömülü; her aşamada seçilebilir |
| Demleme günü | Plan ve gerçekleşen ayrı; sayaç isteğe bağlı; yazdırma ve belge tarama |
| Eşzamanlı üretim | Birden fazla parti aynı anda; işletmede üretim panosu ve tank takvimi |
| Bağlantı | İnternetli çalışma: hesap ve oluşturma işleri sunucuda. Yalnız demleme günü indirilen paketle yerelde çalışır. Çevrimdışı senkron yok |

## Hedef kullanıcılar ve iki mod

Tek uygulama iki modla çalışır: ev biracısı için sade Ev modu, küçük bira fabrikası için İşletme modu. Mod çalışma alanı bazında seçilir, her an değiştirilir; geçişte tarifler ve geçmiş korunur. Gizlenen ayarlar silinmez, tek alan için açılabilir. Ana rakip Brewfather; işletme tarafında Ekos gibi ERP'lerle yarışılmaz.

| Konu | Ev modu | İşletme modu |
| --- | --- | --- |
| Tarif oluşturma | Sihirbaz önde, akıllı varsayılanlar | Editör önde, şablonlar, ölçekleme; tüm formül ve sabitler seçilebilir |
| Birimler | L, kg, °C | Ek olarak hL ve bbl; tipik parti 1 hL – 30 bbl |
| Ekipman | Tek profil, hızlı sihirbaz | Birden fazla brewhouse ve tank, kalibrasyon |
| Stil kılavuzu (varsayılan) | BJCP | BA |
| Demleme günü | Adım adım rehber | Üretim kaydı: operatör, sapma, QC |
| Partiler | Aşamaya göre liste, birden fazla fermentor | Üretim panosu, tank takvimi, transfer, bölme, birleştirme |
| Envanter | Basit stok, alışveriş listesi | Lot takibi, tedarikçi, izlenebilirlik |
| Maliyet | Tarif maliyeti (isteğe bağlı) | Parti ve litre başı maliyet |
| Kullanıcılar | Tek kişi | Çoklu kullanıcı, roller, denetim kaydı |
| Mevzuat | – | ABD: TTB; İngiltere: HMRC; Avustralya: ATO; Almanya: Zoll; Türkiye: ÖTV/TAPDK (uzmanla netleştirilecek) |
| Belge tarama | İsteğe bağlı | "Her partide tarama iste" ayarı, saklama süresi |

### Mod ayrımı: hangi ekran ve özellik hangi modda

Mod çalışma alanı bazında seçilir. **Açık:** modda tam çalışır. **Sade:** aynı ekran, işletmeye özgü alanlar gizli. **Kapalı:** ekran ve menüde görünmez. **Kapalı (açılabilir):** Ev modunda gizli, Ayarlar'dan tek özellik olarak açılabilir (ileri ev biracısı için). Tek kullanıcıya dayanmayan özellikler açılabilir; çok kullanıcı veya mevzuat gerektirenler (roller, onay, denetim kaydı, ortak cihaz, vardiya, mevzuat raporu) Ev modunda açılamaz.

| Ekran veya özellik | Ev | İşletme | Not |
| --- | --- | --- | --- |
| **Tarif** |  |  |  |
| Tarif sihirbazı | Açık (açılışta önde) | Açık | İşletmede editör önde |
| Tarif editörü | Sade | Açık | Ev'de formül ve sabit seçimleri gizli, varsayılan kullanılır |
| Tarif onayı (taslak → incelemede → onaylı) | Kapalı | Açık, zorunlu | Ev'de tarif doğrudan partiye gider |
| Kaynatma programı, mash ve maya, fermantasyon profili, su ve pH, su raporu, mead/cider/seltzer | Açık | Açık |  |
| Tarif ve parti listesi, tarif ve sürüm karşılaştırma, içe aktarma, "Nasıl hesaplandı" | Açık | Açık | İşletmede listede onay durumu filtresi |
| Stil kılavuzu karşılaştırması | Açık | Açık | Varsayılan kılavuz: Ev'de BJCP, İşletmede BA |
| Malzeme detayı, kendi malzemen | Açık | Açık | Lota özel değerler ve malt analizi (COA) yalnız İşletmede |
| Tarif ölçekleme | Açık | Açık | hL ve bbl birimleri İşletmede |
| **Demleme ve fermantasyon** |  |  |  |
| Parti başlatma | Sade | Açık | İşletmede tank ataması ve yalnız onaylı sürüm |
| Demleme günü, Live Activity | Açık (adım adım rehber) | Açık (üretim kaydı) | İşletmede operatör, sapma ve QC kaydı |
| Kâğıt, aktarma ve düzeltme | Açık | Açık | Tarama Ev'de isteğe bağlı; İşletmede zorunlu yapılabilir ve saklama süresince silinemez |
| Fermantasyon, otomatik uyarı kuralları, parti özeti, araçlar | Açık | Açık | İşletmede fermantasyon tank detayına bağlı |
| Ambalaj | Sade | Açık | Ev'de ayrı ekran yok: fermantasyondaki "Şişele / fıçıla" eylemi (priming, adet); İşletmede CIP ve ambalaj kaydı |
| Cihaz eşleştirme | Açık (Tilt, iSpindel, RAPT) | Açık | İşletmede tanka eşleştirme; profesyonel sensörler ve ağ geçidi İşletme 2. sürüm |
| Takvim ve geriye planlama | Açık | Açık | İşletmede tank takvimi ve kapasite çakışması |
| Kalibrasyon | Açık (ekipman profili) | Açık | İşletmede brewhouse verimi, dönüşüm ve süzme verimi ayrı |
| Maya yönetimi (hasat, jenerasyon) | Kapalı (açılabilir) | Açık |  |
| **Üretim** |  |  |  |
| Üretim panosu, Bugün, Pano ve Bugün düzenleme | Kapalı | Açık | Ev'de açılış ekranı tarif listesi, aktif parti kartı üstte |
| Operatör görünümü | Kapalı | Açık | Cihaz başına açılır |
| Onay kutusu, kalite, sapma kaydı | Kapalı | Açık |  |
| Tank detayı, tank hareketi, günlük tank turu | Kapalı | Açık | Ev'de birden fazla fermentor parti listesinde izlenir |
| Lot izlenebilirlik, etiket bilgisi, raporlar ve mevzuat raporu | Kapalı | Açık | Mevzuat raporu ve etiket İşletme 2. sürüm |
| Maliyet | Sade (tarif maliyeti, isteğe bağlı) | Açık (parti ve litre, FIFO) | Parti maliyeti Ev'de açılabilir |
| **Sistem** |  |  |  |
| İlk açılış, hesap, abonelik, arama, boş ve hata durumları | Açık | Açık | Mod ilk açılışta seçilir, sonra Ayarlar › Çalışma alanı › Mod'dan değişir; Ev'de açılabilen özellikler de orada. Abonelik planları moda göre |
| Ekipman sihirbazı | Açık (tek profil) | Açık | İşletmede birden fazla brewhouse ve tank |
| Stok ve alışveriş listesi | Sade (miktar, minimum seviye) | Açık (lot, rezerve, FIFO, COA) | Lot takibi Ev'de açılabilir |
| Mal kabul ve sayım | Sade (stok ekleme) | Açık (lot, barkod, sayım farkı) |  |
| Ayarlar | Sade | Açık | Ev'de Kullanıcılar, Roller ve yetkiler, vergi ülkesi gizli |
| Bildirim yönlendirme | Sade (öncelik, sessiz saat, özet) | Açık (+ vardiya, yükseltme, role göre) |  |
| Kullanıcı davet akışı | Sade | Açık | Ev'de tarif veya partiye düzenleyici/izleyici daveti (paylaş menüsünden); İşletmede rol atamalı ve süreli davet |
| Roller ve yetkiler, ortak cihazda kullanıcı değiştirme, denetim kaydı | Kapalı | Açık | Ev'de açılamaz |
| Birimler | L, kg, °C (bölgeye göre) | + hL, bbl |  |
| Kaplar | Sade (şişe ve keg) | Açık (şişe, kutu, keg, cask) | Vergi ve rapor uygunluğu yalnız İşletmede görünür |
| Bildirim kutusu (çan) | Açık | Açık | İşletmede vardiya, yükseltme ve role göre dağıtım eklenir |
| Giriş ve hesap bağlama, yardım ve destek | Açık | Açık | İşletmede destek erişim iznini yönetici verir; yöneticiye passkey zorunlu |
| Davet kabulü | Sade | Açık | Ev'de tarif ve parti daveti; İşletmede rol ve ortak cihaz PIN'i |

Moddan moda geçişte veri silinmez; Ev'e dönülse de İşletme kayıtları (onaylar, denetim kaydı, lotlar) saklanır, yalnız görünmez.

## Platformlar ve mimari

Apple cihazları native SwiftUI, masaüstü web; hepsi sunucudaki tek hesap motorunu ve tek API'sini paylaşır. Bir formül düzeltmesi tüm platformlara aynı anda gider.

| Platform | Kapsam | Teknoloji |
| --- | --- | --- |
| iPad | Tam özellik | SwiftUI |
| iPhone | Demleme günü, fermantasyon, stok ve alışveriş tam; tarif görüntüleme ve basit düzenleme | SwiftUI |
| Apple Watch | Şimdilik yok; ileride opsiyonel (sayaç, hop alarmı, adım onayı; tasarımı konseptte hazır) | SwiftUI watchOS (ileride) |
| Web (Mac ve PC) | iPad ile aynı, tam özellik; güncel Chrome, Edge, Safari ve Firefox (Windows, macOS, Linux, ChromeOS) | React + TypeScript |
| Mac uygulaması, Android | Yok | – |

- **Hesap motoru:** Rust ile bir kez yazılır ve yalnız sunucuda çalışır. Tarif hesabı, ölçekleme, su ve pH, maliyet, vergi gibi tüm hesaplar ve kayıt oluşturma işleri internetle yapılır. Sık yapılan küçük sorgular (liste, arama, malzeme bilgisi) önbellekten hemen gösterilir ve arkada sunucudan tazelenir; tarif düzenlerken değişiklikler kısa aralıklarla toplu gönderilir.
- **Veri:** tek kaynak sunucudaki veritabanıdır; cihazda çevrimdışı senkron yok, yalnız salt okunur önbellek tutulur. Tek istisna demleme günü: parti başlatılınca demleme paketi (tarif, adımlar, sayaçlar, hedef değerler) cihaza indirilir; sayaç, adım onayı ve ölçüm girişi internetsiz çalışır. Girilen ölçümler gönderim kuyruğunda bekler, bağlantı gelince gönderilir. Bağlantı yokken hesap gerektiren işler (ör. gerçek değerlerle OG/ABV yeniden hesabı) bağlantıyı bekler.
- **Apple'a özgü:** Live Activity ve Dynamic Island, zamana duyarlı bildirimler, VisionKit belge tarama, Bluetooth (Tilt).
- **iPad uygulamasının Apple Silicon Mac'lerde çalışması** App Store Connect'te kapatılır; Mac'te tek deneyim web olur.
- **Maliyet:** arayüz iki kez yazılır (SwiftUI + React); web ve Apple arasında özellik eşitliği için ortak tasarım sistemi ve ekran spesifikasyonları gerekir.

&#91;embedded content: mimari · Apple ve web istemcileri, sunucuda hesap ve veri, demleme günü yerelde\]

Apple uygulamaları ve web aynı API'yi kullanır; hesap motoru, veri, yetki ve bildirim sunucuda. Cihazda yalnız demleme paketi ve salt okunur önbellek var.

### Katmanlar

Altı katman var. Bağımlılık yalnız aşağı doğrudur ve istemci yalnız kenardan girer. Uygulama katmanı hem hesap hem veri katmanını çağırır; hesap katmanı veriye erişmez, yalnız kendisine verilen girdiyle çalışır.

| Katman | Ne yapar | Ne yapmaz | Teknoloji (öneri) |
| --- | --- | --- | --- |
| 1 İstemci | Ekran, giriş, sayaç, bildirim gösterimi; demleme gününde paketi yürütür | Hesap, iş kuralı, yetki kararı | SwiftUI (iPad, iPhone); React + TypeScript (web) |
| 2 Kenar | Tek genel adres; TLS, WAF, hız sınırı, kimlik belirteci kontrolü; web arayüzünü CDN'den dağıtır | İş kuralı | Bulutun yük dengeleyicisi ve CDN'i veya Kubernetes giriş denetleyicisi (Envoy tabanlı) |
| 3 Uygulama | Tarif, parti, stok, rol, onay, bildirim kuralları; veriyi okur ve yazar; hesap motorunu çağırır | Formül | Rust: çekirdek API (modüler monolit), arka plan işleri, cihaz verisi alımı; Apple ve Google girişi API içinde |
| 4 Hesap | Tüm formüller: OG, IBU, renk, su ve pH, maya, hacim, maliyet, vergi, tahminler | Veri okuma veya yazma, kullanıcı ve yetki bilgisi | Rust hesap motoru, ayrı servis, yalnız iç ağda gRPC |
| 5 Veri | Kalıcı kayıt, önbellek, dosya, iş kuyruğu | Hesap | PostgreSQL, Redis, S3 uyumlu nesne deposu, kuyruk |
| 6 Platform | Çalıştırma, ağ, sır, gözlem, dağıtım | – | Kubernetes, Terraform/OpenTofu, GitOps (Argo CD), OpenTelemetry, sır yöneticisi |

### İki kez yazmama kuralı

Arayüz iki kez yazılır (SwiftUI ve React); başka hiçbir şey iki kez yazılmaz.

- **Hesap:** yalnız Rust motorunda. İstemciye hesap kodu (WebAssembly dahil) gönderilmez.
- **İş kuralları ve doğrulama:** yalnız çekirdek API'de. Alan tanımları ve sınırları (min, max, birim, zorunluluk) API'den veri olarak gelir; istemci bunları yalnız gösterir. Sunucu her kaydı yeniden doğrular.
- **API sözleşmesi:** tek OpenAPI dosyası. Swift ve TypeScript istemci kodu bu dosyadan üretilir; elle yazılmaz.
- **Birim dönüşümü ve yerelleştirme:** dönüşüm katsayıları ve metinler tek veri dosyasında; Swift ve TypeScript tarafına üretilir. İstemci yalnız biçimlendirir, hesap yapmaz.
- **Tasarım:** renk, yazı ve boşluk değerleri tokens.json'dan Swift asset kataloğuna ve CSS'e üretilir (tasarım sistemi).
- **Demleme paketi:** sayaç süreleri, adımlar, hedef değerler ve alarm zamanları sunucuda hesaplanıp pakete yazılır. Cihaz yalnız yürütür.

### API ve istemci iletişimi

- **Canlı güncelleme:** açık ekran sunucudan olay akışı alır (Server-Sent Events). Fermantasyon grafiği, bildirim kutusu sayısı, eşzamanlı düzenleme kilidi ve tank durumu yenilemeden güncellenir. Sunucuda olaylar podlar arasında PostgreSQL LISTEN/NOTIFY ile dağıtılır; yük artınca kuyruğa (NATS) geçilir. Uygulama arka plandayken push bildirimi kullanılır.
- **Sürüm uyumu:** App Store'da eski uygulama sürümleri aylarca kullanılır. API sürümlüdür (/v1); geriye uyumsuz değişiklik yeni sürümle gelir, eski sürüm en az 6 ay açık kalır \[doğrulanacak\]. Var olan alanın anlamı değiştirilmez, yalnız yeni alan eklenir. Sunucu desteklenen en düşük uygulama sürümünü bildirir; altındaki uygulama "güncelle" ekranı gösterir. OpenAPI fark kontrolü her derlemede çalışır.
- **Tekrar güvenliği:** kayıt oluşturan her istek istemcinin ürettiği istek anahtarını (idempotency key) taşır. Bağlantı kopup yeniden gönderilen istek ikinci kayıt oluşturmaz. Demleme günü ölçüm kuyruğundaki UUID kuralının bütün API'ye yayılmışı.
- **Dış olaylar:** cihaz webhook'ları, App Store sunucu bildirimleri (abonelik yenileme, iade), web ödeme sağlayıcısının webhook'ları ve GitHub (destek talepleri) aynı alım servisinden girer: imza doğrulanır, kuyruğa yazılır, aynı olay iki kez işlenmez. Abonelik ve yetki durumu yalnız sunucuda tutulur.
- **Demleme paketi bütünlüğü:** paket sunucuda imzalanır ve sürelidir (ör. 72 saat) \[doğrulanacak\]; kuyruktaki ölçümler paket kimliğiyle gönderilir, sunucu paketin o partiye ait olduğunu denetler.

### Destek paneli (iç kullanım)

Destek ekibinin müşteri sorunlarını çözdüğü ayrı bir web uygulaması. Müşteri uygulamasının parçası değil; ekran listesine ve ekran sayılarına girmez. Taslağı konseptte "İç kullanım" grubunda: hesap detayı ve talebin GitHub bağlantısı (talep listesi GitHub'da), arama ve erişim isteği, hesap farkı. Talep ve mesaj müşteri uygulamasındaki Yardım ve destek ekranıyla aynı kayıttır; destek ekibi talepleri GitHub Issues'ta yürütür (aşağıda).

**Ne yapılır**

| İş | Not |
| --- | --- |
| Hesap ve çalışma alanı arama | Kullanıcı kimliği, e-posta, çalışma alanı adı; sonuçta yalnız özet (mod, bölge, abonelik, son giriş) |
| Abonelik durumu | App Store ve web ödeme kaydını yeniden doğrulama; yetkiyi elle düzeltme gerekçeyle |
| Oturum ve cihaz | Oturumları kapatma, cihaz (iSpindel) anahtarını yenileme, giriş yöntemlerini görme |
| Hesap silme ve dışa aktarma | Talepleri izleme; süresi dolan silmeyi tamamlama |
| Silinen kaydı geri getirme | Silinme zamanı işaretli kayıtlar, saklama süresi içinde |
| "Verim kayboldu" ve hesap farkları | Kaydın sürüm geçmişi ve denetim kaydı; hesabı kayıtlı motor sürümüyle ve güncel sürümle yeniden çalıştırıp farkı görme |
| Bildirim teslimi | Bir bildirimin hangi cihaza gittiği, okundu mu, push hatası |
| Sistem durumu | Kuyruk birikimi, cihaz verisi gecikmesi, motor hata oranı (gözlem araçlarına bağlantı) |
| Destek talebi ve mesaj | Talep ve zaman çizelgesi (SupportRequest, SupportMessage); GitHub Issue bağlantısı. İç not müşteriye gitmez; yanıt bildirim kutusuna, push'a ve e-postaya düşer |

**Ne yapılmaz:** tarif, parti, stok gibi müşteri içeriği panelden değiştirilmez. Müşterinin yerine giriş yapılmaz; müşterinin gördüğü ekran yalnız okunur olarak açılabilir.

**Erişim kuralları**

- **Müşteri izni:** çalışma alanının içeriğini (tarif, parti, ölçüm) görmek için müşterinin izni gerekir. İzin destek talebi açılırken veya Ayarlar › Güvenlik › Destek erişimi'nden verilir; 72 saat geçerlidir \[doğrulanacak\]. İşletmede izni yönetici verir. Hesap özeti ve abonelik durumu izinsiz görülür.
- **Gerekçe:** her erişim talep numarası ve gerekçe ister; süre bitince erişim kendiliğinden kapanır.
- **Şeffaflık:** destek erişimleri müşterinin denetim kaydında ve Ayarlar › Güvenlik'te "Destek erişim geçmişi" olarak görünür: kim, ne zaman, neye baktı, ne yaptı.
- **Destek rolleri:** Destek (okuma ve oturum işleri), Destek yöneticisi (abonelik düzeltme, geri getirme, silme tamamlama). Bu roller müşteri rollerinden tamamen ayrıdır.
- **Acil durum:** güvenlik olayı veya yasal zorunlulukta izinsiz erişim yalnız iki kişi onayıyla açılır ve müşteriye sonradan bildirilir.

**Teknik yapı**

- Ayrı bir React uygulaması; aynı tasarım sistemiyle. Genel adresten erişilemez; giriş şirketin Google Workspace hesabı ve passkey ile, kümede çalışan açık kaynak bir OIDC ters vekili (ör. oauth2-proxy) arkasında; buluta bağlı erişim servisi kullanılmaz.
- Çekirdek API'nin ayrı bir iç uç grubu (/internal) kullanılır; bu uçlar genel API ağ geçidinde yayımlanmaz. İş kuralları yine çekirdek API'de, iki kez yazılmaz.
- Her işlem değiştirilemez denetim kaydına yazılır.
- Yol haritası: Ev MVP'de arama, abonelik, oturum, silme talepleri, destek talebi ve mesaj (GitHub eşitlemesi dahil) ve erişim kuralları; diğerleri İşletme aşamasıyla.

### Destek talepleri: GitHub Issues

Destek ekibi talepleri GitHub Issues'ta yürütür. Müşteri GitHub'ı görmez; talebini uygulamadaki Yardım ve destek ekranından açar ve izler. Talebin asıl kaydı çekirdek API'dedir (SupportRequest, SupportMessage); GitHub Issue ekibin çalışma yüzeyidir ve iki yönlü eşitlenir. GitHub'a ulaşılamazsa müşteri yine talep açar ve yazışmasını görür; eşitleme kuyrukta bekler.

| Olay | Uygulama → GitHub | GitHub → uygulama |
| --- | --- | --- |
| Talep açıldı | Özel destek deposunda Issue açılır: başlık, konu ve mod etiketi, uygulama sürümü, talep numarası ve çalışma alanının takma kimliği. Destek paneli bu numarayla açılır; iç adres Issue'ya yazılmaz. Talep numarası ile Issue numarası kayıtta eşlenir | – |
| Müşteri yazdı | Yorum "Müşteri" başlığıyla eklenir; Issue kapalıysa yeniden açılır | – |
| Destek yanıtı | – | `/yanit` ile başlayan yorum (veya destek panelinden yazılan) müşteriye gider: bildirim kutusu, push, e-posta. Diğer yorumlar iç nottur, müşteriye gitmez |
| Durum | Müşteri "Çözüldü" derse Issue kapanır | Issue kapanınca talep "çözüldü"; `bekliyor:musteri` etiketi "yanıtlandı" yapar |
| Hata bağlantısı | – | Destek Issue'su bir geliştirme Issue'suna bağlanır; düzeltmeyi içeren sürüm yayınlanınca müşteriye "2.3.2'de düzeltildi" bildirimi gider |
| Erişim izni | Müşteri izin verdiyse Issue'ya yalnız "izin var, bitiş: …" ve panel bağlantısı yazılır | – |

Etiketler: konu (hesap, abonelik, tarif ve hesap, cihaz, diğer), mod (ev, işletme), öncelik, `bekliyor:musteri`.

**Talep durumları**

| Durum | Ne zaman |
| --- | --- |
| Açık | Talep açıldı; ya da müşteri yanıtlandı veya çözüldü durumundaki talebe yeni mesaj yazdı (Issue yeniden açılır) |
| Yanıtlandı | Müşteriye yanıt gitti (`/yanit` yorumu veya panel); `bekliyor:musteri` etiketi kendiliğinden eklenir |
| Çözüldü | Issue kapandı veya müşteri "Çözüldü" dedi |
| Kapandı | Çözüldü'den 14 gün sonra \[doğrulanacak\]; müşteri sonra yazarsa yeni talep açılır ve eskisine bağlanır |

**Kişisel veri**

- Ayrı, özel bir depo; yalnız destek ekibi erişir. Ürün deposuyla karışmaz.
- Issue'ya ad, e-posta ve çalışma alanı adı yazılmaz; kullanıcı ve çalışma alanı takma kimlikle geçer. Müşterinin yazdığı metin olduğu gibi gider; talep formu "kişisel bilgi yazma" uyarısı gösterir.
- Ekran görüntüsü ve tanılama dosyası GitHub'a yüklenmez; nesne deposunda kalır, Issue'da yalnız ek sayısı yazar, ekler destek panelinde açılır.
- Hesap silinince SupportRequest, SupportMessage ve ekleri silinir, ilgili Issue'lar da silinir; denetim kaydındaki SupportAction satırları kimlik anonimleştirilerek kalır. Issue silinemezse metin anonimleştirilir \[doğrulanacak: GitHub API ile Issue silme yetkisi ve düzenleme geçmişinde eski metnin kalıp kalmadığı\].
- GitHub alt işleyen olarak gizlilik politikasına ve veri işleme sözleşmesine eklenir; veri ABD'de tutulur, İngiltere ve AB aktarımı için GitHub'ın veri işleme sözleşmesi \[doğrulanacak\].

**Teknik yapı**

- Yalnız destek deposuna yetkili bir GitHub App (Issues okuma ve yazma). Anahtarı sır yöneticisinde.
- GitHub'dan gelen olaylar (Issue, yorum, etiket) dış olay alımı servisine webhook olarak gelir; imza doğrulanır, aynı teslim iki kez işlenmez.
- Bizden GitHub'a giden işler outbox ve arka plan işleriyle; API sınırına takılırsa yeniden denenir.
- Yanıt süresi hedefi abonelik katmanıyla birlikte karara bağlanır.

### Hesap motoru servisi

- **Ayrı servis:** dışarıdan erişilemez; yalnız çekirdek API iç ağdan, servisler arası mTLS ile çağırır. Kod istemcide ve genel ağda hiç bulunmaz; bu formül bilgisini de korur.
- **Durumsuz saf fonksiyon:** girdi tam tarif anlık görüntüsü (malzemeler, ekipman, ayarlar), çıktı hesap sonuçları. Veritabanına, kullanıcıya ve yetkiye erişmez. Yatay ölçeklenir.
- **Sürüm damgası:** her sonuç motor sürümünü taşır. Kaydedilen tarif sürümü ve parti, hesaplandığı motor sürümünü saklar. Motor güncellenince eski sonuç değişmez; yeniden hesap farkı gösterilir (İşletmede onaylı tarifte kullanıcı onayıyla).
- **Önbellek:** çekirdek API girdi özetini (hash) anahtar yaparak sonucu Redis'te tutar; aynı girdi ikinci kez hesaplanmaz.
- **Hız hedefi:** tarif hesabı p95 100 ms altında \[doğrulanacak\]. Tarif düzenlerken istemci değişiklikleri yaklaşık 300 ms bekleyip toplu gönderir, dönen sonucu gösterir.
- **Test:** formül test vektörleri (formuller/testler/) ve altın tarif seti her sürümde dağıtımdan önce çalışır; fark varsa dağıtım durur.

### Veri katmanı

| Bileşen | Tutulan | Not |
| --- | --- | --- |
| PostgreSQL (yönetilen) | Tüm kayıtlar, tarif sürümleri, partiler, stok, roller, bildirimler, denetim kaydı | Tek doğruluk kaynağı. Çalışma alanı ayrımı satır düzeyinde güvenlikle; her kayıtta sürüm numarası (iyimser kilit). Okuma yükü için okuma kopyası |
| Ölçümler (aynı PostgreSQL) | Cihaz ve elle ölçümler | Zamana göre bölümlü tablo; eski bölümler özetlenip arşive taşınır. Ayrı zaman serisi veritabanı ilk sürümde yok |
| Redis (yönetilen) | Hesap sonucu önbelleği, oturum, hız sınırı sayaçları | Kaybolursa veri kaybı olmaz, yalnız yavaşlar |
| Nesne deposu (S3 uyumlu) | Taranan kağıtlar, fotoğraflar, PDF ve dışa aktarmalar | Yükleme ve indirme kısa süreli imzalı bağlantıyla doğrudan; İşletme denetim arşivi için değiştirilemez (WORM) saklama |
| Kuyruk | Bildirim, içe aktarma, rapor işleri; cihaz olayları | Başta PostgreSQL üzerinde outbox tablosu ve iş kuyruğu (ek altyapı yok); yük artınca NATS JetStream |
| Arama | Tarif ve malzeme arama | Başta PostgreSQL tam metin araması; ayrı arama motoru gerekmez |

- **Yedek:** sürekli yedek ve noktaya geri dönüş (PITR), başka bölgeye kopya, ayda bir geri yükleme testi. Hedef: veri kaybı en çok 5 dk, kesinti en çok 1 saat \[doğrulanacak\].
- **Veri yeri:** çalışma alanı açılırken bölge seçilir (ABD veya İngiltere); verisi o bölgede kalır. AB bölgesi sonra. İstisna: destek talebinin metni ABD'deki GitHub'da tutulur; bu, bölge seçim ekranında ve gizlilik politikasında açıkça yazılır. İngiltere çalışma alanları için bunun kabul edilebilir olup olmadığı hukuki karar (açık).

### Cihazda tutulan veri

Cihazda olabildiğince az veri tutulur. Hepsi sunucudan yeniden üretilebilir; cihaz kaybolursa veri kaybı olmaz.

| Veri | Nerede | Süre |
| --- | --- | --- |
| Oturum anahtarı | Keychain (Apple); web'de HttpOnly çerez | Oturum boyunca |
| Salt okunur önbellek (son listeler, malzeme kütüphanesi) | Şifreli uygulama alanı | Süreli; açılışta tazelenir, çıkışta silinir |
| Demleme paketi | Şifreli uygulama alanı | Parti bitene kadar |
| Ölçüm kuyruğu | Şifreli uygulama alanı | Sunucu kabul edene kadar (UUID ile tekrar işlenmez) |
| Arayüz tercihleri | Cihaz ayarları | Kalıcı; kişisel veri değil |

Web'de demleme paketi yok; web her işlemde internet ister.

### Bulut ve taşınabilirlik

Hedef: aynı sistem AWS, Azure veya Google Cloud'a kod değişmeden kurulabilsin.

- **Yalnız açık standartlara bağlı yönetilen servisler:** PostgreSQL, Redis, S3 API'si, OIDC, OpenTelemetry, OCI konteyner imajı. Bulutun kendine özgü veritabanı veya kuyruk servisi kullanılmaz.
- **Nesne deposu soyutlaması:** Azure Blob S3 API'si konuşmadığı için depo erişimi tek bir kütüphane arayüzünden yapılır (S3, Azure Blob, GCS destekli).
- **Altyapı kodla:** Terraform/OpenTofu modülleri; her bulut için yalnız alt modül değişir. Kubernetes tanımları ortak.
- **Kimlik:** Apple ve Google OIDC standardıyla doğrulanır; doğrulama kendi kodumuzda, buluta bağlı kimlik servisi yok.
- **İlk bulut seçimi**: Google Cloud (Mimari kararlar). Taşınabilirlik bu kararı geri alınabilir yapar.

### Ortamlar ve işletim

- **Ortamlar:** geliştirme, test, canlı. Hepsi aynı altyapı koduyla; test ortamında anonimleştirilmiş veri.
- **Dağıtım:** GitOps; her değişiklik depo üzerinden. Hesap motoru yeni sürümü önce küçük bir trafik payıyla (canary) açılır.
- **Gözlem:** log, metrik ve iz OpenTelemetry ile; hangi araca gideceği değiştirilebilir. Hesap motoru süresi ve hata oranı için alarm.
- **Güvenlik:** servisler arası mTLS, Kubernetes ağ politikaları (hesap motoruna yalnız çekirdek API erişir), sırlar sır yöneticisinde ve KMS ile şifreli, imaj ve bağımlılık taraması (ayrıntı: Güvenlik bölümü).

### Mimari kararlar

- **Sunucu kodu Rust.** Kodu Claude yazacağı için seçim ölçütü, hatanın kod çalışmadan yakalanması: güçlü tip sistemi, SQL sorgularının derleme sırasında veritabanı şemasına karşı denetlenmesi (sqlx), hesap motoruyla tek dil ve tek araç zinciri, OpenAPI sözleşmesinin koddan üretilmesi. Çekirdek API, arka plan işleri ve cihaz verisi alımı aynı Rust çalışma alanında, ayrı dağıtımlar olarak. İstemcilerde de tipli diller: Swift ve TypeScript (strict).
- **Birleştirme kuralı:** her değişiklik derleme, tip kontrolü, lint, birim testleri, formül test vektörleri ve API sözleşme testinden geçmeden ana dala girmez. Claude'un yazdığı kod da insanın yazdığı gibi bu kapıdan geçer.
- **İlk bulut Google Cloud.** Karşılıklar aşağıdaki tabloda.
- **Kimlik:** Apple ve Google ile giriş, doğrulama çekirdek API'de; ayrı kimlik servisi yok (Kimlik ve giriş).

### Google Cloud karşılıkları

| Bileşen | Google Cloud | Taşınırken |
| --- | --- | --- |
| Kubernetes | GKE Autopilot | EKS, AKS |
| PostgreSQL | Cloud SQL for PostgreSQL (yüksek erişilebilir, okuma kopyası) | RDS, Azure Database for PostgreSQL |
| Redis | Memorystore for Redis | ElastiCache, Azure Cache for Redis |
| Nesne deposu | Cloud Storage | S3, Azure Blob |
| Kenar | Cloud Load Balancing, Cloud Armor (WAF), Cloud CDN | ALB, AWS WAF, CloudFront; Azure Front Door |
| Sır ve şifreleme | Secret Manager, Cloud KMS (Kubernetes'e External Secrets ile) | Secrets Manager ve KMS; Key Vault |
| İmaj deposu | Artifact Registry | ECR, ACR |
| Gözlem | Cloud Logging, Monitoring, Trace (OpenTelemetry üzerinden) | CloudWatch, Azure Monitor veya Grafana |
| E-posta | Google Cloud'da yok; ayrı sağlayıcı (ör. Postmark, SendGrid) \[doğrulanacak\] | Aynı sağlayıcı |
| Bölgeler | ABD: us-east4 (Virginia); İngiltere: europe-west2 (Londra) \[doğrulanacak\] | Aynı şehirlerdeki bölgeler |
| Destek paneli girişi | Kubernetes'te OIDC ters vekili (ör. oauth2-proxy), Google Workspace ile giriş | Aynı (buluttan bağımsız) |

Kullanılmayanlar: Firestore, Pub/Sub, Spanner ve Firebase Authentication gibi Google'a özgü servisler. Bunlar taşınmayı kod değişikliğine çevirir.

### Kimlik ve giriş

- **Ayrı kayıt yok:** kayıt formu ve şifre yok. İlk girişte hesap kendiliğinden açılır; kurulum yalnız mod, bölge ve ekipmanı sorar.
- **Yöntemler:** Apple ile giriş ve Google ile giriş (iPhone, iPad, web). Yedek: e-postaya tek kullanımlık giriş bağlantısı. Apple veya Google hesabı olmayanlar ve işletmede iş e-postasıyla girmek isteyenler için.
- **Doğrulama:** Apple ve Google'ın kimlik belirteci (OIDC) çekirdek API'de imza, yayıncı ve hedef uygulama açısından denetlenir; ardından kendi kısa ömürlü oturumumuz verilir. Kullanıcı kaydında e-posta değil, sağlayıcı ve sağlayıcıdaki kalıcı kimlik tutulur (Apple e-postayı gizleyebilir).
- **Hesap bağlama:** aynı kişi Apple, Google ve e-postayı tek hesaba bağlayabilir (Ayarlar › Güvenlik). Biri kaybolsa da hesaba girilir.
- **Davet:** e-posta adresine göre eşleştirilmez; tek kullanımlık davet bağlantısıyla kabul edilir. Davet edilen kişi istediği yöntemle girer.
- **Yöneticiler:** ikinci adım olarak passkey (Face ID veya Touch ID) zorunlu.
- **Ortak cihaz:** cihazı bir yönetici açar; çalışanlar kişiye özel PIN ile geçer.
- **Apple kuralları:**
  - Google gibi bir üçüncü taraf girişi sunan uygulama, gizlilik odaklı bir seçenek de sunmalı. Apple ile giriş bunu karşılar.
  - Hesap silme uygulama içinden yapılabilmeli; silmede Apple belirteci de iptal edilir.
  - Apple'ın gizli e-posta adreslerine posta göndermek için gönderen alan adımız Apple Developer hesabında kaydedilir. Yoksa hatırlatıcı ve giriş e-postaları ulaşmaz.
- **Google kuralları:** iPhone ve iPad'de giriş sistem tarayıcı oturumuyla yapılır (uygulama içi web görünümü yasak). Google Cloud projesinde OAuth onay ekranı doğrulaması gerekir.
- **İleride:** Microsoft ile giriş, Microsoft 365 kullanan işletmelerden talep gelirse.

## Hesap motoru

Formüllerin çoğu BeerSmith spesifikasyonundan, bilinen tutarsızlıkları düzeltilerek alınır; geç hop IBU ve priming hesabı Brewtarget yaklaşımından gelir. Kaynaklar: formül spesifikasyonu (`formuller/`) ve BeerSmith–Brewtarget karşılaştırması (`karsilastirma/`). İç birim SI.

| Alan | Seçim | Not |
| --- | --- | --- |
| OG / FG | BeerSmith malzeme bazlı FG + Brewtarget kısmi fermente olabilirlik oranı | Mash sıcaklığı düzeltmesi ve maya toleransı açılıp kapatılabilir; sanal OG; şişeleme şekeri hariç |
| ABV | Balling varsayılan | HMRC ve basit formül seçenek |
| Renk | Morey, renk °L | Lovibond katsayısı 1.3546 (BeerSmith'teki 1.3564 hatalı) |
| IBU | Tinseth varsayılan, tam SI sabiti | Geç hop ve whirlpool için mIBU (üstel soğuma); sabit sıcaklıkta hop stand; IBU hacmi parti + trub; geç ekstrakt segmentleri; rakım seçenek; hop yaşlanması |
| Su ve pH | BeerSmith iyon tablosu (molar kütleden, hidrat formları açık) | Malt bazlı grist pH + MPH/BW eğim seçeneği; pKa tabanlı asit; ağırlıklı NNLS eşleştirme. Brewtarget su motoru kaynak olarak kullanılmaz |
| Maya | BeerSmith üstel canlılık, White eğrisi (kırpılmış) | İsteğe bağlı Braukaiser; iki kademe starter; kuru mayada g/hL |
| Karbonasyon | Brewtarget stokiyometrisi + BeerSmith kalıntı CO₂ | Fıçı basıncı BeerSmith polinomu |
| Mead / şarap | BeerSmith formülleri | Tarifteki YAN hatası düzeltilir |
| Mash ve hacim | Tek ısı dengesi fonksiyonu, tahıl hacmi 0.652 L/kg | Ekipman bazlı emilim + ayrı BIAB; %4 soğuma büzülmesi; BeerSmith batch sparge algoritması |

Her formül kararı nedeniyle birlikte karar kaydına (ADR) yazılır ve altın test setiyle (10–20 referans tarif) doğrulanır.

### Eksik hesaplar

Mevcut spesifikasyon BeerSmith formülleri ve Brewtarget karşılaştırmasındaki düzeltmelerden oluşuyor. Aşağıdaki tablo BeerSmith ve Brewtarget'ta olmayan hesaplar; hangilerinin yazıldığı tablonun altındaki Durum notunda, "Sonra" satırları henüz yazılmadı; karşılaştırma bilgiye dayanıyor, rakiplerin güncel özellik listeleriyle doğrulanmadı.

| Eksik hesap | Kullanan ekran / neden | Segment | Aşama |
| --- | --- | --- | --- |
| Fermantasyon tahmini (okumalara eğri oturtma, FG ve bitiş tarihi) | Fermantasyon "Öne Çıkanlar", uyarı kuralları | İkisi | Ev MVP |
| Cihaz kalibrasyonu (iSpindel açı → SG eğrisi, Tilt sapması) | Cihazlar | İkisi | Ev MVP |
| Isınma ve soğuma süresi (güç, hacim, soğutucu) | Demleme günü zaman çizelgesi, takvimde geriye planlama | İkisi | Ev MVP |
| Ekipmana göre ölçekleme (verim ve kayıplar yeniden) | Ölçekleme | İkisi | Ev MVP |
| İyon dengesi kontrolü, sertlik birimleri (°dH, °fH, mEq/L) | Su raporu | İkisi | Ev MVP |
| Dry hop ve meyve emilimi, kazanda hop tortusu kaybı | Hacim zinciri | İkisi | Ev MVP |
| Fermantasyon adımına gravite tetikleyicisi (ör. diacetyl rest FG'ye 2–4 puan kala, lagering süresi ve sıcaklık düşüş hızı) | Fermantasyon profili, uyarı kuralları, Bugün | İkisi | Ev MVP |
| Fıçıda priming miktarı kuralı (şişeyle aynı mı, daha az mı; BeerSmith ve Brewtarget'ta tanımsız) | Araçlar, Şişele / fıçıla | İkisi | Ev MVP |
| Maliyet (FIFO lot fiyatıyla parti ve litre) | Raporlar, stok | İkisi | İşletme çekirdek |
| Harmanlama (ABV, IBU, renk, CO₂) | Tank hareketi › Birleştir | İşletme | İşletme çekirdek |
| Hasat edilen mayayı tekrar kullanma (slurry yoğunluğu, canlılık → hacim) | Maya yönetimi | İşletme | İşletme çekirdek |
| Profesyonel verim (Plato/hL brewhouse verimi; dönüşüm ve süzme verimi ayrı) | Raporlar, kalibrasyon önerisi | İşletme | İşletme çekirdek |
| Malt analizinden ekstrakt potansiyeli (kuru/olduğu gibi, nem, ince/kaba) | Kendi malzemen | İkisi | İşletme çekirdek |
| Cask (İngiltere): düşük karbonasyon hedefi, cask'ta priming ve durultucu (finings), kondisyon süresi (değerler doğrulanacak) | Ambalaj kaydı, araçlar; İngiltere hedef pazar | İşletme | İşletme çekirdek |
| Etiket enerjisi (AB, 100 mL başına kJ/kcal) | Etiket | İşletme | İşletme 2. sürüm |
| Vergi ve beyan (ülkeye göre ÖTV, ABV veya Plato bazı; ülke ayrıntısı "Üst segment ve pazarlar" bölümünde) | Mevzuat raporu | İşletme | İşletme 2. sürüm |
| İngiltere vergisi: ABV bandı, fıçı indirimi, Small Producer Relief tahmini | Mevzuat raporu | İşletme | İşletme 2. sürüm |
| Avustralya yıllık iade sınırı takibi ($400.000) | Mevzuat raporu, Pano | İşletme | İşletme 2. sürüm |
| Depozito ve ambalaj raporu (kap sayısı, malzeme, ağırlık) | Ambalaj kaydı, Mevzuat raporu | İşletme | İşletme 2. sürüm |
| Spunding (tankta basınçla doğal karbonasyon) | Tank detayı, araçlar | İşletme | İşletme 2. sürüm |
| BBT'de zorla karbonasyon: basınç fıçıyla aynı formülle; taşla karbonasyonda süre tahmini | Tank detayı | İşletme | İşletme 2. sürüm |
| Fıçı hattı dengesi (hortum çapı, uzunluk, basınç) | Araçlar | İkisi | Sonra |
| Meyve eklemesi (şeker, hacim, asit) | Tarif editörü, mead/cider | İkisi | Sonra |
| pH'ın sıcaklıkla düzeltilmesi; kaynatma sonrası ve fermentör pH'ı | Su ve pH, kalite | İkisi | Sonra |
| Cider ve şarapta toplam asitlik (TA) ve asit ayarı | Mead/cider tarifi | İkisi | Sonra |
| Küçükler: Noonan IBU, Mosher ve Daniels renk, Terrill doğrusal/kübik ve Novotný refraktometre, BU:GU, hop yağı oranı | Ayarlar › Hesaplama, tarif editörü | İkisi | Sonra |
| Ekşi ve karışık fermantasyon (kettle sour pH hedefi), fıçıda (barrel) yıllandırma | Tarif editörü, fermantasyon profili | İkisi | Sonra |

**Durum:** Ev MVP'deki altı hesabın (`formuller/07-ev-mvp-ek-hesaplar.md`, `testler/07_ev_mvp_ek_tv.py`) ve İşletme çekirdekteki beş hesabın (`formuller/08-isletme-cekirdek-hesaplar.md`, `testler/08_isletme_cekirdek_tv.py`) spesifikasyonu ve test vektörleri yazıldı; az okumayla tahmin, ekstrakt araçları ve soğutmasız IBU da yazıldı (formuller/10). Gravite tetikleyicisi, fıçıda priming ve cask hesapları da yazıldı (formuller/11). Fıçı için "şişenin yarısı" alışkanlığı yerine fizik kullanılır: kafa boşluğu yüzünden fıçı şişeden biraz fazla şeker ister; kısmi priming + CO₂ ile tamamlama ayrı mod. Bunlar BeerSmith'ten değil; endüstri tanımları ve bizim tasarımımız. Doğrulanacak varsayılanlar: ısıtma verimi, soğutucu zaman sabiti ve debisi, hop ve meyve emilimi, doğal gaz ısıl değeri, varsayılan slurry yoğunluğu, ince–kaba farkı; formül 10 ve 11'in varsayılanları (gecikme fazı, FG belirsizliği, ekstrakt yoğunlukları, küp soğuma süresi, tetik eşikleri, lager soğutma hızı ve süresi, kısmi priming oranı, cask hedefi, durultucu oranı, kondisyon süresi). İşletme 2. sürümde depozito raporu ve BBT'de taşla karbonasyon süresi henüz yazılmadı.

İşletme 2. sürümdeki üç hesap da yazıldı (`formuller/09-isletme-2-hesaplar.md`, `testler/09_isletme_2_tv.py`): AB etiket enerjisi, vergi motoru ve spunding. Vergi motoru yalnız yöntemleri içerir (hacim, saf alkol, hL·°P, Plato bandı, yıllık kademe); oranlar, bantlar ve indirimler kaynak ve tarihleriyle veri tablosunda tutulur. Dosyadaki oranlar örnek; Türkiye ÖTV tutarının güncel değeri ve birimi Resmi Gazete'den doğrulanacak.

## Analitik modeller (fikir, onaylanmadı)

**Durum:** aşağıdaki 18 model yalnız fikir olarak listelendi; hiçbiri onaylanmadı ve kapsama alınmadı. "Önerilen aşama" sütunu da öneridir. 1, 3 ve 16'nın hesapları zaten yazıldı ya da tasarlandı (07 §1, Kalibrasyon ekranı, 08 §4); onların analitik model olarak sunulması da bu listenin parçasıdır.

Veri az olduğu için (yılda 10–100 parti) az veriyle çalışan, açıklanabilir ve güven aralığı gösteren modeller; Rust motorunda sunucuda çalışır.

| No | Model | Ne yapar | Yöntem | Segment | Önerilen aşama |
| --- | --- | --- | --- | --- | --- |
| 1 | Fermantasyon bitiş tahmini | FG ve bitiş günü, güven aralığıyla | Üstel azalma eğrisi (07 §1); cihazı olmayan partide 2 elle okumayla (10 §1) | İkisi | Ev MVP |
| 2 | Takılan fermantasyon uyarısı | Eğri beklenenden saparsa erken uyarı | Beklenen eğriden sapma ve eğim eşiği | İkisi | Ev MVP |
| 3 | Kalibrasyon | Ekipman profilini partilerden öğrenir | Kayan ortalama ve 3 parti tutarlılık kuralı | İkisi | Ev MVP |
| 4 | Kişisel attenuasyon | Maya üreticisinin değerini kullanıcının partileriyle düzeltir | Bayesçi güncelleme | İkisi | Ev 2. sürüm |
| 5 | Demleme günü süre tahmini | Adım sürelerini ekipmandan öğrenir, bitiş saati | Isınma/soğuma modeli (07 §3) ve geçmiş süreler | İkisi | Ev 2. sürüm |
| 6 | Maya canlılığı ve hop alfa tahmini | Stoktaki maya ve hopun bugünkü değeri | Yaşlanma formülleri (04 §1.4, 02 §10) | İkisi | Ev MVP |
| 7 | Benzer tarifler ve stile uzaklık | Stil merkezine yakınlık, kütüphanedeki en benzer tarif | Normalleştirilmiş değerler ve malzeme oranlarında uzaklık | İkisi | Ev 2. sürüm |
| 8 | Hop ikamesi | Stokta olmayan hop için en yakın ikame | Yağ bileşimi benzerliği | İkisi | Ev 2. sürüm |
| 9 | Cihaz arızası tespiti | Pil, kayma, donmuş ölçüm | Sinyal gücü, okuma sıklığı, ani sıçrama kuralları | İkisi | Ev MVP |
| 10 | İstatistiksel süreç kontrolü | OG, FG, pH, renk, çözünmüş oksijen kontrol grafikleri; kayma uyarısı | Kontrol grafikleri, sapma kuralları, Cpk | İşletme | İşletme çekirdek |
| 11 | Tadıma neyin etkilediği | Tadım puanı ve kusurlarla süreç değerleri ilişkisi (neden-sonuç değil) | Basit regresyon, grup karşılaştırması; en az 10–15 parti | İşletme | İşletme 2. sürüm |
| 12 | Hammadde ihtiyacı ve sipariş zamanı | 4–8 haftalık ihtiyaç, sipariş tarihi | Plan × tarif tüketimi, geçmiş tedarik süreleri | İşletme | İşletme çekirdek |
| 13 | Son kullanma riski | Lot tarihinden önce bitecek mi | FIFO simülasyonu | İşletme | İşletme çekirdek |
| 14 | Tank kapasitesi ve darboğaz | Doluluk, çakışma, planın tanklara otomatik yerleştirme önerisi | Takvim simülasyonu, kısıt çözücü | İşletme | İşletme 2. sürüm |
| 15 | Maliyet sapması | Litre maliyeti değişiminin fiyat, miktar ve fire etkisine ayrılması | Etki ayrıştırma | İşletme | İşletme 2. sürüm |
| 16 | Verim kaybı analizi | Düşük verim mash'ten mi süzmeden mi | Dönüşüm ve süzme verimi (08 §4) | İşletme | İşletme çekirdek |
| 17 | Anonim ortak veri (isteğe bağlı) | Kullanıcı onayıyla maya ve ekipman istatistikleri birleştirilir; yeni kullanıcıya daha iyi varsayılan | Toplu istatistik; tarif paylaşımı değil | İkisi | Karar gerekli: "topluluk yok" kararına dokunuyor, açık onay (UK GDPR, ABD eyalet yasaları, KVKK) |
| 18 | Doğal dille soru sorma | Kullanıcının kendi verisi üzerinde soru-cevap; cevap veriye bağlantı verir | Büyük dil modeli | İkisi | Karar gerekli: maliyet ve gizlilik; ilk sürüm için gerekmez |

**Ortak ilkeler (onaylanırsa):** tahmin her zaman aralıkla verilir; veri yetersizse model susar ve nedenini söyler; her sonuç "nasıl hesaplandı" sayfasıyla açıklanır; model hiçbir şeyi kendi kendine değiştirmez, yalnız önerir.

## Tarif sihirbazı

Sihirbaz dört yoldan başlar: boş, şablon, var olan tarif veya içe aktarma. Hangi yoldan gelinirse gelinsin aynı adımlardan geçilir; dolu gelen adımlar özet olarak gösterilir, kullanıcı yalnızca değiştirmek istediğine dokunur.

### Başlangıç seçenekleri

| Başlangıç | Ne olur | Sihirbazın özel işi |
| --- | --- | --- |
| Boş | Stil ve hedeflerden başlangıç malt ve hop listesi önerilir | – |
| Şablon | Hazır başlangıç tarifleri (stil başına) | Ekipmana ve hacme ölçekleme |
| Var olan tarif | Kendi tarifinden kopya ya da yeni sürüm | Farklı ekipmana, hacme, verime uyarlama; hedefleri koruyarak ölçekleme; istenen adımda değişiklik |
| İçe aktarma | BeerXML, BeerJSON, BeerSmith `.bsmx`, Brewfather ve Brewer's Friend dışa aktarımları | Eşleştirme ve uyarlama adımı (aşağıda) |

### İçe aktarmada eşleştirme adımı

1. **Malzemeler:** her malzeme kütüphaneyle eşleştirilir (isim, üretici, renk, alfa). Emin olunamayan eşleşme sorulur; eşleşmeyen malzeme dosyadaki değerleriyle yeni kayıt olur. Stok durumu gösterilir.
2. **Ekipman:** kaynak tarifin ekipmanı ve verimi gösterilir, kullanıcının ekipmanı seçilir; hacim ve kayıplar buna göre yeniden hesaplanır.
3. **Hedefleri koru:** "Orijinal OG/IBU/renk korunsun mu?" seçilirse malt ve hop miktarları yeni ekipmana göre ayarlanır; seçilmezse miktarlar aynı kalır, tahminler değişir.
4. **Stil:** kaynak stil kodu seçili kılavuza eşlenir (ör. BJCP 2015 → BJCP 2021 veya BA).
5. **Birimler ve formüller:** kaynaktaki IBU formülü ve birimler okunur; farklıysa iki değer yan yana gösterilir (ör. "kaynakta 38 IBU, bizim hesapla 36").
6. **Özet:** değişen her şey listelenir; kaynak dosya tarife ek olarak saklanır.

### Adımlar

1. Başlangıç (boş, şablon, var olan, içe aktar)
2. Tür: bira, mead, cider, seltzer; all grain, extract, BIAB
3. Ekipman: kayıtlı profil veya 4 soruluk hızlı ekipman sihirbazı
4. Stil: önce kılavuz (BJCP, BA, mead/cider), sonra stil
5. Hedefler: hacim, OG, IBU, renk
6. Malt: önerilen fatura, yüzdeyle düzenleme
7. Hop: acılık, aroma, whirlpool, dry hop; IBU hedefe oturur
8. Maya: paket, yaş, starter önerisi
9. Su (isteğe bağlı): kaynak ve hedef profil, tuz ve asit önerisi
10. Özet: uyarılar, alışveriş listesi, demleme gününü planla

Her adımdan "Editöre geç" ile çıkılır, hiçbir şey kaybolmaz. Apple'da iPhone Kurulum Asistanı gibi ekran başına tek soru; iPad'de form sheet.

## Tarif editörü ve stil kılavuzları

Editör Numbers gibi çalışır: tablo değişince tüm değerler sunucuda yeniden hesaplanır (değişiklikler kısa aralıklarla toplu gönderilir), sağdaki Inspector stil uygunluğunu ve tarif kontrolünü gösterir.

- **Hedefle ayarlama:** OG, IBU, renk hedefi verilir, miktarlar ayarlanır; bir değer kilitlenip diğeri kaydırılır. Tahıl yüzdesi doğrudan düzenlenir.
- **Tarif kontrolü:** sorun, nedeni ve tek dokunuşla düzeltme (ör. "Mash pH 5.62, 3.5 mL %88 laktik ekle").
- **Kaynatma zaman çizelgesi:** iMovie gibi; hop eklemeleri sürüklenebilir klipler, IBU sunucudan güncellenir. iPhone'da salt okunur liste.
- **Dönüşümler:** all grain ↔ extract, ölçekleme (hacim, verim, ekipman), malzeme ikamesi.
- **iPhone:** görüntüleme ve basit düzenleme (miktar, süre); gruplu liste, kaydırarak düzenle/sil, alttan açılan seçici.

### Stil kılavuzları

BJCP 2021, BJCP mead/cider ve Brewers Association gömülü gelir; kullanıcı kendi kılavuzunu oluşturabilir veya içe aktarabilir. BA'nın yeni yıl kılavuzu uygulama güncellemesi beklemeden veri güncellemesiyle gelir.

- **Nerede seçilir:** ayarlarda varsayılan; sihirbazın stil adımında; editör Inspector'ında her an; iki kılavuzu yan yana karşılaştırma; dışa aktarma ve yarışma formunda.
- **Kılavuz değişince:** en yakın stil önerilir (BJCP 18B → BA American-Style Pale Ale), emin olunamazsa sorulur; tarif kontrolü yeni aralıklarla çalışır.
- **Tarif saklar:** kılavuz adı, yıl, stil kodu ve seçim anındaki aralıklar.
- **Lisans:** BA ve BJCP metinlerini dağıtmadan önce kullanım şartları kontrol edilir; izin çıkmazsa yalnız sayısal aralıklar tutulur.

## Demleme günü

Tarif plan, parti gerçekleşendir; demleme gününde yapılan her değişiklik partiye yazılır, tarif değişmez. Üretim ortamı ıslak olduğu için uygulama kağıtla birlikte çalışır.

- **Ekran:** iPhone'da tam ekran tek adım, halka sayaç, "sıradaki adım" kartı, büyük düğmeler, ekran kapanmaz. iPad'de adım listesi, aktif adım ve ölçümler aynı ekranda.
- **Sistem:** kilit ekranında Live Activity, Dynamic Island'da sayaç, zamana duyarlı bildirimler; indirilen demleme paketiyle internetsiz çalışır.
- **Sayaç isteğe bağlı:** her adımda Başlat / Bitir; saatler elle girilir ("Şimdi", "5 dk önce", "15 dk önce", "Plan gibi"); başlangıç + bitiş ya da başlangıç + süre. Adım sayaç bitmeden kapatılabilir; sonraki adımların saatleri kayar.
- **Düzeltmeler:** gerçek miktar, süre, sıcaklık, hacim; malzeme ikamesi; ek su veya şeker; adım ekle, atla, sırasını değiştir. Her sapma işaretlenir, OG/IBU/ABV gerçek değerlerle sunucuda yeniden hesaplanır (bağlantı yoksa bağlantı gelince). İstenirse "Gerçekleşeni tarifin yeni sürümü olarak kaydet".
- **Ölçüm yorumu:** girilen ölçüm yorumlanır (internetle; paketteki hedef değerle basit karşılaştırma internetsiz de çalışır) (ör. "hacim 0.6 L düşük ama SG hedefte, düzeltme gerekmiyor").
- **Yazdırma:** A4 brew sheet; plan değerleri, başladı/bitti/gerçek değer kutuları, ölçüm kutuları, boş satırlar, QR kod. iPad'den AirPrint, web'den yazdır veya PDF.
- **Kağıttan aktarma:** ekran kağıtla aynı sıra ve numaralarla açılır; "plan gibi" tek dokunuş, yalnız farklı değerler yazılır.
- **Belge tarama:** VisionKit ile tarama; QR kod taramayı partiye bağlar; aktarılan değerlerle yan yana görüntülenir. İşletme modunda zorunlu yapılabilir ve saklama süresince silinemez. Yazı tanıma ile ön doldurma sonraki aşama; değerler yalnız öneri olarak gelir.
- **Denetim:** işletme modunda her düzeltme kim, ne zaman, önceki değer ile kaydedilir.

## Fermantasyon ve eşzamanlı üretim

Her demleme ayrı bir parti kaydıdır ve aynı anda istenen sayıda parti yürütülebilir. Parti aşamaları: Planlandı → Demleniyor → Fermantasyonda → Kondisyon → Ambalaja hazır → Ambalajlandı → Kapandı; sapmada Beklemede (ayrıntı: Ekran akışı › Parti durumları).

- **Fermantasyon takibi (Sağlık uygulaması gibi):** Tilt, iSpindel, RAPT okumaları ve elle okuma; SG, sıcaklık ve pH grafiği; "Öne Çıkanlar" yorumları (bitiş tahmini, dry hop zamanı); ana ekran widget'ı.
- **Ev modu:** Partiler listesi aşamaya göre gruplu; birden fazla fermentor; aynı gün iki demlemede iki ayrı Live Activity, kilit ekranında parti adı her zaman görünür.
- **İşletme modu, üretim panosu:** her tank bir durum kutucuğu (Ev uygulaması gibi); aşama sütunları; haftalık tank takvimi; kapasite ve çakışma uyarısı; CIP durumu tankın kullanılabilirliğine bağlı.
- **Tank hareketleri:** transfer (FV → BBT), bölme (bir demleme iki tanka), birleştirme (iki demleme tek tanka), hepsi hacim kayıtlarıyla.
- **İzlenebilirlik:** parti kodu ve lot numarası; hammadde lotundan ambalaja kadar.
- **Bugün görünümü:** tüm partilerin görevleri saatine göre tek listede.
- **Yazdırma:** günlük tank turu listesi (tank, parti, aşama, sıcaklık, SG, görev kutusu), haftalık tank takvimi, fermantasyon okuma ve ambalaj formları.

### Ambalaj ve kap türleri

Dört kap türü var: şişe, kutu, keg ve cask. Tür; karbonasyon yöntemini, ambalaj kaydını, vergi indirimini ve depozito raporunu belirler.

| Kap türü | Tipik boyutlar | Karbonasyon | Ev | İşletme | Vergi ve raporlar |
| --- | --- | --- | --- | --- | --- |
| Şişe | 0,33 L, 0,5 L, 0,75 L; ABD 12 oz (355 mL), 22 oz | Şişede kondisyon (priming) veya karbonatlı dolum | Açık ("Şişele") | Açık | Tek kullanımlık ise ambalaj atığı (EPR) raporu; İngiltere depozito sisteminin camı kapsayıp kapsamadığı doğrulanacak |
| Kutu | 0,33 L, 0,44 L, 0,5 L; ABD 12 oz, 16 oz (473 mL) | Karbonatlı dolum (BBT'den) | Kapalı | Açık | İngiltere depozito sistemi (1 Ekim 2027) ve EPR; dolumda çözünmüş oksijen ölçümü laboratuvar kaydında |
| Keg | Ev: 19 L (corny). İşletme: 20, 30, 50 L; ABD 1/6 bbl (19,5 L), 1/4 bbl (29,3 L), 1/2 bbl (58,7 L) | Basınçla (zorla) veya priming | Açık ("Fıçıla") | Açık | İngiltere fıçı indirimi: 20 L ve üstü, pompa veya gazlı musluk; keg takibi kapsam dışı |
| Cask | Pin 20,5 L, firkin 40,9 L, kilderkin 81,8 L | Cask'ta priming ve durultucu (finings), düşük CO₂ | Kapalı | Açık (İngiltere) | İngiltere fıçı indirimine girer (el pompasıyla servis); pin sınırda, doğrulanacak |

- **Tanım yeri:** Ayarlar › Ekipman ve tanklar › Kaplar. Her kap için tür, hacim, tek kullanımlık mi iade mi, malzeme ve boş ağırlık (EPR için) girilir; kullanıcı yeni boyut ekleyebilir.
- **Ambalaj kaydı (İşletme):** her satır kap + adet; litre, fire ve ambalaj verimi kendiliğinden hesaplanır, lot kodu her satıra yazılır. Bir parti birden fazla kap türüne bölünebilir.
- **Ev modu:** fermantasyondaki "Şişele / fıçıla" eylemi yalnız şişe ve keg sunar; priming veya basınç kap türüne göre önerilir.
- **Karbonasyon önerisi:** şişe → priming, kutu → karbonatlı dolum, keg → basınç, cask → cask priming. Kullanıcı değiştirebilir.
- **Vergi ve raporlar:** fıçı indirimi uygunluğu kap türü ve hacminden; depozito ve EPR raporu kap sayısı, malzeme ve ağırlıktan gelir.

## Apple tasarım ilkeleri

Her ekran benzer işi yapan bir Apple uygulamasının desenini kullanır; web aynı düzeni izler. Aşağıda öne çıkanlar var; tüm ekranların eşleştirmesi konseptin "Her ekranın Apple'daki karşılığı" tablosunda. Fikir alınır, görünüm kopyalanmaz: Apple'ın standart bileşenleri kullanılır.

| Bizim ekran | Apple uygulaması | Alınan fikir |
| --- | --- | --- |
| Tarif sihirbazı | Kurulum Asistanı, Sağlık kurulumu | Ekran başına tek soru, seçim kartları, sabit "Devam" |
| Tarif editörü | Numbers, Notlar | Kenar çubuğu + içerik + Inspector |
| Kaynatma programı | iMovie, GarageBand | Sürüklenebilir hop klipleri |
| Demleme günü | Fitness, Saat, Haritalar | Canlı seans, halka sayaç, sıradaki adım kartı, Live Activity |
| Manuel giriş | Sağlık "Veri Ekle", Takvim | Saat ve sürenin elle girilmesi ve düzenlenmesi |
| Fermantasyon | Sağlık | Öne Çıkanlar, özet kartları, G/H/A grafik, widget |
| Üretim panosu | Ev, Takvim, Anımsatıcılar | Tank kutucukları, haftalık takvim, aşama sütunları |
| Malzeme listesi (iPhone) | Ayarlar, Anımsatıcılar | Gruplu liste, kaydır-düzenle, sheet seçici |
| Belge tarama | Notlar, Dosyalar | VisionKit tarayıcı |

- **Yerleşim:** iPad'de NavigationSplitView + Inspector; iPhone'da en fazla 5 sekme, büyük başlık; seçiciler sheet.
- **Görsel dil:** SF Pro, sayılarda tabular rakam, SF Symbols, sistem renkleri (gri zeminler, yeşil, kırmızı, mavi) + tek vurgu rengi amber, Dynamic Type.
- **Tema:** Açık ve koyu tema; varsayılan "Sistem" (iOS ve tarayıcı ayarını izler). Ayarlar › Görünüm'den Açık veya Koyu seçilebilir; seçim cihaz başına. Her renk iki tema için ayrı tanımlanır.
- **Renk ve karşıtlık:** Amber iki tonda. Yazı ve ikonlarda koyu amber: açık temada #985600 (beyaz üzerinde 5.7:1, gri zemin ve düğme zemininde en az 4.6:1). Geniş alanlarda (sayaç halkası, büyük düğmeler, sıradaki adım kartı, seçili kenar çubuğu, grafik çubukları, takvim şeritleri) parlak amber #E8901F; üzerindeki yazı koyu (#1A1206, 7.5:1). Koyu temada ikisi de #F0A03C (yaklaşık 8:1). Küçük yazıda en az 4.5:1. iOS "Karşıtlığı Artır" açıkken iki ton da koyu ambere döner, üstündeki yazı beyaz olur.
- **Baskı:** Yazdırılan formlar (brew sheet, tank turu, etiket) temadan bağımsız; her zaman beyaz kağıt üzerinde siyah.
- **Operatör görünümü (ISA-101'den):** Üretim panosu, Bugün, Tank detayı ve Demleme günü için isteğe bağlı görünüm. Normal durum gri ve renksiz; aşama renkleri kalkar. Renk yalnız sapmada: 1 kırmızı kare acil, 2 turuncu eşkenar dörtgen uyarı, 3 cam göbeği daire bilgi (şekil ve numara rengi tekrarlar). Değerlerin yanında normal aralık bantlı küçük ölçekler; yanıp sönme yalnız onaylanmamış acil alarmda; onaylama kişiye yazılır. Ayarlar › Görünüm'den cihaz başına açılır (ör. üretim alanındaki duvar iPad'i). Tarif, rapor ve ayar ekranları değişmez; yazı ve bileşenler Apple dilinde kalır.
- **Kontroller:** Stepper, Slider, Picker, segmented control; kaydırarak düzenle/sil; bağlam menüsü; iPad'de klavye kısayolları; dokunma alanı en az 44 pt.
- **Sisteme entegrasyon:** Live Activity, widget, Siri ve Kısayollar, zamana duyarlı bildirim.
- **Erişilebilirlik:** WCAG 2.2 AA hedeflenir. Okunması gereken her metin iki temada da en az 4.5:1; bunun için açık temada ikincil yazı Apple'ın %60 değeri yerine %75 opaklıkta, yeşil, turuncu, kırmızı ve operatör alarm renkleri bir ton koyu (değerler tasarım sisteminde). Odak halkası mavi, 3 px. VoiceOver ölçekleri okur; durum yalnız renkle anlatılmaz; Hareketi Azalt. Native uygulamada sistemin ikincil yazı rengi yerine bu değer, yüksek karşıtlık varyantıyla birlikte asset kataloğunda tanımlanır.
- **Hesap:** Apple veya Google ile giriş (yedek: e-posta bağlantısı); ayrı kayıt formu ve şifre yok. Veri sunucuda olduğu için giriş ilk açılışta istenir.

### Gezinme ve yön bulma

Amaç: kullanıcı her an nerede olduğunu, nasıl geri döneceğini ve yaptığı işin kaydedilip kaydedilmediğini bilir. Apple'ın gezinme ilkeleri (HIG › Navigation) tüm ekranlara uygulanır; örnekler konseptin "Gezinme" sekmesinde.

**Her ekran dört gezinme tipinden birine atanır:**

- **Kök (sekme veya kenar çubuğu):** Tarif listesi, Bugün, Üretim panosu, Fermantasyon, Stok, Takvim, Onaylar, Raporlar, Ayarlar.
- **İleri itme (hiyerarşik):** tarif editörü, su ve pH, mash ve maya, fermantasyon profili, malzeme, karşılaştırma, parti özeti, tank detayı, kalite, CIP ve ambalaj, maya yönetimi, izlenebilirlik, etiket, denetim kaydı, kalibrasyon, cihazlar, kaplar, bildirimler, hesap, giriş yöntemleri, yardım ve destek, mod. Geri düğmesi önceki ekranın adını taşır; soldan kaydırarak geri gidilir.
- **Sheet (kendi içinde biten iş):** parti başlatma, ölçekleme, "nasıl hesaplandı", kendi malzemen, su raporu, tank hareketi, sapma kaydı, mal kabul, içe aktarma, kullanıcı daveti ve değiştirme, Bugün düzenleme, abonelik, destek talebi, e-postayla giriş. Solda Vazgeç, sağda Kaydet veya Bitti.
- **Tam ekran:** demleme günü, tarif sihirbazı, ekipman sihirbazı, ilk açılış, davet kabulü, güncelleme gerekli; duvardaki iPad'de operatör görünümü. Çıkış her zaman sol üstte (Küçült veya Vazgeç).

**Kurallar:**

1. Her ekranın başlığı vardır: iPhone'da kökte büyük başlık, alt ekranda küçük başlık; iPad'de kenar çubuğunda seçili öğe vurgulu.
2. Sheet üstüne sheet açılmaz; sheet içindeki ikinci adım sheet'in kendi içinde ileri itilir.
3. Kaydedilmemiş değişiklik varken Vazgeç veya aşağı kaydırma "Değişiklikler silinsin mi?" diye sorar. Tarif taslağı kendiliğinden kaydedildiği için editörde sorulmaz.
4. Aktif sekmeye ikinci dokunuş o sekmenin köküne döner; her sekme kendi yerini hatırlar.
5. Derin bağlantı (bildirim, QR, arama, widget, Live Activity) ekranı tam geri yolu ile açar; geri düğmesi uygulamadan çıkarmaz, üst ekrana götürür (ör. Fermantasyon › Citra Pale Ale #12 › Uyarı).
6. Süren demleme her yerde görünür: başka ekrana geçince iPhone'da sekme çubuğunun üstünde "Demlemeye dön" şeridi (aktif adım ve kalan süre), iPad'de kenar çubuğunun üstünde. Telefon ve Haritalar'daki süren arama/yol tarifi şeridi gibi.
7. Çok adımlı akışlarda (sihirbazlar, CIP, mal kabul, içe aktarma) "Adım 3/7" göstergesi; Geri önceki adıma, Vazgeç onayla çıkar; yarıda kalan akış taslak olarak saklanır ve listede "Devam et" kartı çıkar.
8. Uygulama yeniden açılınca son ekran geri gelir. Ortak cihazda kullanıcı değişince rolün açılış ekranına gidilir; önceki kullanıcının açık sheet'i kapanır, taslağı saklanır.
9. Mod veya yetki değişince kullanıcı artık göremediği bir ekrandaysa açılış ekranına alınır ve neden olduğu kısa bir notla söylenir.
10. Kaydettikten sonra kullanıcı geldiği ekrana döner ve değişen satır kısa süre vurgulanır.
11. Web: her ekranın kendi adresi var; tarayıcının geri ve ileri düğmeleri çalışır; üç seviyeden derin ekranlarda üstte iz yolu (Üretim › Tanklar › FV2). Apple uygulamalarında iz yolu yok; kenar çubuğu ve geri düğmesi yeter.
12. Operatör görünümü açıkken üstte küçük "Operatör görünümü" etiketi.

**Kaybolma riski olan ekranlar:**

| Ekran | Kullanıcı nasıl kaybolur | Çözüm |
| --- | --- | --- |
| Tarif editörü | Su, mash, malzeme, ölçekleme, "nasıl hesaplandı" sayfaları üst üste açılır | Hepsi tek seviye; malzeme → kendi malzemen sheet içinde ileri itilir; iPad'de Inspector yerinde değişir |
| Tarif sihirbazı | 10 adımda nerede olduğunu ve çıkışı bilmez | Adım göstergesi; her adımda "Editöre geç"; Vazgeç onaylı |
| Demleme günü | Sayaç çalışırken başka ekrana geçer, dönemez | "Demlemeye dön" şeridi; Live Activity aktif adıma açar; tam ekranda Küçült |
| Parti başlatma | Başlattıktan sonra nerede olduğunu bilmez | Başlatınca parti ekranı açılır, üstte "Demlemeyi başlat" düğmesi; geri parti listesine |
| Fermantasyon, Onaylar (bildirimden) | Bildirimden açılır, geri düğmesi yoktur | Tam geri yolu ile açılır |
| Kâğıt ve düzeltme (QR'dan) | QR'dan açılan partide nerede olduğunu bilmez | Başlıkta parti ve tur; geri parti ekranına |
| Üretim panosu → tank detayı → tank hareketi | Üç seviye artı sheet | Tank hareketi sheet; kaydedince tank detayına döner, değişen satır vurgulanır |
| Onaylar | Onaydan sonra seçili öğe kaybolur | Sıradaki bekleyen seçilir; liste boşsa "Hepsi tamam" durumu |
| İzlenebilirlik | Sütun görünümünde derinlik | Üstte yol (Lot › Parti › Ambalaj), her sütun başlıklı |
| Genel arama | Sonuca gidip dönünce arama kaybolur | Dönünce sorgu ve kaydırma yeri korunur |
| Ayarlar | 3–4 seviye (Ekipman › Kaplar › Keg 30 L) | Geri düğmesi önceki başlığı taşır; iPad'de kategori vurgulu; Ayarlar içinde arama |
| CIP, mal kabul, ekipman sihirbazı, içe aktarma | Yarıda bırakılır | Taslak saklanır; "Devam et" kartı |
| Ortak cihaz | Önceki kullanıcının ekranı açık kalır | Kullanıcı değişince rolün açılış ekranı |
| Mod ayrımı | Mod değişince bulunduğu ekran kaybolur | Açılış ekranına geçiş ve not |
| Bugün düzenleme | Düzenleme modunda kalır | "Bitti" belirgin; sekme değişince düzenleme kapanır, değişiklik saklanır |
| Bağlantı yok | Ekran açılmaz | Boş kalmaz; neden ve ne yapılacağı yazar |

## Birimler ve yerelleştirme

Birimler bölge hazır ayarıyla gelir, kategori bazında değiştirilir ve her alanda tek dokunuşla çevrilir; içeride her şey SI saklanır. İlk açılışta bölge cihaz dilinden önerilir, kullanıcı onaylar.

| Kategori | Türkiye ve Avrupa | İngiltere | ABD | Avustralya |
| --- | --- | --- | --- | --- |
| Hacim (ev) | L | L veya imperial galon (4.546 L) | US galon (3.785 L), quart | L |
| Hacim (işletme) | hL | hL, UK brewer's barrel (163.7 L) | US beer barrel, bbl (117.3 L) | hL, L |
| Tahıl | kg | kg (bazı kullanıcılar lb) | lb, oz | kg |
| Hop, tuz, maya | g | g | oz (hop), g (tuz, maya) | g |
| Sıcaklık | °C | °C | °F | °C |
| Yoğunluk | SG veya °Plato | SG (çoğu zaman 1050 biçiminde) | SG | SG |
| Renk | EBC | EBC | SRM, malt rengi °L | EBC |
| Basınç | bar | bar veya psi | psi | kPa veya psi |
| Karbonasyon | g/L veya vol | vol | vol | vol |
| Hop oranı | g/L | g/L | oz/gal, lb/bbl | g/L |
| Su/tahıl oranı | L/kg | L/kg | qt/lb | L/kg |

Tablodaki bölge alışkanlıkları genel bilgiye dayanıyor; beta kullanıcılarıyla doğrulanmalı. İngiltere ve Kanada'da metrik ve İngiliz birimleri karışık kullanılır, bu yüzden kategori bazında değiştirme şart.

- **Üç katman:** bölge hazır ayarı (TR, EU, UK, US, AU) → kategori bazında değişiklik (ör. UK'de tahıl kg, hacim galon) → tek alanda geçici gösterim (dokun, °P ↔ SG).
- **İmperial ve US galon ayrı birimler;** içe aktarmada kaynak dosyanın hangisini kullandığı kontrol edilir (BeerSmith "English" ve "Imperial" ayrı tutar).
- **Hassasiyet birime göre:** SG 3 basamak, °P 1, °F tam sayı, °C 1, g tam sayı, oz 2, kg 2, lb 2 basamak.
- **Yerel biçimler:** ondalık ayırıcı (TR ve AB virgül, UK/US/AU nokta), binlik ayırıcı, tarih ve saat biçimi, haftanın ilk günü, para birimi (TRY, EUR, GBP, USD, AUD). Apple'da Locale ve Measurement API, web'de Intl.
- **Dil:** ilk sürümde Türkçe ve İngilizce, Almanya'ya girişte Almanca; ABD ve İngiltere/Avustralya İngilizcesi ayrımları (color/colour, liter/litre). Yeni dil eklemeye uygun metin kataloğu.
- **Yazıcı çıktıları:** brew sheet ve formlar seçili birimlerle; kağıt boyutu bölgeye göre (US Letter, diğerleri A4).
- **Dosyayla tarif paylaşımı:** tarif SI saklanacağı için alan kişi kendi birimleriyle görür; dışa aktarmada birim seçilir.
- **Mevzuat raporları bölgeye bağlı:** ABD raporları barrel, Türkiye ve AB raporları litre/hL ile; Almanya'da vergi °Plato (Stammwürze) × hL, İngiltere'de litre saf alkol üzerinden. Ülke ayrıntısı "Üst segment ve pazarlar" bölümünde.
- **Stil kılavuzu varsayılanı bölgeyle uyumlu:** BJCP aralıkları SRM ise EBC kullanıcısına çevrilerek gösterilir.

## Eşzamanlı düzenleme ve onay

Klasik check-in/check-out (kayıt kilitleme) kullanılmaz: unutulan kilitler işi durdurur. Kontrol yerine sürüm, onay ve uyarı ile sağlanır; yöntem kayıt türüne göre seçilir.

| Kayıt | Yöntem | Neden |
| --- | --- | --- |
| Tarif | Taslak → yayınlanmış sürüm; düzenleme her zaman taslakta, "yayınla" değişmez yeni sürüm oluşturur | Kontrollü değişiklik kilitsiz; parti hep belirli bir sürüme bağlı |
| Tarif (işletme modu) | Onay akışı: Taslak → İncelemede → Onaylı; yalnız onaylı sürümle üretim; onay yetkisi role bağlı | Onaysız tarifle demleme yapılamaz |
| Aynı taslak iki kişide | Yumuşak kilit: "Ayşe bu tarifi düzenliyor (2 dk önce)" uyarısı, engellemez; çakışmada iki sürüm yan yana, kullanıcı seçer veya birleştirir | Herkes uyarıyı anında görür |
| Demleme günü kaydı | Olay kaydı: her ölçüm, adım, düzeltme ayrı ve zaman damgalı | Birden fazla operatör ve cihaz aynı anda yazar, çakışma olmaz |
| Stok | Hareket defteri (giriş, çıkış, transfer); toplamlar hareketlerden hesaplanır | Çakışma olmaz; eksi stokta uyarı |
| Tank planı | Planlama çakışma kontrolü | Aynı tanka iki parti atanırsa uyarı |
| Profil, ekipman, malzeme kütüphanesi | Alan bazında son yazan kazanır + değişiklik geçmişi | Seyrek değişir, geçmişten geri alınır |
| Ev modu (tek kişi, çok cihaz) | Alan bazında son yazan kazanır + sürüm geçmişi | Tek kullanıcı için kilit gereksiz |

- **Tarifi kilitle:** yanlışlıkla değişikliğe karşı kişisel koruma; ekipler arası kilit değil.
- **Veri modeline etkisi:** `RecipeVersion` durum alanı (taslak / incelemede / onaylı / arşiv), onaylayan ve onay tarihi; işletme modunda `Batch` yalnız onaylı sürüme bağlanır; `EditSession` (kim, hangi taslak, son etkinlik; birkaç dakika etkinlik yoksa düşer); her kayıtta sürüm numarası: kaydederken sürüm değişmişse kayıt reddedilir ve fark gösterilir (iyimser kilitleme).

## Ek özellikler

Profesyonel uygulamalardan alınan ek özellikler aşamalara dağıtıldı; işletme modunun çekirdeği kalite yönetimi ve lota özel hammadde değerleridir. Liste genel bilgiye dayanıyor; rakiplerin güncel özellikleri ayrıca doğrulanacak.

| Özellik | Ne işe yarar | Segment | Aşama |
| --- | --- | --- | --- |
| Geçmiş partilerden ekipman kalibrasyonu | Ölçülen verim, kaynatma kaybı ve kayıplardan profili güncelleme önerisi | İkisi | Ev MVP |
| Kurallar ve otomatik uyarılar | "SG 3 gündür sabit", "sıcaklık 22 °C'yi geçti", "dry hop zamanı" | İkisi | Ev MVP |
| Tarif ve sürüm karşılaştırma | İki tarifi veya sürümü yan yana, malzeme ve tahmin farkları | İkisi | Ev MVP |
| Dekoksiyon aracı | Kaynatılacak mash miktarı ve sıcaklık adımları; formülü var (BeerSmith) | İkisi | Ev MVP |
| Kalori ve karbonhidrat | Bardak başına kalori ve karbonhidrat; formülü var; ABD "Alcohol Facts" kesinleşirse İşletme etiketinde de kullanılır (BeerSmith, Brewer's Friend) | İkisi | Ev MVP |
| Ekstrakt biracısı araçları | Kısmi kaynatma ve üzerine eklenen su; LME, DME ve bal için gram ↔ mL (BeerSmith, Brewer's Friend); formül: 10 §2 | Ev | Ev MVP |
| Parti fotoğrafı ve not | Demleme günü ve tadımda fotoğraf (renk, berraklık, köpük); partinin zaman çizelgesinde (Brewer's Friend) | İkisi | Ev MVP |
| Favoriler ve liste sütunları | Tarifi sabitleme ve favoriye alma; tarif listesinde gösterilecek sütunları seçme (BeerSmith) | İkisi | Ev MVP |
| Şişe ve bardak başına maliyet | Litre maliyetini Kaplar tanımlarıyla şişe ve bardak başına gösterir (Brewer's Friend) | İkisi | Ev MVP (maliyet açıksa); İşletmede parti maliyetiyle |
| Ölçümde kaynak etiketi | Fermantasyon okumalarında kaynak (cihaz / elle / kâğıt); grafikte farklı işaret, kalibrasyon ve tahmin modelinde ayrı ağırlık | İkisi | Ev MVP |
| Elle okuma hatırlatıcısı | Cihazı olmayan partide fermantasyon profiline göre "SG ölçme zamanı" bildirimi (Bildirim yapısı › Hatırlatıcılar) | İkisi | Ev MVP |
| Bildirimden hızlı giriş | Hatırlatıcıya dokununca tek alanlık SG ve sıcaklık girişi | İkisi | Ev MVP |
| Az okumayla fermantasyon tahmini | Elle girilen 2–3 okumada tahminin ne zaman gösterileceği ve güven aralığı; formül: 10 §1 | İkisi | Ev MVP |
| Bildirim kutusu (çan) ve geçmiş | Bütün bildirimler tek listede; onayla, sonra hatırlat, yoksay; olay geçmişi (Bildirim yapısı) | İkisi | Ev MVP |
| Tadım notundan sonraki sürüme | Not, tarifte işaretli taslak değişikliğe dönüşür | İkisi | Ev 2. sürüm |
| Hop yağ profili ve ikame önerisi | Stokta olmayan hop için benzer hop | İkisi | Ev 2. sürüm |
| Geriye planlama | Ambalaj tarihinden demleme, dry hop, soğutma tarihleri | İkisi | Ev MVP |
| Demleme kontrolcülerine tarif gönderme | Grainfather, BrewZilla/RAPT gibi cihazlara program aktarma | Ev | Ev 2. sürüm |
| Laboratuvar su raporu içe aktarma | Rapordan su profili | İkisi | Ev 2. sürüm |
| Şişe ve fıçıda karbonasyon takibi | Kondisyon süresi, hedef CO₂'ye ulaşma | İkisi | Ev 2. sürüm |
| Aynı tarifin partilerini karşılaştırma | Bir tarifin bütün partilerinde OG, FG, verim ve tadım puanı yan yana ve eğilim olarak; İşletme SPC grafiğinin ev sürümü (BeerSmith, Brewer's Friend) | İkisi | Ev 2. sürüm |
| Parti-gyle aracı | Tek mash'ten iki bira: ilk ve ikinci süzmenin yoğunluk ve hacim dağılımı; formülü var (BeerSmith) | İkisi | Ev 2. sürüm |
| Brew sheet'i özelleştirme | Yazdırılan formda hangi bölüm ve alanların olacağını seçme (BeerSmith) | İkisi | Ev 2. sürüm |
| Özel alanlar | Tarif ve partiye kullanıcının kendi alanları (değirmen ayarı, su kaynağı vb.) (BeerSmith) | İkisi | Ev 2. sürüm |
| Yarışma formu çıktısı | BJCP yarışma kayıt bilgisi ve şişe etiketi (BeerSmith) | Ev | Ev 2. sürüm |
| Soğutmasız (no-chill) IBU ayarı | Şıra soğutulmadan bekletildiğinde oluşan ek acılık; Avustralya ve İngiltere'de yaygın (Brewer's Friend, BeerSmith); formül: 10 §3 | İkisi | Ev 2. sürüm |
| Maya hasadı ve jenerasyon takibi | Hasat, jenerasyon, canlılık, hücre sayımı, maya sağlığı uyarısı | İşletme (ileri ev) | İşletme çekirdek |
| Lota özel hammadde değerleri | Hop lotunun gerçek alfa değeri, malt analiz belgesi (COA); tarif lota göre ayarlanır | İşletme | İşletme çekirdek |
| Ürün spesifikasyonu ve tolerans | Bira başına hedef aralıklar; aralık dışı uyarı | İşletme | İşletme çekirdek |
| Ambalaj öncesi serbest bırakma | Değerler ve tadım onayı tamam olmadan ambalaj yok | İşletme | İşletme çekirdek |
| Temizlik (CIP) kaydı | Kimyasal, konsantrasyon, sıcaklık, süre, doğrulama; tankın temiz durumu buradan | İşletme | İşletme çekirdek |
| Verim ve kayıp analizi | Demlemeden ambalaja litre kaybı, brewhouse ve ambalaj verimi | İşletme | İşletme çekirdek |
| Laboratuvar kayıtları | pH, çözünmüş oksijen, CO₂, ABV, mikrobiyoloji | İşletme | İşletme 2. sürüm |
| Tutarlılık grafikleri (SPC) | Partiler arası sapma, kontrol limitleri | İşletme | İşletme 2. sürüm |
| Duyusal değerlendirme | Tadım paneli, tanımlayıcılar, üçgen test | İşletme | İşletme 2. sürüm |
| HACCP kontrol noktaları, çift imza | Kritik adımlarda zorunlu kontrol ve doğrulayan | İşletme | İşletme 2. sürüm |
| Pano ve analizler | Ekipman verimi, maya attenuasyonu, litre başı maliyet, tank kullanımı | İkisi | İşletme 2. sürüm |
| Açık API ve webhook | Tank sensörleri, glikol kontrolcüsü, debimetre (basamaklar ve ilkeler: "Üst segment ve pazarlar") | İşletme | İşletme 2. sürüm |
| Etiket bilgisi üretimi | ABV, lot kodu, son tüketim, alerjen, besin değeri (ülkeye göre zorunluluklar uzmanla netleştirilecek) | İşletme | İşletme 2. sürüm |

## Bildirim yapısı

Bildirimler üç yerden okunur: uygulama içindeki bildirim kutusu (çan), cihaza gelen bildirim (push) ve demleme günü için kilit ekranındaki Live Activity. Öncelik seviyeleri, sessiz saatler, vardiya ve yükseltme kuralları "Üst segment ve pazarlar" bölümünde ve Ayarlar › Bildirimler'de.

### Kanallar

- **Bildirim kutusu (çan):** Her bildirim burada da durur; push kapalı olsa bile kaybolmaz.
- **Push:** iPhone ve iPad'e Apple bildirimi; web'de tarayıcı bildirimi (kullanıcı izin verirse). Apple Watch ileride opsiyonel.
- **E-posta:** Günlük özet (kullanıcı açarsa) ve işlem e-postaları: giriş bağlantısı, davet, destek yanıtı (e-postada yanıt metni yok; yalnız "yanıtınız var" ve uygulamaya bağlantı). Uyarılar e-postayla gitmez.
- **Tek durum:** Bildirim sunucuda tek kayıttır; bir cihazda onaylanan, ertelenen veya yoksayılan bildirim diğer cihazlarda da kapanır ve push geri çekilir.

### Bildirim kutusu (çan)

- **Yeri:** iPhone'da sekme köklerinin sağ üstünde çan, okunmamış sayısı rozetle; iPad ve web'de kenar çubuğunda "Bildirimler" satırı ve üst çubukta çan.
- **Liste:** En yeni üstte; her satırda öncelik işareti (1 Acil, 2 Uyarı, 3 Bilgi), parti ve zaman. Filtreler: Tümü, Okunmamış, Acil, Hatırlatıcılar, Onay bekleyenler.
- **Açınca:** İlgili parti ve adım, geri yolu hazır olarak açılır (Gezinme kuralı 5).
- **Sekmeler:** "Bildirimler" (açık olanlar) ve "Geçmiş".

### Bildirime ne yapılabilir

| Eylem | Ne olur | Kısıt |
| --- | --- | --- |
| Aç | İlgili ekrana gider, bildirim okundu olur | – |
| Onayla / Tamam | Bildirim kapanır; kim ve ne zaman kaydedilir | İşletmede acil uyarıda yükseltmeyi durdurur |
| Sonra hatırlat | 15 dk, 1 saat, yarın sabah veya seçilen saatte yeniden gelir | Acil uyarı en fazla 30 dk ertelenir ve yükseltme süresi işlemeye devam eder |
| Yoksay | Kutudan kalkar, geçmişte kalır | Acil uyarı yoksayılamaz, onay ister; "bu partide bu kuralı sustur" seçeneği ayrıca sorulur |
| Kendiliğinden çözülme | Koşul düzelince (sıcaklık hedefe döndü, okuma girildi) "çözüldü" olarak kapanır, push geri çekilir | – |

### Hatırlatıcılar

- Parti başlatılınca fermantasyon profiline göre kurulur: elle okuma (cihazı olmayan partilerde), dry hop, cold crash, ambalaj. İlgili adım tamamlanınca kendiliğinden kalkar.
- Bildirimden hızlı giriş: hatırlatıcıya dokununca uygulamayı gezmeden tek alanlık SG ve sıcaklık girişi açılır.

### Bildirim geçmişi

- Her bildirimin olayları kaydedilir: oluştu, hangi cihazlara gitti, okundu, onaylandı, ertelendi, yoksayıldı, çözüldü; kim ve ne zaman.
- Görüldüğü yerler: bildirim kutusunun "Geçmiş" sekmesi, partinin zaman çizelgesi; İşletmede onay ve yoksaymalar denetim kaydında da.
- Saklama (öneri): Ev'de 90 gün; İşletmede kayıt saklama süresi boyunca.

### Mod farkı

- **Ev:** Çan, eylemler, hatırlatıcılar ve geçmiş.
- **İşletme:** Ek olarak vardiyaya yönlendirme, yükseltme ve role göre dağıtım. Ortak cihazda bildirim o anki kullanıcıya değil vardiyaya düşer; onaylayan kişi kayda yazılır.

## Üst segment ve pazarlar

ABD'deki butik bira üreticilerinin kullandığı yazılım ve cihazlar ile hedef ülkelerin mevzuatı araştırıldı (Kasım 2026). Ürün özellikleri büyük ölçüde üreticilerin kendi tanıtımlarından, vergi rakamları haber ve resmi kaynaklardan alındı; hepsi resmi kaynaktan tekrar doğrulanacak. Büyüklerin üstünlüğü daha fazla özellikten değil, dört şeyden geliyor: veri cihazlardan kendiliğinden gelir, uyarı doğru kişiye gider, herkes yalnız kendi işini görür, üretim öncesi ve sonrası kayıtlıdır. Aşağıda yalnız plana yeni eklenenler var. Zaten planda olanlar tekrar edilmedi: roller ve yetki matrisi, onay kutusu, lota özel hammadde, maya jenerasyonu, CIP kaydı, kalite ve serbest bırakma, laboratuvar, duyusal değerlendirme.

### Eklenenler ve kullanım yeri

| Özellik | Ne işe yarar | Kullanım yeri | Segment | Aşama |
| --- | --- | --- | --- | --- |
| Düzenlenebilir Pano ve Bugün | Kart ekle, çıkar, sırala; role göre hazır düzen; kayıtlı görünümler (ör. "yalnız aktif fermantasyonlar") | Üretim panosu ve Bugün'de "Düzenle"; web'de aynı | İşletme | İşletme çekirdek |
| Tank widget'ı | Tank sıcaklığı, SG ve sıradaki iş ana ve kilit ekranında | iOS ana ve kilit ekranı | İkisi | Fermantasyon widget'ı Ev MVP; tank widget'ı İşletme çekirdek |
| Beklenen eğriye göre uyarı | Sabit eşik yerine önceki iyi partinin veya fermantasyon tahmininin eğrisinden sapmayı yakalar | Otomatik uyarı kuralları; Fermantasyon | İkisi | Ev MVP (analitik model 2 ile aynı; onay bekliyor) |
| Bildirim öncelik seviyeleri | 3 seviye, operatör görünümündeki alarm seviyeleriyle aynı; en yüksek seviye Apple "Time Sensitive" bildirimi | Ayarlar › Bildirimler | İkisi | Ev MVP |
| Alarm yorgunluğunu önleme | Aynı olayın uyarıları birleşir; sessiz saatler; kritik olmayanlar günlük özette | Ayarlar › Bildirimler | İkisi | Ev MVP |
| Vardiyaya yönlendirme ve yükseltme | Uyarı o anki vardiyaya gider; belirlenen sürede onaylanmazsa bir üst kişiye geçer; bildirimden tek dokunuşla onay | Ayarlar › Bildirimler | İşletme | İşletme çekirdek |
| Vardiya devri | Açık uyarılar ve notlar sonraki vardiyanın Bugün ekranında kart olarak çıkar | Bugün | İşletme | İşletme çekirdek |
| Süreli davet | Danışman veya muhasebeci için bitiş tarihli erişim | Kullanıcı davet akışı; Ayarlar › Kullanıcılar | İşletme | İşletme çekirdek |
| Kişi × yetki tablosu | Rol matrisine ek olarak kimin neye yetkili olduğu kişi bazında tek tabloda | Ayarlar › Roller ve yetkiler | İşletme | İşletme çekirdek |
| Temiz olmayan tank uyarısı | CIP kaydı olmayan tanka parti başlatılırken veya transfer yapılırken uyarı | Parti başlatma; Tank hareketi | İşletme | İşletme çekirdek |
| Geri çağırma raporu | Lot → hangi partiler ve ambalaj kayıtları (kap türü, adet, tarih); rapor ambalajda biter, müşteri bilgisi sistemde yok | Lot izlenebilirlik raporu | İşletme | İşletme çekirdek |
| Tank boşalma önerisi | Hangi tankın ne zaman boşalacağı, yeni partiye uygun tank | Takvim ve geriye planlama; Üretim panosu | İşletme | İşletme 2. sürüm |
| Hobi cihazları için veri alma adresi (webhook) | Tilt köprüleri (TiltBridge, Pitch), iSpindel ve RAPT veriyi bu adrese gönderir | Cihaz eşleştirme; Ayarlar › Cihazlar | İkisi | Ev MVP |
| Profesyonel tank sensörleri | SG, pH, çözünmüş oksijen, basınç; API veya CSV ile | Cihaz eşleştirme; Tank detayı | İşletme | İşletme 2. sürüm |
| Endüstriyel ağ geçidi | Glikol ve tank kontrolcüleri, PLC: Modbus TCP, OPC UA, MQTT | Ayarlar › Cihazlar; Tank detayı; Operatör görünümü | İşletme | Okuma İşletme 2. sürüm; komut gönderme Sonra |
| Sürdürülebilirlik göstergeleri | hL başına su, enerji ve CO₂ tüketimi ve trendi | Raporlar, panolar | İşletme | İşletme 2. sürüm |
| Depozito ve ambalaj raporu | Kap sayısı, malzeme, ağırlık (İngiltere depozito sistemi ve ambalaj atığı raporu) | Ambalaj kaydı; Mevzuat raporu | İşletme | İşletme 2. sürüm |

Kapsam dışı: distribütör satış verisi (VIP iDIG), CRM ve muhasebe. Bunlar ERP alanında kalıyor.

**Not: vergi raporu ve satış verisi.** Ekos, Ollie, Beer30 ve Breww ile karşılaştırmadan sonra "ERP yok" kararı korundu: satış, sevkiyat, fatura ve muhasebe uygulamada olmayacak. Bunun sonucu olarak Mevzuat raporunda (ABD TTB, İngiltere HMRC) satış ve sevkiyat miktarları bizim sistemde doğmaz. Bu miktarlar elle girilir veya başka bir programdan dosya olarak alınır; rapor yalnız üretim ve ambalaj tarafını kendiliğinden doldurur. Rakipler bu raporu satış ve sevkiyat kayıtlarından otomatik dolduruyor; bu bilinen bir fark.

### Cihaz entegrasyonu ilkeleri

- Her cihaz ayrı adaptörle bağlanır; tek üreticiye bağımlı olunmaz (Plaato üretimden kalktı, Brewfather desteği kaldırdı).
- Önce yalnız okuma yapılır. Cihaza komut göndermek (ör. soğutmayı başlat) sonraki bir sürümde ve onayla gelir.
- Sensörün bağlantısı koparsa okumalar sensörde veya ağ geçidinde biriktirilir, sonra sunucuya aktarılır (uygulamanın çevrimdışı çalışmasıyla ilgisi yok).
- Her cihazın kendi erişim anahtarı olur (Güvenlik bölümündeki cihaz anahtarları).
- Her sensör, Kalibrasyon ekranındaki kaydına bağlanır.

### Ülkelere göre yerelleştirme

| Ülke | Vergi esası | Küçük üretici indirimi | Beyan | Etiket ve ambalaj | Uygulamada nerede |
| --- | --- | --- | --- | --- | --- |
| ABD | Barrel başına federal vergi | CBMA indirimli oranı (kalıcı) | TTB 5130.9 aylık; yıllık vergi $50.000 veya altındaysa 5130.26 üç aylık; dönemi izleyen ayın 15'ine kadar | Alcohol Facts ve alerjen kuralları önerildi, henüz kesinleşmedi (kesinleşirse 5 yıl geçiş süresi) | Mevzuat raporu (barrel); Etiket bilgisi |
| İngiltere | Litre saf alkol, ABV bandına göre (3.5–8.5% arası £22.58/L, 1 Şubat 2026'dan) | Small Producer Relief: yılda 4.500 hl saf alkol altı, HMRC hesaplayıcısıyla; 28 Ekim 2026 bütçesiyle genişliyor | Alcohol Duty beyannamesi | 20 L ve üstü fıçıda düşük oran; depozito sistemi 1 Ekim 2027'de başlıyor (üretici kaydı, aylık hacim raporu); ambalaj atığı (EPR) raporu | Mevzuat raporu; Ambalaj kaydında kap türü ve hacmi; SPR uygulamada yalnız tahmin olarak |
| Avustralya | Excise | Yıllık $400.000'a kadar %100 iade (1 Temmuz 2026'dan; önceden $350.000); Ağustos 2025–Şubat 2027 arası enflasyon artışı yok | Excise beyannamesi; Eylül 2026'dan yeni kayıtlarda daha sıkı denetim | Hamilelik uyarısı ve eyalet depozito sistemleri (doğrulanacak) | Mevzuat raporu; Pano'da iade sınırı takibi |
| Almanya / AB | °Plato (Stammwürze) × hL | Mengenstaffel: yılda 200.000 hl altı, kademeli (kademeler doğrulanacak) | Aylık Biersteuer beyanı | AB'de bira muaf ama sektörün %95'i içindekileri, %91'i enerjiyi yazıyor; Almanya'da içindekiler listesi zorunlu; tek kullanımlık ambalajda depozito (doğrulanacak) | Parti başlatma ve Demleme gününde OG °P olarak zorunlu alan; Mevzuat raporu; Etiket bilgisi |
| Türkiye | ÖTV (Ocak 2026: 12,49 TL; birim ve Temmuz tutarı doğrulanacak) | — | ÖTV beyanı, TAPDK | Uzmanla netleştirilecek | Mevzuat raporu; Etiket bilgisi |

**Stammwürze ve Plato:** Plato (°P), şırada çözünmüş ekstraktın ağırlıkça yüzdesidir (12 °P ≈ SG 1.048). Stammwürze, fermantasyon öncesi başlangıç yoğunluğudur; OG'nin Plato cinsinden karşılığı. Almanya'da vergi = hL × °P × derece başına tutar (yaklaşık 0,787 €/hL/°P, tam dereceye yuvarlanır; doğrulanacak). Bu yüzden Almanya'da her partinin OG değeri °P olarak kaydedilir. Hesap motorunda `hl_plato` ve `plato_bandi` yöntemleri bunun için var (formuller/09).

- **Vergi ülkesi seçimi:** Ayarlar › Bölge ve birimler'de yapılır (İşletme). Hesap motorundaki vergi yöntemini, Mevzuat raporunun biçimini ve Etiket bilgisinin zorunlu alanlarını belirler.
- **Hesap motoruna eklenecekler:** "Eksik hesaplar" tablosunda: İngiltere'de ABV bandı + fıçı indirimi + SPR tahmini, Avustralya'da iade sınırı, depozito ve ambalaj raporu.
- **Dil:** İlk sürümde TR ve EN (US/UK/AU); Almanya'ya girişte DE.
- **Veri koruma:** KVKK, GDPR, UK GDPR; ABD eyalet yasaları ve Avustralya Privacy Act (doğrulanacak). Güvenlik bölümüyle birlikte ele alınır.

**Açık kararlar:**

- İlk İşletme pazarı hangisi olacak? Vergi raporu önce o ülke için yazılır.
- Apple "Critical Alert" izni için başvuru (telefon sessizdeyken çalan alarm).
- Endüstriyel ağ geçidi: kendi donanımımız mı, hazır bir ağ geçidi mi?

Kaynaklar: [Breww yetkiler](https://breww.com/docs/user-groups-and-permissions/), [BrewMonitor 2.0](https://www.craftbrewingbusiness.com/news/precision-fermentation-launches-brewmonitor-system-2-0-upgrading-its-flagship-iot-driven-fermentation-monitoring-solution/), [Brewfather entegrasyonlar](https://docs.brewfather.app/integrations), [TTB beyan tarihleri](https://www.ttb.gov/taxes/tax-audit/due-dates-for-operational-reports), [TTB etiket önerisi](https://www.ttb.gov/public-information/featured-stories/ttb-proposes-mandatory-disclosures-major-food-allergens-and), [UK alcohol duty](https://www.legislation.gov.uk/ukpga/2026/11/part/3/crossheading/alcohol-duty/2026-08-18/data.html), [UK bütçe 2026](https://www.morningadvertiser.co.uk/Article/2026/10/28/healey-announces-10m-alcohol-duty-relief-for-small-brewers/), [UK depozito](https://gov.uk/guidance/deposit-return-scheme-drinks-producer-and-retailer-responsibilities), [ATO iade](https://www.ato.gov.au/businesses-and-organisations/gst-excise-and-indirect-taxes/excise-on-alcohol/excise-remissions-for-excisable-alcohol/remission-scheme-for-alcohol-manufacturers), [Destatis Biersteuer](https://www.destatis.de/DE/Methoden/Qualitaet/Qualitaetsberichte/Steuern/biersteuer.pdf?__blob=publicationFile), [Brewers of Europe 2026](https://transition-pathways.europa.eu/agri-food/coc-reports/brewers-europe-report-eu-code-conduct-2026), [AA ÖTV 2026](https://www.aa.com.tr/tr/ekonomi/akaryakit-sigara-ve-alkollu-iceceklerde-otv-artisi-sinirlandirildi/3786557).

## Kullanıcı şikayetlerinden gelen kararlar

Rakip uygulamaların (BeerSmith, Brewfather, Brewer's Friend, Grainfather, Ekos, Ollie, Breww) kullanıcı şikayetleri araştırıldı (forum, uygulama mağazası ve sektör yazıları; sistematik anket değil). Şikayetlerin çoğunu mevcut tasarım karşılıyor; aşağıdaki üç karar bu araştırmadan geldi.

### Deneme süresi

- **1 ay tam deneme:** deneme süresince tüm özellikler açık; hesaplar kısıtlanmaz (BeerSmith denemesinde hesapların güncellenmemesi şikayet konusu).
- Deneme bitince veriler silinmez ve her zaman dışa aktarılabilir; ücretli özellikler salt okunur kalır.
- Fiyatlar ve ücretsiz katmanın sınırı hâlâ açık karar. Karar verilirken katmanlara göre özellik matrisi yazılır: tarif, parti, cihaz, kullanıcı ve depolama sınırları, destek yanıt süresi. Hiçbir katmanda dışa aktarma kapatılmaz.

### Kalibrasyon asistanı

En sık şikayet "tahmin edilen OG ve hacim tutmuyor" ve ekipman profilini doğru girmenin zorluğu.

1. Her partiden sonra ölçülen hacimler (pre-boil, post-boil, fermentör) ve graviteler (ilk şıra, pre-boil, OG) ile gerçek verim ve kayıplar hesaplanır.
2. Fark, ekipman profilindeki her parametre için ayrı izlenir: mash verimi, buharlaşma, tahıl emilimi, kazan ölü hacmi, trub kaybı, soğuma büzülmesi.
3. Aynı yönde tutarlı fark **3 partide** görülünce "ekipman profilini güncelle" önerisi çıkar; eski ve yeni değer ile bunun OG ve hacim tahminine etkisi yan yana gösterilir. Kullanıcı onaylamadan profil değişmez.
4. Tek partilik sapmalar (ölçüm hatası, unutulan adım) öneriye katılmaz; kullanıcı bir partiyi kalibrasyon dışı işaretleyebilir.
5. İşletmede aynı hesap brewhouse verimi, dönüşüm ve süzme verimi ayrımıyla yapılır (`formuller/08`, §4).

Ekranlar: parti özetinde "bu partiden öğrenilen" kartı; Ayarlar › Ekipman ve tanklar'da öneri listesi.

### Sürüm kuralı

Tarif sürümleri kendiliğinden çoğalmamalı (Grainfather'da birikmiş sürüm kopyaları şikayet konusu).

- Tarifteki düzenlemeler taslakta **otomatik kaydedilir**; her kayıt yeni sürüm sayılmaz.
- Yeni sürüm yalnız üç durumda oluşur: bir parti başlatıldığında (parti o sürüme bağlanır), işletmede onaya gönderildiğinde, ya da kullanıcı "yayınla" dediğinde.
- Parti başlatırken tarif seçiminde yalnız son sürüm (işletmede son onaylı sürüm) önerilir; eski sürümler "sürüm geçmişi"nde.
- Hiç partisi olmayan ve 90 gün dokunulmamış ara sürümler arşive önerilir (silinmez).

## Kullanıcılar, roller ve yetkiler

İşletme modu çoklu kullanıcılıdır: her kullanıcının bir veya birden fazla rolü vardır, yetkiler rolden gelir ve arayüz role göre değişir. Roller ve yetkiler hazır gelir; yönetici ayarlardan düzenler, kopyalar veya yeni rol oluşturur.

### Hazır roller

| Rol | Tipik işi | Açılış ekranı |
| --- | --- | --- |
| Sahip / Yönetici | Kullanıcılar, roller, ayarlar, abonelik; her şey | Pano |
| Brewmaster | Tarif yazar ve onaylar, üretim planı, kalite kararı | Üretim panosu |
| Biracı / Operatör | Demleme günü, ölçüm, transfer, CIP kaydı | Bugün (görev listesi) |
| Kalite (QA) | Spesifikasyon, laboratuvar, tadım, serbest bırakma | Kalite kuyruğu |
| Depo | Mal kabul, lot, sayım, stok hareketi | Stok |
| İzleyici / Denetçi | Salt okunur; raporlar ve denetim kaydı | Pano |

### Yetki matrisi (varsayılan)

| Yetki | Yönetici | Brewmaster | Operatör | Kalite | Depo | İzleyici |
| --- | --- | --- | --- | --- | --- | --- |
| Tarif görüntüle | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Tarif düzenle (taslak) | ✓ | ✓ | – | – | – | – |
| Tarif onayla | ✓ | ✓ | – | – | – | – |
| Parti planla, tank ata | ✓ | ✓ | – | – | – | – |
| Demleme kaydı gir, adım kapat | ✓ | ✓ | ✓ | – | – | – |
| Kapanmış partide düzeltme | ✓ | ✓ | – | ✓ | – | – |
| Kalite ölçümü ve tadım | ✓ | ✓ | ✓ | ✓ | – | – |
| Ambalaj için serbest bırak | ✓ | – | – | ✓ | – | – |
| Stok hareketi ve sayım | ✓ | ✓ | – | – | ✓ | – |
| Maliyet ve fiyatları gör | ✓ | ✓ | – | – | ✓ | – |
| Raporlar ve dışa aktarma | ✓ | ✓ | – | ✓ | ✓ | ✓ |
| Ekipman ve profil ayarları | ✓ | ✓ | – | – | – | – |
| Kullanıcı, rol, yetki yönetimi | ✓ | – | – | – | – | – |

### Ayarlarda yönetim

- **Roller düzenlenebilir:** hazır rol kopyalanıp değiştirilir veya sıfırdan rol oluşturulur; yetkiler gruplar halinde (tarif, parti, kalite, stok, maliyet, ayarlar) açılıp kapatılır.
- **Kullanıcı yönetimi:** davet (e-posta), rol atama, geçici devre dışı bırakma, çıkarma; bir kullanıcıya birden fazla rol.
- **Kapsam (isteğe bağlı):** yetki belirli tank veya üretim alanıyla sınırlanabilir.
- **Koruma:** en az bir yönetici kalır; bir kullanıcı kendi yönetici yetkisini kaldıramaz; rol ve yetki değişiklikleri denetim kaydına yazılır.

### Role göre arayüz

- **Açılış ekranı ve sekmeler** role göre: operatöre "Bugün", brewmaster'a üretim panosu, kaliteye kalite kuyruğu.
- **Yetkisi olmayan işlem gizlenir veya salt okunur görünür;** maliyet alanları yetkisi olmayana hiç gösterilmez.
- **Onay gereken işlemler** yetkili kişiye bildirim olarak düşer (tarif onayı, serbest bırakma).

### Ortak cihaz modu

Üretim alanındaki tek iPad'i birden fazla kişi kullanır: hızlı kullanıcı değiştirme (PIN veya Face ID), her işlem o anki kullanıcıya yazılır, belirli süre işlem olmazsa ekran kilitlenir. Yetkiler sunucuda denetlenir. Bağlantı yoksa yalnız indirilmiş demleme paketiyle sayaç ve ölçüm girişi yapılır; ölçümler o anki kullanıcıya yazılıp kuyruğa alınır.

### Ev modunda

Tek kullanıcı; isteğe bağlı olarak bir arkadaşı tarif veya partiye düzenleyici ya da izleyici olarak davet etme. Roller ve yetki matrisi görünmez.

**Veri modeli:** `User`, `Role`, `Permission`, `RoleAssignment` (kullanıcı, rol, isteğe bağlı kapsam), `Invitation`, `DeviceSession` (ortak cihazda aktif kullanıcı). İşletme fiyatlandırması kullanıcı sayısına göre yapılabilir.

## Güvenlik

Güvenlik her katmanda ayrı ele alınır. Öncelik sırası: (1) yetkinin sunucuda denetlenmesi, (2) çalışma alanlarının ayrık tutulması, (3) değiştirilemeyen denetim kaydı, (4) ortak cihazda kişiye özel giriş.

| Katman | Korunan | Güvenlik özellikleri |
| --- | --- | --- |
| Cihaz (iPad, iPhone) | Cihazdaki veri ve belgeler | iOS Data Protection ile şifreli saklama; oturum anahtarları Keychain'de; işletmede Face ID veya PIN ile uygulama kilidi; ortak iPad'de kişiye özel PIN ve işlemsizlikte kilit; uygulama değiştiricide hassas alanlar gizli |
| Web | Tarayıcı oturumu | HttpOnly, Secure, SameSite çerez; CSP; CSRF koruması; paylaşılan bilgisayar için kısa oturum ve "her yerden çıkış" |
| Kimlik doğrulama | Kimin girdiği | Apple ve Google ile giriş, yedek olarak e-posta bağlantısı; şifre yok; yöneticilere passkey ile ikinci adım zorunlu; tek kullanımlık, 7 gün geçerli davet; cihaz oturumunu uzaktan kapatma |
| Yetkilendirme | Kimin neyi görüp değiştirebildiği | Roller ve kapsam sunucuda denetlenir; her istek çalışma alanı ve kapsamla filtrelenir; maliyet gibi alanlar yetkisize hiç gönderilmez |
| API sunucusu | Bulut verisi | TLS; çalışma alanları ayrık (satır düzeyinde güvenlik); hız sınırı; çakışmada sessiz üzerine yazma yok |
| Veri saklama | Veritabanı, dosyalar, yedekler | Şifreli disk ve yedek; belgeler ayrı depoda, kısa süreli imzalı bağlantıyla; yedek başka bölgede, düzenli geri yükleme testi |
| Denetim kaydı | Kim neyi değiştirdi | Yalnız eklenebilir; her kayıt öncekinin özetini (hash) taşır, silme fark edilir; kapanmış partide düzeltme yetki ve gerekçeyle |
| Cihaz entegrasyonları | Tilt, iSpindel, RAPT verisi | Cihaza özel, yenilenebilir anahtarlı veri adresi; aralık dışı değerler reddedilir; Bluetooth cihazları yalnız okunur |
| Dosya içe ve dışa aktarma | BeerXML, BeerJSON, .bsmx, CSV | Güvenli XML ayrıştırıcı (dış varlık ve XML bombasına karşı); boyut sınırı; CSV enjeksiyonuna karşı hücre kaçışı |
| Hesap motoru (Rust) | Hesap sonuçları | Bellek güvenli dil; girdi aralık kontrolü, sıfıra bölme ve sonsuz değer kontrolü; hesaplar yalnız sunucuda çalışır, istemci hesap sonucu gönderemez; test vektörleri her sürümde sunucuda çalıştırılır |
| Ödeme ve abonelik | Ödeme bilgisi | StoreKit ve web ödeme sağlayıcısı; kart bilgisi bize gelmez; abonelik durumu sunucuda doğrulanır |
| Kişisel veri ve uyum | UK GDPR, ABD eyalet gizlilik yasaları, KVKK | En az veri; veri kullanıcınındır ve tamamı dışa aktarılabilir (tarif BeerJSON ve BeerXML; parti, ölçüm ve stok CSV; ekipman, su profilleri, kendi malzemeler, cihaz kayıtları ve destek talepleri JSON; belgeler ve fotoğraflar ZIP); hesap silme; saklanması zorunlu işletme kayıtlarında anonimleştirme; gizlilik politikası ve App Store gizlilik etiketi |
| Geliştirme ve işletim | Kod, sırlar, sunucular | Sırlar sır yöneticisinde; bağımlılık taraması; sunucuya en az yetki ve iki adımlı doğrulama; olay alarmları ve müdahale planı |
| Destek paneli | Müşteri verisine içeriden erişim | Genel adreste yok, OIDC ters vekili ve passkey; müşteri izni ve süreli erişim; içerik değiştirilemez, müşteri yerine giriş yok; her erişim müşterinin denetim kaydında görünür |
| Destek talepleri (GitHub) | Issue'lardaki müşteri metni | Özel depo, erişim yalnız destek ekibine; GitHub organizasyonunda iki adımlı doğrulama zorunlu; GitHub App yalnız bu depoya, anahtarı sır yöneticisinde; /yanit yalnız destek ekibi listesindeki kişilerden kabul edilir; uygulamanın kendi yorumları webhook'ta yok sayılır; müşteriye gitmiş yorum düzenlenirse müşteride de "düzenlendi" notuyla güncellenir, silinirse geri çekilir; Issue işlemleri de SupportAction olarak denetim kaydına yazılır |

**Açık kararlar:**

- Ev modunda hesapsız kullanım yok: veri ve hesaplar sunucuda olduğu için ilk açılışta giriş gerekir (Apple veya Google ile tek dokunuş, ayrı kayıt yok). Bunun ilk açılışa etkisi kullanıcı testinde ölçülecek.
- Sunucu bölgesi: ABD ve İngiltere bölgeleri öneriliyor; çalışma alanı açılışta bölgesini seçer, AB sonra (Platformlar ve mimari › Veri katmanı). Hukuki ve maliyet kararı.

Ayarlar'a "Güvenlik" kategorisi eklenir: iki adımlı doğrulama, oturum açık cihazlar, uygulama kilidi, cihaz (iSpindel) anahtarlarını yenileme, destek erişimi izni ve geçmişi.

## Ayarlar

Ayarlar Apple'ın Ayarlar uygulaması düzenini izler: iPad ve web'de solda renkli ikonlu kategoriler, sağda gruplu ayrıntı; iPhone'da gruplu liste. Ev modunda işletmeye özgü kategoriler gizlenir. Taslak: [Demleme Arayüz Konsepti](https://claude.ai/artifact/BCEHj3wb6xj7eShvjTxKn6), Ayarlar sekmesi.

| Kategori | İçerik | Segment |
| --- | --- | --- |
| Çalışma alanı | Ad, mod (Ev/İşletme), açılış ekranı, Pano ve Bugün kart düzeni (ekranda "Düzenle" ile de) | İkisi |
| Görünüm | Tema: Sistem / Açık / Koyu (cihaz başına); demleme gününde her zaman koyu (varsayılan kapalı); operatör görünümü (üretim ekranlarında normal durum gri, renk yalnız alarmda); uzaktan okunur büyük sayılar; ekran kapanmasın. Yazı boyutu ve karşıtlık iOS ayarlarından gelir | İkisi |
| Bölge ve birimler | Bölge hazır ayarı, vergi ve mevzuat ülkesi (İşletme), dil, sayı biçimi, para birimi, kağıt boyutu; kategori bazında birimler; "hazır ayara dön" | İkisi |
| Ekipman ve tanklar | Brewhouse profilleri, kayıplar, verim; geçmiş partilerden kalibrasyon önerisi; tanklar ve temizlik durumu; kaplar (şişe, kutu, keg, cask) | İkisi (tanklar işletmede) |
| Hesaplama | IBU ve geç hop modeli, rakım, ABV formülü, mash sıcaklığına göre FG, maya toleransı, steep verimi, renk, pH modeli, varsayılan asit | İkisi (evde sade) |
| Stil kılavuzları | Varsayılan kılavuz, yüklü kılavuzlar, kullanıcı kılavuzu | İkisi |
| Kullanıcılar | Davet (süreli davet dahil), rol atama, devre dışı bırakma, ortak cihaz modu, kişi × yetki tablosu | İşletme |
| Roller ve yetkiler | Rol listesi, yeni rol, rol kopyalama; açılış ekranı, kapsam; yetki grupları anahtarlarla; düzenleme yalnız iPad ve web | İşletme |
| Güvenlik | Giriş yöntemleri (Apple, Google, e-posta) ve passkey, oturum açık cihazlar ve uzaktan çıkış, destek erişimi izni ve geçmişi, uygulama kilidi (Face ID, PIN, işlemsizlik süresi), cihaz anahtarlarını yenileme | İkisi (2FA zorunluluğu ve kilit işletmede) |
| Bildirimler | Demleme alarmları, fermantasyon kuralları, onay bekleyenler; 3 öncelik seviyesi, sessiz saatler, günlük özet; vardiyaya yönlendirme ve yükseltme (İşletme) | İkisi |
| Cihazlar | Tilt, iSpindel, RAPT; veri alma adresi (webhook); tank eşleştirme; cihaz anahtarları; profesyonel sensörler ve endüstriyel ağ geçidi (Modbus TCP, OPC UA, MQTT; İşletme 2. sürüm) | İkisi |
| Yazdırma ve formlar | Brew sheet, tank turu, "her partide tarama iste" | İkisi (tarama zorunluluğu işletmede) |
| Veri ve yedekleme | İçe ve dışa aktarma, yedek | İkisi |
| Abonelik | Plan, kullanıcı sayısı, yenileme | İkisi |
| Yardım ve destek | Yardım arama, destek talebi açma, taleplerim, sistem durumu | İkisi |

## Ekran listesi

Konseptte 62 ekranın hepsi taslaklandı (Ev MVP aşamasında 42, İşletme aşamalarında 19, Apple Watch ekranı ileride opsiyonel; bu 19 ekran yalnız İşletme modunda; ekranların hangi modda açık olduğu "Mod ayrımı" tablosunda); konseptte sekmeler Tarif, Demleme, Üretim ve Sistem gruplarında; destek paneli ayrıca İç kullanım grubunda (sayıya girmez). Stok ekranı işletme modunda lot bazında çalışır (rezerve, kullanılabilir, FIFO, COA, lota göre tarif düzeltmesi); ev modunda basit miktar ve minimum seviye.

| Ekran | Durum | Segment | Gerektiği aşama |
| --- | --- | --- | --- |
| Tarif sihirbazı | Tasarlandı | İkisi | Ev MVP |
| Tarif editörü | Tasarlandı | İkisi | Ev MVP |
| Kaynatma programı | Tasarlandı | İkisi | Ev MVP |
| Demleme günü, Live Activity | Tasarlandı | İkisi | Ev MVP |
| Kağıt, aktarma ve düzeltme | Tasarlandı | İkisi | Ev MVP |
| Fermantasyon | Tasarlandı | İkisi | Ev MVP |
| Stok ve alışveriş listesi | Tasarlandı | İkisi | Ev MVP (lotlar işletmede) |
| Ayarlar | Tasarlandı | İkisi | Ev MVP |
| Üretim panosu | Tasarlandı | İşletme | İşletme çekirdek |
| İlk açılış: mod, bölge, ekipman sihirbazı, giriş | Tasarlandı | İkisi | Ev MVP |
| Tarif ve parti listesi: arama, filtre, klasör, etiket | Tasarlandı | İkisi | Ev MVP |
| Su ve mash pH: profil, tuzlar, asit, eşleştirme | Tasarlandı | İkisi | Ev MVP |
| Mash adımları ve maya starter | Tasarlandı | İkisi | Ev MVP |
| İçe aktarma eşleştirme | Tasarlandı | İkisi | Ev MVP |
| Parti özeti: plan–gerçekleşen raporu, tadım notu | Tasarlandı | İkisi | Ev MVP |
| Araçlar: refraktometre, karbonasyon, seyreltme, birim çevirici | Tasarlandı | İkisi | Ev MVP |
| Malzeme kütüphanesi detayı | Tasarlandı | İkisi | Ev MVP |
| Tarif ve sürüm karşılaştırma | Tasarlandı | İkisi | Ev MVP |
| Bugün (operatör açılış ekranı) | Tasarlandı | İşletme | İşletme çekirdek |
| Onay kutusu (tarif onayı, serbest bırakma) | Tasarlandı | İşletme | İşletme çekirdek |
| Kalite: spesifikasyon, laboratuvar, serbest bırakma | Tasarlandı | İşletme | İşletme çekirdek |
| CIP kaydı ve ambalaj kaydı | Tasarlandı | İşletme | İşletme çekirdek |
| Maya yönetimi (hasat, jenerasyon) | Tasarlandı | İşletme | İşletme çekirdek |
| Raporlar, panolar, mevzuat raporu | Tasarlandı | İşletme | İşletme 2. sürüm |
| Fermantasyon profili editörü | Tasarlandı | İkisi | Ev MVP |
| Otomatik uyarı kuralları | Tasarlandı | İkisi | Ev MVP |
| Ekipman sihirbazı (4 soru) | Tasarlandı | İkisi | Ev MVP |
| Cihaz eşleştirme | Tasarlandı | İkisi | Ev MVP |
| Mal kabul ve stok sayımı | Tasarlandı | İkisi | Ev MVP |
| Takvim ve geriye planlama | Tasarlandı | İkisi | Ev MVP |
| Mead, cider, seltzer tarifi | Tasarlandı | İkisi | Ev MVP |
| Boş, bağlantı yok ve hata durumları | Tasarlandı | İkisi | Ev MVP |
| Abonelik ve ödeme | Tasarlandı | İkisi | Ev MVP |
| Global arama | Tasarlandı | İkisi | Ev MVP |
| Apple Watch: sayaç listesi, adım onayı (opsiyonel) | Tasarlandı | İkisi | İleride, opsiyonel |
| Stil kılavuzu karşılaştırması (BJCP ve BA) | Tasarlandı | İkisi | Ev MVP |
| Tank hareketi: transfer, bölme, birleştirme | Tasarlandı | İşletme | İşletme çekirdek |
| Ortak cihazda kullanıcı değiştirme | Tasarlandı | İşletme | İşletme çekirdek |
| Kullanıcı davet akışı | Tasarlandı | İşletme | İşletme çekirdek |
| Denetim kaydı | Tasarlandı | İşletme | İşletme çekirdek |
| Sapma kaydı (kalite) | Tasarlandı | İşletme | İşletme çekirdek |
| Etiket bilgisi | Tasarlandı | İşletme | İşletme 2. sürüm |
| Günlük tank turu çıktısı | Tasarlandı | İşletme | İşletme çekirdek |
| Parti başlatma (tariften partiye) | Tasarlandı | İkisi | Ev MVP |
| Tarif ölçekleme | Tasarlandı | İkisi | Ev MVP |
| Hesap (hesap silme dahil) | Tasarlandı | İkisi | Ev MVP |
| Tank detayı | Tasarlandı | İşletme | İşletme çekirdek |
| Lot izlenebilirlik raporu | Tasarlandı | İşletme | İşletme çekirdek |
| "Nasıl hesaplandı" sayfası | Tasarlandı | İkisi | Ev MVP |
| Kendi malzemesini ekle | Tasarlandı | İkisi | Ev MVP |
| Su analiz raporu girişi | Tasarlandı | İkisi | Ev MVP |
| Kalibrasyon (ekipman profili önerileri, "bu partiden öğrenilen") | Tasarlandı | İkisi | Ev MVP |
| Operatör görünümü (ISA-101 tarzı: Üretim panosu, Bugün, Tank detayı, Demleme günü) | Tasarlandı | İşletme | İşletme çekirdek |
| Pano ve Bugün düzenleme (kart ekle, sırala, kayıtlı görünüm) | Tasarlandı | İşletme | İşletme çekirdek |
| Bildirim yönlendirme (öncelik, sessiz saat, vardiya, yükseltme) | Tasarlandı | İkisi | Ev MVP (vardiya ve yükseltme İşletme çekirdek) |
| Mod ayrımı (Ayarlar › Çalışma alanı › Mod; Ev'de açılabilen özellikler, ekran haritası) | Tasarlandı | İkisi | Ev MVP |
| Kaplar (Ayarlar › Ekipman ve tanklar › Kaplar; şişe, kutu, keg, cask) | Tasarlandı | İkisi | Ev MVP (kutu ve cask İşletme çekirdek) |
| Gezinme kalıpları (geri düğmesi, sheet ve silme onayı, "Demlemeye dön" şeridi, derin bağlantı, "Devam et", web iz yolu) | Tasarlandı | İkisi | Ev MVP |
| Bildirim kutusu (çan): liste, eylemler, geçmiş | Tasarlandı | İkisi | Ev MVP |
| Giriş ve hesap bağlama (e-posta bağlantısı, giriş yöntemleri, passkey, aynı e-posta başka hesapta) | Tasarlandı | İkisi | Ev MVP |
| Yardım ve destek (yardım arama, talep açma, süreli erişim izni, talep yanıtı) | Tasarlandı | İkisi | Ev MVP |
| Davet kabulü (davet edilen taraf, ortak cihaz PIN'i, süresi dolmuş davet) | Tasarlandı | İşletme | İşletme çekirdek |

## Ekran akışı

Ekranlar arası geçişin şeması [Demleme Arayüz Konsepti](https://claude.ai/artifact/BCEHj3wb6xj7eShvjTxKn6)'nde "Ekranlar arası geçiş" bölümünde; kutulara tıklayınca ilgili ekran açılır.

### Ana iş akışı

1. **Tarif:** Sihirbaz, içe aktarma veya tarif listesinden (şablon, mevcut tarif) tarif editörüne. Editörden açılan sayfalar: su ve pH, su raporu, mash ve maya, malzeme ve kendi malzemen, ölçekleme, "nasıl hesaplandı", stil kılavuzları.
2. **Onay (yalnız İşletme):** tarif sürümü onaya gönderilir; onaylı sürümden parti açılır.
3. **Parti başlatma:** tarih, ekipman, hacim, tank; stoktan ayırır, eksikleri alışveriş listesine, hazırlığı takvime ekler. Stok mal kabulle beslenir.
4. **Demleme günü:** sayacı Live Activity ile paylaşır; brew sheet yazdırılır, taranıp "Kağıt ve düzeltme"de işlenir.
5. **Fermantasyon:** fermantasyon profili, cihazlar ve uyarı kuralları besler. İşletmede tank detayı, tank hareketi, kalite ve sapma buradan açılır.
6. **Ambalaj (İşletme):** CIP ve ambalaj kaydı; ardından etiket ve izlenebilirlik. Ev modunda ayrı ambalaj ekranı yok: fermantasyondaki "Şişele / fıçıla" eylemi (priming, adet) partiyi Ambalajlandı yapar, ardından parti özeti gelir.
7. **Parti özeti:** plan–gerçekleşen ve tadım; tadım notu tarifin yeni sürümüne döner.

### Parti durumları

| Durum | Sonraki duruma geçiş |
| --- | --- |
| Planlandı | "Demlemeyi başlat" |
| Demleniyor | maya atıldı |
| Fermantasyonda | FG sabit, cold crash |
| Kondisyon | serbest bırakma: İşletmede Kalite onayı; Ev'de FG birkaç gün sabit kalınca öneri |
| Ambalaja hazır | ambalaj kaydı |
| Ambalajlandı | özet onaylandı |
| Kapandı | düzeltme yalnız yetkiyle, denetim kaydına yazılır |

Sapma açılan parti "Beklemede" durumuna alınır; karardan sonra kaldığı yere döner ya da imha edilir. Durum, üretim panosundaki sütunu ve Bugün listesindeki görevleri belirler.

### Giriş noktaları

| Giriş | Açılan ekran |
| --- | --- |
| İlk açılış | İlk açılış → ekipman sihirbazı |
| Uygulama simgesi · Ev | Tarif listesi; aktif parti varsa üstte kartı |
| Uygulama simgesi · İşletme | Role göre: Operatör Bugün, Brewmaster Üretim panosu, Kalite Onaylar; ortak cihazda önce kullanıcı değiştir |
| Bildirim | Demleme günü, Fermantasyon veya Onaylar; ilgili partinin ilgili adımına; tümü ayrıca bildirim kutusunda (çan) |
| Live Activity | Demleme günü |
| Widget | Fermantasyon |
| Kağıttaki QR kod | Kağıt ve düzeltme (doğru parti ve tur seçili) |
| Spotlight, uygulama içi arama | Arama → kayıt |
| Paylaşılan dosya (BeerXML, .bsmx) | İçe aktarma |
| Çan simgesi | Bildirim kutusu; hatırlatıcıdan hızlı ölçüm girişi |
| E-postadaki giriş bağlantısı | Giriş ve hesap bağlama (15 dk geçerli, tek kullanımlık) |
| Davet bağlantısı | Davet kabulü; İşletmede ardından rolün açılış ekranı. Uygulama yoksa web'de açılır |
| Destek yanıtı | Yardım ve destek › talep |
| Eski uygulama sürümü, süresi dolmuş demleme paketi | Güncelleme gerekli; paket yenileme (Boş ve hata durumları) |
| Abonelik veya deneme bitti | Abonelik; veriler silinmez, ücretli özellikler salt okunur |

## Veri modeli

Veri modeli ilk sürümden itibaren işletme modunu taşıyacak şekilde kurulur; sonradan değiştirmesi en pahalı kısım budur. BeerJSON'a yakın tutulur, iç birim SI.

| Varlık | İçerik |
| --- | --- |
| Workspace, User, Role | Çalışma alanı ve mod (Ev/İşletme), kullanıcılar, düzenlenebilir roller; Permission, RoleAssignment, Invitation, DeviceSession |
| Recipe, RecipeVersion | Değişmez tarif sürümleri (plan); durum: taslak / incelemede / onaylı / arşiv |
| Ingredient, IngredientUse | Kütüphane malzemesi; tarifte referans + hesapta kullanılan özelliklerin anık kopyası |
| Equipment, Profile | Ekipman, mash, kaynatma, fermantasyon, karbonasyon profilleri |
| StyleGuide, Style, StyleMapping | Kılavuz (ad, yıl, lisans), stil aralıkları, kılavuzlar arası eşleşme |
| Batch | Parti: kod, tarif sürümü, aşama, tarihler |
| BatchStep | Planlanan ve gerçek başlangıç/bitiş, durum, giriş kaynağı (sayaç / elle / kağıt) |
| BatchIngredientActual, Deviation | Gerçek miktar ve ikame, plandan sapmalar |
| Measurement, StageEvent | Zaman damgalı ölçümler (kaynak: cihaz / elle / kâğıt) ve aşama geçişleri |
| Vessel, VesselOccupancy, Transfer | Tank/kazan; parti ↔ tank çoktan çoğa (bölme ve birleştirme); transferler |
| StockLot, StockMovement | Lot bazlı stok, hareket defteri |
| ScannedDocument | Taranan kağıt: parti, form türü, tarayan, saklama süresi |
| ImportSource | İçe aktarılan kaynak dosya ve eşleştirme kararları |
| AuditLog | Kim, ne zaman, önceki ve yeni değer; EditSession (yumuşak kilit) ile birlikte |
| PackageType, PackagingRun | Kap tanımı (tür: şişe / kutu / keg / cask, hacim, tek kullanımlık mi, malzeme, ağırlık); ambalaj kaydı satırları (parti, kap, adet, litre, lot, karbonasyon yöntemi) |
| Notification, NotificationEvent, Reminder | Bildirim (öncelik, parti, adım, durum: açık / ertelendi / onaylandı / yoksayıldı / çözüldü); olaylar (gönderildi, okundu, onaylandı, ertelendi; kim, hangi cihaz, ne zaman); hatırlatıcı (kural, zaman, bağlı adım) |
| CalcResult | Kaydedilen hesap sonucu: girdi özeti (hash), motor sürümü, çıktılar; tarif sürümüne ve partiye bağlı |
| UserIdentity | Giriş kimlikleri: sağlayıcı (Apple / Google / e-posta), sağlayıcıdaki kalıcı kimlik, bağlanma zamanı; bir kullanıcıda birden çok; passkey kayıtları |
| SupportGrant, SupportAction | Destek erişim izni (çalışma alanı, veren, bağlı talep (SupportRequest), başlangıç ve bitiş); destek ekibinin her işlemi (kim, neye, ne yaptı, gerekçe) |
| Device, AlertRule | Cihaz (tür, veri adresi anahtarı, bağlı parti veya tank, kalibrasyon kaydı); uyarı kuralı (koşul, öncelik, kapsam, susturma) |
| BrewDayPackage | Demleme paketi: parti, imza, oluşturma ve bitiş zamanı, içerik sürümü; kuyruktaki ölçümler bu kimlikle gelir |
| QualitySpec, LabResult, Release | Bira başına spesifikasyon aralıkları; laboratuvar ve tadım sonuçları; serbest bırakma kararı (kim, ne zaman) |
| CleaningRecord, YeastGeneration | CIP kaydı (tank, kimyasal, konsantrasyon, sıcaklık, süre, doğrulama); maya hasadı ve jenerasyon zinciri |
| Subscription, Entitlement | Abonelik (kaynak: App Store / web, durum, yenileme, deneme); çalışma alanının açık özellikleri; yalnız sunucuda |
| Attachment | Parti fotoğrafı ve notu, tadım fotoğrafı; nesne deposunda, partinin zaman çizelgesine bağlı |
| SupportRequest, SupportMessage | Destek talebi: genel numara (#4182), çalışma alanı ve takma kimliği (rastgele, kalıcı; GitHub'da yalnız bu görünür), açan kullanıcı, konu, başlık, isteğe bağlı ilgili kayıt (tarif sürümü veya parti), durum (açık / yanıtlandı / çözüldü / kapandı), isteğe bağlı tanılama özeti, GitHub Issue numarası ve son eşitleme. Mesaj: yazar (müşteri, destek, sistem), metin, zaman, ekler (nesne deposunda), görünürlük (müşteri / iç not), GitHub yorum kimliği. İç not müşteriye, bildirime ve e-postaya gitmez |

Her kayıtta UUID, güncelleme zamanı, silinme zamanı ve sürüm numarası bulunur. Demleme günü ölçümleri cihazda üretilen UUID ile kuyruğa alınır; sunucu aynı kaydı iki kez işlemez.

## Yol haritası

&#91;embedded content: yol haritası · 3 aşama, 2 kapı\]

Temel aşama altın test seti geçince Ev modu başlar; beta (20–50 ev biracısı) tamamlanınca yayın ve ardından İşletme modu gelir. Tarih henüz belirlenmedi; ekip ve bütçe netleşince eklenecek.

Ek özelliklerin aşamalara dağılımı "Ek özellikler" tablosunda: Ev MVP'ye ekipman kalibrasyonu, otomatik uyarılar, tarif karşılaştırma, geriye planlama, dekoksiyon aracı, kalori, ekstrakt araçları, parti fotoğrafı, favoriler ve şişe başına maliyet; İşletme modu çekirdeğine çoklu kullanıcı ve roller, lota özel hammadde, spesifikasyon ve serbest bırakma, CIP kaydı, verim analizi, maya jenerasyon takibi.

## Planlama işleri ve açık konular

Öncelik sırasıyla ilk beş iş: kullanıcı görüşmeleri ve MVP kapsamı, altın test seti ve formül kararları, veri modeli ve API, teknik deneme, ürün adı ve mevzuat kontrolü.

- [ ] 5–10 ev biracısı ve 2–3 küçük işletmeyle görüşme; MVP kapsamı ve kullanıcı hikayeleri. Kılavuz ve boş karar tablosu: plan/gorusme.md; görüşme notları yazılmadan altın test doldurulmaz
- [ ] Rakip incelemesi: Brewfather, Brewer's Friend, Grainfather; işletme tarafında Ekos, Ollie, Beer30 ve Breww özellikleri karşılaştırıldı (Kasım 2026); FoamByte (ev ve nano üretici, web) ile karşılaştırma karsilastirma/foambyte-\*.md dosyalarında; güncel fiyatlar ve FoamByte ücretsiz katman sınırları açık
- [ ] Altın test seti (10–20 tarif) ve gerçek demleme ölçümleriyle kalibrasyon; MPH ve BW pH modellerinden hangisinin daha doğru olduğu. Boş şablon: formuller/ALTIN-TEST.md (16 senaryo, BeerSmith ekranı ve gerçek ölçümle); Python test vektörleri bu testin yerine geçmez
- [ ] Formül kararlarının karar kaydına (ADR) yazılması; hassasiyet ve yuvarlama kuralları
- [ ] Malzeme veritabanı kaynağı (üretici teknik föyleri, ABD ve İngiltere'de satılan ürünler öncelikli) ve güncelleme süreci
- [ ] Veri modeli ve API; kayıtta sürüm kontrolü
- [ ] Teknik deneme: Rust motorun sunucudaki yanıt süresi (tarif düzenlerken bekleme hissedilmemeli), demleme paketi ve ölçüm kuyruğu, Live Activity ve arka planda sayaç, Bluetooth (Tilt), VisionKit tarama. Atılabilir prototiptir, ana dala girmez; görüşme ve altın test beklenirken yapılabilir
- [ ] İçe aktarma: BeerXML, BeerJSON, `.bsmx`, Brewfather ve Brewer's Friend formatlarının çözümlenmesi; örnek dosyalar
- [ ] Tasarım sistemi, tıklanabilir prototip, kullanıcı testi; erişilebilirlik; TR/EN yerelleştirme
- [ ] Ürün adı ve marka araştırması, alan adı
- [ ] UK GDPR, ABD eyalet gizlilik yasaları ve KVKK; gizlilik politikası, kullanım şartları
- [ ] BJCP ve BA kullanım izinleri
- [ ] Alkol reklam ve tanıtım mevzuatı; App Store yaş sınırı
- [ ] İşletme modu için ilk hedef pazar (ABD veya İngiltere; Avustralya, Almanya/AB ve Türkiye sonra; bkz. "Ülkelere göre yerelleştirme"), vergi rakamlarının resmi kaynaktan doğrulanması ve kayıt saklama süreleri
- [ ] Ekip, bütçe, kilometre taşları; beta programı (20–50 ev biracısı)
- [ ] Altyapı kurulumu: Google Cloud organizasyonu ve geliştirme, test, canlı projeleri; Terraform temeli; birleştirme kapısı (CI); Apple ve Google giriş kayıtları ve OAuth onay ekranı doğrulaması; e-posta sağlayıcısı ve alan adı doğrulaması (Apple gizli e-posta rölesi dahil); destek için özel GitHub deposu ve GitHub App
- [ ] Yayın öncesi bağımsız güvenlik incelemesi ve sızma testi: kimlik, yetki, çalışma alanı ayrımı, destek paneli
- [ ] Destek talepleri (karar verildi): kayıt bizde (SupportRequest, SupportMessage), destek ekibi GitHub Issues'ta, iki yönlü eşitleme; özel destek deposu ve GitHub App kurulumu

**Riskler:** arayüzün iki kez yazılmasının maliyeti; web–Apple özellik eşitliği; el yazısı tanımanın hatalı okuması; işletme segmentinde mevzuat yükünün sürekliliği; her hesabın internete bağlı olması (yanıt süresi, sunucu maliyeti, kesinti); kodun tamamını Claude'un yazması (kimlik, yetki ve destek paneli gibi güvenlik kodunda bağımsız inceleme gerekir).
