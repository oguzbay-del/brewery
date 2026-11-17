# GRISTORA — Current handoff (15 November 2026)

## Fixed decisions and scope

- **Owner decision (Oğuz, 16 Nov 2026):** the public site shows no product-maturity disclaimers; product–site consistency is verified by the owner (doc 41 updated). Disclaimers removed from `marketing-site/` pages, features data, SVG diagrams, social card and validator/test assertions. Results: `astro check` 0 errors, build complete, `validate:site` 2292 checks passed, Playwright 116 passed. Next step: owner review of the live copy.
- Public brand: **GRISTORA**. Descriptor: **Brewing Intelligence**.
- The owner selected the **first logo design shown by the assistant** before the message beginning “Paralel çalışmayı sürdürdüm. Bu turda marka dokümantasyonundan çıkıp ilk logo tasarımını…”. This is the only direction to refine; do not substitute later exploration variants or ask the owner to repeat the decision.
- Reference: https://chatgpt.com/s/m_6af9ac3f07e48191aa8504a2e22a77a9. The connector and web fetcher cannot retrieve the rendered image. Do not invent its geometry or claim a reconstruction is faithful. Exact tracing is pending access to that source, not pending owner preference.
- Public website only. Do not rename or modify the product application, APIs, Rust engine, iOS app, data model, internal identifiers, product roadmap or product-development queue.
- **ChatGPT assignment boundary (owner instruction, 15 Nov 2026):** ChatGPT works only on the public marketing website. It must not touch the main product, Claude's code, instructions, branches, or development queue. The binding rule and website completion/stop criteria are in `marketing-site/CHATGPT-SCOPE.md`. Claude's product-development workflow remains separate and unchanged.
- Primary commercial audience: Professional Breweries. Homebrewers is a distinct secondary path. English-first, US/UK initial markets, self-service discovery.
- The product and public site are in development. No unsupported customer, metric, pricing, trial, signup, integration, production availability, certification, compliance or predictive-AI claims.

## GitHub status

- PR #147 was manually squash-merged: https://github.com/oguzbay-del/brewery/pull/147
- Merge commit: `e34a379370495a68ddcfd1d760a6f9800f50e66b`.
- Marketing Site Checks run #219 (`43717341183`) completed successfully: dependency install, Astro/TypeScript, static build, generated routes/metadata/links, social preview artifact and responsive browser QA.
- Vercel preview was blocked by the existing 24-hour Hobby deployment quota. No live preview/deployment is claimed.
- CI screenshot artifact was downloaded and spot-checked: homepage 1440/390 px, Breweries 390 px and generated 1200×630 social card. No obvious clipping in those spot-checks; the full responsive suite passed. Existing social card still carries legacy Demleme branding and is not a GRISTORA-approved asset.
- No product code or marketing-site source code was changed in this brand/documentation work.

## Canonical GRISTORA documents

- 33 — `web-stratejisi/33-gristora-brand-platform.md`
- 34 — `web-stratejisi/34-gristora-brand-architecture.md`
- 35 — `web-stratejisi/35-gristora-visual-identity-logo-brief.md`
- 36 — `web-stratejisi/36-gristora-public-site-rebrand-plan.md`
- 37 — `web-stratejisi/37-gristora-logo-production-and-qa.md`
- 38 — `web-stratejisi/38-gristora-public-site-brand-implementation-map.md`
- 39 — `web-stratejisi/39-gristora-trademark-domain-preliminary-evidence-log.md`
- 40 — `web-stratejisi/40-gristora-preliminary-goods-services-specification.md`
- 41 — `web-stratejisi/41-gristora-public-marketing-claims-guardrails.md`
- 42 — `web-stratejisi/42-gristora-marketing-site-ci-artifact-review.md`
- 43 — `web-stratejisi/43-gristora-public-homepage-copy-blueprint.md`
- 44 — `web-stratejisi/44-gristora-trademark-counsel-review-brief.md`

## Open work

1. **WP-16 / Issue #145:** access the exact selected first logo artwork; create faithful editable SVG master and only composition-supported lockups; run document 37 QA; owner approval remains pending.
2. **Trademark/domain / Issue #146:** official record-by-record searches in US, UK, Australia and WIPO/Madrid; counsel must assess C-DAC's official **GistOra Tools** software name (one-letter spelling difference, software-sector adjacency; current commercial activity/rights unknown); verify the official Indonesian GISTORA/BASF application `DID2026030217` post-publication status; separately verify the third-party India lead `7525714` in IP India; capture current RDAP/registrar evidence. Indonesia's gazette publication is confirmed (Classes 1/5/31; 9 Apr–9 Jun 2026 publication period), but current status is unknown. No availability conclusion yet.
3. **Goods/services:** document 40 is a counsel-review draft. Class 42 is likely core for hosted SaaS; Class 9 depends on branded downloadable software; Class 35 is not automatic merely because software supports a brewery's internal operations; Class 41 depends on separately marketed training.
4. **WP-17:** public-site source rebrand remains gated by exact logo approval and legal/domain review. When authorised, change only `marketing-site/`, update all public routes/metadata social card/favicon, preserve product maturity disclosures and run the full isolated marketing CI plus manual accessibility/device review. Document 38 lists all 16 generated `/features/{slug}/` routes; validate each route's canonical, OG/Twitter and JSON-LD metadata. Document 43 contains the proposed homepage copy and metadata patterns, not yet approved for publication.
5. **WP-18:** final public-domain, hosting, canonical URL, analytics/consent, contact/conversion route and launch QA decisions remain open.

## Research and monitoring

- Preliminary trademark evidence and limitations are in document 39; proposed class wording/logic in document 40; public claim restrictions in document 41; existing CI visual review in document 42; homepage copy draft in document 43; counsel brief in document 44.
- Existing weekly GRISTORA trademark/domain monitor remains the only recurring automation; its prompt was updated on 15 Nov to track the C-DAC GistOra software lead, Indonesian BASF GISTORA gazette application and separate India BASF index lead. Next scheduled check: 16 Nov 2026, 10:00 Europe/Istanbul. Do not create a duplicate.
- No trademark filing, domain purchase, third-party contact, or public clearance claim is authorised.

---

> This section supersedes older dated snapshots below. Older entries are historical records.

- **Stitch overnight brief (16 Nov 2026, Claude, owner request):** `marketing-site/stitch-brief/STITCH-GECE-TALIMATI.md` has 11 copy-paste Stitch prompts: mobile versions of 10 pages, the 16 `/features/{slug}/` pages (approved copy from `features.ts`, word for word), the feature directory, the 404 page and a consistency pass. Each prompt repeats the visual tokens and the document 41 forbidden list. The file does not contain the API key. Next step: the owner runs the batches in Stitch, then Claude ports the result to Astro.
- Update: `stitch-brief/calistir.py` runs the 39 jobs in `jobs.json` one by one through the Stitch MCP, without an LLM (resumable, state in `durum.json`). It started on 16 Nov 22:44 UTC in the cloud container. To run it again: `STITCH_API_KEY=... python3 -I calistir.py`. Next step: Claude reads the new screens and ports them to Astro.
- **Stitch run complete (17 Nov 2026 02:37 UTC):** 39/39 jobs produced screens. `list_screens` does not show the new screens; their IDs are in `marketing-site/stitch-brief/ekranlar.json`. Use `get_screen` with these IDs for the Astro port. `watchdog.sh` restarts `calistir.py` when it stops.

