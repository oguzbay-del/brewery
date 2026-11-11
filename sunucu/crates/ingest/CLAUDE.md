# ingest

## Ne yapar
- Dış olay alımı (webhook'lar): cihaz, App Store, ödeme ve GitHub kaynakları.
- Şimdilik yalnız `GET /healthz` ve temiz kapanma; kaynak bağlantıları sonraki işlerde.
- Port `PORT` (varsayılan 8082).

## Neye dokunmaz
- Veritabanına ve hesap koduna dokunmaz.
- Olay işleme `worker`'a aittir; bu crate olayları alır.

## Nasıl test edilir
- `sunucu/` içinde: `cargo test -p ingest`
- Lint: `cargo clippy --workspace --all-targets -- -D warnings`

## Kurallar
- Kod İngilizce; belgeler Türkçe.
- Commit mesajı iş numarasıyla başlar: `T0.x: ...`.
