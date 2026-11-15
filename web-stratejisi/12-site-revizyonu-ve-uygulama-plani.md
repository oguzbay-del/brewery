# Demleme — Pazarlama Sitesi Revizyon Planı

**Durum:** UYGULAMA SÜRÜYOR — 15 Kasım 2026 itibarıyla kullanıcı pazarlama sitesi uygulamasına devam edilmesini onayladı.  
**Kapsam:** Yalnız pazarlama sitesi ve web stratejisi dokümanları. Ürün uygulaması/API/iOS/Rust ve Claude geliştirme kuyruğu kapsam dışıdır.  
**Plan sahibi:** Demleme web stratejisi  
**Referans:** `plan/demleme-plani.md`, `web-stratejisi/`, `karsilastirma/`

---

## 1. Amaç ve kapsam

Mevcut pazarlama sitesini parça parça yamamak yerine, kullanıcıdan gelecek ek girdiler de toplandıktan sonra bütünsel olarak yeniden ele almak. Hedef; Demleme'yi yalnızca özellik listesi sunan bir ürün sayfası olmaktan çıkarıp, ziyaretçiye gerçek demleme problemlerini, ürünün yaklaşımını ve teknik güvenilirliğini anlatan; organik trafik ve self-service edinim sağlayabilecek bir kaynağa dönüştürmek.

### Sabit stratejik kararlar

- Birincil ticari audience: **Professional Breweries (Business)**; ana edinim ve self-service funnel bu segmente odaklanır.
- İkincil audience: **Homebrewers (Home)**; ayrı ürün yolu ve içerik kitlesi olarak korunur, Business ile eşit ticari öncelik taşımaz.
- Konumlandırma: teknik; hesaplama doğruluğu, yöntem şeffaflığı, tekrarlanabilirlik ve üretim disiplili.
- Öncelikli pazarlar: ABD ve İngiltere.
- Birincil site dili: İngilizce.
- Ticari yaklaşım: self-service öncelikli; demo ana dönüşüm yolu değil.
- Site ürün uygulaması değildir.
- ERP kapsamı vaat edilmeyecek: satış, sipariş, dağıtım/sevkiyat, fıçı takibi, CRM ve muhasebe.
- Ürün planında onaylanmamış araştırma fikirleri taahhüt gibi sunulmayacak.
- Ürün ve site canlı değilken mevcut olmayan özellik, hesaplayıcı, fiyat, entegrasyon, müşteri, metrik veya ekran varmış gibi gösterilmeyecek.
- Fotoğraf ve gerçek ürün ekranı kullanıcı sağlamadan eklenmeyecek. Mevcut görsel slotları korunacak; kullanıcı onayı olmadan fotoğraf üretilmeyecek veya bulunup eklenmeyecek.
- Bu çalışma yalnızca pazarlama sitesi ve web stratejisi dokümanları kapsamındadır. Ürün uygulaması, API, `web/`, `ios/`, sunucu ve Claude geliştirme kuyruğu değiştirilmeyecek.

## 2. Referans karşılaştırmasından çıkarımlar

`karsilastirma/` klasöründeki FoamByte incelemeleri, hesaplayıcı karşılaştırmaları ve teknik içerik analizleri temel alınacak.

### Alınacak anlatım kalıpları

- Ürün kabiliyetlerini tek bir uzun liste yerine, anlaşılır ve ayrı konu sayfalarıyla açıklamak.
- Recipe Studio / Brewing Profiles / Batch Tracking gibi isimlendirilmiş iş alanlarının her birinde: hangi problem, kim için, nasıl bir iş akışı ve hangi fayda anlatıldığını göstermek.
- Hesaplayıcıları ve teknik bilgi sayfalarını ürün tanıtımından bağımsız, arama niyetine uygun giriş noktaları olarak kullanmak.
- FAQ ve açıklayıcı içerikle ziyaretçinin itiraz ve sorularını karşılamak.
- Sayfa içi bağlantılarla bilgi içeriğini ilgili ürün alanına bağlamak.

### Kopyalanmayacaklar

- Rakibin metni, görsel dili, marka terminolojisi veya sayfa tasarımı kopyalanmayacak.
- Demleme'nin ürün planında olmayan özellikler rakipte bulunduğu için eklenmeyecek.
- Rakibin var olan işlevleri Demleme'de canlıymış gibi anlatılmayacak.
- Demleme'nin ayrıştırıcı noktası teknik açıklık ve ürün planındaki Home/Business iş akışları olacak.

## 3. Önerilen bilgi mimarisi

Bu sitemap taslak öneridir; ek kullanıcı girdileri geldikten sonra kesinleştirilecek.

| Alan | Amaç | Öncelik |
|---|---|---|
| Home | Demleme'yi Professional Breweries için konumlandırmak; Homebrewers'a ikincil yol sunmak | P0 |
| Breweries | Birincil ticari landing page: profesyonel üretim iş akışları, ürün sınırları ve self-service değerlendirme | P0 — primary |
| Product / Features | Onaylı ürün yol haritasını Business öncelikli keşif hiyerarşisiyle anlatmak | P0 |
| Homebrewers | Ayrı Home modu; ikincil audience ve organik içerik yolu | P1 — secondary |
| How It Works | İki kitle için uçtan uca iş akışlarını anlatmak | P1 |
| Calculators | Ücretsiz, doğrulanmış etkileşimli bira üretim hesaplayıcıları | P1 |
| Learn / Knowledge Center | Teknik rehberler, yöntem açıklamaları ve sözlük | P1 |
| Use Cases | Somut kullanıcı senaryoları ve problem-çözüm anlatımı | P1 |
| FAQ | Ürün, yöntem, platform, kapsam ve ticari sorular | P1 |
| Technical Principles | Hesaplama, birim, veri ve doğrulama yaklaşımını açıklamak | P1 |
| Roadmap / About | Ürünün yaklaşımı, geliştirme aşamaları ve kapsam sınırları | P2 |
| Pricing | Ticari kararlar kesinleşince paket ve fiyat bilgisi | P2 / beklemede |
| Sign in / Get started | Gerçek ürün akışları hazır olduğunda | Ürün bağımlılığı |

