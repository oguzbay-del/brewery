-- Bildirim, hatırlatıcı, uyarı, dosya (adr/0007 › 3 "Bildirim, cihaz, dosya"; T2.6).
SET LOCAL ROLE app_owner;

CREATE TABLE notifications (
    id                   uuid PRIMARY KEY,
    workspace_id         uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    recipient_member_id  uuid NOT NULL,
    priority             smallint NOT NULL DEFAULT 2 CHECK (priority BETWEEN 1 AND 3),
    kind                 text NOT NULL CHECK (kind ~ '^[a-z][a-z0-9_.]*$'),
    -- Sistem metni veritabanında tutulmaz: metin anahtarı ve parametre (adr/0007 › 1.9).
    text_key             text NOT NULL CHECK (char_length(text_key) BETWEEN 1 AND 200),
    params               jsonb NOT NULL DEFAULT '{"schema_version": 1}'
                         CHECK (jsonb_typeof(params) = 'object' AND params ? 'schema_version'),
    batch_id             uuid,
    batch_step_id        uuid,
    status               text NOT NULL DEFAULT 'open' CHECK (status IN (
        'open', 'snoozed', 'acknowledged', 'dismissed', 'resolved')),
    snoozed_until        timestamptz,
    created_at           timestamptz NOT NULL DEFAULT now(),
    updated_at           timestamptz NOT NULL DEFAULT now(),
    deleted_at           timestamptz,
    version              bigint NOT NULL DEFAULT 1,
    created_by           uuid REFERENCES users (id),
    updated_by           uuid REFERENCES users (id),
    UNIQUE (workspace_id, id),
    FOREIGN KEY (workspace_id, recipient_member_id) REFERENCES workspace_members (workspace_id, id) ON DELETE CASCADE,
    FOREIGN KEY (workspace_id, batch_id) REFERENCES batches (workspace_id, id) ON DELETE CASCADE,
    FOREIGN KEY (workspace_id, batch_step_id) REFERENCES batch_steps (workspace_id, id) ON DELETE CASCADE,
    CHECK ((status = 'snoozed') = (snoozed_until IS NOT NULL))
);
CREATE INDEX notifications_recipient_idx ON notifications (workspace_id, recipient_member_id, status)
    WHERE deleted_at IS NULL;

-- Yalnız eklenir. push_target hesap tablosudur (workspace_id yok), düz yabancı anahtar.
CREATE TABLE notification_events (
    id               uuid PRIMARY KEY,
    workspace_id     uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    notification_id  uuid NOT NULL,
    kind             text NOT NULL CHECK (kind IN ('sent', 'read', 'acknowledged', 'snoozed')),
    push_target_id   uuid REFERENCES push_targets (id) ON DELETE SET NULL,
    push_result      text CHECK (char_length(push_result) <= 200),
    occurred_at      timestamptz NOT NULL DEFAULT now(),
    created_at       timestamptz NOT NULL DEFAULT now(),
    created_by       uuid REFERENCES users (id),
    FOREIGN KEY (workspace_id, notification_id) REFERENCES notifications (workspace_id, id) ON DELETE CASCADE
);
CREATE INDEX notification_events_notification_idx ON notification_events (workspace_id, notification_id);

