# ADR-0012: Üründe GRISTORA markası

**Durum:** kabul · **Tarih:** 25 Kasım 2026 · **Karar:** Claude (Opus), owner kararlarına dayanarak (Oğuz, 20.11 ve 25.11.2026) · **İş:** G1

ISLER.md'de bu iş için `adr/0010-urun-markasi.md` adı yazılıydı. 0010 numarası Sentry kararına (ADR-0010) verildiği için bu kayıt 0012 numarasını alır.

## Bağlam

- Ürün adı GRISTORA, alan adı gristora.com (owner, 20.11.2026). Marka ve alan adı hukuki incelemesi (#146) sürüyor; alan adını owner satın alacak (O7).
- Uygulamanın (iOS ve web) kendi tasarım sistemi var; pazarlama sitesinin (`marketing-site/`) tasarımı ayrı. İkisinin ortak öğeleri yalnız ad, logo, uygulama ikonu ve marka vurgu rengi.
- Onaylı logo dosyaları `marketing-site/public/brand/` altında (`marketing-site/BRAND-ASSETS.md`). Marka paleti: Forest Green `#0B2F26`, Amber Gold `#F4A261`, Stone Beige `#F7F6F3`, Slate Gray `#475569`.
- Uygulama tasarım sisteminin tek vurgu rengi "mayşe amberi": açık temada yazı için `tint` `#985600`, geniş alan için `tint-fill` `#e8901f`; koyu temada ikisi de `#f0a03c`; yüksek karşıtlık temalarında `#8a4d00` ve `#ffc266`. Okunan her metin en az 4.5:1 karşıtlıkta.
- Koddaki görünen adlar hâlâ eski: iOS `CFBundleDisplayName: Demleme`, bundle kimliği `app.demleme.Demleme` (yer tutucu), web `<title>Demleme</title>`, OpenAPI `info.title: api`, e-posta göndereni `no-reply@demleme.local`. Tam liste: `gelistirme/raporlar/G2-ad-taramasi.md`.

## Karar

### 1. Ad her yerde

| Yer | Değer |
|---|---|
| iOS ana ekran adı (`CFBundleDisplayName`) | GRISTORA |
| App Store adı ve alt başlığı | Ad "GRISTORA". Alt başlık App Store kaydı açılırken owner tarafından seçilir (Aşama 4). |
| Web uygulaması sayfa başlığı | `GRISTORA`; alt sayfalarda `<sayfa adı> · GRISTORA` |
| Web manifest | `name: GRISTORA`, `short_name: GRISTORA` |
| E-posta göndereni | `GRISTORA <no-reply@gristora.com>` (yerelde ve testte `no-reply@gristora.local`) |
| OpenAPI `info.title` | `GRISTORA API` |
| Push bildirimi ve Live Activity başlığı | Sistem uygulama adını gösterir; metinde ad tekrarlanmaz |
| Yazdırılan formlar (brew sheet, etiket) | Altbilgide küçük "GRISTORA" yazısı ve sembol |

"Brewing Intelligence" tanımlayıcısı ve "Clarity for every brew." sözü pazarlama sitesine aittir; uygulama içinde kullanılmaz. Uygulama, gösteremediği bir yeteneği ima eden slogan taşımaz.

### 2. Alan adları

| Alt alan | Görev |
|---|---|
| `gristora.com` | Pazarlama sitesi (ayrı iş akışı) |
| `app.gristora.com` | Web uygulaması |
| `api.gristora.com` | `/v1` API, ingest webhook'ları, `/internal` (oauth2-proxy arkasında, ADR-0009) |
| `auth.gristora.com` | Apple ve Google giriş dönüş adresleri; ileride MCP için OAuth yetkilendirme sunucusu (M1) |
| `mail.gristora.com` | E-posta gönderim alt alanı (SPF, DKIM, DMARC) |

Oturum çerezi `app.gristora.com` ana bilgisayarına bağlıdır (`Domain` özniteliği yok); CORS yalnız `app.gristora.com` kaynağına açılır. Universal link dosyası (`apple-app-site-association`) `app.gristora.com` altında yayımlanır. DNS ve yapılandırma G7'de, alan adı alındıktan sonra yapılır. Production'a geçiş owner onayındadır.

### 3. Ortak marka öğeleri ve vurgu rengi

- **Logo:** Uygulama başlığında `gristora-symbol.svg` ve yanında metin olarak "GRISTORA" kullanılır. Tam logo (tanımlayıcılı) uygulamada kullanılmaz.
- **Uygulama ikonu:** Sembol, Forest Green zemin üzerinde Amber Gold. Açık, koyu ve renklendirilmiş (tinted) sürümler Apple ikon kurallarıyla üretilir (G6).
- **Vurgu rengi:** Marka Amber Gold `#F4A261`, uygulamanın geniş alan vurgusu olur. Yazı vurgusu her temada karşıtlık şartını sağlayan koyu veya parlak tonda kalır.

| Tema | `tint-fill` (geniş alan) | `tint` (yazı, ikon) | `tint-ink` (dolgu üstü yazı) |
|---|---|---|---|
| Açık | `#F4A261` (yeni) | `#985600` (aynı) | `#1a1206` (aynı) |
| Koyu | `#F4A261` (yeni) | `#F4A261` (yeni) | `#1a1206` (aynı) |
| Yüksek karşıtlık açık | `#8a4d00` (aynı) | `#8a4d00` (aynı) | `#ffffff` (aynı) |
| Yüksek karşıtlık koyu | `#ffc266` (aynı) | `#ffc266` (aynı) | `#000000` (aynı) |

Karşıtlık denetimi (WCAG göreli parlaklık, 25.11.2026'da hesaplandı):

- `#1a1206` yazı `#F4A261` dolgu üzerinde: 8.99:1 (şart 4.5:1, geçer).
- `#F4A261` yazı koyu `bg` `#000000` üzerinde 10.19:1, koyu `card` `#1c1c1e` üzerinde 8.25:1 (geçer). Eski `#f0a03c` değeri `card` üzerinde 7.94:1 idi.
- `#F4A261` yazı beyaz üzerinde 2.06:1, `#f2f2f7` üzerinde 1.85:1 (geçmez). Bu yüzden açık temada Amber Gold yalnız dolgu olarak kullanılır; üzerine beyaz yazılmaz, açık zeminde yazı rengi olmaz.
- Yüksek karşıtlık temaları marka rengini kullanmaz; tasarım sistemindeki daha güçlü değerler kalır. Erişilebilirlik markadan önce gelir.

Forest Green, Stone Beige ve Slate Gray uygulama arayüzünde renk token'ı olmaz. Uygulama Apple sistem renklerini izler (tasarım sistemi README). Forest Green yalnız ikon zemininde ve yazdırılan formların sembolünde görünür. Pazarlama sitesinin stilleri uygulamaya alınmaz.

### 4. İç tanımlayıcılar

- **Değişmez:** crate adları, veritabanı adı ve `demleme` rolleri, Kubernetes ad alanları, API yolları, ortam değişkeni adları, depo adı. Bu adlar kullanıcıya görünmez; değişmeleri risk getirir, kullanıcıya değer getirmez. Marka incelemesi (#146) sonucunda ad yeniden değişirse de bu katman etkilenmez.
- **Değişir (görünür ya da kalıcı olanlar):** iOS bundle kimliği önekiyle birlikte `com.gristora.app` olur (test hedefleri `com.gristora.app.tests`, `com.gristora.app.uitests`). Bundle kimliği ilk App Store Connect kaydından sonra değiştirilemez; şimdiki değer zaten yer tutucudur. Değişiklik G6'da, ilk TestFlight yüklemesinden önce yapılır. Production `APPLE_BUNDLE_ID` ayarı aynı değeri alır.
- Kullanıcıya görünen her "Demleme" metni GRISTORA olur (G2 listesi; G3, G4, G5, G6).

## Sonuçlar

- **G3** (Claude): konsept, tasarım sistemi (dosya ve Artifact) ve plan başlığı. Tasarım sisteminde `tint-fill` ve koyu tema `tint` değerleri yukarıdaki tabloya göre değişir; logo ve ikon öğeleri eklenir; `araclar/tutarlilik.py` temiz kalır.
- **G4** (ChatGPT'ye verilebilir, C sınıfı): katalog, e-posta göndereni, OpenAPI başlığı; `make gen`.
- **G5** (ChatGPT'ye verilebilir; `web/` korumalı yol denetimi gelene kadar C sınıfı): web başlığı, favicon, manifest, başlıkta sembol. Pazarlama sitesinin CSS'i kullanılmaz.
- **G6** (Aşama 4): iOS görünen adı, ikon, bundle kimliği.
- **G7**: alan adları; O7 (alan adı satın alma) ve O4'e (Google Cloud) bağlı.
- Owner kararı gereken tek açık nokta: App Store alt başlığı (Aşama 4'te, App Store kaydı açılırken).
- Marka incelemesi (#146) olumsuz biterse yalnız görünen katman (bu ADR'nin 1–3. bölümleri) değişir; yeni bir ADR yazılır.
