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
- `web-stratejisi/README.md`, `01-stratejik-brief.md`–`09-launch-measurement.md` ve `10-teknik-mimari.md` GitHub'da mevcut. Teknik mimari dokümanı Astro + TypeScript önerisidir; kullanıcı onayı bekler.
- Strateji dosyaları iki audience brief'i, marka/mesaj, sitemap, görsel tasarım/UX, dönüşüm, SEO, lansman ölçümü ve teknik mimari önerisini kapsar.
- Bu kayıtlar strateji ve mimari öneri dokümanlarıdır; çalışan pazarlama sitesi veya tamamlanmış görsel tasarım değildir.

**Sonraki adımlar**
1. `web-stratejisi/10-teknik-mimari.md` önerisini kullanıcıyla netleştir: Astro + TypeScript, hosting, domain, içerik ve form/analytics kararları.
2. Stack onaylanınca `marketing-site/` için ayrı teknik plan ve CI tasarla; mevcut `web/` React/Vite ürün uygulamasını değiştirme.
3. Ardından ana sayfa ve iki audience landing page için desktop/mobile wireframe hazırla.
4. Tasarım sistemi token'larını `tasarim-sistemi/tokens.json` kaynağından kullan; paralel token sistemi oluşturma.
5. Fiyat, trial, paket, ödeme sağlayıcısı ve limitler kesinleşmeden fiyat/teklif vaat etme.
6. Yalnız pazarlama sitesi dosyalarına dokun; Claude ürün geliştirme kuyruğunu değiştirme.

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
