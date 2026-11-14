-- T2.9 (F2.1 bulgu 5, 6, 8, 40): kalıcı çalışma alanı silme yalnız worker'da, dar işlevle;
-- silme sonrası çalışma alanı dışında kalan küçük kayıt. Uygulanmış göçler değişmez; bu göç
-- yenidir (adr/0007 › T2.9 düzeltmeleri).
SET LOCAL ROLE app_owner;

-- workspace_tombstones --------------------------------------------------------------------
-- Silinen çalışma alanının kaydı: kim sildi, ne zaman, denetim zincirinin son sırası ve özeti,
-- silinen destek erişimi sayısı, abonelik dış kimlikleri. Çalışma alanı dışındadır (yabancı
-- anahtarı ve RLS'si yoktur, `deleted_workspace_id` adı bilerek `workspace_id` değildir), yalnız
-- eklenir: yalnız `purge_workspace` yazar, uygulama rollerinde INSERT hakkı yoktur. Kişisel
-- veri taşımaz (kullanıcı yalnız UUID ile anılır, destek takma adı kimliktir).
CREATE TABLE workspace_tombstones (
    deleted_workspace_id  uuid PRIMARY KEY,
    deleted_at            timestamptz NOT NULL DEFAULT now(),
    soft_deleted_at       timestamptz NOT NULL,
    soft_deleted_by       uuid,
    mode                  text NOT NULL CHECK (mode IN ('home', 'business')),
    region                text NOT NULL CHECK (region IN ('us', 'uk')),
    support_pseudonym     text NOT NULL,
    last_audit_seq        bigint NOT NULL CHECK (last_audit_seq >= 0),
    last_audit_hash       bytea NOT NULL CHECK (octet_length(last_audit_hash) = 32),
    support_action_count  bigint NOT NULL DEFAULT 0 CHECK (support_action_count >= 0),
    last_support_action_at timestamptz,
    -- Abonelik dış kimlikleri: sonraki App Store bildirimi "çalışma alanı silindi" diye eşlenir.
    external_refs         jsonb NOT NULL DEFAULT '[]' CHECK (jsonb_typeof(external_refs) = 'array')
);

CREATE FUNCTION workspace_tombstones_reject_change() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    RAISE EXCEPTION 'workspace_tombstones is append-only (% rejected)', TG_OP
        USING ERRCODE = 'integrity_constraint_violation';
END
$$;
CREATE TRIGGER no_update_delete BEFORE UPDATE OR DELETE ON workspace_tombstones
    FOR EACH ROW EXECUTE FUNCTION workspace_tombstones_reject_change();
CREATE TRIGGER no_truncate BEFORE TRUNCATE ON workspace_tombstones
    FOR EACH STATEMENT EXECUTE FUNCTION workspace_tombstones_reject_change();
-- Okuma yalnız worker'da (destek aracı "kim sildi" sorusuna buradan yanıt verir).
GRANT SELECT ON workspace_tombstones TO app_worker;

-- Silmeyi engelleyen saklama (0083 işletme alanlarını ekler; burada yalnız tanım) -------------
-- Saklama bitişi gelecekteyse ya da yasal tutma süresiz (bitişsiz) ise kayıt silinemez.
CREATE FUNCTION retention_blocks(p_legal_hold boolean, p_retention_until timestamptz)
    RETURNS boolean
    LANGUAGE sql STABLE PARALLEL SAFE
    AS $$
    SELECT (p_legal_hold AND p_retention_until IS NULL)
        OR coalesce(p_retention_until > now(), false)
$$;

-- Bir çalışma alanında silmeyi engelleyen en geç saklama bitişi: NULL = engel yok,
-- 'infinity' = süresiz yasal tutma. Tablo listesi 0083'te `retention_tables` kaydından
-- gelir; bu işlev orada yenilenir (CREATE OR REPLACE).
CREATE FUNCTION workspace_retention_blocker(p_workspace_id uuid) RETURNS timestamptz
    LANGUAGE plpgsql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
DECLARE
    prev text := current_setting('app.workspace_id', true);
    blocker timestamptz;
BEGIN
    PERFORM set_config('app.workspace_id', p_workspace_id::text, true);
    SELECT max(CASE WHEN legal_hold AND retention_until IS NULL THEN 'infinity'::timestamptz
                    ELSE retention_until END)
    INTO blocker
    FROM (
        SELECT legal_hold, retention_until FROM scanned_documents
        WHERE workspace_id = p_workspace_id
    ) t
    WHERE retention_blocks(legal_hold, retention_until);
    PERFORM set_config('app.workspace_id', coalesce(prev, ''), true);
    RETURN blocker;
END
$$;

-- Listeleme işlevleri sahip rolle çalışır; FORCE RLS sahibi de bağladığı için yalnız okuma
-- yönünde sahibe açık politika gerekir (yazma ve silme bağlam altında kalır).
CREATE POLICY owner_lookup ON workspaces FOR SELECT TO app_owner USING (true);

-- Worker'ın çalışma alanlarını sayması (bağlamsız; RLS yalnız kendi bağlamını gösterir) -------
-- Etkin çalışma alanları: günlük stok yeniden hesabı, bölüm açma, saklama temizliği gibi
-- çalışma alanı başına işler için.
CREATE FUNCTION list_active_workspaces()
    RETURNS TABLE (workspace_id uuid, mode text, region text)
    LANGUAGE sql STABLE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
    SELECT w.id, w.mode, w.region FROM workspaces w WHERE w.deleted_at IS NULL
$$;

-- Yumuşak silinmiş ve bekleme süresi dolmuş çalışma alanları; `blocked_until` doluysa
-- saklama engeli vardır ('infinity' = süresiz tutma): worker o zamana dek (ya da hiç) atlar.
CREATE FUNCTION list_workspaces_pending_purge(p_soft_deleted_before timestamptz)
    RETURNS TABLE (workspace_id uuid, soft_deleted_at timestamptz, blocked_until timestamptz)
    LANGUAGE sql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
    SELECT w.id, w.deleted_at, workspace_retention_blocker(w.id)
    FROM workspaces w
    WHERE w.deleted_at IS NOT NULL AND w.deleted_at < p_soft_deleted_before
    ORDER BY w.deleted_at
$$;

-- Kalıcı çalışma alanı silme -----------------------------------------------------------------
-- Yalnız worker çağırır (app_api'nin workspaces ve users üzerinde DELETE hakkı kalkar). Önce
-- yumuşak silinmiş olmalı (DM001), saklama engeli yoksa (DM002) silme kaydını yazar ve
-- çalışma alanını siler; tablolar yabancı anahtar zinciriyle gider (denetim zinciri dahil,
-- adr/0007 › 6.3). Bekleyen işleri siler, nesne deposu dosyaları için `purge_workspace_objects`
-- işi ekler (anahtarlar `{workspace_id}/` önekiyle başlar). Yoksa (zaten silinmiş) false döner.
CREATE FUNCTION purge_workspace(p_workspace_id uuid) RETURNS boolean
    LANGUAGE plpgsql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
DECLARE
    prev text := current_setting('app.workspace_id', true);
    ws workspaces%ROWTYPE;
    blocker timestamptz;
    head_seq bigint;
    head_hash bytea;
    actions bigint;
    last_action timestamptz;
    refs jsonb;
BEGIN
    PERFORM set_config('app.workspace_id', p_workspace_id::text, true);
    SELECT * INTO ws FROM workspaces WHERE id = p_workspace_id FOR UPDATE;
    IF NOT FOUND THEN
        PERFORM set_config('app.workspace_id', coalesce(prev, ''), true);
        RETURN false;
    END IF;
    IF ws.deleted_at IS NULL THEN
        RAISE EXCEPTION 'workspace % is not soft-deleted', p_workspace_id USING ERRCODE = 'DM001';
    END IF;
    blocker := workspace_retention_blocker(p_workspace_id);
    IF blocker IS NOT NULL THEN
        RAISE EXCEPTION 'workspace % holds records under retention until %', p_workspace_id, blocker
            USING ERRCODE = 'DM002';
    END IF;

    SELECT last_seq, last_hash INTO head_seq, head_hash
    FROM audit_heads WHERE workspace_id = p_workspace_id;
    SELECT count(*), max(created_at) INTO actions, last_action
    FROM support_actions WHERE workspace_id = p_workspace_id;
    SELECT coalesce(jsonb_agg(jsonb_build_object(
               'source', source, 'external_id', external_id, 'product_key', product_key)), '[]')
    INTO refs FROM subscriptions WHERE workspace_id = p_workspace_id;

    INSERT INTO workspace_tombstones (
        deleted_workspace_id, soft_deleted_at, soft_deleted_by, mode, region, support_pseudonym,
        last_audit_seq, last_audit_hash, support_action_count, last_support_action_at, external_refs)
    VALUES (
        ws.id, ws.deleted_at, ws.updated_by, ws.mode, ws.region, ws.support_pseudonym,
        coalesce(head_seq, 0), coalesce(head_hash, decode(repeat('00', 32), 'hex')),
        actions, last_action, refs);

    DELETE FROM scheduled_jobs WHERE workspace_id = p_workspace_id;
    DELETE FROM workspaces WHERE id = p_workspace_id;
    INSERT INTO scheduled_jobs (id, workspace_id, kind, run_at, payload)
    VALUES (gen_random_uuid(), NULL, 'purge_workspace_objects', now(),
            jsonb_build_object('schema_version', 1, 'workspace_id', p_workspace_id,
                               'prefix', p_workspace_id::text || '/'));
    PERFORM set_config('app.workspace_id', coalesce(prev, ''), true);
    RETURN true;
END
$$;

REVOKE ALL ON FUNCTION
    workspace_retention_blocker(uuid),
    list_active_workspaces(),
    list_workspaces_pending_purge(timestamptz),
    purge_workspace(uuid)
    FROM PUBLIC;
GRANT EXECUTE ON FUNCTION
    list_active_workspaces(),
    list_workspaces_pending_purge(timestamptz),
    purge_workspace(uuid)
    TO app_worker;
REVOKE ALL ON FUNCTION retention_blocks(boolean, timestamptz) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION retention_blocks(boolean, timestamptz) TO app_api, app_worker;

-- Silme hakkı: API hiçbir çalışma alanını ya da kullanıcıyı silemez (yumuşak siler; hesap
-- silme `users` satırını mezar taşına çevirir, adr/0007 › 6.2).
REVOKE DELETE ON workspaces FROM app_api;
REVOKE DELETE ON users FROM app_api;

RESET ROLE;
