-- Kullanıcı ayarları (T2.20; adr/0008 › 5 `field_lww`).
-- Kullanıcı başına, çalışma alanı başına bir satır: birim ve görünüm tercihleri. Yapısı çok
-- değişken, içinde arama yapılmayan ayar olduğu için JSONB (adr/0007 › 1.7b); API yazmadan önce
-- bilinen anahtarlara karşı doğrular. Satırı yalnız sahibi görür (kısıtlayıcı politika).
SET LOCAL ROLE app_owner;

CREATE TABLE user_settings (
    id            uuid PRIMARY KEY,
    workspace_id  uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    user_id       uuid NOT NULL REFERENCES users (id),
    settings      jsonb NOT NULL DEFAULT '{"schema_version": 1}'
                  CHECK (jsonb_typeof(settings) = 'object' AND settings ? 'schema_version'),
    created_at    timestamptz NOT NULL DEFAULT now(),
    updated_at    timestamptz NOT NULL DEFAULT now(),
    deleted_at    timestamptz,
    version       bigint NOT NULL DEFAULT 1,
    created_by    uuid REFERENCES users (id),
    updated_by    uuid REFERENCES users (id),
    UNIQUE (workspace_id, id),
    UNIQUE (workspace_id, user_id)
);

SELECT secure_workspace_table('user_settings');
CREATE POLICY own_settings ON user_settings AS RESTRICTIVE
    USING (user_id = (SELECT app_user_id())) WITH CHECK (user_id = (SELECT app_user_id()));

RESET ROLE;
