-- Tarif (adr/0007 › 3 "Tarif"; T2.6).
-- Tarifin kimliği `recipes`, içeriği `recipe_versions` ve satır tablolarındadır. Yayınlanmış
-- (taslak olmayan) sürüm ve satırları değişmez; tetikleyiciler dosyanın sonundadır (madde 3).
SET LOCAL ROLE app_owner;

CREATE TABLE folders (
    id            uuid PRIMARY KEY,
    workspace_id  uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    parent_id     uuid,
    name          text NOT NULL CHECK (char_length(name) BETWEEN 1 AND 200),
    created_at    timestamptz NOT NULL DEFAULT now(),
    updated_at    timestamptz NOT NULL DEFAULT now(),
    deleted_at    timestamptz,
    version       bigint NOT NULL DEFAULT 1,
    created_by    uuid REFERENCES users (id),
    updated_by    uuid REFERENCES users (id),
    UNIQUE (workspace_id, id),
    FOREIGN KEY (workspace_id, parent_id) REFERENCES folders (workspace_id, id),
    CHECK (parent_id IS DISTINCT FROM id)
);

CREATE TABLE recipes (
    id                   uuid PRIMARY KEY,
    workspace_id         uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    name                 text NOT NULL CHECK (char_length(name) BETWEEN 1 AND 200),
    folder_id            uuid,
    -- Kişisel kilit: sahibi tarifi değişikliğe kapatır (Ev; uygulama denetler).
    is_locked            boolean NOT NULL DEFAULT false,
    notes                text,
    -- Son onaylı ve taslak sürüm göstergeleri; yabancı anahtarlar recipe_versions'tan sonra.
    approved_version_id  uuid,
    draft_version_id     uuid,
    search               tsvector GENERATED ALWAYS AS (
                             to_tsvector('simple', name || ' ' || coalesce(notes, ''))
                         ) STORED,
    created_at           timestamptz NOT NULL DEFAULT now(),
    updated_at           timestamptz NOT NULL DEFAULT now(),
    deleted_at           timestamptz,
    version              bigint NOT NULL DEFAULT 1,
    created_by           uuid REFERENCES users (id),
    updated_by           uuid REFERENCES users (id),
    UNIQUE (workspace_id, id),
    FOREIGN KEY (workspace_id, folder_id) REFERENCES folders (workspace_id, id)
);
CREATE INDEX recipes_search_idx ON recipes USING gin (search);
CREATE INDEX recipes_name_trgm_idx ON recipes USING gin (name gin_trgm_ops);
CREATE INDEX recipes_folder_idx ON recipes (workspace_id, folder_id) WHERE deleted_at IS NULL;

-- recipe_versions ----------------------------------------------------------------------
-- Başlık alanları sütun, anlık görüntüler JSONB (adr/0007 › 1.7a; schema_version zorunlu).
CREATE TABLE recipe_versions (
    id                    uuid PRIMARY KEY,
    workspace_id          uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    recipe_id             uuid NOT NULL,
    number                integer NOT NULL CHECK (number >= 1),
    status                text NOT NULL DEFAULT 'draft'
                          CHECK (status IN ('draft', 'in_review', 'approved', 'archived')),
    kind                  text NOT NULL DEFAULT 'all_grain' CHECK (kind IN (
        'extract', 'partial_mash', 'all_grain', 'cider', 'mead', 'wine', 'seltzer')),
    style_id              uuid REFERENCES styles (id),
    equipment_id          uuid,
    batch_volume_l        double precision NOT NULL CHECK (batch_volume_l > 0),
    boil_volume_l         double precision CHECK (boil_volume_l > 0),
    boil_time_min         double precision NOT NULL DEFAULT 60 CHECK (boil_time_min >= 0),
    efficiency_pct        double precision NOT NULL CHECK (efficiency_pct BETWEEN 0 AND 100),
    target_og_sg          double precision, target_fg_sg   double precision,
    target_ibu            double precision, target_srm     double precision,
    target_abv_pct        double precision, target_co2_vols double precision,
    engine_version        text,
    calculated_at         timestamptz,
    change_note           text,
    submitted_by          uuid REFERENCES users (id),
    submitted_at          timestamptz,
    approved_by           uuid REFERENCES users (id),
    approved_at           timestamptz,
    archived_at           timestamptz,
    equipment_snapshot    jsonb NOT NULL DEFAULT '{"schema_version": 1}'
                          CHECK (jsonb_typeof(equipment_snapshot) = 'object' AND equipment_snapshot ? 'schema_version'),
    mash_snapshot         jsonb CHECK (jsonb_typeof(mash_snapshot) = 'object' AND mash_snapshot ? 'schema_version'),
    fermentation_snapshot jsonb CHECK (jsonb_typeof(fermentation_snapshot) = 'object' AND fermentation_snapshot ? 'schema_version'),
    water_snapshot        jsonb CHECK (jsonb_typeof(water_snapshot) = 'object' AND water_snapshot ? 'schema_version'),
    options               jsonb NOT NULL DEFAULT '{"schema_version": 1}'
                          CHECK (jsonb_typeof(options) = 'object' AND options ? 'schema_version'),
    created_at            timestamptz NOT NULL DEFAULT now(),
    updated_at            timestamptz NOT NULL DEFAULT now(),
    deleted_at            timestamptz,
    version               bigint NOT NULL DEFAULT 1,
    created_by            uuid REFERENCES users (id),
    updated_by            uuid REFERENCES users (id),
    UNIQUE (workspace_id, id),
    UNIQUE (workspace_id, recipe_id, number),
    FOREIGN KEY (workspace_id, recipe_id) REFERENCES recipes (workspace_id, id) ON DELETE CASCADE,
    FOREIGN KEY (workspace_id, equipment_id) REFERENCES equipment (workspace_id, id),
    CHECK ((approved_by IS NULL) = (approved_at IS NULL)),
    CHECK (status NOT IN ('approved', 'archived') OR approved_at IS NOT NULL)
);
-- Tarif başına en çok bir taslak.
CREATE UNIQUE INDEX recipe_versions_one_draft_idx ON recipe_versions (workspace_id, recipe_id)
    WHERE status = 'draft' AND deleted_at IS NULL;
