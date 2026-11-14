# Aktif çalışma oturumu

**Son güncelleme:** 2026-11-14  
**Amaç:** Yeni ChatGPT veya Claude oturumunun bağlamı yeniden araştırmadan devralması.  
**Kural:** Bu dosya kısa tutulur. Her anlamlı iş tamamlandığında ve oturum kapatılırken güncellenir.


## Güncel yayın durumu — 14 Kasım 2026

- **Hosting:** Vercel aktif preview/dağıtım hedefi; GitHub Pages yedek yayın. Ticari/public launch henüz yapılmadı.
- **Vercel adresi:** https://brewery-rho.vercel.app (mevcut site preview/deployment adresi).
- **Ana dalın ürün kataloğu öncesi son commit'i:** `5661692327cac99960955bcf0730e68326ebcec0`.
- **Fotoğraflar:** Kullanıcı henüz fotoğraf sağlamadı. Yeni fotoğraf üretme, stok fotoğraf seçme veya otomatik görsel ekleme. Beş `data-image-slot` alanı ve SVG/yer tutucular korunacak; yalnız kullanıcı varlığı/izniyle değiştirilecek.
- **Görsel QA:** Gerçek masaüstü/mobil tarayıcı QA henüz tamamlanmış değil. CI build başarısı görsel QA yerine geçmez.

## Çalışma ilkeleri

- Token verimliliği oturum başında uygulanır; haftalık raporu bekleme.
- Basit, mekanik, düşük riskli işler için Haiku veya erişilebilen en küçük uygun model kullan.
- Açık tanımlı kodlama, test ve dokümantasyon işleri için Sonnet kullan.
- Mimari, veri modeli, güvenlik, zor hata analizi, ürün kararı ve son kritik inceleme için Opus kullan.
- Model adı ve erişilebilirliği araca göre değişebilir. Claude ana oturum modelini kendisi değiştiremez; alt ajan için uygun modeli seç. ChatGPT tarafında da mevcut model seçenekleri arasından işin gerektirdiği en düşük yeterli modeli kullan.
- Bir görev için yalnız gerekli dosyaları oku. Önce arama veya ilgili başlık aralığı; tam dosyayı yalnız gerekirse oku.
- Büyük test çıktısını özetle veya filtrele. Alt ajan raporu en fazla 20 satır olsun.
- Birbirine bağlı değişiklikleri aynı ajanda zincirle. Yalnız bağımsız işler paralel yürüsün; aynı anda en fazla 2–3 ajan.
- Küçük model işi tamamlayamazsa yalnız o işi bir üst modele yükselt. Her işi varsayılan olarak güçlü modele verme.
- Token veya maliyet telemetrisi görünmüyorsa sayı ya da tasarruf yüzdesi uydurma.
- İş bitince değişen dosyaları, test/ doğrulama sonucunu ve sıradaki adımı kısa raporla.

## Aktif iş parçacığı: Pazarlama web sitesi

**Hedef:** Demleme ürününü tanıtan ve self-service edinim sağlayan pazarlama sitesi. Web sitesi ürün uygulamasının kendisi değildir.

**Kararlar**
- Hedef kitle: iki ayrı yol — Homebrewers (Home) ve Professional Breweries (Business).
- Konumlandırma: technical.
- İlk pazarlar: ABD ve İngiltere. Ana dil: İngilizce.
- Ticari yaklaşım: self-service öncelikli; demo ana CTA değildir.
- Ürün ERP değildir. Satış, sipariş, sevkiyat, fatura, muhasebe, CRM veya keg takibi vaat edilmez.
- Ürün kabiliyeti doğrulanmadan özellik veya sonuç iddiası yayınlanmaz.

**Mevcut kayıt**
- `web-stratejisi/README.md`, `01-stratejik-brief.md`–`11-visual-design-wireframes.md` dosyaları GitHub'da mevcut. Astro + TypeScript + SSG mimarisi kullanıcı tarafından onaylandı.
- Strateji dosyaları iki audience brief'i, marka/mesaj, sitemap, görsel tasarım/UX, dönüşüm, SEO, lansman ölçümü ve teknik mimari önerisini kapsar.
- Bu kayıtlar strateji ve mimari öneri dokümanlarıdır; çalışan pazarlama sitesi veya tamamlanmış görsel tasarım değildir.

**Onaylanan kararlar**
- Framework: Astro 7.x (kurulum anındaki güncel stable), TypeScript strict, SSG.
- UI: Astro components; React yalnız gerçekten gerekli interaktif island için.
- İçerik: Markdown/MDX + Content Collections/Zod.
- Hosting adayı: Cloudflare Pages. Domain, analytics, form ve consent sağlayıcısı hâlâ açık karar.
- GitHub: pazarlama sitesi değişiklikleri feature branch + PR; kullanıcı talimatıyla CI ve preview doğrulamasından sonra manuel merge yapılır. GitHub Auto-merge kullanılmaz.

**Tasarım yönü**
- Editorial + Technical + Human. AI landing page klişeleri, sahte dashboard, uydurma metrik ve gradient/glow yok.
- Gerçek ürün ekranı yalnız mevcut uygulamadan; gerçekçi homebrew ve brewery fotoğrafları.
- Ana sayfa, Homebrewers ve Professional Breweries için desktop + mobile wireframe yazıldı: `web-stratejisi/11-visual-design-wireframes.md`.
- Özellik iddiaları ürün akışı/test/dokümanıyla doğrulanmadan yayınlanmaz.

