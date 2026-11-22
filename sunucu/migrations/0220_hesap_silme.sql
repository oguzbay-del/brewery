-- Account deletion requests and account anonymisation (T2.31a).
-- User ids remain as stable references. Audit rows are append-only and are not changed here.
SET LOCAL ROLE app_owner;

CREATE TABLE account_deletion_requests (
    id           uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id      uuid NOT NULL REFERENCES users (id) ON DELETE RESTRICT,
    requested_at timestamptz NOT NULL DEFAULT now(),
    purge_at     timestamptz NOT NULL,
    status       text NOT NULL DEFAULT 'pending'
                 CHECK (status IN ('pending', 'cancelled', 'done')),
    restored_at  timestamptz,
    CHECK (purge_at > requested_at),
    CHECK ((status = 'cancelled') = (restored_at IS NOT NULL))
);

CREATE UNIQUE INDEX account_deletion_pending_user_idx
    ON account_deletion_requests (user_id) WHERE status = 'pending';
CREATE INDEX account_deletion_due_idx
    ON account_deletion_requests (purge_at, id) WHERE status = 'pending';

ALTER TABLE account_deletion_requests ENABLE ROW LEVEL SECURITY;
ALTER TABLE account_deletion_requests FORCE ROW LEVEL SECURITY;
CREATE POLICY own_rows ON account_deletion_requests TO app_api
    USING (user_id = (SELECT app_user_id()))
    WITH CHECK (user_id = (SELECT app_user_id()));
CREATE POLICY worker_all ON account_deletion_requests TO app_worker
    USING (true) WITH CHECK (true);
CREATE POLICY owner_all ON account_deletion_requests TO app_owner
    USING (true) WITH CHECK (true);

-- The owner role also needs explicit policies on account tables protected by FORCE RLS.
-- These policies are only used by the SECURITY DEFINER anonymisation function.
CREATE POLICY account_delete_owner ON passkeys TO app_owner
    USING (true) WITH CHECK (true);

GRANT SELECT, INSERT ON account_deletion_requests TO app_api;
GRANT UPDATE (status, restored_at) ON account_deletion_requests TO app_api;
GRANT SELECT ON account_deletion_requests TO app_worker;
GRANT UPDATE (status, restored_at) ON account_deletion_requests TO app_worker;

-- A second request while one is pending returns the existing request.
CREATE FUNCTION request_account_deletion(p_grace_days integer DEFAULT 30)
    RETURNS TABLE (
        id uuid, user_id uuid, requested_at timestamptz, purge_at timestamptz,
        status text, restored_at timestamptz)
    LANGUAGE plpgsql VOLATILE
    SET search_path = pg_catalog, public, pg_temp
    AS $$
DECLARE
    v_user_id uuid := app_user_id();
BEGIN
    IF v_user_id IS NULL THEN
        RAISE EXCEPTION 'authenticated user is required'
            USING ERRCODE = 'insufficient_privilege';
    END IF;
    IF p_grace_days NOT BETWEEN 1 AND 365 THEN
        RAISE EXCEPTION 'grace period must be between 1 and 365 days'
            USING ERRCODE = 'check_violation';
    END IF;
    IF NOT EXISTS (SELECT 1 FROM users WHERE id = v_user_id AND deleted_at IS NULL) THEN
        RAISE EXCEPTION 'active user is required'
            USING ERRCODE = 'insufficient_privilege';
    END IF;
    RETURN QUERY
    INSERT INTO account_deletion_requests (user_id, purge_at)
    VALUES (v_user_id, now() + make_interval(days => p_grace_days))
    ON CONFLICT (user_id) WHERE status = 'pending'
    DO UPDATE SET status = 'pending'
    RETURNING account_deletion_requests.id, account_deletion_requests.user_id,
              account_deletion_requests.requested_at, account_deletion_requests.purge_at,
              account_deletion_requests.status, account_deletion_requests.restored_at;
END
$$;

CREATE FUNCTION cancel_account_deletion()
    RETURNS TABLE (
        id uuid, user_id uuid, requested_at timestamptz, purge_at timestamptz,
        status text, restored_at timestamptz)
    LANGUAGE sql VOLATILE
    SET search_path = pg_catalog, public, pg_temp
    AS $$
    UPDATE account_deletion_requests
    SET status = 'cancelled', restored_at = now()
    WHERE user_id = app_user_id() AND status = 'pending' AND purge_at > now()
    RETURNING id, user_id, requested_at, purge_at, status, restored_at
$$;

-- The row lock remains held until the caller's transaction ends. Replicas skip rows claimed
-- by another worker. The worker marks a request done only after its purge work succeeds.
CREATE FUNCTION due_account_deletions(p_limit integer DEFAULT 100)
    RETURNS TABLE (request_id uuid, user_id uuid, purge_at timestamptz)
    LANGUAGE plpgsql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
BEGIN
    IF p_limit NOT BETWEEN 1 AND 1000 THEN
        RAISE EXCEPTION 'limit must be between 1 and 1000'
            USING ERRCODE = 'check_violation';
    END IF;
    RETURN QUERY
    SELECT r.id, r.user_id, r.purge_at
    FROM account_deletion_requests r
    WHERE r.status = 'pending' AND r.purge_at <= now()
    ORDER BY r.purge_at, r.id
    LIMIT p_limit
    FOR UPDATE SKIP LOCKED;
END
$$;

-- Keep the user row as a tombstone. Preserve Apple refresh-token ciphertext for the separate
-- token-revocation step (T2.31d); remove all other login/session credentials immediately.
CREATE FUNCTION anonymize_user(p_user_id uuid)
    RETURNS void
    LANGUAGE plpgsql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
BEGIN
    UPDATE users
    SET display_name = 'Deleted user',
        contact_email = NULL,
        contact_email_verified_at = NULL,
        locale = 'en-US',
        deleted_at = COALESCE(deleted_at, now())
    WHERE id = p_user_id;
    IF NOT FOUND THEN
        RAISE EXCEPTION 'user % does not exist', p_user_id
            USING ERRCODE = 'no_data_found';
    END IF;

    UPDATE user_identities
    SET subject = 'deleted:' || user_id::text || ':' || provider,
        provider_refresh_token_enc = CASE
            WHEN provider = 'apple' THEN provider_refresh_token_enc
            ELSE NULL
        END
    WHERE user_id = p_user_id;

    DELETE FROM sessions WHERE user_id = p_user_id;
    DELETE FROM passkeys WHERE user_id = p_user_id;
    DELETE FROM push_targets WHERE user_id = p_user_id;
END
$$;

REVOKE ALL ON FUNCTION request_account_deletion(integer),
    cancel_account_deletion(), due_account_deletions(integer), anonymize_user(uuid) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION request_account_deletion(integer),
    cancel_account_deletion() TO app_api;
GRANT EXECUTE ON FUNCTION due_account_deletions(integer),
    anonymize_user(uuid) TO app_worker;

RESET ROLE;