Navigasyonun son hali sayfa hiyerarşisi ve kullanıcı testleriyle belirlenecek. Gereksiz menü kalabalığı oluşturulmayacak; Calculators ve Learn gibi trafik getiren alanlar kolay bulunacak.

## 4. Yeni içerik ve sayfa grupları

### 4.1 How It Works — P1

Özellik listelerini kullanıcı yolculuğuna dönüştürür. En az iki ayrı akış tasarlanır.

**Home akışı**
1. Recipe design
2. Ingredients, water profile and brewing calculations
3. Brew-day preparation and execution
4. Fermentation monitoring and measurements
5. Packaging / conditioning (ürün planındaki kapsamla sınırlı)
6. Batch review and learning

**Business akışı**
1. Recipe formulation, review and approval
2. Production planning and tank allocation
3. Brew / production records
4. Fermentation and tank workflow
5. Quality checks, deviations and traceability
6. Packaging / compliance kayıtları (yalnız planlanan sürüm kapsamı)

Her aşama için:
- Kullanıcının amacı ve yaşadığı problem
- Gerekli bilgi / girdi
- Planlanan Demleme işlevi
- Üretilen kayıt veya karar
- Home / Business farkı
- Üründeki geliştirme aşaması

Gerçek ekran yoksa sahte UI üretmek yerine süreç diyagramı, metin ve editoryal illüstrasyon kullanılabilir. Ürün arayüzü görseli ancak gerçek ve onaylı ekran sağlanırsa kullanılabilir.

### 4.2 Calculators — P1

Önerilen ilk adaylar (kesin liste değildir):
- OG / FG / ABV
- IBU
- Beer colour (SRM / EBC)
- Priming sugar / carbonation
- Strike water temperature and volume
- Mash water / sparge water / brew-house volume calculations
- Recipe scaling
- Water profile and mineral additions
- Mash pH — yalnız hesap modelinin sınırları ve doğrulaması açıklanabildiğinde

Her hesaplayıcı için ayrı görev kartı ve kabul kriteri:
1. Ürün planı ve mevcut hesap motoru ile kapsam doğrulaması.
2. Kullanılan formül, varsayımlar, birimler ve kaynakların belirlenmesi.
3. Bağımsız referans örnekleri ve sınır durumlarıyla doğrulama.
4. Yuvarlama, birim dönüşümü, giriş doğrulama ve hata mesajları.
5. Sonuçla birlikte “How is this calculated?” açıklaması.
6. Mobil kullanılabilirlik, klavye ve ekran okuyucu erişilebilirliği.
7. İlgili Learn makalesi ve ürün özelliğine bağlantı.
8. Formül doğrulaması ve hukuki/teknik inceleme tamamlanmadan yayınlamama.

Hesaplayıcılar mevcut ürün API'sine bağlanacakmış gibi tasarlanmayacak. Web'de bağımsız çalışacaklarsa hesap motoru ile formül tutarlılığı için ayrıca mimari karar verilecek. Sunucuya erişim veya kullanıcı verisi gereksinimi doğarsa gizlilik ve güvenlik incelemesi yapılacak.

### 4.3 Learn / Knowledge Center — P1

Konu kümeleri:
- Brewing fundamentals
- Recipe formulation
- Malt, hops and yeast
- Water chemistry and mash pH
- Gravity, attenuation, ABV and extract
- IBU, hop utilization and bitterness
- Fermentation, temperature and yeast health
- Carbonation and packaging
- Equipment, calibration and measurement
- Professional brewery operations, quality and traceability

İçerik formatları:
- Pillar guide: geniş ana rehber.
- Supporting article: belirli soruya odaklı makale.
- Calculation explainer: formül, varsayım, örnek ve kaynak.
- Glossary: terim ve birim açıklaması.
- FAQ: kısa, doğrudan yanıt.
- Product connection: ilgili özellik sayfasına bağ.

İçerik bilimsel doğruluk kontrolünden geçecek. Kaynaksız kesin iddia, yapay uzman görüşü, uydurma deneyim veya SEO için anlamsız tekrar kullanılmayacak.

### 4.4 Use Cases — P1

İlk senaryo adayları:
- Bir reçeteyi farklı batch hacmine ölçeklemek.
- Hedef su profiline göre mineral ve pH kararlarını değerlendirmek.
- Demleme günü hedef ve gerçekleşen ölçümleri karşılaştırmak.
- Fermantasyon eğrisindeki sapmayı incelemek.
- Tarif sürümleri arasındaki farkları anlamak.
- İşletmede onaylı reçeteyi üretim partisine dönüştürmek.
- Tank takvimi ve kapasite çakışmasını değerlendirmek.
- Hammadde lotundan partiye izlenebilirlik sağlamak.
- Kalite kontrol ve vardiya kayıtlarını bir arada tutmak.

