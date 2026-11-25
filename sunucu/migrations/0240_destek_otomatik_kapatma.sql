-- T2.43: close resolved support requests after 14 days and expose only their sync metadata.
SET LOCAL ROLE app_owner;

-- app_worker has no table grant; the definer changes only status and returns sync metadata.
CREATE FUNCTION close_resolved_support_requests(p_cutoff timestamptz)
    RETURNS TABLE (workspace_id uuid, id uuid, version bigint)
    LANGUAGE plpgsql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
DECLARE
    target_workspace uuid;
    previous_workspace text := current_setting('app.workspace_id', true);
BEGIN
    -- support_requests has forced workspace RLS; set one workspace at a time under the definer.
    FOR target_workspace IN SELECT id FROM list_workspace_ids()
    LOOP
        PERFORM set_config('app.workspace_id', target_workspace::text, true);
        RETURN QUERY
        UPDATE support_requests AS r
           SET status = 'closed',
               updated_by = NULL
         WHERE r.workspace_id = target_workspace
           AND r.status = 'resolved'
           AND r.deleted_at IS NULL
           AND r.resolved_at < p_cutoff
        RETURNING r.workspace_id, r.id, r.version;
    END LOOP;
    PERFORM set_config('app.workspace_id', coalesce(previous_workspace, ''), true);
END
$$;

REVOKE ALL ON FUNCTION close_resolved_support_requests(timestamptz) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION close_resolved_support_requests(timestamptz) TO app_worker;

RESET ROLE;
