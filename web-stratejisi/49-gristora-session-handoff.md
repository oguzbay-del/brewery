# GRISTORA — Oturum Devam Dosyası

**Son güncelleme:** 18 Kasım 2026  
**Proje:** `oguzbay-del/brewery`  
**Çalışma alanı:** Yalnızca pazarlama sitesi ve açıkça onaylanmış site dokümantasyonu.

## Bu oturumda alınan kararlar

1. Pazarlama sitesi ürün tamamlanmadan yayımlanmayacak. Şu anki hedef üretim yayını değil, siteyi geliştirmek ve hazır tutmaktır.
2. Sitede `In Development`, `Planned`, `Under consideration` gibi ürün olgunluğu etiketleri gösterilmeyecek.
3. Kullanıcının açıkça kaldırılmasını onayladığı ana sayfa ifadeleri:
   - “Clarity for every brew”
   - “Brewing software for professional breweries”
   - “From recipe standard to batch record”
4. Ana sayfadaki bu ifadeler kaldırılırken diğer sayfalara ve daha önce onaylanmış alanlara dokunulmayacak.
5. Önceki denetimdeki “Uydurma veya erken vaatler” listesi, mevcut ürün özelliği veya roadmap taahhüdü sayılmadan ayrı fikir havuzunda saklanacak: `plan/urun-fikirleri-eklenebilecek-ozellikler.md`.
6. Kullanıcı şu alanlarda değişiklik istemedi: üretim bağlamı; batch kayıtları; hesaplamalar; Stitch'in dört kartlık alanı; diğer ana sayfa metinleri ve diğer sayfalar.
7. Otomatik merge yok. Production deploy yok. Her iş ayrı PR ile sunulacak; CI sonucu ve preview kontrol edilecek, merge için ayrıca kullanıcı kararı beklenecek.

## Kapsam sınırları

- İzin verilen: `marketing-site/` ve `web-stratejisi/` altındaki pazarlama sitesi dokümantasyonu.
- Kullanıcının bu oturumdaki özel onayıyla yalnızca fikir havuzu dosyası: `plan/urun-fikirleri-eklenebilecek-ozellikler.md`.
- Dokunulmayacak: ürün uygulaması, API, Rust hesaplama motoru, iOS uygulaması, veri modeli, ürün roadmap'i, ürün iş listesi ve Claude'un ürün geliştirme çalışmaları.
- Root `CLAUDE.md` değiştirilmez.

## Stitch ve GitHub çalışma modeli

- GitHub `main`, yayımlanacak web sitesinin kod ve içerik için tek doğruluk kaynağıdır.
- Google Stitch, tasarım üretim/revizyon ortamıdır; GitHub'ın yerine geçmez.
- Stitch ↔ GitHub çift yönlü senkronizasyonu, yalnızca MCP URL/API key tanımlamakla garanti edilmez. Her aktarım commit/PR üzerinden doğrulanmalıdır.
- Bu ChatGPT oturumunda Google Stitch MCP sunucusuna doğrudan bağlanmış bir bağlantı doğrulanmadı. Stitch'te işlem yapıldığı iddia edilmemelidir.
- Stitch'ten gelen değişiklikler ayrı branch/PR olarak GitHub'a alınmalı; mevcut tasarım sistemi ve gerçek uygulama ile karşılaştırılmalıdır.
- Aynı anda bağımsız işler ayrı branch/PR'larda yürütülebilir. Aynı dosyayı değiştiren işler paralel yazılmamalıdır.

## Başlangıç GitHub durumu

- Repo: `https://github.com/oguzbay-del/brewery`
- `main` başlangıç commit'i: `4a6d27bf52eca12140b1ec9985bd18cc2db87e14`
- Önceki Stitch homepage PR #168 merge edilmişti: `4a6d27bf52eca12140b1ec9985bd18cc2db87e14`.
- Sonraki çalışma için bu dosyayı oku; GitHub'da güncel `main`, açık PR'lar ve CI durumunu yeniden kontrol et. Bu dosyadaki commit ve branch bilgilerini körü körüne güncel kabul etme.

## Bu oturumda açılan işler

