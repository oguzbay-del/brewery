-- Tarif alanı (T2.21): hesap özeti sürümde saklanır; düzenleme oturumu (yumuşak kilit).
-- Yayınlanmış sürüm değişmez; `calc_summary` taslakta yazılır ve yayınla birlikte donar
-- (`recipe_versions_guard` tüm sütunları karşılaştırır).
SET LOCAL ROLE app_owner;

-- Yayın anındaki hesap özeti: motor çıktısı ve `est_cells_ml`, `honey_pct` (adr/0006 açıkları).
ALTER TABLE recipe_versions
    ADD COLUMN calc_summary jsonb
    CHECK (calc_summary IS NULL OR (jsonb_typeof(calc_summary) = 'object' AND calc_summary ? 'schema_version'));

-- Tarif başına en çok bir düzenleme oturumu: tutan kullanıcı ve bitiş zamanı. Süresi geçen
-- oturum kilit sayılmaz; başka kullanıcı devralabilir (kilit yumuşaktır, uyarır, engellemez).
CREATE TABLE edit_sessions (
    id            uuid PRIMARY KEY,
    workspace_id  uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    recipe_id     uuid NOT NULL,
    user_id       uuid NOT NULL REFERENCES users (id),
    started_at    timestamptz NOT NULL DEFAULT now(),
    last_seen_at  timestamptz NOT NULL DEFAULT now(),
    expires_at    timestamptz NOT NULL,
    created_at    timestamptz NOT NULL DEFAULT now(),
    updated_at    timestamptz NOT NULL DEFAULT now(),
    -- Oturum bırakılınca satır silinir; `deleted_at` ortak sütun kuralı için durur, kullanılmaz.
    deleted_at    timestamptz,
    version       bigint NOT NULL DEFAULT 1,
    created_by    uuid REFERENCES users (id),
    updated_by    uuid REFERENCES users (id),
    UNIQUE (workspace_id, id),
    UNIQUE (workspace_id, recipe_id),
    FOREIGN KEY (workspace_id, recipe_id) REFERENCES recipes (workspace_id, id) ON DELETE CASCADE
);

SELECT secure_workspace_table('edit_sessions');

RESET ROLE;
