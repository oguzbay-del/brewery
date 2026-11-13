-- Üyelik, rol, davet ve ortak cihaz (adr/0007 › 3; adr/0009 › 6-8).
-- Çalışma alanı tabloları: `workspace_id NOT NULL`, RLS, üst kayda bileşik yabancı anahtar
-- (adr/0007 › 2.7): üst tabloda UNIQUE (workspace_id, id), alt tabloda
-- FOREIGN KEY (workspace_id, parent_id).
SET LOCAL ROLE app_owner;

-- workspace_members --------------------------------------------------------------------
CREATE TABLE workspace_members (
    id            uuid PRIMARY KEY,
    workspace_id  uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    user_id       uuid NOT NULL REFERENCES users (id),
    status        text NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'disabled')),
    joined_at     timestamptz NOT NULL DEFAULT now(),
    created_at    timestamptz NOT NULL DEFAULT now(),
    updated_at    timestamptz NOT NULL DEFAULT now(),
    deleted_at    timestamptz,
    version       bigint NOT NULL DEFAULT 1,
    created_by    uuid REFERENCES users (id),
    updated_by    uuid REFERENCES users (id),
    UNIQUE (workspace_id, id),
    UNIQUE (workspace_id, user_id)
);
CREATE INDEX workspace_members_user_id_idx ON workspace_members (user_id);

-- roles --------------------------------------------------------------------------------
CREATE TABLE roles (
    id              uuid PRIMARY KEY,
    workspace_id    uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    name            text NOT NULL CHECK (char_length(name) BETWEEN 1 AND 100),
    -- Hazır rolden kopyalandıysa şablon anahtarı (tohum verisi, T2.13).
    template_key    text CHECK (template_key ~ '^[a-z][a-z0-9_]*$'),
    landing_screen  text CHECK (landing_screen ~ '^[a-z][a-z0-9_]*$'),
    created_at      timestamptz NOT NULL DEFAULT now(),
    updated_at      timestamptz NOT NULL DEFAULT now(),
    deleted_at      timestamptz,
    version         bigint NOT NULL DEFAULT 1,
    created_by      uuid REFERENCES users (id),
    updated_by      uuid REFERENCES users (id),
    UNIQUE (workspace_id, id)
);
CREATE UNIQUE INDEX roles_workspace_name_idx ON roles (workspace_id, lower(name))
    WHERE deleted_at IS NULL;

-- role_permissions ---------------------------------------------------------------------
-- İzinler kodda sabit listedir (adr/0009 › 8.1); sütun CHECK listeye bağlar. Yeni izin
-- kodla ve bu CHECK'i değiştiren göçle birlikte gelir.
CREATE TABLE role_permissions (
    workspace_id  uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    role_id       uuid NOT NULL,
    permission    text NOT NULL CHECK (permission IN (
        'recipe.view', 'recipe.edit', 'recipe.approve',
        'batch.plan', 'batch.record', 'batch.correct_closed',
        'quality.record', 'release.approve', 'stock.move', 'cost.view',
        'reports.export', 'settings.equipment', 'users.manage',
        'workspace.manage', 'billing.manage', 'support.grant', 'devices.manage')),
    created_at    timestamptz NOT NULL DEFAULT now(),
    created_by    uuid REFERENCES users (id),
    PRIMARY KEY (workspace_id, role_id, permission),
    FOREIGN KEY (workspace_id, role_id) REFERENCES roles (workspace_id, id) ON DELETE CASCADE
);

