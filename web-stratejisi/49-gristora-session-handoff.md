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
- PR: [#169](https://github.com/oguzbay-del/brewery/pull/169) — açık; Vercel preview hazır. İlk doğrulamada kısa meta açıklaması ve kaldırılan başlığa bağlı aria-labelledby hataları bulundu; ikisi düzeltildi. Son run #44395865907'de Astro/TypeScript, static build, site validation ve responsive browser QA başarılı.
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
- [x] PR #169 CI tamamlandı: Astro/TypeScript, static build, site validation ve responsive browser QA başarılı. PR #170 belge testi başarılı.
- [x] Her iki PR için Vercel preview dağıtımının hazır olduğunu doğrula; production deploy yapılmadı. Görsel tarayıcı QA'sı ayrıca yapılmalı.
- [ ] Google Stitch için ChatGPT'de doğrudan bağlantı/connector bulunmadı; harici MCP bağlantısı veya Stitch export/import akışı kullanıcı ortamında doğrulanmalı. Senkronizasyon iddia etme.
- [ ] Kullanıcı onayı olmadan merge etme.
- [ ] Kullanıcı onayı olmadan yeni sayfa metinleri veya ürün vaatleri değiştirme.

## Ürün olgunluğu — yalnızca iç çalışma bağlamı

Ürün geliştirme kaynağı `gelistirme/DURUM.md` ve ürün planıdır. 18 Kasım 2026 itibarıyla `gelistirme/DURUM.md` Stage 2 (Core API and data) gösteriyor. Bu bilgi pazarlama sitesinde olgunluk etiketi olarak gösterilmez. Ürün kodu veya roadmap'i bu pazarlama çalışması kapsamında değiştirilmez.


## Oturum devamı — doğrulama notu (18 Kasım 2026, 23:59 TRT)

- Güncel main: 4a6d27bf52eca12140b1ec9985bd18cc2db87e14 (Stitch homepage PR #168).
- PR #169 ve #170 hâlâ açık; ikisi de manuel kullanıcı kararı olmadan merge edilmeyecek.
- PR #169 son Marketing site checks run #44395865907 başarılı. Vercel preview HTTP 200 döndü.
- PR #169 preview HTML kontrolünde onaylanan üç ifade bulunmadı; Stitch'in dört kartı (Recipe standards, Understandable calculations, Production context, Useful records) korundu. Bu, HTML smoke check'tir; görsel/cihaz üzerinde manuel QA yerine geçmez.
- PR #170 belge workflow run #44396154481 başarılı. PR #170 preview HTTP 200 döndü; bu preview dokümantasyon branch'inin kendi bazını gösterir ve PR #169 homepage düzeltmesini içermez. Bu nedenle PR #169 için görsel onay kaynağı olarak kullanılmamalıdır.
- Vercel preview HTML yanıtları noindex/canonical meta alanları içeriyor; production deploy yapılmadı.
- Kalan işler: Stitch aktarımının kullanıcı ortamında gerçek commit/PR ile doğrulanması; iki PR'ın görsel tarayıcı/cihaz QA'sı; kullanıcı merge kararı. Bu oturumda merge veya production deploy yapılmadı.


## Merge and continuation update (19 November 2026)

- PR #169 merged into main via squash. Merge commit: 03bde32bd189f1798cacc32d574b075f3bc7540c.
- PR #170 merged into main via squash. Merge commit: 6bb362ce6f2e2ef01e422654f77f16d0b061d3f2.
- Main now points to 6bb362ce6f2e2ef01e422654f77f16d0b061d3f2. Both PRs are closed as merged.
- No production deployment was initiated.
- Next work remains within marketing-site only: run post-merge checks against main; perform visual/mobile QA against a preview built from the merged main; review Stitch/GitHub design parity only if a real Stitch export or connected workflow is available. Do not modify copy or pages outside explicit approval.


## Post-merge QA continuation — 19 November 2026

- PR #169 and #170 are merged. Current main at QA start: `fc5be28070686aaa43140b2ca801e01f4697294a`. The marketing-site source tested below is the same source tree introduced by PR #169 merge commit `03bde32bd189f1798cacc32d574b075f3bc7540c`; subsequent commits were documentation-only.
- Marketing site CI run [#44397172981](https://github.com/oguzbay-del/brewery/actions/runs/44397172981) completed successfully. Astro/TypeScript check, static build, generated route/metadata/link validation, Chromium responsive tests and screenshot upload all succeeded.
- GitHub Pages workflow [#44397173007](https://github.com/oguzbay-del/brewery/actions/runs/44397173007) completed successfully, including the Pages deploy job. Important: the workflow passes `preview: false` to `actions/deploy-pages`, so this is a normal, publicly accessible GitHub Pages deployment at `https://oguzbay-del.github.io/brewery/`, not an ephemeral/private Actions preview. The generated site is configured `noindex, nofollow` with `Disallow: /`; the custom production domain `gristora.com` was not deployed.
- Responsive QA artifact: `marketing-responsive-qa-screenshots` (#14471831688; 31 PNGs). Social card artifact: `marketing-social-card-png` (#14471001544).
- Visual samples reviewed: homepage desktop/mobile, professional breweries mobile, pricing mobile, comparison desktop/mobile, how-it-works mobile and the generated social card. Homepage hierarchy/palette and mobile stacking look coherent; no obvious clipping was seen in reviewed pages. Browser tests assert no horizontal document/body overflow across 11 routes at 1440/1024/768/390/320 px.
- Visual issue recorded for follow-up: the two audience decision cards at the bottom of `/compare/` contain large empty dashed visual frames on both desktop and mobile. They read as missing media/placeholders. Do not invent or add imagery without owner-approved assets/direction; track as an open visual decision.
- The mobile comparison table intentionally scrolls horizontally inside its labelled, keyboard-focusable region; the page itself does not overflow. Automated test covers the scroll container.
- Automated axe suite covers WCAG 2.2 AA tags at desktop and mobile for 11 routes; keyboard skip-link/mobile menu/FAQ disclosure, reduced-motion and scroll-region focus checks are included. Manual keyboard, screen-reader and 200% zoom checks remain outstanding.
- Pricing page explicitly says monetary prices await commercial approval and trial terms are not active in preview. Keep this distinction; do not present them as live commercial offers.
- Stitch parity is still not verifiable from this ChatGPT environment: no Stitch tool/connector is available. Do not claim Stitch was updated or synchronized. Compare only against an actual user-provided Stitch export/reference.
- Production launch remains deferred. Continue only inside `marketing-site/` and its acceptance/handoff docs; do not alter product code or Claude's work.


## Final checkpoint — 19 November 2026

- Documentation PR [#171](https://github.com/oguzbay-del/brewery/pull/171) merged into `main` via squash. Merge commit: `b92d3774e8fec92d74a66b31d1f4cc79c76615bb`.
- PR #171 latest CI run [#44398381059](https://github.com/oguzbay-del/brewery/actions/runs/44398381059) passed. Exact Playwright result: **116 passed**; Astro/TypeScript, static build, generated-site validation and screenshot artifact upload also passed.
- Main now points to `b92d3774e8fec92d74a66b31d1f4cc79c76615bb`. PRs #169, #170 and #171 are merged.
- Visual QA sample expanded to homepage 320/390/1440, Breweries 390/1440, Homebrewers 390, Pricing 390, Compare 390/1440, How It Works 390, FAQ 390, Technical Principles 390, Features 390, recipe-design detail 390, get-started 390 and the social card. No obvious page-level horizontal overflow or broken stacking in the reviewed captures.
- Compare-page audience cards intentionally render dashed empty media frames through CSS pseudo-elements; they are not failed network images. They remain an unfinished visual treatment and need owner-approved imagery or removal/replacement before final public launch.
- GitHub Pages currently publishes a publicly accessible `noindex, nofollow` copy at `https://oguzbay-del.github.io/brewery/` because `actions/deploy-pages` uses `preview: false`. It is not a private/ephemeral preview. The custom `gristora.com` production domain was not deployed. Keep this visibility distinction explicit.
- Remaining gates: complete manual keyboard/screen-reader/200% zoom review; run performance/Lighthouse review; verify the GitHub Pages URL's live HTTP/content response; obtain the actual Stitch export/reference or connected Stitch workflow for design parity. Do not change the Compare visual treatment, disable Pages, or deploy the custom domain without explicit owner approval.


## Deployment guard investigation — 19 November 2026

- Latest main CI after PR #171: marketing checks [#44398637229](https://github.com/oguzbay-del/brewery/actions/runs/44398637229) and GitHub Pages workflow [#44398637250](https://github.com/oguzbay-del/brewery/actions/runs/44398637250) both completed successfully for commit `b92d3774e8fec92d74a66b31d1f4cc79c76615bb`.
- Important correction to the prior "no production deployment" wording: the Vercel Git integration automatically created a deployment with `target: production` for the docs-only main commit `b92d377...`, and again for the latest handoff-only main commit `05d2bd96...`. The latest deployment is READY at `brewery-33oa7zrx5-demleme.vercel.app` and has aliases `brewery-rho.vercel.app`, `brewery-demleme.vercel.app`, `brewery-git-main-demleme.vercel.app`.
- Vercel project `brewery` reports SSO protection enabled for all deployments except custom domains, no custom domains attached, and `live: false`. The deployment is marked production by Vercel, but the project protection state means do not describe it as an unrestricted public launch. This was an automatic Git integration consequence of merging/pushing to `main`, not an intentional manual production promotion.
- GitHub Pages is separately publicly accessible at `https://oguzbay-del.github.io/brewery/` and the workflow uses `preview: false`; generated robots policy is `noindex, nofollow` with `Disallow: /`.
- The configured custom domain `gristora.com` is not attached to this Vercel project in the domain list. No DNS or custom-domain launch was performed.
- Guardrail: until the owner authorizes launch, avoid further pushes/merges to `main` because they trigger Vercel `target: production` deployments. Continue work on feature branches and preview deployments. Before any further main merge, the Vercel Git production-branch / auto-deploy configuration needs owner-approved adjustment. No rollback, domain change, project setting change or deployment cancellation was attempted.
- External live HTTP checks could not be completed from the current runtime (DNS resolution unavailable); use the GitHub Pages and Vercel dashboards/browser from an environment with network access for the final HTTP response check.
