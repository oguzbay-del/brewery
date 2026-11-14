-- F2.1 düzeltmeleri: işlenmiş olay yükü (bulgu 10), idempotency işaretçisi (bulgu 4), eski
-- tarihli ölçümler (bulgu 12), push belirteci devri (bulgu 13).
SET LOCAL ROLE app_owner;

-- Bulgu 10: external_events.payload müşteri içeriği taşıyabilir. API yükü okuyamaz (yalnız
-- tekrar denetimi için kimlik sütunlarını); işlenen satırın yükü boşalır.
REVOKE SELECT ON external_events FROM app_api;
GRANT SELECT (id, source, delivery_id, received_at, processed_at) ON external_events TO app_api;

CREATE FUNCTION external_events_blank_payload() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF OLD.processed_at IS NULL AND NEW.processed_at IS NOT NULL THEN
        NEW.payload := '{}'::jsonb;
    END IF;
    RETURN NEW;
END
$$;
CREATE TRIGGER blank_payload BEFORE UPDATE OF processed_at ON external_events
    FOR EACH ROW EXECUTE FUNCTION external_events_blank_payload();

-- Bulgu 4: iş işlemi kesinleşince anahtar satırı işaretlenir (db::begin, aynı işlemde). Yanıt
-- yazılmadan süreç düşerse devralma, işi yeniden çalıştırmaz.
ALTER TABLE idempotency_keys ADD COLUMN work_committed_at timestamptz;

-- Bulgu 12: 2026-01 öncesi için tek "eski" bölüm. Üst sınırı (gelecek) ve alt sınırı
-- (bozuk saat) API'de `measured_at` aralık denetimi koyar (db::validate_measured_at).
CREATE TABLE measurements_legacy PARTITION OF measurements
    FOR VALUES FROM (MINVALUE) TO ('2026-01-01 00:00:00+00');
ALTER TABLE measurements_legacy ENABLE ROW LEVEL SECURITY;
ALTER TABLE measurements_legacy FORCE ROW LEVEL SECURITY;
CREATE POLICY workspace_isolation ON measurements_legacy
    USING (workspace_id = (SELECT app_workspace_id()))
    WITH CHECK (workspace_id = (SELECT app_workspace_id()));

-- Bulgu 13: APNs belirteci cihaza aittir. Tekillik kullanıcı başına; girişte belirteci devralan
-- dar işlev eski sahibin satırını siler.
ALTER TABLE push_targets DROP CONSTRAINT push_targets_environment_token_key;
ALTER TABLE push_targets ADD CONSTRAINT push_targets_user_environment_token_key
    UNIQUE (user_id, environment, token);
CREATE INDEX push_targets_token_idx ON push_targets (environment, token);
CREATE POLICY owner_takeover ON push_targets TO app_owner USING (true) WITH CHECK (true);

CREATE FUNCTION register_push_target(
    p_id uuid, p_environment text, p_token text, p_app_version text DEFAULT NULL)
    RETURNS uuid
    LANGUAGE plpgsql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
DECLARE
    uid uuid := app_user_id();
    result uuid;
BEGIN
    IF uid IS NULL THEN
        RAISE EXCEPTION 'register_push_target needs a user context'
            USING ERRCODE = 'insufficient_privilege';
    END IF;
    DELETE FROM push_targets
    WHERE environment = p_environment AND token = p_token AND user_id <> uid;
    INSERT INTO push_targets (id, user_id, environment, token, app_version)
    VALUES (p_id, uid, p_environment, p_token, p_app_version)
    ON CONFLICT (user_id, environment, token) DO UPDATE SET app_version = EXCLUDED.app_version
    RETURNING id INTO result;
    RETURN result;
END
$$;
REVOKE ALL ON FUNCTION register_push_target(uuid, text, text, text) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION register_push_target(uuid, text, text, text) TO app_api;

RESET ROLE;
