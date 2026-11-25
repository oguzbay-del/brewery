-- T2.43: close resolved support requests after 14 days and expose only their sync metadata.
SET LOCAL ROLE app_owner;

-- app_worker cannot read support_requests; this narrow function performs the cross-workspace
-- status transition and returns only the identifiers/version needed to enqueue the sync event.
CREATE FUNCTION close_resolved_support_requests(p_cutoff timestamptz)
    RETURNS TABLE (workspace_id uuid, id uuid, version bigint)
    LANGUAGE plpgsql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
BEGIN
    RETURN QUERY
    UPDATE support_requests AS r
       SET status = 'closed',
           updated_by = NULL
     WHERE r.status = 'resolved'
       AND r.deleted_at IS NULL
       AND r.resolved_at < p_cutoff
    RETURNING r.workspace_id, r.id, r.version;
END
$$;

REVOKE ALL ON FUNCTION close_resolved_support_requests(timestamptz) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION close_resolved_support_requests(timestamptz) TO app_worker;

RESET ROLE;
