# api

Çekirdek REST API. Kurallar: `adr/0008-api-kurallari.md`. T2.10 bu kuralların ara katmanlarını ve yardımcılarını yazar; alan uçları (tarif, parti, ...) sonraki işlerde bunları kullanır.

Uçlar şimdilik: `GET /healthz`, `GET /readyz` (veritabanı gerekir; Redis durumu yalnız bilgi), `GET /v1/version`, `GET /v1/meta/fields` (alan kataloğu, ETag, `common::catalog`), `GET /v1/meta/client-requirements`; dosya uçları (T2.18, aşağıda); giriş ve oturum (T2.11): `POST /v1/auth/nonce|apple|google|email/start|email/verify`, `GET|DELETE /v1/me/session`, `GET|DELETE /v1/me/sessions`, `DELETE /v1/me/sessions/{id}`, `GET|POST /v1/me/identities`, `DELETE /v1/me/identities/{id}`, `POST /v1/workspaces`.

## Parçalar

| Modül | Ne yapar |
|---|---|
| `error` | RFC 9457 `ProblemDetails`, `ErrorCode` (19 kodun tek listesi), `ApiError`, `ApiJson` (400/413/415 eşlemesi), `MergePatch` (`application/merge-patch+json`), `ApiQuery`, `404` yedeği |
| `context` | `Authenticator` arayüzü (gerçeği `auth::AuthServices`; `DenyAll` yalnız yedek), `check_csrf` kancası, `RequestContext` (kullanıcı, çalışma alanı, izinler), yol tabanlı üyelik denetimi (üye değilse `404`), `begin()` ile `SET LOCAL app.user_id/workspace_id/actor_kind` açık işlem |
| `auth` | Giriş ve oturum (ADR-0009, T2.11). `oidc`: Apple/Google kimlik belirteci doğrulaması (RS256, JWKS önbelleği: bilinmeyen `kid` yenileme tetikler, en çok 24 saat, yenileme aralığı sınırlı; `iss`, `aud`, `exp`, `iat`, 60 sn kayma). `session`: opak belirteç, PG'de SHA-256 özeti, Redis'te 5 dk önbellek, süre tablosu, `last_activity` dakikada bir. `email`: adres normalleştirme (NFC, küçük harf), SMTP istemcisi (mailpit), `MemoryMailer`. `tokens`: rastgele belirteç, özet, CSRF HMAC. `handlers`: uçlar. `setup`: `POST /v1/workspaces` (kullanıcı yönetici rolüyle ilk üye). `AuthServices` hem `Authenticator` (Bearer ya da `__Host-session` çerezi, CSRF, üyelik ve izinler) hem yönlendirici kurucusudur |
| `idempotency` | `Idempotency-Key` ara katmanı: `idempotency_keys` tablosu, aynı anahtar+gövde yanıtı tekrarlar, farklı gövde `422`, işlenen `409`, anahtar yoksa `428` |
| `concurrency` | `Concurrency` (`optimistic`, `field_lww`, `append`), `check_if_match` (`428`/`412` + `current`), ETag ve `If-None-Match` (`304`) yardımcıları |
| `pagination` | `ListParams`, `SortSpec`, opak imleç (`Cursor`), `Page<T>`; 50 varsayılan, 200 üst sınır |
| `ratelimit` | Redis'te IP, kullanıcı, çalışma alanı başına dakikalık sayaç; `RateLimit`, `RateLimit-Policy`, `Retry-After`. Redis yoksa uygulanmaz (fail open) |
| `redis_client` | Tembel bağlanan, kopunca kendini toplayan Redis tutamağı; her işlem 250 ms ile sınırlı |
| `request` | `X-Request-Id`, W3C `traceparent` ile OpenTelemetry izi, istek span'ı; `/v1` yanıtlarına `Cache-Control: private, no-cache` |
| `app_version` | `X-App-Version` ve `MIN_APP_VERSION_IOS/WEB` ile `426 upgrade_required` |
| `files` | Ek ve taranmış belge uçları: kayıt açılır ve imzalı `PUT` verilir, istemci doğrudan nesne deposuna yükler, `complete` depodaki nesneyi denetler (boyut, tür, SHA-256), indirme kısa süreli imzalı adrestir. Yumuşak silme; yasal tutmada `409`. `storage` crate'ini kullanır, `files::routes(&state)` `api::build`'e verilir |
| `openapi` | utoipa belgesi + `OPERATIONS` tablosu (her işlemin hata kodları, `x-permission`, `x-concurrency`, `security`); test tabloda olmayan işlemi düşürür |

Yeni uç eklerken: `#[utoipa::path]` (operation_id `<kaynak>_<eylem>`, başarı yanıtına örnek), `openapi.rs` içindeki `ApiDoc.paths` ve `OPERATIONS`; yönlendirmede `api::authenticated(&state, api::idempotent(&state, yazma_uçları).merge(okuma_uçları))` ve `api::build(state, ekstra)`; sonra `bash sozlesme/scripts/gen.sh`.

## Dosya uçları (T2.18)

