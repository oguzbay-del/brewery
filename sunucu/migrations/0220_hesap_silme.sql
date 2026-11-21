-- T2.31: hesap silme ve hesap verisi dışa aktarma.
-- account_deletions: silme talebi (talep, kalıcı silme zamanı, geri alma). Kullanıcı başına tek
-- satır; yeni talep satırı yeniler. Kalıcı silmeyi yalnız worker yapar, dar SECURITY DEFINER
-- işlevlerle (adr/0007 › 6.2). Bekleme süresi (30 gün, doğrulanacak) kodda tek sabittir
-- (`db::account::RETENTION_DAYS`); satır `purge_at` değerini taşır.
SET LOCAL ROLE app_owner;

CREATE TABLE account_deletions (
    user_id          uuid PRIMARY KEY REFERENCES users (id),
    requested_at     timestamptz NOT NULL DEFAULT now(),
    purge_at         timestamptz NOT NULL,
    restored_at      timestamptz,
    purged_at        timestamptz,
    attempts         integer NOT NULL DEFAULT 0 CHECK (attempts >= 0),
    last_attempt_at  timestamptz,
    last_error       text,
    created_at       timestamptz NOT NULL DEFAULT now(),
    updated_at       timestamptz NOT NULL DEFAULT now(),
    version          bigint NOT NULL DEFAULT 1,
    CHECK (purge_at > requested_at)
);
CREATE INDEX account_deletions_due_idx ON account_deletions (purge_at)
    WHERE restored_at IS NULL AND purged_at IS NULL;
CREATE TRIGGER touch BEFORE UPDATE ON account_deletions
    FOR EACH ROW EXECUTE FUNCTION touch_row();

ALTER TABLE account_deletions ENABLE ROW LEVEL SECURITY;
ALTER TABLE account_deletions FORCE ROW LEVEL SECURITY;
CREATE POLICY own_rows ON account_deletions
    USING (user_id = (SELECT app_user_id())) WITH CHECK (user_id = (SELECT app_user_id()));
-- Worker kullanıcıları bağlamsız listeler (account_deletions_due).
CREATE POLICY owner_lookup ON account_deletions TO app_owner USING (true) WITH CHECK (true);
GRANT SELECT, INSERT, UPDATE ON account_deletions TO app_api;

-- Destek talebine bağlı kayıtlar talep silinince boşalır: talebi silmek destek eylem kaydını
-- ve izin kaydını silmemeli (yalnız eklenir tablo).
ALTER TABLE support_requests DROP CONSTRAINT support_requests_workspace_id_previous_request_id_fkey;
ALTER TABLE support_requests ADD FOREIGN KEY (workspace_id, previous_request_id)
    REFERENCES support_requests (workspace_id, id) ON DELETE SET NULL (previous_request_id);
ALTER TABLE support_grants DROP CONSTRAINT support_grants_workspace_id_request_id_fkey;
ALTER TABLE support_grants ADD FOREIGN KEY (workspace_id, request_id)
    REFERENCES support_requests (workspace_id, id) ON DELETE SET NULL (request_id);
ALTER TABLE support_actions DROP CONSTRAINT support_actions_workspace_id_request_id_fkey;
ALTER TABLE support_actions ADD FOREIGN KEY (workspace_id, request_id)
    REFERENCES support_requests (workspace_id, id) ON DELETE SET NULL (request_id);

-- Kullanıcının üyeliği olan çalışma alanları: kip, "tek yönetici" ve üye kimliği. Yönetici =
-- bütün çalışma alanı için `users.manage` izni olan etkin üye. Yalnız başka işlevler çağırır
-- (kimseye EXECUTE verilmez).
CREATE FUNCTION account_workspaces_of(p_user_id uuid)
    RETURNS TABLE (workspace_id uuid, mode text, sole_admin boolean, member_id uuid)
    LANGUAGE plpgsql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
