-- T2.28: abonelik hakkı tekilliği.
--
-- Bir aboneliğin bir özellik için tek canlı hakkı olur. Worker bildirimi işlerken hakkı
-- (workspace_id, subscription_id, feature) üzerinden ekler ya da günceller; tekrar eden veya
-- sırası bozuk bildirim ikinci bir satır üretmez.

SET LOCAL ROLE app_owner;

CREATE UNIQUE INDEX entitlements_subscription_feature_key
    ON entitlements (workspace_id, subscription_id, feature)
    WHERE subscription_id IS NOT NULL AND deleted_at IS NULL;

RESET ROLE;
