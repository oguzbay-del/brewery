-- Stok (adr/0007 › 3 "Stok"; T2.6).
-- Doğruluk kaynağı hareketlerdir (defter, yalnız eklenir). `stock_lots.quantity` önbellektir:
-- stock_movements'a eklemede bir tetikleyiciyle aynı işlemde güncellenir. Lot satırı
-- güncellenirken kilitlendiği için eşzamanlı iki hareket sıraya girer. Eksi stok reddedilmez
-- (adr/0007 › 3 "Toplam"); uyarıyı uygulama üretir.
SET LOCAL ROLE app_owner;

CREATE TABLE stock_lots (
    id            uuid PRIMARY KEY,
    workspace_id  uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    ingredient_id uuid NOT NULL REFERENCES ingredients (id),
    -- Evde tek varsayılan lot kullanılır.
    lot_code      text NOT NULL DEFAULT 'default' CHECK (char_length(lot_code) BETWEEN 1 AND 100),
    supplier      text CHECK (char_length(supplier) <= 200),
    received_on   date,
    expires_on    date,
    -- Miktar birimi: kg, L ya da adet (maya paketi); SI metrik takımı (adr/0007 › 1.4).
    unit          text NOT NULL DEFAULT 'kg' CHECK (unit IN ('kg', 'l', 'count')),
    unit_cost     numeric(19,4) CHECK (unit_cost >= 0),
    cost_currency char(3) CHECK (cost_currency ~ '^[A-Z]{3}$'),
    quantity      double precision NOT NULL DEFAULT 0,
    notes         text,
    search        tsvector GENERATED ALWAYS AS (
                      to_tsvector('simple', lot_code || ' ' || coalesce(supplier, ''))
                  ) STORED,
    created_at    timestamptz NOT NULL DEFAULT now(),
    updated_at    timestamptz NOT NULL DEFAULT now(),
    deleted_at    timestamptz,
    version       bigint NOT NULL DEFAULT 1,
    created_by    uuid REFERENCES users (id),
    updated_by    uuid REFERENCES users (id),
    UNIQUE (workspace_id, id),
    CHECK ((unit_cost IS NULL) = (cost_currency IS NULL))
);
CREATE UNIQUE INDEX stock_lots_code_idx ON stock_lots (workspace_id, ingredient_id, lot_code)
    WHERE deleted_at IS NULL;
CREATE INDEX stock_lots_search_idx ON stock_lots USING gin (search);
CREATE TRIGGER shared_ref_ingredient_id BEFORE INSERT OR UPDATE OF ingredient_id, workspace_id ON stock_lots
    FOR EACH ROW EXECUTE FUNCTION check_shared_ref('ingredients', 'ingredient_id');

-- Önbellek yalnız hareket tetikleyicisiyle değişir (iç içe tetikleyici derinliği >= 2).
CREATE FUNCTION stock_lots_guard_quantity() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF NEW.quantity IS DISTINCT FROM OLD.quantity AND pg_trigger_depth() < 2 THEN
        RAISE EXCEPTION 'stock_lots.quantity changes only through stock_movements'
            USING ERRCODE = 'integrity_constraint_violation';
    END IF;
    RETURN NEW;
END
$$;
CREATE TRIGGER a_guard_quantity BEFORE UPDATE ON stock_lots
    FOR EACH ROW EXECUTE FUNCTION stock_lots_guard_quantity();

CREATE TABLE stock_movements (
    id            uuid PRIMARY KEY,
    workspace_id  uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    lot_id        uuid NOT NULL,
    kind          text NOT NULL CHECK (kind IN ('receive', 'consume', 'adjust', 'count', 'transfer')),
    -- İşaretli fark, lotun biriminde: alım +, tüketim -, sayım ve düzeltme fark.
    quantity      double precision NOT NULL,
    batch_id      uuid,
    occurred_at   timestamptz NOT NULL DEFAULT now(),
    reason        text,
    created_at    timestamptz NOT NULL DEFAULT now(),
    created_by    uuid REFERENCES users (id),
    FOREIGN KEY (workspace_id, lot_id) REFERENCES stock_lots (workspace_id, id) ON DELETE CASCADE,
    FOREIGN KEY (workspace_id, batch_id) REFERENCES batches (workspace_id, id),
    CHECK (kind <> 'receive' OR quantity > 0),
    CHECK (kind <> 'consume' OR quantity < 0),
    CHECK (kind IN ('count', 'adjust') OR quantity <> 0)
);
CREATE INDEX stock_movements_lot_idx ON stock_movements (workspace_id, lot_id, occurred_at);
CREATE INDEX stock_movements_batch_idx ON stock_movements (workspace_id, batch_id) WHERE batch_id IS NOT NULL;

CREATE FUNCTION stock_movements_apply() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE stock_lots SET quantity = quantity + NEW.quantity
    WHERE workspace_id = NEW.workspace_id AND id = NEW.lot_id;
    RETURN NULL;
END
$$;
CREATE TRIGGER apply AFTER INSERT ON stock_movements
    FOR EACH ROW EXECUTE FUNCTION stock_movements_apply();

CREATE TABLE shopping_list_items (
    id                         uuid PRIMARY KEY,
    workspace_id               uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    ingredient_id              uuid NOT NULL REFERENCES ingredients (id),
    quantity                   double precision NOT NULL CHECK (quantity >= 0),
    unit                       text NOT NULL DEFAULT 'kg' CHECK (unit IN ('kg', 'l', 'count')),
    source_recipe_version_id   uuid,
    checked                    boolean NOT NULL DEFAULT false,
    created_at                 timestamptz NOT NULL DEFAULT now(),
    updated_at                 timestamptz NOT NULL DEFAULT now(),
    deleted_at                 timestamptz,
    version                    bigint NOT NULL DEFAULT 1,
    created_by                 uuid REFERENCES users (id),
    updated_by                 uuid REFERENCES users (id),
    UNIQUE (workspace_id, id),
    FOREIGN KEY (workspace_id, source_recipe_version_id) REFERENCES recipe_versions (workspace_id, id)
);
CREATE TRIGGER shared_ref_ingredient_id BEFORE INSERT OR UPDATE OF ingredient_id, workspace_id ON shopping_list_items
    FOR EACH ROW EXECUTE FUNCTION check_shared_ref('ingredients', 'ingredient_id');

-- Partinin gerçek malzeme kaydı stok lotuna bağlanır.
ALTER TABLE batch_ingredient_actuals
    ADD FOREIGN KEY (workspace_id, lot_id) REFERENCES stock_lots (workspace_id, id);

SELECT secure_workspace_table(t) FROM unnest(ARRAY['stock_lots', 'shopping_list_items']) t;
SELECT secure_workspace_table('stock_movements', 'append', 'SELECT, INSERT');

RESET ROLE;