Her use case ürün planındaki ilgili ekran ve geliştirme aşamasına bağlanacak. Sonuç, tasarruf, verim artışı veya müşteri başarısı gibi ölçülebilir iddialar kanıt olmadan yazılmayacak.

### 4.5 FAQ — P1

Konu başlıkları:
- Demleme nedir / kimler için?
- Home ve Business modları nasıl ayrılır?
- Hangi cihaz ve platformlar hedefleniyor?
- İnternet ve çevrimdışı çalışma sınırları nedir?
- Hangi stil kılavuzları ve birimler desteklenecek?
- Hesaplamalar nasıl doğrulanıyor?
- Veri, gizlilik, güvenlik ve yedekleme yaklaşımı nedir?
- Ürün ne zaman hazır olacak? (tarih verilmeden, yalnız onaylı durum)
- Fiyat, deneme ve abonelik (karar verilene kadar kesin yanıt yok)
- Neler ürün kapsamı dışında?

### 4.6 Technical Principles — P1

Teknik güven için açıklanabilecek alanlar:
- Formül ve hesaplama şeffaflığı.
- Birim sistemi ve dönüşüm yaklaşımı.
- Ölçülen değer ile hesaplanan/hedef değerin ayrımı.
- Ekipman profili ve kalibrasyonun önemi.
- Reçete sürümleme ve üretim kaydı ilkeleri.
- Test ve doğrulama yaklaşımı.

Yalnız doğrulanmış mimari/ürün kararları yayımlanacak. Güvenlik sertifikası, mevzuat uyumu, uptime, yedekleme SLA'sı veya entegrasyon gibi iddialar kanıt olmadan kullanılmayacak.

## 5. SEO ve edinim planı

### Arama niyeti ayrımı

- Homebrewing sorguları Homebrewers, ilgili Calculators ve Learn içeriklerine yönlenecek.
- Profesyonel üretim sorguları Breweries, Business use cases ve ilgili teknik içeriklere yönlenecek.
- Tek sayfa iki farklı kitleyi ve farklı arama niyetlerini gereksiz yere karıştırmayacak.

### İçerik kümeleri

**Home**
- Home brewing software
- Beer recipe calculator
- Brewing water calculator
- OG / FG / ABV calculator
- IBU calculator
- Mash and strike water calculations
- Homebrew fermentation tracking

**Business**
- Brewery recipe management
- Brewery production planning
- Craft brewery batch records
- Brewery tank scheduling
- Brewing quality and traceability

Arama hacmi, zorluk, niyet ve ürün uygunluğu araştırılmadan anahtar kelime hedefleri kesinleştirilmeyecek.

### Teknik SEO teslimatları

- Her sayfada benzersiz title, description, canonical ve Open Graph.
- Doğru hreflang / dil yaklaşımı (çok dilli yayın kararı verilirse).
- XML sitemap, robots.txt, 404 ve yönlendirme haritası.
- Schema.org yalnız sayfa türü ve gerçek içerik uygunsa.
- Dahili bağlantı, breadcrumb ve semantik başlık hiyerarşisi.
- Core Web Vitals, görsel optimizasyonu ve mobil performans.
- Index/noindex politikası, yinelenen içerik ve canonical kontrolü.
- Search Console ve analitik kurulumu; consent ve gizlilik gereklilikleri.

## 6. Görsel tasarım ve içerik dili

- Mevcut tasarım sistemi ve teknik/editorial yön başlangıç referansı olacak; bütünsel revizyon sırasında yeniden değerlendirilecek.
- FoamByte veya başka rakiplerin tasarımı kopyalanmayacak.
- Home ve Business farklı görsel/mesaj ritmine sahip olabilir; tek marka sistemi korunacak.
- Gerçek ürün ekranı, gerçek fotoğraf, gerçek müşteri kanıtı yoksa bunların yerine sahte içerik konmayacak.
- Her görsel slotu ID ve değişim notuyla korunacak.
- Mobil önce tasarlanacak; masaüstü, tablet ve mobil kırılımlar tanımlanacak.
- Kontrast, tipografi, klavye, odak görünümü, reduced motion ve ekran okuyucu kontrolleri tasarımın parçası olacak.

## 7. Ticari dönüşüm ve ölçüm

Self-service önceliği korunacak. Gerçek kayıt/ürün akışları hazır olana kadar CTA'lar sahte login, satın alma veya demo işlevine bağlanmayacak.

Ölçülecek olaylar (altyapı ve izinler uygun olduğunda):
- Audience path seçimi
- Feature detail görüntüleme
- Calculator başlatma / tamamlama
- Learn içerik keşfi ve ilgili ürün sayfasına geçiş
- FAQ etkileşimi
- Waitlist / kayıt dönüşümü (yalnız gerçekten kurulursa)
- İlk reçete / ilk batch gibi ürün aktivasyonları (ürün analitiği hazır olduğunda)

Pricing ve ticari kararlar beklemede: paket, trial, limit, para birimi, ödeme, vergi, iptal ve iade netleşmeden fiyat tablosu veya ticari vaat yayınlanmayacak.

## 8. Teknik uygulama ve mimari karar kapıları

