-- T2.9 (F2.1 küçük bulgular 14, 16, 17, 19, 21, 22, 23, 24, 26, 32, 36, 39): ucuz ve güvenli
-- düzeltmeler. Kalan küçük bulgular adr/0007 › T2.9 düzeltmeleri'nde gerekçeyle ertelendi.
SET LOCAL ROLE app_owner;

-- 14: ortak kitaplık tohum politikası yalnız ortak (workspace_id NULL) satırlara açılır -----
DO $$
DECLARE
    t text;
BEGIN
    FOREACH t IN ARRAY ARRAY['ingredients', 'ingredient_fermentables', 'ingredient_hops',
                             'ingredient_yeasts', 'ingredient_miscs', 'style_guides', 'styles',
                             'style_mappings', 'package_types'] LOOP
        EXECUTE format('DROP POLICY owner_seed ON %I', t);
        EXECUTE format('CREATE POLICY owner_seed ON %I TO app_owner
                        USING (workspace_id IS NULL) WITH CHECK (workspace_id IS NULL)', t);
    END LOOP;
END
$$;

-- 16: destek iç notu müşteri bağlamında görünmez (yalnız API süzgecine bırakılmaz) ---------
CREATE POLICY customer_visibility ON support_messages AS RESTRICTIVE FOR SELECT
    USING (visibility = 'customer' OR (SELECT app_actor_kind()) IN ('support', 'system'));

-- 17: hesap verisi dışa aktarımı yalnız isteyen kullanıcıya görünür ----------------------------
CREATE POLICY own_account_exports ON export_jobs AS RESTRICTIVE
    USING (scope <> 'account_data' OR created_by = (SELECT app_user_id()))
    WITH CHECK (scope <> 'account_data' OR created_by = (SELECT app_user_id()));

-- 19: API outbox'a yalnız bağlamındaki çalışma alanı için olay ekler -----------------------
CREATE FUNCTION outbox_check_workspace() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF current_user = 'app_api' AND NEW.workspace_id IS NOT NULL
       AND NEW.workspace_id IS DISTINCT FROM app_workspace_id() THEN
        RAISE EXCEPTION 'outbox event for workspace % outside the request context', NEW.workspace_id
            USING ERRCODE = 'insufficient_privilege';
    END IF;
    RETURN NEW;
END
$$;
CREATE TRIGGER workspace_context BEFORE INSERT ON outbox
    FOR EACH ROW EXECUTE FUNCTION outbox_check_workspace();

-- 21: tarifin onaylı ve taslak sürümü o tarifin sürümüdür ------------------------------------
ALTER TABLE recipe_versions ADD CONSTRAINT recipe_versions_recipe_key UNIQUE (workspace_id, recipe_id, id);
DO $$
DECLARE
    con name;
BEGIN
    FOR con IN
        SELECT conname FROM pg_constraint
        WHERE conrelid = 'recipes'::regclass AND contype = 'f'
          AND confrelid = 'recipe_versions'::regclass AND array_length(conkey, 1) = 2
    LOOP
        EXECUTE format('ALTER TABLE recipes DROP CONSTRAINT %I', con);
    END LOOP;
END
$$;
ALTER TABLE recipes
    ADD CONSTRAINT recipes_approved_version_fkey FOREIGN KEY (workspace_id, id, approved_version_id)
        REFERENCES recipe_versions (workspace_id, recipe_id, id),
    ADD CONSTRAINT recipes_draft_version_fkey FOREIGN KEY (workspace_id, id, draft_version_id)
        REFERENCES recipe_versions (workspace_id, recipe_id, id);

-- 22: aynı partiye ait olması gereken başvurular üç sütunlu anahtarla -----------------------
ALTER TABLE brew_day_packages ADD CONSTRAINT brew_day_packages_batch_key UNIQUE (workspace_id, batch_id, id);
ALTER TABLE batch_steps ADD CONSTRAINT batch_steps_batch_key UNIQUE (workspace_id, batch_id, id);
ALTER TABLE batch_ingredient_actuals
    ADD CONSTRAINT batch_ingredient_actuals_batch_key UNIQUE (workspace_id, batch_id, id);
DO $$
DECLARE
    con record;
BEGIN
    FOR con IN
        SELECT c.conrelid::regclass AS tbl, c.conname
        FROM pg_constraint c
        WHERE c.contype = 'f' AND array_length(c.conkey, 1) = 2
          AND ((c.conrelid = 'measurements'::regclass AND c.confrelid = 'brew_day_packages'::regclass)
            OR (c.conrelid = 'deviations'::regclass
                AND c.confrelid IN ('batch_steps'::regclass, 'batch_ingredient_actuals'::regclass)))
    LOOP
        EXECUTE format('ALTER TABLE %s DROP CONSTRAINT %I', con.tbl, con.conname);
    END LOOP;
END
$$;
ALTER TABLE measurements
    ADD CONSTRAINT measurements_package_fkey FOREIGN KEY (workspace_id, batch_id, package_id)
        REFERENCES brew_day_packages (workspace_id, batch_id, id) ON DELETE SET NULL (package_id),
    ADD CONSTRAINT measurements_package_batch_check CHECK (package_id IS NULL OR batch_id IS NOT NULL);
ALTER TABLE deviations
    ADD CONSTRAINT deviations_step_fkey FOREIGN KEY (workspace_id, batch_id, step_id)
        REFERENCES batch_steps (workspace_id, batch_id, id),
    ADD CONSTRAINT deviations_actual_fkey FOREIGN KEY (workspace_id, batch_id, actual_id)
        REFERENCES batch_ingredient_actuals (workspace_id, batch_id, id);

-- 23: malzemenin türü sonradan değişmez (1:1 tür satırı tutarsız kalmasın) -------------------
CREATE FUNCTION ingredients_reject_kind_change() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF NEW.kind IS DISTINCT FROM OLD.kind THEN
        RAISE EXCEPTION 'ingredient % cannot change kind (% -> %)', OLD.id, OLD.kind, NEW.kind
            USING ERRCODE = 'integrity_constraint_violation';
    END IF;
    RETURN NEW;
END
$$;
CREATE TRIGGER a_kind_fixed BEFORE UPDATE OF kind ON ingredients
    FOR EACH ROW EXECUTE FUNCTION ingredients_reject_kind_change();

-- 24: yabancı anahtar indeksleri (silme ve "bu malzeme hangi tarifte" sorguları) -----------
CREATE INDEX calc_results_recipe_version_idx ON calc_results (workspace_id, recipe_version_id)
    WHERE recipe_version_id IS NOT NULL;
CREATE INDEX calc_results_batch_idx ON calc_results (workspace_id, batch_id) WHERE batch_id IS NOT NULL;
CREATE INDEX measurements_package_idx ON measurements (workspace_id, package_id) WHERE package_id IS NOT NULL;
CREATE INDEX tasting_notes_batch_idx ON tasting_notes (workspace_id, batch_id);
CREATE INDEX recipe_fermentables_ingredient_idx ON recipe_fermentables (ingredient_id) WHERE ingredient_id IS NOT NULL;
CREATE INDEX recipe_hops_ingredient_idx ON recipe_hops (ingredient_id) WHERE ingredient_id IS NOT NULL;
CREATE INDEX recipe_yeasts_ingredient_idx ON recipe_yeasts (ingredient_id) WHERE ingredient_id IS NOT NULL;
CREATE INDEX recipe_miscs_ingredient_idx ON recipe_miscs (ingredient_id) WHERE ingredient_id IS NOT NULL;
CREATE INDEX recipe_versions_equipment_idx ON recipe_versions (workspace_id, equipment_id) WHERE equipment_id IS NOT NULL;
CREATE INDEX batches_equipment_idx ON batches (workspace_id, equipment_id) WHERE equipment_id IS NOT NULL;
CREATE INDEX devices_batch_idx ON devices (workspace_id, batch_id) WHERE batch_id IS NOT NULL;
CREATE INDEX lab_results_spec_idx ON lab_results (workspace_id, spec_id) WHERE spec_id IS NOT NULL;
CREATE INDEX releases_packaging_run_idx ON releases (workspace_id, packaging_run_id) WHERE packaging_run_id IS NOT NULL;
CREATE INDEX yeast_generations_source_batch_idx ON yeast_generations (workspace_id, source_batch_id)
    WHERE source_batch_id IS NOT NULL;
CREATE INDEX batch_ingredient_actuals_lot_idx ON batch_ingredient_actuals (workspace_id, lot_id) WHERE lot_id IS NOT NULL;
CREATE INDEX scanned_documents_batch_idx ON scanned_documents (workspace_id, batch_id) WHERE batch_id IS NOT NULL;
CREATE INDEX import_sources_recipe_idx ON import_sources (workspace_id, recipe_id) WHERE recipe_id IS NOT NULL;
CREATE INDEX role_assignments_role_idx ON role_assignments (workspace_id, role_id);
CREATE INDEX invitations_role_idx ON invitations (workspace_id, role_id);

-- 26: bir sonuç iki ayrı düzeltmeyle düzeltilemez ----------------------------------------------
CREATE UNIQUE INDEX lab_results_corrects_idx ON lab_results (workspace_id, corrects_id)
    WHERE corrects_id IS NOT NULL;

-- 32: demleme günü paketi içerik ve imza bakımından değişmez; yalnız geçersiz kılma ve süre ---
REVOKE UPDATE ON brew_day_packages FROM app_api, app_worker;
GRANT UPDATE (expires_at, revoked_at, deleted_at, updated_at, updated_by, version)
    ON brew_day_packages TO app_api, app_worker;

-- 36: çok biçimli hedef kilitlenir (hedef aynı anda silinirse sallanan kapsam kalmaz) ----------
CREATE OR REPLACE FUNCTION check_polymorphic_target() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE
    kind text := to_jsonb(NEW) ->> TG_ARGV[0];
    target uuid := (to_jsonb(NEW) ->> TG_ARGV[1])::uuid;
    tbl text;
    found integer;
BEGIN
    IF kind IS NULL OR target IS NULL THEN
        RETURN NEW;
    END IF;
    tbl := CASE kind
        WHEN 'recipe' THEN 'recipes'
        WHEN 'batch' THEN 'batches'
        WHEN 'vessel' THEN 'vessels'
        WHEN 'area' THEN 'production_areas'
        WHEN 'device' THEN 'devices'
        WHEN 'tasting_note' THEN 'tasting_notes'
        WHEN 'support_message' THEN 'support_messages' END;
    IF tbl IS NULL OR to_regclass(format('public.%I', tbl)) IS NULL THEN
        RAISE EXCEPTION 'scope kind % has no target table yet', kind
            USING ERRCODE = 'foreign_key_violation';
    END IF;
    EXECUTE format('SELECT 1 FROM %I WHERE id = $1 AND workspace_id = $2 FOR KEY SHARE', tbl)
        INTO found USING target, NEW.workspace_id;
    IF found IS NULL THEN
        RAISE EXCEPTION '% % does not exist in this workspace', kind, target
            USING ERRCODE = 'foreign_key_violation';
    END IF;
    RETURN NEW;
END
$$;

-- 39: hesap silinince kişisel alanlar boşaltılmış olmalı (mezar taşı, adr/0007 › 6.2) --------
ALTER TABLE users ADD CONSTRAINT users_tombstone_check
    CHECK (deleted_at IS NULL OR (contact_email IS NULL AND display_name = ''));

RESET ROLE;
