-- Parti ve demleme günü (adr/0007 › 3 "Parti ve demleme günü"; T2.6).
SET LOCAL ROLE app_owner;

-- batches ------------------------------------------------------------------------------
CREATE TABLE batches (
    id                 uuid PRIMARY KEY,
    workspace_id       uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    -- İnsana gösterilen numara; çalışma alanında tekil (adr/0007 › 1.1).
    code               text NOT NULL CHECK (char_length(code) BETWEEN 1 AND 50),
    name               text NOT NULL DEFAULT '' CHECK (char_length(name) <= 200),
    recipe_version_id  uuid NOT NULL,
    equipment_id       uuid,
    stage              text NOT NULL DEFAULT 'planned' CHECK (stage IN (
        'planned', 'brewing', 'fermenting', 'conditioning', 'packaging', 'completed', 'cancelled')),
    planned_date       date,
    started_at         timestamptz,
    completed_at       timestamptz,
    target_volume_l    double precision CHECK (target_volume_l > 0),
    actual_volume_l    double precision CHECK (actual_volume_l >= 0),
    engine_version     text,
    notes              text,
    search             tsvector GENERATED ALWAYS AS (
                           to_tsvector('simple', code || ' ' || name || ' ' || coalesce(notes, ''))
                       ) STORED,
    created_at         timestamptz NOT NULL DEFAULT now(),
    updated_at         timestamptz NOT NULL DEFAULT now(),
    deleted_at         timestamptz,
    version            bigint NOT NULL DEFAULT 1,
    created_by         uuid REFERENCES users (id),
    updated_by         uuid REFERENCES users (id),
    UNIQUE (workspace_id, id),
    FOREIGN KEY (workspace_id, recipe_version_id) REFERENCES recipe_versions (workspace_id, id),
    FOREIGN KEY (workspace_id, equipment_id) REFERENCES equipment (workspace_id, id)
);
CREATE UNIQUE INDEX batches_code_idx ON batches (workspace_id, code) WHERE deleted_at IS NULL;
CREATE INDEX batches_search_idx ON batches USING gin (search);
CREATE INDEX batches_name_trgm_idx ON batches USING gin (name gin_trgm_ops);
CREATE INDEX batches_stage_idx ON batches (workspace_id, stage) WHERE deleted_at IS NULL;
CREATE INDEX batches_recipe_version_idx ON batches (workspace_id, recipe_version_id);

-- Parti yalnız yayınlanmış (değişmez) sürümden açılır: İşletmede `approved` (adr/0007 › 3),
-- Evde `approved` ya da sonradan `archived` olmuş sürüm. Taslak ve incelemedeki sürüm
-- değişebildiği için partinin tarifi olamaz.
CREATE FUNCTION batches_check_version() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE
    s text;
BEGIN
    SELECT status INTO s FROM recipe_versions
    WHERE id = NEW.recipe_version_id AND workspace_id = NEW.workspace_id;
    IF s IS NULL OR s NOT IN ('approved', 'archived') THEN
        RAISE EXCEPTION 'a batch needs a published recipe version (status is %)', coalesce(s, 'missing')
            USING ERRCODE = 'integrity_constraint_violation';
    END IF;
    IF s = 'archived' AND EXISTS (SELECT 1 FROM workspaces
                                  WHERE id = NEW.workspace_id AND mode = 'business') THEN
        RAISE EXCEPTION 'a business batch needs an approved recipe version'
            USING ERRCODE = 'integrity_constraint_violation';
    END IF;
    RETURN NEW;
END
$$;
CREATE TRIGGER version_check BEFORE INSERT OR UPDATE OF recipe_version_id ON batches
    FOR EACH ROW EXECUTE FUNCTION batches_check_version();

CREATE TABLE batch_tags (
    workspace_id  uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    batch_id      uuid NOT NULL,
    tag_id        uuid NOT NULL,
    created_at    timestamptz NOT NULL DEFAULT now(),
    created_by    uuid REFERENCES users (id),
    PRIMARY KEY (workspace_id, batch_id, tag_id),
    FOREIGN KEY (workspace_id, batch_id) REFERENCES batches (workspace_id, id) ON DELETE CASCADE,
    FOREIGN KEY (workspace_id, tag_id) REFERENCES tags (workspace_id, id) ON DELETE CASCADE
);

