-- Vergi oran tabloları (adr/0007 › 3 "İşletme"; formuller/09 §2; ADR-0006 vergi açığı).
-- Ortak (çalışma alanı dışı) referans veridir: `workspace_id` yoktur, RLS yoktur; uygulama
-- rolleri yalnız okur, yazımı göç ve tohum (app_owner) yapar. Oranlar "doğrulanacak" başlar;
-- yalnız resmi kaynak ve kaynak tarihi birlikte kaydedilince `verified` olur (09 vektör 2.14).
-- Motor yöntemi koddadır, oran ve eşikler buradadır; T2.8 bu satırları motora girdi verir.
SET LOCAL ROLE app_owner;

CREATE TABLE tax_jurisdictions (
    id          uuid PRIMARY KEY,
    -- ISO 3166-1 alfa-2, isteğe bağlı bölge (US-CA).
    code        text NOT NULL UNIQUE CHECK (code ~ '^[A-Z]{2}(-[A-Z0-9]{1,3})?$'),
    name        text NOT NULL CHECK (char_length(name) BETWEEN 1 AND 200),
    currency    char(3) NOT NULL CHECK (currency ~ '^[A-Z]{3}$'),
    created_at  timestamptz NOT NULL DEFAULT now(),
    updated_at  timestamptz NOT NULL DEFAULT now()
);