#variable_conflict use_column
DECLARE
    prev_user text := current_setting('app.user_id', true);
    prev_ws text := current_setting('app.workspace_id', true);
    ids uuid[];
    ws uuid;
    mine bigint;
    others bigint;
    m text;
    m_id uuid;
BEGIN
    PERFORM set_config('app.user_id', p_user_id::text, true);
    PERFORM set_config('app.workspace_id', '', true);
    SELECT array_agg(wm.workspace_id ORDER BY wm.joined_at, wm.id) INTO ids
    FROM workspace_members wm WHERE wm.user_id = p_user_id AND wm.deleted_at IS NULL;
    FOREACH ws IN ARRAY coalesce(ids, ARRAY[]::uuid[]) LOOP
        PERFORM set_config('app.workspace_id', ws::text, true);
        SELECT w.mode INTO m FROM workspaces w WHERE w.id = ws AND w.deleted_at IS NULL;
        CONTINUE WHEN NOT FOUND;
        SELECT wm.id INTO m_id FROM workspace_members wm
        WHERE wm.workspace_id = ws AND wm.user_id = p_user_id;
        SELECT count(DISTINCT x.id) FILTER (WHERE x.user_id = p_user_id),
               count(DISTINCT x.id) FILTER (WHERE x.user_id <> p_user_id)
          INTO mine, others
        FROM (SELECT mm.id, mm.user_id FROM workspace_members mm
              JOIN role_assignments a ON a.workspace_id = mm.workspace_id AND a.member_id = mm.id
                   AND a.deleted_at IS NULL AND a.scope_kind IS NULL
              JOIN roles r ON r.workspace_id = a.workspace_id AND r.id = a.role_id
                   AND r.deleted_at IS NULL
              JOIN role_permissions p ON p.workspace_id = r.workspace_id AND p.role_id = r.id
                   AND p.permission = 'users.manage'
              WHERE mm.workspace_id = ws AND mm.status = 'active' AND mm.deleted_at IS NULL) x;
        workspace_id := ws;
        mode := m;
        sole_admin := mine > 0 AND others = 0;
        member_id := m_id;
        RETURN NEXT;
    END LOOP;
    PERFORM set_config('app.user_id', coalesce(prev_user, ''), true);
    PERFORM set_config('app.workspace_id', coalesce(prev_ws, ''), true);
END
$$;
REVOKE ALL ON FUNCTION account_workspaces_of(uuid) FROM PUBLIC;

-- Süresi dolan, geri alınmamış, henüz silinmemiş talepler. Başarısız deneme 15 dakika bekler.
CREATE FUNCTION account_deletions_due(p_limit integer DEFAULT 20)
    RETURNS TABLE (user_id uuid)
    LANGUAGE sql STABLE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
    SELECT d.user_id FROM account_deletions d
    WHERE d.purge_at <= now() AND d.restored_at IS NULL AND d.purged_at IS NULL
      AND (d.last_attempt_at IS NULL OR d.last_attempt_at < now() - interval '15 minutes')
    ORDER BY d.purge_at
    LIMIT p_limit
$$;

CREATE FUNCTION account_deletion_failed(p_user_id uuid, p_error text)
    RETURNS void
    LANGUAGE sql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
    UPDATE account_deletions
       SET attempts = attempts + 1, last_attempt_at = now(), last_error = left(p_error, 300)
     WHERE user_id = p_user_id AND purged_at IS NULL
$$;

-- Apple yenileme belirteçleri (zarf şifreli); iptal için worker okur (adr/0009 › 1.5).
CREATE FUNCTION account_apple_tokens(p_user_id uuid)
    RETURNS TABLE (identity_id uuid, token_enc bytea)
    LANGUAGE sql STABLE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
    SELECT i.id, i.provider_refresh_token_enc FROM user_identities i
    WHERE i.user_id = p_user_id AND i.provider = 'apple'
      AND i.provider_refresh_token_enc IS NOT NULL
$$;

CREATE FUNCTION account_apple_token_cleared(p_identity_id uuid)
    RETURNS void
    LANGUAGE sql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
    UPDATE user_identities SET provider_refresh_token_enc = NULL WHERE id = p_identity_id
