-- Bütün tabloların kullandığı ortak işlevler. Sonraki göçler SET LOCAL ROLE app_owner ile
-- başlar ve RESET ROLE ile biter: nesneler app_owner'ındır, sqlx kaydı göç rolüyle yazılır.
SET LOCAL ROLE app_owner;

-- İstek bağlamı (adr/0007 › 2.3). set_config(.., true) işlem sonunda ayarı NULL değil boş
-- metne çevirir; boş metin uuid'e çevrilemez, bu yüzden NULLIF. Bağlam yoksa NULL döner ve
-- hiçbir satır eşleşmez. Politikalar işlevi `(SELECT app_workspace_id())` biçiminde çağırır:
-- sorgu başına bir kez hesaplanır.
CREATE FUNCTION app_workspace_id() RETURNS uuid
    LANGUAGE sql STABLE PARALLEL SAFE
    AS $$ SELECT nullif(current_setting('app.workspace_id', true), '')::uuid $$;

CREATE FUNCTION app_user_id() RETURNS uuid
    LANGUAGE sql STABLE PARALLEL SAFE
    AS $$ SELECT nullif(current_setting('app.user_id', true), '')::uuid $$;

CREATE FUNCTION app_actor_kind() RETURNS text
    LANGUAGE sql STABLE PARALLEL SAFE
    AS $$ SELECT nullif(current_setting('app.actor_kind', true), '') $$;

GRANT EXECUTE ON FUNCTION app_workspace_id(), app_user_id(), app_actor_kind()
    TO app_api, app_worker;

-- İyimser kilit (adr/0007 › 1.2): her UPDATE'te version artar, updated_at yenilenir.
CREATE FUNCTION touch_row() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    NEW.version := OLD.version + 1;
    NEW.updated_at := now();
    RETURN NEW;
END
$$;

RESET ROLE;
