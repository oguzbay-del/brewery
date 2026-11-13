# db

PostgreSQL erişim katmanı (sqlx). Kararlar: `adr/0007-veri-modeli.md`.

| Parça | Ne yapar |
|---|---|
| `database_url()` | Bağlantı adresini `DATABASE_URL` ya da `DATABASE_URL_FILE` (dosyadan, Kubernetes Secret bağlaması için) okur; ikisi birden verilirse hata |
| `connect`, `connect_from_env` | Havuzu kurar |
| `migrate` | `sunucu/migrations/` göçlerini uygular (derleme zamanında gömülür) |
| `set_workspace_id` | İşlem içinde `app.workspace_id` bağlamını ayarlar (RLS politikalarının girdisi; ADR-0007 madde 3). Tam bağlam (`app.user_id`, `app.actor_kind`) T2.10'da |
| `migrate` ikilisi | `cargo run -p db --bin migrate`; compose ve Kubernetes göç Job'ı bunu çalıştırır |

Komutlar (kökten): `make migrate` yerel compose veritabanına göç uygular; `make sqlx-prepare` `sunucu/.sqlx/` önbelleğini üretir (commit edilir; `SQLX_OFFLINE=true` ile derlenir).
