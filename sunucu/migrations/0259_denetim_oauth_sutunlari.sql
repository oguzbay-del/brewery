-- M2: denetim satırına kanal ve OAuth bilgisi (adr/0015 › 5.2, 5.3).
--
-- via: işlemin kanalı (`app` uygulama oturumu, `mcp`, `api` açık API). oauth_client_id,
-- oauth_grant_id, oauth_scope: OAuth belirteciyle yapılan yazmada istemci, onay ve çağrının
-- kapsamı. chain_version: özet zincirinin biçimi. Sürüm 1 eski satırlardır (yeni sütunlar
-- özete girmez); sürüm 2 satırlarda dört yeni alan kanonik biçimde özete girer. Eski satırlar
-- eski biçimle doğrulanır. Yeni satırları API sürüm 2 ile yazar.
SET LOCAL ROLE app_owner;

ALTER TABLE audit_log
    ADD COLUMN chain_version  smallint NOT NULL DEFAULT 1 CHECK (chain_version IN (1, 2)),
    ADD COLUMN via            text NOT NULL DEFAULT 'app' CHECK (via IN ('app', 'mcp', 'api')),
    ADD COLUMN oauth_client_id uuid,
    ADD COLUMN oauth_grant_id  uuid,
    ADD COLUMN oauth_scope     text CHECK (char_length(oauth_scope) <= 100);

RESET ROLE;
