-- T2.9 (F2.1 bulgu 1, 7): kalıcı silmede değişmez ya da yalnız eklenen satırların başvurusu
-- anlık kopyaya iner, asıl kayıt silinir; yayınlama ile satır yazımı yarışı kapanır.
-- Karar (14.11.2026, F2.1 karar 2): başvuru NULL'a çekilir, ad ve kod gibi gösterilen değer
-- anlık kopya sütununda kalır (tarif sürümü ve satırlarında kopya zaten vardır).
SET LOCAL ROLE app_owner;

-- Yabancı anahtarı ON DELETE SET NULL (ya da CASCADE) ile yeniden kurar; adı korunur.
-- Bileşik anahtarda (workspace_id, sütun) yalnız sütun boşalır (PostgreSQL 15 sütun listesi).
CREATE FUNCTION pg_temp.fk_on_delete(
    p_table regclass, p_col name, p_ref regclass, p_action text) RETURNS void
    LANGUAGE plpgsql
    AS $$
DECLARE
    con record;
    col_attnum smallint;
    composite boolean;
    cols text;
    refcols text;
BEGIN
    SELECT attnum INTO col_attnum FROM pg_attribute WHERE attrelid = p_table AND attname = p_col;
    SELECT * INTO con FROM pg_constraint c
    WHERE c.conrelid = p_table AND c.contype = 'f' AND c.confrelid = p_ref
      AND col_attnum = ANY (c.conkey);
    IF NOT FOUND THEN
        RAISE EXCEPTION 'no foreign key on %.% to %', p_table, p_col, p_ref;
    END IF;
    composite := array_length(con.conkey, 1) = 2;
    cols := CASE WHEN composite THEN format('workspace_id, %I', p_col) ELSE format('%I', p_col) END;
    refcols := CASE WHEN composite THEN 'workspace_id, id' ELSE 'id' END;
    EXECUTE format('ALTER TABLE %s DROP CONSTRAINT %I', p_table, con.conname);
    EXECUTE format(
        'ALTER TABLE %s ADD CONSTRAINT %I FOREIGN KEY (%s) REFERENCES %s (%s) ON DELETE %s%s',
        p_table, con.conname, cols, p_ref, refcols, p_action,
        CASE WHEN p_action = 'SET NULL' AND composite THEN format(' (%I)', p_col) ELSE '' END);
END
$$;

-- Anlık kopya sütunları -------------------------------------------------------------------
ALTER TABLE batches                  ADD COLUMN equipment_name text CHECK (char_length(equipment_name) <= 200);
ALTER TABLE batch_ingredient_actuals ADD COLUMN substitute_ingredient_name text CHECK (char_length(substitute_ingredient_name) <= 200);
ALTER TABLE batch_ingredient_actuals ADD COLUMN lot_code text CHECK (char_length(lot_code) <= 100);
ALTER TABLE stock_movements          ADD COLUMN batch_code text CHECK (char_length(batch_code) <= 50);
ALTER TABLE scanned_documents        ADD COLUMN batch_code text CHECK (char_length(batch_code) <= 50);
ALTER TABLE yeast_generations        ADD COLUMN source_batch_code text CHECK (char_length(source_batch_code) <= 50);
-- Ölçümde cihaz adını ingest yazar (en sıcak tablo, tetikleyici yok).
ALTER TABLE measurements             ADD COLUMN device_name text CHECK (char_length(device_name) <= 200);

-- Başvuru doluysa hedefin değerini kopyalar; boşaldığında (kalıcı silme) eski kopya kalır.
-- TG_ARGV: hedef tablo, başvuru sütunu, hedefte kopyalanan sütun, kopya sütunu.
CREATE FUNCTION snapshot_reference() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE
    ref_id uuid := (to_jsonb(NEW) ->> TG_ARGV[1])::uuid;
    val text;
BEGIN
    IF ref_id IS NOT NULL THEN
        EXECUTE format('SELECT %I::text FROM %I WHERE id = $1', TG_ARGV[2], TG_ARGV[0])
            INTO val USING ref_id;
        IF val IS NOT NULL THEN
            NEW := jsonb_populate_record(NEW, jsonb_build_object(TG_ARGV[3], val));
        END IF;
    END IF;
    RETURN NEW;