- Mevcut Astro + TypeScript statik site yapısı başlangıç noktasıdır; yeniden değerlendirme yapılmadan framework değiştirilmeyecek.
- Vercel tercih edilen hosting; Hobby planın ticari kullanım sınırları nedeniyle ticari yayına geçiş öncesi uygun plan ve domain kararı alınmalı.
- Calculators için statik istemci hesaplaması mı, ortak hesap motoru/API mi kullanılacağı formül ve ürün mimarisi denetiminden sonra kararlaştırılmalı.
- Blog/Learn içerik yönetimi için Markdown/MDX veya hafif CMS seçenekleri; içerik hacmi ve editör ihtiyacına göre değerlendirilmeli.
- Gereksiz bağımlılık, büyük JS paketi ve üçüncü taraf izleyici eklenmemeli.
- Güvenlik, gizlilik, çerez/consent, form spam koruması ve veri saklama tasarıma dahil edilmeli.
- Site üretim kodu ve ürün uygulaması birbirinden izole tutulmalı.

## 9. İş paketleri ve küçük görev dilimleri

İşler birbirinden doğrulanabilir, küçük PR'lara ayrılacak. Aynı anda aynı dosyalara yazan paralel ajanlar kullanılmayacak.

| Paket | İş | Çıktı / kabul kriteri | Bağımlılık |
|---|---|---|---|
| WP-00 | Yeni kullanıcı girdilerini toplama ve karar günlüğü | Açık sorular, sabit kararlar, değişiklik listesi | Kullanıcı girdileri |
| WP-01 | Mevcut site ve repo envanteri | Sayfa, bileşen, route, link, SEO, görsel slotu ve içerik matrisi | WP-00 |
| WP-02 | Rakip anlatım ve pazar araştırması | FoamByte + seçilecek rakipler için kaynaklı benchmark matrisi | Rakip listesi / WP-00 |
| WP-03 | Ürün-planı ve iddia denetimi | Her pazarlama iddiasının ürün planı, audience ve release aşamasıyla eşleşmesi | WP-01 |
| WP-04 | Sitemap ve kullanıcı yolculukları | Home/Business ayrı akışlar, sayfa hiyerarşisi, CTA haritası | WP-02, WP-03 |
| WP-05 | İçerik stratejisi ve mesaj mimarisi | Sayfa bazlı amaç, arama niyeti, H1, içerik blokları ve iç link planı | WP-04 |
| WP-06 | Wireframe ve UX | Mobil/desktop wireframe, navigasyon, etkileşim ve erişilebilirlik kriterleri | WP-04 |
| WP-07 | Görsel sistem ve art direction | Renk, tipografi, grid, görsel slotları, gerçek asset gereksinimleri | WP-00, WP-06 |
| WP-08 | How It Works ve Use Cases içerik tasarımı | Home/Business akış ve senaryo metinleri; ürün iddiası kontrolü | WP-03, WP-05 |
| WP-09 | Calculator formül ve doğruluk denetimi | Formül kaynakları, test vektörleri, sınır durumları, yuvarlama kuralları | Ürün hesap motoru ve kaynak dosyaları |
| WP-10 | Calculator UX ve teknik prototip | Tek hesaplayıcıda doğrulanmış referans uygulama ve testler | WP-09, WP-06 |
| WP-11 | Learn bilgi mimarisi ve ilk içerik seti | Pillar/supporting içerik haritası, kaynaklı taslaklar, FAQ | WP-05 |
| WP-12 | SEO teknik altyapısı | Metadata, canonical, sitemap, schema, robots, redirect planı | WP-04, WP-05 |
| WP-13 | Astro site geliştirme | Onaylı tasarıma göre sayfa ve bileşenler; ürün uygulamasına dokunmadan | WP-06, WP-07, WP-08 |
| WP-14 | İçerik ve calculator entegrasyonu | Onaylı içerik ve test edilmiş araçların sayfalara eklenmesi | WP-09–WP-13 |
| WP-15 | Analitik, privacy ve dönüşüm | Consent uyumlu event planı, gerçek CTA hedefleri | WP-00, WP-12 |
| WP-16 | QA ve yayın hazırlığı | Link/route, formül, responsive, erişilebilirlik, SEO, performans ve güvenlik testleri | WP-13–WP-15 |
| WP-17 | Lansman ve izleme | Domain, uygun Vercel planı, Search Console, hata ve dönüşüm takibi | WP-16, ticari onay |

### Her iş paketi için standart teslimat

- Kapsam ve kapsam dışı maddeler.
- Kaynak doküman ve karar referansları.
- Küçük, bağımsız görev listesi.
- Dosya/route etkisi.
- Kabul kriterleri ve test yöntemi.
- Riskler ve açık sorular.
- Kullanıcı onayı gerektiren kararlar.
- PR açıklaması ve doğrulama sonucu.

## 10. Model, ajan ve token/maliyet optimizasyonu

Model seçimi işin risk ve karmaşıklığına göre yapılacak; her görevde en büyük model kullanılmayacak.

