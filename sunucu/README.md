# sunucu

Rust çalışma alanı. Crate'ler `crates/` altında: `engine` (hesap motoru kütüphanesi), `engine-service` (gRPC), `api` (çekirdek API), `worker` (arka plan işleri), `ingest` (dış olay alımı), `common` (ortak), `db` (PostgreSQL erişimi, göçler: `migrations/`). Düzen: `adr/0001-depo-duzeni.md`. İskelet: iş T0.2.
