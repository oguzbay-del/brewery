# api

## Ne yapar
- Çekirdek REST API; şimdilik `GET /healthz` ve `GET /v1/version`.
- OpenAPI koddan üretilir (`gen-openapi` ikilisi); uç noktalar sonraki işlerde gelir.
- Port `PORT` (varsayılan 8080).

## Neye dokunmaz
- Formül içermez; hesap gerektiğinde `engine-service`'ten gRPC ile istenir.
- Veritabanına ve hesap koduna dokunmaz (şimdilik).

## Nasıl test edilir
- `sunucu/` içinde: `cargo test -p api`
- Lint: `cargo clippy --workspace --all-targets -- -D warnings`

## Kurallar
- `sozlesme/openapi/` altındaki OpenAPI dosyası elle düzenlenmez; `gen-openapi` ile yeniden üretilir.
- Kod İngilizce; belgeler Türkçe.
- Commit mesajı iş numarasıyla başlar: `T0.x: ...`.
