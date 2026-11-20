-- T2.39 (F2.1 karar 12): worker günlük olarak bütün denetim zincirlerinin son özetini ve silme
-- kayıtlarını nesne deposuna yazar. Çalışma alanları arası okuma için iki dar SECURITY DEFINER
-- işlev: yalnız kimlik, sıra ve özet döner (müşteri içeriği yok). app_owner'a yalnız okuma
-- politikası verilir; yazma hakkı yoktur.
SET LOCAL ROLE app_owner;

CREATE POLICY owner_read ON audit_heads FOR SELECT TO app_owner USING (true);

CREATE FUNCTION list_audit_heads()
    RETURNS TABLE (workspace_id uuid, last_seq bigint, last_hash bytea)
    LANGUAGE sql STABLE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
    SELECT h.workspace_id, h.last_seq, h.last_hash FROM audit_heads h ORDER BY h.workspace_id
$$;

CREATE FUNCTION list_workspace_tombstones()
    RETURNS TABLE (former_workspace_id uuid, deleted_at timestamptz,
                   audit_last_seq bigint, audit_last_hash bytea)
    LANGUAGE sql STABLE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
    SELECT t.former_workspace_id, t.deleted_at, t.audit_last_seq, t.audit_last_hash
    FROM workspace_tombstones t ORDER BY t.former_workspace_id
$$;

REVOKE ALL ON FUNCTION list_audit_heads(), list_workspace_tombstones() FROM PUBLIC;
GRANT EXECUTE ON FUNCTION list_audit_heads(), list_workspace_tombstones() TO app_worker;

RESET ROLE;
