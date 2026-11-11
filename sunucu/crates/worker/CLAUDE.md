# worker

## Ne yapar
- Arka plan işleri; şimdilik yalnız `GET /healthz` ve temiz kapanma.
- NATS tüketicileri sonraki işlerde gelir.
- Port `PORT` (varsayılan 8081).

## Neye dokunmaz
- Veritabanına ve hesap koduna dokunmaz.
- Dış olay alımı (webhook'lar) `ingest`'e aittir.

## Nasıl test edilir
- `sunucu/` içinde: `cargo test -p worker`
- Lint: `cargo clippy --workspace --all-targets -- -D warnings`

## Kurallar
- Kod İngilizce; belgeler Türkçe.
- Commit mesajı iş numarasıyla başlar: `T0.x: ...`.
