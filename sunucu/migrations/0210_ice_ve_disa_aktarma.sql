-- T2.26: içe ve dışa aktarma.
-- import_sources: BeerJSON biçimi, ayrıştırılmış belge (`parsed`) ve özgün dosya adı.
-- export_jobs: BeerJSON biçimi, parametreler, hata, boyut ve 'expired' durumu. Worker işi
-- `claim_export_job()` ile sahiplenir (çalışma alanları arası okuma; RLS altındaki tabloya
-- yalnız bu dar SECURITY DEFINER işlevleri bakar). Tam dışa aktarma 24 saatte bir (API denetler;
-- indeks sorguyu hızlandırır).
SET LOCAL ROLE app_owner;

ALTER TABLE import_sources DROP CONSTRAINT import_sources_format_check;
ALTER TABLE import_sources ADD CONSTRAINT import_sources_format_check
    CHECK (format IN ('beerxml', 'beerjson', 'beersmith', 'brewfather', 'json', 'csv'));
ALTER TABLE import_sources
    ADD COLUMN parsed jsonb CHECK (parsed IS NULL OR jsonb_typeof(parsed) = 'object'),
    ADD COLUMN file_name text CHECK (char_length(file_name) <= 255);

ALTER TABLE export_jobs DROP CONSTRAINT export_jobs_format_check;
ALTER TABLE export_jobs ADD CONSTRAINT export_jobs_format_check
    CHECK (format IN ('beerxml', 'beerjson', 'json', 'csv', 'pdf', 'zip'));
ALTER TABLE export_jobs DROP CONSTRAINT export_jobs_status_check;
ALTER TABLE export_jobs ADD CONSTRAINT export_jobs_status_check
    CHECK (status IN ('queued', 'running', 'done', 'failed', 'expired'));
ALTER TABLE export_jobs
    ADD COLUMN params jsonb NOT NULL DEFAULT '{}'::jsonb CHECK (jsonb_typeof(params) = 'object'),
    ADD COLUMN error text,
    ADD COLUMN size_bytes bigint CHECK (size_bytes >= 0),
    ADD COLUMN started_at timestamptz;

CREATE INDEX export_jobs_full_idx ON export_jobs (workspace_id, created_at)
    WHERE scope IN ('workspace', 'account_data');
CREATE INDEX export_jobs_queue_idx ON export_jobs (created_at) WHERE status IN ('queued', 'running');

-- İşlevler app_owner olarak çalışır ve tabloda FORCE RLS vardır: yalnız bu işlevlere açık politika.
CREATE POLICY owner_lookup ON export_jobs TO app_owner USING (true) WITH CHECK (true);

-- Sıradaki işi sahiplenir; 15 dakikadan uzun "running" kalan iş (çöken worker) yeniden alınır.
CREATE FUNCTION claim_export_job() RETURNS TABLE (id uuid, workspace_id uuid)
    LANGUAGE sql SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
    UPDATE export_jobs j
       SET status = 'running', started_at = now()
     WHERE j.id = (SELECT q.id FROM export_jobs q
                    WHERE q.deleted_at IS NULL
                      AND (q.status = 'queued'
                           OR (q.status = 'running' AND q.started_at < now() - interval '15 minutes'))
                    ORDER BY q.created_at
                    FOR UPDATE SKIP LOCKED
                    LIMIT 1)
    RETURNING j.id, j.workspace_id
$$;

-- Süresi dolan dışa aktarmaları 'expired' yapar ve nesne anahtarlarını döndürür (worker nesneyi siler).
CREATE FUNCTION expire_export_jobs(p_limit integer DEFAULT 100)
    RETURNS TABLE (id uuid, workspace_id uuid, result_object_key text)
    LANGUAGE sql SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
    WITH old AS (
        SELECT q.id, q.result_object_key AS key FROM export_jobs q
         WHERE q.status = 'done' AND q.expires_at < now()
         ORDER BY q.expires_at
         FOR UPDATE SKIP LOCKED
         LIMIT p_limit)
    UPDATE export_jobs j
       SET status = 'expired', result_object_key = NULL
      FROM old
     WHERE j.id = old.id
    RETURNING j.id, j.workspace_id, old.key
$$;

REVOKE ALL ON FUNCTION claim_export_job(), expire_export_jobs(integer) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION claim_export_job(), expire_export_jobs(integer) TO app_worker;

RESET ROLE;
