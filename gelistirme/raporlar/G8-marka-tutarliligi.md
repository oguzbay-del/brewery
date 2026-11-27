# G8 — Marka ve ürün tutarlılığı

27 Kasım 2026 — Taslak inceleme. Referans: [ADR-0012](../../adr/0012-urun-markasi.md), onaylı logo [#148](https://github.com/oguzbay-del/brewery/pull/148), [G5 #429](https://github.com/oguzbay-del/brewery/pull/429) ve [G6 #431](https://github.com/oguzbay-del/brewery/pull/431) (ikisi de birleştirilmiş).

## 1. Uyumsuzluklar ve öneriler

| Bulgu | Kaynak | Öneri |
|---|---|---|
| iOS görünen adı GRISTORA, ancak uygulama bundle kimliği hâlâ `app.demleme.Demleme`. ADR-0012 §4 hedefi `com.gristora.app`. | [ios/project.yml](../../ios/project.yml), ADR-0012 §4 | İlk TestFlight yüklemesinden önce uygulama ve test hedeflerinin kimliklerini ADR ile eşleştir; imzalama, Apple giriş ve universal link etkilerini Claude ile doğrula. İç proje ve hedef adlarının değişmesi gerekmez. |

## 2. Doğrulanmış ortak öğeler

| Öğe | Web uygulaması | iOS | Pazarlama |
|---|---|---|---|
| Görünen ad | [index.html](../../web/index.html): GRISTORA; [manifest](../../web/public/manifest.webmanifest): name ve short_name GRISTORA | [project.yml](../../ios/project.yml): INFOPLIST_KEY_CFBundleDisplayName GRISTORA | [BRAND-ASSETS.md](../../marketing-site/BRAND-ASSETS.md): GRISTORA |
| Favicon | [favicon.svg](../../web/public/favicon.svg) | AppIcon ayarı var; varyantların cihazda görsel doğrulaması açık | [favicon.svg](../../marketing-site/public/favicon.svg); web uygulamasındaki SVG içeriğiyle birebir aynı |
| Logo | #429: başlık sembolü ve duyarlı yazı | #431: açık, koyu ve renklendirilmiş ikon, launch screen; cihaz testi henüz doğrulanmadı | [BRAND-ASSETS.md](../../marketing-site/BRAND-ASSETS.md): #148 onaylı varlıklar; gerçek rota kullanımı ayrıca incelenecek |

## 3. Ürün iddiaları

İddiaların rota bazlı envanteri ve her birinin uygulama/API veya ISLER satırı ile eşlemesi henüz tamamlanmadı. Bu taslak ürünün yayınlanabilirliğini veya özelliklerin mevcut olduğunu onaylamaz.

İnceleme devamında EN/TR rotaları, özellik metinleri, ekran görüntüleri, sunucu uçları ve gerçek kullanıcı akışları kontrol edilecek. Pazarlama metni veya uygulama kodu bu rapor kapsamında değiştirilmez.
