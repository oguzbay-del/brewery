-- B2: abonelik ve hak için plan anahtarı ve koltuk sayısı (adr/0013).
--
-- Plan anahtarı yalnız 'home' veya 'business' olur; katman adları, koltuk sınırları ve fiyatlar
-- karar bekler (adr/0013 › Açık kararlar). NULL plan, modül kümesinin çalışma alanı kipinden
-- geldiği anlamına gelir (eski satırlar, promosyon, elle düzeltme).

SET LOCAL ROLE app_owner;

ALTER TABLE subscriptions
    ADD COLUMN plan_key text CHECK (plan_key IN ('home', 'business')),
    ADD COLUMN seats integer CHECK (seats BETWEEN 1 AND 100000);

ALTER TABLE entitlements
    ADD COLUMN plan_key text CHECK (plan_key IN ('home', 'business')),
    ADD COLUMN seats integer CHECK (seats BETWEEN 1 AND 100000);

RESET ROLE;
