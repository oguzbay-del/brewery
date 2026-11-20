-- Son silinenler (T2.37): 30 günü geçen yumuşak silmeleri kalıcı siler (adr/0007 › 6.1).
-- Çağıran RLS altındadır (SECURITY INVOKER): worker bağlamı çalışma alanına kurar ve yalnız o
-- çalışma alanının satırları görünür. Kullanımdaki kayıt kalıcı silinmez (F2.1 karar 9):
-- yabancı anahtar ya da saklama tetikleyicisi silmeyi reddederse kayıt yumuşak silinmiş kalır
-- ve sayaçta `kept` olarak görünür. Her kayıt kendi alt işleminde silinir; bir ret ötekileri
-- etkilemez. Kullanıcı kaydı olmayan (kitaplık) malzeme çalışma alanı bağlamında görünmez.
SET LOCAL ROLE app_owner;

CREATE FUNCTION purge_expired_soft_deletes(p_cutoff timestamptz)
    RETURNS TABLE (kind text, purged bigint, kept bigint)
    LANGUAGE plpgsql VOLATILE
    SET search_path = pg_catalog, public, pg_temp
    AS $$
DECLARE
    spec constant text[][] := ARRAY[
        ARRAY['batch', 'batches'],
        ARRAY['recipe', 'recipes'],
        ARRAY['equipment', 'equipment'],
        ARRAY['ingredient', 'ingredients']];
    r record;
    n integer;
BEGIN
    FOR n IN 1 .. array_length(spec, 1) LOOP
        kind := spec[n][1];
        purged := 0;
        kept := 0;
        FOR r IN EXECUTE format(
            'SELECT id FROM %I WHERE deleted_at IS NOT NULL AND deleted_at < $1 %s ORDER BY id',
            spec[n][2],
            CASE WHEN spec[n][2] = 'ingredients' THEN 'AND source = ''user''' ELSE '' END)
            USING p_cutoff
        LOOP
            BEGIN
                EXECUTE format('DELETE FROM %I WHERE id = $1 AND deleted_at < $2', spec[n][2])
                    USING r.id, p_cutoff;
                purged := purged + 1;
            EXCEPTION WHEN integrity_constraint_violation THEN
                kept := kept + 1;
            END;
        END LOOP;
        RETURN NEXT;
    END LOOP;
END
$$;

REVOKE ALL ON FUNCTION purge_expired_soft_deletes(timestamptz) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION purge_expired_soft_deletes(timestamptz) TO app_worker;

RESET ROLE;