### A. Ana sayfa kopya temizliği
- Branch: `work/gristora-homepage-copy-removal-2026-11-18`
- Dosya: `marketing-site/src/pages/index.astro`
- Yalnızca onaylanan üç ifadeyi kaldır.
- Kaldırma; sayfa yapısını, dört kartlık Stitch bölümünü, diğer bölümleri veya diğer sayfaları değiştirmemeli.
- Güncel branch commit: `9095502148cf6a677ac04978531156dbc3416361`
- PR: [#169](https://github.com/oguzbay-del/brewery/pull/169) — açık; Vercel preview hazır. İlk doğrulama, kısa meta açıklaması ve kaldırılan başlığa bağlı aria-labelledby nedeniyle başarısız oldu; ikisi düzeltildi. Son run #44395865907'de Astro/TypeScript, build ve site validation başarılı; responsive browser QA devam ediyor.
- Preview: https://brewery-git-work-gristora-homepage-copy-removal-3330b2-demleme.vercel.app

### B. Ürün fikir havuzu
- Branch: `docs/gristora-session-and-product-ideas-2026-11-18`
- Dosya: `plan/urun-fikirleri-eklenebilecek-ozellikler.md`
- Fikir havuzu mevcut ürün kapsamı değildir; ürün roadmap'ine otomatik aktarılmaz.
- PR: [#170](https://github.com/oguzbay-del/brewery/pull/170) — açık; tutarlılık/formül kontrolü başarılı, Vercel preview hazır.
- Preview: https://brewery-git-docs-gristora-session-and-product-id-69d6ef-demleme.vercel.app

### C. Oturum handoff ve yapılacaklar
- Bu dosya: `web-stratejisi/49-gristora-session-handoff.md`
- Her oturum sonunda tamamlanan işleri, açık işleri, PR/commit/CI/preview bağlantılarını ve kullanıcı kararlarını güncelle.
- Yeni oturumda kullanıcı “buradan devam et” derse önce bu dosyayı, ardından GitHub güncel durumunu oku.

## Paralel çalışma planı

- **Track A — Homepage:** yalnızca onaylanan ifadelerin kaldırılması. PR #169.
- **Track B — Product ideas register:** 10 maddelik fikir havuzunu ayrı `plan/` dokümanına kaydetme. PR #170.
- **Track C — Session continuity:** bu dosya ve sonraki oturumların buradan devam etmesi. PR #170.
- **Track D — Stitch/GitHub alignment:** tasarım aktarımını commit/PR ile doğrulama. ChatGPT'de doğrudan Stitch bağlantısı yok; kullanıcı ortamında MCP veya dışa/içe aktarma akışı doğrulanmadan tamamlandı sayılmaz.
- **Track E — Final site QA:** ürün tamamlanıp yayın öncesi onay verildiğinde sayfaları, responsive düzeni, linkleri, metadata'yı ve gerçek ürün kapsamıyla tutarlılığı denetleme. Şimdilik beklemede; mevcut sayfaları değiştirme.

## Devam edilecek yapılacaklar

- [x] Ana sayfa değişikliğinde onaylanan ifadelerin kaldırıldığını ve hedef ifadelerin kalmadığını metin taramasıyla doğrula.
- [x] Ürün fikir havuzu ayrı doküman olarak kaydedildi.
- [x] Bu handoff dosyası PR numaraları ve branch bilgileriyle güncellendi.
- [ ] PR #169 responsive browser QA'nın tamamlanmasını doğrula; Astro/TypeScript, static build ve 2,290 site validation kontrolü başarılı. PR #170 belge testi başarılı.
- [x] Her iki PR için Vercel preview dağıtımının hazır olduğunu doğrula; production deploy yapılmadı. Görsel tarayıcı QA'sı ayrıca yapılmalı.
- [ ] Google Stitch için ChatGPT'de doğrudan bağlantı/connector bulunmadı; harici MCP bağlantısı veya Stitch export/import akışı kullanıcı ortamında doğrulanmalı. Senkronizasyon iddia etme.
- [ ] Kullanıcı onayı olmadan merge etme.
- [ ] Kullanıcı onayı olmadan yeni sayfa metinleri veya ürün vaatleri değiştirme.

## Ürün olgunluğu — yalnızca iç çalışma bağlamı

Ürün geliştirme kaynağı `gelistirme/DURUM.md` ve ürün planıdır. 18 Kasım 2026 itibarıyla `gelistirme/DURUM.md` Stage 2 (Core API and data) gösteriyor. Bu bilgi pazarlama sitesinde olgunluk etiketi olarak gösterilmez. Ürün kodu veya roadmap'i bu pazarlama çalışması kapsamında değiştirilmez.
