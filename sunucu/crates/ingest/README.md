# ingest

Dış olay alımı (webhook'lar). Şimdilik yalnız `GET /healthz` ve temiz kapanma; kaynak bağlantıları sonraki işlerde.
Port `PORT` ortam değişkeninden (varsayılan 8082). Veritabanına ve hesap koduna dokunmaz.
Test: `cargo test -p ingest` (`sunucu/` içinde).
