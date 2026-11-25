-- T2.44: track purged upload bytes and expose workspace-scoped cleanup operations.
SET LOCAL ROLE app_owner;

ALTER TABLE attachments ADD COLUMN object_purged_at timestamptz;
ALTER TABLE scanned_documents ADD COLUMN object_purged_at timestamptz;

CREATE INDEX attachments_purgeable_idx
    ON attachments (workspace_id, created_at, id) WHERE object_purged_at IS NULL;
CREATE INDEX scanned_documents_purgeable_idx
    ON scanned_documents (workspace_id, created_at, id) WHERE object_purged_at IS NULL;

-- The caller sets app.workspace_id and remains under forced workspace RLS.
CREATE FUNCTION list_purgeable_uploads(
    p_incomplete_before timestamptz,
    p_limit integer
)
RETURNS TABLE (kind text, id uuid, object_key text)
LANGUAGE sql STABLE
SECURITY INVOKER
SET search_path = pg_catalog, public, pg_temp
AS $$
    SELECT candidates.kind, candidates.id, candidates.object_key
      FROM (
          SELECT 'attachment'::text AS kind, a.id, a.object_key, a.created_at
            FROM public.attachments AS a
           WHERE a.object_purged_at IS NULL
             AND (a.deleted_at IS NOT NULL
                  OR (a.uploaded_at IS NULL AND a.created_at < p_incomplete_before))
          UNION ALL
          SELECT 'scanned_document'::text AS kind, d.id, d.object_key, d.created_at
            FROM public.scanned_documents AS d
           WHERE d.object_purged_at IS NULL
             AND (d.deleted_at IS NOT NULL
                  OR (d.uploaded_at IS NULL AND d.created_at < p_incomplete_before))
             AND (NOT d.legal_hold
                  OR (d.retention_until IS NOT NULL AND d.retention_until <= now()))
      ) AS candidates
     ORDER BY candidates.created_at, candidates.id
     LIMIT LEAST(GREATEST(0, p_limit), 200)
$$;

CREATE FUNCTION mark_upload_purged(p_kind text, p_id uuid)
RETURNS boolean
LANGUAGE plpgsql VOLATILE
SECURITY INVOKER
SET search_path = pg_catalog, public, pg_temp
AS $$
BEGIN
    IF p_kind = 'attachment' THEN
        UPDATE public.attachments
           SET object_purged_at = now(),
               deleted_at = CASE WHEN uploaded_at IS NULL
                                 THEN COALESCE(deleted_at, now())
                                 ELSE deleted_at END
         WHERE id = p_id AND object_purged_at IS NULL;
    ELSIF p_kind = 'scanned_document' THEN
        UPDATE public.scanned_documents
           SET object_purged_at = now(),
               deleted_at = CASE WHEN uploaded_at IS NULL
                                 THEN COALESCE(deleted_at, now())
                                 ELSE deleted_at END
         WHERE id = p_id
           AND object_purged_at IS NULL
           AND (NOT legal_hold
                OR (retention_until IS NOT NULL AND retention_until <= now()));
    ELSE
        RETURN false;
    END IF;
    RETURN FOUND;
END
$$;

REVOKE ALL ON FUNCTION list_purgeable_uploads(timestamptz, integer) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION list_purgeable_uploads(timestamptz, integer) TO app_worker;
REVOKE ALL ON FUNCTION mark_upload_purged(text, uuid) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION mark_upload_purged(text, uuid) TO app_worker;

-- The invoker functions need only the columns used by the cleanup worker.
GRANT SELECT ON attachments, scanned_documents TO app_worker;
GRANT UPDATE (object_purged_at, deleted_at) ON attachments, scanned_documents TO app_worker;

RESET ROLE;
