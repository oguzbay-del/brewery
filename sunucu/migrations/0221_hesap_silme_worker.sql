-- T2.31c: support corrections use the same guarded account-deletion request path.
-- The worker sets app.user_id only after validating the target membership in the workspace.
SET LOCAL ROLE app_owner;

GRANT EXECUTE ON FUNCTION request_account_deletion(integer) TO app_worker;

RESET ROLE;
