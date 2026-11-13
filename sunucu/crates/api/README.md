# api

Çekirdek REST API. Kurallar: `adr/0008-api-kurallari.md`. T2.10 bu kuralların ara katmanlarını ve yardımcılarını yazar; alan uçları (tarif, parti, ...) sonraki işlerde bunları kullanır.

Uçlar şimdilik: `GET /healthz`, `GET /readyz` (veritabanı gerekir; Redis durumu yalnız bilgi), `GET /v1/version`, `GET /v1/meta/fields` (alan kataloğu, ETag, `common::catalog`), `GET /v1/meta/client-requirements`.

## Parçalar

| Modül | Ne yapar |
|---|---|
| `error` | RFC 9457 `ProblemDetails`, `ErrorCode` (19 kodun tek listesi), `ApiError`, `ApiJson` (400/413/415 eşlemesi), `MergePatch` (`application/merge-patch+json`), `ApiQuery`, `404` yedeği |
| `context` | `Authenticator` arayüzü (T2.11 gerçeğini yazar; `DenyAll` o zamana kadar `401` verir), `RequestContext` (kullanıcı, çalışma alanı, izinler), yol tabanlı üyelik denetimi (üye değilse `404`), `begin()` ile `SET LOCAL app.user_id/workspace_id/actor_kind` açık işlem |
| `idempotency` | `Idempotency-Key` ara katmanı: `idempotency_keys` tablosu, aynı anahtar+gövde yanıtı tekrarlar, farklı gövde `422`, işlenen `409`, anahtar yoksa `428` |
| `concurrency` | `Concurrency` (`optimistic`, `field_lww`, `append`), `check_if_match` (`428`/`412` + `current`), ETag ve `If-None-Match` (`304`) yardımcıları |
| `pagination` | `ListParams`, `SortSpec`, opak imleç (`Cursor`), `Page<T>`; 50 varsayılan, 200 üst sınır |
| `ratelimit` | Redis'te IP, kullanıcı, çalışma alanı başına dakikalık sayaç; `RateLimit`, `RateLimit-Policy`, `Retry-After`. Redis yoksa uygulanmaz (fail open) |
| `redis_client` | Tembel bağlanan, kopunca kendini toplayan Redis tutamağı; her işlem 250 ms ile sınırlı |
| `request` | `X-Request-Id`, W3C `traceparent` ile OpenTelemetry izi, istek span'ı; `/v1` yanıtlarına `Cache-Control: private, no-cache` |
| `app_version` | `X-App-Version` ve `MIN_APP_VERSION_IOS/WEB` ile `426 upgrade_required` |
| `openapi` | utoipa belgesi + `OPERATIONS` tablosu (her işlemin hata kodları, `x-permission`, `x-concurrency`, `security`); test tabloda olmayan işlemi düşürür |

Yeni uç eklerken: `#[utoipa::path]` (operation_id `<kaynak>_<eylem>`, başarı yanıtına örnek), `openapi.rs` içindeki `ApiDoc.paths` ve `OPERATIONS`; yönlendirmede `api::authenticated(&state, api::idempotent(&state, yazma_uçları).merge(okuma_uçları))` ve `api::build(state, ekstra)`; sonra `bash sozlesme/scripts/gen.sh`.

## Yapılandırma (ortam; sırlar `*_FILE` ile de)

`PORT`, `SERVICE_NAME`, `DATABASE_URL` (`app_api` rolü; `DATABASE_URL_FILE`), `REDIS_URL` (`REDIS_URL_FILE`; yoksa Redis'siz çalışır), `MIN_APP_VERSION_IOS`, `MIN_APP_VERSION_WEB`, `RATE_LIMIT_IP_PER_MINUTE` (300), `RATE_LIMIT_USER_PER_MINUTE` (600), `RATE_LIMIT_WORKSPACE_PER_MINUTE` (1200), `TRUST_FORWARDED_FOR` (`true` ise IP `X-Forwarded-For`'un ilk girdisi; yalnız güvenilir vekil arkasında), `OTEL_EXPORTER_OTLP_ENDPOINT` (verilirse izler OTLP/HTTP ile gider; yoksa dışa aktarım kapalı, izler yine günlükte), `RUST_LOG`.

## Test

`sunucu/` içinde: `DATABASE_URL=... cargo test -p api`. `tests/adr0008.rs` ADR-0008'in her kuralını demo bir kaynak (`tests/support`) üzerinden sınar: gerçek PostgreSQL (`#[sqlx::test]`, `app_api` rolüyle, RLS açık) ve rate limit için `REDIS_URL` (varsayılan `redis://127.0.0.1:6379`) gerekir. Redis kopma testi arada kesilebilen bir TCP vekili kullanır, compose Redis'ine dokunmaz. `tests/trace.rs` iz testidir; iş parçacığına bağlı abone kurduğu için ayrı ikilidedir.
