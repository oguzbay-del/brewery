-- M2 incelemesi: OAuth işlevlerinde çağıran denetimi, üyelik ve çalışma alanı kapanınca
-- onayların kapanması (adr/0015 › 2.7), kod tekrarında iptalin denetime yazılması için
-- kullanıcı ve çalışma alanı, bakımda kullanılmayan CIMD kayıtlarının silinmesi.
-- 0257 değiştirilmez; işlevler CREATE OR REPLACE ile yenilenir.
SET LOCAL ROLE app_owner;

-- Çağıran, çalışma alanının yöneticisi mi (workspace.manage, kapsamsız atama)? Bağlamdaki
-- kullanıcı ve çalışma alanına bakar; çağıranın verdiği kimliğe güvenmez.
CREATE FUNCTION oauth_is_workspace_admin(p_workspace uuid)
    RETURNS boolean
    LANGUAGE sql STABLE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
    SELECT p_workspace IS NOT DISTINCT FROM app_workspace_id()
       AND p_workspace IS NOT NULL
       AND EXISTS (
        SELECT 1
        FROM workspace_members m
        JOIN role_assignments a ON a.workspace_id = m.workspace_id AND a.member_id = m.id
             AND a.deleted_at IS NULL AND a.scope_kind IS NULL
        JOIN roles r ON r.workspace_id = a.workspace_id AND r.id = a.role_id
             AND r.deleted_at IS NULL
        JOIN role_permissions p ON p.workspace_id = r.workspace_id AND p.role_id = r.id
             AND p.permission = 'workspace.manage'
        WHERE m.workspace_id = p_workspace AND m.user_id = app_user_id()
          AND m.status = 'active' AND m.deleted_at IS NULL)
$$;
REVOKE ALL ON FUNCTION oauth_is_workspace_admin(uuid) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION oauth_is_workspace_admin(uuid) TO app_api;

-- Onay yalnız bağlamdaki kullanıcı ve onun üyesi olduğu çalışma alanı için açılır.
CREATE OR REPLACE FUNCTION oauth_grant_upsert(
    p_id uuid, p_user uuid, p_workspace uuid, p_client uuid, p_scopes text[], p_resource text)
    RETURNS TABLE (grant_id uuid, scopes text[], widened boolean)
    LANGUAGE plpgsql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
DECLARE
    existing oauth_grants%ROWTYPE;
    merged text[];
BEGIN
    IF p_user IS DISTINCT FROM app_user_id() THEN
        RAISE EXCEPTION 'oauth grant for another user' USING ERRCODE = 'insufficient_privilege';
    END IF;
    IF NOT EXISTS (SELECT 1 FROM workspace_members m
                   WHERE m.workspace_id = p_workspace AND m.user_id = p_user
                     AND m.status = 'active' AND m.deleted_at IS NULL) THEN
        RAISE EXCEPTION 'not a member of the workspace' USING ERRCODE = 'insufficient_privilege';
    END IF;
    SELECT * INTO existing FROM oauth_grants g
    WHERE g.user_id = p_user AND g.workspace_id = p_workspace AND g.client_id = p_client
      AND g.resource = p_resource AND g.revoked_at IS NULL
    FOR UPDATE;
    IF FOUND THEN
        SELECT array_agg(DISTINCT s ORDER BY s) INTO merged
        FROM unnest(existing.scopes || p_scopes) s;
        UPDATE oauth_grants g SET scopes = merged, updated_at = now() WHERE g.id = existing.id;
        UPDATE oauth_clients c SET last_consented_at = now() WHERE c.id = p_client;
        RETURN QUERY SELECT existing.id, merged, merged <> existing.scopes;
    ELSE
        SELECT array_agg(DISTINCT s ORDER BY s) INTO merged FROM unnest(p_scopes) s;
        INSERT INTO oauth_grants (id, user_id, workspace_id, client_id, scopes, resource)
        VALUES (p_id, p_user, p_workspace, p_client, merged, p_resource);
        UPDATE oauth_clients c SET last_consented_at = now() WHERE c.id = p_client;
        RETURN QUERY SELECT p_id, merged, false;
    END IF;
END
$$;