| İş türü | Önerilen yaklaşım |
|---|---|
| Repo envanteri, link listesi, metadata kontrolü, dosya sınıflandırma | Küçük/ekonomik model veya deterministik script |
| Tekrarlı sayfa iskeletleri, frontmatter, basit Astro/Markdown düzenlemeleri | Küçük kodlama modeli; mevcut şablon ve testleri takip et |
| İçerik taslağı, başlık varyantı, FAQ gruplama | Ekonomik model; son editoryal onay daha güçlü modelde |
| Ürün planı ile özellik/iddia eşleştirme | Güçlü akıl yürütme modeli; ürün planı doğrudan kaynak |
| Rakip konumlandırma, site mimarisi, SEO stratejisi | Güçlü araştırma/sentez modeli; web kaynakları ve repo belgeleriyle doğrula |
| Formül, birim, hesap doğruluğu, mevzuat ve güvenlik iddiaları | Güçlü model + deterministik test + yetkin insan/uzman doğrulaması; model cevabı tek başına yeterli değil |
| Kritik mimari karar, güvenlik ve yayın kapısı | Güçlü model ve bağımsız ikinci kontrol |
| Son kod incelemesi ve release gate | Kodlama modelinden bağımsız inceleme; CI ve gerçek QA ile doğrula |

### Ajan kullanım kuralları

- Alt ajanlar yalnızca birbirinden bağımsız araştırma veya inceleme görevlerinde kullanılacak.
- Örnek roller: rakip/SEO araştırmacısı, ürün iddia denetçisi, UX içerik denetçisi, teknik/formül doğrulama denetçisi.
- Aynı anda aynı dosyayı değiştiren ajanlar çalıştırılmayacak.
- Araştırma ajanları kaynak, alıntı/parafraz, tarih ve güven düzeyi döndürecek; doğrudan ana dala yazmayacak.
- Bir ana koordinatör sonuçları birleştirip tutarlılık kontrolü yapacak.
- Büyük bağlamı her göreve taşımak yerine yalnız ilgili dosya ve karar özetleri verilecek.
- Tekrarlı repo okuması yerine dosya envanteri ve kısa bağlam özeti kullanılacak.
- Önce küçük bir dikey dilim tamamlanıp ölçülecek; sonra aynı desen çoğaltılacak.
- Token maliyeti düşürmek için gereksiz uzun rapor, tekrar eden web araması ve aynı içeriğin birden fazla modelde yeniden yazılması önlenecek.
- Model isimleri ve erişim seviyeleri oturum anındaki kullanılabilir seçeneklere göre seçilecek; sabit bir model/özellik varsayılmayacak.

## 11. Git, CI ve yayın kuralları

- Planlama ve uygulama ayrı branch/PR'larda tutulacak.
- Bir PR tek bir iş paketini veya küçük ve ilişkili bir görev grubunu kapsayacak.
- PR öncesi Astro/TypeScript check, static build ve ilgili testler çalışacak.
- Preview deploy ve değişen sayfaların doğrulaması beklenecek.
- Kullanıcının belirttiği tercih gereği GitHub Auto-merge kullanılmayacak; merge manuel ve beklenen head SHA doğrulanarak yapılacak.
- Merge sonrası main CI ve Vercel deployment kontrol edilecek.
- Browser QA yapılmadıysa yapılmış gibi raporlanmayacak.
- Fotoğraf, domain, pricing, signup, waitlist ve ticari plan gibi kullanıcı girdisi gereken kararlar varsayılmayacak.

## 12. Aşamalı uygulama sırası

1. Kullanıcının ek bilgi ve kararlarını bekle.
2. Kullanıcı “devam et” dediğinde WP-00 ile girdileri ve açık kararları gözden geçir; yeni bilgi beklenmiyorsa uygulamaya başla.
3. WP-01–WP-05 ile mevcut site, rakip, ürün iddiası, sitemap ve içerik stratejisini tamamla.
4. Wireframe ve art direction onayını al (WP-06/07).
5. Bir dikey dilim seç: örneğin How It Works sayfası; tasarım, içerik, geliştirme ve QA uçtan uca.
6. Kullanıcı onayıyla diğer sayfa gruplarını küçük PR'larla tamamla.
7. Calculator işlerini yalnız formül doğrulaması ve teknik mimari kararı sonrası başlat.
8. Learn içeriklerini kaynak ve teknik editoryal kontrolle ekle.
9. SEO, analitik, privacy, responsive ve erişilebilirlik QA'sını tamamla.
10. Ticari yayın koşulları ve hosting planı doğrulanınca lansman yap.

## 13. Bekleyen kullanıcı girdileri

Bu plan bilinçli olarak açık bırakılmıştır. Kullanıcı ek bilgiler paylaşacaktır; gelmeden nihai tasarım ve kapsam dondurulmayacaktır.

Beklenen olası girdiler (şimdilik soru veya zorunlu form değil):
- Ek rakip siteler veya örnek alınacak sayfalar.
- Özellikle beğenilen anlatım, gezinme, görsel veya etkileşim örnekleri.
- Marka tonu ve tasarım yönü için ek tercihler.
- Hedef kullanıcıların öncelik sırası veya önemli kullanım senaryoları.
- Fotoğraf, gerçek ekran veya marka varlıkları.
- Hesaplayıcıların ilk sürüm önceliği.
- Pricing, waitlist ve ürün lansmanına ilişkin ticari kararlar.

**Önemli:** Bu doküman planlama kaydıdır; kendiliğinden uygulama izni değildir. Kullanıcı “devam et” dediğinde, kaydedilmiş bağlamı yeniden sormadan sıradaki güvenli iş paketi uygulanacaktır. Yeni bilgi bekleniyorsa önce plan güncellenir.

---
## 20. Yeni zorunlu tasarım ve kalite standardı — 14 Kasım 2026