CREATE INDEX recipe_versions_style_idx ON recipe_versions (style_id) WHERE style_id IS NOT NULL;

ALTER TABLE recipes
    ADD FOREIGN KEY (workspace_id, approved_version_id) REFERENCES recipe_versions (workspace_id, id),
    ADD FOREIGN KEY (workspace_id, draft_version_id) REFERENCES recipe_versions (workspace_id, id);

CREATE TRIGGER shared_ref_style_id BEFORE INSERT OR UPDATE OF style_id, workspace_id ON recipe_versions
    FOR EACH ROW EXECUTE FUNCTION check_shared_ref('styles', 'style_id');

-- IngredientUse: sürüm × malzeme, türe göre dört tablo ----------------------------------
-- Hesapta kullanılan özellikler satıra kopyalanır; ingredient_id yalnız kaynağı gösterir,
-- kütüphane değişse de tarif değişmez. Sıra `position` (sürüm içinde tekil, ertelenebilir).
CREATE TABLE recipe_fermentables (
    id                 uuid PRIMARY KEY,
    workspace_id       uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    recipe_version_id  uuid NOT NULL,
    position           integer NOT NULL CHECK (position >= 0),
    ingredient_id      uuid REFERENCES ingredients (id),
    name               text NOT NULL CHECK (char_length(name) BETWEEN 1 AND 200),
    amount_kg          double precision NOT NULL CHECK (amount_kg >= 0),
    usage              text NOT NULL DEFAULT 'mash' CHECK (usage IN (
        'mash', 'steep', 'boil', 'whirlpool', 'primary', 'secondary', 'tertiary', 'bottling', 'sparge')),
    grain_type         text NOT NULL CHECK (grain_type IN (
        'grain', 'extract', 'sugar', 'adjunct', 'dry_extract', 'fruit', 'juice', 'honey', 'acid')),
    yield_pct          double precision NOT NULL CHECK (yield_pct BETWEEN 0 AND 100),
    color_srm          double precision NOT NULL DEFAULT 0 CHECK (color_srm >= 0),
    not_fermentable    boolean NOT NULL DEFAULT false,
    late_extract_min   double precision NOT NULL DEFAULT 0 CHECK (late_extract_min >= 0),
    max_in_batch_pct   double precision NOT NULL DEFAULT 100 CHECK (max_in_batch_pct BETWEEN 0 AND 100),
    ibu_l_per_kg       double precision NOT NULL DEFAULT 0 CHECK (ibu_l_per_kg >= 0),
    created_at         timestamptz NOT NULL DEFAULT now(),
    updated_at         timestamptz NOT NULL DEFAULT now(),
    deleted_at         timestamptz,
    version            bigint NOT NULL DEFAULT 1,
    created_by         uuid REFERENCES users (id),
    updated_by         uuid REFERENCES users (id),
    UNIQUE (workspace_id, id),
    UNIQUE (workspace_id, recipe_version_id, position) DEFERRABLE INITIALLY DEFERRED,
    FOREIGN KEY (workspace_id, recipe_version_id) REFERENCES recipe_versions (workspace_id, id) ON DELETE CASCADE
);

