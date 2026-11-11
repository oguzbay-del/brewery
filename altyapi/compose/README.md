# altyapi/compose

Bütün sistemi yerelde tek komutla ayağa kaldırır: PostgreSQL, Redis, NATS, dört Rust servisi ve sahte dış servisler. Gereken: Docker (Compose v2) ve `curl`.

## Komutlar (depo kökünden)

| Komut | Ne yapar |
|---|---|
| `make up` | İmajları derler, her şeyi başlatır, sağlık denetimleri geçene kadar bekler |
| `make smoke` | Her servisin sağlığını ve sahte servislerin yanıtını denetler; hata varsa 1 döner |
| `make ps` / `make logs` | Durum / günlük (`make logs S=api` tek servis) |
| `make down` | Durdurur ve kaldırır; veri hacimleri kalır (`make down ARGS=-v` hacimleri de siler) |
| `make test` / `make lint` | `cargo test --workspace` / `cargo fmt --check` ve `clippy -D warnings` |

Ortam değişkenlerinin hepsinin varsayılanı var; değiştirmek için `.env.example` dosyasını `.env` olarak kopyalayın (`.env` git'e girmez).

## Portlar (yalnız 127.0.0.1)

| Servis | Port | Not |
|---|---|---|
| api | 8080 | `GET /healthz` |
| worker | 8081 | `GET /healthz` |
| ingest | 8082 | `GET /healthz` |
| engine-service | 50051 | gRPC; sağlık için `grpc_health_probe` imajın içinde |
| PostgreSQL | 5432 | kullanıcı/veritabanı `demleme` (yalnız yerel örnek değer) |
| Redis | 6379 | |
| NATS | 4222, 8222 | JetStream açık; 8222 izleme (`/healthz`, `/jsz`) |
| Mailpit | 1025 (SMTP), 8025 (arayüz) | |
| github-mock | 8090 | WireMock |
| apns-mock | 8091 | WireMock |

Servislere şu ortam değişkenleri verilir (şimdilik kullanılmayabilir): `DATABASE_URL`, `REDIS_URL`, `NATS_URL`, `ENGINE_URL`, `SMTP_HOST`/`SMTP_PORT`, `GITHUB_API_URL`, `APNS_URL`.

## Sahte servisler

- **Mailpit:** giden e-postayı yakalar, gerçek posta göndermez. Arayüz http://127.0.0.1:8025.
- **github-mock (WireMock):** `POST /repos/{owner}/{repo}/issues` ve `POST /repos/{owner}/{repo}/issues/{n}/comments` her zaman 201 döner.
- **apns-mock (WireMock):** `POST /3/device/{token}` 200 döner (`apns-id` başlığıyla). Gerçek APNs HTTPS/HTTP2'dir; burada düz HTTP.
- Eşlemeler `mocks/github/mappings/` ve `mocks/apns/mappings/` altındadır; yeni uç için bir JSON dosyası eklemek yeter. İstekleri görmek: `http://127.0.0.1:8090/__admin/requests`.

## Notlar

- Dockerfile `sunucu/Dockerfile`'dır (cargo-chef, tek dosya, `BIN` ile servis seçilir; çalışma imajı debian-slim, root olmayan kullanıcı). Bağımlılık katmanı dört servis için ortaktır, ilk derleme birkaç dakika sürer, sonrası önbellekten gelir.
- TLS'i araya giren bir proxy arkasındaysanız (kurumsal ağ, korumalı ortam): `make up BUILD_CA_FILE=/yol/ca-bundle.crt`. CA dosyası `altyapi/compose/.build-ca.pem` olarak kopyalanır (git dışı). `docker compose` komutunu `make` olmadan çalıştırırsanız bu dosyanın var olması gerekir (`touch altyapi/compose/.build-ca.pem`).
- Servis kodu değişince `make up` yeniden derler. Port çakışırsa `.env` içinde portu değiştirin.
