-- T2.31d: revoke the Apple refresh token after account deletion.
-- app_worker gets no table access. It calls only these guarded functions.
SET LOCAL ROLE app_owner;

ALTER TABLE user_identities
    ADD COLUMN apple_revoke_attempts smallint NOT NULL DEFAULT 0
    CHECK (apple_revoke_attempts BETWEEN 0 AND 3);

-- Lists stored Apple ciphertexts of deleted accounts that still need a revoke.
CREATE FUNCTION apple_revoke_pending(p_limit integer DEFAULT 100)
    RETURNS TABLE (identity_id uuid, ciphertext bytea, attempts smallint)
    LANGUAGE plpgsql STABLE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
BEGIN
    IF p_limit NOT BETWEEN 1 AND 1000 THEN
        RAISE EXCEPTION 'limit must be between 1 and 1000'
            USING ERRCODE = 'check_violation';
    END IF;
    RETURN QUERY
    SELECT i.id, i.provider_refresh_token_enc, i.apple_revoke_attempts
    FROM user_identities i
    WHERE i.provider = 'apple'
      AND i.provider_refresh_token_enc IS NOT NULL
      AND EXISTS (
          SELECT 1 FROM account_deletion_requests r
          WHERE r.user_id = i.user_id AND r.status = 'done'
      )
    ORDER BY i.id
    LIMIT p_limit;
END
$$;

-- Clears the ciphertext after a successful revoke. Returns true when a row changed.
CREATE FUNCTION apple_revoke_clear(p_identity_id uuid)
    RETURNS boolean
    LANGUAGE plpgsql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
DECLARE
    v_rows integer;
BEGIN
    UPDATE user_identities i
    SET provider_refresh_token_enc = NULL
    WHERE i.id = p_identity_id
      AND i.provider = 'apple'
      AND i.provider_refresh_token_enc IS NOT NULL
      AND EXISTS (
          SELECT 1 FROM account_deletion_requests r
          WHERE r.user_id = i.user_id AND r.status = 'done'
      );
    GET DIAGNOSTICS v_rows = ROW_COUNT;
    RETURN v_rows > 0;
END
$$;

-- Counts one failed revoke. The third failure clears the ciphertext. Returns the new count.
CREATE FUNCTION apple_revoke_failed(p_identity_id uuid)
    RETURNS smallint
    LANGUAGE plpgsql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
DECLARE
    v_attempts smallint;
BEGIN
    UPDATE user_identities i
    SET apple_revoke_attempts = least(i.apple_revoke_attempts + 1, 3),
        provider_refresh_token_enc = CASE
            WHEN i.apple_revoke_attempts + 1 >= 3 THEN NULL
            ELSE i.provider_refresh_token_enc
        END
    WHERE i.id = p_identity_id
      AND i.provider = 'apple'
      AND i.provider_refresh_token_enc IS NOT NULL
      AND EXISTS (
          SELECT 1 FROM account_deletion_requests r
          WHERE r.user_id = i.user_id AND r.status = 'done'
      )
    RETURNING i.apple_revoke_attempts INTO v_attempts;
    RETURN v_attempts;
END
$$;

REVOKE ALL ON FUNCTION
    apple_revoke_pending(integer),
    apple_revoke_clear(uuid),
    apple_revoke_failed(uuid)
FROM PUBLIC, app_api;

GRANT EXECUTE ON FUNCTION
    apple_revoke_pending(integer),
    apple_revoke_clear(uuid),
    apple_revoke_failed(uuid)
TO app_worker;

RESET ROLE;
