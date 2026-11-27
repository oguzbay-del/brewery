# ADR-0016: Arayüz kalite programı (UIQ), GRISTORA UI Kit ve Görsel Kalite Kapısı

**Durum:** kabul · **Tarih:** 27 Kasım 2026 · **Karar:** Claude (Opus), owner talebine dayanarak (Oğuz, 27.11.2026, "Apple-Inspired UI Excellence Program v3") · **İş:** UIQ-01…UIQ-16

Program tanımı, bileşen standardı ve iş listesi: `gelistirme/uiq/README.md`. Bu kayıt yalnız kararları ve gerekçeleri tutar.

## Bağlam

- Aşama 3 (web, Home MVP ekranları) yeni açıldı; Aşama 4 iOS. Ekran sayısı az: web'de kabuk, kaplar, modül ayarı (~500 satır CSS); iOS'ta sekme kabuğu, tarama ve modül ekranları (~1 700 satır Swift). UI Kit'i şimdi kurmak, yüzlerce ekranı sonradan taşımaktan ucuzdur.
- Tasarım sistemi (`tasarim-sistemi/`) zaten Apple yaklaşımını, üç temayı, ISA-101 operatör görünümünü ve 11 bileşeni (AlarmMark, BrewReturnBar, Button, Callout, Cover, GroupedList, NavBar, SegmentedControl, SummaryCard, TabBar, Toggle) tanımlıyor. Web'de bunların React karşılığı yok; ekranlar kendi CSS'lerini yazıyor.
- Denetimde görülen boşluklar (ayrıntı: `gelistirme/raporlar/UIQ-01-gorsel-denetim.md`):
  - Web token üreticisi `size` grubunu CSS'e yazmıyor; iOS üreticisi yalnız renk üretiyor (boşluk, köşe, boyut Swift'e gelmiyor).
  - Katman (z-index), kenar çubuğu ve Inspector genişliği, tablo satır yüksekliği, veri yoğunluğu, hareket süresi token'ı yok.
  - Web kabuğu üst gezinme çubuğu; kenar çubuğu + içerik + Inspector düzeni yok; `@media (max-width: 400px)` ile görüntü alanına bağlı.
  - iPad'de `TabView` + `NavigationStack` tek sütun; `NavigationSplitView` ve `inspector` kullanılmıyor.
  - Görsel regresyon testi yok; erişilebilirlik testi yalnız web'de ve bazı ekranlarda.
- Owner istekleri: React Aria Components tek yeni dış UI kütüphanesi; listelenen paketler (MUI, Tailwind, Radix, Storybook, TanStack Table, ECharts, Swift Snapshot Testing vb.) kapsam dışı; ek maliyet 0 USD; iPhone ve küçük iPad bu programın dışında.

## Karar

### 1. İki platform, tek standart

- **GRISTORA React UI Kit:** `web/src/kit/`. **GRISTORA SwiftUI Kit:** `ios/Demleme/Kit/`. İkisi aynı bileşen adlarını ve aynı token'ları kullanır; görsel olarak aynı aile, piksel kopyası değil.
- Tek token kaynağı `tasarim-sistemi/tokens.json`. Web CSS'i `web/scripts/gen-tokens.mjs`, Swift dosyaları `ios/scripts/gen_colors.py` üretir; üretilen dosya elle değişmez.
- iOS Kit'i önce sistem bileşenini kullanır (`Button` + `ButtonStyle`, `List`, `Table`, `Toggle`, `Picker`, `ContentUnavailableView`, `.inspector`, `NavigationSplitView`). Kit yalnız sistemde olmayanı (AlarmMark, SummaryCard, StatusBadge, BrewReturnBar) ve stil uzantılarını ekler.
- Owner'ın listesindeki ad kavramsaldır. Aynı işi yapan bileşenler birleştirilir (eşleme: `gelistirme/uiq/README.md` › Bileşen envanteri). Örnek: PrimaryButton/SecondaryButton/IconButton → `Button` varyantları; WarningBanner → `Callout`; EmptyState/ErrorState/LoadingState → `StateView`; MetricCard → `SummaryCard`. Alana özgü bileşen (TankCard, takvim şeridi) Kit'e değil özellik koduna girer ve Kit bileşenlerinden kurulur.

### 2. React Aria Components