**Sonraki adımlar**
1. Ana sayfa ve iki audience landing page'i 1440px desktop / 390px mobile ölçülerinde görsel olarak gözden geçir.
2. Fotoğrafları, aşağıdaki slot envanterine göre lisans ve son varlıklar onaylandığında değiştir.
3. App URL, sign-in URL, domain ve analytics/form kararlarını ayrıca al.
4. Fiyat, trial, paket, ödeme sağlayıcısı ve limitler kesinleşmeden fiyat/teklif vaat etme.
5. Claude ürün geliştirme kuyruğuna dokunma.

**Son çalışma — 14 Kasım 2026**
- `marketing/homepage-prototype` dalı `main` üzerinden açıldı.
- `marketing-site/` altında izole Astro + TypeScript + SSG homepage prototipi eklendi.
- Desktop/mobile düzen, audience yolları, teknik mesaj bölümü ve erişilebilir mobil menü hazırlandı.
- Görsel öğeler özgün SVG illüstrasyonlarıdır; gerçek ürün ekranı veya ürün işlevi taklidi değildir.
- Mevcut tasarım token'ları ve sistem font yığını kullanıldı.
- GitHub üzerinden dosya ve değişiklik kaydı doğrulandı. Yerel `npm install`, `astro check` ve production build henüz çalıştırılmadı.
- Yalnız `marketing-site/` değişikliklerinde çalışan GitHub Actions workflow eklendi: Astro check + SSG build. İlk PR workflow sonucu bekleniyor.
- Gerçek fotoğraf varlıkları, ürün ekranları ve uygulama CTA URL'leri bekliyor.

**Sınır:** Bu pazarlama iş parçacığı `gelistirme/DURUM.md`, `gelistirme/ISLER.md`, ürün kodu, API, Claude geliştirme sırası ve commit'lerini değiştirmez.

## Ayrı iş parçacığı: Ürün geliştirme

