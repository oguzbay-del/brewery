-- T2.9 (F2.1 bulgu 11, 34): İşletmede zorunlu saklama. Saklama bitişi ve yasal tutma
-- alanları ilgili İşletme tablolarına; bitişi gelmemiş ya da süresiz tutmalı kayıt silinemez
-- (çalışma alanı silme zinciri dahil). Süreler `retention_rules` yapılandırmasındadır ve
-- doğrulanana dek `unverified` (Aşama 6'da kesinleşir; adr/0007 › T2.9 düzeltmeleri).
SET LOCAL ROLE app_owner;

-- Yapılandırma: bölge × kayıt türü → yıl. Ortak referans veridir (vergi tabloları gibi
-- çalışma alanı ve RLS yok, uygulama rollerine yalnız SELECT).
CREATE TABLE retention_rules (
    region               text NOT NULL CHECK (region IN ('us', 'uk')),
    record_kind          text NOT NULL CHECK (record_kind ~ '^[a-z][a-z0-9_]*$'),
    retention_years      integer NOT NULL CHECK (retention_years BETWEEN 1 AND 100),
    verification_status  text NOT NULL DEFAULT 'unverified'
                         CHECK (verification_status IN ('unverified', 'verified')),
    source_note          text NOT NULL,
    PRIMARY KEY (region, record_kind)
);
GRANT SELECT ON retention_rules TO app_api, app_worker;

-- Rakamlar doğrulanacak: ABD (TTB, bira üreticisi kayıtları) ve İngiltere (HMRC, bira
-- vergisi kayıtları) için resmî kaynaktan teyit edilmedi.
INSERT INTO retention_rules (region, record_kind, retention_years, source_note)
SELECT r.region, k.record_kind, r.years, r.note
FROM (VALUES ('us', 3, 'TTB brewer records (doğrulanacak)'),
             ('uk', 6, 'HMRC excise records (doğrulanacak)')) r(region, years, note)
CROSS JOIN (VALUES ('batches'), ('transfers'), ('packaging_runs'), ('lab_results'),
                   ('releases'), ('cleaning_records'), ('vessel_occupancies')) k(record_kind);

-- Kayıt tarihinden itibaren varsayılan saklama bitişi; kayıt kapanınca uç tarihi ileri
-- taşıyabilir. Kural yoksa NULL.
CREATE FUNCTION default_retention_until(p_region text, p_record_kind text, p_from timestamptz)
    RETURNS timestamptz
    LANGUAGE sql STABLE
    AS $$
    SELECT p_from + make_interval(years => r.retention_years)
    FROM retention_rules r WHERE r.region = p_region AND r.record_kind = p_record_kind
$$;
GRANT EXECUTE ON FUNCTION default_retention_until(text, text, timestamptz) TO app_api, app_worker;
REVOKE ALL ON FUNCTION default_retention_until(text, text, timestamptz) FROM PUBLIC;

-- Alanlar ---------------------------------------------------------------------------------
ALTER TABLE batches            ADD COLUMN retention_until timestamptz, ADD COLUMN legal_hold boolean NOT NULL DEFAULT false;
ALTER TABLE transfers          ADD COLUMN retention_until timestamptz, ADD COLUMN legal_hold boolean NOT NULL DEFAULT false;
ALTER TABLE packaging_runs     ADD COLUMN retention_until timestamptz, ADD COLUMN legal_hold boolean NOT NULL DEFAULT false;
ALTER TABLE lab_results        ADD COLUMN retention_until timestamptz, ADD COLUMN legal_hold boolean NOT NULL DEFAULT false;
ALTER TABLE releases           ADD COLUMN retention_until timestamptz, ADD COLUMN legal_hold boolean NOT NULL DEFAULT false;
ALTER TABLE cleaning_records   ADD COLUMN retention_until timestamptz, ADD COLUMN legal_hold boolean NOT NULL DEFAULT false;
ALTER TABLE vessel_occupancies ADD COLUMN retention_until timestamptz, ADD COLUMN legal_hold boolean NOT NULL DEFAULT false;

-- lab_results ve releases yalnız eklenir; saklama alanları yine de ayarlanabilir (kapanışta
-- bitiş uzatılır, yasal tutma konur). Başka sütun güncellenemez.
GRANT UPDATE (retention_until, legal_hold) ON lab_results, releases TO app_api, app_worker;

-- Koruma ----------------------------------------------------------------------------------
-- Silme: saklama süresince ve süresiz yasal tutmada reddedilir (zincirden gelen silme dahil).
CREATE FUNCTION retention_guard_delete() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF retention_blocks(OLD.legal_hold, OLD.retention_until) THEN
        RAISE EXCEPTION '% % is under retention', TG_TABLE_NAME, OLD.id
            USING ERRCODE = 'integrity_constraint_violation';
    END IF;
    RETURN OLD;
END
$$;

-- Güncelleme: yasal tutma yalnız `release_legal_hold` işleviyle (sahip rolün işlevi) kalkar,
-- saklama bitişi kısaltılamaz ve kaldırılamaz, saklamadaki kayıt yumuşak silinemez.
CREATE FUNCTION retention_guard_update() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF OLD.legal_hold AND NOT NEW.legal_hold AND current_user <> 'app_owner' THEN
        RAISE EXCEPTION 'legal hold on % % is lifted only by release_legal_hold()', TG_TABLE_NAME, OLD.id
            USING ERRCODE = 'integrity_constraint_violation';
    END IF;
    IF OLD.retention_until IS NOT NULL
       AND (NEW.retention_until IS NULL OR NEW.retention_until < OLD.retention_until)
       AND current_user <> 'app_owner' THEN
        RAISE EXCEPTION 'retention of % % cannot be shortened', TG_TABLE_NAME, OLD.id
            USING ERRCODE = 'integrity_constraint_violation';
    END IF;
    IF (to_jsonb(OLD) ->> 'deleted_at') IS NULL AND (to_jsonb(NEW) ->> 'deleted_at') IS NOT NULL
       AND retention_blocks(OLD.legal_hold, OLD.retention_until) THEN
        RAISE EXCEPTION '% % is under retention', TG_TABLE_NAME, OLD.id
            USING ERRCODE = 'integrity_constraint_violation';
    END IF;
    RETURN NEW;
END
$$;

DO $$
DECLARE
    t text;
BEGIN
    FOREACH t IN ARRAY ARRAY['batches', 'transfers', 'packaging_runs', 'lab_results', 'releases',
                             'cleaning_records', 'vessel_occupancies'] LOOP
        EXECUTE format('CREATE TRIGGER retention_delete BEFORE DELETE ON %I
                        FOR EACH ROW EXECUTE FUNCTION retention_guard_delete()', t);
        EXECUTE format('CREATE TRIGGER a_retention_update BEFORE UPDATE ON %I
                        FOR EACH ROW EXECUTE FUNCTION retention_guard_update()', t);
    END LOOP;
END
$$;
-- scanned_documents: silme ve yumuşak silme korumaları 0011 ve 0070'te; tutmayı kaldırma ve
-- bitişi kısaltma kuralı aynıdır.
CREATE TRIGGER a_retention_update BEFORE UPDATE ON scanned_documents
    FOR EACH ROW EXECUTE FUNCTION retention_guard_update();

-- Yasal tutmayı kaldırır. Yalnız bir kullanıcı bağlamında (çalışma alanı bağlamı gerekir),
-- tablo listesi sabit; API `workspace.manage` izni arar ve denetim kaydını yazar.
CREATE FUNCTION release_legal_hold(p_table text, p_id uuid) RETURNS boolean
    LANGUAGE plpgsql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
DECLARE
    n bigint;
BEGIN
    IF p_table <> ALL (ARRAY['batches', 'transfers', 'packaging_runs', 'lab_results', 'releases',
                             'cleaning_records', 'vessel_occupancies', 'scanned_documents']) THEN
        RAISE EXCEPTION 'table % has no legal hold', p_table;
    END IF;
    IF app_workspace_id() IS NULL OR app_actor_kind() IS DISTINCT FROM 'user' THEN
        RAISE EXCEPTION 'legal hold is lifted by a user inside a workspace context'
            USING ERRCODE = 'insufficient_privilege';
    END IF;
    EXECUTE format('UPDATE %I SET legal_hold = false WHERE id = $1 AND workspace_id = $2 AND legal_hold',
                   p_table)
        USING p_id, app_workspace_id();
    GET DIAGNOSTICS n = ROW_COUNT;
    RETURN n > 0;
END
$$;
REVOKE ALL ON FUNCTION release_legal_hold(text, uuid) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION release_legal_hold(text, uuid) TO app_api;

-- Çalışma alanı silme engeli bütün saklamalı tablolara bakar (0080'deki tanımın yerine).
CREATE OR REPLACE FUNCTION workspace_retention_blocker(p_workspace_id uuid) RETURNS timestamptz
    LANGUAGE plpgsql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
DECLARE
    prev text := current_setting('app.workspace_id', true);
    blocker timestamptz;
BEGIN
    PERFORM set_config('app.workspace_id', p_workspace_id::text, true);
    SELECT max(CASE WHEN legal_hold AND retention_until IS NULL THEN 'infinity'::timestamptz
                    ELSE retention_until END)
    INTO blocker
    FROM (
        SELECT legal_hold, retention_until FROM batches WHERE workspace_id = p_workspace_id
        UNION ALL SELECT legal_hold, retention_until FROM transfers WHERE workspace_id = p_workspace_id
        UNION ALL SELECT legal_hold, retention_until FROM packaging_runs WHERE workspace_id = p_workspace_id
        UNION ALL SELECT legal_hold, retention_until FROM lab_results WHERE workspace_id = p_workspace_id
        UNION ALL SELECT legal_hold, retention_until FROM releases WHERE workspace_id = p_workspace_id
        UNION ALL SELECT legal_hold, retention_until FROM cleaning_records WHERE workspace_id = p_workspace_id
        UNION ALL SELECT legal_hold, retention_until FROM vessel_occupancies WHERE workspace_id = p_workspace_id
        -- Taranmış belgede eski kural (0011): yalnız yasal tutma engeller, bitiş gelince kalkar.
        UNION ALL SELECT legal_hold AND (retention_until IS NULL OR retention_until > now()),
                         CASE WHEN legal_hold AND retention_until > now() THEN retention_until END
                  FROM scanned_documents WHERE workspace_id = p_workspace_id
    ) t
    WHERE retention_blocks(legal_hold, retention_until);
    PERFORM set_config('app.workspace_id', coalesce(prev, ''), true);
    RETURN blocker;
END
$$;

RESET ROLE;
