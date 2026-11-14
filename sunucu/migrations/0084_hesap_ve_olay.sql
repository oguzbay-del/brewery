-- T2.9 (F2.1 bulgu 10, 12, 13, 31, 41): hesap günlüğü, push belirteci sahipliği, webhook
-- yükü saklaması, eski tarihli ölçüm bölümü, partisiz cihaz ölçümü.
SET LOCAL ROLE app_owner;

-- account_events (bulgu 41; karar 6) --------------------------------------------------------
-- Hesap düzeyi olaylar (giriş yöntemi ekleme/kaldırma, passkey, oturum kapatma, hesap
-- silme): çalışma alanı yoktur ya da birden çoktur, bu yüzden denetim zincirine (çalışma
-- alanı başına) değil kullanıcıya bağlı zincirsiz günlüğe yazılır. Yalnız eklenir; kişisel
-- veri taşımaz (ad ve e-posta yok, kullanıcı UUID'dir). RLS kullanıcı bağlamıyla.
CREATE TABLE account_events (
    id          uuid PRIMARY KEY,
    user_id     uuid NOT NULL REFERENCES users (id),
    kind        text NOT NULL CHECK (kind ~ '^[a-z][a-z0-9_.]*$' AND char_length(kind) <= 100),
    actor_kind  text NOT NULL CHECK (actor_kind IN ('user', 'support', 'system')),
    actor_id    uuid,
    session_id  uuid,
    details     jsonb NOT NULL DEFAULT '{"schema_version": 1}'
                CHECK (jsonb_typeof(details) = 'object' AND details ? 'schema_version'),
    created_at  timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX account_events_user_idx ON account_events (user_id, created_at DESC);
ALTER TABLE account_events ENABLE ROW LEVEL SECURITY;
ALTER TABLE account_events FORCE ROW LEVEL SECURITY;
CREATE POLICY own_rows ON account_events
    USING (user_id = (SELECT app_user_id())) WITH CHECK (user_id = (SELECT app_user_id()));
GRANT SELECT, INSERT ON account_events TO app_api, app_worker;

CREATE FUNCTION account_events_reject_change() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    RAISE EXCEPTION 'account_events is append-only (% rejected)', TG_OP
        USING ERRCODE = 'integrity_constraint_violation';
END
$$;
CREATE TRIGGER no_update_delete BEFORE UPDATE OR DELETE ON account_events
    FOR EACH ROW EXECUTE FUNCTION account_events_reject_change();
CREATE TRIGGER no_truncate BEFORE TRUNCATE ON account_events
    FOR EACH STATEMENT EXECUTE FUNCTION account_events_reject_change();

-- push_targets (bulgu 13) ----------------------------------------------------------------------
-- APNs belirteci cihaza aittir: aynı cihazda çıkış yapan A'nın satırı, giriş yapan B'nin
-- kaydını engellememeli. Tekillik kullanıcı başınadır; belirteci devralmayı dar işlev yapar.
ALTER TABLE push_targets DROP CONSTRAINT push_targets_environment_token_key;
ALTER TABLE push_targets
    ADD CONSTRAINT push_targets_user_environment_token_key UNIQUE (user_id, environment, token);
CREATE INDEX push_targets_token_idx ON push_targets (environment, token);
-- İşlev başka kullanıcıların satırını siler: yalnız onun için sahip rolüne açık politika.
CREATE POLICY owner_lookup ON push_targets TO app_owner USING (true) WITH CHECK (true);

-- Geçerli kullanıcı (bağlam) için belirteci kaydeder; aynı belirteci taşıyan öteki
-- kullanıcıların satırlarını siler (cihaz el değiştirdi). Satırın kimliğini döndürür.
CREATE FUNCTION claim_push_target(
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
        RAISE EXCEPTION 'claim_push_target needs a user context'
            USING ERRCODE = 'insufficient_privilege';
    END IF;
    DELETE FROM push_targets
    WHERE environment = p_environment AND token = p_token AND user_id <> uid;
    INSERT INTO push_targets (id, user_id, environment, token, app_version)
    VALUES (p_id, uid, p_environment, p_token, p_app_version)
    ON CONFLICT (user_id, environment, token)
        DO UPDATE SET app_version = excluded.app_version, last_error = NULL, last_error_at = NULL
    RETURNING id INTO result;
    RETURN result;
END
$$;
REVOKE ALL ON FUNCTION claim_push_target(uuid, text, text, text) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION claim_push_target(uuid, text, text, text) TO app_api;

-- external_events (bulgu 10) ------------------------------------------------------------------
-- Webhook yükü müşteri içeriği taşıyabilir (destek mesajı, cihaz ölçümü) ve çalışma alanı
-- bağlamsızdır: API okuyamaz (yalnız "bu teslimat görüldü mü" sorusu), işlenen satırın yükü
-- kısa sürede boşaltılır, satır sonunda silinir. Süreler "doğrulanacak" (adr/0007).
REVOKE SELECT ON external_events FROM app_api;
ALTER TABLE external_events ADD COLUMN payload_purged_at timestamptz;

CREATE FUNCTION external_event_seen(p_source text, p_delivery_id text) RETURNS boolean
    LANGUAGE sql STABLE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
    SELECT EXISTS (SELECT 1 FROM external_events WHERE source = p_source AND delivery_id = p_delivery_id)
$$;

-- İşlenmiş olayların yükünü boşaltır, eski işlenmiş satırları siler; etkilenen satır sayısı döner.
CREATE FUNCTION purge_external_events(
    p_blank_after interval DEFAULT interval '7 days',
    p_delete_after interval DEFAULT interval '90 days') RETURNS bigint
    LANGUAGE plpgsql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
DECLARE
    blanked bigint;
    deleted bigint;
BEGIN
    UPDATE external_events
    SET payload = '{"purged": true}', payload_purged_at = now()
    WHERE processed_at IS NOT NULL AND processed_at < now() - p_blank_after
      AND payload_purged_at IS NULL;
    GET DIAGNOSTICS blanked = ROW_COUNT;
    DELETE FROM external_events
    WHERE processed_at IS NOT NULL AND processed_at < now() - p_delete_after;
    GET DIAGNOSTICS deleted = ROW_COUNT;
    RETURN blanked + deleted;
END
$$;
REVOKE ALL ON FUNCTION external_event_seen(text, text), purge_external_events(interval, interval)
    FROM PUBLIC;
GRANT EXECUTE ON FUNCTION external_event_seen(text, text) TO app_api;
GRANT EXECUTE ON FUNCTION purge_external_events(interval, interval) TO app_worker;

-- measurements: eski bölüm ve partisiz ölçüm (bulgu 12, 31) ---------------------------------
-- 2000-01-01 ile 2026-01-01 arası tek "eski" bölüm (kâğıt kayıtların sonradan girişi);
-- 2000 öncesi (saati sıfırlanmış cihaz) ve bölümü açılmamış gelecek zaman 23514 ile düşer,
-- API bunu ölçüm başına `measured_at` aralık hatası olarak döndürür.
CREATE TABLE measurements_legacy PARTITION OF measurements
    FOR VALUES FROM ('2000-01-01 00:00:00+00') TO ('2026-01-01 00:00:00+00');
ALTER TABLE measurements_legacy ENABLE ROW LEVEL SECURITY;
ALTER TABLE measurements_legacy FORCE ROW LEVEL SECURITY;
CREATE POLICY workspace_isolation ON measurements_legacy
    USING (workspace_id = (SELECT app_workspace_id()))
    WITH CHECK (workspace_id = (SELECT app_workspace_id()));

-- Cihaz ölçümü partiye ya da tanka bağlıdır; ikisi de yoksa (boştaki cihaz) reddedilir
-- (F2.1 karar 5). Tank silinince başvuru boşalır; yalnız tanka bağlı ölçüm varsa tank
-- kalıcı silinmez (CHECK düşer): ölçüm başka başvuru taşımıyor.
ALTER TABLE measurements ALTER COLUMN batch_id DROP NOT NULL;
ALTER TABLE measurements ADD COLUMN vessel_id uuid;
ALTER TABLE measurements ADD CONSTRAINT measurements_target_check
    CHECK (num_nonnulls(batch_id, vessel_id) >= 1);
ALTER TABLE measurements ADD CONSTRAINT measurements_vessel_fkey
    FOREIGN KEY (workspace_id, vessel_id) REFERENCES vessels (workspace_id, id)
    ON DELETE SET NULL (vessel_id);
CREATE INDEX measurements_vessel_idx ON measurements (workspace_id, vessel_id, measured_at)
    WHERE vessel_id IS NOT NULL;

RESET ROLE;
