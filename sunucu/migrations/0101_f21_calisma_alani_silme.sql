-- F2.1 düzeltmeleri: kalıcı silme ve saklama (bulgu 5, 6, 7, 8, 11; DURUM "F2.1 decisions").
-- Kalıcı silmeyi yalnız worker yapar, dar bir SECURITY DEFINER işlevle; app_api'nin DELETE
-- hakkı kalkar. Silinen çalışma alanından küçük bir kayıt ve denetim zincirinin son özeti kalır.
SET LOCAL ROLE app_owner;

-- Bulgu 6: API yumuşak siler (adr/0008 › 1.4); kalıcı silme API'de yapılamaz.
REVOKE DELETE ON workspaces, users FROM app_api;

-- Bulgu 5: worker çalışma alanlarını bağlamsız sayar. Yalnız kimlik ve silinme zamanı döner.
CREATE POLICY owner_list ON workspaces FOR SELECT TO app_owner USING (true);

CREATE FUNCTION list_workspace_ids(p_soft_deleted_only boolean DEFAULT false)
    RETURNS TABLE (id uuid, deleted_at timestamptz)
    LANGUAGE sql STABLE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
    SELECT w.id, w.deleted_at FROM workspaces w
    WHERE NOT p_soft_deleted_only OR w.deleted_at IS NOT NULL
    ORDER BY w.id
$$;

-- Bulgu 8: silinen çalışma alanının kaydı. Çalışma alanının dışındadır (yabancı anahtarı ve
-- RLS bağlamı yoktur); yalnız eklenir. Kişisel veri taşımaz (ad ve e-posta yazılmaz).
CREATE TABLE workspace_tombstones (
    former_workspace_id  uuid PRIMARY KEY,
    mode                 text NOT NULL,
    region               text NOT NULL,
    workspace_created_at timestamptz NOT NULL,
    soft_deleted_at      timestamptz NOT NULL,
    deleted_at           timestamptz NOT NULL DEFAULT now(),
    deleted_by           uuid,
    reason               text,
    -- Zincirin son sırası ve özeti (zincir yoksa 0 ve sıfır özet).
    audit_last_seq       bigint NOT NULL CHECK (audit_last_seq >= 0),
    audit_last_hash      bytea NOT NULL CHECK (octet_length(audit_last_hash) = 32),
    support_action_count integer NOT NULL CHECK (support_action_count >= 0),
    support_last_action_at timestamptz
);
ALTER TABLE workspace_tombstones ENABLE ROW LEVEL SECURITY;
ALTER TABLE workspace_tombstones FORCE ROW LEVEL SECURITY;
CREATE POLICY owner_only ON workspace_tombstones TO app_owner USING (true) WITH CHECK (true);

CREATE FUNCTION workspace_tombstones_reject_change() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    RAISE EXCEPTION 'workspace_tombstones is append-only (% rejected)', TG_OP
        USING ERRCODE = 'integrity_constraint_violation';
END
$$;
CREATE TRIGGER no_update_delete BEFORE UPDATE OR DELETE ON workspace_tombstones
    FOR EACH ROW EXECUTE FUNCTION workspace_tombstones_reject_change();
CREATE TRIGGER no_truncate BEFORE TRUNCATE ON workspace_tombstones
    FOR EACH STATEMENT EXECUTE FUNCTION workspace_tombstones_reject_change();

-- Bulgu 5, 6, 8: kalıcı çalışma alanı silme. Yalnız yumuşak silinmiş çalışma alanı silinir.
-- Saklama kuralı (yasal tutma, saklama süresi) silmeyi tetikleyicilerle engeller; o zaman
-- tombstone da yazılmaz (aynı işlem geri alınır). Bekleme süresini (30 gün) worker uygular.
CREATE FUNCTION purge_workspace(
    p_workspace_id uuid, p_deleted_by uuid DEFAULT NULL, p_reason text DEFAULT NULL)
    RETURNS void
    LANGUAGE plpgsql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
DECLARE
    w workspaces%ROWTYPE;
    previous_context text := current_setting('app.workspace_id', true);
    head_seq bigint;
    head_hash bytea;
    actions integer;
    last_action timestamptz;
