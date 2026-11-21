-- T2.29: destek talebi GitHub eşlemesi. Müşteri mesajı ve talep durumu GitHub'a worker'dan gider;
-- GitHub yorumu ve Issue durumu ingest'ten (webhook) ve worker'dan gelir.
SET LOCAL ROLE app_owner;

-- Mesajın GitHub tarafı: son eşitlenen sürüm (düzenleme algısı) ve GitHub'da silinme zamanı.
ALTER TABLE support_messages
    ADD COLUMN github_synced_version bigint,
    ADD COLUMN github_deleted_at timestamptz;
CREATE UNIQUE INDEX support_messages_github_comment_idx ON support_messages (github_comment_id)
    WHERE github_comment_id IS NOT NULL;

-- Talebin GitHub tarafı: Issue'ya en son yazılan durum ve çözüldü zamanı.
ALTER TABLE support_requests
    ADD COLUMN github_synced_status text CHECK (github_synced_status IN (
        'open', 'answered', 'resolved', 'closed')),
    ADD COLUMN resolved_at timestamptz;
CREATE UNIQUE INDEX support_requests_issue_idx ON support_requests (github_issue_number)
    WHERE github_issue_number IS NOT NULL;

-- Destek ekibi listesi: `/yanit` yalnız bu GitHub kullanıcılarından kabul edilir.
ALTER TABLE staff_users ADD COLUMN github_login text CHECK (github_login = lower(github_login));
CREATE UNIQUE INDEX staff_users_github_login_idx ON staff_users (github_login)
    WHERE github_login IS NOT NULL;
GRANT SELECT ON staff_users TO app_worker;

-- Bir talep için tek bekleyen eşitleme işi: GitHub'a ulaşılamazsa iş yeniden denenir.
CREATE UNIQUE INDEX scheduled_jobs_support_sync_idx
    ON scheduled_jobs ((payload ->> 'request_id'))
    WHERE kind = 'support.github_sync' AND status IN ('pending', 'running');

-- Çalışma alanları arası arama (webhook: Issue numarası; destek paneli: genel numara). Yalnız
-- kimlik döner.
CREATE POLICY owner_read ON support_requests FOR SELECT TO app_owner USING (true);

CREATE FUNCTION support_find_request(p_issue integer DEFAULT NULL, p_number bigint DEFAULT NULL)
    RETURNS TABLE (workspace_id uuid, id uuid)
    LANGUAGE sql STABLE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
    SELECT r.workspace_id, r.id FROM support_requests r
    WHERE r.deleted_at IS NULL
      AND ((p_issue IS NOT NULL AND r.github_issue_number = p_issue)
           OR (p_number IS NOT NULL AND r.public_number = p_number))
    LIMIT 1
$$;
REVOKE ALL ON FUNCTION support_find_request(integer, bigint) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION support_find_request(integer, bigint) TO app_api, app_worker;

RESET ROLE;
