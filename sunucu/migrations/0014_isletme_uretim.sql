-- İşletme üretim tabloları (adr/0007 › 3 "İşletme"; T2.7): üretim alanı, tank, doluluk,
-- transfer, ambalaj, kalite, temizlik ve maya jenerasyonu. Hepsi çalışma alanına aittir ve
-- RLS altındadır (`secure_workspace_table`). Uçlar Aşama 6'dadır.
-- Gerçekleşen (actual) tank doluluğu çakışmaz (EXCLUDE); planlanan doluluk çakışabilir ve
-- yalnız uyarıdır (plan › Eşzamanlı düzenleme › Tank planı).
CREATE EXTENSION IF NOT EXISTS btree_gist;

SET LOCAL ROLE app_owner;

-- production_areas, vessels ----------------------------------------------------------------
CREATE TABLE production_areas (
    id            uuid PRIMARY KEY,
    workspace_id  uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    name          text NOT NULL CHECK (char_length(name) BETWEEN 1 AND 200),
    notes         text,
    created_at    timestamptz NOT NULL DEFAULT now(),
    updated_at    timestamptz NOT NULL DEFAULT now(),
    deleted_at    timestamptz,
    version       bigint NOT NULL DEFAULT 1,
    created_by    uuid REFERENCES users (id),
    updated_by    uuid REFERENCES users (id),
    UNIQUE (workspace_id, id)
);
CREATE UNIQUE INDEX production_areas_name_idx ON production_areas (workspace_id, lower(name))
    WHERE deleted_at IS NULL;

CREATE TABLE vessels (
    id               uuid PRIMARY KEY,
    workspace_id     uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    -- İnsana gösterilen kısa ad (FV-1, BBT-2); çalışma alanında tekil.
    code             text NOT NULL CHECK (char_length(code) BETWEEN 1 AND 50),
    name             text NOT NULL DEFAULT '' CHECK (char_length(name) <= 200),
    kind             text NOT NULL CHECK (kind IN (
        'fermenter', 'brite', 'kettle', 'mash_tun', 'hlt', 'yeast_brink', 'keg_store', 'other')),
    capacity_l       double precision NOT NULL CHECK (capacity_l > 0),
    -- Tank basınç sınırı (spunding uyarısı için, 09 §3.4); basınçsız kapta boş.
    max_pressure_bar double precision CHECK (max_pressure_bar > 0),
    area_id          uuid,
    cleaning_status  text NOT NULL DEFAULT 'dirty' CHECK (cleaning_status IN (
        'dirty', 'cleaning', 'clean', 'sanitized')),
    cleaned_at       timestamptz,
    notes            text,
    created_at       timestamptz NOT NULL DEFAULT now(),
    updated_at       timestamptz NOT NULL DEFAULT now(),
    deleted_at       timestamptz,
    version          bigint NOT NULL DEFAULT 1,
    created_by       uuid REFERENCES users (id),
    updated_by       uuid REFERENCES users (id),
    UNIQUE (workspace_id, id),
    -- Alan silinince tank alansız kalır (yalnız area_id boşalır, workspace_id değil).
    FOREIGN KEY (workspace_id, area_id) REFERENCES production_areas (workspace_id, id)
        ON DELETE SET NULL (area_id)
);
CREATE UNIQUE INDEX vessels_code_idx ON vessels (workspace_id, code) WHERE deleted_at IS NULL;
CREATE INDEX vessels_area_idx ON vessels (workspace_id, area_id) WHERE deleted_at IS NULL;

-- Cihazın bağlı olduğu tank (T2.6 notu: yabancı anahtar tablolar gelince).
ALTER TABLE devices
    ADD FOREIGN KEY (workspace_id, vessel_id) REFERENCES vessels (workspace_id, id)
        ON DELETE SET NULL (vessel_id);