CREATE TABLE recipe_hops (
    id                 uuid PRIMARY KEY,
    workspace_id       uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    recipe_version_id  uuid NOT NULL,
    position           integer NOT NULL CHECK (position >= 0),
    ingredient_id      uuid REFERENCES ingredients (id),
    name               text NOT NULL CHECK (char_length(name) BETWEEN 1 AND 200),
    amount_kg          double precision NOT NULL CHECK (amount_kg >= 0),
    alpha_pct          double precision NOT NULL CHECK (alpha_pct BETWEEN 0 AND 100),
    form               text NOT NULL DEFAULT 'pellet' CHECK (form IN (
        'pellet', 'plug', 'leaf', 'co2_extract', 'isomerized_extract')),
    usage              text NOT NULL DEFAULT 'boil' CHECK (usage IN (
        'boil', 'dry_hop', 'mash', 'first_wort', 'whirlpool')),
    time_min           double precision NOT NULL DEFAULT 0 CHECK (time_min >= 0),
    whirlpool_temp_c   double precision,
    created_at         timestamptz NOT NULL DEFAULT now(),
    updated_at         timestamptz NOT NULL DEFAULT now(),
    deleted_at         timestamptz,
    version            bigint NOT NULL DEFAULT 1,
    created_by         uuid REFERENCES users (id),
    updated_by         uuid REFERENCES users (id),
    UNIQUE (workspace_id, id),
    UNIQUE (workspace_id, recipe_version_id, position) DEFERRABLE INITIALLY DEFERRED,
    FOREIGN KEY (workspace_id, recipe_version_id) REFERENCES recipe_versions (workspace_id, id) ON DELETE CASCADE
);

CREATE TABLE recipe_yeasts (
    id                       uuid PRIMARY KEY,
    workspace_id             uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    recipe_version_id        uuid NOT NULL,
    position                 integer NOT NULL CHECK (position >= 0),
    ingredient_id            uuid REFERENCES ingredients (id),
    name                     text NOT NULL CHECK (char_length(name) BETWEEN 1 AND 200),
    form                     text NOT NULL CHECK (form IN ('liquid', 'dry', 'slant', 'culture')),
    min_attenuation_pct      double precision NOT NULL CHECK (min_attenuation_pct BETWEEN 0 AND 100),
    max_attenuation_pct      double precision NOT NULL CHECK (max_attenuation_pct BETWEEN 0 AND 100),
    tolerance_abv_pct        double precision CHECK (tolerance_abv_pct >= 0),
    add_to_secondary         boolean NOT NULL DEFAULT false,
    total_cells_billion      double precision CHECK (total_cells_billion >= 0),
    viable_cells_billion     double precision CHECK (viable_cells_billion >= 0),
    packs                    double precision NOT NULL DEFAULT 1 CHECK (packs >= 0),
    viability_pct            double precision CHECK (viability_pct BETWEEN 0 AND 100),
    use_packages             boolean NOT NULL DEFAULT true,
    amount_kg                double precision NOT NULL DEFAULT 0 CHECK (amount_kg >= 0),
    cells_per_ml_billion     double precision CHECK (cells_per_ml_billion >= 0),
    dry_pitch_rate_g_per_hl  double precision CHECK (dry_pitch_rate_g_per_hl >= 0),
    created_at               timestamptz NOT NULL DEFAULT now(),
    updated_at               timestamptz NOT NULL DEFAULT now(),
    deleted_at               timestamptz,
    version                  bigint NOT NULL DEFAULT 1,
    created_by               uuid REFERENCES users (id),
    updated_by               uuid REFERENCES users (id),
    UNIQUE (workspace_id, id),
    UNIQUE (workspace_id, recipe_version_id, position) DEFERRABLE INITIALLY DEFERRED,
    FOREIGN KEY (workspace_id, recipe_version_id) REFERENCES recipe_versions (workspace_id, id) ON DELETE CASCADE,
    CHECK (min_attenuation_pct <= max_attenuation_pct)
);

