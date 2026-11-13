-- Kütüphane ve tanımlar (adr/0007 › 3 "Kütüphane ve tanımlar"; T2.6).
-- Ortak kütüphane (workspace_id NULL) ile çalışma alanına özel kayıtlar aynı tabloda tutulur:
-- okuma `workspace_id IS NULL OR workspace_id = bağlam`, yazma yalnız `workspace_id = bağlam`;
-- ortak satırı yalnız app_owner (göç ve tohum) yazar (adr/0007 › 2.2).
-- `pg_trgm` yönetilen PostgreSQL'lerde ek yetki gerektirmeyen güvenilir uzantıdır (adr/0007 › 1.10).
CREATE EXTENSION IF NOT EXISTS pg_trgm;

SET LOCAL ROLE app_owner;

-- Ortak kurulum işlevi --------------------------------------------------------------------
-- Tablo göçleri RLS'yi, politikayı, sürüm tetikleyicisini ve hakları bu işlevle kurar (T2.7 de
-- kullanır). Kipler:
--   standard : workspace_id = bağlam (okuma ve yazma)
--   shared   : okuma `workspace_id IS NULL OR = bağlam`, yazma yalnız `= bağlam`; ek olarak
--              ortak satırları göç ve tohum yazabilsin diye app_owner'a açık `owner_seed`
--   append   : standard + yalnız eklenir (touch yok); hakları çağıran daraltır
-- `version` sütunu olan tabloya touch_row() tetikleyicisi eklenir.
CREATE FUNCTION secure_workspace_table(
    p_table text,
    p_mode text DEFAULT 'standard',
    p_privileges text DEFAULT 'SELECT, INSERT, UPDATE, DELETE')
    RETURNS void
    LANGUAGE plpgsql
    AS $$
DECLARE
    ctx constant text := '(SELECT app_workspace_id())';
