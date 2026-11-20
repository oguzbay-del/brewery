# GRISTORA — Cloudflare Pages Kurulum Özeti

**Son güncelleme:** 21 Kasım 2026  
**Repository:** [oguzbay-del/brewery](https://github.com/oguzbay-del/brewery)  
**Canlı alan adı:** [https://gristora.com](https://gristora.com)

## 1. Yapılanlar

- Cloudflare hesabı ve Pages tarafı hazırlandı.
- GitHub hesabı Cloudflare Pages ile yetkilendirildi.
- `oguzbay-del/brewery` repository'si Cloudflare Pages projesine eklendi.
- Kullanıcı teyidine göre `gristora.com` alan adı Cloudflare üzerinde çalışır durumda. GitHub Cloudflare Pages deployment check'i `bef1ed0` commit'i için başarılı döndü.
- Böylece GitHub repository'sinden Cloudflare Pages'e Git entegrasyonu üzerinden otomatik build/deploy akışı kurulmuş oldu. Cloudflare, bağlı repository'ye gönderilen commit'ler için deployment başlatabilir; production dışındaki dallar preview deployment üretebilir.

> Not: Cloudflare dashboard'una doğrudan erişimimiz olmadığı için deployment geçmişi, son build sonucu, DNS kayıtları ve tüm build ayarları bağımsız olarak API üzerinden doğrulanmadı. Alan adının çalıştığı bilgisi kullanıcı teyidine dayanır.

## 2. GRISTORA için build yapılandırması

Repository bir monorepo olduğu ve Astro sitesi `marketing-site/` altında bulunduğu için Pages yapılandırmasında aşağıdaki değerler kullanılmalı / dashboard'da teyit edilmeli:

| Ayar | Değer |
|---|---|
| Repository | `oguzbay-del/brewery` |
| Root directory | `marketing-site` |
| Framework preset | Astro |
| Production branch | `main` |
| Build command | `npm run build` |
| Build output directory | `dist` |
| Node.js | `22` |

Astro build komutu, projedeki build script'ine göre fotoğraf optimizasyonu ve sosyal paylaşım görseli üretimini de çalıştırır.

## 3. Deployment akışı

1. Geliştirme değişiklikleri GitHub'da ayrı bir feature branch üzerinde yapılır.
2. Pull request açıldığında Cloudflare Pages preview deployment üretir (repository içinden açılan PR'lar için).
3. Preview URL üzerinde görsel, responsive, SEO ve işlev kontrolleri yapılır.
4. Onaydan sonra PR birleştirilir.
5. `main` dalına gelen commit, Cloudflare Pages production deployment'ını tetikler ve bağlı production alan adını günceller.

**Production güvenliği:** `main` merge/push işlemleri ve canlı siteyi etkileyen değişiklikler, açık onay olmadan yapılmamalıdır. Vercel ve GitHub Pages yapılandırmaları ayrı sistemlerdir; Cloudflare entegrasyonu bunları otomatik olarak kaldırmaz veya eşitlemez.

## 4. Alan adı ve SEO notları

- Canlı adres: `https://gristora.com` (kullanıcı tarafından çalışır olduğu teyit edildi).
- `www.gristora.com` yönlendirmesi ayrıca doğrulanmalı.
- HTTPS ve sertifika tarayıcıda kontrol edilmeli; alan adının açılması tek başına tüm DNS/SSL yapılandırmasının denetlendiği anlamına gelmez.
- Google tarafından indekslenebilirlik için production HTML sayfalarında `noindex` direktifi bulunmamalı; robots.txt ve canonical URL'ler de kontrol edilmeli.
- Cloudflare Pages preview adresleri ve GitHub Pages test ortamları arama motorlarına indeksletilmemeli.

## 5. SEO ve dil değiştirme düzeltmeleri

- [x] İngilizce ve Türkçe içerik sayfalarında indeksleme yalnızca `gristora.com` canonical host'u için açılacak şekilde düzenlendi (`index, follow`).
- [x] Preview / test host'ları `noindex, nofollow` korumasında bırakıldı; `robots.txt` production dışındaki host'larda taramayı engelliyor.
- [x] Ortak `SiteHeader.astro` bileşeninde EN/TR dil bağlantısı yer alıyor; bağlantı aynı sayfanın karşı dildeki karşılığına yönleniyor.
- [x] Production build ve Cloudflare Pages deployment tamamlandı (`bef1ed0`, Deploy successful).
- [x] Build doğrulaması: production için EN/TR sayfalar `index, follow`; preview ve GitHub Pages test host'ları `noindex, nofollow`.
- [x] Production sitemap'ine Türkçe ve İngilizce sayfalar dahil; preview sitemap'leri Türkçe sayfaları dışarıda bırakıyor.
- [x] CI, robots.txt, canonical URL, sitemap hedefleri ve karşılıklı EN/TR dil bağlantılarını doğruluyor.
- [ ] `www.gristora.com` → `gristora.com` yönlendirmesini doğrula.
- [x] Cloudflare Pages production deployment başarılı; yayınlanan commit `bef1ed0`.
- [ ] `www.gristora.com` → `gristora.com` yönlendirmesini tarayıcı/HTTP üzerinden ayrıca doğrula.

## 6. Resmî Cloudflare kaynakları

- [Pages Git integration](https://developers.cloudflare.com/pages/configuration/git-integration/)
- [Git integration ile Pages kurulumu](https://developers.cloudflare.com/pages/get-started/git-integration/)
- [Astro'yu Cloudflare Pages üzerinde yayınlama](https://developers.cloudflare.com/pages/framework-guides/deploy-an-astro-site/)
- [Preview deployments](https://developers.cloudflare.com/pages/configuration/preview-deployments/)

---

Bu dosya kurulumun ve kararların kısa operasyonel kaydıdır. Cloudflare Pages deployment check'i production commit'i için başarılıdır; `www` yönlendirmesi ayrıca doğrulanmayı beklemektedir.