Bu bölüm, kullanıcının verdiği UI/UX ve frontend yönlendirmesini Demleme'ye uyarlanmış kalıcı tasarım standardı olarak tanımlar. Tüm siteyi kapsar; yalnız ana sayfaya uygulanıp alt sayfalar eski standartta bırakılamaz.

### 20.1 Tasarım hedefi ve referans çıtası
- Tasarım ve uygulama titizliği Apple, Vercel ve Stripe seviyesinde hedeflenir: güçlü tipografi, kusursuz hizalama, tutarlı bileşenler, net bilgi hiyerarşisi, hızlı algılanan değer ve güvenilir etkileşim.
- Bu markaların tasarımını, metinlerini veya bileşenlerini kopyalama. Referans, kalite çıtasıdır; Demleme'nin kendi görsel kimliği oluşturulacaktır.
- Genel yön: ultra-modern, minimalist, temiz, teknik ve güven veren; yenilikçi ancak gösterişsiz.
- Açık ve koyu tema desteği tasarım sisteminde düşünülür. Hangi temanın varsayılan olacağı marka/okunabilirlik testinden sonra kararlaştırılır.
- Generous whitespace esastır. İçerik kartları, başlıklar ve tablolar sıkıştırılmaz.
- Bento Grid yalnızca içerik önceliğini ve ilişkileri daha iyi anlattığında kullanılır; her bölümü aynı kart ızgarasına dönüştürmek yasaktır.
- Glassmorphism, blur, gradyan, glow, derin gölge ve hover ölçekleme kontrollü mikro detaylardır. Okunabilirlik, performans veya teknik güven pahasına kullanılmaz.
- Hareket ve geçişler kısa, amaca hizmet eden ve reduced-motion tercihine saygılı olmalıdır. Sürekli hareket eden dekoratif öğeler kullanılmaz.

### 20.2 Design tokens ve renk sistemi
İlk aşamada sabit renk kodu dayatılmaz. Kullanıcının örnek verdiği Indigo/Cyan ve koyu/açık arka plan kodları referans örnekleridir; onaylanmış Demleme marka renkleri değildir.
- Önce mevcut logo, marka varlıkları, mevcut site ve kullanıcıdan gelecek örnekler denetlenir.
- Ardından Primary, Accent, Background, Surface, Border, Text, Muted Text, Success, Warning ve Error semantic token'ları tanımlanır.
- Açık/koyu temada aynı semantic token isimleri korunur; kontrast ve durum renkleri iki temada ayrı ayrı doğrulanır.
- Saf beyaz/siyah yerine göz yormayan yüzey tonları tercih edilebilir; ancak kontrast WCAG 2.2 AA hedefini karşılamalıdır.
- Renk tek başına durum veya anlam iletmemelidir.
- Renk paleti ve token değerleri görsel sistem onayından önce site geneline yaygın biçimde uygulanmaz.

### 20.3 Tipografi ve ritim
- Modern, okunabilir sans-serif aileleri değerlendirilir (ör. Inter, Plus Jakarta Sans, Outfit); nihai font performans, lisans, karakter kapsamı ve görsel uyuma göre seçilir.
- Modular scale için 1.25 veya 1.333 oranlarından biri seçilir; başlık, gövde, küçük metin ve etiket ölçüleri tek bir tutarlı ölçeğe bağlanır.
- Gövde metninde yaklaşık 1.6 line-height başlangıç hedefidir; uzun teknik içeriklerde satır uzunluğu ve paragraf aralığı ayrıca optimize edilir.
- H1 her sayfada tek ve açık olmalı; H2/H3 hiyerarşisi içerik sırasını yansıtmalıdır.
- Teknik tablolar, ölçü birimleri ve sayısal değerler hizalı ve taranabilir olmalıdır; mümkünse tabular numerals kullanılır.

### 20.4 Zorunlu sayfa akışı ve bileşen yaklaşımı
Aşağıdaki bölümler sayfa amacına göre uyarlanır; her sayfaya mekanik olarak eklenmez:
1. Navbar: Minimal logo, anlaşılır menü, Home/Business ayrımı ve bağlama uygun tek ana CTA. Sticky ve blur yalnız scroll davranışında okunabilirlik/erişilebilirlik korunuyorsa.
2. Hero: Tek bir güçlü H1, kısa ve özgül alt açıklama, birincil CTA ve gerekirse ikincil metin bağlantısı. Arka plan görseli yalnız gerçek/izinli varlık veya açıkça şematik özgün görselleştirme olabilir.
3. Güven/kanıt alanı: Partner veya müşteri logoları yalnız gerçek ilişki ve kullanım izni varsa. Aksi durumda logo şeridi kaldırılır; yerine yöntem, kapsam veya doğrulanmış teknik ilkeler anlatılır.
4. Özellik anlatımı: Önceliğe göre değişen Bento Grid veya editoryal bölümler. Her kartın bir amacı ve anlamlı hedef sayfa bağlantısı bulunur.
5. Etkileşimli anlatım: Tabs, stepper, karşılaştırma veya hesaplayıcı yalnız gerçek ve doğrulanmış işlevi açıklıyorsa. Çalışmayan demo veya ürün arayüzü taklidi yapılmaz.
6. Sosyal kanıt: Testimonial, yıldız, avatar, müşteri adı ve başarı metriği yalnız gerçek, doğrulanmış ve yayın izni alınmışsa. Şu anda böyle veri yoksa bölüm kullanılmaz.
7. CTA: Sayfanın sonunda kullanıcının bulunduğu aşamaya uygun, çalışan ve dürüst aksiyon. Ürün hazır değilse kayıt/satın alma/demo akışı varmış gibi davranılmaz.
8. Footer: Mantıklı sütunlar, önemli sayfalara bağlantılar, yasal/gizlilik bağlantıları yalnız gerçek sayfalar hazır olduğunda; sosyal ikonlar yalnız aktif hesaplar varsa.
Her bileşen mobil, tablet ve masaüstünde tasarlanır. Hover tek etkileşim yöntemi olamaz; klavye ve dokunmatik kullanımı da desteklenir.