-- vessel_occupancies ------------------------------------------------------------------------
-- Parti ↔ tank, zaman aralığı `[başlangıç, bitiş)`; bitiş boşsa tank hâlâ dolu. Bölme: bir
-- parti iki tankta; birleştirme: hedef tankın dolulukları art arda, karışımın partisi hedefe
-- yazılır (transfer kaydı kaynak partileri taşır). Çakışma engeli yalnız `actual` satırdadır.
CREATE TABLE vessel_occupancies (
    id            uuid PRIMARY KEY,
    workspace_id  uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    batch_id      uuid NOT NULL,
    vessel_id     uuid NOT NULL,
    status        text NOT NULL DEFAULT 'planned' CHECK (status IN ('planned', 'actual')),
    period        tstzrange NOT NULL CHECK (NOT isempty(period) AND NOT lower_inf(period)),
    volume_l      double precision CHECK (volume_l > 0),
    notes         text,
    created_at    timestamptz NOT NULL DEFAULT now(),
    updated_at    timestamptz NOT NULL DEFAULT now(),
    deleted_at    timestamptz,
    version       bigint NOT NULL DEFAULT 1,
    created_by    uuid REFERENCES users (id),
    updated_by    uuid REFERENCES users (id),
    UNIQUE (workspace_id, id),
    -- Gerçekleşen doluluğun hacmi bilinir; planlananınki boş olabilir.
    CHECK (status = 'planned' OR volume_l IS NOT NULL),
    FOREIGN KEY (workspace_id, batch_id) REFERENCES batches (workspace_id, id) ON DELETE CASCADE,
    FOREIGN KEY (workspace_id, vessel_id) REFERENCES vessels (workspace_id, id) ON DELETE CASCADE,
    CONSTRAINT vessel_occupancies_no_actual_overlap
        EXCLUDE USING gist (vessel_id WITH =, period WITH &&)
        WHERE (status = 'actual' AND deleted_at IS NULL)
);
CREATE INDEX vessel_occupancies_batch_idx ON vessel_occupancies (workspace_id, batch_id);
CREATE INDEX vessel_occupancies_vessel_idx ON vessel_occupancies USING gist (vessel_id, period)
    WHERE deleted_at IS NULL;

-- transfers ---------------------------------------------------------------------------------
-- Tanklar arası hacim hareketi. `disposal`: hedefsiz (imha), `receipt`: kaynaksız (dışarıdan
-- gelen, ör. iade ya da satın alınan wort). Bölme ve birleştirmede ilgili partiler
-- `transfer_batches` satırlarındadır (hacimleriyle).
CREATE TABLE transfers (
    id               uuid PRIMARY KEY,
    workspace_id     uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    kind             text NOT NULL DEFAULT 'transfer' CHECK (kind IN (
        'transfer', 'split', 'merge', 'disposal', 'receipt')),
    source_vessel_id uuid,
    target_vessel_id uuid,
    occurred_at      timestamptz NOT NULL DEFAULT now(),
    volume_l         double precision NOT NULL CHECK (volume_l > 0),
    loss_l           double precision NOT NULL DEFAULT 0 CHECK (loss_l >= 0),
    performed_by     uuid REFERENCES users (id),
    notes            text,
    created_at       timestamptz NOT NULL DEFAULT now(),
    updated_at       timestamptz NOT NULL DEFAULT now(),
    deleted_at       timestamptz,
    version          bigint NOT NULL DEFAULT 1,
    created_by       uuid REFERENCES users (id),
    updated_by       uuid REFERENCES users (id),
    UNIQUE (workspace_id, id),
    CHECK (loss_l <= volume_l),
    CHECK (source_vessel_id IS DISTINCT FROM target_vessel_id),
    CHECK (CASE kind
        WHEN 'disposal' THEN source_vessel_id IS NOT NULL AND target_vessel_id IS NULL
        WHEN 'receipt' THEN source_vessel_id IS NULL AND target_vessel_id IS NOT NULL
        ELSE source_vessel_id IS NOT NULL AND target_vessel_id IS NOT NULL END),
    FOREIGN KEY (workspace_id, source_vessel_id) REFERENCES vessels (workspace_id, id),
    FOREIGN KEY (workspace_id, target_vessel_id) REFERENCES vessels (workspace_id, id)
);
CREATE INDEX transfers_source_idx ON transfers (workspace_id, source_vessel_id, occurred_at);
CREATE INDEX transfers_target_idx ON transfers (workspace_id, target_vessel_id, occurred_at);

