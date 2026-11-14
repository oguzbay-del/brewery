# db

PostgreSQL erişim katmanı (sqlx). Kararlar: `adr/0007-veri-modeli.md`.

| Parça | Ne yapar |
|---|---|
| `database_url()` | Bağlantı adresini `DATABASE_URL` ya da `DATABASE_URL_FILE` (dosyadan, Kubernetes Secret bağlaması için) okur; ikisi birden verilirse hata |
| `connect`, `connect_from_env` | Havuzu kurar |
| `migrate` | `sunucu/migrations/` göçlerini uygular (derleme zamanında gömülür) |
| `DbContext`, `ActorKind`, `begin`, `set_context` | Tam istek bağlamı: `app.user_id`, `app.workspace_id`, `app.actor_kind` (`user`/`support`/`system`/`device`) işlemde `SET LOCAL` ile ayarlanır; `support` işlemi `READ ONLY` açar. `begin(pool, &ctx)` bağlamı uygulanmış işlemi verir (API her isteği böyle yürütür; ADR-0007 madde 2.3). Tek tek: `set_workspace_id`, `set_user_id`, `set_actor_kind` |
| `connect_lazy` | İlk kullanımda bağlanan havuz; API veritabanı henüz yokken de açılır, `/readyz` `503` verir |
| `migrate_database_url()` | Göç adresi: `MIGRATE_DATABASE_URL` (ya da `_FILE`) varsa o, yoksa `database_url()` |
| `provision_role`, `AppRole`, `password_from_env` | `app_api` / `app_worker` rollerine oturum açma ve parola verir (parola `APP_API_PASSWORD`, `APP_WORKER_PASSWORD` ya da `*_FILE`) |
| `migrate` ikilisi | `cargo run -p db --bin migrate`; göçleri uygular, sonra roller için parola varsa `provision_role` çalıştırır. Compose ve Kubernetes göç Job'ı bunu çalıştırır |
| `append_audit`, `NewAuditEntry`, `AuditEvent` | Denetim kaydına ekleme (T2.14): istek işleminde `audit_heads` satırını `FOR UPDATE` kilitler, `hash = SHA-256(prev_hash ‖ RFC 8785 kanonik satır)` hesaplar, satırı ve başı yazar. Olay türleri `AuditEvent::ALL` (rol, yetki, kapanmış parti düzeltmesi, ...); gerekçe zorunlu olanlar `requires_reason()`. Kişisel alan için `changed_field` (yalnız "değişti" işareti; değerin özeti yazılmaz) |
| `verify_chain`, `verify_all_chains`, `audit-verify` ikilisi | Zinciri yeniden hesaplar: `audit-verify --all` ya da `--workspace <uuid>` (`MIGRATE_DATABASE_URL`, RLS'yi aşan rol); çıkış 0 sağlam, 1 bozuk, 2 hata |

| `outbox::enqueue`, `NewOutboxEvent`, `Entity` | Kaydı yazan işlemin içinde outbox satırı ekler (UUIDv7 kimlik döner); konu `a.b_c` biçiminde olmalı. Olay yalnız kimlik, sürüm ve küçük `data` taşır (T2.15) |

Komutlar (kökten): `make migrate` yerel compose veritabanına göç uygular; `make sqlx-prepare` `sunucu/.sqlx/` önbelleğini üretir (commit edilir; `SQLX_OFFLINE=true` ile derlenir).

## Göçler (T2.5)

| Göç | İçerik |
|---|---|
| `0002_roller` | `app_owner`, `app_api`, `app_worker` (NOLOGIN, yetkisiz), şema hakları |
| `0003_ortak_islevler` | `app_workspace_id()`, `app_user_id()`, `app_actor_kind()`, `touch_row()` (sürüm ve `updated_at` tetikleyicisi) |
| `0004_calisma_alani_ve_hesap` | `workspaces`, `users`, `user_identities`, `passkeys`, `sessions`, `push_targets`, `email_login_tokens`, `auth_requests`, `staff_users`; giriş öncesi işlevler |
| `0005_uyelik_ve_yetki` | `workspace_members`, `roles`, `role_permissions`, `role_assignments`, `invitations`, `shared_devices`, `member_pins`, `device_sessions` |
| `0006_denetim_ve_altyapi` | `audit_log`, `audit_heads`, `change_history`, `idempotency_keys`, `outbox`, `external_events`, `scheduled_jobs` |

## Ev MVP göçleri (T2.6)

| Göç | İçerik |
|---|---|
| `0007_kutuphane_ve_tanimlar` | `secure_workspace_table()` (RLS, politika, sürüm tetikleyicisi ve hak kurulumu; T2.7 de kullanır), `check_shared_ref()`; `ingredients` + `ingredient_fermentables/hops/yeasts/miscs`, `equipment`, `mash_profiles(+steps)`, `fermentation_profiles(+steps)`, `carbonation_profiles`, `water_profiles`, `style_guides`, `styles`, `style_mappings`, `package_types` |
| `0008_tarif` | `folders`, `recipes`, `recipe_versions`, `recipe_fermentables/hops/yeasts/miscs`, `recipe_water_additions`, `tags`, `recipe_tags`; değişmezlik tetikleyicileri |
| `0009_parti_ve_demleme_gunu` | `batches`, `batch_tags`, `favorites`, `batch_steps`, `batch_ingredient_actuals`, `deviations`, `brew_day_packages`, `devices`, `measurements` (aylık bölümlü), `stage_events`, `tasting_notes`, `calc_results`; `create_measurement_partition()`, `ensure_measurement_partitions()` |
| `0010_stok` | `stock_lots`, `stock_movements` (defter, önbellek tetikleyicisi), `shopping_list_items` |
| `0011_bildirim_cihaz_dosya` | `notifications`, `notification_events`, `reminders`, `alert_rules`, `attachments`, `scanned_documents`, `import_sources`, `export_jobs` |
| `0012_abonelik_ve_destek` | `subscriptions`, `entitlements`, `support_requests`, `support_messages`, `support_grants`, `support_actions` |
| `0013_kapsam_butunlugu` | `check_polymorphic_target()` (`role_assignments`, `invitations`, `alert_rules` kapsamı; `attachments` bağlı kaydı), `drop_scope_references()` |
| `0050_islenmis_olaylar` (T2.15) | `processed_events (consumer, event_id)`: tüketici çerçevesinin tekrar denetimi; yalnız `app_worker` okur, ekler, siler |

## İşletme göçleri (T2.7)

| Göç | İçerik |
|---|---|
| `0014_isletme_uretim` | `production_areas`, `vessels`, `vessel_occupancies`, `transfers`, `transfer_batches`, `packaging_runs`, `packaging_run_lines`, `quality_specs`, `lab_results` (yalnız eklenir), `releases` (yalnız eklenir), `cleaning_records`, `yeast_generations`, `yeast_generation_batches`; `devices.vessel_id` yabancı anahtarı; `drop_scope_references` `vessel` alarm kurallarını da siler; `btree_gist` |
| `0015_vergi_tablolari` | `tax_jurisdictions`, `tax_rate_tables`, `tax_rate_bands` (ortak referans veri) |

- Gerçekleşen (`actual`) tank doluluğu aynı tankta çakışmaz (`vessel_occupancies_no_actual_overlap`, `EXCLUDE USING gist`; yumuşak silinen satır sayılmaz, bitiş `[)`). Planlanan doluluk çakışabilir (planlama uyarısı, ADR-0007).
- `lab_results` ve `releases` uygulama rollerine yalnız INSERT ve SELECT verir; düzeltme yeni satırdır (`lab_results.corrects_id`). Bekletme ve red gerekçesiz olmaz.
- Vergi tabloları `workspace_id`'siz ortak veridir (RLS yok, `staff_users` gibi `SHARED`); uygulama rolleri yalnız okur, yazımı göç ve tohum (`app_owner`) yapar. Aynı ülke, `rate_class` ve `chain_position` için yürürlük aralıkları çakışmaz (`tax_rate_tables_no_overlap`). Bant ve kademeler işlem sonunda denetlenir: sıra ardışık, üst sınırlar kesin artan, son bant açık uçlu (motor D8 kuralı). `verified` satır resmi kaynak, adı, adresi ve tarihiyle gelir; ABV ya da °P kullanan yöntemde girdi yuvarlaması açıkça yazılır. Oran satırı tohumu yok (resmi kaynaktan doğrulanınca yüklenir).

`EditSession` tablo değildir (ADR-0007 madde 4: Redis'te süreli anahtar). Yeni tablo göçleri `secure_workspace_table(tablo, kip, haklar)` ile kurulur; kipler `standard`, `shared` (workspace_id NULL = ortak satır), `append` (yalnız eklenir).

Kurallar (testlerle zorlanır, `tests/catalog.rs`):
- Ortak kütüphane satırı (`ingredients`, `ingredient_*`, `style_guides`, `styles`, `style_mappings`, `package_types`; `workspace_id` NULL) herkes tarafından okunur; uygulama rolü yazamaz, yalnız `app_owner` yazar (`owner_seed` politikası: göç ve tohum).
- Ortak olabilen kayda giden başvuru (malzeme, stil, kılavuz) bileşik anahtarla kurulamaz: `shared_ref_<sütun>` adlı `check_shared_ref` tetikleyicisi şarttır, yoksa katalog testi düşer.
- Yayınlanmış (`draft` olmayan) `recipe_versions` ve satır tabloları değişmez; yalnız `approved -> archived` geçişi serbest. Parti yalnız yayınlanmış sürümden açılır.
- `measurements` yalnız eklenir, `measured_at` üzerinde aylık bölümlüdür; her bölümde RLS ve politika ayrıca kurulur (`create_measurement_partition`, yalnız `app_owner`/göç; worker `ensure_measurement_partitions(n)` çağırır). Bölüm yoksa ekleme `23514` ile düşer; ilk bölümler 2026-01'den göç günü + 2 aya kadar açılır. Tekrar gönderim `INSERT ... ON CONFLICT DO NOTHING` ile.
- `stock_lots.quantity` yalnız `stock_movements` ekleme tetikleyicisiyle değişir; `stock_movements`, `measurements`, `stage_events`, `notification_events`, `support_actions` uygulama rollerine yalnız INSERT ve SELECT verir. `subscriptions` ve `entitlements`'ı yalnız `app_worker` yazar.
- `role_assignments`, `invitations`, `alert_rules` kapsamı ve `attachments` bağlı kaydı (`scope_id`/`record_id`) için tür sütununa göre hedef tablo ve çalışma alanı denetlenir; `vessel` ve `area` türleri `0014`ten beri çalışır (hedef silinince atama, davet ve tank alarm kuralı gider). Hedef silinince bağlı atama ve davetler silinir.

Giriş öncesi işlevler (yalnız `app_api` çağırır): `find_user_by_identity`, `find_accounts_by_email` (0030; "aynı e-posta başka hesapta" ipucu), `find_session_by_token_hash` (0030'da `remember` ve `created_at` eklendi), `find_shared_device_by_key_hash`, `find_invitation_by_token_hash`, `email_login_token_create/consume`, `auth_request_create/consume`. Bakım işlevleri (yalnız `app_worker`): `purge_expired_auth_rows`, `purge_expired_idempotency_keys`.