$$;

-- Silinecek dış kayıtlar: GitHub Issue'ları ve nesne deposu anahtarları. Silme öncesi worker
-- bunları kaldırır. kind: 'home_workspace' (tek yöneticisi olunan Ev çalışma alanı), 'issue',
-- 'object'.
CREATE FUNCTION account_collect(p_user_id uuid)
    RETURNS TABLE (kind text, workspace_id uuid, ref_id uuid, issue_number integer,
                   object_key text)
    LANGUAGE plpgsql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
#variable_conflict use_column
DECLARE
    prev_user text := current_setting('app.user_id', true);
    prev_ws text := current_setting('app.workspace_id', true);
    w record;
BEGIN
    FOR w IN SELECT * FROM account_workspaces_of(p_user_id) LOOP
        PERFORM set_config('app.user_id', p_user_id::text, true);
        PERFORM set_config('app.workspace_id', w.workspace_id::text, true);
        IF w.mode = 'home' AND w.sole_admin THEN
            RETURN QUERY SELECT 'home_workspace'::text, w.workspace_id, w.workspace_id, NULL::integer, NULL::text;
            RETURN QUERY SELECT 'issue'::text, r.workspace_id, r.id, r.github_issue_number, NULL::text
                FROM support_requests r WHERE r.workspace_id = w.workspace_id AND r.github_issue_number IS NOT NULL;
            RETURN QUERY SELECT 'object'::text, a.workspace_id, a.id, NULL::integer, a.object_key
                FROM attachments a WHERE a.workspace_id = w.workspace_id;
            RETURN QUERY SELECT 'object'::text, d.workspace_id, d.id, NULL::integer, d.object_key
                FROM scanned_documents d WHERE d.workspace_id = w.workspace_id;
            RETURN QUERY SELECT 'object'::text, s.workspace_id, s.id, NULL::integer, s.object_key
                FROM import_sources s WHERE s.workspace_id = w.workspace_id;
            RETURN QUERY SELECT 'object'::text, e.workspace_id, e.id, NULL::integer, e.result_object_key
                FROM export_jobs e WHERE e.workspace_id = w.workspace_id AND e.result_object_key IS NOT NULL;
        ELSE
            RETURN QUERY SELECT 'issue'::text, r.workspace_id, r.id, r.github_issue_number, NULL::text
                FROM support_requests r
                WHERE r.workspace_id = w.workspace_id AND r.created_by = p_user_id
                  AND r.github_issue_number IS NOT NULL;
            RETURN QUERY SELECT 'object'::text, a.workspace_id, a.id, NULL::integer, a.object_key
                FROM attachments a
                WHERE a.workspace_id = w.workspace_id AND a.record_kind = 'support_message'
                  AND a.record_id IN (SELECT m.id FROM support_messages m
                      WHERE m.author_user_id = p_user_id
                         OR m.request_id IN (SELECT q.id FROM support_requests q WHERE q.created_by = p_user_id));
            RETURN QUERY SELECT 'object'::text, e.workspace_id, e.id, NULL::integer, e.result_object_key
                FROM export_jobs e
                WHERE e.workspace_id = w.workspace_id AND e.created_by = p_user_id
                  AND e.scope = 'account_data' AND e.result_object_key IS NOT NULL;
        END IF;
    END LOOP;
    PERFORM set_config('app.user_id', coalesce(prev_user, ''), true);
    PERFORM set_config('app.workspace_id', coalesce(prev_ws, ''), true);
END
$$;

-- Kalıcı silme (veritabanı kısmı). Worker önce dış kayıtları (Apple belirteci, Issue, nesne)
-- kaldırır, sonra bunu çağırır. `p_home_workspaces`: worker'ın topladığı liste; fark çıkarsa
-- işlem geri alınır (worker yeniden toplar). Denetim satırlarına dokunmaz.
CREATE FUNCTION account_purge(p_user_id uuid, p_home_workspaces uuid[])
    RETURNS void
    LANGUAGE plpgsql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
