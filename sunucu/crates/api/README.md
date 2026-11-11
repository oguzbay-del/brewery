# api

Çekirdek REST API. Şimdilik yalnız `GET /healthz`; uç noktalar ve OpenAPI sonraki işlerde.
Port `PORT` ortam değişkeninden (varsayılan 8080). Veritabanına ve hesap koduna dokunmaz.
Test: `cargo test -p api` (`sunucu/` içinde).
