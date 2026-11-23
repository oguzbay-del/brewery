-- T2.31c: narrow SECURITY DEFINER contract for scheduled account deletion.
-- No direct table grants are added. app_worker can call only these guarded functions.
SET LOCAL ROLE app_owner;

CREATE FUNCTION account_deletion_memberships(p_user_id uuid)
    RETURNS TABLE (
        workspace_id uuid,
        mode text,
        member_status text,
        other_active_members integer
    )
    LANGUAGE plpgsql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
BEGIN
    PERFORM 1
    FROM account_deletion_requests r
    WHERE r.user_id = p_user_id AND r.status = 'pending' AND r.purge_at <= now()
    FOR UPDATE;
    IF NOT FOUND THEN
        RAISE EXCEPTION 'due_pending_deletion_required'
            USING ERRCODE = 'insufficient_privilege';
    END IF;

    RETURN QUERY
    SELECT
        m.workspace_id,
        w.mode,
        m.status,
        (
            SELECT count(*)::integer
            FROM workspace_members other_member
            WHERE other_member.workspace_id = m.workspace_id
              AND other_member.user_id <> p_user_id
              AND other_member.status = 'active'
              AND other_member.deleted_at IS NULL
        ) AS other_active_members
    FROM workspace_members m
    JOIN workspaces w ON w.id = m.workspace_id
    WHERE m.user_id = p_user_id AND m.deleted_at IS NULL
    ORDER BY m.workspace_id;
END
$$;

CREATE FUNCTION account_deletion_leave_business(p_user_id uuid)
    RETURNS void
    LANGUAGE plpgsql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
DECLARE
    v_member record;
    v_active_owners integer;
    v_is_owner boolean;
BEGIN
    PERFORM 1
    FROM account_deletion_requests r
    WHERE r.user_id = p_user_id AND r.status = 'pending' AND r.purge_at <= now()
    FOR UPDATE;
    IF NOT FOUND THEN
        RAISE EXCEPTION 'due_pending_deletion_required'
            USING ERRCODE = 'insufficient_privilege';
    END IF;

    FOR v_member IN
        SELECT m.id, m.workspace_id, m.status
        FROM workspace_members m
        JOIN workspaces w ON w.id = m.workspace_id
        WHERE m.user_id = p_user_id
          AND w.mode = 'business'
          AND m.deleted_at IS NULL
        ORDER BY m.workspace_id
    LOOP
        v_is_owner := false;
        IF v_member.status = 'active' THEN
            SELECT EXISTS (
                SELECT 1
                FROM role_assignments ra
                JOIN roles r
                  ON r.workspace_id = ra.workspace_id AND r.id = ra.role_id
                 AND r.deleted_at IS NULL
                JOIN role_permissions rp
                  ON rp.workspace_id = r.workspace_id AND rp.role_id = r.id
                 AND rp.permission = 'users.manage'
                WHERE ra.workspace_id = v_member.workspace_id
                  AND ra.member_id = v_member.id
                  AND ra.deleted_at IS NULL
                  AND ra.scope_kind IS NULL
            ) INTO v_is_owner;
        END IF;

        IF v_is_owner THEN
            SELECT count(DISTINCT owner_member.id)::integer
            INTO v_active_owners
            FROM workspace_members owner_member
            JOIN role_assignments owner_assignment
              ON owner_assignment.workspace_id = owner_member.workspace_id
             AND owner_assignment.member_id = owner_member.id
             AND owner_assignment.deleted_at IS NULL
             AND owner_assignment.scope_kind IS NULL
            JOIN roles owner_role
              ON owner_role.workspace_id = owner_assignment.workspace_id
             AND owner_role.id = owner_assignment.role_id
             AND owner_role.deleted_at IS NULL
            JOIN role_permissions owner_permission
              ON owner_permission.workspace_id = owner_role.workspace_id
             AND owner_permission.role_id = owner_role.id
             AND owner_permission.permission = 'users.manage'
            WHERE owner_member.workspace_id = v_member.workspace_id
              AND owner_member.status = 'active'
              AND owner_member.deleted_at IS NULL;

            IF v_active_owners <= 1 THEN
                RAISE EXCEPTION USING ERRCODE = 'P0001', MESSAGE = 'last_owner';
            END IF;
        END IF;

        UPDATE workspace_members
        SET status = 'disabled', deleted_at = now()
        WHERE id = v_member.id AND deleted_at IS NULL;
    END LOOP;
END
$$;

CREATE FUNCTION account_deletion_soft_delete_home(p_user_id uuid)
    RETURNS SETOF uuid
    LANGUAGE plpgsql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
DECLARE
    v_workspace_id uuid;