- Aktif aşama: Stage 2 — core API and data.
- Son bilinen sıradaki işler: `gelistirme/DURUM.md`'deki T2.19 → T2.21 → T2.22 zincirini tek ajana ver; T2.20 bağımsızsa paralel yürüyebilir.
- Geliştirme işine devam etme talimatı gelirse önce `gelistirme/DURUM.md`'yi oku; `gelistirme/ISLER.md` içinden yalnız aktif aşama ve ilgili bağımlılıkları incele.
- Ürün geliştirme sırasını bu dosyadan değiştirme. Asıl iş durumu `gelistirme/DURUM.md` ve `gelistirme/ISLER.md)'dir.

## Oturum kapatma / devir

Her anlamlı işten sonra bu dosyayı güncelle:
- tamamlanan iş ve doğrulanan dosyalar;
- değişikliklerin test veya kontrol sonucu;
- açık kararlar ve engeller;
- tek bir net sıradaki adım.
- Tamamlanmayan işi tamamlandı gibi yazma. Bilinmeyen token tüketimini tahmin etme.


## Web framework araştırması — 14 Kasım 2026

**Tamamlandı:** Resmî kaynaklarla Astro, Next.js App Router ve SvelteKit karşılaştırıldı. Güncel Astro 7.4 bilgisi ve Cloudflare Pages GitHub preview akışı kontrol edildi.

**Önerilen mimari:** Astro 7.x + TypeScript strict/strictest + SSG + Markdown/MDX Content Collections/Zod + CSS token'ları + Cloudflare Pages. İlk sürümde backend/database/CMS yok; Astro components varsayılan, React yalnız gerekirse island.

**Gerekçe:** Pazarlama sitesi içerik ve SEO ağırlıklı. Astro bu ihtiyaca doğrudan odaklanıyor. Next.js static export yapabilir ama ihtiyaç olmayan server/runtime özelliklerini de getirir; SvelteKit statik üretim yapabilir ancak mevcut React/TypeScript web ürününden farklı UI ekosistemi oluşturur.

**Kaydedilen teknik doküman:** `web-stratejisi/10-teknik-mimari.md` — framework karşılaştırması, TypeScript/AI kodlama ilkeleri, repo yerleşimi, rendering, içerik/SEO, tasarım token'ları, hosting, CI/CD, kalite kapıları ve Mermaid sistem akışı.

**Sıradaki adım:** Mimari tercihi esas alarak yalnız pazarlama sitesi için bilgi mimarisi ve desktop/mobile wireframe taslağı hazırlamak. Henüz `marketing-site/` kod iskeleti oluşturulmadı. `web/` ürün uygulaması ve Claude geliştirme kuyruğu değiştirilmedi.


**Son çalışma — 14 Kasım 2026 (devam)**

- Kullanıcı talimatıyla fotoğraflar sonraki aşamaya bırakıldı; mevcut illüstrasyonlar ve yer tutucular geçici olarak korundu.
- Ana sayfada fotoğraf değişim noktaları HTML yorumları ve `data-image-slot` kimlikleriyle işaretlendi.
- `/homebrewers` ve `/breweries` için ayrı İngilizce audience landing page'leri eklendi.
- `marketing-site/README.md` içine beş görsel slotunun dosya/kimlik/değiştirilecek içerik envanteri eklendi.
- Değişen dosyalar: `marketing-site/src/pages/index.astro`, `homebrewers.astro`, `breweries.astro`, `src/styles/global.css`, `README.md`.
- GitHub dosya yazma işlemleri başarılı. GitHub Actions üzerinden `npm install`, `astro check` ve `astro build` başarılı tamamlandı. PR #3 draft olarak bırakıldı; merge yapılmadı.
- Ürün uygulaması, `web/` ve Claude ürün geliştirme kuyruğu değiştirilmedi.


**Son çalışma — 14 Kasım 2026 (bağlantı ve mesaj denetimi)**
- Ana sayfa kartları, hero CTA'ları ve navigasyon Homebrewers / Breweries sayfalarına yönlendirilecek şekilde düzeltildi.
- Audience metinleri kesinleşmemiş ürün fonksiyonlarını vaat etmek yerine teknik yaklaşım ve odak alanlarını anlatacak şekilde yumuşatıldı.
- Fotoğraf yer tutucusu arka planı düz renge çevrildi; görsel slot kimlikleri korundu.
- README kapsamı iki ayrı audience landing page'ini ve güncel CTA davranışını yansıtacak şekilde güncellendi.
- GitHub üzerinden üç sayfanın içerikleri ve navigasyon hedefleri incelendi. Astro check ve SSG production build GitHub Actions run #11 içinde başarıyla tamamlandı.
- CI adımları: Node 22 kurulumu, dependency install, `npm run check`, `npm run build` — tümü success.
- Gerçek tarayıcıda 1440/390 görsel kontrolü henüz yapılmadı; deploy edilmiş preview URL yok.
- PR #3 draft; merge edilmedi. Kod değişiklikleri test edildi; sonrasında yalnız OTURUM dokümantasyonu güncellendi.
- **Sıradaki adım:** Cloudflare Pages preview/deploy bağlantısı mevcutsa gerçek responsive QA; yoksa kullanıcıdan hosting/preview erişimi için karar al. Fotoğrafları ve app CTA URL'lerini beklet.


**Son çalışma — 14 Kasım 2026 (CI ve statik bağlantı QA)**
- GitHub Actions run #11 başarıyla tamamlandı: Node kurulumu, dependency install, npm run check (Astro/TypeScript) ve npm run build adımlarının tamamı başarılı.
- Ana sayfa ve iki audience sayfasının dahili anchor hedefleri kaynak üzerinden denetlendi; eksik anchor bulunmadı. Audience sayfaları için /, /homebrewers, /breweries ve ana sayfa section linkleri tutarlı.
- PR #3 açık, draft ve merge edilmedi. Head branch marketing/homepage-prototype; base main.
- Bu oturumda gerçek tarayıcı / cihaz önizlemesi sağlayan bir araç yok; 1440px ve 390px görsel QA yapılmış sayılmıyor. GitHub CI başarısı görsel doğrulama yerine geçmez.
- **Sıradaki adım:** preview ortamı/erişimi sağlandığında masaüstü ve mobil görsel QA; ardından CTA hedefleri ve production hosting/domain kararları.


**CI doğrulaması — 14 Kasım 2026 (son kontrol)**
- Marketing site workflow run #11 ve #12: **success**.
- Tamamlanan adımlar: Node setup, npm dependency install, Astro and TypeScript checks, static site build.
- Kaynak kontrolü: homepage anchor hedefleri geçerli; /homebrewers ve /breweries route bağlantıları doğru.
- Branch diff: 26 commit ahead, 0 behind; PR #3 açık, draft ve mergeable.
- Görsel QA için canlı preview/browser henüz yok. Bu nedenle 1440px / 390px ekranlarda görsel doğrulama yapılmış gibi raporlanmamalı.



**GitHub Pages preview hazırlığı — 14 Kasım 2026**
- Kullanıcı GitHub Pages ile devam edilmesini onayladı. Repository metadata kontrolünde `has_pages: false`; Pages henüz etkin değil.
- Astro config `site: https://oguzbay-del.github.io` ve `base: /brewery` olarak ayarlandı.
- Ana sayfa ve iki audience sayfasındaki iç linkler Astro `BASE_URL` üzerinden repo alt dizinine uyumlu hale getirildi.
- `.github/workflows/marketing-site-pages.yml` eklendi: branch push / workflow_dispatch → Astro check + build → Pages artifact → deploy. Mevcut PR check workflow'u ayrı bırakıldı.
- Hedef preview URL: `https://oguzbay-del.github.io/brewery/` (Pages etkinleştirilip deployment başarılı olunca erişilebilir).
- Repository sahibi GitHub Pages'i etkinleştirdi; kaynak GitHub Actions olarak ayarlandı (kullanıcı teyidi).
- Bu aşamada main merge edilmedi; `web/`, ürün API'leri ve Claude ürün geliştirme kuyruğu değiştirilmedi.
- **Sıradaki adım:** yeni branch push sonrası Pages deployment sonucunu ve canlı URL'yi doğrula; ardından 1440px/390px görsel QA yap.


**Pages workflow sonucu — 14 Kasım 2026**
- Pages workflow ilk denemesinde build başarılı, deploy ise Pages etkin olmadığı için 404 ile başarısız olmuştu.
- Kullanıcı Pages'i etkinleştirdiğini teyit etti. README'deki preview notu güncellendi; bu commit deployment'ı yeniden tetiklemeli.
- PR #3 son kontrolünde Marketing site checks ve genel altyapı workflow success; PR hâlâ draft ve açık.
- Preview canlı olduktan sonra masaüstü/mobil görsel QA yapılacak. Görsel QA tamamlanmadan PR merge edilmeyecek.
- Kullanıcının açık talimatı: Auto-merge kullanma; CI ve preview QA tamamlanınca PR'ı manuel merge et.


