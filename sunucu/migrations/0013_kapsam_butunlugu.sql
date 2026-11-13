-- Çok biçimli başvuruların bütünlüğü (adr/0007; T2.5 notu: role_assignments.scope_id).
-- `scope_id` gibi sütunlar farklı tablolara işaret eder ve yabancı anahtar kurulamaz;
-- tetikleyici, tür sütununa göre hedef tablonun aynı çalışma alanında o satırı barındırdığını
-- denetler. Hedef tablo henüz yoksa (vessels ve production_areas T2.7'de gelir) o tür reddedilir.
-- Çağıran RLS altındadır: başka çalışma alanının satırı görünmez.
SET LOCAL ROLE app_owner;

-- TG_ARGV[0] tür sütunu, TG_ARGV[1] kimlik sütunu.
CREATE FUNCTION check_polymorphic_target() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE
    kind text := to_jsonb(NEW) ->> TG_ARGV[0];
    target uuid := (to_jsonb(NEW) ->> TG_ARGV[1])::uuid;
    tbl text;
    n bigint;
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
    EXECUTE format('SELECT count(*) FROM %I WHERE id = $1 AND workspace_id = $2', tbl)
        INTO n USING target, NEW.workspace_id;
    IF n = 0 THEN
        RAISE EXCEPTION '% % does not exist in this workspace', kind, target
            USING ERRCODE = 'foreign_key_violation';
    END IF;
    RETURN NEW;
END
$$;

CREATE TRIGGER scope_target BEFORE INSERT OR UPDATE OF scope_kind, scope_id, workspace_id ON role_assignments
    FOR EACH ROW EXECUTE FUNCTION check_polymorphic_target('scope_kind', 'scope_id');
CREATE TRIGGER scope_target BEFORE INSERT OR UPDATE OF scope_kind, scope_id, workspace_id ON invitations
    FOR EACH ROW EXECUTE FUNCTION check_polymorphic_target('scope_kind', 'scope_id');
CREATE TRIGGER scope_target BEFORE INSERT OR UPDATE OF scope_kind, scope_id, workspace_id ON alert_rules
    FOR EACH ROW EXECUTE FUNCTION check_polymorphic_target('scope_kind', 'scope_id');
CREATE TRIGGER record_target BEFORE INSERT OR UPDATE OF record_kind, record_id, workspace_id ON attachments
    FOR EACH ROW EXECUTE FUNCTION check_polymorphic_target('record_kind', 'record_id');

-- Hedef satır silinince kapsamlı atama ve davetler de gider (sallanan kapsam kalmaz).
-- TG_ARGV[0] kapsam türü. Hedefin silinmesi hard delete'tir (yumuşak silme saklama süresi
-- dolunca); alarm kuralları ve ekler hedefleriyle birlikte temizlenir.
CREATE FUNCTION drop_scope_references() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    DELETE FROM role_assignments
    WHERE workspace_id = OLD.workspace_id AND scope_kind = TG_ARGV[0] AND scope_id = OLD.id;
    DELETE FROM invitations
    WHERE workspace_id = OLD.workspace_id AND scope_kind = TG_ARGV[0] AND scope_id = OLD.id;
    IF TG_ARGV[0] IN ('batch', 'device') THEN
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
CREATE TRIGGER drop_scope AFTER DELETE ON recipes
    FOR EACH ROW EXECUTE FUNCTION drop_scope_references('recipe');
CREATE TRIGGER drop_scope AFTER DELETE ON batches
    FOR EACH ROW EXECUTE FUNCTION drop_scope_references('batch', 'batch');
CREATE TRIGGER drop_scope AFTER DELETE ON devices
    FOR EACH ROW EXECUTE FUNCTION drop_scope_references('device');
CREATE TRIGGER drop_scope AFTER DELETE ON tasting_notes
    FOR EACH ROW EXECUTE FUNCTION drop_scope_references('tasting_note', 'tasting_note');
CREATE TRIGGER drop_scope AFTER DELETE ON support_messages
    FOR EACH ROW EXECUTE FUNCTION drop_scope_references('support_message', 'support_message');

RESET ROLE;
