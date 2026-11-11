# worker

Arka plan işleri. Şimdilik yalnız `GET /healthz` ve temiz kapanma; NATS tüketicileri sonraki işlerde.
Port `PORT` ortam değişkeninden (varsayılan 8081). Veritabanına ve hesap koduna dokunmaz.
Test: `cargo test -p worker` (`sunucu/` içinde).