`POST|GET /v1/workspaces/{id}/attachments`, `GET|DELETE .../attachments/{id}`, `POST .../attachments/{id}/complete`, `GET .../attachments/{id}/download`; aynısı `scanned-documents` için. Dosyalar API'den geçmez (en çok 25 MiB, tür listesi `files.rs`). Akış, hata kodları ve sınırlar: `adr/0008-api-kurallari.md` T2.18 notları. `S3_ENDPOINT` yoksa bu uçlar `503` verir.

## Yapılandırma (ortam; sırlar `*_FILE` ile de)

Nesne deposu için `S3_*` değişkenleri: `sunucu/crates/storage/README.md`.


`PORT`, `SERVICE_NAME`, `DATABASE_URL` (`app_api` rolü; `DATABASE_URL_FILE`), `REDIS_URL` (`REDIS_URL_FILE`; yoksa Redis'siz çalışır), `MIN_APP_VERSION_IOS`, `MIN_APP_VERSION_WEB`, `RATE_LIMIT_IP_PER_MINUTE` (300), `RATE_LIMIT_USER_PER_MINUTE` (600), `RATE_LIMIT_WORKSPACE_PER_MINUTE` (1200), `TRUST_FORWARDED_FOR` (`true` ise IP `X-Forwarded-For`'un ilk girdisi; yalnız güvenilir vekil arkasında), `OTEL_EXPORTER_OTLP_ENDPOINT` (verilirse izler OTLP/HTTP ile gider; yoksa dışa aktarım kapalı, izler yine günlükte), `RUST_LOG`.

## Test

`sunucu/` içinde: `DATABASE_URL=... cargo test -p api`. `tests/adr0008.rs` ADR-0008'in her kuralını demo bir kaynak (`tests/support`) üzerinden sınar: gerçek PostgreSQL (`#[sqlx::test]`, `app_api` rolüyle, RLS açık) ve rate limit için `REDIS_URL` (varsayılan `redis://127.0.0.1:6379`) gerekir. Redis kopma testi arada kesilebilen bir TCP vekili kullanır, compose Redis'ine dokunmaz. `tests/files.rs` dosya uçlarını uçtan uca sınar: PostgreSQL ve compose'taki `objectstore` (127.0.0.1:8333; `S3_*` ile değişir) gerekir; yükleme ve indirme imzalı adreslerle doğrudan depoya gider. `tests/trace.rs` iz testidir; iş parçacığına bağlı abone kurduğu için ayrı ikilidedir.
`APPLE_CLIENT_IDS`, `GOOGLE_CLIENT_IDS` (virgüllü; boşsa sağlayıcı kapalı, uç `404`), `APPLE_ISSUER`/`GOOGLE_ISSUER`, `APPLE_JWKS_URL`/`GOOGLE_JWKS_URL` (yerelde sahte sağlayıcıya), `JWKS_MIN_REFRESH_SECS` (30), `WEB_BASE_URL` (e-posta bağlantısı), `WEB_ORIGINS` (çerezli yazmada izinli `Origin`; varsayılan `WEB_BASE_URL`'in kökeni), `CSRF_HMAC_KEY` (`_FILE`; yoksa süreç başına rastgele, uyarı verir), `SMTP_HOST`/`SMTP_PORT` (yoksa e-posta bellekte kalır, gönderilmez), `MAIL_FROM`, `PORT`, `SERVICE_NAME`, `DATABASE_URL` (`app_api` rolü; `DATABASE_URL_FILE`), `REDIS_URL` (`REDIS_URL_FILE`; yoksa Redis'siz çalışır), `MIN_APP_VERSION_IOS`, `MIN_APP_VERSION_WEB`, `RATE_LIMIT_IP_PER_MINUTE` (300), `RATE_LIMIT_USER_PER_MINUTE` (600), `RATE_LIMIT_WORKSPACE_PER_MINUTE` (1200), `TRUST_FORWARDED_FOR` (`true` ise IP `X-Forwarded-For`'un ilk girdisi; yalnız güvenilir vekil arkasında), `OTEL_EXPORTER_OTLP_ENDPOINT` (verilirse izler OTLP/HTTP ile gider; yoksa dışa aktarım kapalı, izler yine günlükte), `RUST_LOG`.

## Test

`sunucu/` içinde: `DATABASE_URL=... cargo test -p api`. `tests/adr0008.rs` ADR-0008'in her kuralını demo bir kaynak (`tests/support`) üzerinden sınar: gerçek PostgreSQL (`#[sqlx::test]`, `app_api` rolüyle, RLS açık) ve rate limit için `REDIS_URL` (varsayılan `redis://127.0.0.1:6379`) gerekir. Redis kopma testi arada kesilebilen bir TCP vekili kullanır, compose Redis'ine dokunmaz. `tests/auth.rs` T2.11'in uçtan uca testidir: süreç içinde `fake-oidc` (sahte Apple/Google sağlayıcısı), `MemoryMailer`, gerçek PG ve Redis; bir test mailpit'e (SMTP 1025, arayüz 8025) gerçek posta yollar (`docker compose -f altyapi/compose/compose.yaml up -d --wait mailpit`). `tests/trace.rs` iz testidir; iş parçacığına bağlı abone kurduğu için ayrı ikilidedir.