-- Tetiklenmesi scheduled_jobs üzerinden olur; scheduled_jobs çalışma alanları arası
-- tablodur, bu yüzden `scheduled_job_id` için yabancı anahtar yoktur.
CREATE TABLE reminders (
    id                   uuid PRIMARY KEY,
    workspace_id         uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    recipient_member_id  uuid,
    batch_id             uuid,
    batch_step_id        uuid,
    rule                 text NOT NULL CHECK (rule IN ('at_time', 'before_step', 'at_step', 'after_step')),
    offset_min           integer NOT NULL DEFAULT 0,
    remind_at            timestamptz NOT NULL,
    status               text NOT NULL DEFAULT 'scheduled' CHECK (status IN ('scheduled', 'fired', 'cancelled')),
    scheduled_job_id     uuid,
    created_at           timestamptz NOT NULL DEFAULT now(),
    updated_at           timestamptz NOT NULL DEFAULT now(),
    deleted_at           timestamptz,
    version              bigint NOT NULL DEFAULT 1,
    created_by           uuid REFERENCES users (id),
    updated_by           uuid REFERENCES users (id),
    UNIQUE (workspace_id, id),
    FOREIGN KEY (workspace_id, recipient_member_id) REFERENCES workspace_members (workspace_id, id) ON DELETE CASCADE,
    FOREIGN KEY (workspace_id, batch_id) REFERENCES batches (workspace_id, id) ON DELETE CASCADE,
    FOREIGN KEY (workspace_id, batch_step_id) REFERENCES batch_steps (workspace_id, id) ON DELETE CASCADE,
    CHECK (rule = 'at_time' OR batch_step_id IS NOT NULL)
);
CREATE INDEX reminders_due_idx ON reminders (remind_at) WHERE status = 'scheduled' AND deleted_at IS NULL;

-- Koşul JSONB'dir ve bir Rust tipine karşılık gelir (adr/0007 › 1.7b). Kapsam (parti, tank,
-- cihaz) çok biçimlidir: scope_id'yi tetikleyici denetler (0013).
CREATE TABLE alert_rules (
    id             uuid PRIMARY KEY,
    workspace_id   uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    name           text NOT NULL CHECK (char_length(name) BETWEEN 1 AND 200),
    condition      jsonb NOT NULL CHECK (jsonb_typeof(condition) = 'object' AND condition ? 'schema_version'),
    priority       smallint NOT NULL DEFAULT 2 CHECK (priority BETWEEN 1 AND 3),
    scope_kind     text CHECK (scope_kind IN ('batch', 'vessel', 'device')),
    scope_id       uuid,
    enabled        boolean NOT NULL DEFAULT true,
    muted_until    timestamptz,
    created_at     timestamptz NOT NULL DEFAULT now(),
    updated_at     timestamptz NOT NULL DEFAULT now(),
    deleted_at     timestamptz,
    version        bigint NOT NULL DEFAULT 1,
    created_by     uuid REFERENCES users (id),
    updated_by     uuid REFERENCES users (id),
    UNIQUE (workspace_id, id),
    CHECK ((scope_kind IS NULL) = (scope_id IS NULL))
);

-- attachments --------------------------------------------------------------------------
-- Bağlı kayıt çok biçimlidir (parti, tadım notu, destek mesajı): hedefi tetikleyici denetler.
CREATE TABLE attachments (
    id             uuid PRIMARY KEY,
    workspace_id   uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    kind           text NOT NULL CHECK (kind IN ('photo', 'note', 'file')),
    record_kind    text NOT NULL CHECK (record_kind IN ('batch', 'tasting_note', 'support_message')),
    record_id      uuid NOT NULL,
    object_key     text NOT NULL CHECK (char_length(object_key) BETWEEN 1 AND 1024),
    content_type   text NOT NULL CHECK (char_length(content_type) BETWEEN 1 AND 255),
    size_bytes     bigint NOT NULL CHECK (size_bytes >= 0),
    sha256         bytea NOT NULL CHECK (octet_length(sha256) = 32),
    caption        text,
    created_at     timestamptz NOT NULL DEFAULT now(),
    updated_at     timestamptz NOT NULL DEFAULT now(),
    deleted_at     timestamptz,
    version        bigint NOT NULL DEFAULT 1,
    created_by     uuid REFERENCES users (id),
    updated_by     uuid REFERENCES users (id),
    UNIQUE (workspace_id, id)
);
CREATE INDEX attachments_record_idx ON attachments (workspace_id, record_kind, record_id);