BEGIN
    -- Çalışma alanı satırlarına RLS politikaları üzerinden erişilir: bağlam geçici olarak ona kurulur.
    PERFORM set_config('app.workspace_id', p_workspace_id::text, true);
    SELECT * INTO w FROM workspaces WHERE id = p_workspace_id FOR UPDATE;
    IF NOT FOUND THEN
        RAISE EXCEPTION 'workspace % does not exist', p_workspace_id
            USING ERRCODE = 'no_data_found';
    END IF;
    IF w.deleted_at IS NULL THEN
        RAISE EXCEPTION 'workspace % is not soft-deleted', p_workspace_id
            USING ERRCODE = 'integrity_constraint_violation';
    END IF;
    SELECT last_seq, last_hash INTO head_seq, head_hash
    FROM audit_heads WHERE workspace_id = p_workspace_id;
    IF NOT FOUND THEN
        head_seq := 0;
        head_hash := decode(repeat('00', 32), 'hex');
    END IF;
    SELECT count(*), max(created_at) INTO actions, last_action
    FROM support_actions WHERE workspace_id = p_workspace_id;
    INSERT INTO workspace_tombstones (
        former_workspace_id, mode, region, workspace_created_at, soft_deleted_at,
        deleted_by, reason, audit_last_seq, audit_last_hash, support_action_count,
        support_last_action_at)
    VALUES (
        w.id, w.mode, w.region, w.created_at, w.deleted_at,
        coalesce(p_deleted_by, w.updated_by), p_reason, head_seq, head_hash, actions, last_action);
    DELETE FROM workspaces WHERE id = p_workspace_id;
    PERFORM set_config('app.workspace_id', coalesce(previous_context, ''), true);
END
$$;

REVOKE ALL ON FUNCTION list_workspace_ids(boolean), purge_workspace(uuid, uuid, text) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION list_workspace_ids(boolean), purge_workspace(uuid, uuid, text) TO app_worker;

-- Bulgu 7 (karar: değişmez başvuru anlık kopyaya indirilir, sonra asıl kayıt silinir) ----------
-- Başvurulan kayıt silinmeden önce BEFORE DELETE tetikleyicisi, başvuran satırlara kaydın
-- anlık kopyasını yazar; ardından yabancı anahtar eylemi başvuruyu boşaltır (SET NULL).
-- Tarif sürümünde (`equipment_snapshot`) ve tarif satırlarında (malzeme) değerler zaten kopyadır;
-- ayrı kopya gerekmez, yalnız başvuru boşalır.
ALTER TABLE batches           ADD COLUMN equipment_snapshot jsonb;
ALTER TABLE measurements      ADD COLUMN device_snapshot jsonb;
ALTER TABLE stock_movements   ADD COLUMN batch_snapshot jsonb;
ALTER TABLE yeast_generations ADD COLUMN source_batch_snapshot jsonb;

-- Sürüm koruması: başvuru boşaltılırken (iç içe tetikleyici) `equipment_id` boşalabilir.
CREATE OR REPLACE FUNCTION recipe_versions_guard() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE
    free_keys text[] := ARRAY['version', 'updated_at', 'updated_by', 'status'];
BEGIN
    IF TG_OP = 'DELETE' THEN
        IF OLD.status <> 'draft' AND pg_trigger_depth() < 2 THEN
            RAISE EXCEPTION 'published recipe version is immutable (DELETE rejected)'
                USING ERRCODE = 'integrity_constraint_violation';
        END IF;
        RETURN OLD;
    END IF;
    IF OLD.status = 'draft' THEN
        RETURN NEW;
    END IF;
    IF OLD.status = 'in_review' THEN
        free_keys := free_keys || ARRAY['approved_by', 'approved_at', 'submitted_by', 'submitted_at'];
        IF NEW.status NOT IN ('in_review', 'draft', 'approved') THEN
            RAISE EXCEPTION 'recipe version cannot go from in_review to %', NEW.status
                USING ERRCODE = 'integrity_constraint_violation';
        END IF;
    ELSIF OLD.status = 'approved' THEN
        free_keys := free_keys || ARRAY['archived_at'];
        IF NEW.status NOT IN ('approved', 'archived') THEN
            RAISE EXCEPTION 'recipe version cannot go from approved to %', NEW.status
                USING ERRCODE = 'integrity_constraint_violation';
        END IF;
    ELSIF NEW.status <> 'archived' THEN
        RAISE EXCEPTION 'archived recipe version is immutable'
            USING ERRCODE = 'integrity_constraint_violation';
    END IF;
    IF pg_trigger_depth() > 1 AND NEW.equipment_id IS NULL THEN
        -- Kalıcı silme (adr/0007 › 6.1): `equipment_snapshot` sürümde zaten kopyadır; başvuru boşalır.
        free_keys := free_keys || ARRAY['equipment_id'];
    END IF;
    IF (to_jsonb(NEW) - free_keys) IS DISTINCT FROM (to_jsonb(OLD) - free_keys) THEN
        RAISE EXCEPTION 'published recipe version is immutable (UPDATE rejected)'
            USING ERRCODE = 'integrity_constraint_violation';
    END IF;
    RETURN NEW;
