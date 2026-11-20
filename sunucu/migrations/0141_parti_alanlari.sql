-- Parti alanı (T2.22): tarif satırı başına tek gerçekleşen miktar kaydı.
-- Planlı satır için (parti, satır türü, satır) tekildir; plan dışı malzeme (satırsız) serbesttir.
SET LOCAL ROLE app_owner;

CREATE UNIQUE INDEX batch_ingredient_actuals_line_idx
    ON batch_ingredient_actuals (workspace_id, batch_id, line_kind, line_id)
    WHERE line_id IS NOT NULL AND deleted_at IS NULL;

RESET ROLE;