#variable_conflict use_column
DECLARE
    prev_user text := current_setting('app.user_id', true);
    prev_ws text := current_setting('app.workspace_id', true);
    d account_deletions%ROWTYPE;
    w record;
    found_home uuid[] := ARRAY[]::uuid[];
BEGIN
    PERFORM set_config('app.user_id', p_user_id::text, true);
    SELECT * INTO d FROM account_deletions WHERE user_id = p_user_id FOR UPDATE;
    IF NOT FOUND OR d.restored_at IS NOT NULL OR d.purged_at IS NOT NULL OR d.purge_at > now() THEN
        RAISE EXCEPTION 'account % has no due deletion request', p_user_id
            USING ERRCODE = 'integrity_constraint_violation';
    END IF;
    FOR w IN SELECT * FROM account_workspaces_of(p_user_id) LOOP
        PERFORM set_config('app.user_id', p_user_id::text, true);
        PERFORM set_config('app.workspace_id', w.workspace_id::text, true);
        IF w.mode = 'home' AND w.sole_admin THEN
            found_home := found_home || w.workspace_id;
            IF NOT (w.workspace_id = ANY (p_home_workspaces)) THEN
                RAISE EXCEPTION 'workspace % was not in the collected list', w.workspace_id
                    USING ERRCODE = 'integrity_constraint_violation';
            END IF;
            UPDATE workspaces SET deleted_at = coalesce(deleted_at, now())
             WHERE id = w.workspace_id;
            PERFORM purge_workspace(w.workspace_id, p_user_id, 'account_deletion');
            PERFORM set_config('app.user_id', p_user_id::text, true);
            CONTINUE;
        END IF;
        -- Kalan çalışma alanı: kullanıcının destek verisi gider, üyelik kapanır, UUID kalır.
        DELETE FROM attachments WHERE record_kind = 'support_message' AND record_id IN (
            SELECT m.id FROM support_messages m
            WHERE m.author_user_id = p_user_id
               OR m.request_id IN (SELECT q.id FROM support_requests q WHERE q.created_by = p_user_id));
        DELETE FROM support_messages
         WHERE author_user_id = p_user_id
            OR request_id IN (SELECT q.id FROM support_requests q WHERE q.created_by = p_user_id);
        DELETE FROM support_requests WHERE created_by = p_user_id;
        DELETE FROM export_jobs WHERE created_by = p_user_id AND scope = 'account_data';
        DELETE FROM notifications WHERE recipient_member_id = w.member_id;
        DELETE FROM reminders WHERE recipient_member_id = w.member_id;
        DELETE FROM member_pins WHERE member_id = w.member_id;
        DELETE FROM device_sessions WHERE member_id = w.member_id;
        DELETE FROM favorites WHERE user_id = p_user_id;
        DELETE FROM user_settings WHERE user_id = p_user_id;
        DELETE FROM notification_preferences WHERE user_id = p_user_id;
        DELETE FROM edit_sessions WHERE user_id = p_user_id;
        UPDATE role_assignments SET deleted_at = now()
         WHERE member_id = w.member_id AND deleted_at IS NULL;
        UPDATE workspace_members SET status = 'disabled', deleted_at = now()
         WHERE id = w.member_id AND deleted_at IS NULL;
        UPDATE workspaces SET authz_version = authz_version + 1 WHERE id = w.workspace_id;
    END LOOP;
    IF EXISTS (SELECT 1 FROM unnest(p_home_workspaces) h WHERE NOT (h = ANY (found_home))) THEN
        RAISE EXCEPTION 'a collected workspace is no longer a sole-admin home workspace'
            USING ERRCODE = 'integrity_constraint_violation';
    END IF;

    PERFORM set_config('app.workspace_id', '', true);
    PERFORM set_config('app.user_id', p_user_id::text, true);
    DELETE FROM passkeys WHERE user_id = p_user_id;
    DELETE FROM sessions WHERE user_id = p_user_id;
    DELETE FROM push_targets WHERE user_id = p_user_id;
    DELETE FROM webauthn_challenges WHERE user_id = p_user_id;
    DELETE FROM idempotency_keys WHERE user_id = p_user_id;
    DELETE FROM user_identities WHERE user_id = p_user_id;
    UPDATE users SET display_name = '', contact_email = NULL, contact_email_verified_at = NULL,
                     locale = 'en-US', deleted_at = coalesce(deleted_at, now())
     WHERE id = p_user_id;
    UPDATE account_deletions SET purged_at = now() WHERE user_id = p_user_id;
    PERFORM set_config('app.user_id', coalesce(prev_user, ''), true);
    PERFORM set_config('app.workspace_id', coalesce(prev_ws, ''), true);