-- role_assignments ---------------------------------------------------------------------
-- Üye × rol, isteğe bağlı kapsam. Ev modunda arkadaş paylaşımı da bu yolla yapılır.
-- scope_id farklı tablolara (tank, alan, tarif, parti) işaret edebildiği için yabancı
-- anahtarı yoktur; T2.6/T2.7 kapsam hedefini denetleyen tetikleyici ekler.
CREATE TABLE role_assignments (
    id            uuid PRIMARY KEY,
    workspace_id  uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    member_id     uuid NOT NULL,
    role_id       uuid NOT NULL,
    scope_kind    text CHECK (scope_kind IN ('vessel', 'area', 'recipe', 'batch')),
    scope_id      uuid,
    created_at    timestamptz NOT NULL DEFAULT now(),
    updated_at    timestamptz NOT NULL DEFAULT now(),
    deleted_at    timestamptz,
    version       bigint NOT NULL DEFAULT 1,
    created_by    uuid REFERENCES users (id),
    updated_by    uuid REFERENCES users (id),
    CHECK ((scope_kind IS NULL) = (scope_id IS NULL)),
    FOREIGN KEY (workspace_id, member_id) REFERENCES workspace_members (workspace_id, id) ON DELETE CASCADE,
    FOREIGN KEY (workspace_id, role_id) REFERENCES roles (workspace_id, id)
);
-- Aynı üye, rol ve kapsam bir kez atanır (kapsamsız atama da: NULL'lar eşit sayılır).
CREATE UNIQUE INDEX role_assignments_unique_idx
    ON role_assignments (workspace_id, member_id, role_id, scope_kind, scope_id) NULLS NOT DISTINCT
    WHERE deleted_at IS NULL;
CREATE INDEX role_assignments_member_idx ON role_assignments (workspace_id, member_id);

-- invitations --------------------------------------------------------------------------
-- Belirteç 32 bayt, yalnız özeti saklanır; 7 gün geçerli, tek kullanımlık (adr/0009 › 6).
-- Davet eden `created_by`'dır.
CREATE TABLE invitations (
    id            uuid PRIMARY KEY,
    workspace_id  uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    token_hash    bytea NOT NULL UNIQUE CHECK (octet_length(token_hash) = 32),
    role_id       uuid NOT NULL,
    scope_kind    text CHECK (scope_kind IN ('vessel', 'area', 'recipe', 'batch')),
    scope_id      uuid,
    expires_at    timestamptz NOT NULL,
    accepted_by   uuid REFERENCES users (id),
    accepted_at   timestamptz,
    revoked_at    timestamptz,
    created_at    timestamptz NOT NULL DEFAULT now(),
    updated_at    timestamptz NOT NULL DEFAULT now(),
    deleted_at    timestamptz,
    version       bigint NOT NULL DEFAULT 1,
    created_by    uuid REFERENCES users (id),
    updated_by    uuid REFERENCES users (id),
    CHECK ((scope_kind IS NULL) = (scope_id IS NULL)),
    CHECK ((accepted_by IS NULL) = (accepted_at IS NULL)),
    FOREIGN KEY (workspace_id, role_id) REFERENCES roles (workspace_id, id)
);
CREATE INDEX invitations_workspace_idx ON invitations (workspace_id);

-- shared_devices -----------------------------------------------------------------------
CREATE TABLE shared_devices (
    id                 uuid PRIMARY KEY,
    workspace_id       uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    name               text NOT NULL CHECK (char_length(name) BETWEEN 1 AND 100),
    opened_by          uuid NOT NULL REFERENCES users (id),
    -- Cihaz anahtarı cihazda üretilir; sunucu yalnız SHA-256 özetini tutar (adr/0009 › 7.1).
    device_key_hash    bytea NOT NULL UNIQUE CHECK (octet_length(device_key_hash) = 32),
    idle_timeout_min   integer NOT NULL DEFAULT 5 CHECK (idle_timeout_min BETWEEN 1 AND 60),
    revoked_at         timestamptz,
    created_at         timestamptz NOT NULL DEFAULT now(),
    updated_at         timestamptz NOT NULL DEFAULT now(),
    deleted_at         timestamptz,
    version            bigint NOT NULL DEFAULT 1,
    created_by         uuid REFERENCES users (id),
    updated_by         uuid REFERENCES users (id),
    UNIQUE (workspace_id, id)
);

ALTER TABLE sessions
    ADD CONSTRAINT sessions_shared_device_id_fkey
    FOREIGN KEY (shared_device_id) REFERENCES shared_devices (id) ON DELETE CASCADE;

-- member_pins --------------------------------------------------------------------------
CREATE TABLE member_pins (
    id               uuid PRIMARY KEY,
    workspace_id     uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    member_id        uuid NOT NULL,
    -- Argon2id PHC dizgisi (adr/0009 › 7.2); PIN'in kendisi saklanmaz.
    pin_hash         text NOT NULL,
    failed_attempts  integer NOT NULL DEFAULT 0 CHECK (failed_attempts >= 0),
    locked_until     timestamptz,
    created_at       timestamptz NOT NULL DEFAULT now(),
    updated_at       timestamptz NOT NULL DEFAULT now(),
    deleted_at       timestamptz,
    version          bigint NOT NULL DEFAULT 1,
    created_by       uuid REFERENCES users (id),
    updated_by       uuid REFERENCES users (id),
    UNIQUE (workspace_id, member_id),
    FOREIGN KEY (workspace_id, member_id) REFERENCES workspace_members (workspace_id, id) ON DELETE CASCADE
);

-- device_sessions ----------------------------------------------------------------------
CREATE TABLE device_sessions (
    id                 uuid PRIMARY KEY,
    workspace_id       uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    shared_device_id   uuid NOT NULL,
    member_id          uuid NOT NULL,
    session_id         uuid REFERENCES sessions (id) ON DELETE SET NULL,
    started_at         timestamptz NOT NULL DEFAULT now(),
    last_activity_at   timestamptz NOT NULL DEFAULT now(),
    ended_at           timestamptz,
    created_at         timestamptz NOT NULL DEFAULT now(),
    updated_at         timestamptz NOT NULL DEFAULT now(),
    deleted_at         timestamptz,
    version            bigint NOT NULL DEFAULT 1,
    created_by         uuid REFERENCES users (id),
    updated_by         uuid REFERENCES users (id),
    FOREIGN KEY (workspace_id, shared_device_id) REFERENCES shared_devices (workspace_id, id) ON DELETE CASCADE,
    FOREIGN KEY (workspace_id, member_id) REFERENCES workspace_members (workspace_id, id) ON DELETE CASCADE
);
CREATE INDEX device_sessions_active_idx ON device_sessions (workspace_id, shared_device_id)
    WHERE ended_at IS NULL;

-- Ortak kurulum: sürüm tetikleyicisi, RLS, haklar ----------------------------------------
CREATE TRIGGER touch BEFORE UPDATE ON workspace_members FOR EACH ROW EXECUTE FUNCTION touch_row();
CREATE TRIGGER touch BEFORE UPDATE ON roles             FOR EACH ROW EXECUTE FUNCTION touch_row();
CREATE TRIGGER touch BEFORE UPDATE ON role_assignments  FOR EACH ROW EXECUTE FUNCTION touch_row();
CREATE TRIGGER touch BEFORE UPDATE ON invitations       FOR EACH ROW EXECUTE FUNCTION touch_row();
CREATE TRIGGER touch BEFORE UPDATE ON shared_devices    FOR EACH ROW EXECUTE FUNCTION touch_row();
CREATE TRIGGER touch BEFORE UPDATE ON member_pins       FOR EACH ROW EXECUTE FUNCTION touch_row();
CREATE TRIGGER touch BEFORE UPDATE ON device_sessions   FOR EACH ROW EXECUTE FUNCTION touch_row();

DO $$
DECLARE
    t text;
BEGIN
    FOREACH t IN ARRAY ARRAY[
        'workspace_members', 'roles', 'role_permissions', 'role_assignments',
        'invitations', 'shared_devices', 'member_pins', 'device_sessions']
    LOOP
        EXECUTE format('ALTER TABLE %I ENABLE ROW LEVEL SECURITY', t);
        EXECUTE format('ALTER TABLE %I FORCE ROW LEVEL SECURITY', t);
        IF t <> 'workspace_members' THEN
            EXECUTE format(
                'CREATE POLICY workspace_isolation ON %I
                     USING (workspace_id = (SELECT app_workspace_id()))
                     WITH CHECK (workspace_id = (SELECT app_workspace_id()))', t);
        END IF;
        EXECUTE format(
            'GRANT SELECT, INSERT, UPDATE, DELETE ON %I TO app_api, app_worker', t);
    END LOOP;
END
$$;

-- workspace_members: kullanıcı kendi üyeliklerini bütün çalışma alanlarında listeleyebilir
-- (adr/0007 satır `workspace_members`). Okuma kuralı ADR'deki gibi; yazma yalnız çalışma
-- alanı bağlamında olur, yoksa kullanıcı kendi üyeliğini (ör. devre dışı → etkin) değiştirir.
CREATE POLICY member_select ON workspace_members FOR SELECT
    USING (user_id = (SELECT app_user_id()) OR workspace_id = (SELECT app_workspace_id()));
CREATE POLICY member_insert ON workspace_members FOR INSERT
    WITH CHECK (workspace_id = (SELECT app_workspace_id()));
CREATE POLICY member_update ON workspace_members FOR UPDATE
    USING (workspace_id = (SELECT app_workspace_id()))
    WITH CHECK (workspace_id = (SELECT app_workspace_id()));
CREATE POLICY member_delete ON workspace_members FOR DELETE
    USING (workspace_id = (SELECT app_workspace_id()));

-- Girişten önceki aramalar (adr/0007 › 2.6): sahibe açık okuma politikası + dar işlev.
CREATE POLICY owner_lookup ON shared_devices TO app_owner USING (true) WITH CHECK (true);
CREATE POLICY owner_lookup ON invitations    TO app_owner USING (true) WITH CHECK (true);

-- Cihaz anahtarıyla cihazı bulur (iptal edilmişse ya da silinmişse bulunmaz).
CREATE FUNCTION find_shared_device_by_key_hash(p_key_hash bytea)
    RETURNS TABLE (id uuid, workspace_id uuid, idle_timeout_min integer)
    LANGUAGE sql STABLE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
    SELECT d.id, d.workspace_id, d.idle_timeout_min
    FROM shared_devices d
    WHERE d.device_key_hash = p_key_hash AND d.revoked_at IS NULL AND d.deleted_at IS NULL
$$;

-- Davet belirteciyle daveti bulur. Süresi dolmuş, kabul edilmiş, iptal edilmiş ya da
-- silinmiş davet hiç bulunmaz ("404 gibi görünür", adr/0009 › 6.3).
CREATE FUNCTION find_invitation_by_token_hash(p_token_hash bytea)
    RETURNS TABLE (
        id uuid, workspace_id uuid, role_id uuid, scope_kind text, scope_id uuid,
        expires_at timestamptz, invited_by uuid)
    LANGUAGE sql STABLE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
    SELECT i.id, i.workspace_id, i.role_id, i.scope_kind, i.scope_id, i.expires_at, i.created_by
    FROM invitations i
    WHERE i.token_hash = p_token_hash
      AND i.accepted_at IS NULL AND i.revoked_at IS NULL AND i.deleted_at IS NULL
      AND i.expires_at > now()
$$;

REVOKE ALL ON FUNCTION
    find_shared_device_by_key_hash(bytea), find_invitation_by_token_hash(bytea)
    FROM PUBLIC;
GRANT EXECUTE ON FUNCTION
    find_shared_device_by_key_hash(bytea), find_invitation_by_token_hash(bytea)
    TO app_api;

RESET ROLE;