END
$$;

CREATE TRIGGER snapshot_equipment BEFORE INSERT OR UPDATE OF equipment_id ON batches
    FOR EACH ROW EXECUTE FUNCTION snapshot_reference('equipment', 'equipment_id', 'name', 'equipment_name');
CREATE TRIGGER snapshot_substitute BEFORE INSERT OR UPDATE OF substitute_ingredient_id ON batch_ingredient_actuals
    FOR EACH ROW EXECUTE FUNCTION snapshot_reference(
        'ingredients', 'substitute_ingredient_id', 'name', 'substitute_ingredient_name');
CREATE TRIGGER snapshot_lot BEFORE INSERT OR UPDATE OF lot_id ON batch_ingredient_actuals
    FOR EACH ROW EXECUTE FUNCTION snapshot_reference('stock_lots', 'lot_id', 'lot_code', 'lot_code');
CREATE TRIGGER snapshot_batch BEFORE INSERT ON stock_movements
    FOR EACH ROW EXECUTE FUNCTION snapshot_reference('batches', 'batch_id', 'code', 'batch_code');
CREATE TRIGGER snapshot_batch BEFORE INSERT OR UPDATE OF batch_id ON scanned_documents
    FOR EACH ROW EXECUTE FUNCTION snapshot_reference('batches', 'batch_id', 'code', 'batch_code');
CREATE TRIGGER snapshot_batch BEFORE INSERT OR UPDATE OF source_batch_id ON yeast_generations
    FOR EACH ROW EXECUTE FUNCTION snapshot_reference('batches', 'source_batch_id', 'code', 'source_batch_code');

-- Başvurular: silinen kaydın yerine NULL ----------------------------------------------------
SELECT pg_temp.fk_on_delete('recipe_versions', 'equipment_id', 'equipment', 'SET NULL');
SELECT pg_temp.fk_on_delete('batches', 'equipment_id', 'equipment', 'SET NULL');
SELECT pg_temp.fk_on_delete(t, 'ingredient_id', 'ingredients', 'SET NULL')
FROM unnest(ARRAY['recipe_fermentables', 'recipe_hops', 'recipe_yeasts', 'recipe_miscs']::regclass[]) t;
SELECT pg_temp.fk_on_delete('batch_ingredient_actuals', 'substitute_ingredient_id', 'ingredients', 'SET NULL');
SELECT pg_temp.fk_on_delete('batch_ingredient_actuals', 'lot_id', 'stock_lots', 'SET NULL');
SELECT pg_temp.fk_on_delete('devices', 'batch_id', 'batches', 'SET NULL');
SELECT pg_temp.fk_on_delete('stock_movements', 'batch_id', 'batches', 'SET NULL');
SELECT pg_temp.fk_on_delete('scanned_documents', 'batch_id', 'batches', 'SET NULL');
SELECT pg_temp.fk_on_delete('yeast_generations', 'source_batch_id', 'batches', 'SET NULL');
SELECT pg_temp.fk_on_delete('import_sources', 'recipe_id', 'recipes', 'SET NULL');
SELECT pg_temp.fk_on_delete('shopping_list_items', 'source_recipe_version_id', 'recipe_versions', 'SET NULL');
-- Alışveriş listesi satırı malzemesiz anlamsızdır.
SELECT pg_temp.fk_on_delete('shopping_list_items', 'ingredient_id', 'ingredients', 'CASCADE');
-- Ölçüm: cihaz ve paket silinince ölçüm kalır (cihaz adı ölçümde); cihaz kısıtı gevşer.
SELECT pg_temp.fk_on_delete('measurements', 'device_id', 'devices', 'SET NULL');
SELECT pg_temp.fk_on_delete('measurements', 'package_id', 'brew_day_packages', 'SET NULL');

DO $$
DECLARE
    con name;
BEGIN
    FOR con IN
        SELECT conname FROM pg_constraint
        WHERE conrelid = 'measurements'::regclass AND contype = 'c'
          AND pg_get_constraintdef(oid) LIKE '%device_id%'
    LOOP
        EXECUTE format('ALTER TABLE measurements DROP CONSTRAINT %I', con);
    END LOOP;
