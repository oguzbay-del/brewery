# UIQ-01: Mevcut web ve iPad ekranlarının görsel kalite denetimi

**Tarih:** 27 Kasım 2026 · **Yapan:** Claude (tarama Sonnet alt ajanı, inceleme Opus) · **Program:** ADR-0016, `gelistirme/uiq/README.md`

## Yöntem

- **Web:** üretim yapısı (`npm run build` + `vite preview`), e2e testlerindeki API sahte yanıtları. 5 rota (`/`, `/recipes`, `/batches`, `/containers`, `/settings`) × 7 test görüntü alanı × 4 tema = 140 sayfa; ayrıca 640×360 ve 320×180 CSS px yeniden akış (açık tema). Ölçülenler: yatay taşma, görüntü alanı dışına çıkan öğe, metin kırpılması, etkileşimli öğe çakışması, axe-core ihlalleri. Ekran görüntüsü 1366×1024 açık ve koyu.
- **iPad:** Linux'ta Swift derlenmediği için kod incelemesi. Simülatör görüntüsü UIQ-11 ile CI'da alınacak.

## Web bulguları

| # | Bulgu | Önem | Durum |
|---|---|---|---|
| 1 | **Yüksek karşıtlık + koyu sistem temasında ekran okunmuyor.** `:root[data-theme="contrast-dark"]` bloğu yalnız 5 token'ı (sep, label2, tint, tint-fill, tint-ink) yazıyordu; zemin ve metin açık temada kalıyordu. Karşıtlık 1.15–1.59:1; her rotada ve her görüntü alanında axe `color-contrast` (35 kayıt). | Yüksek | **Düzeltildi** (bu iş): `web/scripts/gen-tokens.mjs` bloğa önce koyu temel renkleri, sonra contrast-dark değerlerini yazar. Kalıcı test UIQ-10 matrisinde (contrast-dark dahil). |
| 2 | `/containers` 320 CSS px'te 4 px yatay taşıyor (tablo `Status` sütunu x=325, bölüm ve form x=324). %400 yakınlaştırmada yeniden akış ihlali (WCAG 1.4.10). 640 px'te sorun yok. | Orta | UIQ-15 (UIQ-06 kabuğundan sonra yeniden ölçülür) |
| 3 | Geniş ekranda düzen boş: `/containers` ~1 040 px genişlikte sola yaslı; 1920 ve 2560 px'te ekranın yarısı boş. Diğer rotalarda yalnız başlık ve not var. Kenar çubuğu ve Inspector yok. | Orta (tasarım) | UIQ-06 |
| 4 | Üst gezinme çubuğu: kenar çubuğu yerine yatay bağlantılar; tema seçimi başlıkta (Ayarlar'da olmalı). | Orta (tasarım) | UIQ-06, UIQ-13 |
| 5 | Ekrana özel bileşenler: `.module-message`, `.module-banner`, `.module-notice` aynı işi yapıyor (Callout); `.app-shell__theme-control select` kendi stilini yazıyor; `44px` ham değer (`tap-min` yerine); odak halkası kuralı iki yerde. | Düşük | UIQ-13 |
| 6 | Kabuk `@media (max-width: 400px)` ile görüntü alanına bakıyor; bileşenlerde container query yok. | Düşük | UIQ-06 |
| 7 | 60 "kırpılma" kaydı görsel olarak gizli erişilebilirlik metni (`containers__visually-hidden`, `#containers-counts-heading`). Kasıtlı; test izin listesine `.visually-hidden` deseni olarak girer. | – | UIQ-10 (yanlış alarm kuralı) |

Temiz çıkanlar: açık, koyu ve yüksek karşıtlık temalarında 7 görüntü alanında axe ihlali yok; 820–2560 px arasında taşma yok; etkileşimli öğe çakışması yok.

## Token ve üretim bulguları

| # | Bulgu | Durum |
|---|---|---|
| 8 | Web üreticisi `size` grubunu (`tap-min`, `badge-size`, `toggle-width`) CSS'e yazmıyordu. | Düzeltildi (UIQ-03a) |
| 9 | iOS üreticisi yalnız renk üretiyordu; boşluk, köşe, boyut Swift'te yoktu. | Düzeltildi (UIQ-03a, `DemlemeMetric`) |
| 10 | Katman, kenar çubuğu/Inspector genişliği, tablo satırı, yoğunluk, hareket token'ı yoktu. | Düzeltildi (UIQ-03a) |
| 11 | Grafik serisi paleti yok. | UIQ-03b |

## iPad bulguları (kod incelemesi)

| # | Bulgu | Durum |
|---|---|---|
| 12 | Kök `TabView` + her sekmede `NavigationStack`: iPad'de tek sütun, kenar çubuğu yok. iPadOS 18 `.sidebarAdaptable` kullanılmıyor. | UIQ-07 |
| 13 | `NavigationSplitView` ve `.inspector` hiç yok. | UIQ-07 |
| 14 | Sekme seçimi `@State`; sahne geri yüklenince ve yön değişiminde korunması için `@SceneStorage` yok. | UIQ-07 |
| 15 | `ModuleUpgradeMessage` ham `spacing: 8` ve `.padding()`; `ContentUnavailableView` kullanılabilir. `ModuleNoticeView` Callout'un kopyası. | UIQ-13 |
| 16 | Ekran görüntüsü ve `performAccessibilityAudit()` testi yok. | UIQ-11 |
| 17 | `TARGETED_DEVICE_FAMILY` "1,2"; iPhone saha görünümü (K4, #391) sürüyor. Program iPhone'u optimize etmez, silmez (ADR-0016 › 6). | Bilgi |

## Sonuç

Tek ciddi kusur (yüksek karşıtlık koyu tema) bu işte düzeltildi. Diğer bulgular eksik yapıdan (kabuk, Kit, test) geliyor ve UIQ iş listesinde karşılığı var. Ekran sayısı az olduğu için taşıma maliyeti düşük; yeni Aşama 3 ekranları Kit ile yazılmalı.
