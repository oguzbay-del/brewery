-- Giriş ve oturum için ek girişten önceki işlevler (adr/0009 › 1.6, 3; T2.11).
--   * find_accounts_by_email: "aynı e-posta başka hesapta" ipucu için, kimlik yöntemlerini döndürür.
--   * find_session_by_token_hash: oturum süresi ve "son girişten bu yana" denetimi için remember
--     ve created_at eklenir (dönüş türü değiştiği için işlev yeniden yaratılır).
SET LOCAL ROLE app_owner;

-- users tablosu FORCE RLS altındadır; sahibe açık okuma politikası yalnız aşağıdaki
-- SECURITY DEFINER işlevinin okuyabilmesi içindir (uygulama rolleri sahip değildir).
CREATE POLICY owner_lookup ON users TO app_owner USING (true) WITH CHECK (true);

-- Doğrulanmış bu adresi iletişim adresi olarak taşıyan ya da bu adresle e-posta kimliği
-- olan hesapların giriş yöntemleri. Kullanıcı kimliği ve sağlayıcı adı döner
-- (arayan kendi hesabını dışarıda bırakır); hesap birleştirme yoktur.
CREATE FUNCTION find_accounts_by_email(p_email text)
    RETURNS TABLE (user_id uuid, provider text)
    LANGUAGE sql STABLE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
    SELECT DISTINCT i.user_id, i.provider
    FROM user_identities i
    WHERE i.user_id IN (
        SELECT u.id FROM users u
        WHERE u.contact_email = p_email
          AND u.contact_email_verified_at IS NOT NULL
          AND u.deleted_at IS NULL
        UNION
        SELECT e.user_id FROM user_identities e
        WHERE e.provider = 'email' AND e.subject = p_email)
$$;

DROP FUNCTION find_session_by_token_hash(bytea);
CREATE FUNCTION find_session_by_token_hash(p_token_hash bytea)
    RETURNS TABLE (
        id uuid, user_id uuid, client_kind text, shared_device_id uuid, remember boolean,
        created_at timestamptz, last_activity_at timestamptz, expires_at timestamptz,
        step_up_at timestamptz, revoked_at timestamptz)
    LANGUAGE sql STABLE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
    SELECT s.id, s.user_id, s.client_kind, s.shared_device_id, s.remember,
           s.created_at, s.last_activity_at, s.expires_at, s.step_up_at, s.revoked_at
    FROM sessions s
    WHERE s.token_hash = p_token_hash
$$;

REVOKE ALL ON FUNCTION find_accounts_by_email(text), find_session_by_token_hash(bytea)
    FROM PUBLIC;
GRANT EXECUTE ON FUNCTION find_accounts_by_email(text), find_session_by_token_hash(bytea)
    TO app_api;

RESET ROLE;