### 20.5 Home ve Business ayrımı — zorunlu karşılaştırma standardı
Homebrewers ve Professional Breweries aynı ürün markasını paylaşır; ancak ihtiyaçları, terminolojileri, karar kriterleri, içerik sıraları ve CTA'ları ayrı tasarlanır. Tek bir genel sayfada iki profili belirsizce karıştırmak kabul edilmez.

| Boyut | Homebrewers (Home) | Professional Breweries (Business) |
|---|---|---|
| Kullanıcı | Evde kendi ekipmanıyla bira/fermente içecek üreten kişi | Profesyonel bira üretim işletmesi ve üretim ekibi |
| Temel hedef | Reçete geliştirmek, demlemeyi planlamak, ölçmek ve sonuçları öğrenmek | Reçete/üretim standardı, planlama, kayıt, kalite ve parti izlenebilirliği |
| Dil | Açıklayıcı, öğretici, bireysel üretici odaklı | Operasyonel, teknik, ekip ve süreç odaklı |
| Öncelikli konular | Reçete, hesaplamalar, su, ekipman, brew day, fermantasyon | Reçete onayı, üretim planı, tank/parti, kalite, lot ve vardiya kayıtları |
| Değer anlatımı | Daha tutarlı ve anlaşılır ev üretim süreci | Daha kontrollü, izlenebilir ve ekipçe yürütülebilen üretim süreçleri |
| Kanıt standardı | Formül, varsayım, birim ve eğitim içeriği | Süreç, kayıt modeli, yetki ve izlenebilirlik tasarımının doğruluğu |
| CTA | Ürün erişimi gerçekten hazırsa Get started; değilse gerçek ve onaylı bekleme listesi/ürün güncellemesi | Business kapsamı ve erişim gerçekten hazırsa ilgili aksiyon; değilse dürüst bilgi/iletişim yolu |
| SEO niyeti | Home brewing, recipe calculator, water, ABV, IBU, fermentation | Brewery software, production planning, batch records, quality, traceability |

Karşılaştırma tabloları sayfa içinde görünür olmalı. Özellikle Home vs Business, planned capability vs current availability, manual workflow vs Demleme'nin planlanan yaklaşımı gibi farklar metin içinde kaybolmamalıdır. Tablo kullanımı mobilde yatay kaydırma veya anlamlı stacked layout ile erişilebilir olmalıdır.
Her özellik ve use case için iç değerlendirme tablosu şu alanları içermelidir: audience, kullanıcı problemi, beklenen girdi, iş akışı, ürün planındaki karşılık, roadmap durumu, kanıt/kaynak, site sayfası ve CTA. Home ve Business için aynı özellik farklı anlam taşıyorsa iki ayrı anlatım hazırlanmalıdır.

### 20.6 Bilgi kalitesi ve anlatım sırası
FoamByte karşılaştırması yalnız özellik sayısı üzerinden yapılmayacak. Her rakip sayfası ve Demleme sayfası şu eksenlerde değerlendirilecektir:

| Değerlendirme alanı | Kontrol sorusu |
|---|---|
| İçerik kapsamı | Konunun temel ve ileri seviye soruları yanıtlanıyor mu? |
| Bilgi doğruluğu | Formül, teknik terim, birim ve iddialar güvenilir kaynaklarla doğrulanmış mı? |
| Anlatım | Ön bilgi gerektirmeden anlaşılır, doğru sırada ve örnekli mi? |
| Bilgi akışı | Problem → bağlam → yöntem → örnek → sınırlama → ürün bağlantısı sırası mantıklı mı? |
| Kitle uyumu | Home ve Business ihtiyaçları ayrılmış mı; jargon doğru seviyede mi? |
| Görsel/UX | Sayfa taranabilir mi; başlık, tablo, diyagram ve boşluklar bilgiyi destekliyor mu? |
| Etkileşim | Hesaplayıcı veya örnek gerçekten çalışıyor ve doğrulanmış mı? |
| SEO | Arama niyeti, başlık, snippet, dahili bağlantı ve konu kümesi uyumlu mu? |
| Güven | Kaynaklar, varsayımlar, ürün durumu ve sınırlamalar açık mı? |
| Dönüşüm | Ziyaretçiye bir sonraki anlamlı adım net biçimde gösteriliyor mu? |

Teknik anlatım için varsayılan yapı: Problem → neden önemli → temel kavramlar → yöntem/formül → adım adım örnek → varsayımlar ve sınırlar → Home/Business uygulaması → ilgili Demleme kabiliyeti ve roadmap durumu → kaynaklar/sonraki okuma.
Rakip incelemesinde her sayfa için URL, içerik başlıkları, tablo ve etkileşim örnekleri, kaynak kalitesi, güçlü/zayıf yönler, Demleme'ye uyarlama fırsatı ve kopyalanmaması gereken unsurlar kaydedilir. Sadece erişilebilen gerçek sayfalar değerlendirilir; rakip hakkında varsayım yazılmaz.

