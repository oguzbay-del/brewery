-- B3: Paddle müşteri kimliği. Müşteri portalı oturumu için gerekir; ingest Paddle olaylarından
-- yazar, worker işler (`subscriptions` yalnız worker tarafından yazılır).
SET LOCAL ROLE app_owner;

ALTER TABLE subscriptions
    ADD COLUMN paddle_customer_id text CHECK (char_length(paddle_customer_id) BETWEEN 1 AND 255);

RESET ROLE;