**Pages yayını ve PR tamamlandı — 14 Kasım 2026**
- PR #3 `Marketing: responsive Demleme homepage prototype` kullanıcı talimatına uygun şekilde Auto-merge olmadan manuel squash merge edildi.
- Merge commit: `b360545f931b8dcef17f7a97efed88f7b5f90e31`.
- GitHub Pages deployment workflow, feature branch deploy ortamı hatasından kaçınmak için yalnızca `main` push'larında çalışacak şekilde düzenlendi. `actions/configure-pages@v5` eklendi.
- Main üzerindeki Pages workflow run #43629069191: **success**. Configure Pages, Astro/TypeScript check, static build, artifact upload ve deploy adımları başarılı.
- Marketing site checks main run #43629069151: **success**.
- Yayın adresi: https://oguzbay-del.github.io/brewery/
- PR #3 kapatıldı/merge edildi; GitHub Auto-merge kullanılmadı.
- Değişiklik kapsamı: yalnız `.github/workflows/marketing-site*.yml`, `marketing-site/` ve oturum dokümantasyonu. `web/`, ürün API'leri ve Claude ürün geliştirme kuyruğu değiştirilmedi.
- **Açık QA:** Yayın başarılı; ancak bu oturumdaki browser inspection aracı siteyi açamadığı için 1440px desktop / 390px mobile görsel QA tamamlandı diye raporlanmıyor. Beş geçici görsel slotu lisanslı fotoğraflar gelene kadar korunuyor.
- Ayrı altyapı workflow'u merge sonrasında `cancelled` olarak sonuçlandı; marketing CI ve Pages deploy başarıyla tamamlandı. Altyapı dosyaları bu PR kapsamında değiştirilmedi.
- **Sıradaki adım:** Canlı siteyi gerçek tarayıcıda desktop/mobile kontrol et; fotoğraf ve onaylı app/sign-in URL'leri gelene kadar placeholder ve CTA'ları değiştirme.


**SEO ve taranabilirlik iyileştirmesi — 14 Kasım 2026**
- PR #4 manuel squash merge edildi; Auto-merge kullanılmadı.
- Merge commit: `63cce6f4c56ed2e5bbee44211b689a677c77e269`.
- Ana sayfa, Homebrewers ve Breweries sayfalarına canonical URL, Open Graph ve Twitter metadata eklendi.
- SVG favicon, üç sayfalı sitemap.xml ve sitemap referanslı robots.txt eklendi.
- PR marketing site Astro/TypeScript check ve static build: **success**.
- Main push sonrası marketing site checks run #43629643512: **success**. Pages deployment run #43629643581: **success** (configure, Astro/TypeScript check, build, artifact upload ve deploy).
- Canlı URL aynı: https://oguzbay-del.github.io/brewery/
- Browser-based 1440px/390px görsel QA hâlâ tamamlanmadı; ağ erişimi bu çalışma ortamında alan adına erişemedi.
- Değişiklik yalnız `marketing-site/` içeriğinde; ürün uygulaması, API ve ürün geliştirme kuyruğuna dokunulmadı.
- **Sıradaki adım:** yeni Pages deploy'unu doğrula; ardından gerçek cihaz/tarayıcı üzerinde responsive görsel QA. Fotoğraflar ve onaylı app/sign-in URL'leri bekleniyor.

**Audience sayfaları — CTA ve gezinme iyileştirmesi (14 Kasım 2026)**
- Homebrewers ve Breweries sayfalarının birincil CTA'ları artık ilgili sayfadaki içerik odağına gider; kullanıcıyı ana sayfaya geri döndürmez.
- Her iki sayfanın kapanış CTA'sı “Explore both brewing paths” olarak netleştirildi; ana sayfaya dönüş hedefi korundu.
- Değişiklikler yalnız pazarlama sayfalarında. Ürün özelliği, fiyat, kayıt veya app URL'si uydurulmadı.
- Branch: `marketing/audience-journey-polish`. CI ve manuel PR merge bekleniyor; görsel QA ayrı açık konu.

**CTA iyileştirmesi yayınlandı — 14 Kasım 2026**
- PR #5 manuel squash merge edildi; Auto-merge kullanılmadı. Merge commit: `919bf450453505fbc01bbc0a2938955c3012004c`.
- PR marketing CI #43630573120: **success** (dependency install, Astro/TypeScript check, static build).
- Main marketing CI #43630647610: **success**.
- Main Pages deploy #43630647549: **success**. Canlı adres: https://oguzbay-del.github.io/brewery/.
- Değişiklikler Homebrewers/Breweries CTA hedeflerini iyileştirdi; ürün iddiası veya uygulama bağlantısı eklenmedi.
- Gerçek tarayıcıda responsive görsel QA hâlâ açık; bu ortamda alan adına erişim kısıtı nedeniyle tamamlanmadı.
- Sıradaki iş: sayfaların gerçek 1440px desktop / 390px mobile görsel QA'sı. Fotoğraflar ve onaylı app/sign-in URL'leri bekleniyor.

