# db

PostgreSQL erişim katmanı (sqlx). Kararlar: `adr/0007-veri-modeli.md`.

| Parça | Ne yapar |
|---|---|
| `database_url()` | Bağlantı adresini `DATABASE_URL` ya da `DATABASE_URL_FILE` (dosyadan, Kubernetes Secret bağlaması için) okur; ikisi birden verilirse hata |
| `connect`, `connect_from_env` | Havuzu kurar |
| `migrate` | `sunucu/migrations/` göçlerini uygular (derleme zamanında gömülür) |
| `set_workspace_id` | İşlem içinde `app.workspace_id` bağlamını ayarlar (RLS politikalarının girdisi; ADR-0007 madde 3). Tam bağlam (`app.user_id`, `app.actor_kind`) T2.10'da |
| `migrate_database_url()` | Göç adresi: `MIGRATE_DATABASE_URL` (ya da `_FILE`) varsa o, yoksa `database_url()` |
| `provision_role`, `AppRole`, `password_from_env` | `app_api` / `app_worker` rollerine oturum açma ve parola verir (parola `APP_API_PASSWORD`, `APP_WORKER_PASSWORD` ya da `*_FILE`) |
| `migrate` ikilisi | `cargo run -p db --bin migrate`; göçleri uygular, sonra roller için parola varsa `provision_role` çalıştırır. Compose ve Kubernetes göç Job'ı bunu çalıştırır |

Komutlar (kökten): `make migrate` yerel compose veritabanına göç uygular; `make sqlx-prepare` `sunucu/.sqlx/` önbelleğini üretir (commit edilir; `SQLX_OFFLINE=true` ile derlenir).

## Göçler (T2.5)

| Göç | İçerik |
|---|---|
| `0002_roller` | `app_owner`, `app_api`, `app_worker` (NOLOGIN, yetkisiz), şema hakları |
| `0003_ortak_islevler` | `app_workspace_id()`, `app_user_id()`, `app_actor_kind()`, `touch_row()` (sürüm ve `updated_at` tetikleyicisi) |
| `0004_calisma_alani_ve_hesap` | `workspaces`, `users`, `user_identities`, `passkeys`, `sessions`, `push_targets`, `email_login_tokens`, `auth_requests`, `staff_users`; giriş öncesi işlevler |
| `0005_uyelik_ve_yetki` | `workspace_members`, `roles`, `role_permissions`, `role_assignments`, `invitations`, `shared_devices`, `member_pins`, `device_sessions` |
| `0006_denetim_ve_altyapi` | `audit_log`, `audit_heads`, `change_history`, `idempotency_keys`, `outbox`, `external_events`, `scheduled_jobs` |

Giriş öncesi işlevler (yalnız `app_api` çağırır): `find_user_by_identity`, `find_session_by_token_hash`, `find_shared_device_by_key_hash`, `find_invitation_by_token_hash`, `email_login_token_create/consume`, `auth_request_create/consume`. Bakım işlevleri (yalnız `app_worker`): `purge_expired_auth_rows`, `purge_expired_idempotency_keys`.