CREATE TABLE transfer_batches (
    id            uuid PRIMARY KEY,
    workspace_id  uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    transfer_id   uuid NOT NULL,
    batch_id      uuid NOT NULL,
    volume_l      double precision NOT NULL CHECK (volume_l > 0),
    created_at    timestamptz NOT NULL DEFAULT now(),
    updated_at    timestamptz NOT NULL DEFAULT now(),
    deleted_at    timestamptz,
    version       bigint NOT NULL DEFAULT 1,
    created_by    uuid REFERENCES users (id),
    updated_by    uuid REFERENCES users (id),
    UNIQUE (workspace_id, id),
    FOREIGN KEY (workspace_id, transfer_id) REFERENCES transfers (workspace_id, id) ON DELETE CASCADE,
    FOREIGN KEY (workspace_id, batch_id) REFERENCES batches (workspace_id, id) ON DELETE CASCADE
);
CREATE UNIQUE INDEX transfer_batches_unique_idx ON transfer_batches (workspace_id, transfer_id, batch_id)
    WHERE deleted_at IS NULL;
CREATE INDEX transfer_batches_batch_idx ON transfer_batches (workspace_id, batch_id);

-- packaging_runs, packaging_run_lines ---------------------------------------------------------
CREATE TABLE packaging_runs (
    id            uuid PRIMARY KEY,
    workspace_id  uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    packaged_at   timestamptz NOT NULL DEFAULT now(),
    vessel_id     uuid,
    notes         text,
    created_at    timestamptz NOT NULL DEFAULT now(),
    updated_at    timestamptz NOT NULL DEFAULT now(),
    deleted_at    timestamptz,
    version       bigint NOT NULL DEFAULT 1,
    created_by    uuid REFERENCES users (id),
    updated_by    uuid REFERENCES users (id),
    UNIQUE (workspace_id, id),
    FOREIGN KEY (workspace_id, vessel_id) REFERENCES vessels (workspace_id, id)
);
CREATE INDEX packaging_runs_time_idx ON packaging_runs (workspace_id, packaged_at);

CREATE TABLE packaging_run_lines (
    id                 uuid PRIMARY KEY,
    workspace_id       uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    run_id             uuid NOT NULL,
    batch_id           uuid NOT NULL,
    -- Kap türü ortak olabilir (varsayılan kaplar): shared_ref_package_type_id.
    package_type_id    uuid NOT NULL REFERENCES package_types (id),
    quantity           integer NOT NULL CHECK (quantity > 0),
    volume_l           double precision NOT NULL CHECK (volume_l > 0),
    -- Ürün lotu (izlenebilirlik: hammadde lotundan ambalaja).
    lot_code           text NOT NULL CHECK (char_length(lot_code) BETWEEN 1 AND 100),
    carbonation_method text CHECK (carbonation_method IN ('forced', 'priming', 'natural', 'spunding')),
    created_at         timestamptz NOT NULL DEFAULT now(),
    updated_at         timestamptz NOT NULL DEFAULT now(),
    deleted_at         timestamptz,
    version            bigint NOT NULL DEFAULT 1,
    created_by         uuid REFERENCES users (id),
    updated_by         uuid REFERENCES users (id),
    UNIQUE (workspace_id, id),
    FOREIGN KEY (workspace_id, run_id) REFERENCES packaging_runs (workspace_id, id) ON DELETE CASCADE,
    FOREIGN KEY (workspace_id, batch_id) REFERENCES batches (workspace_id, id) ON DELETE CASCADE
);
CREATE TRIGGER shared_ref_package_type_id BEFORE INSERT OR UPDATE OF package_type_id, workspace_id ON packaging_run_lines
    FOR EACH ROW EXECUTE FUNCTION check_shared_ref('package_types', 'package_type_id');
CREATE INDEX packaging_run_lines_batch_idx ON packaging_run_lines (workspace_id, batch_id);
CREATE INDEX packaging_run_lines_lot_idx ON packaging_run_lines (workspace_id, lot_code);