**Ürün olgunluğu ve pazarlama iddiaları — 14 Kasım 2026**
- Ürün deposu read-only incelendi: `web/src/App.tsx` ve iOS `ContentView.swift` hâlâ başlangıç ekranı iskeleti; web istemcisinde çalışan ürün akışı görünmüyor.
- Hesap motorunda recipe, mash, fermentation, brewery, yeast ve water gRPC servisleri mevcut; ancak API modül kaydı şu an health/meta, events, files ve auth modüllerini içeriyor. API `/v1/version` yanıtında engine sürümü de henüz `null`.
- Bu nedenle motorun hesaplama kodu yazılmış olsa da bunu ziyaretçinin bugün kullanabileceği uygulama özelliği gibi pazarlamak doğru değil.
- Pazarlama ana sayfası ve iki audience sayfasına görünür “in development” durumu eklendi; çalışma alanı/araç varmış izlenimi veren ifadeler yumuşatıldı. Audience maddeleri “design priorities” olarak etiketlendi ve kullanılabilir özellik listesi olmadığı belirtildi.
- Ürün kodu, API, `web/`, iOS ve geliştirme kuyruğu değiştirilmedi. Yalnız `marketing-site/` ve bu oturum notu değişiyor.

**Ürün olgunluğu mesajı yayında — 14 Kasım 2026**
- PR #6 manuel squash merge edildi; Auto-merge kullanılmadı. Merge commit: `87c36c944dae088da6b18a52f27e375d3ed13bfd`.
- PR CI #43631065843: **success** (son head: `cb2c0e2c9a25107dbf53136b1c807e5cddec2f6a`).
- Main marketing checks #43631137613: **success**. Pages deploy #43631137672: **success**.
- Ana sayfa ve iki audience sayfası artık ürünün geliştirme aşamasında olduğunu açıkça belirtiyor. Audience maddeleri kullanılabilir özellik iddiası değil, tasarım önceliği olarak etiketli.
- Yayın: https://oguzbay-del.github.io/brewery/
- Görsel tarayıcı QA (1440px / 390px) hâlâ açık; gerçek cihaz/browser erişimi gerekir.
- Sıradaki adım: erişilebilir bir tarayıcı önizlemesinde responsive QA; sonrasında ürün API ve istemci akışları gerçekten yayınlandıkça pazarlama iddialarını kademeli doğrulamak.

**Erişilebilir gezinme iyileştirmesi — 14 Kasım 2026**
- Üç sayfaya klavye kullanıcıları için görünür olduğunda açılan “Skip to main content” bağlantısı eklendi.
- Tüm ana içerik alanları `main-content` hedefi aldı; sayfa içi hedeflerde kaydırma boşluğu tanımlandı.
- Mobil menü özet düğmesinin erişilebilir adı “Toggle navigation menu” olarak düzeltildi.
- Mevcut görünüm korunarak yalnız gezinme/erişilebilirlik katmanı değiştirildi. Gerçek ekran okuyucu ve cihaz testi henüz yapılmadı.

**Klavye gezinmesi erişilebilirliği yayında — 14 Kasım 2026**
- PR #7 manuel squash merge edildi; Auto-merge kullanılmadı. Merge commit: `7728f64a88eef1f53233b8a523e867d46a9d705e`.
- PR CI #43631421615: **success**. Main CI #43631476299: **success**. GitHub Pages deploy #43631476340: **success**.
- Üç pazarlama sayfasına klavye ile görünür skip link eklendi; main landmark hedefleri tanımlandı; mobil menü erişilebilir adı iyileştirildi.
- Ekran okuyucu ve gerçek cihaz testi henüz yapılmadı; gerçek tarayıcı QA açık.
- Canlı site: https://oguzbay-del.github.io/brewery/.

**Özel 404 sayfası — 14 Kasım 2026**
- GitHub Pages için markalı `404.html` üretilecek Astro `404.astro` sayfası eklendi.
- Geçersiz/eski bağlantılardan ana sayfaya, Homebrewers veya Breweries sayfasına dönüş sağlandı.
- Sayfa `noindex, follow` ile arama indeksinden hariç tutuldu; erişilebilir skip link içeriyor.
- Sitemap'e 404 dahil edilmedi. Ürün veya pazarlama iddiaları değişmedi.

**Markalı 404 sayfası yayında — 14 Kasım 2026**
- PR #8 manuel squash merge edildi; Auto-merge kullanılmadı. Merge commit: `6fe86aee64bc57d81b846322bc3c30282aeab9e0`.
- PR CI #43631631449: **success**. Main CI #43631699802: **success**. GitHub Pages deploy #43631699807: **success**.
- Astro `404.astro` sayfası Pages için özel 404 çıktısı üretir; ana sayfa ve iki audience yoluna dönüş bağlantısı içerir. `noindex, follow` kullanır ve sitemap'e eklenmez.
- Canlı site: https://oguzbay-del.github.io/brewery/.

**Alt sayfa gezinme hedefleri düzeltildi — 14 Kasım 2026**
- Ana sayfanın logo ve “Back to top” bağlantılarının hedefi olan `#top` kimliği header'a eklendi.
- Homebrewers/Breweries iç bağlantıları tutarlı trailing-slash biçimine alındı; Astro `trailingSlash: "always"` ile aynı URL standardına sabitlendi.
- Bu düzeltme, 404 sayfasındaki zaten slash kullanan hedeflerle URL tutarlılığı sağlar. Canlı tarayıcı/HTTP testi ortam erişimi nedeniyle ayrıca doğrulanamadı.
- PR #9 manuel squash merge edildi; Auto-merge kullanılmadı. Merge commit: `7a6ef089eb57c7941ea0a3e6fe5509f553953374`.
- PR CI #43632140191, main CI #43632202886 ve Pages deploy #43632202813: **success**.
- Canlı Pages deployment tamamlandı. Ancak bu ortamın tarayıcı erişimi alan adını açamadığı için her bağlantı için gerçek HTTP/cihaz testi yapıldığı iddia edilmemelidir.


