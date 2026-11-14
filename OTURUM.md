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
- GitHub: pazarlama sitesi değişiklikleri feature branch + PR; kullanıcı talimatıyla Auto Merge kullan.

**Tasarım yönü**
- Editorial + Technical + Human. AI landing page klişeleri, sahte dashboard, uydurma metrik ve gradient/glow yok.
- Gerçek ürün ekranı yalnız mevcut uygulamadan; gerçekçi homebrew ve brewery fotoğrafları.
- Ana sayfa, Homebrewers ve Professional Breweries için desktop + mobile wireframe yazıldı: `web-stratejisi/11-visual-design-wireframes.md`.
- Özellik iddiaları ürün akışı/test/dokümanıyla doğrulanmadan yayınlanmaz.

**Sonraki adımlar**
1. Homepage prototipinin desktop/mobile görsel yönünü kullanıcıyla gözden geçir; fotoğraf varlıklarını ve gerçek ürün ekranlarını onayla.
2. Tasarım onayından sonra iki audience landing page'i aynı sistemle hazırla.
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
