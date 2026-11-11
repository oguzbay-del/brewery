# ADR-0001: Depo düzeni ve çalışma kuralları

**Durum:** kabul · **Tarih:** 11 Kasım 2026 · **İş:** T0.1

## Bağlam

Kodun tamamını Claude yazar, ürün sahibi iOS cihaz testini ve hesap kurulumlarını yapar (plan › Geliştirme düzeni). Tek depo istendi: bir değişikliğin sunucu, istemci ve sözleşmedeki etkisi aynı anda görülmeli; her oturum depoyu açıp kaldığı yerden devam edebilmeli.

## Karar

### Klasörler

| Klasör | İçerik |
|---|---|
| `sunucu/` | Rust çalışma alanı (Cargo workspace). Kütüphaneler ve servisler `crates/` altında |
| `sunucu/crates/engine` | Hesap motoru kütüphanesi: saf fonksiyonlar, veritabanı ve ağ yok |
| `sunucu/crates/engine-service` | Hesap motorunu gRPC ile sunan servis (yalnız iç ağ) |
| `sunucu/crates/api` | Çekirdek API (REST, OpenAPI koddan üretilir) |
| `sunucu/crates/worker` | Arka plan işleri (NATS tüketicileri) |
| `sunucu/crates/ingest` | Dış olay alımı (cihaz, App Store, ödeme, GitHub webhook'ları) |
| `sunucu/crates/common` | Ortak tipler, yapılandırma, gözlem (OpenTelemetry) |
| `web/` | React + TypeScript (Vite) |
| `ios/` | SwiftUI uygulaması; Xcode projesi XcodeGen ile `project.yml`'dan üretilir (proje dosyası elle düzenlenmez) |
| `sozlesme/openapi/` | Üretilen OpenAPI dosyası (depoda tutulur; fark kontrolü CI'da) |
| `sozlesme/proto/` | Hesap motoru gRPC tanımları |
| `altyapi/compose/` | Yerel tek komutla çalışan sistem ve sahte servisler (GitHub, e-posta, APNs) |
| `altyapi/k8s/` | Kubernetes tanımları, Kustomize: `base/` ve `overlays/` (yerel, test, canlı) |
| `altyapi/terraform/` | Bulut altyapısı: `modules/` ve ortam başına kök (`envs/`) |
| `adr/` | Karar kayıtları (`NNNN-kisa-ad.md`) |
| `gelistirme/` | İş listesi ve durum |

Var olan klasörler aynen kalır: `plan/`, `arayuz/`, `tasarim-sistemi/`, `formuller/`, `karsilastirma/`, `erisilebilirlik/`, `araclar/`.

### Dil

- **Kod İngilizce:** tanımlayıcılar, crate, paket, modül ve dosya adları, kod içi yorumlar. Kütüphaneler, hata mesajları ve araçlar İngilizce; karışık dil hata ve arama maliyeti doğurur.
- **Belgeler Türkçe:** README, CLAUDE.md, ADR, takip dosyaları, commit mesajları. Depodaki mevcut belgelerle aynı.
- **Kullanıcıya görünen metin** kodda yazılmaz; metin kataloğundan gelir (Türkçe ve İngilizce, plan › Birimler ve yerelleştirme).

### Araç sürümleri

Her araç sürümü depoda sabitlenir: `rust-toolchain.toml` (Rust), `.nvmrc` ve `package.json` `engines` (Node), CI'da Xcode sürümü, `altyapi/terraform/.terraform-version`. Sürüm yükseltmesi ayrı bir iştir.

### Biçim ve denetim

- Rust: `rustfmt`, `clippy -D warnings`.
- Web: ESLint, Prettier, TypeScript `strict`.
- iOS: SwiftFormat, SwiftLint.
- Kubernetes: Kustomize (şablon dili yok; Helm kullanılmaz).

### Dal ve commit

- Her iş kısa ömürlü bir dalda: `is/T0.2-rust-iskeleti`. Testler geçince `main`'e hızlı ileri alınır (fast-forward); CI kurulduktan sonra CI yeşil olmadan alınmaz.
- Commit mesajı iş numarasıyla başlar: `T0.2: Rust çalışma alanı iskeleti`.
- Bir commit bir iş. İş yarım kalırsa dalda bekler, `gelistirme/DURUM.md`'de yazar.

### Tek komut

Kökte `Makefile`: `make up`, `make down`, `make test`, `make lint`, `make gen` (sözleşme ve istemci üretimi), `make kind-test`. Her yeni modül bu hedeflere bağlanır.

## Sonuçlar

- Her oturum `CLAUDE.md` ve `gelistirme/DURUM.md` ile başlar, depo düzenini bu kayıttan öğrenir.
- XcodeGen sayesinde iOS projesi Linux'ta metin olarak düzenlenebilir; derleme macOS CI'da.
- Kod ve belge dilinin ayrı olması bilinçli: kod ekosistemle uyumlu, belgeler ekiple.
