-- Denetim kaydı ve altyapı tabloları (adr/0007 › 3 "Altyapı tabloları", › 5; adr/0008 › 6).
SET LOCAL ROLE app_owner;

-- audit_log ----------------------------------------------------------------------------
-- Yalnız eklenir (adr/0007 › 5.4). Uygulama rollerinde yalnız INSERT ve SELECT hakkı var;
-- UPDATE, DELETE ve TRUNCATE'i tetikleyiciler sahip rolde de reddeder. Zincir (özet,
-- kanonik biçim, doğrulama) Rust'tadır (T2.14); veritabanı yalnız biçimi ve sırayı korur.
-- Kişi yalnız UUID ile anılır; ad, e-posta ve serbest metin değerlere yazılmaz.
CREATE TABLE audit_log (
    id              uuid PRIMARY KEY,
    workspace_id    uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    seq             bigint NOT NULL CHECK (seq >= 1),
    created_at      timestamptz NOT NULL DEFAULT now(),
    actor_kind      text NOT NULL CHECK (actor_kind IN ('user', 'support', 'system', 'device')),
    actor_id        uuid,
    action          text NOT NULL CHECK (char_length(action) BETWEEN 1 AND 100),
    record_kind     text NOT NULL CHECK (char_length(record_kind) BETWEEN 1 AND 100),
    record_id       uuid,
    previous_value  jsonb,
    new_value       jsonb,
    reason          text,
    prev_hash       bytea NOT NULL CHECK (octet_length(prev_hash) = 32),
    hash            bytea NOT NULL CHECK (octet_length(hash) = 32),
    UNIQUE (workspace_id, seq)
);
CREATE INDEX audit_log_record_idx ON audit_log (workspace_id, record_kind, record_id);

-- Çalışma alanı başına zincirin son sırası ve özeti; ekleme bu satırı kilitleyerek sıraya girer.
CREATE TABLE audit_heads (
    workspace_id  uuid PRIMARY KEY REFERENCES workspaces (id) ON DELETE CASCADE,
    last_seq      bigint NOT NULL CHECK (last_seq >= 0),
    last_hash     bytea NOT NULL CHECK (octet_length(last_hash) = 32),
    updated_at    timestamptz NOT NULL DEFAULT now()
);

CREATE FUNCTION audit_log_reject_change() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- Çalışma alanı silinirken yabancı anahtar zinciri satırları siler (adr/0007 › 6.3);
    -- bu silme, tetikleyici derinliği 1'i aşan tek yoldur. Doğrudan DELETE derinlik 1'dir.
    IF TG_OP = 'DELETE' AND pg_trigger_depth() > 1 THEN
        RETURN OLD;
    END IF;
    RAISE EXCEPTION 'audit_log is append-only (% rejected)', TG_OP
        USING ERRCODE = 'integrity_constraint_violation';
END
$$;

CREATE TRIGGER no_update_delete BEFORE UPDATE OR DELETE ON audit_log
    FOR EACH ROW EXECUTE FUNCTION audit_log_reject_change();
CREATE TRIGGER no_truncate BEFORE TRUNCATE ON audit_log
    FOR EACH STATEMENT EXECUTE FUNCTION audit_log_reject_change();

-- change_history -----------------------------------------------------------------------
-- Geri alınabilir değişiklik geçmişi (ekipman, profil, kütüphane, taslak tarif); saklama
-- süreli, denetim kaydı değildir. Eski kayıtları worker kendi çalışma alanı bağlamında siler.
CREATE TABLE change_history (
    id              uuid PRIMARY KEY,
    workspace_id    uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    table_name      text NOT NULL CHECK (char_length(table_name) BETWEEN 1 AND 63),
    record_id       uuid NOT NULL,
    record_version  bigint NOT NULL,
    previous_value  jsonb NOT NULL,
    created_at      timestamptz NOT NULL DEFAULT now(),
    created_by      uuid REFERENCES users (id)
);
CREATE INDEX change_history_record_idx
    ON change_history (workspace_id, table_name, record_id, record_version DESC);