-- Favori kullanıcıya özeldir: çalışma alanı politikasına ek kısıtlayıcı kullanıcı politikası.
CREATE TABLE favorites (
    id            uuid PRIMARY KEY,
    workspace_id  uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    user_id       uuid NOT NULL REFERENCES users (id),
    recipe_id     uuid,
    batch_id      uuid,
    created_at    timestamptz NOT NULL DEFAULT now(),
    CHECK (num_nonnulls(recipe_id, batch_id) = 1),
    FOREIGN KEY (workspace_id, recipe_id) REFERENCES recipes (workspace_id, id) ON DELETE CASCADE,
    FOREIGN KEY (workspace_id, batch_id) REFERENCES batches (workspace_id, id) ON DELETE CASCADE
);
CREATE UNIQUE INDEX favorites_unique_idx ON favorites (workspace_id, user_id, recipe_id, batch_id)
    NULLS NOT DISTINCT;

-- batch_steps --------------------------------------------------------------------------
CREATE TABLE batch_steps (
    id                uuid PRIMARY KEY,
    workspace_id      uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    batch_id          uuid NOT NULL,
    position          integer NOT NULL CHECK (position >= 0),
    kind              text NOT NULL CHECK (kind ~ '^[a-z][a-z0-9_]*$'),
    name              text NOT NULL DEFAULT '' CHECK (char_length(name) <= 200),
    planned_start_at  timestamptz,
    planned_end_at    timestamptz,
    actual_start_at   timestamptz,
    actual_end_at     timestamptz,
    status            text NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'running', 'done', 'skipped')),
    input_source      text CHECK (input_source IN ('timer', 'manual', 'paper')),
    closed_by         uuid REFERENCES users (id),
    created_at        timestamptz NOT NULL DEFAULT now(),
    updated_at        timestamptz NOT NULL DEFAULT now(),
    deleted_at        timestamptz,
    version           bigint NOT NULL DEFAULT 1,
    created_by        uuid REFERENCES users (id),
    updated_by        uuid REFERENCES users (id),
    UNIQUE (workspace_id, id),
    UNIQUE (workspace_id, batch_id, position) DEFERRABLE INITIALLY DEFERRED,
    FOREIGN KEY (workspace_id, batch_id) REFERENCES batches (workspace_id, id) ON DELETE CASCADE,
    CHECK (planned_end_at IS NULL OR planned_start_at IS NULL OR planned_end_at >= planned_start_at),
    CHECK (actual_end_at IS NULL OR actual_start_at IS NULL OR actual_end_at >= actual_start_at)
);

-- batch_ingredient_actuals -------------------------------------------------------------
-- İlgili tarif satırı dört satır tablosundan birindedir; yabancı anahtar kurulamaz, tetikleyici
-- hedefin partinin tarif sürümüne ait olduğunu denetler (aşağıda). Stok lotu 0010'da bağlanır.
CREATE TABLE batch_ingredient_actuals (
    id                       uuid PRIMARY KEY,
    workspace_id             uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    batch_id                 uuid NOT NULL,
    line_kind                text CHECK (line_kind IN ('fermentable', 'hop', 'yeast', 'misc', 'water_addition')),
    line_id                  uuid,
    substitute_ingredient_id uuid REFERENCES ingredients (id),
    lot_id                   uuid,
    actual_amount_kg         double precision NOT NULL CHECK (actual_amount_kg >= 0),
    notes                    text,
    created_at               timestamptz NOT NULL DEFAULT now(),
    updated_at               timestamptz NOT NULL DEFAULT now(),
    deleted_at               timestamptz,
    version                  bigint NOT NULL DEFAULT 1,
    created_by               uuid REFERENCES users (id),
    updated_by               uuid REFERENCES users (id),
    UNIQUE (workspace_id, id),
    FOREIGN KEY (workspace_id, batch_id) REFERENCES batches (workspace_id, id) ON DELETE CASCADE,
    CHECK ((line_kind IS NULL) = (line_id IS NULL)),
    -- Satırsız kayıt (plan dışı eklenen malzeme) malzemesini `substitute_ingredient_id` ile bildirir.
    CHECK (line_id IS NOT NULL OR substitute_ingredient_id IS NOT NULL)
);
CREATE INDEX batch_ingredient_actuals_batch_idx ON batch_ingredient_actuals (workspace_id, batch_id);
CREATE TRIGGER shared_ref_substitute_ingredient_id
    BEFORE INSERT OR UPDATE OF substitute_ingredient_id, workspace_id ON batch_ingredient_actuals
    FOR EACH ROW EXECUTE FUNCTION check_shared_ref('ingredients', 'substitute_ingredient_id');