### 20.7 İçerik ve tasarım kabul kriterleri
Bir sayfa “tamamlandı” sayılmadan önce:
- Tek bir ana hedef kitle ve arama/kullanıcı niyeti açıkça tanımlanmış olmalı.
- H1 ve ilk ekran, sayfanın ne anlattığını ve kimin için olduğunu birkaç saniyede açıklamalı.
- Bölümler doğru bilgi sırasına sahip olmalı; tekrar ve boş pazarlama cümleleri temizlenmeli.
- Teknik iddialar ürün planı, birincil kaynak veya doğrulanmış testle desteklenmeli.
- Planned / in development / available ayrımı görünür olmalı.
- Gerekli yerde Home ve Business farkı tabloyla gösterilmeli.
- İlgili konu, özellik, use case ve Learn sayfalarına bağlar çalışmalı.
- CTA gerçek bir hedefe gitmeli.
- Mobil/tablet/desktop responsive düzen, klavye erişimi, focus state, kontrast, reduced motion ve semantik HTML kontrol edilmeli.
- SEO metadata ve dahili bağlantılar tamamlanmalı.
- Görsel QA yapılmadan “responsive QA tamamlandı” denmemeli.

### 20.8 Görsel doğruluk ve güven kuralları
- Gerçek müşteri/partner logoları, testimonial, yıldız, avatar, kullanıcı sayısı, başarı metriği veya case study uydurulmaz.
- Ürün arayüzü gibi görünen sahte dashboard, sahte cihaz bağlantısı veya gerçek olmayan entegrasyon akışı üretilmez.
- Soyut SVG/şema kullanılabilir; bunun gerçek ürün ekranı olmadığı açık olmalıdır.
- Kullanıcı fotoğraf sağlamadıkça fotoğraf, stok fotoğraf veya AI fotoğrafı eklenmez; mevcut slot kimlikleri ve değişim notları korunur.
- Her içerik iddiası kaynak/ürün-planı referansına bağlanır; roadmap dışı fikirler “research / under consideration” olarak etiketlenir.
- Demo CTA ve “Watch demo” yalnız gerçekten izlenebilir, doğru ürünü gösteren demo mevcutsa kullanılır.

## 21. Oturum devri ve devam etme protokolü
- Yeni oturumda kullanıcı “devam et” dediğinde ilk iş OTURUM.md, web-stratejisi/12-site-revizyonu-ve-uygulama-plani.md, web-stratejisi/README.md ve ilgili GitHub PR/branch/CI durumunu okumaktır.
- Kullanıcıdan bu belgelerde kayıtlı hedefleri, kararları, kısıtları veya önceki talimatları yeniden anlatması istenmez.
- “Devam et” komutu, bu plan için bir sonraki güvenli ve bağımlılıkları hazır iş paketini başlatma/ilerletme yetkisidir. Kullanıcı yeni bilgi veya açık bir bekleme koşulu verdiyse önce plan ve karar kaydı güncellenir; çelişki varsa daha yeni açık talimat geçerlidir.
- İlk devam adımında tüm siteyi tek seferde kodlamaya girişilmez. Önce mevcut repo ve açık PR durumu kontrol edilir; sonra WP-00'dan itibaren küçük, doğrulanabilir ve bağımsız işlere ayrılarak ilerlenir.
- Her oturum sonunda plan ve OTURUM.md güncellenir: son tamamlanan iş, commit/PR, doğrulama, engeller ve tek net sonraki adım.
- Kullanıcıya aynı bağlam tekrar tekrar sorulmaz; yalnızca gerçekten yeni ve zorunlu bir karar eksikse tek net soru sorulur.
- GitHub Auto-merge kullanılmaz. Manuel merge öncesi CI, Vercel Preview, mergeability ve beklenen head SHA doğrulanır.

## 22. Tasarım ve uygulama iş paketlerine eklenen görevler
Mevcut WP-00—WP-17 iş paketleri bu yeni standarda göre genişletilir:
- WP-02: FoamByte ve diğer rakiplerde içerik + tasarım + bilgi kalitesi + bilgi akışı + iki audience ayrımı için karşılaştırmalı skor kartı.
- WP-04: Home/Business ayrı sitemap, gezinme ve sayfa ilişkileri; gerekli karşılaştırma tabloları.
- WP-05: Her sayfa için problem–yöntem–örnek–sınırlama–kanıt–ürün bağlantısı içerik şablonu.
- WP-06: Desktop/tablet/mobile wireframe; navbar, hero, bento, etkileşim, CTA ve footer varyantları.
- WP-07: Açık/koyu tema, semantic design tokens, tipografi modular scale, spacing, grid, motion ve component states.
- WP-08: Home ve Business için ayrı How It Works akışları ve yan yana fark tabloları.
- WP-11: Learn içerikleri için kaynak/kanıt standardı, teknik örnekler, glossary ve FAQ.
- WP-12: Sayfa başına audience/search intent ve konu kümeleriyle teknik SEO.
- WP-13—WP-16: Responsive/accessible component implementation, content integrity, visual regression ve gerçek cihaz/tarayıcı QA.
- Her WP'nin tamamlanma tanımı 20.7 kabul kriterlerini karşılamalıdır.