END
$$;

-- Genel anlık kopya tetikleyicisi. TG_ARGV: (başvuran tablo, başvuru sütunu, kopya sütunu)
-- üçlüleri. Çağıran RLS altındadır; bağlam geçici olarak silinen satırın çalışma alanına kurulur.
CREATE FUNCTION snapshot_references() RETURNS trigger
    LANGUAGE plpgsql SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
DECLARE
    snapshot jsonb;
    previous_context text := current_setting('app.workspace_id', true);
    i integer := 0;
BEGIN
    snapshot := CASE TG_TABLE_NAME
        WHEN 'equipment' THEN to_jsonb(OLD) - 'search'
        WHEN 'batches' THEN jsonb_build_object('id', OLD.id, 'code', to_jsonb(OLD) -> 'code', 'name', to_jsonb(OLD) -> 'name')
        WHEN 'devices' THEN jsonb_build_object('id', OLD.id, 'kind', to_jsonb(OLD) -> 'kind', 'name', to_jsonb(OLD) -> 'name')
        ELSE jsonb_build_object('id', OLD.id)
    END;
    PERFORM set_config('app.workspace_id', OLD.workspace_id::text, true);
    WHILE i + 2 < TG_NARGS LOOP
        EXECUTE format('UPDATE %I SET %I = $1 WHERE workspace_id = $2 AND %I = $3',
                       TG_ARGV[i], TG_ARGV[i + 2], TG_ARGV[i + 1])
            USING snapshot, OLD.workspace_id, OLD.id;
        i := i + 3;
    END LOOP;
    PERFORM set_config('app.workspace_id', coalesce(previous_context, ''), true);
    RETURN OLD;
END
$$;
REVOKE ALL ON FUNCTION snapshot_references() FROM PUBLIC;

CREATE TRIGGER snapshot_refs BEFORE DELETE ON equipment
    FOR EACH ROW EXECUTE FUNCTION snapshot_references(
        'batches', 'equipment_id', 'equipment_snapshot');
CREATE TRIGGER snapshot_refs BEFORE DELETE ON devices
    FOR EACH ROW EXECUTE FUNCTION snapshot_references(
        'measurements', 'device_id', 'device_snapshot');
CREATE TRIGGER snapshot_refs BEFORE DELETE ON batches
    FOR EACH ROW EXECUTE FUNCTION snapshot_references(
        'stock_movements', 'batch_id', 'batch_snapshot',
        'yeast_generations', 'source_batch_id', 'source_batch_snapshot');

-- Ölçüm: cihaz başvurusu boşalınca kopya kalır.
ALTER TABLE measurements DROP CONSTRAINT measurements_check;
ALTER TABLE measurements ADD CONSTRAINT measurements_device_check CHECK (
    (source = 'device' OR device_id IS NULL)
    AND (source <> 'device' OR device_id IS NOT NULL OR device_snapshot IS NOT NULL));

-- Başvuruyu boşaltan yabancı anahtarlar.
DO $$
DECLARE
    r record;