-- quality_specs, lab_results, releases ---------------------------------------------------------
-- Parametreler kendi kanonik biriminde saklanır: abv_pct (%), og ve fg (SG), ibu, srm, ph,
-- co2_vols (hacim), dissolved_oxygen_ppb, turbidity_ntu, diacetyl_ppb, microbiology (1 geçti, 0 kaldı),
-- sensory_score (1-5).
CREATE TABLE quality_specs (
    id            uuid PRIMARY KEY,
    workspace_id  uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    recipe_id     uuid NOT NULL,
    parameter     text NOT NULL CHECK (parameter IN (
        'abv_pct', 'og', 'fg', 'ibu', 'srm', 'ph', 'co2_vols', 'dissolved_oxygen_ppb',
        'turbidity_ntu', 'diacetyl_ppb', 'microbiology', 'sensory_score')),
    min_value     double precision,
    max_value     double precision,
    required      boolean NOT NULL DEFAULT true,
    notes         text,
    created_at    timestamptz NOT NULL DEFAULT now(),
    updated_at    timestamptz NOT NULL DEFAULT now(),
    deleted_at    timestamptz,
    version       bigint NOT NULL DEFAULT 1,
    created_by    uuid REFERENCES users (id),
    updated_by    uuid REFERENCES users (id),
    UNIQUE (workspace_id, id),
    CHECK (min_value IS NOT NULL OR max_value IS NOT NULL),
    CHECK (min_value IS NULL OR max_value IS NULL OR min_value <= max_value),
    FOREIGN KEY (workspace_id, recipe_id) REFERENCES recipes (workspace_id, id) ON DELETE CASCADE
);
CREATE UNIQUE INDEX quality_specs_unique_idx ON quality_specs (workspace_id, recipe_id, parameter)
    WHERE deleted_at IS NULL;

-- Laboratuvar ve tadım sonuçları yalnız eklenir; düzeltme `corrects_id` ile yeni satırdır.
CREATE TABLE lab_results (
    id            uuid PRIMARY KEY,
    workspace_id  uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    batch_id      uuid NOT NULL,
    spec_id       uuid,
    parameter     text NOT NULL CHECK (parameter IN (
        'abv_pct', 'og', 'fg', 'ibu', 'srm', 'ph', 'co2_vols', 'dissolved_oxygen_ppb',
        'turbidity_ntu', 'diacetyl_ppb', 'microbiology', 'sensory_score')),
    value         double precision NOT NULL,
    source        text NOT NULL CHECK (source IN ('lab', 'in_house', 'tasting')),
    lab_name      text CHECK (char_length(lab_name) <= 200),
    sample_code   text CHECK (char_length(sample_code) <= 100),
    measured_at   timestamptz NOT NULL DEFAULT now(),
    corrects_id   uuid,
    notes         text,
    created_at    timestamptz NOT NULL DEFAULT now(),
    created_by    uuid REFERENCES users (id),
    UNIQUE (workspace_id, id),
    FOREIGN KEY (workspace_id, batch_id) REFERENCES batches (workspace_id, id) ON DELETE CASCADE,
    FOREIGN KEY (workspace_id, spec_id) REFERENCES quality_specs (workspace_id, id),
    FOREIGN KEY (workspace_id, corrects_id) REFERENCES lab_results (workspace_id, id)
);
CREATE INDEX lab_results_batch_idx ON lab_results (workspace_id, batch_id, measured_at);

-- Serbest bırakma kararı yalnız eklenir (kim, ne zaman); geçerli karar partinin son satırıdır.
CREATE TABLE releases (
    id               uuid PRIMARY KEY,
    workspace_id     uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    batch_id         uuid NOT NULL,
    packaging_run_id uuid,
    decision         text NOT NULL CHECK (decision IN ('released', 'held', 'rejected')),
    decided_by       uuid NOT NULL REFERENCES users (id),
    decided_at       timestamptz NOT NULL DEFAULT now(),
    reason           text,
    created_at       timestamptz NOT NULL DEFAULT now(),
    created_by       uuid REFERENCES users (id),
    UNIQUE (workspace_id, id),
    -- Bekletme ve red gerekçesiz olmaz.
    CHECK (decision = 'released' OR char_length(btrim(coalesce(reason, ''))) > 0),
    FOREIGN KEY (workspace_id, batch_id) REFERENCES batches (workspace_id, id) ON DELETE CASCADE,
    FOREIGN KEY (workspace_id, packaging_run_id) REFERENCES packaging_runs (workspace_id, id)
);
CREATE INDEX releases_batch_idx ON releases (workspace_id, batch_id, decided_at);

