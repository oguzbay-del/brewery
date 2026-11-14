-- F2.1 düzeltmeleri: tarif satırı yarışı (bulgu 1), stok önbelleği ve sayım (bulgu 2, 3).
-- Uygulanmış göç dosyaları değişmez; işlevler CREATE OR REPLACE ile yenilenir.
SET LOCAL ROLE app_owner;

-- Bulgu 1: satır tetikleyicisi üst sürümün durumunu kilitli okur. Yabancı anahtar denetimi
-- yalnız FOR KEY SHARE alır ve `status` güncellemesinin FOR NO KEY UPDATE'iyle çakışmaz; FOR SHARE
-- çakışır, ekleme yayın işleminin bitmesini bekler ve yayınlanmış durumu görür.
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
    -- Başvurulan malzeme kalıcı silinince yabancı anahtar eylemi `ingredient_id`'yi boşaltır
    -- (adr/0007 › 6.1; satır değerleri zaten kopyadır). Başka alan değişmez.
    IF TG_OP = 'UPDATE' AND pg_trigger_depth() > 1
       AND NEW.ingredient_id IS NULL AND OLD.ingredient_id IS NOT NULL
       AND (to_jsonb(NEW) - ARRAY['ingredient_id', 'version', 'updated_at'])
           = (to_jsonb(OLD) - ARRAY['ingredient_id', 'version', 'updated_at']) THEN
        RETURN NEW;
    END IF;
    FOR v IN
        SELECT DISTINCT x FROM unnest(ARRAY[
            CASE WHEN TG_OP <> 'INSERT' THEN OLD.recipe_version_id END,
            CASE WHEN TG_OP <> 'DELETE' THEN NEW.recipe_version_id END]) x
        WHERE x IS NOT NULL
        ORDER BY x
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

-- Bulgu 2: lot miktarı yalnız hareketlerden oluşur; açılış miktarı `receive` hareketiyle girer.
CREATE FUNCTION stock_lots_guard_quantity_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF NEW.quantity <> 0 THEN
        RAISE EXCEPTION 'stock_lots.quantity starts at 0 and changes only through stock_movements'
            USING ERRCODE = 'integrity_constraint_violation';
    END IF;
    RETURN NEW;
END
$$;
CREATE TRIGGER a_guard_quantity_insert BEFORE INSERT ON stock_lots
    FOR EACH ROW EXECUTE FUNCTION stock_lots_guard_quantity_insert();

-- Bulgu 3: sayım hareketi sayılan mutlak değeri taşır; farkı tetikleyici lot kilidi altında
-- hesaplar. İstemcinin okuduğu eski miktar sonucu etkilemez.
ALTER TABLE stock_movements ADD COLUMN counted_quantity double precision;
ALTER TABLE stock_movements ADD CONSTRAINT stock_movements_counted_kind_check
    CHECK (counted_quantity IS NULL OR kind = 'count');

CREATE FUNCTION stock_movements_prepare() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE
    current_quantity double precision;
BEGIN
    IF NEW.kind = 'count' THEN
        IF NEW.counted_quantity IS NULL THEN
            RAISE EXCEPTION 'a count movement carries counted_quantity'
                USING ERRCODE = 'check_violation';
        END IF;
        SELECT quantity INTO current_quantity FROM stock_lots
        WHERE workspace_id = NEW.workspace_id AND id = NEW.lot_id
        FOR UPDATE;
        IF FOUND THEN
            NEW.quantity := NEW.counted_quantity - current_quantity;
        END IF;
    END IF;
    RETURN NEW;
END
$$;
CREATE TRIGGER a_prepare BEFORE INSERT ON stock_movements
    FOR EACH ROW EXECUTE FUNCTION stock_movements_prepare();

RESET ROLE;
