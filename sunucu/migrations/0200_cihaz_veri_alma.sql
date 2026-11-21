-- Cihaz veri alma adresi (plan › Cihazlar; T2.25). ingest hizmeti anahtarı çalışma alanı
-- bilinmeden çözer: `devices` FORCE RLS altındadır, bu yüzden dar bir SECURITY DEFINER işlevi
-- anahtar özetinden cihazı bulur (aynı kalıp: 0030). İşlev yalnız özet bilen birine kimlik,
-- çalışma alanı, tür ve kalibrasyonu verir; anahtar özeti dışarı çıkmaz.
SET LOCAL ROLE app_owner;

CREATE POLICY owner_lookup ON devices TO app_owner USING (true) WITH CHECK (true);

CREATE FUNCTION find_device_by_key_hash(p_key_hash bytea)
    RETURNS TABLE (id uuid, workspace_id uuid, kind text, calibration jsonb)
    LANGUAGE sql STABLE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
    SELECT d.id, d.workspace_id, d.kind, d.calibration
    FROM devices d
    WHERE d.ingest_key_hash = p_key_hash
      AND d.deleted_at IS NULL
$$;

REVOKE ALL ON FUNCTION find_device_by_key_hash(bytea) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION find_device_by_key_hash(bytea) TO app_api;

RESET ROLE;
