-- K2: three container permissions, the `saha` preset role and a backfill for Business workspaces.
-- Home workspaces get nothing (owner decision 2026-11-25).
SET LOCAL ROLE app_owner;

ALTER TABLE role_permissions DROP CONSTRAINT role_permissions_permission_check;
ALTER TABLE role_permissions ADD CONSTRAINT role_permissions_permission_check CHECK (permission IN (
    'recipe.view', 'recipe.edit', 'recipe.approve',
    'batch.plan', 'batch.record', 'batch.correct_closed',
    'quality.record', 'release.approve', 'stock.move', 'cost.view',
    'reports.export', 'settings.equipment', 'users.manage',
    'workspace.manage', 'billing.manage', 'support.grant', 'devices.manage',
    'field.view', 'container.status', 'container.manage'));

-- The backfill reads and writes rows of every workspace. The owner role is exempt from row
-- level security only while FORCE is off, so switch it off inside this transaction.
ALTER TABLE workspaces NO FORCE ROW LEVEL SECURITY;
ALTER TABLE roles NO FORCE ROW LEVEL SECURITY;
ALTER TABLE role_permissions NO FORCE ROW LEVEL SECURITY;

-- The preset role `saha` for each Business workspace that has no role of that name yet.
INSERT INTO roles (id, workspace_id, name, template_key, landing_screen)
SELECT gen_random_uuid(), w.id, 'Field', 'saha', 'field'
FROM workspaces w
WHERE w.mode = 'business'
  AND NOT EXISTS (SELECT 1 FROM roles r WHERE r.workspace_id = w.id
                  AND (r.template_key = 'saha' OR lower(r.name) = 'field')
                  AND r.deleted_at IS NULL);

INSERT INTO role_permissions (workspace_id, role_id, permission)
SELECT r.workspace_id, r.id, p.permission
FROM roles r
JOIN workspaces w ON w.id = r.workspace_id AND w.mode = 'business'
JOIN (VALUES
    ('admin', 'field.view'), ('admin', 'container.status'), ('admin', 'container.manage'),
    ('brewmaster', 'field.view'), ('brewmaster', 'container.status'),
    ('brewmaster', 'container.manage'),
    ('operator', 'field.view'), ('operator', 'container.status'),
    ('quality', 'field.view'),
    ('warehouse', 'field.view'), ('warehouse', 'container.status'),
    ('warehouse', 'container.manage'),
    ('saha', 'field.view'), ('saha', 'container.status')
) AS p (template_key, permission) ON p.template_key = r.template_key
WHERE r.deleted_at IS NULL
ON CONFLICT DO NOTHING;

ALTER TABLE workspaces FORCE ROW LEVEL SECURITY;
ALTER TABLE roles FORCE ROW LEVEL SECURITY;
ALTER TABLE role_permissions FORCE ROW LEVEL SECURITY;

RESET ROLE;
