-- Çalışma alanı ve hesap tabloları (adr/0007 › 3 "Hesap, çalışma alanı, yetki"; adr/0009).
-- Hesap tabloları çalışma alanından bağımsızdır; politika `user_id = app.user_id`. Girişten
-- önceki tablolar (email_login_tokens, auth_requests) uygulama rollerine kapalıdır, yalnız
-- aşağıdaki dar SECURITY DEFINER işlevlerle erişilir (adr/0007 › 2.6).
SET LOCAL ROLE app_owner;

-- workspaces ---------------------------------------------------------------------------
CREATE TABLE workspaces (
    id                uuid PRIMARY KEY,
    name              text NOT NULL CHECK (char_length(name) BETWEEN 1 AND 200),
    mode              text NOT NULL CHECK (mode IN ('home', 'business')),
    region            text NOT NULL CHECK (region IN ('us', 'uk')),
    country           char(2) NOT NULL CHECK (country ~ '^[A-Z]{2}$'),
    time_zone         text NOT NULL CHECK (char_length(time_zone) BETWEEN 1 AND 64),
    support_pseudonym text NOT NULL UNIQUE CHECK (char_length(support_pseudonym) BETWEEN 8 AND 64),
    settings          jsonb NOT NULL DEFAULT '{"schema_version": 1}'
                      CHECK (jsonb_typeof(settings) = 'object' AND settings ? 'schema_version'),
    -- Rol, atama ve üyelik değişiminde artar; etkin izin önbelleği buna bağlıdır (adr/0009 › 8.4).
    authz_version     bigint NOT NULL DEFAULT 1,
    created_at        timestamptz NOT NULL DEFAULT now(),
    updated_at        timestamptz NOT NULL DEFAULT now(),
    deleted_at        timestamptz,
    version           bigint NOT NULL DEFAULT 1,
    created_by        uuid,
    updated_by        uuid
);

-- Çalışma alanı bilgisi yalnız kendi bağlamında görünür. Yeni çalışma alanı açılırken API
-- önce kimliği üretip bağlama yazar, sonra ekler (WITH CHECK aynı koşul).
ALTER TABLE workspaces ENABLE ROW LEVEL SECURITY;
ALTER TABLE workspaces FORCE ROW LEVEL SECURITY;
CREATE POLICY workspace_isolation ON workspaces
    USING (id = (SELECT app_workspace_id()))
    WITH CHECK (id = (SELECT app_workspace_id()));
CREATE TRIGGER touch BEFORE UPDATE ON workspaces
    FOR EACH ROW EXECUTE FUNCTION touch_row();

-- users --------------------------------------------------------------------------------
-- E-posta kimlik değildir (adr/0009 › 1.4). Hesap silinince satır mezar taşı olarak kalır:
-- kişisel alanlar boşaltılır, deleted_at dolar (adr/0007 › 6.2).
CREATE TABLE users (
    id                         uuid PRIMARY KEY,
    display_name               text NOT NULL DEFAULT '' CHECK (char_length(display_name) <= 200),
    contact_email              text CHECK (contact_email = lower(contact_email)),
    contact_email_verified_at  timestamptz,
    locale                     text NOT NULL DEFAULT 'en-US' CHECK (char_length(locale) BETWEEN 2 AND 35),
    created_at                 timestamptz NOT NULL DEFAULT now(),
    updated_at                 timestamptz NOT NULL DEFAULT now(),
    deleted_at                 timestamptz,
    version                    bigint NOT NULL DEFAULT 1,
    CHECK (contact_email_verified_at IS NULL OR contact_email IS NOT NULL)
);
-- Aynı doğrulanmış adresi taşıyan başka hesap araması (adr/0009 › 1.6); kimlik değil, tekil değil.
CREATE INDEX users_contact_email_idx ON users (contact_email) WHERE contact_email IS NOT NULL;

ALTER TABLE users ENABLE ROW LEVEL SECURITY;
ALTER TABLE users FORCE ROW LEVEL SECURITY;
CREATE POLICY own_row ON users
    USING (id = (SELECT app_user_id()))
    WITH CHECK (id = (SELECT app_user_id()));
CREATE TRIGGER touch BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION touch_row();

ALTER TABLE workspaces
    ADD CONSTRAINT workspaces_created_by_fkey FOREIGN KEY (created_by) REFERENCES users (id),
    ADD CONSTRAINT workspaces_updated_by_fkey FOREIGN KEY (updated_by) REFERENCES users (id);

