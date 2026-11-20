-- Ekipman verim ayrımı (T2.40): dönüşüm ve süzme verimi, ikisi de boş olabilir.
-- İkisi birlikte dolu ya da birlikte boştur; motor çarpımı verim olarak kullanır.
SET LOCAL ROLE app_owner;

ALTER TABLE equipment
    ADD COLUMN conversion_efficiency_pct double precision
        CHECK (conversion_efficiency_pct > 0 AND conversion_efficiency_pct <= 100),
    ADD COLUMN lauter_efficiency_pct double precision
        CHECK (lauter_efficiency_pct > 0 AND lauter_efficiency_pct <= 100),
    ADD CONSTRAINT equipment_efficiency_split_ck
        CHECK ((conversion_efficiency_pct IS NULL) = (lauter_efficiency_pct IS NULL));

RESET ROLE;