- **Stitch port of all main pages (16 Nov 2026, Claude):** Pricing, How it works, Product, FAQ, Technical principles, Compare and Get started now use the Stitch layout. Each page has a `<main>` class, and its rules in `stitch-layout.css` are scoped to that class. Workflow: Stitch generates a screen from the approved copy plus guardrail rules, a Sonnet subagent ports it to Astro, and Claude reviews and pushes. The CLI and prompts are in the session scratchpad (not in the repository, because they contain the API key). Checks: validate:site 2292/2292, Playwright 116/116, no horizontal overflow at 400 and 1280 px on 11 routes. Open: mobile Stitch screens, the 16 `/features/{slug}/` detail pages and the 404 page are not ported.

- **Stitch layout update (16 Nov 2026, Claude, owner request):** Claude read the Stitch project "GRISTORA Marketing Website Implementation" through the Stitch MCP HTTP endpoint. Stitch did not push to GitHub; its "PR summary" screens are text only. The owner chose "Stitch layout, approved copy". The homepage now uses the Stitch structure: a dark forest workflow panel (labelled concept, no live data), fact tiles, numbered cards, a dark quote band, audience cards with checklists, a product-status band and a dark CTA band. The Breweries and Homebrewers heroes use a dark band. New file: `marketing-site/src/styles/stitch-layout.css`. The Stitch copy was not used: it has a fabricated testimonial, unapproved prices, telemetry/IoT/SCADA and predictive claims (document 41). Checks: astro check 0 errors, validate:site 2309/2309, Playwright 116/116. The owner must rotate the Stitch API key that was pasted in chat; it is not stored in the repository. Next step: owner visual review on GitHub Pages.

- **Stitch visual language applied (15 Nov 2026, Claude, owner request):** `main` `c835f54`. Merged the 4 `marketing/brand-board-alignment` commits plus a fix. Fonts are self-hosted with `@fontsource-variable` (Inter, Plus Jakarta Sans); the Google Fonts `@import` blocked the stylesheet and failed 61 browser tests. The hero stays light like the Stitch screens (the dark hero failed contrast at 2.13:1). Pill navigation, amber pill CTAs, 18/24 px cards. Copy, claims and page structure did not change. The Stitch screen copy (telemetry, Tilt/iSpindel, predictive alerts, metrics) breaks document 41 and was not used. Local checks: astro check 0 errors, validate:site 2310/2310, Playwright 116/116. Next step: owner visual review of the GitHub Pages preview.
- **Professional Breweries landing metadata:** PR #134 improves the Brewery page's Open Graph/Twitter descriptions and makes the meta description explicit that the planned workflows remain in development. Added generated-HTML assertions for brewery-first audience and workflow topics. GitHub Actions run #196 passed Astro/TypeScript, static build, 1,738 static validation checks and responsive browser QA. Vercel Preview was blocked by the existing Hobby build-rate-limit gate. Squash merge: `2d00b265b5e739100dd90eeda20541d892200557`.\n- **FAQ Business-first SEO:** PR #131 aligns FAQ title, meta description, Open Graph/Twitter metadata, structured data and page introduction with professional breweries as the primary commercial audience; Homebrewers remain a separate secondary path. Added generated-HTML regression checks. GitHub Actions run #192 passed Astro/TypeScript, static build, generated-site validation, social preview generation and responsive browser QA. Vercel Preview remains blocked by the existing Hobby build-rate-limit gate. Squash merge: `bcbeab686cfd111798dac47f2abb100a6799b669`.
- **Homepage Business feature priority:** PR #129 moves brewery-only capabilities ahead of shared Home/Business features in the six-card homepage overview, without changing feature stages or product claims. GitHub Actions run #188 passed all build, generated-site, responsive browser QA and screenshot-artifact steps. The Vercel status remains blocked by the existing Hobby build-rate-limit gate; this is separate from GitHub Actions. Squash merge: `7e81cbdc2124fc6c5dac40f91c4de51146aa3639`.

