-- T2.55 (F2.2 bulgu G7): support_find_workspace personel kontrolünü kendisi yapar.
-- İstek bağlamı `app.actor_kind = 'staff'` ve `app.staff_id` (etkin personel) taşımıyorsa işlev
-- satır döndürmez (hata yok). Ad, dönüş sütunları ve yetkiler 0184 ile aynıdır.
SET LOCAL ROLE app_owner;

CREATE OR REPLACE FUNCTION support_find_workspace(p_query text)
    RETURNS TABLE (workspace_id uuid, name text, mode text, region text, pseudonym text,
                   created_at timestamptz)
    LANGUAGE sql STABLE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
    SELECT w.id, w.name, w.mode, w.region, w.support_pseudonym, w.created_at
    FROM workspaces w
    WHERE current_setting('app.actor_kind', true) = 'staff'
      AND EXISTS (SELECT 1 FROM staff_users s
                  WHERE s.id = NULLIF(current_setting('app.staff_id', true), '')::uuid
                    AND s.status = 'active' AND s.deleted_at IS NULL)
      AND w.deleted_at IS NULL
      AND (w.support_pseudonym = p_query
           OR w.id::text = p_query
           OR w.id IN (SELECT r.workspace_id FROM support_requests r
                       WHERE r.public_number::text = ltrim(p_query, '#')))
    ORDER BY w.created_at
    LIMIT 20
$$;
REVOKE ALL ON FUNCTION support_find_workspace(text) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION support_find_workspace(text) TO app_api;

RESET ROLE;
