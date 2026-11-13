# worker

## Ne yapar
- `GET /healthz` ve temiz kapanma.
- Arka plan (`background.rs`, T2.15): outbox -> NATS JetStream aktarıcısı (`events::relay`), `processed_events` temizliği; sonraki işlerin tüketicileri (`events::consumer::run`) buraya kaydolur. `DATABASE_URL` (`app_worker`) ve `NATS_URL` yoksa yalnız sağlık ucu çalışır.
- Port `PORT` (varsayılan 8081).

## Neye dokunmaz
- Hesap koduna dokunmaz. Veritabanına yalnız `app_worker` rolüyle ve `events`/`db` üzerinden erişir.
- Dış olay alımı (webhook'lar) `ingest`'e aittir.

## Nasıl test edilir
- `sunucu/` içinde: `cargo test -p worker`; aktarıcı ve tüketici testleri `crates/events/tests/` içindedir (PostgreSQL ve NATS gerekir).
- Lint: `cargo clippy --workspace --all-targets -- -D warnings`

## Kurallar
- Kod İngilizce; belgeler Türkçe.
- Commit mesajı iş numarasıyla başlar: `T0.x: ...`.
