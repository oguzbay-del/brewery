# db

## Ne yapar
- PostgreSQL erişiminin tek kapısı: sqlx havuzu (`DATABASE_URL` ya da `DATABASE_URL_FILE`), göçler, istek bağlamı yardımcıları (`DbContext`, `begin`: işlem başına `SET LOCAL app.*`).
- Göçler `sunucu/migrations/` altındadır ve `migrate()` ile (ya da `migrate` ikilisiyle) uygulanır; imaja gömülüdür.
- `migrate` ikilisi compose'da ve Kubernetes'te tek seferlik çalışır; uygulama servisleri göç çalıştırmaz.

## Neye dokunmaz
- Tablo ve sütun kararları `adr/0007-veri-modeli.md`'dedir; Ev tabloları (T2.6) `0007`–`0013` göçlerindedir, İşletme (T2.7) kendi göçünde gelir.
- Uygulama servislerinin iş mantığı burada yazılmaz.

## Roller ve göç kuralları
- Üç rol: `app_owner` (tablo sahibi, oturum açamaz), `app_api`, `app_worker` (RLS altında, sahip değil, BYPASSRLS yok). `0002_roller.sql` rolleri yoksa oluşturur (NOLOGIN); göçlerde parola yoktur.
- Oturum açma ve parola göçten sonra `migrate` ikilisinde verilir (`APP_API_PASSWORD`, `APP_WORKER_PASSWORD`, ya da `*_FILE`); tekrar çalışması parolayı yeniler. Göç rolü `MIGRATE_DATABASE_URL` (yoksa `DATABASE_URL`) ile bağlanır: süper kullanıcı ya da CREATEROLE sahibi ve veritabanı sahibi. Uygulama servisleri `DATABASE_URL` ile `app_api` (worker: `app_worker`) olarak bağlanır.
- Her tablo göçü `SET LOCAL ROLE app_owner;` ile başlar ve `RESET ROLE;` ile biter: nesneler `app_owner`'ındır, sqlx kaydı göç rolüyle yazılır. Her tablo için `ENABLE` + `FORCE ROW LEVEL SECURITY`, en az bir politika ve açık `GRANT`; `PUBLIC`'e hak verilmez.
- Politikalar `app_workspace_id()` ve `app_user_id()` işlevlerini `(SELECT ...)` biçiminde kullanır (boş metni NULL sayar; `SET LOCAL` bitince ayar boş metne döner).
- Girişten önceki aramalar dar `SECURITY DEFINER` işlevlerdir (sahibi `app_owner`, `search_path` sabit, `PUBLIC` çağıramaz); bu tablolarda yalnız `app_owner`'a açık `owner_lookup` politikası bulunur. Yenisi eklenirse `tests/catalog.rs` içindeki `OWNER_LOOKUP` listesi güncellenir.
- `workspace_id` olan tabloda üst kayda bileşik yabancı anahtar (`(workspace_id, parent_id)`), katalog testi zorlar. Çalışma alanları arası tablolar (`outbox`, `scheduled_jobs`, `external_events`, `processed_events`) RLS'siz, hakla korunur; listede olmayan tablo RLS'siz bırakılırsa test düşer.
- Denetim zinciri `src/audit.rs`'dedir (özet ve doğrulama Rust'ta, göç yok): ekleme yalnız `append_audit` ile, istek işleminde; satır alanları değişirse `canonical_row` biçimi de değişir ve eski zincir bozulur (yeni biçim sürümü gerekir). Test: `tests/audit_chain.rs`.
- `audit_log` yalnız eklenir: uygulama rollerinde UPDATE/DELETE/TRUNCATE hakkı yoktur, tetikleyici sahip rolde de reddeder (çalışma alanı silinirken yabancı anahtar zinciri hariç).

- Silme (T2.9): `app_api`'nin `workspaces` ve `users` üzerinde DELETE hakkı yoktur; kalıcı çalışma alanı silme yalnız worker'da `purge_workspace()` ile (silme kaydı `workspace_tombstones`'a yazılır). Kalıcı kayıt silmede değişmez başvurular NULL'a iner, değer kopya sütununda kalır (`snapshot_reference`).
- Saklama (İşletme): `retention_until` + `legal_hold` taşıyan tabloların silinmesi ve yumuşak silinmesi reddedilir; yasal tutmayı yalnız `release_legal_hold()` kaldırır. Süreler `retention_rules` (doğrulanacak).
- `pg_trigger_depth()` kapıları (stok önbelleği, değişmez sürüm, denetim kaydı) tetikleyiciden gelen her yazıya açıktır: bu tablolara yazan yeni tetikleyici işlevi `tests/f21_fixes.rs` listesine bilinçli eklenir.
- `external_events` yükünü API okuyamaz (`external_event_seen()`); hesap düzeyi olaylar `account_events`'e yazılır (zincirsiz, kullanıcıya bağlı). Denetim satırında kişisel alan için yalnız `changed_marker()`.

## Nasıl test edilir
- Gerçek PostgreSQL gerekir: `make up` (ya da yalnız `docker compose -f altyapi/compose/compose.yaml up -d postgres`), sonra kökte `make test`. Elle: `DATABASE_URL=postgres://demleme:demleme-yerel@127.0.0.1:5432/demleme cargo test -p db`.
- `#[sqlx::test]` her test için geçici veritabanı açar, göçleri uygular ve sonda siler. Uygulama davranışı `tests/common`'daki `pool_as` ile sınanır: bağlantılar `SET ROLE app_api` (ya da `app_worker`, `app_owner`) ile açılır; süper kullanıcının kendi RLS muafiyeti `SET ROLE` sonrası geçerli olmaz. Süper kullanıcı oturumuyla `SET ROLE` her role geçebilir; bu yüzden "uygulama rolü sahip olamaz" gibi savlar `pg_roles`/katalogdan sınanır.
- Testler: `migrations.rs` (göç), `rls.rs` (çalışma alanı ayrımı), `catalog.rs` (katalog taraması, haklar), `audit_log.rs`, `auth_access.rs` (hesap tabloları ve giriş öncesi işlevler), `infra.rs` (outbox, tekrar güvenliği), `library.rs` (ortak kütüphane), `recipes.rs` (değişmez sürüm), `batches.rs` (parti, ölçüm bölümleri, stok, kapsam tetikleyicisi, yazma kısıtları), `f21_fixes.rs` (F2.1 düzeltmeleri, T2.9).
- Sorgu önbelleği: kökte `make sqlx-prepare`; `sunucu/.sqlx/` commit edilir, derleme `SQLX_OFFLINE=true` ile çalışır.

- Yeni tablo göçü `secure_workspace_table(tablo, kip, haklar)` çağırır (0007). Ortak satır tutan tablo `shared` kipindedir, ortak kayda başvuru `shared_ref_<sütun>` tetikleyicisi ister ve tablo `tests/catalog.rs` `OWNER_SEED` listesine eklenir; ölçüm gibi bölümlü tablonun her bölümü ayrıca RLS ve politika taşır.

## Kurallar
- Göç dosyaları `NNNN_ad.sql` (dört haneli sıra, küçük harf, alt çizgi); uygulanmış göç değiştirilmez, yeni göç eklenir.
- Kod İngilizce; belgeler Türkçe.
- Commit mesajı iş numarasıyla başlar: `T2.x: ...`.