CREATE OR REPLACE FUNCTION oauth_grant_revoke(p_grant uuid, p_user uuid, p_reason text)
    RETURNS TABLE (grant_id uuid, workspace_id uuid, client_pk uuid)
    LANGUAGE sql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
    WITH g AS (
        UPDATE oauth_grants x SET revoked_at = now(), revoked_reason = p_reason
        WHERE x.id = p_grant AND x.user_id = p_user AND x.user_id = app_user_id()
          AND x.revoked_at IS NULL
        RETURNING x.id, x.workspace_id, x.client_id
    ), t AS (
        UPDATE oauth_tokens x SET revoked_at = now()
        WHERE x.grant_id IN (SELECT id FROM g) AND x.revoked_at IS NULL
        RETURNING 1
    )
    SELECT g.id, g.workspace_id, g.client_id FROM g
$$;

CREATE OR REPLACE FUNCTION oauth_grant_revoke_all_user(p_user uuid, p_reason text)
    RETURNS TABLE (grant_id uuid, workspace_id uuid, client_pk uuid)
    LANGUAGE sql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
    WITH g AS (
        UPDATE oauth_grants x SET revoked_at = now(), revoked_reason = p_reason
        WHERE x.user_id = p_user AND x.user_id = app_user_id() AND x.revoked_at IS NULL
        RETURNING x.id, x.workspace_id, x.client_id
    ), t AS (
        UPDATE oauth_tokens x SET revoked_at = now()
        WHERE x.grant_id IN (SELECT id FROM g) AND x.revoked_at IS NULL
        RETURNING 1
    )
    SELECT g.id, g.workspace_id, g.client_id FROM g
$$;

CREATE OR REPLACE FUNCTION oauth_workspace_revoke_all(
    p_workspace uuid, p_resource text, p_reason text)
    RETURNS TABLE (grant_id uuid, user_id uuid, client_pk uuid)
    LANGUAGE sql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
    WITH g AS (
        UPDATE oauth_grants x SET revoked_at = now(), revoked_reason = p_reason
        WHERE x.workspace_id = p_workspace AND x.resource = p_resource
          AND x.revoked_at IS NULL AND oauth_is_workspace_admin(p_workspace)
        RETURNING x.id, x.user_id, x.client_id
    ), t AS (
        UPDATE oauth_tokens x SET revoked_at = now()
        WHERE x.grant_id IN (SELECT id FROM g) AND x.revoked_at IS NULL
        RETURNING 1
    )
    SELECT g.id, g.user_id, g.client_id FROM g
$$;

-- Kod tüketimi artık onayın kullanıcısını ve çalışma alanını da döndürür (tekrar iptali
-- denetime yazılır). Dönüş türü değiştiği için işlev yeniden oluşturulur.
DROP FUNCTION oauth_code_consume(bytea);
CREATE FUNCTION oauth_code_consume(p_hash bytea)
    RETURNS TABLE (grant_id uuid, client_pk uuid, redirect_uri text, code_challenge text,
                   scopes text[], resource text, replayed boolean, user_id uuid,
                   workspace_id uuid)
    LANGUAGE plpgsql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
DECLARE
    c oauth_codes%ROWTYPE;
    g oauth_grants%ROWTYPE;
BEGIN
    SELECT * INTO c FROM oauth_codes x WHERE x.code_hash = p_hash FOR UPDATE;
    IF NOT FOUND THEN
        RETURN;
    END IF;
    SELECT * INTO g FROM oauth_grants x WHERE x.id = c.grant_id;
    IF c.consumed_at IS NOT NULL THEN
        UPDATE oauth_grants x SET revoked_at = now(), revoked_reason = 'code_replay'
        WHERE x.id = c.grant_id AND x.revoked_at IS NULL;
        UPDATE oauth_tokens t SET revoked_at = now()
        WHERE t.grant_id = c.grant_id AND t.revoked_at IS NULL;
        RETURN QUERY SELECT c.grant_id, c.client_id, c.redirect_uri, c.code_challenge,
                            c.scopes, c.resource, true, g.user_id, g.workspace_id;
        RETURN;
    END IF;
    IF c.expires_at <= now() THEN
        RETURN;
    END IF;
    UPDATE oauth_codes x SET consumed_at = now() WHERE x.id = c.id;
    RETURN QUERY SELECT c.grant_id, c.client_id, c.redirect_uri, c.code_challenge,
                        c.scopes, c.resource, false, g.user_id, g.workspace_id;
END
$$;
REVOKE ALL ON FUNCTION oauth_code_consume(bytea) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION oauth_code_consume(bytea) TO app_api;