CREATE INDEX change_history_created_at_idx ON change_history (workspace_id, created_at);

-- idempotency_keys ---------------------------------------------------------------------
-- (çalışma alanı veya kullanıcı, anahtar) → istek özeti ve yanıt, 24 saat (adr/0008 › 6).
-- Çalışma alanı uçları workspace_id'yi, çalışma alanından bağımsız uçlar (ör. çalışma alanı
-- açma) user_id'yi doldurur; ikisi birden dolu olamaz. Yanıt gelene kadar response_status
-- NULL'dır (409 idempotency_in_progress).
CREATE TABLE idempotency_keys (
    id               uuid PRIMARY KEY,
    workspace_id     uuid REFERENCES workspaces (id) ON DELETE CASCADE,
    user_id          uuid REFERENCES users (id) ON DELETE CASCADE,
    key              text NOT NULL CHECK (char_length(key) BETWEEN 1 AND 255),
    request_hash     bytea NOT NULL CHECK (octet_length(request_hash) = 32),
    response_status  integer CHECK (response_status BETWEEN 100 AND 599),
    response_body    bytea,
    created_at       timestamptz NOT NULL DEFAULT now(),
    expires_at       timestamptz NOT NULL DEFAULT now() + interval '24 hours',
    CHECK (num_nonnulls(workspace_id, user_id) = 1),
    CHECK (response_body IS NULL OR response_status IS NOT NULL)
);
CREATE UNIQUE INDEX idempotency_keys_workspace_key_idx
    ON idempotency_keys (workspace_id, key) WHERE workspace_id IS NOT NULL;
CREATE UNIQUE INDEX idempotency_keys_user_key_idx
    ON idempotency_keys (user_id, key) WHERE user_id IS NOT NULL;
CREATE INDEX idempotency_keys_expires_idx ON idempotency_keys (expires_at);

-- Süresi dolanları siler (worker; RLS altında bağlamsız toplu silme için dar işlev).
CREATE FUNCTION purge_expired_idempotency_keys() RETURNS bigint
    LANGUAGE plpgsql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
DECLARE
    n bigint;
BEGIN
    DELETE FROM idempotency_keys WHERE expires_at < now();
    GET DIAGNOSTICS n = ROW_COUNT;
    RETURN n;
END
$$;
CREATE POLICY owner_purge ON idempotency_keys TO app_owner USING (true);

-- Çalışma alanları arası tablolar (adr/0007 › 2.5): RLS yok, hakla korunur ----------------
-- Müşteri içeriği taşımaz, yalnız kimlik ve olay yükü taşır. workspace_id'nin yabancı
-- anahtarı yoktur: silinen çalışma alanının olayı ve işi yayımlanıp tamamlanmalıdır.

CREATE TABLE outbox (
    id            uuid PRIMARY KEY,
    topic         text NOT NULL CHECK (char_length(topic) BETWEEN 1 AND 200),
    workspace_id  uuid,
    payload       jsonb NOT NULL CHECK (jsonb_typeof(payload) = 'object' AND payload ? 'schema_version'),
    created_at    timestamptz NOT NULL DEFAULT now(),
    published_at  timestamptz
);
CREATE INDEX outbox_unpublished_idx ON outbox (created_at) WHERE published_at IS NULL;
CREATE INDEX outbox_published_idx ON outbox (published_at) WHERE published_at IS NOT NULL;

CREATE TABLE external_events (
    id            uuid PRIMARY KEY,
    source        text NOT NULL CHECK (source IN ('github', 'app_store', 'web_payment', 'device')),
    delivery_id   text NOT NULL CHECK (char_length(delivery_id) BETWEEN 1 AND 255),
    payload       jsonb NOT NULL,
    received_at   timestamptz NOT NULL DEFAULT now(),
    processed_at  timestamptz,
    UNIQUE (source, delivery_id)
);
CREATE INDEX external_events_unprocessed_idx ON external_events (received_at)
    WHERE processed_at IS NULL;

