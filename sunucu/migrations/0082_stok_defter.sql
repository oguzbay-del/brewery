-- T2.9 (F2.1 bulgu 2, 3): stok defteri. Lot satırı miktarla eklenemez (açılış miktarı
-- `receive` hareketiyle girer); sayım hareketi sayılan mutlak değeri taşır, farkı tetikleyici
-- lot kilidi altında hesaplar (eşzamanlı tüketimde sayım yanlış sonuç vermez).
SET LOCAL ROLE app_owner;

CREATE FUNCTION stock_lots_reject_initial_quantity() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF NEW.quantity <> 0 THEN
        RAISE EXCEPTION 'stock_lots.quantity starts at 0; receive stock with a stock_movements row'
            USING ERRCODE = 'integrity_constraint_violation';
    END IF;
    RETURN NEW;
END
$$;
CREATE TRIGGER a_initial_quantity BEFORE INSERT ON stock_lots
    FOR EACH ROW EXECUTE FUNCTION stock_lots_reject_initial_quantity();

ALTER TABLE stock_movements ADD COLUMN counted_quantity double precision
    CHECK (counted_quantity >= 0);
-- Sayımda `quantity` (işaretli fark) tetikleyiciden gelir; istemci vermez.
ALTER TABLE stock_movements ALTER COLUMN quantity SET DEFAULT 0;
ALTER TABLE stock_movements ADD CONSTRAINT stock_movements_count_check
    CHECK ((kind = 'count') = (counted_quantity IS NOT NULL));

CREATE FUNCTION stock_movements_resolve_count() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE
    current_quantity double precision;
BEGIN
    IF NEW.kind = 'count' THEN
        IF NEW.counted_quantity IS NULL THEN
            RAISE EXCEPTION 'a count movement needs counted_quantity'
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
CREATE TRIGGER a_resolve_count BEFORE INSERT ON stock_movements
    FOR EACH ROW EXECUTE FUNCTION stock_movements_resolve_count();

RESET ROLE;
