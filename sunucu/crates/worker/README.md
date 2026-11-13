# worker

Arka plan işleri: `GET /healthz`, temiz kapanma ve outbox -> NATS JetStream aktarıcısı (T2.15; `events` crate'i). Tüketiciler `background.rs`'e kaydolur.
Port `PORT` ortam değişkeninden (varsayılan 8081). Hesap koduna dokunmaz; veritabanına `app_worker` rolüyle erişir (`DATABASE_URL`, `NATS_URL`).
Test: `cargo test -p worker` (`sunucu/` içinde).