-- Bakım: kullanılmayan CIMD kayıtları da silinir (önbellek süresi bitmiş, onayı yok).
CREATE OR REPLACE FUNCTION oauth_purge_expired(p_older_than interval DEFAULT interval '1 day')
    RETURNS bigint
    LANGUAGE plpgsql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
DECLARE
    n bigint := 0;
    m bigint;
BEGIN
    DELETE FROM oauth_requests WHERE expires_at < now() - p_older_than;
    GET DIAGNOSTICS m = ROW_COUNT; n := n + m;
    DELETE FROM oauth_codes WHERE expires_at < now() - p_older_than;
    GET DIAGNOSTICS m = ROW_COUNT; n := n + m;
    DELETE FROM oauth_tokens WHERE expires_at < now() - interval '90 days';
    GET DIAGNOSTICS m = ROW_COUNT; n := n + m;
    DELETE FROM oauth_clients c
    WHERE c.kind = 'dcr' AND c.last_consented_at IS NULL
      AND c.created_at < now() - interval '30 days'
      AND NOT EXISTS (SELECT 1 FROM oauth_grants g WHERE g.client_id = c.id);
    GET DIAGNOSTICS m = ROW_COUNT; n := n + m;
    DELETE FROM oauth_clients c
    WHERE c.kind = 'cimd' AND c.metadata_expires_at < now() - p_older_than
      AND NOT EXISTS (SELECT 1 FROM oauth_grants g WHERE g.client_id = c.id);
    GET DIAGNOSTICS m = ROW_COUNT; n := n + m;
    RETURN n;
END
$$;

-- Üyelik kapanınca o kullanıcının o çalışma alanındaki onayları kapanır; üye yeniden
-- eklenirse eski belirteç geri gelmez (adr/0015 › 2.7).
CREATE FUNCTION oauth_close_member_grants() RETURNS trigger
    LANGUAGE plpgsql SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
BEGIN
    IF TG_OP = 'DELETE'
       OR (OLD.status = 'active' AND OLD.deleted_at IS NULL
           AND (NEW.status <> 'active' OR NEW.deleted_at IS NOT NULL)) THEN
        IF OLD.user_id IS NOT NULL THEN
            UPDATE oauth_grants g SET revoked_at = now(), revoked_reason = 'membership_closed'
            WHERE g.user_id = OLD.user_id AND g.workspace_id = OLD.workspace_id
              AND g.revoked_at IS NULL;
            UPDATE oauth_tokens t SET revoked_at = now()
            WHERE t.revoked_at IS NULL AND t.grant_id IN (
                SELECT g.id FROM oauth_grants g
                WHERE g.user_id = OLD.user_id AND g.workspace_id = OLD.workspace_id);
        END IF;
    END IF;
    IF TG_OP = 'DELETE' THEN
        RETURN OLD;
    END IF;
    RETURN NEW;
END
$$;
REVOKE ALL ON FUNCTION oauth_close_member_grants() FROM PUBLIC;
CREATE TRIGGER oauth_close_grants AFTER UPDATE OR DELETE ON workspace_members
    FOR EACH ROW EXECUTE FUNCTION oauth_close_member_grants();

-- Çalışma alanı silinince (yumuşak silme) bütün onayları kapanır.
CREATE FUNCTION oauth_close_workspace_grants() RETURNS trigger
    LANGUAGE plpgsql SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
BEGIN
    IF OLD.deleted_at IS NULL AND NEW.deleted_at IS NOT NULL THEN
        UPDATE oauth_grants g SET revoked_at = now(), revoked_reason = 'workspace_deleted'
        WHERE g.workspace_id = NEW.id AND g.revoked_at IS NULL;
        UPDATE oauth_tokens t SET revoked_at = now()
        WHERE t.revoked_at IS NULL AND t.grant_id IN (
            SELECT g.id FROM oauth_grants g WHERE g.workspace_id = NEW.id);
    END IF;
    RETURN NEW;
END
$$;
REVOKE ALL ON FUNCTION oauth_close_workspace_grants() FROM PUBLIC;
CREATE TRIGGER oauth_close_grants AFTER UPDATE OF deleted_at ON workspaces
    FOR EACH ROW EXECUTE FUNCTION oauth_close_workspace_grants();

RESET ROLE;