BEGIN
    IF p_mode NOT IN ('standard', 'shared', 'append') THEN
        RAISE EXCEPTION 'unknown mode %', p_mode;
    END IF;
    EXECUTE format('ALTER TABLE %I ENABLE ROW LEVEL SECURITY', p_table);
    EXECUTE format('ALTER TABLE %I FORCE ROW LEVEL SECURITY', p_table);
    IF p_mode = 'shared' THEN
        EXECUTE format('CREATE POLICY shared_read ON %I FOR SELECT
                        USING (workspace_id IS NULL OR workspace_id = %s)', p_table, ctx);
        EXECUTE format('CREATE POLICY workspace_insert ON %I FOR INSERT
                        WITH CHECK (workspace_id = %s)', p_table, ctx);
        EXECUTE format('CREATE POLICY workspace_update ON %I FOR UPDATE
                        USING (workspace_id = %s) WITH CHECK (workspace_id = %s)', p_table, ctx, ctx);
        EXECUTE format('CREATE POLICY workspace_delete ON %I FOR DELETE
                        USING (workspace_id = %s)', p_table, ctx);
        EXECUTE format('CREATE POLICY owner_seed ON %I TO app_owner USING (true) WITH CHECK (true)',
                       p_table);
    ELSE
        EXECUTE format('CREATE POLICY workspace_isolation ON %I
                        USING (workspace_id = %s) WITH CHECK (workspace_id = %s)', p_table, ctx, ctx);
    END IF;
    IF EXISTS (SELECT 1 FROM pg_attribute
               WHERE attrelid = format('public.%I', p_table)::regclass
                 AND attname = 'version' AND NOT attisdropped) THEN
        EXECUTE format('CREATE TRIGGER touch BEFORE UPDATE ON %I
                        FOR EACH ROW EXECUTE FUNCTION touch_row()', p_table);
    END IF;
    EXECUTE format('GRANT %s ON %I TO app_api, app_worker', p_privileges, p_table);
END
$$;
REVOKE ALL ON FUNCTION secure_workspace_table(text, text, text) FROM PUBLIC;

-- Ortak olabilen kayda giden başvuru (adr/0007 › 2.7) -------------------------------------
-- Bileşik yabancı anahtar ortak (workspace_id NULL) satıra kurulamaz; bu tetikleyici hedefin
-- görünür olmasını ve `workspace_id`'sinin NULL ya da satırınkiyle aynı olmasını ister.
-- Çağıran RLS altındadır: başka çalışma alanının satırı görünmez, "yok" sayılır.
-- Kullanım: CREATE TRIGGER shared_ref_<sütun> ... EXECUTE FUNCTION check_shared_ref('<hedef tablo>', '<sütun>').
-- Ad kuralı katalog testince zorlanır.
CREATE FUNCTION check_shared_ref() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE
    ref_id uuid := (to_jsonb(NEW) ->> TG_ARGV[1])::uuid;
    ok bigint;
BEGIN
    IF ref_id IS NULL THEN
        RETURN NEW;
    END IF;
    EXECUTE format(
        'SELECT count(*) FROM %I WHERE id = $1 AND (workspace_id IS NULL OR workspace_id = $2)',
        TG_ARGV[0])
        INTO ok USING ref_id, NEW.workspace_id;
    IF ok = 0 THEN
        RAISE EXCEPTION '% % is not visible from this workspace', TG_ARGV[0], ref_id
            USING ERRCODE = 'foreign_key_violation';
    END IF;
    RETURN NEW;
END
$$;

-- ingredients --------------------------------------------------------------------------
CREATE TABLE ingredients (
    id              uuid PRIMARY KEY,
    workspace_id    uuid REFERENCES workspaces (id) ON DELETE CASCADE,
    kind            text NOT NULL CHECK (kind IN ('fermentable', 'hop', 'yeast', 'misc')),
    name            text NOT NULL CHECK (char_length(name) BETWEEN 1 AND 200),
    manufacturer    text CHECK (char_length(manufacturer) <= 200),
    product_code    text CHECK (char_length(product_code) <= 100),
    origin          text CHECK (char_length(origin) <= 100),
    notes           text,
    price           numeric(19,4) CHECK (price >= 0),
    price_currency  char(3) CHECK (price_currency ~ '^[A-Z]{3}$'),
    source          text NOT NULL CHECK (source IN ('library', 'user')),
    search          tsvector GENERATED ALWAYS AS (
                        to_tsvector('simple', name || ' ' || coalesce(manufacturer, '') || ' ' || coalesce(origin, ''))
                    ) STORED,
    created_at      timestamptz NOT NULL DEFAULT now(),
    updated_at      timestamptz NOT NULL DEFAULT now(),
    deleted_at      timestamptz,
    version         bigint NOT NULL DEFAULT 1,
    created_by      uuid REFERENCES users (id),
    updated_by      uuid REFERENCES users (id),
    UNIQUE (workspace_id, id),
    -- Kitaplık kaydı ortaktır, kullanıcı kaydı bir çalışma alanına aittir.
    CHECK ((source = 'library') = (workspace_id IS NULL)),
    -- Fiyat kg başınadır; para birimi fiyatla birlikte gelir (adr/0007 › 1.5).
    CHECK ((price IS NULL) = (price_currency IS NULL))
);
CREATE INDEX ingredients_search_idx ON ingredients USING gin (search);
CREATE INDEX ingredients_name_trgm_idx ON ingredients USING gin (name gin_trgm_ops);
CREATE INDEX ingredients_workspace_kind_idx ON ingredients (workspace_id, kind) WHERE deleted_at IS NULL;

-- Türe özgü 1:1 tablolar (`id` ingredients.id ile aynı). Ortak satırda workspace_id NULL
-- olduğundan bileşik anahtar tek başına yetmez (NULL eşleşmeyi atlar): tetikleyici üst kaydın
-- varlığını, türünü ve aynı workspace_id'yi denetler.
CREATE FUNCTION check_ingredient_subtype() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM ingredients
                   WHERE id = NEW.id AND kind = TG_ARGV[0]
                     AND workspace_id IS NOT DISTINCT FROM NEW.workspace_id) THEN
        RAISE EXCEPTION 'no % ingredient % in the same scope', TG_ARGV[0], NEW.id
            USING ERRCODE = 'foreign_key_violation';
    END IF;
    RETURN NEW;
END
$$;

CREATE TABLE ingredient_fermentables (
    id                      uuid PRIMARY KEY,
    workspace_id            uuid REFERENCES workspaces (id) ON DELETE CASCADE,
    grain_type              text NOT NULL CHECK (grain_type IN (
        'grain', 'extract', 'sugar', 'adjunct', 'dry_extract', 'fruit', 'juice', 'honey', 'acid')),
    yield_pct               double precision NOT NULL CHECK (yield_pct BETWEEN 0 AND 100),
    color_srm               double precision NOT NULL DEFAULT 0 CHECK (color_srm >= 0),
    moisture_pct            double precision CHECK (moisture_pct BETWEEN 0 AND 100),
    diastatic_power_lintner double precision CHECK (diastatic_power_lintner >= 0),
    protein_pct             double precision CHECK (protein_pct BETWEEN 0 AND 100),
    max_in_batch_pct        double precision NOT NULL DEFAULT 100 CHECK (max_in_batch_pct BETWEEN 0 AND 100),
    not_fermentable         boolean NOT NULL DEFAULT false,
    -- Şerbetçiotlu ekstrelerin IBU katkısı (motorda ibu_gal_per_lb); birim IBU·L/kg.
    ibu_l_per_kg            double precision NOT NULL DEFAULT 0 CHECK (ibu_l_per_kg >= 0),
    created_at              timestamptz NOT NULL DEFAULT now(),
    updated_at              timestamptz NOT NULL DEFAULT now(),
    deleted_at              timestamptz,
    version                 bigint NOT NULL DEFAULT 1,
    created_by              uuid REFERENCES users (id),
    updated_by              uuid REFERENCES users (id),
    FOREIGN KEY (workspace_id, id) REFERENCES ingredients (workspace_id, id) ON DELETE CASCADE
);
CREATE TRIGGER subtype BEFORE INSERT OR UPDATE OF id, workspace_id ON ingredient_fermentables
    FOR EACH ROW EXECUTE FUNCTION check_ingredient_subtype('fermentable');

CREATE TABLE ingredient_hops (
    id              uuid PRIMARY KEY,
    workspace_id    uuid REFERENCES workspaces (id) ON DELETE CASCADE,
    form            text NOT NULL DEFAULT 'pellet' CHECK (form IN (
        'pellet', 'plug', 'leaf', 'co2_extract', 'isomerized_extract')),
    alpha_pct       double precision NOT NULL CHECK (alpha_pct BETWEEN 0 AND 100),
    beta_pct        double precision CHECK (beta_pct BETWEEN 0 AND 100),
    hsi_pct         double precision CHECK (hsi_pct BETWEEN 0 AND 100),
    created_at      timestamptz NOT NULL DEFAULT now(),
    updated_at      timestamptz NOT NULL DEFAULT now(),
    deleted_at      timestamptz,
    version         bigint NOT NULL DEFAULT 1,
    created_by      uuid REFERENCES users (id),
    updated_by      uuid REFERENCES users (id),
    FOREIGN KEY (workspace_id, id) REFERENCES ingredients (workspace_id, id) ON DELETE CASCADE
);
CREATE TRIGGER subtype BEFORE INSERT OR UPDATE OF id, workspace_id ON ingredient_hops
    FOR EACH ROW EXECUTE FUNCTION check_ingredient_subtype('hop');

CREATE TABLE ingredient_yeasts (
    id                       uuid PRIMARY KEY,
    workspace_id             uuid REFERENCES workspaces (id) ON DELETE CASCADE,
    form                     text NOT NULL CHECK (form IN ('liquid', 'dry', 'slant', 'culture')),
    min_attenuation_pct      double precision NOT NULL CHECK (min_attenuation_pct BETWEEN 0 AND 100),
    max_attenuation_pct      double precision NOT NULL CHECK (max_attenuation_pct BETWEEN 0 AND 100),
    tolerance_abv_pct        double precision CHECK (tolerance_abv_pct >= 0),
    flocculation             text CHECK (flocculation IN ('low', 'medium', 'high', 'very_high')),
    min_temp_c               double precision,
    max_temp_c               double precision,
    total_cells_billion      double precision CHECK (total_cells_billion >= 0),
    cells_per_ml_billion     double precision CHECK (cells_per_ml_billion >= 0),
    dry_pitch_rate_g_per_hl  double precision CHECK (dry_pitch_rate_g_per_hl >= 0),
    created_at               timestamptz NOT NULL DEFAULT now(),
    updated_at               timestamptz NOT NULL DEFAULT now(),
    deleted_at               timestamptz,
    version                  bigint NOT NULL DEFAULT 1,
    created_by               uuid REFERENCES users (id),
    updated_by               uuid REFERENCES users (id),
    FOREIGN KEY (workspace_id, id) REFERENCES ingredients (workspace_id, id) ON DELETE CASCADE,
    CHECK (min_attenuation_pct <= max_attenuation_pct),
    CHECK (min_temp_c IS NULL OR max_temp_c IS NULL OR min_temp_c <= max_temp_c)
);
CREATE TRIGGER subtype BEFORE INSERT OR UPDATE OF id, workspace_id ON ingredient_yeasts
    FOR EACH ROW EXECUTE FUNCTION check_ingredient_subtype('yeast');

CREATE TABLE ingredient_miscs (
    id              uuid PRIMARY KEY,
    workspace_id    uuid REFERENCES workspaces (id) ON DELETE CASCADE,
    misc_kind       text NOT NULL DEFAULT 'other' CHECK (misc_kind IN (
        'spice', 'fining', 'herb', 'flavor', 'water_agent', 'other')),
    default_usage   text CHECK (default_usage IN (
        'boil', 'mash', 'primary', 'secondary', 'bottling', 'sparge', 'whirlpool')),
    -- Sıvı katkı L, ötekiler kg ile ölçülür (stok birimi buna uyar).
    is_liquid       boolean NOT NULL DEFAULT false,
    created_at      timestamptz NOT NULL DEFAULT now(),
    updated_at      timestamptz NOT NULL DEFAULT now(),
    deleted_at      timestamptz,
    version         bigint NOT NULL DEFAULT 1,
    created_by      uuid REFERENCES users (id),
    updated_by      uuid REFERENCES users (id),
    FOREIGN KEY (workspace_id, id) REFERENCES ingredients (workspace_id, id) ON DELETE CASCADE
);
CREATE TRIGGER subtype BEFORE INSERT OR UPDATE OF id, workspace_id ON ingredient_miscs
    FOR EACH ROW EXECUTE FUNCTION check_ingredient_subtype('misc');

-- equipment ----------------------------------------------------------------------------
-- Motorun Equipment alanları ve 05 hacim girdileri (spec 05); hacimler L, kütle kg.
CREATE TABLE equipment (
    id                        uuid PRIMARY KEY,
    workspace_id              uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    name                      text NOT NULL CHECK (char_length(name) BETWEEN 1 AND 200),
    is_default                boolean NOT NULL DEFAULT false,
    batch_volume_l            double precision NOT NULL CHECK (batch_volume_l > 0),
    boil_volume_l             double precision CHECK (boil_volume_l > 0),
    boil_time_min             double precision NOT NULL DEFAULT 60 CHECK (boil_time_min >= 0),
    boil_off_l_per_h          double precision NOT NULL DEFAULT 0 CHECK (boil_off_l_per_h >= 0),
    trub_loss_l               double precision NOT NULL DEFAULT 0 CHECK (trub_loss_l >= 0),
    pre_trub_loss_l           double precision NOT NULL DEFAULT 0 CHECK (pre_trub_loss_l >= 0),
    fermenter_loss_l          double precision NOT NULL DEFAULT 0 CHECK (fermenter_loss_l >= 0),
    top_up_l                  double precision NOT NULL DEFAULT 0 CHECK (top_up_l >= 0),
    cooling_shrinkage_pct     double precision NOT NULL DEFAULT 4 CHECK (cooling_shrinkage_pct BETWEEN 0 AND 100),
    mash_tun_volume_l         double precision CHECK (mash_tun_volume_l > 0),
    mash_tun_mass_kg          double precision NOT NULL DEFAULT 0 CHECK (mash_tun_mass_kg >= 0),
    mash_tun_specific_heat    double precision NOT NULL DEFAULT 0.12 CHECK (mash_tun_specific_heat >= 0),
    mash_dead_space_l         double precision NOT NULL DEFAULT 0 CHECK (mash_dead_space_l >= 0),
    grain_absorption_l_per_kg double precision NOT NULL DEFAULT 1.04 CHECK (grain_absorption_l_per_kg >= 0),
    efficiency_pct            double precision NOT NULL CHECK (efficiency_pct BETWEEN 0 AND 100),
    altitude_m                double precision NOT NULL DEFAULT 0,
    hop_utilization_pct       double precision NOT NULL DEFAULT 100 CHECK (hop_utilization_pct >= 0),
    whirlpool_carry           boolean NOT NULL DEFAULT false,
    whirlpool_time_min        double precision NOT NULL DEFAULT 0 CHECK (whirlpool_time_min >= 0),
    notes                     text,
    created_at                timestamptz NOT NULL DEFAULT now(),
    updated_at                timestamptz NOT NULL DEFAULT now(),
    deleted_at                timestamptz,
    version                   bigint NOT NULL DEFAULT 1,
    created_by                uuid REFERENCES users (id),
    updated_by                uuid REFERENCES users (id),
    UNIQUE (workspace_id, id)
);
CREATE UNIQUE INDEX equipment_default_idx ON equipment (workspace_id)
    WHERE is_default AND deleted_at IS NULL;

-- Profiller (mash, fermantasyon, karbonasyon, su) --------------------------------------
CREATE TABLE mash_profiles (
    id              uuid PRIMARY KEY,
    workspace_id    uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    name            text NOT NULL CHECK (char_length(name) BETWEEN 1 AND 200),
    biab            boolean NOT NULL DEFAULT false,
    grain_temp_c    double precision,
    sparge_temp_c   double precision,
    target_ph       double precision CHECK (target_ph BETWEEN 0 AND 14),
    notes           text,
    created_at      timestamptz NOT NULL DEFAULT now(),
    updated_at      timestamptz NOT NULL DEFAULT now(),
    deleted_at      timestamptz,
    version         bigint NOT NULL DEFAULT 1,
    created_by      uuid REFERENCES users (id),
    updated_by      uuid REFERENCES users (id),
    UNIQUE (workspace_id, id)
);

CREATE TABLE mash_profile_steps (
    id              uuid PRIMARY KEY,
    workspace_id    uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    profile_id      uuid NOT NULL,
    position        integer NOT NULL CHECK (position >= 0),
    name            text NOT NULL DEFAULT '' CHECK (char_length(name) <= 200),
    kind            text NOT NULL CHECK (kind IN ('infusion', 'decoction', 'temperature')),
    temp_c          double precision NOT NULL,
    time_min        double precision NOT NULL CHECK (time_min >= 0),
    ramp_min        double precision NOT NULL DEFAULT 0 CHECK (ramp_min >= 0),
    infusion_l      double precision CHECK (infusion_l >= 0),
    infusion_temp_c double precision,
    created_at      timestamptz NOT NULL DEFAULT now(),
    updated_at      timestamptz NOT NULL DEFAULT now(),
    deleted_at      timestamptz,
    version         bigint NOT NULL DEFAULT 1,
    created_by      uuid REFERENCES users (id),
    updated_by      uuid REFERENCES users (id),
    FOREIGN KEY (workspace_id, profile_id) REFERENCES mash_profiles (workspace_id, id) ON DELETE CASCADE,
    UNIQUE (workspace_id, profile_id, position) DEFERRABLE INITIALLY DEFERRED
);

CREATE TABLE fermentation_profiles (
    id              uuid PRIMARY KEY,
    workspace_id    uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    name            text NOT NULL CHECK (char_length(name) BETWEEN 1 AND 200),
    notes           text,
    created_at      timestamptz NOT NULL DEFAULT now(),
    updated_at      timestamptz NOT NULL DEFAULT now(),
    deleted_at      timestamptz,
    version         bigint NOT NULL DEFAULT 1,
    created_by      uuid REFERENCES users (id),
    updated_by      uuid REFERENCES users (id),
    UNIQUE (workspace_id, id)
);

CREATE TABLE fermentation_profile_steps (
    id              uuid PRIMARY KEY,
    workspace_id    uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    profile_id      uuid NOT NULL,
    position        integer NOT NULL CHECK (position >= 0),
    name            text NOT NULL DEFAULT '' CHECK (char_length(name) <= 200),
    temp_c          double precision NOT NULL,
    duration_min    double precision NOT NULL CHECK (duration_min >= 0),
    ramp_min        double precision NOT NULL DEFAULT 0 CHECK (ramp_min >= 0),
    created_at      timestamptz NOT NULL DEFAULT now(),
    updated_at      timestamptz NOT NULL DEFAULT now(),
    deleted_at      timestamptz,
    version         bigint NOT NULL DEFAULT 1,
    created_by      uuid REFERENCES users (id),
    updated_by      uuid REFERENCES users (id),
    FOREIGN KEY (workspace_id, profile_id) REFERENCES fermentation_profiles (workspace_id, id) ON DELETE CASCADE,
    UNIQUE (workspace_id, profile_id, position) DEFERRABLE INITIALLY DEFERRED
);

CREATE TABLE carbonation_profiles (
    id               uuid PRIMARY KEY,
    workspace_id     uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    name             text NOT NULL CHECK (char_length(name) BETWEEN 1 AND 200),
    method           text NOT NULL CHECK (method IN ('forced', 'priming', 'natural', 'spunding')),
    target_co2_vols  double precision NOT NULL CHECK (target_co2_vols >= 0),
    temp_c           double precision,
    priming_agent    text CHECK (priming_agent IN ('corn_sugar', 'table_sugar', 'dme', 'honey')),
    notes            text,
    created_at       timestamptz NOT NULL DEFAULT now(),
    updated_at       timestamptz NOT NULL DEFAULT now(),
    deleted_at       timestamptz,
    version          bigint NOT NULL DEFAULT 1,
    created_by       uuid REFERENCES users (id),
    updated_by       uuid REFERENCES users (id),
    UNIQUE (workspace_id, id)
);

-- Kaynak suyu ve hedef profiller; iyonlar mg/L. `source_document_id` taranan su analizi
-- belgesine bağlanır; yabancı anahtar scanned_documents ile birlikte eklenir (0011).
CREATE TABLE water_profiles (
    id                  uuid PRIMARY KEY,
    workspace_id        uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    name                text NOT NULL CHECK (char_length(name) BETWEEN 1 AND 200),
    kind                text NOT NULL CHECK (kind IN ('source', 'target')),
    calcium_mg_l        double precision NOT NULL DEFAULT 0 CHECK (calcium_mg_l >= 0),
    magnesium_mg_l      double precision NOT NULL DEFAULT 0 CHECK (magnesium_mg_l >= 0),
    sodium_mg_l         double precision NOT NULL DEFAULT 0 CHECK (sodium_mg_l >= 0),
    chloride_mg_l       double precision NOT NULL DEFAULT 0 CHECK (chloride_mg_l >= 0),
    sulfate_mg_l        double precision NOT NULL DEFAULT 0 CHECK (sulfate_mg_l >= 0),
    bicarbonate_mg_l    double precision NOT NULL DEFAULT 0 CHECK (bicarbonate_mg_l >= 0),
    alkalinity_mg_l     double precision CHECK (alkalinity_mg_l >= 0),
    ph                  double precision CHECK (ph BETWEEN 0 AND 14),
    source_document_id  uuid,
    notes               text,
    created_at          timestamptz NOT NULL DEFAULT now(),
    updated_at          timestamptz NOT NULL DEFAULT now(),
    deleted_at          timestamptz,
    version             bigint NOT NULL DEFAULT 1,
    created_by          uuid REFERENCES users (id),
    updated_by          uuid REFERENCES users (id),
    UNIQUE (workspace_id, id)
);

-- Stil kılavuzları, stiller, eşleşmeler (ortak + çalışma alanına özel) ---------------------
CREATE TABLE style_guides (
    id            uuid PRIMARY KEY,
    workspace_id  uuid REFERENCES workspaces (id) ON DELETE CASCADE,
    name          text NOT NULL CHECK (char_length(name) BETWEEN 1 AND 200),
    year          integer CHECK (year BETWEEN 1900 AND 2200),
    license_note  text,
    created_at    timestamptz NOT NULL DEFAULT now(),
    updated_at    timestamptz NOT NULL DEFAULT now(),
    deleted_at    timestamptz,
    version       bigint NOT NULL DEFAULT 1,
    created_by    uuid REFERENCES users (id),
    updated_by    uuid REFERENCES users (id),
    UNIQUE (workspace_id, id)
);

CREATE TABLE styles (
    id            uuid PRIMARY KEY,
    workspace_id  uuid REFERENCES workspaces (id) ON DELETE CASCADE,
    guide_id      uuid NOT NULL REFERENCES style_guides (id),
    category      text CHECK (char_length(category) <= 200),
    code          text CHECK (char_length(code) <= 20),
    name          text NOT NULL CHECK (char_length(name) BETWEEN 1 AND 200),
    og_min_sg     double precision, og_max_sg     double precision,
    fg_min_sg     double precision, fg_max_sg     double precision,
    ibu_min       double precision, ibu_max       double precision,
    srm_min       double precision, srm_max       double precision,
    abv_min_pct   double precision, abv_max_pct   double precision,
    co2_min_vols  double precision, co2_max_vols  double precision,
    notes         text,
    created_at    timestamptz NOT NULL DEFAULT now(),
    updated_at    timestamptz NOT NULL DEFAULT now(),
    deleted_at    timestamptz,
    version       bigint NOT NULL DEFAULT 1,
    created_by    uuid REFERENCES users (id),
    updated_by    uuid REFERENCES users (id),
    UNIQUE (workspace_id, id),
    CHECK (og_min_sg <= og_max_sg AND fg_min_sg <= fg_max_sg AND ibu_min <= ibu_max
       AND srm_min <= srm_max AND abv_min_pct <= abv_max_pct AND co2_min_vols <= co2_max_vols)
);
CREATE INDEX styles_guide_idx ON styles (guide_id);
CREATE INDEX styles_name_trgm_idx ON styles USING gin (name gin_trgm_ops);

CREATE TABLE style_mappings (
    id             uuid PRIMARY KEY,
    workspace_id   uuid REFERENCES workspaces (id) ON DELETE CASCADE,
    from_style_id  uuid NOT NULL REFERENCES styles (id),
    to_style_id    uuid NOT NULL REFERENCES styles (id),
    relation       text NOT NULL DEFAULT 'closest' CHECK (relation IN ('equivalent', 'closest')),
    notes          text,
    created_at     timestamptz NOT NULL DEFAULT now(),
    updated_at     timestamptz NOT NULL DEFAULT now(),
    deleted_at     timestamptz,
    version        bigint NOT NULL DEFAULT 1,
    created_by     uuid REFERENCES users (id),
    updated_by     uuid REFERENCES users (id),
    UNIQUE (workspace_id, id),
    CHECK (from_style_id <> to_style_id)
);
CREATE UNIQUE INDEX style_mappings_pair_idx ON style_mappings (workspace_id, from_style_id, to_style_id)
    NULLS NOT DISTINCT WHERE deleted_at IS NULL;

-- package_types --------------------------------------------------------------------------
CREATE TABLE package_types (
    id            uuid PRIMARY KEY,
    workspace_id  uuid REFERENCES workspaces (id) ON DELETE CASCADE,
    kind          text NOT NULL CHECK (kind IN ('bottle', 'can', 'keg', 'cask')),
    name          text NOT NULL CHECK (char_length(name) BETWEEN 1 AND 200),
    volume_l      double precision NOT NULL CHECK (volume_l > 0),
    single_use    boolean NOT NULL DEFAULT false,
    material      text CHECK (char_length(material) <= 100),
    tare_kg       double precision CHECK (tare_kg >= 0),
    created_at    timestamptz NOT NULL DEFAULT now(),
    updated_at    timestamptz NOT NULL DEFAULT now(),
    deleted_at    timestamptz,
    version       bigint NOT NULL DEFAULT 1,
    created_by    uuid REFERENCES users (id),
    updated_by    uuid REFERENCES users (id),
    UNIQUE (workspace_id, id)
);

-- Ortak hedefe başvuru tetikleyicileri
CREATE TRIGGER shared_ref_guide_id BEFORE INSERT OR UPDATE OF guide_id, workspace_id ON styles
    FOR EACH ROW EXECUTE FUNCTION check_shared_ref('style_guides', 'guide_id');
CREATE TRIGGER shared_ref_from_style_id BEFORE INSERT OR UPDATE OF from_style_id, workspace_id ON style_mappings
    FOR EACH ROW EXECUTE FUNCTION check_shared_ref('styles', 'from_style_id');
CREATE TRIGGER shared_ref_to_style_id BEFORE INSERT OR UPDATE OF to_style_id, workspace_id ON style_mappings
    FOR EACH ROW EXECUTE FUNCTION check_shared_ref('styles', 'to_style_id');

-- RLS, sürüm tetikleyicisi ve haklar ---------------------------------------------------------
SELECT secure_workspace_table(t, 'shared') FROM unnest(ARRAY[
    'ingredients', 'ingredient_fermentables', 'ingredient_hops', 'ingredient_yeasts',
    'ingredient_miscs', 'style_guides', 'styles', 'style_mappings', 'package_types']) t;
SELECT secure_workspace_table(t) FROM unnest(ARRAY[
    'equipment', 'mash_profiles', 'mash_profile_steps', 'fermentation_profiles',
    'fermentation_profile_steps', 'carbonation_profiles', 'water_profiles']) t;

RESET ROLE;