CREATE FUNCTION batch_actual_check_line() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE
    tbl text;
    n bigint;
BEGIN
    IF NEW.line_id IS NULL THEN
        RETURN NEW;
    END IF;
    tbl := CASE NEW.line_kind
        WHEN 'fermentable' THEN 'recipe_fermentables'
        WHEN 'hop' THEN 'recipe_hops'
        WHEN 'yeast' THEN 'recipe_yeasts'
        WHEN 'misc' THEN 'recipe_miscs'
        WHEN 'water_addition' THEN 'recipe_water_additions' END;
    EXECUTE format(
        'SELECT count(*) FROM %I l JOIN batches b ON b.workspace_id = l.workspace_id
                                                  AND b.recipe_version_id = l.recipe_version_id
         WHERE l.id = $1 AND l.workspace_id = $2 AND b.id = $3', tbl)
        INTO n USING NEW.line_id, NEW.workspace_id, NEW.batch_id;
    IF n = 0 THEN
        RAISE EXCEPTION '% % is not a line of this batch''s recipe version', NEW.line_kind, NEW.line_id
            USING ERRCODE = 'foreign_key_violation';
    END IF;
    RETURN NEW;
END
$$;
CREATE TRIGGER line_check BEFORE INSERT OR UPDATE OF line_kind, line_id, batch_id ON batch_ingredient_actuals
    FOR EACH ROW EXECUTE FUNCTION batch_actual_check_line();

-- deviations ---------------------------------------------------------------------------
CREATE TABLE deviations (
    id            uuid PRIMARY KEY,
    workspace_id  uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    batch_id      uuid NOT NULL,
    step_id       uuid,
    actual_id     uuid,
    kind          text NOT NULL CHECK (kind ~ '^[a-z][a-z0-9_]*$'),
    description   text NOT NULL DEFAULT '',
    -- İşletmede kalite sapma kaydı da bu tablodur (adr/0007 › 3).
    severity      text NOT NULL DEFAULT 'info' CHECK (severity IN ('info', 'minor', 'major', 'critical')),
    status        text NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'closed')),
    closed_at     timestamptz,
    closed_by     uuid REFERENCES users (id),
    closure_note  text,
    created_at    timestamptz NOT NULL DEFAULT now(),
    updated_at    timestamptz NOT NULL DEFAULT now(),
    deleted_at    timestamptz,
    version       bigint NOT NULL DEFAULT 1,
    created_by    uuid REFERENCES users (id),
    updated_by    uuid REFERENCES users (id),
    UNIQUE (workspace_id, id),
    FOREIGN KEY (workspace_id, batch_id) REFERENCES batches (workspace_id, id) ON DELETE CASCADE,
    FOREIGN KEY (workspace_id, step_id) REFERENCES batch_steps (workspace_id, id),
    FOREIGN KEY (workspace_id, actual_id) REFERENCES batch_ingredient_actuals (workspace_id, id),
    CHECK ((status = 'closed') = (closed_at IS NOT NULL))
);
CREATE INDEX deviations_batch_idx ON deviations (workspace_id, batch_id);

