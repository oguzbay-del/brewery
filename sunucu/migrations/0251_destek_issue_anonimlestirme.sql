-- T2.31e: anonymize the GitHub Issues of a deleted account.
-- The purge removes the support rows with the Home workspace. The Issue numbers must be saved
-- first, in the same transaction. They stay on the deletion request until GitHub accepts the edit.
-- app_worker gets no table access. It calls only these guarded functions.
SET LOCAL ROLE app_owner;

ALTER TABLE account_deletion_requests
    ADD COLUMN support_issue_numbers integer[] NOT NULL DEFAULT '{}',
    ADD COLUMN support_issue_attempts integer NOT NULL DEFAULT 0 CHECK (support_issue_attempts >= 0);

-- Saves the Issue numbers of the support requests that the user opened. Call it before the purge.
-- Returns the count of saved numbers.
CREATE FUNCTION account_deletion_support_issues(p_user_id uuid)
    RETURNS integer
    LANGUAGE plpgsql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
DECLARE
    v_request_id uuid;
    v_numbers integer[];
BEGIN
    SELECT r.id INTO v_request_id
    FROM account_deletion_requests r
    WHERE r.user_id = p_user_id AND r.status = 'pending' AND r.purge_at <= now()
    ORDER BY r.purge_at, r.id
    LIMIT 1
    FOR UPDATE;
    IF v_request_id IS NULL THEN
        RAISE EXCEPTION 'due_pending_deletion_required'
            USING ERRCODE = 'insufficient_privilege';
    END IF;
    SELECT coalesce(array_agg(DISTINCT s.github_issue_number ORDER BY s.github_issue_number), '{}')
    INTO v_numbers
    FROM support_requests s
    WHERE s.created_by = p_user_id AND s.github_issue_number IS NOT NULL;
    UPDATE account_deletion_requests
    SET support_issue_numbers = v_numbers
    WHERE id = v_request_id;
    RETURN cardinality(v_numbers);
END
$$;

-- Lists the Issue numbers that still need an edit, with the failed attempts of their request.
CREATE FUNCTION support_issue_anonymize_pending(p_limit integer DEFAULT 100)
    RETURNS TABLE (request_id uuid, issue_number integer, attempts integer)
    LANGUAGE plpgsql STABLE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
BEGIN
    IF p_limit NOT BETWEEN 1 AND 1000 THEN
        RAISE EXCEPTION 'limit must be between 1 and 1000'
            USING ERRCODE = 'check_violation';
    END IF;
    RETURN QUERY
    SELECT r.id, n.issue, r.support_issue_attempts
    FROM account_deletion_requests r
    CROSS JOIN LATERAL unnest(r.support_issue_numbers) AS n(issue)
    WHERE r.status = 'done' AND cardinality(r.support_issue_numbers) > 0
    ORDER BY r.id, n.issue
    LIMIT p_limit;
END
$$;

-- Removes one Issue number after GitHub accepted the edit. Returns true when a number was removed.
CREATE FUNCTION support_issue_anonymize_done(p_request_id uuid, p_issue integer)
    RETURNS boolean
    LANGUAGE plpgsql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
DECLARE
    v_rows integer;
BEGIN
    UPDATE account_deletion_requests r
    SET support_issue_numbers = array_remove(r.support_issue_numbers, p_issue),
        support_issue_attempts = CASE
            WHEN cardinality(array_remove(r.support_issue_numbers, p_issue)) = 0 THEN 0
            ELSE r.support_issue_attempts
        END
    WHERE r.id = p_request_id AND r.status = 'done' AND p_issue = ANY (r.support_issue_numbers);
    GET DIAGNOSTICS v_rows = ROW_COUNT;
    RETURN v_rows > 0;
END
$$;

-- Counts one failed edit for the request. Returns the new count.
CREATE FUNCTION support_issue_anonymize_failed(p_request_id uuid)
    RETURNS integer
    LANGUAGE plpgsql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
DECLARE
    v_attempts integer;
BEGIN
    UPDATE account_deletion_requests r
    SET support_issue_attempts = least(r.support_issue_attempts + 1, 1000000)
    WHERE r.id = p_request_id AND r.status = 'done' AND cardinality(r.support_issue_numbers) > 0
    RETURNING r.support_issue_attempts INTO v_attempts;
    RETURN v_attempts;
END
$$;

REVOKE ALL ON FUNCTION
    account_deletion_support_issues(uuid),
    support_issue_anonymize_pending(integer),
    support_issue_anonymize_done(uuid, integer),
    support_issue_anonymize_failed(uuid)
FROM PUBLIC, app_api;

GRANT EXECUTE ON FUNCTION
    account_deletion_support_issues(uuid),
    support_issue_anonymize_pending(integer),
    support_issue_anonymize_done(uuid, integer),
    support_issue_anonymize_failed(uuid)
TO app_worker;

RESET ROLE;