-- user_identities ----------------------------------------------------------------------
CREATE TABLE user_identities (
    id                          uuid PRIMARY KEY,
    user_id                     uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    provider                    text NOT NULL CHECK (provider IN ('apple', 'google', 'email')),
    -- Sağlayıcıdaki kalıcı kimlik; e-postada normalize adres (küçük harf, NFC).
    subject                     text NOT NULL CHECK (char_length(subject) BETWEEN 1 AND 320),
    -- Apple yenileme belirteci, zarf şifreli (adr/0009 › 1.5).
    provider_refresh_token_enc  bytea,
    linked_at                   timestamptz NOT NULL DEFAULT now(),
    created_at                  timestamptz NOT NULL DEFAULT now(),
    updated_at                  timestamptz NOT NULL DEFAULT now(),
    version                     bigint NOT NULL DEFAULT 1,
    UNIQUE (provider, subject)
);
CREATE INDEX user_identities_user_id_idx ON user_identities (user_id);

-- passkeys -----------------------------------------------------------------------------
CREATE TABLE passkeys (
    id             uuid PRIMARY KEY,
    user_id        uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    credential_id  bytea NOT NULL UNIQUE,
    public_key     bytea NOT NULL,
    sign_count     bigint NOT NULL DEFAULT 0 CHECK (sign_count >= 0),
    name           text NOT NULL DEFAULT '' CHECK (char_length(name) <= 100),
    last_used_at   timestamptz,
    created_at     timestamptz NOT NULL DEFAULT now(),
    updated_at     timestamptz NOT NULL DEFAULT now(),
    version        bigint NOT NULL DEFAULT 1
);
CREATE INDEX passkeys_user_id_idx ON passkeys (user_id);

-- sessions -----------------------------------------------------------------------------
-- Belirteç opaktır; yalnız SHA-256 özeti saklanır (adr/0009 › 3). shared_device_id için
-- yabancı anahtar shared_devices ile birlikte eklenir (0005).
CREATE TABLE sessions (
    id                    uuid PRIMARY KEY,
    user_id               uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    token_hash            bytea NOT NULL UNIQUE CHECK (octet_length(token_hash) = 32),
    client_kind           text NOT NULL CHECK (client_kind IN ('ios', 'web', 'shared_device')),
    shared_device_id      uuid,
    device_name           text NOT NULL DEFAULT '' CHECK (char_length(device_name) <= 100),
    approx_location       text,
    remember              boolean NOT NULL DEFAULT false,
    last_activity_at      timestamptz NOT NULL DEFAULT now(),
    expires_at            timestamptz NOT NULL,
    step_up_at            timestamptz,
    revoked_at            timestamptz,
    created_at            timestamptz NOT NULL DEFAULT now(),
    updated_at            timestamptz NOT NULL DEFAULT now(),
    version               bigint NOT NULL DEFAULT 1,
    CHECK ((client_kind = 'shared_device') = (shared_device_id IS NOT NULL))
);
CREATE INDEX sessions_user_id_idx ON sessions (user_id) WHERE revoked_at IS NULL;

-- push_targets -------------------------------------------------------------------------
CREATE TABLE push_targets (
    id               uuid PRIMARY KEY,
    user_id          uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    environment      text NOT NULL CHECK (environment IN ('sandbox', 'production')),
    token            text NOT NULL CHECK (char_length(token) BETWEEN 1 AND 512),
    app_version      text,
    last_success_at  timestamptz,
    last_error       text,
    last_error_at    timestamptz,
    created_at       timestamptz NOT NULL DEFAULT now(),
    updated_at       timestamptz NOT NULL DEFAULT now(),
    version          bigint NOT NULL DEFAULT 1,
    UNIQUE (environment, token)
);
CREATE INDEX push_targets_user_id_idx ON push_targets (user_id);

-- email_login_tokens / auth_requests (girişten önce, kullanıcısız) ----------------------
CREATE TABLE email_login_tokens (
    id           uuid PRIMARY KEY,
    token_hash   bytea NOT NULL UNIQUE CHECK (octet_length(token_hash) = 32),
    email        text NOT NULL CHECK (email = lower(email)),
    expires_at   timestamptz NOT NULL,
    consumed_at  timestamptz,
    created_at   timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE auth_requests (
    id             uuid PRIMARY KEY,
    nonce          text NOT NULL UNIQUE,
    state          text UNIQUE,
    pkce_verifier  text,
    expires_at     timestamptz NOT NULL,
    consumed_at    timestamptz,
    created_at     timestamptz NOT NULL DEFAULT now()
);

-- staff_users --------------------------------------------------------------------------
-- Destek ekibi (ortak tablo, çalışma alanı dışı). Yalnız göç ve tohum rolü yazar; API okur.
CREATE TABLE staff_users (
    id               uuid PRIMARY KEY,
    google_subject   text NOT NULL UNIQUE,
    email            text NOT NULL CHECK (email = lower(email)),
    role             text NOT NULL CHECK (role IN ('support', 'support_admin')),
    status           text NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'disabled')),
    created_at       timestamptz NOT NULL DEFAULT now(),
    updated_at       timestamptz NOT NULL DEFAULT now(),
    deleted_at       timestamptz,
    version          bigint NOT NULL DEFAULT 1
);
CREATE TRIGGER touch BEFORE UPDATE ON staff_users
    FOR EACH ROW EXECUTE FUNCTION touch_row();

