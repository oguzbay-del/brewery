# GRISTORA — Çalışma Kaydı
Son güncelleme: 21.11.2026

## Aktif çalışma
- Repo: https://github.com/oguzbay-del/brewery
- Web sitesi: https://gristora.com
- Site dizini: `marketing-site/`
- Teknoloji: Astro
- Aktif geliştirme dalı: `work/gristora-modular-product-presentation-2026-11-21`
- Ana PR: https://github.com/oguzbay-del/brewery/pull/178
- PR #179: W12 QA kanıtları; açık durumda olduğu biliniyor.
- PR #180: SEO/dil değiştirici çalışmaları PR #178 dalına aktarılmış; #180 kapalı. Bu değişikliklerin `main` dalına alındığı varsayılmamalı.

## Tamamlanan / dalda bulunan işler
- Üretim sitemap'ine Türkçe sayfaların eklenmesi; preview ortamlarının sitemap kapsamı dışında tutulması.
- Dil değiştirici bağlantısının base path'i koruyacak şekilde düzenlenmesi.
- Cloudflare Pages preview derlemelerinde canonical host'un preview URL olması; production canonical ayarının korunması.
- Site doğrulama betiğinin iki dilin indeksleme politikasına göre güncellenmesi.
- 320px responsive testlerinde taşan elemanları teşhis etmek için test çıktısına `overflowers` ayrıntısı eklenmesi.

## Açık teknik konu
- Responsive browser QA'da 320px genişlikte sayfalarda yaklaşık 18px yatay taşma ve karşılaştırma tablosunda başarısız testler görülmüştü.
- Henüz kök neden kesinleşmedi; genel `overflow-x: hidden` bandı ile sorunu gizleme.
- Sonraki adım: PR #178 dalındaki en güncel CI/test loglarını incele, `overflowers` çıktısından gerçek taşan elementi belirle, doğru CSS düzeltmesini yap ve testleri yeniden çalıştır.

## CI / deploy durumu
- PR #178 dalındaki bilinen son commit: `d81aacc6b9b77836302224398647cad331424f0a` (devam etmeden önce güncel dal SHA'sını doğrula).
- Bu commit için bilinen workflow run: `44836020483`; son kayıtta hâlâ çalışıyordu. Güncel durumunu kontrol et.
- Cloudflare Pages build ile Cloudflare Workers build check'lerini birbirine karıştırma; Workers check hatasının Pages üretim deploy'unu etkileyip etkilemediğini loglardan doğrula.
- Üretim deploy'u henüz doğrulanmış değil. PR #178'in `main` dalına merge edildiğini veya production'a çıktığını varsayma.
- Cloudflare Pages yapılandırması: repo `oguzbay-del/brewery`, root `marketing-site`, build `npm run build`, output `dist`, Node 22, production branch `main`.
- Kullanıcı `gristora.com` custom domain'ini Cloudflare Pages'e ekledi.
- Gereksiz Vercel deploy'larından kaçın; bu proje için Cloudflare Pages Git entegrasyonunu kullan.

## Sıradaki işler
1. PR #178 ve ilgili CI run'ın güncel durumunu kontrol et.
2. Responsive taşmanın kaynağını teşhis edip CSS'i düzelt.
3. Build, SEO/i18n validator ve responsive testlerini yeniden çalıştır.
4. Gerekli değişiklikleri bu geliştirme dalına commit et.
5. Bu dosyayı ve `yaptığımız.md` proje kaydını güncel tut.
6. Tüm gerekli kontroller yeşil olduğunda PR #178'i `main`'e alma ve Cloudflare production deploy durumunu doğrulama adımına geç. Başarısız kontroller varken production'a gönderme.

## Ürün kapsamı notu
GRISTORA dokuz modüllü bir brewing workspace olarak anlatılır: Core, Devices, Stock, Production, Quality, Cost & Reports, Tax & Compliance, Team, Connections. Ürünü tam ERP olarak tanımlama; canlı olmayan özellikleri mevcut özellikmiş gibi sunma.

## Çalışma tercihi
Kullanıcı devamlılık, paralel ilerleme ve gerçek uygulama bekliyor. Aynı bilgileri tekrar sorma; her oturumda önce bu dosyayı ve GitHub'daki güncel durumu kontrol ederek devam et. Marka/domain dış araştırma adımlarını kullanıcı yeniden istemedikçe açma.
