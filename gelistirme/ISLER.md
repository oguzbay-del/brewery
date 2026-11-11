# İş listesi

Geliştirmenin tek takip yeri. Durum değişince bu dosya ve `DURUM.md` aynı commit'te güncellenir.

**Durum:** `bekliyor` · `sürüyor` · `bitti (tarih)` · `engelli (neden)`. İşin commit'i mesajı iş numarasıyla başlayan commit'tir (`git log --grep '^T0.2:'`).
**Kim:** `Claude` · `Oğuz` (ürün sahibi)
**Model:** işi kimin yaptığı. Opus işi ana oturumda yapılır; diğerleri alt ajana (Agent aracı) verilir.

## Model dağılımı

| Model | Ne zaman | Örnek |
|---|---|---|
| Opus (ana oturum) | Karar, tasarım, alt ajana iş tarifi yazma, sonucu inceleme, kapı kontrolü | Depo düzeni, veri modeli tasarımı, ADR, inceleme |
| Sonnet (alt ajan) | Tarifi net kod işi: yazma, test, derleme, hata düzeltme, belge | Rust iskeleti, Docker Compose, React ekranı, Terraform |
| Haiku (alt ajan) | Mekanik, tekrar eden, riski düşük | Şablondan dosya üretme, yeniden adlandırma, log tarama, sayım |
| Fable (alt ajan) | Yalnız kapılarda bağımsız doğrulama | Formül doğrulaması, veri modeli ve güvenlik incelemesi |

Alt ajan iş tarifi her zaman şunları içerir: okunacak dosyalar, yapılacak iş, bitti ölçütü, çalıştırılacak testler, "en fazla 20 satır rapor". Alt ajan commit atmaz; commit'i ana oturum inceledikten sonra atar.

## Aşama 0: Temel

Kapı: bütün sistem yerelde tek komutla ayağa kalkıyor, CI yeşil, kind üzerinde duman testi geçiyor.

| İş | Tanım | Bitti ölçütü | Bağlı | Kim | Model | Durum |
|---|---|---|---|---|---|---|
| T0.1 | Depo düzeni ve ADR-0001: klasörler (`sunucu/` Rust çalışma alanı, `web/`, `ios/`, `sozlesme/`, `altyapi/`), adlandırma, dal ve commit kuralları | `adr/0001-depo-duzeni.md` yazıldı, klasörler ve README'leri var | – | Claude | Opus | bitti (11.11.2026) |
| T0.2 | Rust çalışma alanı (`sunucu/`): `engine` (kütüphane), `engine-service` (gRPC), `api`, `worker`, `ingest`, `common`; her serviste sağlık ucu; `rust-toolchain.toml` | `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test` geçiyor | T0.1 | Claude | Sonnet | bitti (11.11.2026) |
| T0.3 | Docker Compose: PostgreSQL, Redis, NATS, API, hesap motoru; sahte GitHub, e-posta ve APNs | `make up` ile hepsi ayağa kalkıyor, sağlık uçları 200 | T0.2 | Claude | Sonnet | sürüyor |
| T0.4 | API sözleşmesi iskeleti: OpenAPI koddan üretiliyor; Swift ve TypeScript istemci üretimi; gRPC proto | Üretim betiği çalışıyor, fark kontrolü CI'da | T0.2 | Claude | Sonnet | sürüyor |
| T0.5 | Web iskeleti: React + TypeScript (strict), tasarım token'ları `tokens.json`'dan CSS'e; Playwright duman testi | `npm test` ve Playwright geçiyor | T0.1 | Claude | Sonnet | bitti (11.11.2026) |
| T0.6 | iOS iskeleti: SwiftUI proje, token'lardan asset kataloğu üretimi, birim test hedefi | Proje dosyaları hazır; derleme macOS CI'da (T0.9) | T0.1 | Claude | Sonnet | bitti (11.11.2026; derleme macOS CI bekliyor) |
| T0.7 | Kubernetes tanımları (Helm veya Kustomize) ve kind duman testi betiği | `make kind-test` geçiyor | T0.3 | Claude | Sonnet | bekliyor |
| T0.8 | Terraform iskeleti (GKE Autopilot, Cloud SQL, Memorystore, Cloud Storage, Artifact Registry) | `terraform validate` geçiyor; `plan` için hesap gerekmez | T0.1 | Claude | Sonnet | sürüyor |
| T0.9 | CI: Rust, web, sözleşme farkı, Python test vektörleri, tutarlılık betiği, Compose duman testi; macOS iş akışı tanımı | İş akışı dosyaları yazıldı, yerelde `act` veya eşdeğer ile denendi | T0.2–T0.5 | Claude | Sonnet | bekliyor |
| T0.10 | Modül README ve CLAUDE.md şablonları (ne yapar, neye dokunmaz, nasıl test edilir) | Her modülde var | T0.2, T0.5, T0.6 | Claude | Haiku | sürüyor |
| T0.11 | Aşama 0 kapı kontrolü ve inceleme | Kapı maddeleri sağlandı; sapmalar ADR'de | T0.3–T0.10 | Claude | Opus | bekliyor |
| T0.12 | Tasarım sistemi: koyu tema için yüksek karşıtlık değerleri (5 token); web ve iOS üretimi | Oranlar: metin 7:1, ayırıcı 3:1; web ve iOS üretim denetimleri geçiyor; tasarım sistemi yayını güncel | T0.5, T0.6 | Claude | Sonnet | sürüyor |

