# Demleme — Pazarlama Sitesi Revizyon Planı

**Durum:** PLANLAMA — uygulamaya başlanmayacak.  
**Onay kapısı:** Kullanıcı açıkça “başla” demeden tasarım, kodlama, içerik üretimi, sayfa oluşturma veya mevcut sitenin revizyonu yapılmaz.  
**Plan sahibi:** Demleme web stratejisi  
**Referans:** `plan/demleme-plani.md`, `web-stratejisi/`, `karsilastirma/`

---

## 1. Amaç ve kapsam

Mevcut pazarlama sitesini parça parça yamamak yerine, kullanıcıdan gelecek ek girdiler de toplandıktan sonra bütünsel olarak yeniden ele almak. Hedef; Demleme'yi yalnızca özellik listesi sunan bir ürün sayfası olmaktan çıkarıp, ziyaretçiye gerçek demleme problemlerini, ürünün yaklaşımını ve teknik güvenilirliğini anlatan; organik trafik ve self-service edinim sağlayabilecek bir kaynağa dönüştürmek.

### Sabit stratejik kararlar

- İki hedef kitle: **Homebrewers (Home)** ve **Professional Breweries (Business)**.
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
| Home | Demleme'nin ne olduğunu anlatmak, iki kitleyi doğru yola yönlendirmek | P0 |
| Product / Features | Onaylı ürün yol haritasını alanlara göre anlatmak | P0 |
| Homebrewers | Ev biracısının işleri, problemleri ve planlanan Home modu | P0 |
| Breweries | Profesyonel üretim ekibinin işleri ve Business modu | P0 |
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
2. “Başla” onayı alındığında WP-00 ile girdileri ve açık kararları dondur.
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

**Önemli:** Bu doküman planlama kaydıdır; kendiliğinden uygulama izni değildir. Kullanıcının açık “başla” talimatı olmadan hiçbir iş paketi uygulanmayacaktır.