- **Latest main:** `0b7438974658527a27765b6de256759ba0e84846` (PR #136, handoff documentation). PRs #113, #115, #116, #118, #119, #120, #121, #122, #124, #125, #126, #127, #128, #129, #130, #131, #132, #134, #135 and #136 are merged. **Open PRs: 0 at the time of this handoff.**
- **Business-first regression QA:** PR #121 adds generated-HTML checks for navigation order, primary header/homepage CTA destinations, How It Works section/comparison order, Product directory and FAQ audience hierarchy, and shared feature audience-label order. CI #43702027885 passed **1,642 static checks and 78 Playwright tests**.
- **Typed feature data:** PR #122 defines closed TypeScript unions for audience labels and roadmap stages, and types the stage-description registry against the same vocabulary.
- **Feature catalogue integrity:** PR #125 adds build-time checks for valid/unique slugs, unique/non-empty audience and roadmap labels, and non-empty sections. CI #43702595367 passed Astro/TypeScript, static build, generated-site validation, social-card artifact generation and responsive Playwright QA.
- **Metadata description quality:** PR #126 adds a quote-safe HTML meta parser and non-empty/length checks for meta, Open Graph and Twitter descriptions. CI #43702511826 passed all marketing checks, including responsive Playwright and axe. PR #123 was closed as superseded.
- **Build and QA documentation:** PR #124 updates root and marketing-site READMEs with the isolated Astro workflow, build/validation/browser commands, social-card generation, Vercel preview restrictions and GitHub Pages fallback.
- **Social preview pipeline:** PR #113 adds a reproducible Resvg step to `npm run build`. It generates `public/social-card.png` from the editable 1200×630 SVG source; generated PNG is ignored by Git and validated in `dist` for PNG signature, exact dimensions and <=512 KiB. PR #116 uploads the generated PNG as a short-lived CI artifact.
- **Social card visual approval:** PR #118 replaces the initial monospaced fallback with explicit DejaVu Sans/Serif. The final artifact was downloaded and visually inspected at full size and 320×168: clean serif headline, sans-serif supporting text, Business-first audience line, no clipping. CI #43701628298 generated a 57,529-byte 1200×630 PNG; static validator reported 1,579 checks passed and Playwright passed 78/78.
- **Social metadata remains gated:** PR #120 records visual QA approval in `web-stratejisi/26-release-gates-and-public-claims.md`. The PNG is a build artifact, not committed binary. Do not wire `og:image` or `twitter:image` until the final public origin is confirmed; after that, verify absolute HTTPS URL, reachability and representative platform previews.
- **CI runtime and artifacts:** PR #116 updates checkout/setup-node/upload-artifact to Node 24-compatible major versions and uploads the raster social card for visual review. Marketing site CI #43701065504 passed.
- **CI scope optimization:** PR #119 separates actionlint into its own workflow and narrows infrastructure smoke tests to infrastructure sources and the infrastructure workflow itself. Actionlint #43701678113 passed; infrastructure smoke #43701678102 completed successfully for Compose, Kustomize/kind and Terraform.
- **GitHub Pages deployment:** The push-to-main Pages workflow completed successfully on run #65 (`43707349438`) at commit `2d00b265b5e739100dd90eeda20541d892200557` (PR #134). Both build and deploy jobs passed, so the deployed snapshot includes PRs #129, #131 and #134. PR #136 only changes handoff documentation. The separate main marketing-site check run #197 is still completing responsive browser QA; do not conflate it with the successful Pages deploy.
- **Social metadata:** PR #103 added metadata validation; PR #105 completed Twitter title/description on all generated routes. Runs #43698119511 and #43698364890 passed, including responsive Playwright QA.
- **Accessibility validation:** PR #106 strengthened static checks for accessible names, ARIA references, iframe titles and safe new-tab links. Merged as `51184c08d8304f80f8d0bf9a5d2d12b8592410f3`.
- **Sitemap / robots:** PR #109 added XML namespace/declaration, duplicate/canonical/same-origin/route target checks and exact robots sitemap validation. Merged as `18a9cfb4f7409fe9db94501e050c34e4982b8c7c`. Final CI run #43699516581 passed.
- **Commercial strategy:** PRs #110 and #111 align strategy docs and SEO clusters around Business-first, self-service acquisition. US and UK remain initial markets; English is primary. No pricing, free tier, trial, billing, sign-up or onboarding destination has been approved.
- **Release register:** `web-stratejisi/26-release-gates-and-public-claims.md` remains the source of truth. Social raster export and visual QA are complete; final-origin image metadata and final-domain/platform validation remain open.
- **Preview / deployment:** Vercel Preview remains blocked by the 24-hour Hobby deployment quota; do not retry until reset. GitHub Pages is the temporary preview fallback, but the latest deployment still requires an explicit successful workflow run. No production domain or Vercel settings changed.
- **WP-09 / WP-10:** primary-source and sample-handling/gravity evidence remain research only. No formula, calculator or public technical article is approved. Measurement provenance remains a concept, not an approved schema/API or shipped feature.
- **Manual QA remains open:** real screen-reader testing, human keyboard/focus review, 200% zoom/reflow and final visual review on real devices. Automated axe/Playwright is supplemental, not a substitute.
- **Commercial decisions open:** production domain/hosting plan; app URL/sign-in/registration; pricing/free tier/trial/limits/billing; analytics/consent provider/events; approved photography/product screenshots; legal/contact content.
- **Release policy:** batch changes, inspect diffs and CI, merge manually; do not alter Vercel or production settings without explicit approval. No product app/API/Rust or Claude queue changes in this work batch.
- **Superseded PRs:** #114, #117 and #123 were closed and replaced by rebased/fixed PRs #115, #118 and #126. No duplicate open branches remain.


## Son devam — 15 Kasım 2026

- PR #84 manuel squash merge edildi: `a94c64e5c1a9d2b6980bc6f18bcc9318de87c377`. GitHub/Vercel batch çalışma politikası `web-stratejisi/25-github-vercel-batched-deployment-policy.md` içinde kalıcı.
- PR #85 manuel squash merge edildi: `359396dc0ab0f27b8f747cb8adbe0d70a23eb7ac`. Tüm iç sayfalara Schema.org BreadcrumbList JSON-LD ve canonical/origin/sıra kontrolleri eklendi.
- PR #85 GitHub Actions Marketing site checks run #116: **success**, Playwright **63/63 passed**; screenshot artifact #14152329367 yüklendi.
- Vercel Preview otomatik entegrasyon nedeniyle denendi ancak `api-deployments-young-hobby-team-24h` limiti nedeniyle başarısız oldu. Bu GitHub Actions build/test başarısızlığı değildir. Vercel proje/production ayarlarına dokunulmadı.
- PR #86 açık: `ux/visible-breadcrumb-navigation`. Erişilebilir görünür breadcrumb component'i, responsive stiller, 7 iç rota görünürlük testi ve mobil klavye testi eklendi.
- GitHub Actions Marketing site checks run #118: **success** — Astro/TypeScript, static build, **684 validator checks**, 16 axe WCAG 2.2 AA scan ve Playwright **78/78 passed**. Screenshot artifact #14152513961 yüklendi; Homebrewers mobile, FAQ mobile ve feature detail desktop screenshot'ları gözden geçirildi, belirgin görsel sorun yok.
- PR #86 Vercel Preview kota limiti nedeniyle başarısız olabilir; Vercel ayarlarına dokunulmadı. Bu Preview sonucu GitHub Actions testlerinden ayrı.
- **Sıradaki adım:** PR #86 tam diff ve başarılı CI sonrası manuel merge; Vercel Preview kota engeli sürerken Vercel'i bekleme.


## Güncel yayın durumu — 15 Kasım 2026

- **Hosting:** Vercel aktif preview/dağıtım hedefi; GitHub Pages yedek yayın. Ticari/public launch henüz yapılmadı.
- **Vercel adresi:** https://brewery-rho.vercel.app (mevcut site preview/deployment adresi).
- **Main güncel commit:** `359396dc0ab0f27b8f747cb8adbe0d70a23eb7ac` (PR #85 breadcrumb structured data merged).
- **Görsel yaklaşımı:** Kullanıcı, konuya uygun çok sayıda özgün grafik/görsel üretimini onayladı. Öncelik özel SVG diyagramlar ve açıklayıcı veri grafikleri; gerektiğinde özgün editoryal illüstrasyonlar. Stok fotoğraf yok; mevcut homepage hero görseli ayrıca onay alınmadan değiştirilmez.
- **Görsel QA:** Gerçek masaüstü/mobil tarayıcı QA henüz tamamlanmış değil. CI build başarısı görsel QA yerine geçmez.



## Son devam özeti — 15 Kasım 2026

- Kullanıcı, Vercel çalışmasa da marketing-site kod işlerinin GitHub üzerinden sürdürülmesini ve Vercel geri geldiğinde Preview/browser QA yapılmasını açıkça onayladı.
- Geçici preview: https://oguzbay-del.github.io/brewery/ — Pages build ve statik validator başarılı; bu URL'nin gerçek tarayıcı render QA'sı henüz yapılmadı.
- PR #63 Vercel Preview kontrolü başarılı ve geri bildirim sayısı sıfır; GitHub Pages deploy #62 sonrası başarılı. Gerçek masaüstü/tablet/mobil görsel QA henüz yapılmadı ve public release gate olarak korunuyor.
- PR #52 merge: Business için Recipe version → Batch record → Ingredient lots → Vessel events → Quality records ilişkisini anlatan, aşama etiketli record model.
- PR #53 merge: Homebrewer için Recipe/targets → Ingredients/water → Brew-day actuals → Fermentation observations → Tasting/review kayıt döngüsü.
- PR #57 merge: duplicate ID, tek H1/main landmark, skip-link, nav accessible name ve image alt attribute statik smoke kontrolleri; README içindeki hatalı literal \\n satır sonları düzeltildi.
- PR #54–56 ilk/ara denemeler olarak kapatıldı; aktif açık PR yok.
- Son başarılı marketing CI: run #77 (Astro/TypeScript, build, route/metadata/link ve accessibility smoke checks). GitHub Pages deployment run #28 son kontrol anında tamamlanmayı bekliyor.
- **Sıradaki adım:** Pages run #28 sonucunu doğrula; Home ve Business içeriklerini son kez claim/UX açısından gözden geçir; Vercel erişilebilir olduğunda 1440/1024/768/390/320 px gerçek browser QA, keyboard/screen reader ve link interaction testlerini tamamla.

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


## WP-08 — SEO, teknik yayın ve içerik kalite standardı (14 Kasım 2026)

- Rapor: `web-stratejisi/19-seo-technical-publishing-standard.md`.
- URL/search intent sahipliği, on-page içerik kalitesi, crawl/indexation, sitemap/robots/canonical, redirect, metadata ve JSON-LD kuralları tanımlandı.
- ABD/İngiltere lokalizasyonu, US/Imperial/metric birim ayrımı, hreflang koşulları ve iç link modeli yazıldı.
- Core Web Vitals, erişilebilirlik, analytics/privacy ve yayın kontrol kapıları eklendi.
- FoamByte benchmark dersleri Demleme'ye özgü içerik/kanıt yaklaşımıyla eşlendi; kopyalama yok.
- **Sıradaki adım:** WP-09 — teknik içerik ve hesaplayıcı doğrulama backlog'u. Formül testleri tamamlanmadan calculator implementasyonu/yayını yok.


## WP-10 — Sayfa içerik backlog'u ve evidence register (14 Kasım 2026)

- Rapor: `web-stratejisi/21-page-content-evidence-register.md`.
- P0/P1/P2/P3 sayfa backlog'u; Home ve Business içerik farkları; claim/evidence register şeması ve içerik türüne göre kanıt paketi tanımlandı.
- Ürün aşaması, teknik kanıt, gerçek CTA ve erişilebilirlik/SEO kapıları geçmeden yayın yok.
- Pricing, public roadmap dates, customer stories, testimonials, security badges, compliance guarantees ve AI/prediction claims hold'da.
- PR #16 ve #19 tarihsel/açık PR olarak kaldı; WP-03 claim audit'in main'e temiz entegrasyonu ayrıca ele alınacak.
- **Sıradaki adım:** WP-11 — mevcut marketing-site uygulamasını WP-04–10 ile karşılaştıran gap analysis ve uygulama sırası. Production koduna geçiş için ayrıca açık onay gerekir.


## WP-11 — Mevcut marketing-site gap analysis (14 Kasım 2026)

- Rapor: `web-stratejisi/22-existing-site-gap-analysis-implementation-sequence.md`.
- Güncel Astro route, feature data, CSS, token kaynağı, sitemap/robots ve SEO metadata source-level incelendi.
- Mevcut site: homepage + iki audience landing + feature index/16 detail; How It Works, Learn, Calculators, FAQ, Technical Principles ve Use Cases henüz yok.
- Feature status dilinin Planned / Under consideration / Requires validation ayrımını yeterince taşımama riski belirlendi.
- CSS token değerlerinin canonical JSON'dan kopya tutulduğu, ortak header/metadata yapısının tekrarlandığı ve responsive/WCAG QA'nın gerçek tarayıcıda yapılmadığı kaydedildi.
- Sıradaki uygulama sırası ve release gate yazıldı; `marketing-site/` değiştirilmedi.
- **Sıradaki adım:** WP-12 — uygulama spesifikasyonu ve acceptance checklist'i birleştir; production implementasyon için kullanıcıdan açık onay bekle.


## WP-03 — Feature claim denetimi (14 Kasım 2026)

- Rapor: `web-stratejisi/14-feature-claim-denetimi.md`.
- 16 feature alanı ürün planı domain'lerine, audience ve release stage'e eşlendi.
- Kritik çelişkiler: fermantasyon bitiş/FG tahminleri onaysız analitik modeller; eğri sapma uyarısı onay bekleyen model 2; ekipman kalibrasyonu Home MVP ile onaysız model 3 arasında çelişkili.
- Mash pH, public calculator kapsamı, cihaz/protokol ve read/control ayrımı, mevzuat ve security iddiaları için ek doğrulama şart.
- Pazarlama “Available” durumunu yalnız deploy edilmiş/test edilmiş müşteri akışında kullanmalı.
- **Sıradaki adım:** PR #16 feature catalog diff'ini claim audit ile denetle; yalnız doğrulanmış, muhafazakâr metinleri entegre et.


## WP-12 — Uygulama spesifikasyonu ve release gate (14 Kasım 2026)

- Rapor: `web-stratejisi/23-implementation-spec-release-gate.md`.
- WP-03–WP-11 kararları, hedef sitemap, Home/Business içerik tasarımı, implementasyon workstream'leri, bağımlılıklar, kalite kabul kriterleri ve release/merge protokolü birleştirildi.
- İlk sürüm kapsamı ve calculator/pricing/roadmap/account CTA gibi sonraki kapılar ayrıştırıldı.
- Production uygulama için açık gate korunuyor; bu PR yalnız dokümantasyon.
- **Sıradaki adım:** PR #16 feature catalogue değişikliklerini WP-03 ile denetle; ardından production implementation gate için kullanıcı onayını al. Bu onay olmadan `marketing-site/` değiştirilmeyecek.


## Uygulama yetkisi ve Vercel geçici erteleme — 15 Kasım 2026

- Kullanıcı, bekleyen pazarlama sitesi işlerinin kodla tamamlanmasını ve Vercel erişilebilir olmasa da GitHub üzerinden devam edilmesini açıkça onayladı.
- Uygulama yetkisi yalnız `marketing-site/` pazarlama sitesini kapsar. Ürün uygulaması/API/iOS/Rust, hesaplayıcıların yayına alınması, görsel satın alma/üretme, analytics/form/consent, fiyatlandırma ve production release bu onayın dışındadır.
- Vercel Preview ve gerçek tarayıcı QA geçici olarak ertelendi. Vercel yeniden çalıştığında Preview deployment, 1440/1024/768/390/320 px responsive QA, link/interaction ve erişilebilirlik kontrolleri yapılmadan public release tamamlanmış sayılmaz.
- Bu arada GitHub Actions, Astro check, static build, source/diff review ve PR bazlı manuel merge akışı devam edecek. Auto-merge kullanılmayacak.
- Yeni uygulama dalı: `marketing/audience-pages-content-implementation`.
- Bu dalda claim-safe feature catalogue değişiklikleri PR #40'tan alındı; Homebrewers ve Professional Breweries sayfaları ayrı workflow, FAQ, teknik açıklama ve ERP sınırıyla yeniden yazıldı. Yeni audience bileşen CSS'i eklendi.
- Fotoğraf slotları/yer tutucular korundu; yeni görsel veya sahte ürün ekranı eklenmedi. Hesaplayıcı, pricing veya signup CTA'sı eklenmedi.
- İlk doğrulama: GitHub kaynak yazımları başarılı. GitHub Actions PR kontrolü ve Astro check/build sonucu bekleniyor; sonuç gelmeden build başarılı denmeyecek.
- **Sıradaki adım:** PR aç, GitHub Actions sonuçlarını izle, hata varsa düzelt. Vercel QA'yı post-deploy gate olarak takip et.


## Son doğrulama ve merge — 15 Kasım 2026

- PR #43 manuel squash-merge edildi: `814dbcfded8985549e892c175219902b202f7b88`.
- GitHub Actions run #56 başarılı: dependency install, Astro/TypeScript check ve static site build tamamlandı.
- Vercel Preview status'u build-rate-limit nedeniyle failure; kullanıcı talimatıyla bu engel geçici olarak ertelendi. Bu, Vercel görsel QA'nın yapıldığı anlamına gelmez.
- Homebrewers ve Breweries sayfaları artık ayrı workflow, role/FAQ, teknik açıklama ve kapsam anlatısı içeriyor.
- PR #40'ın claim-safe feature katalog düzeltmeleri PR #43 ile main'e alındı.
- WP-13 ve WP-14 dokümanları ile README indeks temizliği bu dokümantasyon PR'ında konsolide ediliyor.
- **Sıradaki adım:** How It Works ve Technical Principles sayfalarını ekle; feature detail sayfalarının claim/evidence şablonunu tamamla. Vercel Preview ve gerçek responsive QA'yı release öncesi tamamla.


## WP-12 uygulama devamı — How It Works ve Technical Principles (15 Kasım 2026)

- Yeni sayfalar: `/how-it-works/` (Home ve Business için ayrı workflow) ve `/technical-principles/` (method, units, planned-vs-actual, measurement context, model validation ve product-stage ilkeleri).
- İki sayfa için canonical/OG/Twitter metadata ve WebPage JSON-LD eklendi.
- Desktop ve mobile navigasyon tüm ana pazarlama sayfalarında yeni sayfalara bağlandı.
- Sitemap route listesi iki yeni sayfayı kapsayacak şekilde güncellendi.
- Yeni sayfa CSS'i `marketing-site/src/styles/learning-pages.css` içinde izole edildi.
- İçerik ürünün mevcutta yayında olduğu iddiasını yapmaz; ürün planı ve tasarım ilkesi olarak çerçevelenir.
- İlk branch'teki GitHub Actions run #58 başarılı: Astro/TypeScript check ve static build. Main'e yeni branch olarak taşındıktan sonra CI yeniden çalıştırılacak.
- Vercel Preview ve responsive browser QA ertelenmiş release gate olarak kalır.


## WP-12 uygulama devamı — FAQ ve feature evidence context (15 Kasım 2026)

- `/faq/` eklendi: ürünün geliştirme durumu, Home/Business hedef kitlesi, ERP sınırı, hesaplayıcılar, tahmin modelleri, API/webhook ve ticari durum hakkında doğrulanmış kapsam yanıtları.
- FAQ için canonical/OG/Twitter metadata ve WebPage JSON-LD eklendi; desktop/mobile navigasyon ve sitemap güncellendi.
- Feature detail şablonuna intended inputs/records, intended outputs, limitations ve evidence status açıklaması eklendi. Her sayfanın roadmap olduğunu ve public validation evidence yayınlanmadığını açıkça belirtir.
- `marketing-site/README.md` güncel sayfa envanteri, geçici GitHub Pages preview ve QA kontrol listesiyle güncellendi.
- PR #46 manuel squash-merge edildi: `8921cdf8051d9578f27609ff82fdfc3dfdf6c913`.
- GitHub Actions run #60 başarılı; GitHub Pages deploy run #20 başarılı. Geçici preview URL: https://oguzbay-del.github.io/brewery/
- Vercel Preview halen build-rate-limit engelinde. GitHub Pages deployment başarıyla yayınlandı; ancak bu oturumdaki web tarayıcı aracı URL'yi açamadığı için gerçek render/visual QA yapılmış sayılmaz.
- **Sıradaki adım:** PR #47'de FAQ + feature evidence context değişikliklerini CI ile doğrula; ardından feature-detail claim review, link/sitemap/metadata audit ve 320–1440 px browser QA için kalan listeyi sürdür.


## WP-12 kalite otomasyonu — statik site doğrulayıcı (15 Kasım 2026)

- `marketing-site/scripts/validate-site.mjs` eklendi.
- Validator build çıktısında zorunlu route'ları, sitemap hedeflerini, robots sitemap referansını, sayfa title/description/canonical alanlarını, yerel href/src dosyalarını, fragment anchor'larını ve kaldırılması gereken eski riskli metinleri denetler.
- `npm run validate:site` script'i eklendi.
- Hem PR/push marketing-site workflow'una hem GitHub Pages build/deploy workflow'una build sonrasında validator adımı eklendi.
- Amaç: Vercel olmadan da GitHub Actions üzerinde tekrarlanabilir statik QA yapmak; gerçek responsive/browser QA'nın yerini almaz.


## Responsive header refinement — 15 Kasım 2026

- Ana navigasyon altı bağlantıya çıktığı için 761–980 px tablet genişliklerinde desktop nav yerine kompakt mobil menüye geçiş eklendi.
- 981 px üzeri desktop nav; 760 px ve altı mevcut mobil nav davranışı korunur.
- Gerçek tablet tarayıcı kontrolü GitHub Pages/Vercel browser QA kapısında yapılacak.


## Feature stage wording refinement — 15 Kasım 2026

- Feature detail sayfalarında tüm özelliklere ortak “IN DEVELOPMENT” ve “Planned capability” etiketleri yerine “PRODUCT ROADMAP” ve “Roadmap scope” kullanıldı.
- Böylece Homebrew MVP, Business core, later release ve Under consideration ayrımı yalnız stage listesinde değil, sayfa girişinde de doğru bağlamla sunulur.
- Özellikle research-only alt kabiliyetlerin aktif geliştirme vaadi gibi algılanma riski azaltılır.


## SEO doğrulama kapsamının genişletilmesi — 15 Kasım 2026

- Statik site validator'ına her HTML sayfası için Open Graph title/description/url, Twitter card, HTML language, canonical-path eşleşmesi ve JSON-LD varlığı/geçerli JSON kontrolü eklendi.
- Sitemap'in yalnız hedefleri değil, üretilen her public HTML sayfasını kapsaması da kontrol ediliyor.
- PR #50 feature detail roadmap etiketleri güncellendi ve merge edildi.
- GitHub Pages deployment run #23 başarılı; validator Pages base path `/brewery` ile de build ve sitemap/link kontrollerini geçti.


## FB-04 — Business record model anlatımı (15 Kasım 2026)

- Professional Breweries sayfasına ayrı bir “Record Model” bölümü eklendi.
- Recipe version → production/batch record → ingredient lots → vessel events → quality records ilişkisi açıklandı.
- Her kayıt tipi ilgili ürün alanı ve Business release aşamasıyla etiketlendi.
- Bunun canlı veri modeli veya mevcut iş akışı olmadığı; ürün roadmap'i olduğu açıkça belirtildi.
- Otomatik compliance, recall execution veya regulatory approval iddiası özellikle dışarıda bırakıldı.
- Yeni görsel, fake UI veya ürün ekranı üretilmedi; responsive HTML/CSS ile erişilebilir bir ilişki listesi kullanıldı.


## FB-03 — Homebrewer record loop (15 Kasım 2026)

- Homebrewers sayfasına recipe/targets → ingredients/water → brew-day actuals → fermentation observations → tasting/review sırasını anlatan ayrı bir kayıt tablosu eklendi.
- Her satırda kayda girecek bilgi ve bu bilginin homebrewer'a sağlayacağı yorum bağlamı ayrıştırıldı.
- Manual/device measurement source ayrımı ve tahmin modellerinin mevcut olmadığı açıkça korundu.
- Homebrew learning loop ile Business lot/vessel/shift/quality kayıt modeli arasındaki fark ayrıca belirtildi.
- Böylece iki audience sayfası yalnız farklı başlıklar değil, farklı kayıt ve kullanım amaçları anlatıyor.


## Marketing accessibility smoke checks — 15 Kasım 2026

- Statik validator'a duplicate HTML ID, her sayfada tek H1, tek main landmark, skip-link hedefi, nav landmark accessible name ve img alt attribute kontrolleri eklendi.
- README içindeki hatalı literal \\n satır sonları gerçek Markdown satır sonlarına dönüştürüldü.
- Bu kontroller temel statik regresyon yakalar; klavye, screen reader, kontrast, focus görünürlüğü ve responsive davranış için gerçek browser QA hâlâ zorunlu.


## Homepage hero refinement — 15 Kasım 2026

- Kullanıcının isteği doğrultusunda yalnız homepage Hero alanında tipografi, CTA hiyerarşisi ve dikey boşluklar iyileştirildi.
- Başlığın “care about the details.” bölümü ayrı satırda daha güçlü ve bold vurgulanıyor; mobil font/line-height değerleri ayrıca ayarlandı.
- Homebrewing birincil CTA büyütüldü; Professional Breweries bağlantısı çerçeveli ikincil CTA olarak güçlendirildi.
- Mevcut renk paleti, illüstrasyon/SVG, sayfa içeriği ve diğer bölümlerin spacing değerleri değiştirilmedi.
- CSS ayrı `home-hero-refinement.css` dosyasında izole edildi.
- Branch: `design/homepage-hero-refinement`. CI ve gerçek responsive browser QA henüz tamamlanmadı; merge öncesi CI, Vercel/Pages preview ve QA durumu kontrol edilecek.


## Original visual assets — 15 Kasım 2026

- Added three original SVG diagrams: Homebrewing learning loop, Professional brewery record relationships, and an explicitly simulated fermentation profile.
- Integrated the Home and Business diagrams into `/how-it-works/` and the simulated profile into `/technical-principles/`.
- Accessible alt text and captions included; simulated values labelled as illustrative, not a product output or brewing recommendation.
- No stock imagery, unapproved product UI, or unsupported capability claims added.
- PR #62 merged to main after successful Astro/TypeScript, build and static route/metadata/link validation.
- GitHub Pages deployment triggered from merge; responsive browser QA remains pending.


## Visual asset library — phase 2 — 15 Kasım 2026

- Added target-vs-actual and measurement-source-context SVG diagrams; all example data and conceptual relationships are explicitly labelled.
- Technical Principles now presents fermentation, target-vs-actual and measurement-source visuals in a responsive gallery.
- Added `web-stratejisi/21-visual-asset-library.md` with visual classes, asset inventory, evidence rules, accessibility and publication checklist.
- No product UI, live integration, real batch data, brewing advice or unapproved capability claim is implied.
- PR #63 merged to main after successful Astro/TypeScript, build and static route/metadata/link validation. GitHub Pages deployment for the merge is queued; responsive browser QA remains pending.


## Handoff update — PR #63 merged

- Main commit: `85547a0a4cd33d2209a6dee9b32b7b764d3904d1`.
- PR #62 and #63 are merged. Main now contains five original SVG visuals and the visual asset library guide.
- PR #63 CI passed; GitHub Pages deployment run #43684298501 and main checks run #43684298471 were queued at the time of this handoff update.
- Vercel PR preview: `https://brewery-git-design-visual-library-phase-2-demleme.vercel.app/`; Vercel preview comment check passed, but actual browser QA has not been completed.
- Next: complete visual asset backlog in small batches, then run full responsive/browser/accessibility QA before release.


## Audience visual assets — phase 3 — 15 Kasım 2026

- Added a Homebrewer ingredient-context diagram: recipe specification → selected inputs → brew-day actuals → batch review.
- Added a Professional Breweries batch timeline: recipe version → batch plan → production record → vessel events → quality review.
- Both are explicitly labelled conceptual/roadmap visuals, not released workflows or general ERP claims.
- Updated the visual asset inventory and moved completed items out of the backlog.
- PR #65 merged to main after successful Astro/TypeScript, build and static route/metadata/link validation. New GitHub Pages deployment #43684462054 and main checks #43684462086 were queued.


## Handoff update — PR #65 merged

- Main commit: `91b197425c30fcdd4b0331b98d56d537eb7a7e90`.
- PR #65 added Homebrew ingredient-context and Professional Breweries batch-timeline visuals; CI and Pages deployment passed.
- PR #67 replaced the Homebrewers and Breweries hero placeholders with original editorial SVG illustrations; CI passed.
- Total original SVG assets in marketing site: nine (seven explanatory diagrams plus two audience hero illustrations).
- PR #63 and #65 GitHub Pages deployments and main checks completed successfully. PR #67 merge deployment/checks are queued.
- Actual responsive/browser QA remains pending; Vercel preview links are available per PR but have not been visually inspected at desktop/tablet/mobile widths.


## Editorial brewing illustrations — 15 Kasım 2026

- Replaced the Homebrewers hero placeholder with an original vector illustration of a homebrewing workspace (kettle, fermenter, hydrometer, malt and hops).
- Replaced the Breweries hero placeholder with an original vector illustration of stainless-steel brewery vessels and pipework.
- Added responsive figure styling and captions; no stock photos or product UI used.
- Updated the visual asset library inventory; homepage hero remains unchanged.
- Branch: `design/editorial-brewing-illustrations`; CI and responsive browser QA required before merge.


## Handoff update — PR #67 merged

- Main commit: `8d8826bdf9e574fc24bfbe30cded127ef2c3e1c5`.
- PR #67 merged after successful CI; both audience hero placeholders now use original editorial vector illustrations.
- Nine original SVG assets are now in the marketing site: seven technical/workflow diagrams and two hero illustrations.
- GitHub Pages deployment #43684614831 and main checks #43684614839 are queued.
- Homepage hero illustration remains unchanged. Responsive visual/browser QA remains pending.


## Responsive browser QA automation — 15 Kasım 2026

- Added Playwright 1.65.1 and Chromium browser smoke tests to the marketing-site CI workflow.
- Seven routes are checked at 1440 / 1024 / 768 / 390 / 320 px for overflow, route response, main/H1 structure, image alt/loading and browser page errors.
- Primary navigation interaction is tested at all five widths. CI retains screenshot artifacts for review.
- Cleaned the homepage hero illustration's stale placeholder/photography wording without changing its appearance.
- Updated WP-13 visual QA and visual inventory status; manual visual, keyboard, screen-reader, contrast and zoom review remains a release gate.
- Branch: `qa/playwright-responsive-smoke`; PR and CI validation pending.


## Handoff update — responsive browser QA branch

- Main base commit: `8e9e99310c6bf69dfca92146a5b142bc3a301b23` (PR #68 documentation handoff merged).
- Added Playwright-based responsive browser checks and screenshot artifacts for the marketing site. PR #69 pending.
- Nine public SVG assets and the homepage inline SVG are integrated; no photography or product UI has been introduced.
- Automated checks are not human visual QA. Manual review of screenshots, keyboard focus, screen-reader flow, contrast and 200% zoom remains open.


## Screenshot review finding — Business grid refinement

- Inspected the first Playwright screenshot artifact across all five viewport widths.
- The five-item Business workflow and record-model grids exposed a visually empty sixth cell because the grid container background showed through.
- Added a Business-only card treatment with explicit gaps and borders; Homebrewer six-step grid and homepage hero remain unchanged.
- Re-run browser QA passed (40/40); refreshed Business desktop/mobile screenshots confirm the empty grid cells are gone.


## Handoff update — PR #69 merged

- Main commit: `a3f33aa5905c3df5603fb35dbd9452137975049b`.
- PR #69 merged after successful Astro/TypeScript checks, production build, static route/metadata/link validation and Playwright Chromium responsive QA.
- Browser QA: 40/40 tests passed. Seven routes checked at 1440 / 1024 / 768 / 390 / 320 px; image alt attributes/loading, horizontal overflow, browser page errors, and primary navigation interaction covered.
- Screenshot artifact: `marketing-responsive-qa-screenshots` (23 full-page screenshots, 10.3 MB). First-pass review included Home, Homebrewers and Breweries desktop/mobile; the Business five-card grid's empty background cells were corrected and rechecked.
- GitHub Pages deployment #43687312428 and main checks #43687312488 are queued.
- Remaining release gates: manual keyboard/focus review, screen-reader flow, contrast, 200% zoom, and Vercel preview visual verification. Automated Chromium smoke tests do not close those gates.


## Accessibility audit automation — 15 Kasım 2026

- Added axe-core Playwright checks against WCAG 2.0/2.1/2.2 A/AA tags for seven public routes at desktop (1440 px) and mobile (390 px).
- Added keyboard checks for skip-link navigation, mobile menu disclosure and FAQ disclosure.
- Added a 720 px narrow-desktop layout proxy on Home, Homebrewers and Breweries. This is not a substitute for testing actual browser zoom at 200%.
- CI results will determine the next remediation batch; no claim of WCAG conformance until manual review is complete.
- Branch: `qa/axe-accessibility-audit`; PR and CI pending.


## Axe audit findings and first remediation — 15 Kasım 2026

- Initial axe run found two repeated WCAG contrast issues: the 11px homepage hero note and 9px footer metadata used `#77796f` at 3.96–3.98:1 against light backgrounds (AA target 4.5:1).
- It also found the horizontally scrollable Professional Breweries role table was not keyboard-focusable.
- Updated those small-text colours to `#66685e` (calculated contrast above 5:1 on both site surfaces); darkened small labels in the homepage hero SVG.
- Made the role table wrapper a labelled, focusable region and added a visible keyboard focus outline.
- PR #71 CI is re-running; further axe findings will be fixed before merge.


## Follow-up axe finding — Homebrewers table

- Re-run cleared all contrast violations and the Business role-table issue.
- The Homebrewers page uses the same horizontally scrollable table wrapper and also required keyboard focus; added a descriptive region name and `tabindex=0` there too.
- Awaiting final axe and keyboard test rerun.


## Handoff update — PR #71 merged

- Main commit: `b8520a2806938664f114218fe88eb1a82cbbfde8`.
- PR #71 merged after all CI checks passed, including 40 responsive route/viewport checks, 14 axe WCAG A/AA route scans, 3 keyboard interaction tests and 3 narrow-desktop (720px) layout proxy tests: 60/60 total.
- Axe findings fixed: small hero-note/footer metadata text contrast raised from 3.96–3.98:1 to above 5:1; both Homebrewers and Breweries horizontally scrollable tables are now labelled, focusable regions with visible focus styling.
- Automated WCAG-tagged scans are clean on the seven routes at 1440px and 390px. This is not a WCAG conformance certification or substitute for assistive-technology testing.
- Main checks #43687858228 and GitHub Pages deployment #43687858222 are queued.
- Manual release gates remain: screen-reader testing, complete keyboard/focus review, real 200% browser zoom, contrast review of embedded SVG text, and Vercel preview verification.


## Visual evidence diagrams — phase 4 — 15 Kasım 2026

- Added a Homebrewer fermentation observation timeline with simulated gravity/temperature values, day markers and explicit source/time context.
- Added a conceptual equipment calibration record model linking instrument, reference, observed reading, tolerance review and recorded status.
- Both visuals are explicitly caveated: no product-output, live-integration, automated recommendation or released-feature claim.
- Integrated into Homebrewers and Technical Principles pages, updated visual inventory and benchmark evidence register.
- Branch: `design/visual-evidence-phase-4`; CI and screenshot review pending.


- The two detailed SVGs now use labelled, keyboard-focusable horizontal scroll regions on narrow screens so chart labels remain readable instead of shrinking to tiny text. Responsive and axe tests will verify no page-level overflow and keyboard access.


## Handoff update — PR #73 merged

- Main commit: `205e67dbcd6272eb122705f8292f2796eb7a0137`.
- PR #73 merged after successful responsive and axe/keyboard CI: 60/60 tests passed.
- Added two original SVG assets: Homebrewers fermentation observation timeline and Technical Principles equipment calibration record model. Total public original SVGs: eleven, plus the existing inline homepage hero illustration.
- The two detailed diagrams use labelled, keyboard-focusable horizontal scroll regions on narrow screens to preserve label readability; responsive overflow and axe checks passed.
- GitHub Pages deployment #43688305665 and main checks #43688305663 are in progress.
- Manual screen-reader, full keyboard/focus, real 200% browser zoom, embedded SVG text contrast and Vercel preview verification remain release gates.


## Audience comparison matrix — 15 Kasım 2026

- Added a side-by-side Homebrewers vs Professional Breweries comparison table to `/how-it-works/`.
- Six dimensions: primary user, core unit of work, typical records, measurement context, main question and scope boundary.
- The table is explicitly roadmap/information-needs context, not a feature-availability promise. The mobile table region is labelled and keyboard-focusable.
- PR #75 merged; responsive, axe and keyboard CI passed (60/60).

- Added a homepage text link from the two audience cards to the new side-by-side comparison matrix.


## Comparison page screenshot review — 15 Kasım 2026

- First responsive screenshot review confirmed the comparison matrix reads as intended at desktop and mobile widths.
- It also exposed the same empty sixth-cell background in the five-step Business workflow on How It Works; changed those cards to separate bordered items with transparent grid gaps.
- Added a mobile scroll hint for the side-by-side matrix.
- Re-run responsive/axe checks passed (60/60); see PR #75 handoff.


## Handoff update — PR #75 merged

- Main commit: `2a1bc325d8827dba11ade7e7c6e19a75d2f0e974` (PR #76 documentation handoff merged).
- PR #75 merged after successful Astro/TypeScript, build, static validation and 60/60 responsive/axe/keyboard tests.
- Added the side-by-side Homebrewers / Professional Breweries matrix to `/how-it-works/#audience-comparison`, with a homepage entry link. Six comparison dimensions are included.
- Screenshot review fixed the five-step Business workflow's empty grid cell and added a mobile horizontal-scroll hint for the comparison table.
- Main checks #43688777093 and GitHub Pages deployment #43688777068 completed successfully.
- Remaining manual release gates unchanged: screen-reader, full keyboard/focus, real 200% zoom, embedded SVG text contrast, Vercel preview.


## Handoff update — final status 15 Kasım 2026

- Current main: `2a1bc325d8827dba11ade7e7c6e19a75d2f0e974`.
- PR #75 comparison matrix and homepage link are merged; 60/60 responsive/axe/keyboard checks passed.
- Main CI #43688777093 and GitHub Pages deployment #43688777068 succeeded.
- Vercel Preview continues to fail because the Vercel Hobby build-rate limit is exhausted; this is a hosting-plan limit, not a code/build failure. GitHub Pages is the current working preview.
- Current public original visuals: eleven SVG files plus the existing inline homepage hero illustration.
- Remaining manual release gates: screen-reader, complete keyboard/focus review, real 200% browser zoom, contrast review of embedded SVG text, and Vercel preview verification when the build limit resets or hosting is upgraded.


## Keyboard focus coverage — 15 Kasım 2026

- Added explicit Playwright assertions that the horizontally scrollable Homebrewers, Breweries and audience-comparison tables receive a visible 3px keyboard focus outline at mobile width.
- Updated stale homepage comments: the audience card artwork is original inline SVG, not a pending licensed-photo placeholder.
- Branch: `qa/keyboard-focus-coverage`; CI and axe rerun pending.


## Handoff update — PR #78 merged

- Main commit: `383efc77c03e3d740f1f0054ad6f8628818cecce`.
- PR #78 added real Tab-navigation checks for the three scrollable table regions (Homebrewers, Breweries and How It Works) and confirmed visible 3px focus outlines.
- Final PR CI passed 63/63 tests: 40 responsive checks, 14 axe scans, 3 disclosure/skip-link keyboard checks, 3 narrow-desktop layout proxies and 3 table-focus checks.
- Homepage audience artwork comments now correctly describe the inline SVGs as original editorial illustrations.
- Main checks #43689057629 and GitHub Pages deployment #43689057632 are queued.
- Vercel Hobby build-rate-limit remains the only hosting preview blocker; GitHub Pages is the current preview. Manual screen-reader and real 200% zoom review remain open.


## FAQ structured data — 15 Kasım 2026

- Changed the FAQ route JSON-LD from generic WebPage to FAQPage with Question / acceptedAnswer entries generated directly from the visible FAQ content.
- Extended the static site validator to ensure the FAQ schema question count matches the rendered FAQ items and every answer is present.
- No FAQ rich-result or ranking guarantee is claimed; this is structured semantic metadata.
- Branch: `seo/faq-structured-data`; CI and axe/browser checks pending.


## Handoff update — PR #80 merged

- Main commit: `0b727c66d0a2383d98bea2744e73355c58ceeb6a` (PR #81 documentation handoff merged).
- PR #80 merged with FAQPage JSON-LD generated from the visible FAQ data and static validation that question count/answers match the rendered content.
- PR CI #43689184058 passed: Astro/TypeScript, build, static validator, 40 responsive checks, 14 axe scans, 3 disclosure/skip-link keyboard checks, 3 narrow-desktop proxies and 3 table-focus checks (63/63 browser tests).
- Main checks #43689298960 and GitHub Pages deployment #43689298947 completed successfully.
- No FAQ rich-result or ranking guarantee is claimed. Manual screen-reader and actual 200% zoom checks remain open.


## Current handoff — 15 Kasım 2026

- Current main: `0b727c66d0a2383d98bea2744e73355c58ceeb6a`.
- PR #80 FAQPage structured data merged; static validator confirms 11 visible Q&A entries match the FAQ schema.
- Main checks #43689298960 and GitHub Pages deployment #43689298947 both succeeded after PR #80.
- Current preview: https://oguzbay-del.github.io/brewery/ . Vercel Preview remains blocked by the Hobby build-rate limit, not by site build/test failures.
- Visual inventory: eleven public original SVG assets plus the existing inline homepage hero and audience-card illustrations.
- Browser QA: 63/63 responsive, axe and keyboard tests passed. Remaining manual release gates: screen-reader, full keyboard/focus review, actual 200% browser zoom, and embedded SVG text contrast review.


## Handoff update — PR #82 merged

- Current main commit: `f584e389d5a3296f52a0656b657f1a3eb6e81f1d`.
- PR #82 synchronized the handoff and benchmark documents after the FAQPage structured-data release.
- Main build, static validation, 63/63 responsive/axe/keyboard tests and GitHub Pages deployment #43689298947 all succeeded on PR #80's code commit.
- GitHub Pages preview: https://oguzbay-del.github.io/brewery/ . Vercel preview remains blocked by the Hobby build-rate limit.
- Remaining manual gates: screen-reader testing and real 200% browser zoom; embedded SVG text contrast review is not covered by axe's full SVG rendering analysis.

## Handoff update — PR #138 and strategy sync — 15 November 2026

- Current main: `7ac3fbef80ced26b81869a90f7be7f6b57869b8d`.
- PR #138 merged: desktop and mobile navigation now expose the current page with `aria-current="page"`; the current item also has a visible accent style. Nested feature detail pages identify Product as the active section.
- PR #138 browser QA: 85/85 tests passed, including responsive checks and automated WCAG 2.2 AA scans. Static validation: 1,739 checks passed.
- Main CI run #43710054711 succeeded. GitHub Pages deployment #43710054632 succeeded on the same main commit.
- PR #139 merged the Business-first strategy implementation status and updated the strategy index.
- GitHub Pages preview: https://oguzbay-del.github.io/brewery/ . Vercel Preview remains blocked by the Hobby build-rate limit.
- Manual release gates remain: screen-reader testing, complete keyboard/focus review, real 200% browser zoom, final-domain metadata/social preview validation, production hosting/domain, product onboarding URL, pricing, privacy/consent and legal/contact decisions.

## Handoff update — PR #141 analytical-claim guardrails — 15 November 2026

- Current main: `53af627bdd9881efe3435d3b78a0d355dd2ee6b7`.
- PR #141 merged: unapproved analytical-model ideas are no longer presented as committed product features in yeast, device, quality, inventory, cost and reporting copy.
- The affected feature pages explicitly label yeast viability/hop-alpha estimates, device fault diagnosis, SPC, sensory correlations, ingredient-demand forecasts, expiry risk, tank bottleneck optimisation and detailed cost-variance attribution as “Under consideration”.
- Product-plan decisions remain unchanged: analytical models 1–3, 6 and 9–15 are still unapproved. The copy does not promote them as available or committed capabilities.
- Static validation: 1,745 checks passed, 0 failures. Responsive and automated WCAG 2.2 AA browser suite: 85/85 passed.
- Main CI run #43710624091 succeeded. GitHub Pages deployment #43710624041 succeeded on the same main commit.
- GitHub Pages preview: https://oguzbay-del.github.io/brewery/ . Vercel Preview remains blocked by the Hobby build-rate limit.
- Remaining release gates are unchanged: human screen-reader/keyboard/200% zoom review, final production domain and hosting, app onboarding URL, pricing, privacy/consent and legal/contact decisions.