## Aşama 1: Hesap motoru

Kapı: Python referansıyla Rust motoru bütün test vektörlerinde eşit; Fable formül doğrulamasının bulguları kapandı.
Ayrıntılı iş listesi aşama başında yazılır (Opus).

| İş | Tanım | Bitti ölçütü | Bağlı | Kim | Model | Durum |
|---|---|---|---|---|---|---|
| F1 | Formüllerin bağımsız doğrulaması: 03 su ve pH, 09 vergi yöntemleri, 10, 11 | Rapor `gelistirme/raporlar/F1-formul-dogrulama.md`; bulgular formüllere işlendi | – | Claude | Fable | sürüyor |
| T1.0 | Aşama 1 iş listesini çıkar (formül dosyası başına bir iş) | Bu tabloda T1.x satırları | T0.11 | Claude | Opus | bekliyor |

## Aşama 2–6

Aşamalar plan › Geliştirme düzeni ve plan › Yol haritasında. Her aşamanın iş listesi bir önceki aşama kapandığında yazılır (Opus).

| Aşama | İçerik | Kapı |
|---|---|---|
| 2 Çekirdek API ve veri | Veri modeli, göçler, giriş, yetki, NATS, Redis, destek ve GitHub | Uçtan uca testler; Fable veri modeli ve güvenlik incelemesi |
| 3 Web | Ev MVP ekranları | Playwright, erişilebilirlik |
| 4 iOS | SwiftUI, demleme paketi, Live Activity | macOS CI yeşil; cihaz testi (Oğuz) |
| 5 Ev MVP beta | Bulut kurulumu, beta | Görüşmeler ve altın test tamam; bağımsız güvenlik incelemesi |
| 6 İşletme | İşletme ekranları | Aşama aşama |

## Oğuz'un işleri (paralel)

| İş | Tanım | Gerektiği yer | Durum |
|---|---|---|---|
| O1 | Kullanıcı görüşmeleri (`plan/gorusme.md`) | Aşama 5 kapısı | bekliyor |
| O2 | BeerSmith ekranından altın test sayıları (`formuller/ALTIN-TEST.md`) | Aşama 5 kapısı | bekliyor |
| O3 | Apple Developer hesabı, macOS CI (GitHub Actions macOS veya Xcode Cloud) | T0.9, Aşama 4 | bekliyor |
| O4 | Google Cloud organizasyonu ve faturalandırma | Aşama 5 | bekliyor |
| O5 | GitHub'da eski 7 dalı silme | – | bekliyor |