- Sürüm `react-aria-components@1.22.1` (Apache-2.0; React 19 eşdüzey bağımlılığı uyumlu; 9 Ekim 2026 yayını). Sürüm sabitlenir (`^` yok), güncelleme Claude incelemesiyle.
- Yalnız davranış, erişilebilirlik, klavye ve konumlandırma için. Hiçbir varsayılan stil kullanılmaz; görünüm `web/src/kit/*.css` ve token'lardan gelir (`data-*` durum seçicileri: `[data-selected]`, `[data-focus-visible]`, `[data-disabled]`, `[data-invalid]`, `[data-pending]`).
- Ekran kodu React Aria'yı doğrudan içe aktarmaz; yalnız `web/src/kit/` aktarır. ESLint `no-restricted-imports` kuralı bunu denetler. Böylece kütüphane değişirse yalnız Kit değişir.
- Geçişli bağımlılıklar (`react-aria`, `react-stately`, `@internationalized/date`, `@internationalized/string`) kabul edilir; ayrı doğrudan bağımlılık olarak eklenmez.
- Önce pilot (UIQ-08): `Button`, `Switch` (Toggle), `Select`, `Dialog`. Pilot ölçütü: paket boyutu artışı gzip ≤ 60 KB, axe temiz, klavye testi geçer, Vite 8 ve TypeScript 6 ile derleme temiz. Ölçüt tutmazsa yaygınlaştırma durur ve owner'a sorulur.

### 3. Web ikonları ve grafikler

- SF Symbols ve Apple yazıları web'e gömülmez. Web için GRISTORA'nın kendi küçük SVG ikon seti `web/src/kit/icons/` altında, SF Symbols'e benzemeyen özgün çizimle, 24 px ızgara ve `currentColor` ile çizilir. Dış ikon paketi eklenmez.
- Grafik kütüphanesi eklenmez. Web'de `ChartContainer` başlık, metin özet ve tablo alternatifi taşıyan bir çerçevedir; çizgi ve çubuk grafik yalın SVG ile çizilir. iOS'ta Swift Charts. Grafik verisi sunucudan hazır gelir; istemci hesap yapmaz, yalnız ölçekler.

### 4. Uyarlanır düzen

- **Web:** `AppShell` = Sidebar + Content + Inspector, CSS Grid, `minmax(0, 1fr)`, `min-inline-size: 0`, mantıksal özellikler. Bileşenler `@container` sorgusuyla kendi alanlarına göre uyarlanır; yalnız kabuk görüntü alanı genişliğine bakar.
  - Genişlik ≥ `bp-inspector` (1060 px): üç sütun.
  - `bp-sidebar` (740 px) ≤ genişlik < `bp-inspector`: Inspector sheet olur (sağdan kayan, `z-sheet`).
  - Genişlik < `bp-sidebar`: kenar çubuğu da katman olur (`z-sidebar-overlay`). Bu, 820 px dikey iPad'de içeriği sıkıştırmak yerine paneli taşımaktır.
- `overflow: hidden` ile taşma gizlenmez. Kaydırma yalnız tanımlı bölgelerde olur (tablo gövdesi, kenar çubuğu listesi, Inspector) ve o bölge `data-scroll-region` taşır; geometri testleri bu bölgeleri ayrı denetler.
- Katmanlar yalnız `z` token'larıyla. Menü ve popover React Aria ile konumlanır, görüntü alanına çarpınca ters yöne açılır.
- **iPad:** kök `TabView` iPadOS 18'in `.sidebarAdaptable` stiline geçer (iPad'de kenar çubuğu, iPhone'da sekme çubuğu; iPhone kodu silinmez). Liste → ayrıntı ekranları `NavigationSplitView`, yardımcı panel `.inspector`. Seçim ve düzenleme durumu `@SceneStorage` ve görünüm modelinde tutulur; yön değişince kaybolmaz. Sabit cihaz genişliği, `offset` ve mutlak konum kullanılmaz; `ViewThatFits`, `ScaledMetric`, `safeAreaInset`.
- Liquid Glass yalnız sistemin gezinme katmanında ve yalnız iOS 26 ve sonrasında (asgari hedef iPadOS 18). İçerik, tablo ve operatör görünümü camsızdır.

### 5. Görsel Kalite Kapısı (CI)