END
$$;
-- Cihaz kaynağı dışında cihaz kimliği olamaz; cihaz kaydı silinince kimlik boşalabilir.
ALTER TABLE measurements ADD CONSTRAINT measurements_device_source_check
    CHECK (source = 'device' OR device_id IS NULL);

-- Gerçek malzeme kaydı bir tarif satırına ya da ikame malzemeye dayanır; ikame malzeme
-- kalıcı silinince adı kopyada kalır, kısıt kopyayı da kabul eder.
DO $$
DECLARE
    con name;
BEGIN
    FOR con IN
        SELECT conname FROM pg_constraint
        WHERE conrelid = 'batch_ingredient_actuals'::regclass AND contype = 'c'
          AND pg_get_constraintdef(oid) LIKE '%substitute_ingredient_id IS NOT NULL%'
    LOOP
        EXECUTE format('ALTER TABLE batch_ingredient_actuals DROP CONSTRAINT %I', con);
    END LOOP;
END
$$;
ALTER TABLE batch_ingredient_actuals ADD CONSTRAINT batch_ingredient_actuals_source_check
    CHECK (line_id IS NOT NULL OR substitute_ingredient_id IS NOT NULL
           OR substitute_ingredient_name IS NOT NULL);

-- Değişmezlik tetikleyicileri: yalnız başvuru boşalması geçer ------------------------------
-- Yayınlanmış sürümde ekipman başvurusunun boşalması (equipment_snapshot sürümde kalır) ve
-- satırlarda kaynak malzeme başvurusunun boşalması (ad ve özellikler satırdadır) tek izinli
-- değişikliktir. Aynı işlev satır tetikleyicisinde üst sürümün durumunu `FOR SHARE` ile okur
-- (F2.1 bulgu 1): yabancı anahtar denetimi `FOR KEY SHARE` alır, `status` güncellemesi
-- `FOR NO KEY UPDATE`; ikisi çakışmaz, `FOR SHARE` çakışır ve yayınlama ile satır yazımı sıraya girer.
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
    IF NEW.equipment_id IS NULL AND OLD.equipment_id IS NOT NULL THEN
        free_keys := free_keys || ARRAY['equipment_id'];
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
    IF (to_jsonb(NEW) - free_keys) IS DISTINCT FROM (to_jsonb(OLD) - free_keys) THEN
        RAISE EXCEPTION 'published recipe version is immutable (UPDATE rejected)'
            USING ERRCODE = 'integrity_constraint_violation';
    END IF;
    RETURN NEW;
END
$$;

CREATE OR REPLACE FUNCTION recipe_version_lines_guard() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE
    v uuid;
    s text;
BEGIN
    IF TG_OP = 'DELETE' AND pg_trigger_depth() > 1 THEN
        RETURN OLD;
    END IF;
    IF TG_OP = 'UPDATE'
       AND (to_jsonb(NEW) ->> 'ingredient_id') IS NULL
       AND (to_jsonb(OLD) ->> 'ingredient_id') IS NOT NULL
       AND (to_jsonb(NEW) - 'ingredient_id') = (to_jsonb(OLD) - 'ingredient_id') THEN
        RETURN NEW;
    END IF;
    FOR v IN
        SELECT DISTINCT x FROM unnest(ARRAY[
            CASE WHEN TG_OP <> 'INSERT' THEN OLD.recipe_version_id END,
            CASE WHEN TG_OP <> 'DELETE' THEN NEW.recipe_version_id END]) x
        WHERE x IS NOT NULL
    LOOP
        SELECT status INTO s FROM recipe_versions
        WHERE id = v AND workspace_id = coalesce(NEW.workspace_id, OLD.workspace_id)
        FOR SHARE;
        IF s IS NOT NULL AND s <> 'draft' THEN
            RAISE EXCEPTION 'lines of a published recipe version are immutable (% rejected)', TG_OP
                USING ERRCODE = 'integrity_constraint_violation';
        END IF;
    END LOOP;
    RETURN CASE TG_OP WHEN 'DELETE' THEN OLD ELSE NEW END;
END
$$;

RESET ROLE;