BEGIN
    PERFORM 1
    FROM account_deletion_requests r
    WHERE r.user_id = p_user_id AND r.status = 'pending' AND r.purge_at <= now()
    FOR UPDATE;
    IF NOT FOUND THEN
        RAISE EXCEPTION 'due_pending_deletion_required'
            USING ERRCODE = 'insufficient_privilege';
    END IF;

    -- A Home workspace with only this member can be purged immediately after the request grace
    -- period. Shared Home workspaces remain; only this user's membership is disabled.
    FOR v_workspace_id IN
        SELECT w.id
        FROM workspaces w
        JOIN workspace_members m ON m.workspace_id = w.id
        WHERE w.mode = 'home'
          AND m.user_id = p_user_id
          AND m.deleted_at IS NULL
        GROUP BY w.id
        HAVING count(*) = 1
        ORDER BY w.id
    LOOP
        UPDATE workspaces
        SET deleted_at = COALESCE(deleted_at, now())
        WHERE id = v_workspace_id;
        RETURN NEXT v_workspace_id;
    END LOOP;

    UPDATE workspace_members m
    SET status = 'disabled', deleted_at = now()
    FROM workspaces w
    WHERE w.id = m.workspace_id
      AND w.mode = 'home'
      AND m.user_id = p_user_id
      AND m.deleted_at IS NULL
      AND EXISTS (
          SELECT 1
          FROM workspace_members other_member
          WHERE other_member.workspace_id = m.workspace_id
            AND other_member.user_id <> p_user_id
            AND other_member.deleted_at IS NULL
      );
END
$$;

CREATE FUNCTION account_deletion_object_keys(p_workspace_id uuid)
    RETURNS SETOF text
    LANGUAGE plpgsql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
DECLARE
    v_table text;
    v_column text;
    v_key text;
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM workspaces w
        WHERE w.id = p_workspace_id AND w.mode = 'home' AND w.deleted_at IS NOT NULL
    ) THEN
        RAISE EXCEPTION 'workspace_not_soft_deleted'
            USING ERRCODE = 'insufficient_privilege';
    END IF;

    FOR v_table, v_column IN
        SELECT c.table_name, c.column_name
        FROM information_schema.columns c
        WHERE c.table_schema = 'public'
          AND c.data_type IN ('text', 'character varying')
          AND (c.column_name = 'object_key' OR c.column_name LIKE '%_object_key')
          AND EXISTS (
              SELECT 1
              FROM information_schema.columns w
              WHERE w.table_schema = c.table_schema
                AND w.table_name = c.table_name
                AND w.column_name = 'workspace_id'
          )
        ORDER BY c.table_name, c.column_name
    LOOP
        FOR v_key IN EXECUTE format(
            'SELECT %I FROM public.%I WHERE workspace_id = $1 AND %I IS NOT NULL',
            v_column, v_table, v_column
        ) USING p_workspace_id
        LOOP
            RETURN NEXT v_key;
        END LOOP;
    END LOOP;
END
$$;

CREATE FUNCTION account_deletion_from_support(p_correction_id uuid)
    RETURNS account_deletion_requests
    LANGUAGE plpgsql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
DECLARE
    v_correction support_corrections%ROWTYPE;
    v_user_id uuid;
    v_grace_days integer;
    v_previous_user_id text;
    v_request account_deletion_requests%ROWTYPE;
BEGIN
    SELECT * INTO v_correction
    FROM support_corrections c
    WHERE c.id = p_correction_id
      AND c.kind = 'deletion_request'
      AND c.status = 'pending'
    FOR UPDATE;
    IF NOT FOUND THEN
        RAISE EXCEPTION 'pending_deletion_request_correction_required'
            USING ERRCODE = 'no_data_found';
    END IF;
    IF v_correction.workspace_id IS DISTINCT FROM app_workspace_id() THEN
        RAISE EXCEPTION 'workspace_context_mismatch'
            USING ERRCODE = 'insufficient_privilege';
    END IF;

    v_user_id := NULLIF(v_correction.params ->> 'user_id', '')::uuid;
    IF v_user_id IS NULL OR NOT EXISTS (
        SELECT 1
        FROM workspace_members m
        WHERE m.workspace_id = v_correction.workspace_id
          AND m.user_id = v_user_id
          AND m.deleted_at IS NULL
    ) THEN
        RAISE EXCEPTION 'target_user_not_member'
            USING ERRCODE = 'insufficient_privilege';
    END IF;

    v_grace_days := COALESCE(
        NULLIF(current_setting('app.account_deletion_grace_days', true), '')::integer,
        30
    );
    IF v_grace_days < 1 THEN
        RAISE EXCEPTION 'grace_period_must_be_positive'
            USING ERRCODE = 'check_violation';
    END IF;

    v_previous_user_id := current_setting('app.user_id', true);
    PERFORM set_config('app.user_id', v_user_id::text, true);
    SELECT * INTO v_request FROM request_account_deletion(v_grace_days);
    PERFORM set_config('app.user_id', COALESCE(v_previous_user_id, ''), true);

    UPDATE support_corrections
    SET status = 'applied',
        result = format('account deletion scheduled until %s', v_request.purge_at),
        applied_at = now(),
        params = params - 'user_id'
    WHERE id = p_correction_id;

    RETURN v_request;
END
$$;

REVOKE ALL ON FUNCTION
    account_deletion_memberships(uuid),
    account_deletion_leave_business(uuid),
    account_deletion_soft_delete_home(uuid),
    account_deletion_object_keys(uuid),
    account_deletion_from_support(uuid)
FROM PUBLIC, app_api;

GRANT EXECUTE ON FUNCTION
    account_deletion_memberships(uuid),
    account_deletion_leave_business(uuid),
    account_deletion_soft_delete_home(uuid),
    account_deletion_object_keys(uuid),
    account_deletion_from_support(uuid)
TO app_worker;

RESET ROLE;