END
$$;

-- Hesap verisi dışa aktarma (worker): kullanıcının üye olduğu çalışma alanları.
CREATE FUNCTION account_export_workspaces(p_user_id uuid)
    RETURNS TABLE (workspace_id uuid, mode text, sole_admin boolean, member_id uuid)
    LANGUAGE sql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$ SELECT * FROM account_workspaces_of(p_user_id) $$;

-- Hesap profili: kimlik sağlayıcıları (belirteç yok), passkey adları, oturumlar, push adresleri.
CREATE FUNCTION account_export_profile(p_user_id uuid)
    RETURNS jsonb
    LANGUAGE plpgsql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
DECLARE
    prev_user text := current_setting('app.user_id', true);
    doc jsonb;
BEGIN
    PERFORM set_config('app.user_id', p_user_id::text, true);
    SELECT jsonb_build_object(
        'user', (SELECT to_jsonb(u) - ARRAY['deleted_at'] FROM users u WHERE u.id = p_user_id),
        'identities', (SELECT coalesce(jsonb_agg(jsonb_build_object(
                'provider', i.provider, 'subject', i.subject, 'linked_at', i.linked_at)
                ORDER BY i.linked_at), '[]'::jsonb) FROM user_identities i WHERE i.user_id = p_user_id),
        'passkeys', (SELECT coalesce(jsonb_agg(jsonb_build_object(
                'name', k.name, 'created_at', k.created_at, 'last_used_at', k.last_used_at)
                ORDER BY k.created_at), '[]'::jsonb) FROM passkeys k WHERE k.user_id = p_user_id),
        'sessions', (SELECT coalesce(jsonb_agg(jsonb_build_object(
                'client_kind', s.client_kind, 'device_name', s.device_name,
                'approx_location', s.approx_location, 'created_at', s.created_at,
                'last_activity_at', s.last_activity_at, 'revoked_at', s.revoked_at)
                ORDER BY s.created_at), '[]'::jsonb) FROM sessions s WHERE s.user_id = p_user_id),
        'push_targets', (SELECT coalesce(jsonb_agg(jsonb_build_object(
                'environment', t.environment, 'app_version', t.app_version,
                'created_at', t.created_at) ORDER BY t.created_at), '[]'::jsonb)
                FROM push_targets t WHERE t.user_id = p_user_id))
    INTO doc;
    PERFORM set_config('app.user_id', coalesce(prev_user, ''), true);
    RETURN doc;
END
$$;

REVOKE ALL ON FUNCTION
    account_deletions_due(integer), account_deletion_failed(uuid, text),
    account_apple_tokens(uuid), account_apple_token_cleared(uuid), account_collect(uuid),
    account_purge(uuid, uuid[]), account_export_workspaces(uuid), account_export_profile(uuid)
    FROM PUBLIC;
GRANT EXECUTE ON FUNCTION
    account_deletions_due(integer), account_deletion_failed(uuid, text),
    account_apple_tokens(uuid), account_apple_token_cleared(uuid), account_collect(uuid),
    account_purge(uuid, uuid[]), account_export_workspaces(uuid), account_export_profile(uuid)
    TO app_worker;

RESET ROLE;