- **Web** (`web.yml` içinde ayrı bir `visual` işi): Playwright + axe-core, mevcut sürümler. Kontroller: geometri (çakışma, taşma, kırpılma, hizalama), `toHaveScreenshot` karşılaştırması, axe (WCAG 2.2 AA etiketleri), üç tema + yüksek karşıtlık koyu, TR ve EN, uzun metin, boş/yoğun/yükleniyor/hata durumları, 7 test görüntü alanı ve %200/%400 yakınlaştırma yeniden akışı.
- Test yüzeyi: üretim yapısına girmeyen bir **Kit galerisi** (`web/kit.html`, `vite build --mode kit`, ayrı çıktı dizini) ve gerçek ekranlar (API sahte yanıtlarıyla). Storybook yok.
- Referans görüntüler yalnız CI'da, sabit ortamda üretilir: sabit `ubuntu-24.04`, sabit Playwright Chromium, `deviceScaleFactor: 1`, animasyon kapalı, sabit saat ve yazı tipi. Yerelde üretilen referans kabul edilmez.
- Referans elle tetiklenen bir iş (`workflow_dispatch`) ile yapı çıktısı olarak üretilir; Claude görüntüleri inceler ve commit eder. **Referans dosyaları (`**/*-snapshots/**`, `ios/DemlemeUITests/Baselines/**`) C sınıfıdır**; ChatGPT değiştiremez, otomatik güncelleme yoktur. Bu kural `marketing-site/CHATGPT-SCOPE.md` › Permission classes'a eklenir.
- Kapı önce rapor modunda çalışır (başarısızlık işi kırmızı yapar ama zorunlu kontrol değildir). Referanslar iki hafta kararlı kalınca zorunlu kontrol olur; bunu owner dal koruma ayarında açar. Mevcut CI güvenlik ve merge kuralları değişmez.
- **iPad** (`ios.yml` içinde ayrı bir hedef): XCTest + XCUITest, `XCUIScreenshot` ve `XCTAttachment`, `performAccessibilityAudit()`. Simülatörler: iPad Pro 11 inç ve iPad Pro 13 inç, yatay ve dikey; sabit Xcode ve iOS çalışma zamanı. Karşılaştırma için test hedefinde küçük bir yerel yardımcı (CoreGraphics piksel farkı, eşik). Dış anlık görüntü kütüphanesi yok.
- Kasıtlı katmanlar (modal, popover, yapışkan araç çubuğu, `data-scroll-region`) testte izin listesiyle tanımlanır; yanlış alarm vermez.
- Otomatik testler elle yapılan görsel ve erişilebilirlik incelemesinin yerine geçmez. Her aşama kapısında Claude kritik ekranları elle inceler; cihaz testi owner'dadır.

### 6. Hedef cihazlar ve kapsam sınırı

- Web test görüntü alanları: 820×1180, 1180×820, 1024×1366, 1366×1024, 1280×720, 1920×1080, 2560×1440. Chrome, Edge, Safari'nin güncel sürümleri. CI'da Chromium ve WebKit (Playwright'in kendi WebKit'i, ek maliyet yok); Edge Chromium ile temsil edilir.
- iPad: 11 ve 13 inç, iki yön.
- **iPhone ile fark:** plan iPhone'u kapsıyor (plan › iPhone kapsamı; K4 saha görünümü, #391). Bu program iPhone'u optimize etmez ama iPhone kodunu, hedefini ve testlerini silmez; `TARGETED_DEVICE_FAMILY` "1,2" kalır. iPhone'u bozan Kit değişikliği iPhone UI testinde yakalanır. Web'in 400 px kontrolü konseptte sürer; ürün web'inde dar ekran için yalnız WCAG yeniden akışı (320 CSS px) korunur.

### 7. Mimari sınırlar

- Rust sunucu, hesap motoru, OpenAPI sözleşmesi ve iş kuralları değişmez. İstemcide formül yok; birim dönüşümü dahil sayısal değeri sunucu verir (birim dönüşümünün istemcide gösterim olarak yapılıp yapılamayacağı: to be verified, plan › Birimler).
- Uygulanmamış özellik var gibi gösterilmez: galeri ve testlerdeki örnek veri yalnız test yapısındadır.
- Yeni ücretli hizmet yok. Yeni dış bağımlılık Claude mimari onayı ister (React Aria dışında).

### 8. İş bölümü

- Claude: mimari, token ve bileşen standardı, C sınıfı inceleme, referans görüntü onayı, kritik tasarım incelemesi (Opus); rutin koordinasyon Sonnet, etiket ve sınıflandırma Haiku.
- ChatGPT: B ve A sınıfındaki geliştirme ve test işleri (`web/src/kit/**`, `web/e2e/**`, galeri), `assigned-chatgpt` issue'larıyla.
- iOS kodu C sınıfıdır; Claude Sonnet alt ajanıyla yazar, owner cihazda dener.

## Sonuçlar

- Yeni ekranlar Kit dışı bileşen ve ekrana özel renk, boyut, z-index yazmaz; ESLint ve stylelint olmadan, `web/src/kit/` dışındaki CSS'te ham `px` renk ve z-index değerini bulan küçük bir betik (`web/scripts/check-css.mjs`, UIQ-06) bunu denetler.
- Tasarım sistemi README'si ve yayını (Artifact) yeni token grupları ve "Uyarlanır düzen" bölümüyle güncellenir (UIQ-03).
- Görsel referansların bakımı Claude'a iş yükü getirir; buna karşılık görsel hata ana dala girmeden yakalanır.
- React Aria pilotu ölçütü tutmazsa Kit bileşenleri yerel HTML üzerinde kalır; ekran kodu Kit'e bağlı olduğu için bu değişiklik ekranları etkilemez.