-- Hesap tablolarının ortak kurulumu ------------------------------------------------------
CREATE TRIGGER touch BEFORE UPDATE ON user_identities FOR EACH ROW EXECUTE FUNCTION touch_row();
CREATE TRIGGER touch BEFORE UPDATE ON passkeys        FOR EACH ROW EXECUTE FUNCTION touch_row();
CREATE TRIGGER touch BEFORE UPDATE ON sessions        FOR EACH ROW EXECUTE FUNCTION touch_row();
CREATE TRIGGER touch BEFORE UPDATE ON push_targets    FOR EACH ROW EXECUTE FUNCTION touch_row();

ALTER TABLE user_identities ENABLE ROW LEVEL SECURITY;
ALTER TABLE user_identities FORCE ROW LEVEL SECURITY;
ALTER TABLE passkeys        ENABLE ROW LEVEL SECURITY;
ALTER TABLE passkeys        FORCE ROW LEVEL SECURITY;
ALTER TABLE sessions        ENABLE ROW LEVEL SECURITY;
ALTER TABLE sessions        FORCE ROW LEVEL SECURITY;
ALTER TABLE push_targets    ENABLE ROW LEVEL SECURITY;
ALTER TABLE push_targets    FORCE ROW LEVEL SECURITY;
ALTER TABLE email_login_tokens ENABLE ROW LEVEL SECURITY;
ALTER TABLE email_login_tokens FORCE ROW LEVEL SECURITY;
ALTER TABLE auth_requests   ENABLE ROW LEVEL SECURITY;
ALTER TABLE auth_requests   FORCE ROW LEVEL SECURITY;

CREATE POLICY own_rows ON user_identities
    USING (user_id = (SELECT app_user_id())) WITH CHECK (user_id = (SELECT app_user_id()));
CREATE POLICY own_rows ON passkeys
    USING (user_id = (SELECT app_user_id())) WITH CHECK (user_id = (SELECT app_user_id()));
CREATE POLICY own_rows ON sessions
    USING (user_id = (SELECT app_user_id())) WITH CHECK (user_id = (SELECT app_user_id()));
CREATE POLICY own_rows ON push_targets
    USING (user_id = (SELECT app_user_id())) WITH CHECK (user_id = (SELECT app_user_id()));

-- Girişten önceki işlevler tabloya app_owner olarak erişir; FORCE RLS sahibi de bağladığı
-- için bu tablolarda sahibe açık bir politika gerekir. Uygulama rolleri bu politikadan
-- yararlanamaz (rol sahip değildir, tabloda hakkı yoktur; yalnız işlevi çağırabilir).
CREATE POLICY owner_lookup ON user_identities TO app_owner USING (true) WITH CHECK (true);
CREATE POLICY owner_lookup ON sessions        TO app_owner USING (true) WITH CHECK (true);
CREATE POLICY owner_lookup ON email_login_tokens TO app_owner USING (true) WITH CHECK (true);
CREATE POLICY owner_lookup ON auth_requests   TO app_owner USING (true) WITH CHECK (true);

-- Haklar -------------------------------------------------------------------------------
GRANT SELECT, INSERT, UPDATE, DELETE ON users, user_identities, passkeys, sessions, push_targets
    TO app_api;
GRANT SELECT ON users TO app_worker;
GRANT SELECT, INSERT, UPDATE, DELETE ON push_targets TO app_worker;
-- Çalışma alanını API ekler/günceller/siler; worker okur (bağlamında).
GRANT SELECT, INSERT, UPDATE, DELETE ON workspaces TO app_api;
GRANT SELECT ON workspaces TO app_worker;
GRANT SELECT ON staff_users TO app_api;
-- email_login_tokens ve auth_requests: uygulama rollerine hak yok, yalnız işlevler.