-- scanned_documents --------------------------------------------------------------------
-- Yasal tutma işaretli belge saklama süresi dolmadan silinmez; çalışma alanı silme zinciri
-- de bu kurala takılır (adr/0007 › 6.3).
CREATE TABLE scanned_documents (
    id               uuid PRIMARY KEY,
    workspace_id     uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    batch_id         uuid,
    form_type        text NOT NULL CHECK (form_type ~ '^[a-z][a-z0-9_]*$'),
    scanned_by       uuid REFERENCES users (id),
    object_key       text NOT NULL CHECK (char_length(object_key) BETWEEN 1 AND 1024),
    content_type     text NOT NULL CHECK (char_length(content_type) BETWEEN 1 AND 255),
    size_bytes       bigint NOT NULL CHECK (size_bytes >= 0),
    sha256           bytea NOT NULL CHECK (octet_length(sha256) = 32),
    retention_until  timestamptz,
    legal_hold       boolean NOT NULL DEFAULT false,
    created_at       timestamptz NOT NULL DEFAULT now(),
    updated_at       timestamptz NOT NULL DEFAULT now(),
    deleted_at       timestamptz,
    version          bigint NOT NULL DEFAULT 1,
    created_by       uuid REFERENCES users (id),
    updated_by       uuid REFERENCES users (id),
    UNIQUE (workspace_id, id),
    FOREIGN KEY (workspace_id, batch_id) REFERENCES batches (workspace_id, id)
);

CREATE FUNCTION scanned_documents_guard_hold() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF OLD.legal_hold AND (OLD.retention_until IS NULL OR OLD.retention_until > now()) THEN
        RAISE EXCEPTION 'scanned document % is under legal hold', OLD.id
            USING ERRCODE = 'integrity_constraint_violation';
    END IF;
    RETURN OLD;
END
$$;
CREATE TRIGGER legal_hold BEFORE DELETE ON scanned_documents
    FOR EACH ROW EXECUTE FUNCTION scanned_documents_guard_hold();

ALTER TABLE water_profiles
    ADD FOREIGN KEY (workspace_id, source_document_id) REFERENCES scanned_documents (workspace_id, id);

-- import_sources, export_jobs ---------------------------------------------------------------
CREATE TABLE import_sources (
    id                  uuid PRIMARY KEY,
    workspace_id        uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    object_key          text NOT NULL CHECK (char_length(object_key) BETWEEN 1 AND 1024),
    format              text NOT NULL CHECK (format IN ('beerxml', 'beersmith', 'brewfather', 'json', 'csv')),
    status              text NOT NULL DEFAULT 'uploaded' CHECK (status IN (
        'uploaded', 'parsing', 'parsed', 'failed', 'applied')),
    mapping_decisions   jsonb CHECK (jsonb_typeof(mapping_decisions) = 'object' AND mapping_decisions ? 'schema_version'),
    recipe_id           uuid,
    error               text,
    created_at          timestamptz NOT NULL DEFAULT now(),
    updated_at          timestamptz NOT NULL DEFAULT now(),
    deleted_at          timestamptz,
    version             bigint NOT NULL DEFAULT 1,
    created_by          uuid REFERENCES users (id),
    updated_by          uuid REFERENCES users (id),
    UNIQUE (workspace_id, id),
    FOREIGN KEY (workspace_id, recipe_id) REFERENCES recipes (workspace_id, id)
);

CREATE TABLE export_jobs (
    id                 uuid PRIMARY KEY,
    workspace_id       uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    scope              text NOT NULL CHECK (scope IN ('recipe', 'batch', 'workspace', 'account_data')),
    scope_id           uuid,
    format             text NOT NULL CHECK (format IN ('beerxml', 'json', 'csv', 'pdf', 'zip')),
    status             text NOT NULL DEFAULT 'queued' CHECK (status IN ('queued', 'running', 'done', 'failed')),
    result_object_key  text,
    expires_at         timestamptz,
    created_at         timestamptz NOT NULL DEFAULT now(),
    updated_at         timestamptz NOT NULL DEFAULT now(),
    deleted_at         timestamptz,
    version            bigint NOT NULL DEFAULT 1,
    created_by         uuid REFERENCES users (id),
    updated_by         uuid REFERENCES users (id),
    UNIQUE (workspace_id, id)
);

SELECT secure_workspace_table(t) FROM unnest(ARRAY[
    'notifications', 'reminders', 'alert_rules', 'attachments', 'scanned_documents',
    'import_sources', 'export_jobs']) t;
SELECT secure_workspace_table('notification_events', 'append', 'SELECT, INSERT');

RESET ROLE;
