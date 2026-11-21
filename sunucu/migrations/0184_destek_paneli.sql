-- T2.30: destek paneli. Abonelik ve yetki düzeltmesi bir istek olarak yazılır, worker uygular
-- (yalnız app_worker `subscriptions` ve `entitlements` yazar). Acil erişim iki farklı destek
-- çalışanının onayıyla açılır. Çalışma alanı araması dar bir SECURITY DEFINER işleviyle yapılır.
SET LOCAL ROLE app_owner;

-- support_corrections: destek çalışanının gerekçeli düzeltme isteği. API ekler, worker uygular.
CREATE TABLE support_corrections (
    id            uuid PRIMARY KEY,
    workspace_id  uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    staff_id      uuid NOT NULL REFERENCES staff_users (id),
    kind          text NOT NULL CHECK (kind IN (
        'entitlement_grant', 'entitlement_revoke', 'subscription_reverify', 'deletion_request')),
    params        jsonb NOT NULL DEFAULT '{"schema_version": 1}'
                  CHECK (jsonb_typeof(params) = 'object' AND params ? 'schema_version'),
    reason        text NOT NULL CHECK (char_length(reason) BETWEEN 1 AND 500),
    status        text NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'applied', 'failed')),
    result        text CHECK (char_length(result) <= 500),
    applied_at    timestamptz,
    created_at    timestamptz NOT NULL DEFAULT now(),
    updated_at    timestamptz NOT NULL DEFAULT now(),
    deleted_at    timestamptz,
    version       bigint NOT NULL DEFAULT 1,
    created_by    uuid REFERENCES users (id),
    updated_by    uuid REFERENCES users (id),
    UNIQUE (workspace_id, id)
);
CREATE INDEX support_corrections_workspace_idx ON support_corrections (workspace_id, created_at);
SELECT secure_workspace_table('support_corrections');

-- support_emergency_requests: iki onaylı acil erişim. İsteyen ve onaylayan farklı kişidir; onay
-- gelince support_grants satırı (emergency) açılır.
CREATE TABLE support_emergency_requests (
    id                      uuid PRIMARY KEY,
    workspace_id            uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    requested_by_staff_id   uuid NOT NULL REFERENCES staff_users (id),
    reason                  text NOT NULL CHECK (char_length(reason) BETWEEN 1 AND 500),
    approved_by_staff_id    uuid REFERENCES staff_users (id),
    grant_id                uuid,
    status                  text NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'approved')),
    created_at              timestamptz NOT NULL DEFAULT now(),
    updated_at              timestamptz NOT NULL DEFAULT now(),
    deleted_at              timestamptz,
    version                 bigint NOT NULL DEFAULT 1,
    created_by              uuid REFERENCES users (id),
    updated_by              uuid REFERENCES users (id),
    UNIQUE (workspace_id, id),
    FOREIGN KEY (workspace_id, grant_id) REFERENCES support_grants (workspace_id, id),
    CHECK (approved_by_staff_id IS NULL OR approved_by_staff_id <> requested_by_staff_id),
    CHECK ((status = 'approved') = (approved_by_staff_id IS NOT NULL AND grant_id IS NOT NULL))
);
SELECT secure_workspace_table('support_emergency_requests');

-- Destek paneli aramasında çalışma alanı bulma: takma kimlik, kimlik ya da talep numarası.
-- Yalnız üst veri döner (müşteri içeriği yok). `workspaces` tablosunun owner_lookup politikası
-- ve support_requests'in owner_read politikası (0182) bu işlev içindir.
CREATE FUNCTION support_find_workspace(p_query text)
    RETURNS TABLE (workspace_id uuid, name text, mode text, region text, pseudonym text,
                   created_at timestamptz)
    LANGUAGE sql STABLE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
    SELECT w.id, w.name, w.mode, w.region, w.support_pseudonym, w.created_at
    FROM workspaces w
    WHERE w.deleted_at IS NULL
      AND (w.support_pseudonym = p_query
           OR w.id::text = p_query
           OR w.id IN (SELECT r.workspace_id FROM support_requests r
                       WHERE r.public_number::text = ltrim(p_query, '#')))
    ORDER BY w.created_at
    LIMIT 20
$$;
REVOKE ALL ON FUNCTION support_find_workspace(text) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION support_find_workspace(text) TO app_api;

RESET ROLE;
