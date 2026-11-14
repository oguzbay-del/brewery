-- Passkey ve ortak cihaz (adr/0009 › 5 ve 7; T2.12).
--   * webauthn_challenges: kayıt ve doğrulama töreninin sunucu tarafı durumu (tek kullanımlık,
--     kısa ömürlü). Redis'e bağımlı değil: istekler farklı kopyalara düşebilir ve Redis yoksa da
--     tören çalışmalıdır.
--   * sessions.idle_timeout_secs: ortak cihaz oturumunun işlemsizlik süresi (cihaza göre
--     değişir; boşsa istemci türünün varsayılanı geçerlidir).
--   * find_session_by_token_hash: yeni sütunu döndürür.
--   * shared_device_member_tiles: ortak cihazdaki kullanıcı kutucukları (PIN'i olan üyeler).
--   * close_shared_device_sessions: cihazın bütün oturumlarını kapatır.

SET LOCAL ROLE app_owner;

CREATE TABLE webauthn_challenges (
    id          uuid PRIMARY KEY,
    user_id     uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    kind        text NOT NULL CHECK (kind IN ('registration', 'authentication')),
    -- webauthn-rs'in serileştirilmiş durumu (PasskeyRegistration ya da PasskeyAuthentication).
    state       jsonb NOT NULL,
    expires_at  timestamptz NOT NULL,
    created_at  timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX webauthn_challenges_user_idx ON webauthn_challenges (user_id);

ALTER TABLE webauthn_challenges ENABLE ROW LEVEL SECURITY;
ALTER TABLE webauthn_challenges FORCE ROW LEVEL SECURITY;
CREATE POLICY own_rows ON webauthn_challenges
    USING (user_id = (SELECT app_user_id())) WITH CHECK (user_id = (SELECT app_user_id()));
GRANT SELECT, INSERT, UPDATE, DELETE ON webauthn_challenges TO app_api;

ALTER TABLE sessions
    ADD COLUMN idle_timeout_secs integer CHECK (idle_timeout_secs IS NULL OR idle_timeout_secs > 0);

DROP FUNCTION find_session_by_token_hash(bytea);
CREATE FUNCTION find_session_by_token_hash(p_token_hash bytea)
    RETURNS TABLE (
        id uuid, user_id uuid, client_kind text, shared_device_id uuid, remember boolean,
        created_at timestamptz, last_activity_at timestamptz, expires_at timestamptz,
        step_up_at timestamptz, revoked_at timestamptz, idle_timeout_secs integer)
    LANGUAGE sql STABLE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
    SELECT s.id, s.user_id, s.client_kind, s.shared_device_id, s.remember,
           s.created_at, s.last_activity_at, s.expires_at, s.step_up_at, s.revoked_at,
           s.idle_timeout_secs
    FROM sessions s
    WHERE s.token_hash = p_token_hash
$$;

-- Kutucuklar yalnız PIN'i olan etkin üyeleri gösterir. İşlev yalnız isteğin çalışma alanı
-- için çalışır (başka çalışma alanının adları sorulamaz).
CREATE FUNCTION shared_device_member_tiles(p_workspace_id uuid)
    RETURNS TABLE (member_id uuid, display_name text)
    LANGUAGE sql STABLE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
    SELECT m.id, u.display_name
    FROM workspace_members m
    JOIN member_pins p ON p.workspace_id = m.workspace_id AND p.member_id = m.id
                      AND p.deleted_at IS NULL
    JOIN users u ON u.id = m.user_id AND u.deleted_at IS NULL
    WHERE m.workspace_id = p_workspace_id
      AND p_workspace_id = (SELECT app_workspace_id())
      AND m.status = 'active' AND m.deleted_at IS NULL
    ORDER BY u.display_name, m.id
$$;

-- Cihazın bütün oturumlarını kapatır (cihaz iptali, anahtar yenileme, kullanıcı değişimi,
-- kilitleme). Oturumlar başka kullanıcılara ait olduğundan satır güvenliği doğrudan
-- güncellemeye izin vermez; işlev yalnız isteğin çalışma alanındaki cihaz için çalışır.
-- Önbelleği silmek için özetleri döndürür.
CREATE FUNCTION close_shared_device_sessions(p_device_id uuid)
    RETURNS TABLE (token_hash bytea)
    LANGUAGE sql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
    UPDATE sessions s SET revoked_at = now()
    WHERE s.shared_device_id = p_device_id AND s.revoked_at IS NULL
      AND EXISTS (SELECT 1 FROM shared_devices d
                  WHERE d.id = p_device_id
                    AND d.workspace_id = (SELECT app_workspace_id()))
    RETURNING s.token_hash
$$;

REVOKE ALL ON FUNCTION find_session_by_token_hash(bytea), shared_device_member_tiles(uuid),
    close_shared_device_sessions(uuid) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION find_session_by_token_hash(bytea), shared_device_member_tiles(uuid),
    close_shared_device_sessions(uuid) TO app_api;

RESET ROLE;