CREATE TABLE recipe_miscs (
    id                 uuid PRIMARY KEY,
    workspace_id       uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    recipe_version_id  uuid NOT NULL,
    position           integer NOT NULL CHECK (position >= 0),
    ingredient_id      uuid REFERENCES ingredients (id),
    name               text NOT NULL CHECK (char_length(name) BETWEEN 1 AND 200),
    amount             double precision NOT NULL CHECK (amount >= 0),
    -- Sıvı katkı L, ötekiler kg.
    is_liquid          boolean NOT NULL DEFAULT false,
    usage              text NOT NULL DEFAULT 'boil' CHECK (usage IN (
        'boil', 'mash', 'primary', 'secondary', 'bottling', 'sparge', 'whirlpool')),
    time_min           double precision NOT NULL DEFAULT 0 CHECK (time_min >= 0),
    created_at         timestamptz NOT NULL DEFAULT now(),
    updated_at         timestamptz NOT NULL DEFAULT now(),
    deleted_at         timestamptz,
    version            bigint NOT NULL DEFAULT 1,
    created_by         uuid REFERENCES users (id),
    updated_by         uuid REFERENCES users (id),
    UNIQUE (workspace_id, id),
    UNIQUE (workspace_id, recipe_version_id, position) DEFERRABLE INITIALLY DEFERRED,
    FOREIGN KEY (workspace_id, recipe_version_id) REFERENCES recipe_versions (workspace_id, id) ON DELETE CASCADE
);

-- Tuz ve asit eklemeleri (su ve pH ekranı). Asitte miktar, `concentration_pct` kadar
-- derişik çözeltinin kütlesidir.
CREATE TABLE recipe_water_additions (
    id                 uuid PRIMARY KEY,
    workspace_id       uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    recipe_version_id  uuid NOT NULL,
    position           integer NOT NULL CHECK (position >= 0),
    agent              text NOT NULL CHECK (agent IN (
        'gypsum', 'calcium_chloride', 'epsom_salt', 'baking_soda', 'chalk', 'table_salt',
        'slaked_lime', 'magnesium_chloride', 'lactic_acid', 'phosphoric_acid', 'acid_malt')),
    stage              text NOT NULL DEFAULT 'mash' CHECK (stage IN ('mash', 'sparge', 'boil')),
    amount_kg          double precision NOT NULL CHECK (amount_kg >= 0),
    concentration_pct  double precision CHECK (concentration_pct BETWEEN 0 AND 100),
    created_at         timestamptz NOT NULL DEFAULT now(),
    updated_at         timestamptz NOT NULL DEFAULT now(),
    deleted_at         timestamptz,
    version            bigint NOT NULL DEFAULT 1,
    created_by         uuid REFERENCES users (id),
    updated_by         uuid REFERENCES users (id),
    UNIQUE (workspace_id, id),
    UNIQUE (workspace_id, recipe_version_id, position) DEFERRABLE INITIALLY DEFERRED,
    FOREIGN KEY (workspace_id, recipe_version_id) REFERENCES recipe_versions (workspace_id, id) ON DELETE CASCADE
);

CREATE TRIGGER shared_ref_ingredient_id BEFORE INSERT OR UPDATE OF ingredient_id, workspace_id ON recipe_fermentables
    FOR EACH ROW EXECUTE FUNCTION check_shared_ref('ingredients', 'ingredient_id');
CREATE TRIGGER shared_ref_ingredient_id BEFORE INSERT OR UPDATE OF ingredient_id, workspace_id ON recipe_hops
    FOR EACH ROW EXECUTE FUNCTION check_shared_ref('ingredients', 'ingredient_id');
CREATE TRIGGER shared_ref_ingredient_id BEFORE INSERT OR UPDATE OF ingredient_id, workspace_id ON recipe_yeasts
    FOR EACH ROW EXECUTE FUNCTION check_shared_ref('ingredients', 'ingredient_id');
CREATE TRIGGER shared_ref_ingredient_id BEFORE INSERT OR UPDATE OF ingredient_id, workspace_id ON recipe_miscs
    FOR EACH ROW EXECUTE FUNCTION check_shared_ref('ingredients', 'ingredient_id');

-- Etiketler -----------------------------------------------------------------------------
CREATE TABLE tags (
    id            uuid PRIMARY KEY,
    workspace_id  uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    name          text NOT NULL CHECK (char_length(name) BETWEEN 1 AND 100),
    color         text CHECK (color ~ '^#[0-9a-fA-F]{6}$'),
    created_at    timestamptz NOT NULL DEFAULT now(),
    updated_at    timestamptz NOT NULL DEFAULT now(),
    deleted_at    timestamptz,
    version       bigint NOT NULL DEFAULT 1,
    created_by    uuid REFERENCES users (id),
    updated_by    uuid REFERENCES users (id),
    UNIQUE (workspace_id, id)
);
CREATE UNIQUE INDEX tags_name_idx ON tags (workspace_id, lower(name)) WHERE deleted_at IS NULL;