CREATE TABLE scheduled_jobs (
    id            uuid PRIMARY KEY,
    workspace_id  uuid,
    kind          text NOT NULL CHECK (char_length(kind) BETWEEN 1 AND 100),
    run_at        timestamptz NOT NULL,
    status        text NOT NULL DEFAULT 'pending'
                  CHECK (status IN ('pending', 'running', 'done', 'failed')),
    attempts      integer NOT NULL DEFAULT 0 CHECK (attempts >= 0),
    last_error    text,
    payload       jsonb NOT NULL DEFAULT '{"schema_version": 1}'
                  CHECK (jsonb_typeof(payload) = 'object' AND payload ? 'schema_version'),
    created_at    timestamptz NOT NULL DEFAULT now(),
    updated_at    timestamptz NOT NULL DEFAULT now(),
    version       bigint NOT NULL DEFAULT 1
);
CREATE INDEX scheduled_jobs_due_idx ON scheduled_jobs (run_at) WHERE status = 'pending';
CREATE TRIGGER touch BEFORE UPDATE ON scheduled_jobs FOR EACH ROW EXECUTE FUNCTION touch_row();

-- RLS ----------------------------------------------------------------------------------
ALTER TABLE audit_log        ENABLE ROW LEVEL SECURITY;
ALTER TABLE audit_log        FORCE ROW LEVEL SECURITY;
ALTER TABLE audit_heads      ENABLE ROW LEVEL SECURITY;
ALTER TABLE audit_heads      FORCE ROW LEVEL SECURITY;
ALTER TABLE change_history   ENABLE ROW LEVEL SECURITY;
ALTER TABLE change_history   FORCE ROW LEVEL SECURITY;
ALTER TABLE idempotency_keys ENABLE ROW LEVEL SECURITY;
ALTER TABLE idempotency_keys FORCE ROW LEVEL SECURITY;

CREATE POLICY workspace_isolation ON audit_log
    USING (workspace_id = (SELECT app_workspace_id()))
    WITH CHECK (workspace_id = (SELECT app_workspace_id()));
CREATE POLICY workspace_isolation ON audit_heads
    USING (workspace_id = (SELECT app_workspace_id()))
    WITH CHECK (workspace_id = (SELECT app_workspace_id()));
CREATE POLICY workspace_isolation ON change_history
    USING (workspace_id = (SELECT app_workspace_id()))
    WITH CHECK (workspace_id = (SELECT app_workspace_id()));
CREATE POLICY scope_isolation ON idempotency_keys
    USING (workspace_id = (SELECT app_workspace_id()) OR user_id = (SELECT app_user_id()))
    WITH CHECK (workspace_id = (SELECT app_workspace_id()) OR user_id = (SELECT app_user_id()));

-- Haklar -------------------------------------------------------------------------------
GRANT SELECT, INSERT ON audit_log TO app_api, app_worker;
GRANT SELECT, INSERT, UPDATE ON audit_heads TO app_api, app_worker;
GRANT SELECT, INSERT, DELETE ON change_history TO app_api, app_worker;
GRANT SELECT, INSERT, UPDATE, DELETE ON idempotency_keys TO app_api;

-- outbox: API yalnız ekler; okuma, işaretleme ve silme worker'da.
GRANT INSERT ON outbox TO app_api;
GRANT SELECT, INSERT, UPDATE, DELETE ON outbox TO app_worker;
-- external_events: ingest (app_api) ekler ve kendi tekrar denetimi için okur.
GRANT SELECT, INSERT ON external_events TO app_api;
GRANT SELECT, INSERT, UPDATE, DELETE ON external_events TO app_worker;
GRANT INSERT ON scheduled_jobs TO app_api;
GRANT SELECT, INSERT, UPDATE, DELETE ON scheduled_jobs TO app_worker;

REVOKE ALL ON FUNCTION purge_expired_idempotency_keys() FROM PUBLIC;
GRANT EXECUTE ON FUNCTION purge_expired_idempotency_keys() TO app_worker;

RESET ROLE;