-- brew_day_packages --------------------------------------------------------------------
CREATE TABLE brew_day_packages (
    id               uuid PRIMARY KEY,
    workspace_id     uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    batch_id         uuid NOT NULL,
    content_version  integer NOT NULL CHECK (content_version >= 1),
    content          jsonb NOT NULL CHECK (jsonb_typeof(content) = 'object' AND content ? 'schema_version'),
    signature        bytea NOT NULL,
    key_id           text NOT NULL CHECK (char_length(key_id) BETWEEN 1 AND 100),
    expires_at       timestamptz,
    revoked_at       timestamptz,
    created_at       timestamptz NOT NULL DEFAULT now(),
    updated_at       timestamptz NOT NULL DEFAULT now(),
    deleted_at       timestamptz,
    version          bigint NOT NULL DEFAULT 1,
    created_by       uuid REFERENCES users (id),
    updated_by       uuid REFERENCES users (id),
    UNIQUE (workspace_id, id),
    UNIQUE (workspace_id, batch_id, content_version),
    FOREIGN KEY (workspace_id, batch_id) REFERENCES batches (workspace_id, id) ON DELETE CASCADE
);

-- devices ------------------------------------------------------------------------------
-- Cihaz veri adresi anahtarı yalnız SHA-256 özetiyle saklanır. vessel_id (tank) için yabancı
-- anahtar İşletme göçünde (T2.7) vessels ile birlikte eklenir.
CREATE TABLE devices (
    id                   uuid PRIMARY KEY,
    workspace_id         uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    kind                 text NOT NULL CHECK (kind IN ('ispindel', 'rapt', 'tilt', 'other')),
    name                 text NOT NULL CHECK (char_length(name) BETWEEN 1 AND 200),
    ingest_key_hash      bytea NOT NULL UNIQUE CHECK (octet_length(ingest_key_hash) = 32),
    ingest_key_rotated_at timestamptz NOT NULL DEFAULT now(),
    batch_id             uuid,
    vessel_id            uuid,
    calibration          jsonb CHECK (jsonb_typeof(calibration) = 'object' AND calibration ? 'schema_version'),
    last_data_at         timestamptz,
    created_at           timestamptz NOT NULL DEFAULT now(),
    updated_at           timestamptz NOT NULL DEFAULT now(),
    deleted_at           timestamptz,
    version              bigint NOT NULL DEFAULT 1,
    created_by           uuid REFERENCES users (id),
    updated_by           uuid REFERENCES users (id),
    UNIQUE (workspace_id, id),
    FOREIGN KEY (workspace_id, batch_id) REFERENCES batches (workspace_id, id)
);

-- measurements -------------------------------------------------------------------------
-- measured_at üzerinde aylık aralık bölümlü, yalnız eklenir. Birincil anahtar bölüm anahtarını
-- içerir: istemci aynı ölçümü aynı UUID ve zamanla yeniden gönderir, `ON CONFLICT DO NOTHING`
-- ikinci kayıt oluşturmaz. Bölümleri create_measurement_partition() açar; her bölümde RLS ve
-- politika ayrıca kurulur (politika bölümlere kalıtılmaz). Değer SI birimindedir (adr/0007 › 1.4).
CREATE TABLE measurements (
    id            uuid NOT NULL,
    workspace_id  uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    batch_id      uuid NOT NULL,
    kind          text NOT NULL CHECK (kind IN (
        'sg', 'temp_c', 'ph', 'pressure_bar', 'volume_l', 'abv_pct', 'battery_v', 'signal_dbm')),
    value         double precision NOT NULL,
    measured_at   timestamptz NOT NULL,
    source        text NOT NULL CHECK (source IN ('device', 'manual', 'paper')),
    device_id     uuid,
    package_id    uuid,
    created_at    timestamptz NOT NULL DEFAULT now(),
    created_by    uuid REFERENCES users (id),
    PRIMARY KEY (id, measured_at),
    FOREIGN KEY (workspace_id, batch_id) REFERENCES batches (workspace_id, id) ON DELETE CASCADE,
    FOREIGN KEY (workspace_id, device_id) REFERENCES devices (workspace_id, id),
    FOREIGN KEY (workspace_id, package_id) REFERENCES brew_day_packages (workspace_id, id),
    CHECK ((source = 'device') = (device_id IS NOT NULL))
) PARTITION BY RANGE (measured_at);
CREATE INDEX measurements_batch_idx ON measurements (workspace_id, batch_id, measured_at);
CREATE INDEX measurements_device_idx ON measurements (workspace_id, device_id, measured_at)
    WHERE device_id IS NOT NULL;