-- cleaning_records ---------------------------------------------------------------------------
CREATE TABLE cleaning_records (
    id                  uuid PRIMARY KEY,
    workspace_id        uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    vessel_id           uuid NOT NULL,
    kind                text NOT NULL DEFAULT 'cip' CHECK (kind IN ('cip', 'sanitize', 'rinse', 'manual')),
    performed_at        timestamptz NOT NULL DEFAULT now(),
    performed_by        uuid REFERENCES users (id),
    chemical            text CHECK (char_length(chemical) <= 200),
    concentration_pct   double precision CHECK (concentration_pct >= 0),
    temp_c              double precision,
    duration_min        double precision CHECK (duration_min >= 0),
    verification_method text CHECK (verification_method IN ('visual', 'atp', 'ph_strip', 'other')),
    verified_by         uuid REFERENCES users (id),
    verified_at         timestamptz,
    notes               text,
    created_at          timestamptz NOT NULL DEFAULT now(),
    updated_at          timestamptz NOT NULL DEFAULT now(),
    deleted_at          timestamptz,
    version             bigint NOT NULL DEFAULT 1,
    created_by          uuid REFERENCES users (id),
    updated_by          uuid REFERENCES users (id),
    UNIQUE (workspace_id, id),
    -- Doğrulama kim, ne zaman ve hangi yöntemle birlikte yazılır.
    CHECK ((verified_by IS NULL) = (verified_at IS NULL)),
    CHECK (verified_by IS NULL OR verification_method IS NOT NULL),
    FOREIGN KEY (workspace_id, vessel_id) REFERENCES vessels (workspace_id, id) ON DELETE CASCADE
);
CREATE INDEX cleaning_records_vessel_idx ON cleaning_records (workspace_id, vessel_id, performed_at);

-- yeast_generations, yeast_generation_batches ----------------------------------------------------
CREATE TABLE yeast_generations (
    id                  uuid PRIMARY KEY,
    workspace_id        uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    yeast_ingredient_id uuid NOT NULL REFERENCES ingredients (id),
    generation          integer NOT NULL CHECK (generation >= 0),
    parent_id           uuid,
    source_batch_id     uuid,
    harvested_at        timestamptz NOT NULL DEFAULT now(),
    slurry_ml           double precision CHECK (slurry_ml >= 0),
    cell_count_billion  double precision CHECK (cell_count_billion >= 0),
    viability_pct       double precision CHECK (viability_pct BETWEEN 0 AND 100),
    discarded_at        timestamptz,
    notes               text,
    created_at          timestamptz NOT NULL DEFAULT now(),
    updated_at          timestamptz NOT NULL DEFAULT now(),
    deleted_at          timestamptz,
    version             bigint NOT NULL DEFAULT 1,
    created_by          uuid REFERENCES users (id),
    updated_by          uuid REFERENCES users (id),
    UNIQUE (workspace_id, id),
    CHECK (parent_id IS NULL OR generation >= 1),
    FOREIGN KEY (workspace_id, parent_id) REFERENCES yeast_generations (workspace_id, id),
    FOREIGN KEY (workspace_id, source_batch_id) REFERENCES batches (workspace_id, id)
);
CREATE TRIGGER shared_ref_yeast_ingredient_id BEFORE INSERT OR UPDATE OF yeast_ingredient_id, workspace_id ON yeast_generations
    FOR EACH ROW EXECUTE FUNCTION check_shared_ref('ingredients', 'yeast_ingredient_id');
CREATE INDEX yeast_generations_parent_idx ON yeast_generations (workspace_id, parent_id);

-- Üst jenerasyon aynı maya kaydından ve bir önceki jenerasyondan olmalıdır; zincirde döngü
-- kurulamaz (jenerasyon numarası her adımda artar).
CREATE FUNCTION yeast_generation_check_parent() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE
    p yeast_generations%ROWTYPE;