-- Bağlantı tabloları: eklenir ve silinir, güncellenmez.
CREATE TABLE recipe_tags (
    workspace_id  uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    recipe_id     uuid NOT NULL,
    tag_id        uuid NOT NULL,
    created_at    timestamptz NOT NULL DEFAULT now(),
    created_by    uuid REFERENCES users (id),
    PRIMARY KEY (workspace_id, recipe_id, tag_id),
    FOREIGN KEY (workspace_id, recipe_id) REFERENCES recipes (workspace_id, id) ON DELETE CASCADE,
    FOREIGN KEY (workspace_id, tag_id) REFERENCES tags (workspace_id, id) ON DELETE CASCADE
);

-- Değişmezlik (adr/0007 › 3 "Değişmezlik") ----------------------------------------------
-- Taslak olmayan sürümde içerik değiştiren UPDATE ve DELETE reddedilir. İzinli geçişler:
--   draft     -> her değişiklik (yayınlama dahil)
--   in_review -> status (draft'a dönüş, approved) ve onay alanları
--   approved  -> yalnız archived'a geçiş (archived_at)
--   archived  -> değişmez
-- Doğrudan DELETE reddedilir; tarif ya da çalışma alanı silinirken yabancı anahtar
-- zincirinden gelen silme (pg_trigger_depth() > 1) geçer (adr/0007 › 6.3, audit_log gibi).
CREATE FUNCTION recipe_versions_guard() RETURNS trigger
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
    IF (to_jsonb(NEW) - free_keys) IS DISTINCT FROM (to_jsonb(OLD) - free_keys) THEN
        RAISE EXCEPTION 'published recipe version is immutable (UPDATE rejected)'
            USING ERRCODE = 'integrity_constraint_violation';
    END IF;
    RETURN NEW;
END
$$;
CREATE TRIGGER a_immutable BEFORE UPDATE OR DELETE ON recipe_versions
    FOR EACH ROW EXECUTE FUNCTION recipe_versions_guard();

-- Satır tabloları: üst sürüm taslak değilse ekleme, güncelleme ve silme reddedilir.
CREATE FUNCTION recipe_version_lines_guard() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE
    v uuid;
    s text;
BEGIN
    IF TG_OP = 'DELETE' AND pg_trigger_depth() > 1 THEN
        RETURN OLD;
    END IF;
    FOR v IN
        SELECT DISTINCT x FROM unnest(ARRAY[
            CASE WHEN TG_OP <> 'INSERT' THEN OLD.recipe_version_id END,
            CASE WHEN TG_OP <> 'DELETE' THEN NEW.recipe_version_id END]) x
        WHERE x IS NOT NULL
    LOOP
        SELECT status INTO s FROM recipe_versions
        WHERE id = v AND workspace_id = coalesce(NEW.workspace_id, OLD.workspace_id);
        IF s IS NOT NULL AND s <> 'draft' THEN
            RAISE EXCEPTION 'lines of a published recipe version are immutable (% rejected)', TG_OP
                USING ERRCODE = 'integrity_constraint_violation';
        END IF;
    END LOOP;
    RETURN CASE TG_OP WHEN 'DELETE' THEN OLD ELSE NEW END;
END
$$;
CREATE TRIGGER a_immutable BEFORE INSERT OR UPDATE OR DELETE ON recipe_fermentables
    FOR EACH ROW EXECUTE FUNCTION recipe_version_lines_guard();
CREATE TRIGGER a_immutable BEFORE INSERT OR UPDATE OR DELETE ON recipe_hops
    FOR EACH ROW EXECUTE FUNCTION recipe_version_lines_guard();
CREATE TRIGGER a_immutable BEFORE INSERT OR UPDATE OR DELETE ON recipe_yeasts
    FOR EACH ROW EXECUTE FUNCTION recipe_version_lines_guard();
CREATE TRIGGER a_immutable BEFORE INSERT OR UPDATE OR DELETE ON recipe_miscs
    FOR EACH ROW EXECUTE FUNCTION recipe_version_lines_guard();
CREATE TRIGGER a_immutable BEFORE INSERT OR UPDATE OR DELETE ON recipe_water_additions
    FOR EACH ROW EXECUTE FUNCTION recipe_version_lines_guard();

-- RLS, sürüm tetikleyicisi ve haklar ---------------------------------------------------------
SELECT secure_workspace_table(t) FROM unnest(ARRAY[
    'folders', 'recipes', 'recipe_versions', 'recipe_fermentables', 'recipe_hops',
    'recipe_yeasts', 'recipe_miscs', 'recipe_water_additions', 'tags']) t;
SELECT secure_workspace_table('recipe_tags', 'standard', 'SELECT, INSERT, DELETE');

RESET ROLE;