-- Girişten önceki aramalar (adr/0007 › 2.6) ---------------------------------------------
-- Her biri tek amaçlı, yalnız gereken sütunları döndürür; search_path sabittir.

CREATE FUNCTION find_user_by_identity(p_provider text, p_subject text)
    RETURNS TABLE (user_id uuid, identity_id uuid)
    LANGUAGE sql STABLE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
    SELECT i.user_id, i.id
    FROM user_identities i
    WHERE i.provider = p_provider AND i.subject = p_subject
$$;

CREATE FUNCTION find_session_by_token_hash(p_token_hash bytea)
    RETURNS TABLE (
        id uuid, user_id uuid, client_kind text, shared_device_id uuid,
        last_activity_at timestamptz, expires_at timestamptz,
        step_up_at timestamptz, revoked_at timestamptz)
    LANGUAGE sql STABLE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
    SELECT s.id, s.user_id, s.client_kind, s.shared_device_id,
           s.last_activity_at, s.expires_at, s.step_up_at, s.revoked_at
    FROM sessions s
    WHERE s.token_hash = p_token_hash
$$;

CREATE FUNCTION email_login_token_create(
    p_id uuid, p_token_hash bytea, p_email text, p_expires_at timestamptz)
    RETURNS void
    LANGUAGE sql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
    INSERT INTO email_login_tokens (id, token_hash, email, expires_at)
    VALUES (p_id, p_token_hash, p_email, p_expires_at)
$$;

-- Tek kullanımlık: geçerli ve harcanmamışsa işaretler ve adresi döndürür, yoksa satır yok.
CREATE FUNCTION email_login_token_consume(p_token_hash bytea)
    RETURNS TABLE (email text)
    LANGUAGE sql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
    UPDATE email_login_tokens t
    SET consumed_at = now()
    WHERE t.token_hash = p_token_hash AND t.consumed_at IS NULL AND t.expires_at > now()
    RETURNING t.email
$$;

CREATE FUNCTION auth_request_create(
    p_id uuid, p_nonce text, p_state text, p_pkce_verifier text, p_expires_at timestamptz)
    RETURNS void
    LANGUAGE sql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
    INSERT INTO auth_requests (id, nonce, state, pkce_verifier, expires_at)
    VALUES (p_id, p_nonce, p_state, p_pkce_verifier, p_expires_at)
$$;

-- Nonce (iOS) ya da state (web) ile, biri dolu biri NULL; tek kullanımlık.
CREATE FUNCTION auth_request_consume(p_nonce text, p_state text)
    RETURNS TABLE (nonce text, pkce_verifier text)
    LANGUAGE sql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
    UPDATE auth_requests a
    SET consumed_at = now()
    WHERE a.consumed_at IS NULL AND a.expires_at > now()
      AND num_nonnulls(p_nonce, p_state) = 1
      AND ((p_nonce IS NOT NULL AND a.nonce = p_nonce)
        OR (p_state IS NOT NULL AND a.state = p_state))
    RETURNING a.nonce, a.pkce_verifier
$$;

-- Süresi dolmuş giriş kayıtlarını siler (worker, bağlamsız bakım işi).
CREATE FUNCTION purge_expired_auth_rows(p_older_than interval DEFAULT interval '1 day')
    RETURNS bigint
    LANGUAGE plpgsql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
DECLARE
    n bigint;
    m bigint;
BEGIN
    DELETE FROM email_login_tokens WHERE expires_at < now() - p_older_than;
    GET DIAGNOSTICS n = ROW_COUNT;
    DELETE FROM auth_requests WHERE expires_at < now() - p_older_than;
    GET DIAGNOSTICS m = ROW_COUNT;
    RETURN n + m;
END
$$;

REVOKE ALL ON FUNCTION
    find_user_by_identity(text, text),
    find_session_by_token_hash(bytea),
    email_login_token_create(uuid, bytea, text, timestamptz),
    email_login_token_consume(bytea),
    auth_request_create(uuid, text, text, text, timestamptz),
    auth_request_consume(text, text),
    purge_expired_auth_rows(interval)
    FROM PUBLIC;
GRANT EXECUTE ON FUNCTION
    find_user_by_identity(text, text),
    find_session_by_token_hash(bytea),
    email_login_token_create(uuid, bytea, text, timestamptz),
    email_login_token_consume(bytea),
    auth_request_create(uuid, text, text, text, timestamptz),
    auth_request_consume(text, text)
    TO app_api;
GRANT EXECUTE ON FUNCTION purge_expired_auth_rows(interval) TO app_worker;

RESET ROLE;