BEGIN
    IF NEW.parent_id IS NULL THEN
        RETURN NEW;
    END IF;
    SELECT * INTO p FROM yeast_generations
    WHERE id = NEW.parent_id AND workspace_id = NEW.workspace_id;
    IF p.id IS NULL OR p.yeast_ingredient_id <> NEW.yeast_ingredient_id
       OR p.generation <> NEW.generation - 1 THEN
        RAISE EXCEPTION 'parent generation must be of the same yeast and exactly one generation earlier'
            USING ERRCODE = 'integrity_constraint_violation';
    END IF;
    RETURN NEW;
END
$$;
CREATE TRIGGER parent_check BEFORE INSERT OR UPDATE OF parent_id, generation, yeast_ingredient_id ON yeast_generations
    FOR EACH ROW EXECUTE FUNCTION yeast_generation_check_parent();

-- Hasadın hangi partilere atıldığı (hedef partiler).
CREATE TABLE yeast_generation_batches (
    id             uuid PRIMARY KEY,
    workspace_id   uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    generation_id  uuid NOT NULL,
    batch_id       uuid NOT NULL,
    pitched_at     timestamptz NOT NULL DEFAULT now(),
    pitch_ml       double precision CHECK (pitch_ml >= 0),
    created_at     timestamptz NOT NULL DEFAULT now(),
    updated_at     timestamptz NOT NULL DEFAULT now(),
    deleted_at     timestamptz,
    version        bigint NOT NULL DEFAULT 1,
    created_by     uuid REFERENCES users (id),
    updated_by     uuid REFERENCES users (id),
    UNIQUE (workspace_id, id),
    FOREIGN KEY (workspace_id, generation_id) REFERENCES yeast_generations (workspace_id, id) ON DELETE CASCADE,
    FOREIGN KEY (workspace_id, batch_id) REFERENCES batches (workspace_id, id) ON DELETE CASCADE
);
CREATE UNIQUE INDEX yeast_generation_batches_unique_idx
    ON yeast_generation_batches (workspace_id, generation_id, batch_id) WHERE deleted_at IS NULL;
CREATE INDEX yeast_generation_batches_batch_idx ON yeast_generation_batches (workspace_id, batch_id);

-- Kapsam bütünlüğü: vessel ve area türleri artık çalışır (check_polymorphic_target 0013'te
-- tablo adını zaten tanır). Hedef silinince bağlı atama, davet ve alarm kuralı da gider;
-- drop_scope_references `vessel` için alarm kurallarını da kapsayacak biçimde yenilenir.
CREATE OR REPLACE FUNCTION drop_scope_references() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    DELETE FROM role_assignments
    WHERE workspace_id = OLD.workspace_id AND scope_kind = TG_ARGV[0] AND scope_id = OLD.id;
    DELETE FROM invitations
    WHERE workspace_id = OLD.workspace_id AND scope_kind = TG_ARGV[0] AND scope_id = OLD.id;
    IF TG_ARGV[0] IN ('batch', 'device', 'vessel') THEN
        DELETE FROM alert_rules
        WHERE workspace_id = OLD.workspace_id AND scope_kind = TG_ARGV[0] AND scope_id = OLD.id;
    END IF;
    IF TG_ARGV[1] IS NOT NULL THEN
        DELETE FROM attachments
        WHERE workspace_id = OLD.workspace_id AND record_kind = TG_ARGV[1] AND record_id = OLD.id;
    END IF;
    RETURN OLD;
END
$$;
CREATE TRIGGER drop_scope AFTER DELETE ON vessels
    FOR EACH ROW EXECUTE FUNCTION drop_scope_references('vessel');
CREATE TRIGGER drop_scope AFTER DELETE ON production_areas
    FOR EACH ROW EXECUTE FUNCTION drop_scope_references('area');

SELECT secure_workspace_table(t) FROM unnest(ARRAY[
    'production_areas', 'vessels', 'vessel_occupancies', 'transfers', 'transfer_batches',
    'packaging_runs', 'packaging_run_lines', 'quality_specs', 'cleaning_records',
    'yeast_generations', 'yeast_generation_batches']) t;
-- Karar ve sonuç kayıtları yalnız eklenir (düzeltme yeni satırdır).
SELECT secure_workspace_table(t, 'append', 'SELECT, INSERT') FROM unnest(ARRAY[
    'lab_results', 'releases']) t;

RESET ROLE;