-- Verilen ayın bölümünü (UTC, ayın ilk günü) açar; varsa dokunmaz. Bölüm app_owner'ındır.
-- Bölüm sınırı ay başıdır; uygulama saatleri UTC gönderir.
CREATE FUNCTION create_measurement_partition(p_month date) RETURNS text
    LANGUAGE plpgsql SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
DECLARE
    first_day date := date_trunc('month', p_month)::date;
    part text := format('measurements_y%sm%s', to_char(first_day, 'YYYY'), to_char(first_day, 'MM'));
BEGIN
    IF to_regclass(format('public.%I', part)) IS NULL THEN
        EXECUTE format(
            'CREATE TABLE public.%I PARTITION OF public.measurements
                 FOR VALUES FROM (%L) TO (%L)',
            part,
            first_day::timestamp AT TIME ZONE 'UTC',
            (first_day + interval '1 month')::timestamp AT TIME ZONE 'UTC');
        EXECUTE format('ALTER TABLE public.%I ENABLE ROW LEVEL SECURITY', part);
        EXECUTE format('ALTER TABLE public.%I FORCE ROW LEVEL SECURITY', part);
        EXECUTE format(
            'CREATE POLICY workspace_isolation ON public.%I
                 USING (workspace_id = (SELECT app_workspace_id()))
                 WITH CHECK (workspace_id = (SELECT app_workspace_id()))', part);
    END IF;
    RETURN part;
END
$$;

-- Bu aydan başlayarak p_months_ahead ay sonrasına kadar bölümleri açar (worker iki ay önceden).
CREATE FUNCTION ensure_measurement_partitions(p_months_ahead integer DEFAULT 2) RETURNS integer
    LANGUAGE plpgsql SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
DECLARE
    m integer;
    created integer := 0;
    part text;
BEGIN
    IF p_months_ahead NOT BETWEEN 0 AND 24 THEN
        RAISE EXCEPTION 'p_months_ahead must be between 0 and 24';
    END IF;
    FOR m IN 0..p_months_ahead LOOP
        part := format('measurements_y%sm%s',
            to_char(date_trunc('month', now() AT TIME ZONE 'UTC') + make_interval(months => m), 'YYYY'),
            to_char(date_trunc('month', now() AT TIME ZONE 'UTC') + make_interval(months => m), 'MM'));
        IF to_regclass(format('public.%I', part)) IS NULL THEN
            PERFORM create_measurement_partition(
                (date_trunc('month', now() AT TIME ZONE 'UTC') + make_interval(months => m))::date);
            created := created + 1;
        END IF;
    END LOOP;
    RETURN created;
END
$$;
REVOKE ALL ON FUNCTION create_measurement_partition(date), ensure_measurement_partitions(integer) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ensure_measurement_partitions(integer) TO app_worker;

-- İlk bölümler: 2026-01'den bugünden iki ay sonrasına (sabit başlangıç, göçten bağımsız).
SELECT create_measurement_partition(m::date)
FROM generate_series(
    DATE '2026-01-01',
    date_trunc('month', now() AT TIME ZONE 'UTC') + interval '2 months',
    interval '1 month') m;

-- stage_events, tasting_notes, calc_results ----------------------------------------------
CREATE TABLE stage_events (
    id            uuid PRIMARY KEY,
    workspace_id  uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    batch_id      uuid NOT NULL,
    from_stage    text CHECK (from_stage IN (
        'planned', 'brewing', 'fermenting', 'conditioning', 'packaging', 'completed', 'cancelled')),
    to_stage      text NOT NULL CHECK (to_stage IN (
        'planned', 'brewing', 'fermenting', 'conditioning', 'packaging', 'completed', 'cancelled')),
    occurred_at   timestamptz NOT NULL DEFAULT now(),
    created_at    timestamptz NOT NULL DEFAULT now(),
    created_by    uuid REFERENCES users (id),
    FOREIGN KEY (workspace_id, batch_id) REFERENCES batches (workspace_id, id) ON DELETE CASCADE
);
CREATE INDEX stage_events_batch_idx ON stage_events (workspace_id, batch_id, occurred_at);

CREATE TABLE tasting_notes (
    id            uuid PRIMARY KEY,
    workspace_id  uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    batch_id      uuid NOT NULL,
    rating        smallint CHECK (rating BETWEEN 1 AND 5),
    note          text NOT NULL DEFAULT '',
    tasted_at     timestamptz NOT NULL DEFAULT now(),
    created_at    timestamptz NOT NULL DEFAULT now(),
    updated_at    timestamptz NOT NULL DEFAULT now(),
    deleted_at    timestamptz,
    version       bigint NOT NULL DEFAULT 1,
    created_by    uuid REFERENCES users (id),
    updated_by    uuid REFERENCES users (id),
    UNIQUE (workspace_id, id),
    FOREIGN KEY (workspace_id, batch_id) REFERENCES batches (workspace_id, id) ON DELETE CASCADE
);

-- Değişmez hesap sonucu. Redis önbelleği bunun önündedir ve kaybolabilir (adr/0007 › 3).
-- Aynı girdi iki hedefe bağlanamaz (tekillik girdi başınadır); bağlantı bilgi amaçlıdır.
CREATE TABLE calc_results (
    id                 uuid PRIMARY KEY,
    workspace_id       uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    kind               text NOT NULL CHECK (kind ~ '^[a-z][a-z0-9_]*$'),
    input_hash         bytea NOT NULL CHECK (octet_length(input_hash) = 32),
    engine_version     text NOT NULL CHECK (char_length(engine_version) BETWEEN 1 AND 100),
    output             jsonb NOT NULL CHECK (jsonb_typeof(output) = 'object' AND output ? 'schema_version'),
    recipe_version_id  uuid,
    batch_id           uuid,
    created_at         timestamptz NOT NULL DEFAULT now(),
    created_by         uuid REFERENCES users (id),
    UNIQUE (workspace_id, kind, input_hash, engine_version),
    FOREIGN KEY (workspace_id, recipe_version_id) REFERENCES recipe_versions (workspace_id, id) ON DELETE CASCADE,
    FOREIGN KEY (workspace_id, batch_id) REFERENCES batches (workspace_id, id) ON DELETE CASCADE,
    CHECK (num_nonnulls(recipe_version_id, batch_id) <= 1)
);

-- RLS, sürüm tetikleyicisi ve haklar ---------------------------------------------------------
SELECT secure_workspace_table(t) FROM unnest(ARRAY[
    'batches', 'batch_steps', 'batch_ingredient_actuals', 'deviations',
    'brew_day_packages', 'devices', 'tasting_notes']) t;
SELECT secure_workspace_table('batch_tags', 'standard', 'SELECT, INSERT, DELETE');
SELECT secure_workspace_table('favorites', 'standard', 'SELECT, INSERT, DELETE');
CREATE POLICY own_favorites ON favorites AS RESTRICTIVE
    USING (user_id = (SELECT app_user_id())) WITH CHECK (user_id = (SELECT app_user_id()));
-- Yalnız eklenen tablolar: bağlı oldukları kayıtla birlikte (yabancı anahtar zinciri) gider.
SELECT secure_workspace_table(t, 'append', 'SELECT, INSERT') FROM unnest(ARRAY[
    'measurements', 'stage_events']) t;
SELECT secure_workspace_table('calc_results', 'append', 'SELECT, INSERT, DELETE');

RESET ROLE;
