-- M2: OAuth 2.1 yetkilendirme sunucusu (adr/0015-mcp-ve-oauth.md › 2.8).
--
-- Tablolar: oauth_clients (CIMD önbelleği ve DCR kayıtları), oauth_requests (onay ekranına
-- giden bekleyen yetkilendirme isteği), oauth_grants (kullanıcı, çalışma alanı, istemci,
-- kapsamlar, kaynak), oauth_codes (60 sn, tek kullanımlık), oauth_tokens (yalnız SHA-256
-- özeti). Kod, istek ve belirteç tabloları uygulama rollerine kapalıdır; yalnız dar
-- SECURITY DEFINER işlevlerle erişilir (auth_requests gibi). oauth_grants'i kullanıcı kendi
-- bağlamında okur (Bağlı uygulamalar); oauth_clients herkese açık bilgi taşır ve okunur.
SET LOCAL ROLE app_owner;

-- İşletme yöneticisinin MCP kapatma anahtarı (adr/0015 › 3.9).
ALTER TABLE workspaces ADD COLUMN mcp_enabled boolean NOT NULL DEFAULT true;

CREATE TABLE oauth_clients (
    id                          uuid PRIMARY KEY,
    client_id                   text NOT NULL UNIQUE CHECK (char_length(client_id) BETWEEN 1 AND 2048),
    kind                        text NOT NULL CHECK (kind IN ('cimd', 'dcr', 'preregistered')),
    client_name                 text NOT NULL CHECK (char_length(client_name) BETWEEN 1 AND 200),
    logo_uri                    text CHECK (char_length(logo_uri) <= 2048),
    client_uri                  text CHECK (char_length(client_uri) <= 2048),
    redirect_uris               text[] NOT NULL CHECK (cardinality(redirect_uris) BETWEEN 1 AND 20),
    token_endpoint_auth_method  text NOT NULL DEFAULT 'none'
                                CHECK (token_endpoint_auth_method IN ('none', 'private_key_jwt')),
    -- CIMD belge önbelleğinin sonu (en çok 24 saat).
    metadata_expires_at         timestamptz,
    metadata_fetched_at         timestamptz,
    last_consented_at           timestamptz,
    created_at                  timestamptz NOT NULL DEFAULT now(),
    updated_at                  timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE oauth_grants (
    id             uuid PRIMARY KEY,
    user_id        uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    workspace_id   uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    client_id      uuid NOT NULL REFERENCES oauth_clients (id) ON DELETE CASCADE,
    scopes         text[] NOT NULL CHECK (cardinality(scopes) >= 1),
    resource       text NOT NULL CHECK (char_length(resource) BETWEEN 1 AND 2048),
    created_at     timestamptz NOT NULL DEFAULT now(),
    updated_at     timestamptz NOT NULL DEFAULT now(),
    last_used_at   timestamptz,
    revoked_at     timestamptz,
    revoked_reason text CHECK (char_length(revoked_reason) <= 100)
);
-- Aynı (kullanıcı, çalışma alanı, istemci, kaynak) için tek etkin onay; yeni onay onu genişletir.
CREATE UNIQUE INDEX oauth_grants_active_idx
    ON oauth_grants (user_id, workspace_id, client_id, resource) WHERE revoked_at IS NULL;
CREATE INDEX oauth_grants_workspace_idx ON oauth_grants (workspace_id) WHERE revoked_at IS NULL;
CREATE INDEX oauth_grants_client_idx ON oauth_grants (client_id);

CREATE TABLE oauth_requests (
    id              uuid PRIMARY KEY,
    request_hash    bytea NOT NULL UNIQUE CHECK (octet_length(request_hash) = 32),
    client_id       uuid NOT NULL REFERENCES oauth_clients (id) ON DELETE CASCADE,
    redirect_uri    text NOT NULL,
    scopes          text[] NOT NULL,
    state           text NOT NULL,
    code_challenge  text NOT NULL CHECK (char_length(code_challenge) = 43),
    resource        text NOT NULL,
    expires_at      timestamptz NOT NULL,
    consumed_at     timestamptz,
    created_at      timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE oauth_codes (
    id              uuid PRIMARY KEY,
    code_hash       bytea NOT NULL UNIQUE CHECK (octet_length(code_hash) = 32),
    grant_id        uuid NOT NULL REFERENCES oauth_grants (id) ON DELETE CASCADE,
    client_id       uuid NOT NULL REFERENCES oauth_clients (id) ON DELETE CASCADE,
    redirect_uri    text NOT NULL,
    code_challenge  text NOT NULL CHECK (char_length(code_challenge) = 43),
    scopes          text[] NOT NULL,
    resource        text NOT NULL,
    expires_at      timestamptz NOT NULL,
    consumed_at     timestamptz,
    created_at      timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE oauth_tokens (
    id          uuid PRIMARY KEY,
    grant_id    uuid NOT NULL REFERENCES oauth_grants (id) ON DELETE CASCADE,
    kind        text NOT NULL CHECK (kind IN ('access', 'refresh')),
    token_hash  bytea NOT NULL UNIQUE CHECK (octet_length(token_hash) = 32),
    -- Yenileme zinciri: grant kimliği (bir grant bir aile).
    family_id   uuid NOT NULL,
    parent_id   uuid,
    scopes      text[] NOT NULL,
    expires_at  timestamptz NOT NULL,
    -- Döndürülen yenileme belirteci kullanılmış sayılır; yeniden gelirse zincir iptal olur.
    used_at     timestamptz,
    revoked_at  timestamptz,
    created_at  timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX oauth_tokens_grant_idx ON oauth_tokens (grant_id);

ALTER TABLE oauth_clients  ENABLE ROW LEVEL SECURITY;
ALTER TABLE oauth_clients  FORCE ROW LEVEL SECURITY;
ALTER TABLE oauth_grants   ENABLE ROW LEVEL SECURITY;
ALTER TABLE oauth_grants   FORCE ROW LEVEL SECURITY;
ALTER TABLE oauth_requests ENABLE ROW LEVEL SECURITY;
ALTER TABLE oauth_requests FORCE ROW LEVEL SECURITY;
ALTER TABLE oauth_codes    ENABLE ROW LEVEL SECURITY;
ALTER TABLE oauth_codes    FORCE ROW LEVEL SECURITY;
ALTER TABLE oauth_tokens   ENABLE ROW LEVEL SECURITY;
ALTER TABLE oauth_tokens   FORCE ROW LEVEL SECURITY;

-- İstemci bilgisi (ad, logo, yönlendirme adresleri) onay ekranında gösterilir; herkese açıktır.
CREATE POLICY read_all ON oauth_clients FOR SELECT USING (true);
CREATE POLICY own_rows ON oauth_grants
    USING (user_id = (SELECT app_user_id())) WITH CHECK (user_id = (SELECT app_user_id()));
CREATE POLICY owner_lookup ON oauth_clients  TO app_owner USING (true) WITH CHECK (true);
CREATE POLICY owner_lookup ON oauth_grants   TO app_owner USING (true) WITH CHECK (true);
CREATE POLICY owner_lookup ON oauth_requests TO app_owner USING (true) WITH CHECK (true);
CREATE POLICY owner_lookup ON oauth_codes    TO app_owner USING (true) WITH CHECK (true);
CREATE POLICY owner_lookup ON oauth_tokens   TO app_owner USING (true) WITH CHECK (true);

-- Haklar: API istemciyi ve kendi onaylarını okur; bütün yazma işlevlerle yapılır.
GRANT SELECT ON oauth_clients, oauth_grants TO app_api;

-- İşlevler ------------------------------------------------------------------------------

-- İstemciyi kaydeder ya da günceller (CIMD belgesi yenilenince, DCR kaydında).
CREATE FUNCTION oauth_client_upsert(
    p_id uuid, p_client_id text, p_kind text, p_name text, p_logo_uri text, p_client_uri text,
    p_redirect_uris text[], p_auth_method text, p_metadata_expires_at timestamptz)
    RETURNS uuid
    LANGUAGE sql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
    INSERT INTO oauth_clients (id, client_id, kind, client_name, logo_uri, client_uri,
                               redirect_uris, token_endpoint_auth_method,
                               metadata_expires_at, metadata_fetched_at)
    VALUES (p_id, p_client_id, p_kind, p_name, p_logo_uri, p_client_uri, p_redirect_uris,
            p_auth_method, p_metadata_expires_at,
            CASE WHEN p_kind = 'cimd' THEN now() END)
    ON CONFLICT (client_id) DO UPDATE
    SET client_name = EXCLUDED.client_name, logo_uri = EXCLUDED.logo_uri,
        client_uri = EXCLUDED.client_uri, redirect_uris = EXCLUDED.redirect_uris,
        token_endpoint_auth_method = EXCLUDED.token_endpoint_auth_method,
        metadata_expires_at = EXCLUDED.metadata_expires_at,
        metadata_fetched_at = EXCLUDED.metadata_fetched_at,
        updated_at = now()
    RETURNING id
$$;

-- Bekleyen yetkilendirme isteği (onay ekranının kısa ömürlü kimliği).
CREATE FUNCTION oauth_request_create(
    p_id uuid, p_hash bytea, p_client uuid, p_redirect_uri text, p_scopes text[],
    p_state text, p_challenge text, p_resource text, p_ttl_secs integer)
    RETURNS void
    LANGUAGE sql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
    INSERT INTO oauth_requests (id, request_hash, client_id, redirect_uri, scopes, state,
                                code_challenge, resource, expires_at)
    VALUES (p_id, p_hash, p_client, p_redirect_uri, p_scopes, p_state, p_challenge, p_resource,
            now() + make_interval(secs => p_ttl_secs))
$$;

CREATE FUNCTION oauth_request_get(p_hash bytea)
    RETURNS TABLE (client_pk uuid, redirect_uri text, scopes text[], state text,
                   code_challenge text, resource text, expires_at timestamptz)
    LANGUAGE sql STABLE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
    SELECT r.client_id, r.redirect_uri, r.scopes, r.state, r.code_challenge, r.resource,
           r.expires_at
    FROM oauth_requests r
    WHERE r.request_hash = p_hash AND r.consumed_at IS NULL AND r.expires_at > now()
$$;

-- Tek kullanımlık: kararı işaretler ve satırı döndürür.
CREATE FUNCTION oauth_request_consume(p_hash bytea)
    RETURNS TABLE (client_pk uuid, redirect_uri text, scopes text[], state text,
                   code_challenge text, resource text)
    LANGUAGE sql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
    UPDATE oauth_requests r
    SET consumed_at = now()
    WHERE r.request_hash = p_hash AND r.consumed_at IS NULL AND r.expires_at > now()
    RETURNING r.client_id, r.redirect_uri, r.scopes, r.state, r.code_challenge, r.resource
$$;

-- Onay verir: aynı (kullanıcı, çalışma alanı, istemci, kaynak) için etkin onay varsa kapsamları
-- birleştirir (adr/0015 › 8.4), yoksa yeni onay açar. widened: mevcut onay genişledi.
CREATE FUNCTION oauth_grant_upsert(
    p_id uuid, p_user uuid, p_workspace uuid, p_client uuid, p_scopes text[], p_resource text)
    RETURNS TABLE (grant_id uuid, scopes text[], widened boolean)
    LANGUAGE plpgsql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
DECLARE
    existing oauth_grants%ROWTYPE;
    merged text[];
BEGIN
    SELECT * INTO existing FROM oauth_grants g
    WHERE g.user_id = p_user AND g.workspace_id = p_workspace AND g.client_id = p_client
      AND g.resource = p_resource AND g.revoked_at IS NULL
    FOR UPDATE;
    IF FOUND THEN
        SELECT array_agg(DISTINCT s ORDER BY s) INTO merged
        FROM unnest(existing.scopes || p_scopes) s;
        UPDATE oauth_grants g SET scopes = merged, updated_at = now() WHERE g.id = existing.id;
        UPDATE oauth_clients c SET last_consented_at = now() WHERE c.id = p_client;
        RETURN QUERY SELECT existing.id, merged, merged <> existing.scopes;
    ELSE
        SELECT array_agg(DISTINCT s ORDER BY s) INTO merged FROM unnest(p_scopes) s;
        INSERT INTO oauth_grants (id, user_id, workspace_id, client_id, scopes, resource)
        VALUES (p_id, p_user, p_workspace, p_client, merged, p_resource);
        UPDATE oauth_clients c SET last_consented_at = now() WHERE c.id = p_client;
        RETURN QUERY SELECT p_id, merged, false;
    END IF;
END
$$;

CREATE FUNCTION oauth_code_create(
    p_id uuid, p_hash bytea, p_grant uuid, p_client uuid, p_redirect_uri text,
    p_challenge text, p_scopes text[], p_resource text, p_ttl_secs integer)
    RETURNS void
    LANGUAGE sql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
    INSERT INTO oauth_codes (id, code_hash, grant_id, client_id, redirect_uri, code_challenge,
                             scopes, resource, expires_at)
    VALUES (p_id, p_hash, p_grant, p_client, p_redirect_uri, p_challenge, p_scopes, p_resource,
            now() + make_interval(secs => p_ttl_secs))
$$;

-- Kodu tüketir. Kod daha önce harcanmışsa replayed = true döner ve onayın bütün belirteçleri
-- iptal edilir (OAuth 2.1 › 4.1.3). Süresi dolmuş ya da bilinmeyen kod: satır yok.
CREATE FUNCTION oauth_code_consume(p_hash bytea)
    RETURNS TABLE (grant_id uuid, client_pk uuid, redirect_uri text, code_challenge text,
                   scopes text[], resource text, replayed boolean)
    LANGUAGE plpgsql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
DECLARE
    c oauth_codes%ROWTYPE;
BEGIN
    SELECT * INTO c FROM oauth_codes x WHERE x.code_hash = p_hash FOR UPDATE;
    IF NOT FOUND THEN
        RETURN;
    END IF;
    IF c.consumed_at IS NOT NULL THEN
        UPDATE oauth_grants g SET revoked_at = now(), revoked_reason = 'code_replay'
        WHERE g.id = c.grant_id AND g.revoked_at IS NULL;
        UPDATE oauth_tokens t SET revoked_at = now()
        WHERE t.grant_id = c.grant_id AND t.revoked_at IS NULL;
        RETURN QUERY SELECT c.grant_id, c.client_id, c.redirect_uri, c.code_challenge,
                            c.scopes, c.resource, true;
        RETURN;
    END IF;
    IF c.expires_at <= now() THEN
        RETURN;
    END IF;
    UPDATE oauth_codes x SET consumed_at = now() WHERE x.id = c.id;
    RETURN QUERY SELECT c.grant_id, c.client_id, c.redirect_uri, c.code_challenge,
                        c.scopes, c.resource, false;
END
$$;

-- Erişim ve yenileme belirtecini birlikte açar (yalnız etkin onay için).
CREATE FUNCTION oauth_token_issue(
    p_grant uuid, p_scopes text[], p_access_hash bytea, p_refresh_hash bytea,
    p_access_ttl_secs integer, p_refresh_ttl_secs integer)
    RETURNS boolean
    LANGUAGE plpgsql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
DECLARE
    active boolean;
BEGIN
    SELECT g.revoked_at IS NULL INTO active FROM oauth_grants g WHERE g.id = p_grant FOR UPDATE;
    IF NOT COALESCE(active, false) THEN
        RETURN false;
    END IF;
    INSERT INTO oauth_tokens (id, grant_id, kind, token_hash, family_id, scopes, expires_at)
    VALUES (gen_random_uuid(), p_grant, 'access', p_access_hash, p_grant, p_scopes,
            now() + make_interval(secs => p_access_ttl_secs)),
           (gen_random_uuid(), p_grant, 'refresh', p_refresh_hash, p_grant, p_scopes,
            now() + make_interval(secs => p_refresh_ttl_secs));
    UPDATE oauth_grants g SET last_used_at = now() WHERE g.id = p_grant;
    RETURN true;
END
$$;

-- Yenileme belirtecini döndürür (adr/0015 › 2.6). status: ok | invalid | invalid_scope | reuse.
-- reuse: kullanılmış belirteç yeniden geldi; onayın bütün belirteçleri iptal edildi.
CREATE FUNCTION oauth_refresh_rotate(
    p_old_hash bytea, p_client uuid, p_requested_scopes text[],
    p_new_access_hash bytea, p_new_refresh_hash bytea,
    p_access_ttl_secs integer, p_refresh_ttl_secs integer)
    RETURNS TABLE (status text, grant_id uuid, user_id uuid, workspace_id uuid,
                   scopes text[], resource text)
    LANGUAGE plpgsql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
DECLARE
    t oauth_tokens%ROWTYPE;
    g oauth_grants%ROWTYPE;
    narrowed text[];
BEGIN
    SELECT * INTO t FROM oauth_tokens x WHERE x.token_hash = p_old_hash AND x.kind = 'refresh'
    FOR UPDATE;
    IF NOT FOUND THEN
        RETURN QUERY SELECT 'invalid', NULL::uuid, NULL::uuid, NULL::uuid, NULL::text[], NULL::text;
        RETURN;
    END IF;
    SELECT * INTO g FROM oauth_grants x WHERE x.id = t.grant_id FOR UPDATE;
    IF g.client_id <> p_client THEN
        RETURN QUERY SELECT 'invalid', NULL::uuid, NULL::uuid, NULL::uuid, NULL::text[], NULL::text;
        RETURN;
    END IF;
    IF t.used_at IS NOT NULL AND g.revoked_at IS NULL THEN
        UPDATE oauth_grants x SET revoked_at = now(), revoked_reason = 'refresh_reuse'
        WHERE x.id = g.id;
        UPDATE oauth_tokens x SET revoked_at = now()
        WHERE x.grant_id = g.id AND x.revoked_at IS NULL;
        RETURN QUERY SELECT 'reuse', g.id, g.user_id, g.workspace_id, g.scopes, g.resource;
        RETURN;
    END IF;
    IF t.used_at IS NOT NULL OR t.revoked_at IS NOT NULL OR g.revoked_at IS NOT NULL
       OR t.expires_at <= now() THEN
        RETURN QUERY SELECT 'invalid', NULL::uuid, NULL::uuid, NULL::uuid, NULL::text[], NULL::text;
        RETURN;
    END IF;
    -- Kapsam daraltılabilir, genişletilemez.
    IF p_requested_scopes IS NULL THEN
        narrowed := t.scopes;
    ELSIF p_requested_scopes <@ t.scopes AND cardinality(p_requested_scopes) >= 1 THEN
        narrowed := p_requested_scopes;
    ELSE
        RETURN QUERY SELECT 'invalid_scope', NULL::uuid, NULL::uuid, NULL::uuid, NULL::text[],
                            NULL::text;
        RETURN;
    END IF;
    UPDATE oauth_tokens x SET used_at = now() WHERE x.id = t.id;
    INSERT INTO oauth_tokens (id, grant_id, kind, token_hash, family_id, parent_id, scopes,
                              expires_at)
    VALUES (gen_random_uuid(), g.id, 'access', p_new_access_hash, t.family_id, t.id, narrowed,
            now() + make_interval(secs => p_access_ttl_secs)),
           (gen_random_uuid(), g.id, 'refresh', p_new_refresh_hash, t.family_id, t.id, narrowed,
            now() + make_interval(secs => p_refresh_ttl_secs));
    UPDATE oauth_grants x SET last_used_at = now() WHERE x.id = g.id;
    RETURN QUERY SELECT 'ok', g.id, g.user_id, g.workspace_id, narrowed, g.resource;
END
$$;

-- Belirteç doğrulaması: kaydı döndürür. usable: iptal edilmemiş, süresi dolmamış, kullanılmamış
-- ve onayı açık. Son kullanım zamanı en çok dakikada bir yazılır (adr/0015 › 5.5).
CREATE FUNCTION oauth_token_lookup(p_hash bytea)
    RETURNS TABLE (token_id uuid, kind text, grant_id uuid, user_id uuid, workspace_id uuid,
                   client_pk uuid, client_id text, client_name text, scopes text[],
                   resource text, expires_at timestamptz, usable boolean)
    LANGUAGE plpgsql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
DECLARE
    r record;
    ok boolean;
BEGIN
    SELECT t.id AS token_id, t.kind, t.grant_id, g.user_id, g.workspace_id,
           g.client_id AS client_pk, c.client_id, c.client_name, t.scopes, g.resource,
           t.expires_at,
           (t.revoked_at IS NULL AND t.used_at IS NULL AND t.expires_at > now()
            AND g.revoked_at IS NULL) AS usable,
           g.last_used_at
    INTO r
    FROM oauth_tokens t
    JOIN oauth_grants g ON g.id = t.grant_id
    JOIN oauth_clients c ON c.id = g.client_id
    WHERE t.token_hash = p_hash;
    IF NOT FOUND THEN
        RETURN;
    END IF;
    ok := r.usable;
    IF ok AND r.kind = 'access'
       AND (r.last_used_at IS NULL OR r.last_used_at < now() - interval '1 minute') THEN
        UPDATE oauth_grants g SET last_used_at = now() WHERE g.id = r.grant_id;
    END IF;
    RETURN QUERY SELECT r.token_id, r.kind, r.grant_id, r.user_id, r.workspace_id, r.client_pk,
                        r.client_id, r.client_name, r.scopes, r.resource, r.expires_at, ok;
END
$$;

-- RFC 7009: yenileme belirtecini iptal etmek bütün onayı kapatır, erişim belirteci yalnız kendini.
-- İstemci kendi belirtecini iptal eder; başkasının belirteci sessizce yok sayılır.
CREATE FUNCTION oauth_token_revoke(p_hash bytea, p_client uuid)
    RETURNS TABLE (grant_id uuid, workspace_id uuid, user_id uuid, whole_grant boolean)
    LANGUAGE plpgsql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
DECLARE
    t oauth_tokens%ROWTYPE;
    g oauth_grants%ROWTYPE;
BEGIN
    SELECT * INTO t FROM oauth_tokens x WHERE x.token_hash = p_hash FOR UPDATE;
    IF NOT FOUND THEN
        RETURN;
    END IF;
    SELECT * INTO g FROM oauth_grants x WHERE x.id = t.grant_id FOR UPDATE;
    IF g.client_id <> p_client THEN
        RETURN;
    END IF;
    IF t.kind = 'refresh' THEN
        IF g.revoked_at IS NULL THEN
            UPDATE oauth_grants x SET revoked_at = now(), revoked_reason = 'client_revoked'
            WHERE x.id = g.id;
            UPDATE oauth_tokens x SET revoked_at = now()
            WHERE x.grant_id = g.id AND x.revoked_at IS NULL;
            RETURN QUERY SELECT g.id, g.workspace_id, g.user_id, true;
        END IF;
    ELSE
        UPDATE oauth_tokens x SET revoked_at = now() WHERE x.id = t.id AND x.revoked_at IS NULL;
    END IF;
END
$$;

-- Kullanıcının kendi onayını kaldırması (Bağlı uygulamalar). Başkasının onayı: satır yok.
CREATE FUNCTION oauth_grant_revoke(p_grant uuid, p_user uuid, p_reason text)
    RETURNS TABLE (grant_id uuid, workspace_id uuid, client_pk uuid)
    LANGUAGE sql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
    WITH g AS (
        UPDATE oauth_grants x SET revoked_at = now(), revoked_reason = p_reason
        WHERE x.id = p_grant AND x.user_id = p_user AND x.revoked_at IS NULL
        RETURNING x.id, x.workspace_id, x.client_id
    ), t AS (
        UPDATE oauth_tokens x SET revoked_at = now()
        WHERE x.grant_id IN (SELECT id FROM g) AND x.revoked_at IS NULL
        RETURNING 1
    )
    SELECT g.id, g.workspace_id, g.client_id FROM g
$$;

-- "Hepsini kaldır" ve hesap silme: kullanıcının bütün etkin onayları.
CREATE FUNCTION oauth_grant_revoke_all_user(p_user uuid, p_reason text)
    RETURNS TABLE (grant_id uuid, workspace_id uuid, client_pk uuid)
    LANGUAGE sql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
    WITH g AS (
        UPDATE oauth_grants x SET revoked_at = now(), revoked_reason = p_reason
        WHERE x.user_id = p_user AND x.revoked_at IS NULL
        RETURNING x.id, x.workspace_id, x.client_id
    ), t AS (
        UPDATE oauth_tokens x SET revoked_at = now()
        WHERE x.grant_id IN (SELECT id FROM g) AND x.revoked_at IS NULL
        RETURNING 1
    )
    SELECT g.id, g.workspace_id, g.client_id FROM g
$$;

-- Çalışma alanı kapatma anahtarı (adr/0015 › 3.9): o kaynağa bağlı bütün onaylar kapanır.
CREATE FUNCTION oauth_workspace_revoke_all(p_workspace uuid, p_resource text, p_reason text)
    RETURNS TABLE (grant_id uuid, user_id uuid, client_pk uuid)
    LANGUAGE sql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
    WITH g AS (
        UPDATE oauth_grants x SET revoked_at = now(), revoked_reason = p_reason
        WHERE x.workspace_id = p_workspace AND x.resource = p_resource AND x.revoked_at IS NULL
        RETURNING x.id, x.user_id, x.client_id
    ), t AS (
        UPDATE oauth_tokens x SET revoked_at = now()
        WHERE x.grant_id IN (SELECT id FROM g) AND x.revoked_at IS NULL
        RETURNING 1
    )
    SELECT g.id, g.user_id, g.client_id FROM g
$$;

-- Bakım (worker): süresi dolmuş kod ve istekler, eski belirteçler, 30 günde hiç onay almamış
-- DCR kayıtları (adr/0015 › 2.3).
CREATE FUNCTION oauth_purge_expired(p_older_than interval DEFAULT interval '1 day')
    RETURNS bigint
    LANGUAGE plpgsql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
DECLARE
    n bigint := 0;
    m bigint;
BEGIN
    DELETE FROM oauth_requests WHERE expires_at < now() - p_older_than;
    GET DIAGNOSTICS m = ROW_COUNT; n := n + m;
    DELETE FROM oauth_codes WHERE expires_at < now() - p_older_than;
    GET DIAGNOSTICS m = ROW_COUNT; n := n + m;
    DELETE FROM oauth_tokens WHERE expires_at < now() - interval '90 days';
    GET DIAGNOSTICS m = ROW_COUNT; n := n + m;
    DELETE FROM oauth_clients c
    WHERE c.kind = 'dcr' AND c.last_consented_at IS NULL
      AND c.created_at < now() - interval '30 days'
      AND NOT EXISTS (SELECT 1 FROM oauth_grants g WHERE g.client_id = c.id);
    GET DIAGNOSTICS m = ROW_COUNT; n := n + m;
    RETURN n;
END
$$;

REVOKE ALL ON FUNCTION
    oauth_client_upsert(uuid, text, text, text, text, text, text[], text, timestamptz),
    oauth_request_create(uuid, bytea, uuid, text, text[], text, text, text, integer),
    oauth_request_get(bytea),
    oauth_request_consume(bytea),
    oauth_grant_upsert(uuid, uuid, uuid, uuid, text[], text),
    oauth_code_create(uuid, bytea, uuid, uuid, text, text, text[], text, integer),
    oauth_code_consume(bytea),
    oauth_token_issue(uuid, text[], bytea, bytea, integer, integer),
    oauth_refresh_rotate(bytea, uuid, text[], bytea, bytea, integer, integer),
    oauth_token_lookup(bytea),
    oauth_token_revoke(bytea, uuid),
    oauth_grant_revoke(uuid, uuid, text),
    oauth_grant_revoke_all_user(uuid, text),
    oauth_workspace_revoke_all(uuid, text, text),
    oauth_purge_expired(interval)
    FROM PUBLIC;
GRANT EXECUTE ON FUNCTION
    oauth_client_upsert(uuid, text, text, text, text, text, text[], text, timestamptz),
    oauth_request_create(uuid, bytea, uuid, text, text[], text, text, text, integer),
    oauth_request_get(bytea),
    oauth_request_consume(bytea),
    oauth_grant_upsert(uuid, uuid, uuid, uuid, text[], text),
    oauth_code_create(uuid, bytea, uuid, uuid, text, text, text[], text, integer),
    oauth_code_consume(bytea),
    oauth_token_issue(uuid, text[], bytea, bytea, integer, integer),
    oauth_refresh_rotate(bytea, uuid, text[], bytea, bytea, integer, integer),
    oauth_token_lookup(bytea),
    oauth_token_revoke(bytea, uuid),
    oauth_grant_revoke(uuid, uuid, text),
    oauth_grant_revoke_all_user(uuid, text),
    oauth_workspace_revoke_all(uuid, text, text)
    TO app_api;
GRANT EXECUTE ON FUNCTION oauth_purge_expired(interval) TO app_worker;

RESET ROLE;
