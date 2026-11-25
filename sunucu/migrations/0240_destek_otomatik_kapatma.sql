-- T2.43: close resolved support requests after 14 days.
SET LOCAL ROLE app_owner;

-- Keep maintenance access narrow: expose only changed request identifiers and their new version.
CREATE FUNCTION close_resolved_support_requests(p_cutoff timestamptz)
    RETURNS TABLE (workspace_id uuid, request_id uuid, version bigint)
    LANGUAGE sql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
    UPDATE public.support_requests AS r
       SET status = 'closed',
           updated_by = NULL
     WHERE r.status = 'resolved'
       AND r.deleted_at IS NULL
       AND r.resolved_at < p_cutoff
    RETURNING r.workspace_id, r.id, r.version
$$;

REVOKE ALL ON FUNCTION close_resolved_support_requests(timestamptz) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION close_resolved_support_requests(timestamptz) TO app_worker;

RESET ROLE;
