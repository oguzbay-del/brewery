-- Veritabanı rolleri (adr/0007-veri-modeli.md › 2.4).
--   app_owner : tabloların sahibi; yalnız göç ve tohum işi kullanır (oturum açamaz).
--   app_api   : API ve ingest; tablo sahibi değil, BYPASSRLS yok.
--   app_worker: arka plan işleri; RLS altında, çalışma alanları arası tablolara ayrıca yetkili.
-- Roller küme çapındadır; göç yoksa oluşturur (NOLOGIN). Oturum açma ve parola göçten
-- sonra `migrate` ikilisi tarafından ortamdaki sırlarla verilir (db::provision_roles);
-- göçlerde parola bulunmaz. Bu göçü çalıştıran rol süper kullanıcı ya da CREATEROLE sahibi
-- ve veritabanının sahibi olmalıdır.

-- Roller küme çapında, danışma kilidi ise veritabanı başınadır: paralel göçler (her sqlx::test
-- kendi veritabanında) aynı rolü aynı anda oluşturabilir. Bu yüzden çakışma hatası yakalanır.
DO $$
DECLARE
    r text;
BEGIN
    FOREACH r IN ARRAY ARRAY['app_owner', 'app_api', 'app_worker'] LOOP
        IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = r) THEN
            BEGIN
                EXECUTE format(
                    'CREATE ROLE %I NOLOGIN NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE NOREPLICATION',
                    r);
            EXCEPTION WHEN duplicate_object OR unique_violation THEN
                NULL; -- başka bir göç aynı anda oluşturdu
            END;
        END IF;
    END LOOP;

    -- Başka yoldan oluşturulmuş bir rol yetkili çıkmışsa göç durur.
    IF EXISTS (
        SELECT FROM pg_roles
        WHERE rolname IN ('app_owner', 'app_api', 'app_worker')
          AND (rolsuper OR rolbypassrls OR rolcreaterole OR rolcreatedb)
    ) THEN
        RAISE EXCEPTION 'app_owner, app_api and app_worker must not be superuser, BYPASSRLS, CREATEROLE or CREATEDB';
    END IF;

    -- Süper kullanıcı olmayan göç rolü, sonraki göçlerde SET ROLE app_owner yapabilsin.
    IF NOT pg_has_role(current_user, 'app_owner', 'MEMBER') THEN
        BEGIN
            EXECUTE format('GRANT app_owner TO %I', current_user);
        EXCEPTION WHEN duplicate_object OR unique_violation THEN
            NULL;
        END;
    END IF;
END
$$;

-- Şema: yalnız app_owner nesne yaratır; uygulama rolleri kullanır.
REVOKE CREATE ON SCHEMA public FROM PUBLIC;
GRANT USAGE, CREATE ON SCHEMA public TO app_owner;
GRANT USAGE ON SCHEMA public TO app_api, app_worker;