**SEO ve yapılandırılmış veri — 14 Kasım 2026**
- Homepage'e `WebSite`, Homebrewers ve Breweries sayfalarına canonical URL'leri kullanan `WebPage` JSON-LD eklendi.
- PR #10 CI run #43632635360 başarıyla tamamlandı; head SHA `2e137a9568aacdf05c4ebf90944ff0e9b6765fac` doğrulandıktan sonra Auto-merge olmadan manuel squash merge edildi.
- Merge commit / main HEAD: `667a8554f2161693a3472eef548ade8e6b14d8b9`. Post-merge main CI ve Pages durumu bu oturumda doğrulanamadı.
- Gerçek HTTP/browser kontrolü DNS erişimi olmadığı için yapılamadı; görsel QA tamamlandı olarak raporlanmıyor.

**Vercel geçiş hazırlığı — 14 Kasım 2026**
- Ayrı branch: `hosting/vercel-migration`. Astro config Vercel'de root base, GitHub Pages fallback'inde `/brewery` kullanacak şekilde ayrıştırıldı.
- Sitemap ve robots.txt Astro endpoint'lerine taşındı; aktif host ve base path'ten build-time üretilir. Pages workflow `GITHUB_PAGES=true` ile fallback davranışını korur.
- `marketing-site/README.md` içine Vercel import ayarları, root directory, build/output ayarları ve launch QA checklist eklendi.
- Vercel hesabı kullanıcı tarafından açıldı; proje henüz import edilmedi. Hobby plan ticari yayın için uygun olmadığından Pro yükseltmesi canlıya geçmeden önce gerekli.
- Bu değişiklikler henüz PR/CI ile doğrulanmadı. GitHub Pages kapatılmayacak; Vercel Preview ve production domain/HTTPS/SEO kontrolleri tamamlanana kadar fallback olarak kalacak.


**Ürün kapsamı pazarlama sitesine aktarılıyor — 14 Kasım 2026**
- Kanonik ürün kaynağı `plan/demleme-plani.md` okundu; iki modlu ürün yapısı, Ev MVP, İşletme çekirdeği ve sonraki sürümler ayrıştırıldı.
- Marketing site için `src/data/features.ts` ortak özellik kataloğu eklendi. 16 ürün alanı ayrı detay sayfalarıyla yayınlanacak: recipe design, brewing calculations, water/mash/yeast, brew day, fermentation, equipment/calibration, planning, inventory/costing, production management, quality/lab, packaging/compliance, teams/notifications, connected devices, imports/exports/tools, platform/security/data, analytics/reports .
- `/features/` ürün kataloğu; `/features/[slug]/` her alan için ayrı detay sayfası. Ana sayfa ve Homebrewers/Breweries landing page'leri katalogla ilişkilendirildi.
- Sitemap, feature data kaynağından bütün detay sayfalarını otomatik ekleyecek şekilde güncellendi.
- Her detay sayfası audience ve geliştirme aşaması etiketlerini taşır. Araştırma/fikir aşamasındaki analitik modeller committed feature gibi sunulmaz. ERP kapsam dışı alanları (satış, sipariş, dağıtım, keg tracking, CRM, muhasebe) ürün vaadine eklenmez.
- Ürün ve site henüz live değil: tüm metinler roadmap/planned language kullanır. Fotoğraf eklenmedi; mevcut görsel yer tutucular korunur.
- Yalnız `marketing-site/` ve `OTURUM.md` değiştirildi; `web/`, `ios/`, sunucu, API ve Claude ürün geliştirme kuyruğu değiştirilmedi.
- **Sıradaki adım:** Astro check + build, route/sitemap kontrolü; sonra PR ve CI. Browser/mobile görsel QA ayrıca yapılacak.

## Demleme pazarlama sitesi — kalıcı devam talimatı (14 Kasım 2026)

