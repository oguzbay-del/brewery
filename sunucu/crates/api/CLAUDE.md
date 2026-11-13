# api

## Ne yapar
- Çekirdek REST API: ADR-0008'in ara katmanları (hata zarfı, istek bağlamı, tekrar anahtarı, ETag/If-Match, imleçli sayfalama, hız sınırı, istek kimliği ve iz, sürüm uyumu) ve OpenAPI üretimi. Ayrıntı: `README.md`.
- Uçlar: `/healthz`, `/readyz`, `/v1/version`, `/v1/meta/fields`, `/v1/meta/client-requirements`, `GET /v1/workspaces/{ws}/events` (SSE, `events.rs`, T2.15: her bağlantı NATS JetStream'den kendi sıralı tüketicisiyle okur; üyelik 30 sn'de bir denetlenir; `NATS_URL` yoksa 503). Alan uçları sonraki işlerde. Dosya uçları (`files.rs`: attachments, scanned-documents; T2.18).
- Veritabanına `db` crate'i üzerinden (`app_api` rolü) erişir; her istek bağlamı `RequestContext::begin` ile `SET LOCAL` yapılmış işlemde yürür.
- Motor bağlantısı (T2.16): `src/engine.rs`. `ENGINE_URL` ile tonic kanalı, motor sürüm damgası, girdi özetiyle (protobuf kanonik bayt + SHA-256) Redis önbelleği, `calc_results` kaydı, motor durumunun hata zarfına eşlenmesi. Hesap isteyen uç `EngineClient::calculate` kullanır. Çıktı `calc_results.output` içinde base64 protobuf.
- Port `PORT` (varsayılan 8080).

## Neye dokunmaz
- Formül içermez; hesap gerektiğinde `engine-service`'ten gRPC ile istenir.
- Kimlik doğrulama gövdesi (oturum, Apple/Google) burada değil T2.11'dedir; `Authenticator` arayüzü hazır, `main` şimdilik `DenyAll` kullanır.
- Nesne deposu istemcisi `storage` crate'indedir; burada yalnız uçlar ve doğrulama vardır.
- Göç eklemez (T2.18'in `0070` göçü istisna); tablo kararları `adr/0007-veri-modeli.md`.

## Nasıl test edilir
- `sunucu/` içinde: `DATABASE_URL=... cargo test -p api` (PostgreSQL, Redis ve (`tests/sse.rs` için) NATS JetStream, (`tests/files.rs` için) nesne deposu gerekir: `docker compose -f altyapi/compose/compose.yaml up -d --wait postgres redis nats objectstore`).
- Lint: `cargo clippy --workspace --all-targets -- -D warnings`
- Sözleşme: `bash sozlesme/scripts/check.sh`

## Kurallar
- `sozlesme/openapi/` altındaki OpenAPI dosyası elle düzenlenmez; `gen-openapi` ile (`bash sozlesme/scripts/gen.sh`) yeniden üretilir.
- Yeni uç `openapi.rs` içindeki `OPERATIONS` tablosuna satır ekler (hata kodları, `x-permission`, `x-concurrency`); test eksiği yakalar.
- Hata her zaman `ApiError` ile döner, kendi JSON biçimi kurulmaz. Redis hatası isteği düşürmez (`redis_client` `None` döner; hız sınırı fail open).
- Kod İngilizce; belgeler Türkçe.
- Commit mesajı iş numarasıyla başlar: `T2.x: ...`.
