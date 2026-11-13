# sunucu: Rust çalışma alanı

## Ne yapar
- Cargo workspace; crate'ler `crates/` altında: engine, engine-service, api, worker, ingest, common, db, events, storage.
- Sürüm `rust-toolchain.toml` ile sabit; ortak bağımlılıklar crate'lerde `workspace = true` ile gelir.
- Düzen kararı: `adr/0001-depo-duzeni.md`.

## Neye dokunmaz
- `Cargo.lock` elle düzenlenmez; cargo üretir.
- `sozlesme/` ve `altyapi/` bu klasörün dışındadır; buradan değiştirilmez.
- İstemci (`web/`, `ios/`) kodu burada yazılmaz.

## Nasıl test edilir
- `sunucu/` içinde: `cargo clippy --workspace --all-targets -- -D warnings`
- Tek crate: `cargo test -p <crate>` (ör. `cargo test -p engine`).
- `db` testleri gerçek PostgreSQL ister (`DATABASE_URL`); kökte `make test` compose varsayılanını verir (önce `make up` ya da yalnız postgres). Ayrıntı: `crates/db/CLAUDE.md`.

## Kurallar
- Kod İngilizce; README ve CLAUDE.md Türkçe (ADR-0001).
- Biçim `rustfmt`; uyarı kabul edilmez (`clippy -D warnings`).
- Commit mesajı iş numarasıyla başlar: `T0.x: ...`.