-- Bir ülkenin bir yöntem adımı ve yürürlük aralığı. Birden çok yöntem zincirdir
-- (`chain_position` sırayı verir); aynı ülke, oran sınıfı ve zincir adımında yürürlük
-- aralıkları çakışmaz (EXCLUDE).
CREATE TABLE tax_rate_tables (
    id                uuid PRIMARY KEY,
    jurisdiction_id   uuid NOT NULL REFERENCES tax_jurisdictions (id) ON DELETE CASCADE,
    -- Aynı ülkede ayrı oranlar (ör. fıçı bira): sınıf tekil anahtara girer.
    rate_class        text NOT NULL DEFAULT 'beer' CHECK (char_length(rate_class) BETWEEN 1 AND 50),
    chain_position    smallint NOT NULL DEFAULT 1 CHECK (chain_position >= 1),
    method            text NOT NULL CHECK (method IN (
        'hacim', 'saf_alkol', 'hl_plato', 'plato_bandi', 'kademeli_hacim')),
    -- Yürürlük `[başlangıç, bitiş)`; bitiş boşsa süresiz.
    valid_during      daterange NOT NULL CHECK (NOT isempty(valid_during) AND NOT lower_inf(valid_during)),
    -- Verginin doğduğu an (09 §2.1).
    taxable_event     text NOT NULL CHECK (taxable_event IN ('packaging', 'excise_exit', 'sale')),
    abv_source        text CHECK (abv_source IN ('measured', 'declared')),
    -- Tek oran (hacim, hl_plato; saf_alkol'de bant yoksa). Birimi yönteme göre: hL, hL·°P, LPA başına.
    base_rate         numeric(19,6) CHECK (base_rate >= 0),
    -- Para ve yuvarlama (09 §2.2 ortak kurallar).
    rounding_scope    text NOT NULL DEFAULT 'line' CHECK (rounding_scope IN ('line', 'total')),
    rounding_mode     text NOT NULL DEFAULT 'half_up' CHECK (rounding_mode IN ('half_up', 'half_even', 'up', 'down')),
    rounding_dp       smallint NOT NULL DEFAULT 2 CHECK (rounding_dp BETWEEN 0 AND 4),
    -- ABV ve °P girdi yuvarlaması: ABV ya da °P kullanan yöntemde açıkça yazılır ('none' =
    -- yuvarlama yok); tanımsız bırakılamaz. `input_dp` yalnız yönlü yuvarlamada.
    input_rounding    text CHECK (input_rounding IN ('none', 'up', 'down', 'nearest')),
    input_dp          smallint CHECK (input_dp BETWEEN 0 AND 4),
    -- Bant üst sınırının banda dahil olup olmadığı (saf_alkol, plato_bandi: açıkça yazılır).
    bounds_inclusive  boolean,
    -- İndirim (ör. küçük üretici, LPA başına tutar) ve koşulları; oranı aşamaz (motor kenetler).
    discount_rate       numeric(19,6) CHECK (discount_rate >= 0),
    discount_conditions jsonb CHECK (discount_conditions IS NULL
        OR (jsonb_typeof(discount_conditions) = 'object' AND discount_conditions ? 'schema_version')),
    -- Kaynak ve doğrulama durumu.
    status            text NOT NULL DEFAULT 'unverified' CHECK (status IN ('unverified', 'verified')),
    source_name       text CHECK (char_length(source_name) <= 300),
    source_url        text CHECK (char_length(source_url) <= 2000),
    source_date       date,
    source_official   boolean NOT NULL DEFAULT false,
    notes             text,
    created_at        timestamptz NOT NULL DEFAULT now(),
    updated_at        timestamptz NOT NULL DEFAULT now(),
    CHECK (method NOT IN ('hacim', 'hl_plato') OR base_rate IS NOT NULL),
    CHECK (method NOT IN ('plato_bandi', 'kademeli_hacim') OR base_rate IS NULL),
    CHECK ((method IN ('saf_alkol', 'hl_plato', 'plato_bandi')) = (input_rounding IS NOT NULL)),
    CHECK ((input_rounding IN ('up', 'down', 'nearest')) = (input_dp IS NOT NULL)),
    CHECK ((method IN ('saf_alkol', 'plato_bandi')) = (bounds_inclusive IS NOT NULL)),
    CHECK (method = 'saf_alkol' OR abv_source IS NULL),
    CHECK (method <> 'saf_alkol' OR abv_source IS NOT NULL),
    CHECK (discount_rate IS NULL OR method = 'saf_alkol'),
    -- Doğrulanmış satır resmi kaynak, adı ve tarihiyle gelir.
    CHECK (status = 'unverified'
           OR (source_official AND source_name IS NOT NULL AND source_url IS NOT NULL AND source_date IS NOT NULL)),
    CONSTRAINT tax_rate_tables_no_overlap
        EXCLUDE USING gist (jurisdiction_id WITH =, rate_class WITH =, chain_position WITH =,
                            valid_during WITH &&)
);
CREATE INDEX tax_rate_tables_lookup_idx ON tax_rate_tables USING gist (jurisdiction_id, valid_during);

-- Bant ve kademeler: saf_alkol (ABV bandı), plato_bandi (°P bandı), kademeli_hacim (yıllık
-- kümülatif hL kademesi). `upper_bound` boş = açık uç.
CREATE TABLE tax_rate_bands (
    id           uuid PRIMARY KEY,
    table_id     uuid NOT NULL REFERENCES tax_rate_tables (id) ON DELETE CASCADE,
    position     smallint NOT NULL CHECK (position >= 1),
    upper_bound  numeric(19,6) CHECK (upper_bound > 0),
    rate         numeric(19,6) NOT NULL CHECK (rate >= 0),
    created_at   timestamptz NOT NULL DEFAULT now(),
    updated_at   timestamptz NOT NULL DEFAULT now(),
    UNIQUE (table_id, position)
);

-- Motorun D8 kuralı (09 §2.2): üst sınırlar kesin artan, açık uç yalnız sonda ve son bant
-- açık uçlu. Satırlar tek tek eklendiği için işlem sonunda denetlenir; boş liste geçerlidir.
CREATE FUNCTION check_tax_bands() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE
    tbl uuid := coalesce(NEW.table_id, OLD.table_id);
    n integer;
    bad boolean;
    last_open boolean;
BEGIN
    SELECT count(*) INTO n FROM tax_rate_bands WHERE table_id = tbl;
    IF n = 0 THEN
        RETURN NULL;
    END IF;
    SELECT bool_or(upper_bound IS NULL AND position < n)
           OR bool_or(upper_bound <= lag_bound)
           OR bool_or(position <> rn),
           bool_or(upper_bound IS NULL AND position = n)
      INTO bad, last_open
    FROM (SELECT position, upper_bound,
                 lag(upper_bound) OVER (ORDER BY position) AS lag_bound,
                 row_number() OVER (ORDER BY position) AS rn
          FROM tax_rate_bands WHERE table_id = tbl) b;
    IF coalesce(bad, false) OR NOT coalesce(last_open, false) THEN
        RAISE EXCEPTION 'tax bands of % must be consecutive, strictly increasing and end open-ended', tbl
            USING ERRCODE = 'check_violation';
    END IF;
    RETURN NULL;
END
$$;
CREATE CONSTRAINT TRIGGER bands_valid AFTER INSERT OR UPDATE OR DELETE ON tax_rate_bands
    DEFERRABLE INITIALLY DEFERRED
    FOR EACH ROW EXECUTE FUNCTION check_tax_bands();

-- Ortak veri: uygulama rolleri yalnız okur, yazımı app_owner (göç ve tohum) yapar.
GRANT SELECT ON tax_jurisdictions, tax_rate_tables, tax_rate_bands TO app_api, app_worker;

RESET ROLE;