- Ana proje kitabı: web-stratejisi/12-site-revizyonu-ve-uygulama-plani.md. Yeni oturumda önce bu dosyayı ve bu OTURUM.md kaydını oku; kullanıcıdan daha önce verilmiş kararları tekrar isteme.
- Kullanıcı yeni oturumda “devam et” dediğinde son GitHub durumunu (main, açık PR'lar, branch head, CI, Vercel Preview) kontrol et ve planın sıradaki güvenli iş paketini başlat/ilerlet. Her defasında aynı strateji açıklamasını tekrar etme.
- Site bütünsel revize edilecek. FoamByte karşılaştırması; içerik, tasarım, bilgi kalitesi, bilgi akışı, teknik doğruluk ve Home/Business profillerinin ayrı anlatımı üzerinden yapılacak. Home vs Business farkları gerektiğinde net karşılaştırma tablolarıyla gösterilecek.
- Tasarım çıtası Apple/Vercel/Stripe kalitesi; minimalist, modern, teknik, güven veren, bol boşluklu, responsive ve erişilebilir. Bento/blur/glass/gradient/shadow yalnız amaca hizmet ederse. Açık/koyu tema ve semantic design tokens değerlendirilecek. Örnek renk kodları onaylı marka renkleri değildir.
- Navbar, hero, kanıt alanı, özellik anlatımı, gerçek işlevli etkileşim, CTA ve footer sayfa amacına göre tasarlanır. Gerçek müşteri logoları/testimonial/metric yoksa sosyal kanıt bölümü uydurulmaz; gerekirse çıkarılır. Sahte ürün ekranı, fotoğraf, fiyat, entegrasyon veya işlev yok.
- Home ve Business ayrı kullanıcı yolculukları, dil, SEO, içerik sırası ve CTA ile tasarlanır. Teknik anlatım problem → bağlam → yöntem → örnek → sınırlama → ürün bağlantısı akışını izler.
- Token/model optimizasyonu: mekanik envanter ve tekrar işleri en küçük yeterli model/script; içerik taslakları ekonomik model + editoryal kontrol; mimari, teknik doğruluk, formül, güvenlik ve kritik kararlar güçlü model/uzman kontrolü. Aynı dosyaya paralel yazım yok; alt ajan raporları kısa.
- “Devam et” ile küçük, doğrulanabilir iş paketleri ilerletilir; açık karar veya kullanıcıdan beklenen girdi varsa önce plan güncellenir. GitHub Auto-merge yok; manuel merge için CI, Vercel Preview, mergeability ve expected head SHA kontrol edilir.
- Plan dosyası PR #17 içinde güncellendi. PR #17 Vercel deployment ve Preview Comments başarılı; güncel commit ve PR merge durumu devam oturumunda tekrar doğrulanmalı. PR #16 ayrıca açık; özellik kataloğu kapsamı içerir ve bu bütünsel revizyonla karıştırılmamalıdır.
- Bu oturumda site kodu/tasarımı değiştirilmedi; yalnız plan ve oturum devir belgeleri güncellendi.


## Devam oturumu — site envanteri ve FoamByte benchmark (14 Kasım 2026)

- PR #17 merged; main HEAD `8be1771c5cd68bfeca2e8e2b208d7a68241fab97`. Vercel repository status PR #16 head'i için success; PR #16 hâlâ açık ve eski main tabanlı. Doğrudan merge edilmedi; güncel main ile uyum/mergeability kontrolü gerekli.
- WP-01 kaynak kodu/repo envanteri tamamlandı. İncelenen route'lar: `/`, `/homebrewers/`, `/breweries/`, `/features/`, `/features/[slug]/`, `/404`, `/sitemap.xml`, `/robots.txt`.
- Mevcut artılar: Astro/TypeScript SSG, ayrı Home/Business landing page'leri, merkezi feature katalog datası, canonical/OG/Twitter/JSON-LD, native mobil menü, skip-link/focus-visible ve tanımlı fotoğraf slotları.
- WP-02 FoamByte desk research: homepage, features, calculators, pricing, FAQ ve 17 Sep 2026 site redesign yazısı incelendi. Güçlü alanlar: adlandırılmış feature anlatımı, 25+ calculator dizini, FAQ, pricing ve sosyal kanıt. Demleme için uyarlama: teknik doğruluk, formül/varsayım/kaynak açıklığı ve daha keskin Home/Business ayrımı; rakip metin/tasarım/kanıt kopyalanmayacak.
- Denetim raporu: `web-stratejisi/13-mevcut-site-ve-foambyte-denetimi.md` (audit branch / PR).
- Canlı Vercel site bu oturumda tarayıcı ile açılamadı. 1440px/tablet/390px visual QA, HTTP sitemap/robots/canonical testi ve ürün planı satır bazlı iddia denetimi tamamlandı sayılmıyor.
- Bu oturumda site kodu/tasarımı değiştirilmedi. Yalnız denetim dokümanı ve bu devir kaydı değiştirildi.
- **Sıradaki adım:** WP-03 — 16 feature sayfasındaki her iddiayı `plan/demleme-plani.md` ve roadmap aşamalarıyla eşleştir. PR #16'nın eski base/mergeability durumunu ayrıca çözmeden merge etme.


## WP-04 — Home / Business sitemap, user journeys ve CTA mimarisi (14 Kasım 2026)

- Rapor: `web-stratejisi/15-home-business-sitemap-user-journeys-cta.md`.
- Global sitemap ve önerilen ana navigasyon tanımlandı.
- Homebrewers ve Professional Breweries için ayrı ziyaretçi soruları, giriş sayfaları, içerik sırası, journey adımları ve use-case örnekleri yazıldı.
- İki profil; iş birimi, değer dili, veri örneği, teknik güven, CTA ve kaçınılacak vaatler bakımından karşılaştırıldı.
- CTA'lar ürün yaşam döngüsüne bağlandı: gerçek ürün/hesap akışı yokken Start free, trial, pricing, signup, purchase veya demo çağrısı kullanılmayacak. Waitlist/update ancak gerçek endpoint ve privacy/consent onayı varsa.
- Calculators ve Learn SEO girişleri olarak konumlandı; formül doğrulaması olmadan calculator yayınlanmayacak.
- Business sayfasında ERP dışı kapsam (sales, orders, distribution/shipping, keg tracking, CRM, accounting) açıkça korunuyor.
- PR19/PR16 durumları bu WP branch'inde değiştirilmedi; site kodu ve ürün uygulaması değiştirilmedi.
- **Sıradaki adım:** WP-05 — sayfa bazında mesaj mimarisi, value proposition/headline seçenekleri, içerik brief'leri, proof requirements, terminoloji ve SEO intent. Final production copy ve canlı site değişikliği henüz yok.


## WP-05 — İçerik ve mesaj mimarisi (14 Kasım 2026)

- Rapor: `web-stratejisi/16-content-messaging-architecture.md`.
- Homebrewers ve Professional Breweries için ayrı value proposition, mesaj sütunları, sayfa brief'leri ve CTA sınırları tamamlandı.
- Teknik, ürün, cihaz, güvenlik, mevzuat ve ticari iddialar için kanıt gereklilikleri tanımlandı.
- Terminoloji, US/UK dil ve birim kuralları, SEO kümeleri ve içerik yayın iş akışı yazıldı.
- Üretim metni ve canlı site uygulaması bu paketin kapsamı değil.

## WP-06 — Görsel yön ve tasarım sistemi (14 Kasım 2026)

- Rapor: `web-stratejisi/17-visual-direction-design-system.md`.
- Mevcut `tasarim-sistemi/tokens.json` tek token kaynağı olarak korundu; yeni marka renkleri icat edilmedi.
- Tipografi rolleri, spacing/grid, 1440/1024/768/390/320 responsive kuralları ve ortak component envanteri tanımlandı.
- Home ve Business için farklı sayfa kompozisyonu, görsel bağlam, veri örneği ve içerik ritmi belirlendi.
- FoamByte kıyaslaması kopyalama amacıyla değil, discoverability, calculator utility, proof ve bilgi yoğunluğu dersleri için kullanıldı.
- Beş mevcut `data-image-slot` korunacak; kullanıcı onayı olmadan fotoğraf üretilmeyecek veya bulunup eklenmeyecek.
- WCAG 2.2 AA hedefi, klavye/touch, reduced motion ve responsive QA kabul kriterleri eklendi.
- Yalnız strateji dokümanları değiştirildi; `marketing-site/`, `web/`, ürün API'leri, iOS ve Claude ürün geliştirme kuyruğuna dokunulmadı.
- **Sıradaki adım:** temiz PR için CI/Vercel kontrolü; sonrasında WP-07 sayfa bazlı wireframe refinement. Production uygulama için ayrıca açık kullanıcı onayı gerekir.


## WP-07 — Sayfa bazlı wireframe ve içerik-layout eşlemesi (14 Kasım 2026)

- Rapor: `web-stratejisi/18-page-level-wireframes-content-layout.md`.
- Homepage, Homebrewers, Professional Breweries, Features, How It Works, Calculators, Learn, FAQ ve Technical Principles için bölüm sırası, layout ve mobil davranış tanımlandı.
- Home ve Business için ayrı süreç diyagramları, veri örnekleri, CTA ve görsel bağlamı belirlendi.
- Business sayfasında ERP dışı kapsam görünür bir bölüm olarak zorunlu tutuldu.
- Tüm CTA'lar gerçek sayfalara/kanıtlanmış ürün akışlarına bağlı; mevcut olmayan signup/trial/demo akışları kullanılmayacak.
- Bu çalışma yalnızca wireframe dokümantasyonu; site kodu veya canlı deployment değiştirilmedi.
- **Sıradaki adım:** WP-07 PR CI/Vercel Preview kontrolü; ardından WP-08 SEO ve teknik içerik yayın standardı. Canlı tasarım QA ve production uygulama için ayrıca kullanıcı onayı gerekir.


## WP-09 — Teknik içerik ve hesaplayıcı doğrulama backlog'u (14 Kasım 2026)

- Rapor: `web-stratejisi/20-technical-content-calculator-validation.md`.
- Ürün planındaki sunucu taraflı Rust hesap motoru ile bağımsız public web calculator kararı birbirinden ayrıldı.
- OG/gravity, ABV, recipe scaling/volume, IBU, colour, water/mineral, mash pH, yeast, boil-off, efficiency ve costing için kanıt/test backlog'u oluşturuldu.
- Bağımsız golden vectors, boundary/invalid, unit equivalence, rounding, locale, regression ve cross-platform parity testleri tanımlandı.
- Fermentation prediction, deviation alerts ve onaysız analitik modeller public calculator taahhüdü dışında tutuldu.
- Public calculator için ayrıca ürün sahibi onayı ve teknik doğrulama kapısı zorunlu.
- **Sıradaki adım:** WP-10 — sayfa bazlı içerik backlog'u ve evidence register. Formül/ürün kararları varsayılmayacak.


## WP-10 — Sayfa içerik backlog'u ve evidence register (14 Kasım 2026)

- Rapor: `web-stratejisi/21-page-content-evidence-register.md`.
- P0/P1/P2/P3 sayfa backlog'u; Home ve Business içerik farkları; claim/evidence register şeması ve içerik türüne göre kanıt paketi tanımlandı.
- Ürün aşaması, teknik kanıt, gerçek CTA ve erişilebilirlik/SEO kapıları geçmeden yayın yok.
- Pricing, public roadmap dates, customer stories, testimonials, security badges, compliance guarantees ve AI/prediction claims hold'da.
- PR #16 ve #19 tarihsel/açık PR olarak kaldı; WP-03 claim audit'in main'e temiz entegrasyonu ayrıca ele alınacak.
- **Sıradaki adım:** WP-11 — mevcut marketing-site uygulamasını WP-04–10 ile karşılaştıran gap analysis ve uygulama sırası. Production koduna geçiş için ayrıca açık onay gerekir.
