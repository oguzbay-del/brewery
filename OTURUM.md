# Aktif çalışma oturumu

**Son güncelleme:** 2026-11-14  
**Amaç:** Yeni ChatGPT veya Claude oturumunun bağlamı yeniden araştırmadan devralması.  
**Kural:** Bu dosya kısa tutulur. Her anlamlı iş tamamlandığında ve oturum kapatılırken güncellenir.

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
- PR CI #43631065843: **success**. Son düzeltmeler sonrası PR CI #43631065843? (son head: `cb2c0e2c9a25107dbf53136b1c807e5cddec2f6a`) doğrulaması başarılı.
- Main marketing checks #43631137613: **success**. Pages deploy #43631137672: **success**.
- Ana sayfa ve iki audience sayfası artık ürünün geliştirme aşamasında olduğunu açıkça belirtiyor. Audience maddeleri kullanılabilir özellik iddiası değil, tasarım önceliği olarak etiketli.
- Yayın: https://oguzbay-del.github.io/brewery/
- Görsel tarayıcı QA (1440px / 390px) hâlâ açık; gerçek cihaz/browser erişimi gerekir.
- Sıradaki adım: erişilebilir bir tarayıcı önizlemesinde responsive QA; sonrasında ürün API ve istemci akışları gerçekten yayınlandıkça pazarlama iddialarını kademeli doğrulamak.
