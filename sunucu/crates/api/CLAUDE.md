# api

## Ne yapar
- Çekirdek REST API: ADR-0008'in ara katmanları (hata zarfı, istek bağlamı, tekrar anahtarı, ETag/If-Match, imleçli sayfalama, hız sınırı, istek kimliği ve iz, sürüm uyumu) ve OpenAPI üretimi. Ayrıntı: `README.md`.
- Uçlar: `/healthz`, `/readyz`, `/v1/version`, `/v1/meta/fields`, `/v1/meta/client-requirements`. Alan uçları sonraki işlerde.
- Veritabanına `db` crate'i üzerinden (`app_api` rolü) erişir; her istek bağlamı `RequestContext::begin` ile `SET LOCAL` yapılmış işlemde yürür.
- Port `PORT` (varsayılan 8080).

## Neye dokunmaz
- Formül içermez; hesap gerektiğinde `engine-service`'ten gRPC ile istenir.
- Kimlik doğrulama gövdesi (oturum, Apple/Google) burada değil T2.11'dedir; `Authenticator` arayüzü hazır, `main` şimdilik `DenyAll` kullanır.
- Göç eklemez; tablo kararları `adr/0007-veri-modeli.md`.

## Nasıl test edilir
- `sunucu/` içinde: `DATABASE_URL=... cargo test -p api` (PostgreSQL ve Redis gerekir: `docker compose -f altyapi/compose/compose.yaml up -d --wait postgres redis`).
- Lint: `cargo clippy --workspace --all-targets -- -D warnings`
- Sözleşme: `bash sozlesme/scripts/check.sh`

## Kurallar
- `sozlesme/openapi/` altındaki OpenAPI dosyası elle düzenlenmez; `gen-openapi` ile (`bash sozlesme/scripts/gen.sh`) yeniden üretilir.
- Yeni uç `openapi.rs` içindeki `OPERATIONS` tablosuna satır ekler (hata kodları, `x-permission`, `x-concurrency`); test eksiği yakalar.
- Hata her zaman `ApiError` ile döner, kendi JSON biçimi kurulmaz. Redis hatası isteği düşürmez (`redis_client` `None` döner; hız sınırı fail open).
- Kod İngilizce; belgeler Türkçe.
- Commit mesajı iş numarasıyla başlar: `T2.x: ...`.