BEGIN
    FOR r IN SELECT * FROM (VALUES
        ('recipe_versions', 'recipe_versions_workspace_id_equipment_id_fkey',
            '(workspace_id, equipment_id)', 'equipment (workspace_id, id)', '(equipment_id)'),
        ('batches', 'batches_workspace_id_equipment_id_fkey',
            '(workspace_id, equipment_id)', 'equipment (workspace_id, id)', '(equipment_id)'),
        ('measurements', 'measurements_workspace_id_device_id_fkey',
            '(workspace_id, device_id)', 'devices (workspace_id, id)', '(device_id)'),
        ('stock_movements', 'stock_movements_workspace_id_batch_id_fkey',
            '(workspace_id, batch_id)', 'batches (workspace_id, id)', '(batch_id)'),
        ('yeast_generations', 'yeast_generations_workspace_id_source_batch_id_fkey',
            '(workspace_id, source_batch_id)', 'batches (workspace_id, id)', '(source_batch_id)'),
        ('devices', 'devices_workspace_id_batch_id_fkey',
            '(workspace_id, batch_id)', 'batches (workspace_id, id)', '(batch_id)'),
        ('scanned_documents', 'scanned_documents_workspace_id_batch_id_fkey',
            '(workspace_id, batch_id)', 'batches (workspace_id, id)', '(batch_id)'),
        ('import_sources', 'import_sources_workspace_id_recipe_id_fkey',
            '(workspace_id, recipe_id)', 'recipes (workspace_id, id)', '(recipe_id)'),
        ('recipe_fermentables', 'recipe_fermentables_ingredient_id_fkey',
            '(ingredient_id)', 'ingredients (id)', ''),
        ('recipe_hops', 'recipe_hops_ingredient_id_fkey', '(ingredient_id)', 'ingredients (id)', ''),
        ('recipe_yeasts', 'recipe_yeasts_ingredient_id_fkey', '(ingredient_id)', 'ingredients (id)', ''),
        ('recipe_miscs', 'recipe_miscs_ingredient_id_fkey', '(ingredient_id)', 'ingredients (id)', '')
    ) v(tbl, con, cols, ref, nullcols)
    LOOP
        EXECUTE format('ALTER TABLE %I DROP CONSTRAINT %I', r.tbl, r.con);
        EXECUTE format(
            'ALTER TABLE %I ADD CONSTRAINT %I FOREIGN KEY %s REFERENCES %s ON DELETE SET NULL %s',
            r.tbl, r.con, r.cols, r.ref, r.nullcols);
    END LOOP;
END
$$;

-- Bulgu 11 (karar: altyapı şimdi, süreler yapılandırmada, Aşama 6 belirler) ----------------------
-- Saklama bitişi ve yasal tutma: koruma sürerken kayıt silinmez ve yumuşak silinmez; çalışma
-- alanı silme de bu kurala takılır (kaskad silme tetikleyiciyi çalıştırır).
-- Korumalı: retention_until ileride, ya da yasal tutma var ve bitiş boş (süresiz).
ALTER TABLE batches        ADD COLUMN retention_until timestamptz, ADD COLUMN legal_hold boolean NOT NULL DEFAULT false;
ALTER TABLE packaging_runs ADD COLUMN retention_until timestamptz, ADD COLUMN legal_hold boolean NOT NULL DEFAULT false;
ALTER TABLE transfers      ADD COLUMN retention_until timestamptz, ADD COLUMN legal_hold boolean NOT NULL DEFAULT false;
ALTER TABLE releases       ADD COLUMN retention_until timestamptz, ADD COLUMN legal_hold boolean NOT NULL DEFAULT false;

CREATE FUNCTION retention_guard_delete() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF OLD.retention_until > now() OR (OLD.legal_hold AND OLD.retention_until IS NULL) THEN
        RAISE EXCEPTION '% % is under mandatory retention or legal hold', TG_TABLE_NAME, OLD.id
            USING ERRCODE = 'integrity_constraint_violation';
    END IF;
    RETURN OLD;
END
$$;
CREATE FUNCTION retention_guard_soft_delete() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF OLD.deleted_at IS NULL AND NEW.deleted_at IS NOT NULL
       AND (OLD.retention_until > now() OR (OLD.legal_hold AND OLD.retention_until IS NULL)) THEN
        RAISE EXCEPTION '% % is under mandatory retention or legal hold', TG_TABLE_NAME, OLD.id
            USING ERRCODE = 'integrity_constraint_violation';
    END IF;
    RETURN NEW;
END
$$;
CREATE TRIGGER retention_hold BEFORE DELETE ON batches
    FOR EACH ROW EXECUTE FUNCTION retention_guard_delete();
CREATE TRIGGER retention_hold BEFORE DELETE ON packaging_runs
    FOR EACH ROW EXECUTE FUNCTION retention_guard_delete();
CREATE TRIGGER retention_hold BEFORE DELETE ON transfers
    FOR EACH ROW EXECUTE FUNCTION retention_guard_delete();
CREATE TRIGGER retention_hold BEFORE DELETE ON releases
    FOR EACH ROW EXECUTE FUNCTION retention_guard_delete();
CREATE TRIGGER retention_hold_soft BEFORE UPDATE OF deleted_at ON batches
    FOR EACH ROW EXECUTE FUNCTION retention_guard_soft_delete();
CREATE TRIGGER retention_hold_soft BEFORE UPDATE OF deleted_at ON packaging_runs
    FOR EACH ROW EXECUTE FUNCTION retention_guard_soft_delete();
CREATE TRIGGER retention_hold_soft BEFORE UPDATE OF deleted_at ON transfers
    FOR EACH ROW EXECUTE FUNCTION retention_guard_soft_delete();

RESET ROLE;
