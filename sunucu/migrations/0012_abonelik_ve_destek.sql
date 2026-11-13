-- Abonelik ve destek (adr/0007 › 3 "Abonelik ve destek"; T2.6).
SET LOCAL ROLE app_owner;

-- subscriptions ve entitlements'ı yalnız sunucu yazar: app_api yalnız okur, yazma worker'dadır
-- (App Store ve ödeme olayları external_events'ten worker işler).
CREATE TABLE subscriptions (
    id                uuid PRIMARY KEY,
    workspace_id      uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    source            text NOT NULL CHECK (source IN ('app_store', 'web')),
    external_id       text NOT NULL CHECK (char_length(external_id) BETWEEN 1 AND 255),
    product_key       text NOT NULL CHECK (char_length(product_key) BETWEEN 1 AND 200),
    status            text NOT NULL CHECK (status IN ('trial', 'active', 'grace', 'expired', 'cancelled')),
    period_ends_at    timestamptz,
    trial_ends_at     timestamptz,
    last_verified_at  timestamptz,
    created_at        timestamptz NOT NULL DEFAULT now(),
    updated_at        timestamptz NOT NULL DEFAULT now(),
    deleted_at        timestamptz,
    version           bigint NOT NULL DEFAULT 1,
    created_by        uuid REFERENCES users (id),
    updated_by        uuid REFERENCES users (id),
    UNIQUE (workspace_id, id),
    UNIQUE (source, external_id)
);

CREATE TABLE entitlements (
    id                    uuid PRIMARY KEY,
    workspace_id          uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    feature               text NOT NULL CHECK (feature ~ '^[a-z][a-z0-9_.]*$'),
    source                text NOT NULL CHECK (source IN ('subscription', 'trial', 'promo', 'manual')),
    subscription_id       uuid,
    valid_from            timestamptz NOT NULL DEFAULT now(),
    valid_until           timestamptz,
    -- Elle düzeltmede gerekçe ve düzelten destek çalışanı zorunludur (adr/0007 › 3).
    reason                text,
    corrected_by_staff_id uuid REFERENCES staff_users (id),
    created_at            timestamptz NOT NULL DEFAULT now(),
    updated_at            timestamptz NOT NULL DEFAULT now(),
    deleted_at            timestamptz,
    version               bigint NOT NULL DEFAULT 1,
    created_by            uuid REFERENCES users (id),
    updated_by            uuid REFERENCES users (id),
    UNIQUE (workspace_id, id),
    FOREIGN KEY (workspace_id, subscription_id) REFERENCES subscriptions (workspace_id, id),
    CHECK (valid_until IS NULL OR valid_until > valid_from),
    CHECK (source <> 'manual' OR (reason IS NOT NULL AND corrected_by_staff_id IS NOT NULL)),
    CHECK (source <> 'subscription' OR subscription_id IS NOT NULL)
);
CREATE INDEX entitlements_workspace_idx ON entitlements (workspace_id, feature) WHERE deleted_at IS NULL;

-- support_requests ---------------------------------------------------------------------
CREATE TABLE support_requests (
    id                    uuid PRIMARY KEY,
    workspace_id          uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    -- Genel sıra: bütün çalışma alanlarında artan, insana gösterilen numara (adr/0007 › 1.1).
    public_number         bigint GENERATED ALWAYS AS IDENTITY UNIQUE,
    topic                 text NOT NULL CHECK (topic ~ '^[a-z][a-z0-9_]*$'),
    title                 text NOT NULL CHECK (char_length(title) BETWEEN 1 AND 200),
    related_record_kind   text CHECK (related_record_kind ~ '^[a-z][a-z0-9_]*$'),
    related_record_id     uuid,
    status                text NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'answered', 'resolved', 'closed')),
    diagnostics           jsonb CHECK (jsonb_typeof(diagnostics) = 'object' AND diagnostics ? 'schema_version'),
    github_issue_number   integer CHECK (github_issue_number > 0),
    github_synced_at      timestamptz,
    previous_request_id   uuid,
    created_at            timestamptz NOT NULL DEFAULT now(),
    updated_at            timestamptz NOT NULL DEFAULT now(),
    deleted_at            timestamptz,
    version               bigint NOT NULL DEFAULT 1,
    created_by            uuid REFERENCES users (id),
    updated_by            uuid REFERENCES users (id),
    UNIQUE (workspace_id, id),
    FOREIGN KEY (workspace_id, previous_request_id) REFERENCES support_requests (workspace_id, id),
    CHECK ((related_record_kind IS NULL) = (related_record_id IS NULL))
);
CREATE INDEX support_requests_workspace_idx ON support_requests (workspace_id, status);

CREATE TABLE support_messages (
    id                 uuid PRIMARY KEY,
    workspace_id       uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    request_id         uuid NOT NULL,
    author_kind        text NOT NULL CHECK (author_kind IN ('customer', 'staff', 'system')),
    author_user_id     uuid REFERENCES users (id),
    author_staff_id    uuid REFERENCES staff_users (id),
    body               text NOT NULL CHECK (char_length(body) <= 20000),
    visibility         text NOT NULL DEFAULT 'customer' CHECK (visibility IN ('customer', 'internal')),
    github_comment_id  bigint,
    edited_at          timestamptz,
    retracted_at       timestamptz,
    created_at         timestamptz NOT NULL DEFAULT now(),
    updated_at         timestamptz NOT NULL DEFAULT now(),
    deleted_at         timestamptz,
    version            bigint NOT NULL DEFAULT 1,
    created_by         uuid REFERENCES users (id),
    updated_by         uuid REFERENCES users (id),
    UNIQUE (workspace_id, id),
    FOREIGN KEY (workspace_id, request_id) REFERENCES support_requests (workspace_id, id) ON DELETE CASCADE,
    CHECK (num_nonnulls(author_user_id, author_staff_id) <= 1),
    CHECK (author_kind <> 'customer' OR author_user_id IS NOT NULL),
    CHECK (author_kind <> 'staff' OR author_staff_id IS NOT NULL),
    CHECK (author_kind <> 'system' OR num_nonnulls(author_user_id, author_staff_id) = 0),
    CHECK (author_kind <> 'customer' OR visibility = 'customer')
);
CREATE INDEX support_messages_request_idx ON support_messages (workspace_id, request_id, created_at);

-- support_grants -----------------------------------------------------------------------
-- Müşterinin verdiği erişim izni: en çok 72 saat (adr/0007 › 3). Acil erişimde iki farklı
-- onaylayan ve müşteriye bildirim zamanı kayıtlıdır; olağan izinde veren üye zorunludur.
CREATE TABLE support_grants (
    id                    uuid PRIMARY KEY,
    workspace_id          uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    request_id            uuid,
    granted_by_member_id  uuid,
    starts_at             timestamptz NOT NULL DEFAULT now(),
    ends_at               timestamptz NOT NULL,
    revoked_at            timestamptz,
    emergency             boolean NOT NULL DEFAULT false,
    approver_1_staff_id   uuid REFERENCES staff_users (id),
    approver_2_staff_id   uuid REFERENCES staff_users (id),
    customer_notified_at  timestamptz,
    created_at            timestamptz NOT NULL DEFAULT now(),
    updated_at            timestamptz NOT NULL DEFAULT now(),
    deleted_at            timestamptz,
    version               bigint NOT NULL DEFAULT 1,
    created_by            uuid REFERENCES users (id),
    updated_by            uuid REFERENCES users (id),
    UNIQUE (workspace_id, id),
    FOREIGN KEY (workspace_id, request_id) REFERENCES support_requests (workspace_id, id),
    FOREIGN KEY (workspace_id, granted_by_member_id) REFERENCES workspace_members (workspace_id, id),
    CHECK (ends_at > starts_at AND ends_at <= starts_at + interval '72 hours'),
    CHECK (emergency OR granted_by_member_id IS NOT NULL),
    CHECK (NOT emergency OR (approver_1_staff_id IS NOT NULL AND approver_2_staff_id IS NOT NULL
                             AND approver_1_staff_id <> approver_2_staff_id))
);
CREATE INDEX support_grants_active_idx ON support_grants (workspace_id, ends_at) WHERE revoked_at IS NULL;

-- support_actions: destek ekibinin her işlemi, yalnız eklenir (adr/0007 › 3).
CREATE TABLE support_actions (
    id             uuid PRIMARY KEY,
    workspace_id   uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    staff_id       uuid NOT NULL REFERENCES staff_users (id),
    request_id     uuid,
    action         text NOT NULL CHECK (char_length(action) BETWEEN 1 AND 100),
    record_kind    text CHECK (char_length(record_kind) BETWEEN 1 AND 100),
    record_id      uuid,
    reason         text NOT NULL CHECK (char_length(reason) >= 1),
    created_at     timestamptz NOT NULL DEFAULT now(),
    FOREIGN KEY (workspace_id, request_id) REFERENCES support_requests (workspace_id, id)
);
CREATE INDEX support_actions_workspace_idx ON support_actions (workspace_id, created_at);

SELECT secure_workspace_table(t, 'standard', 'SELECT') FROM unnest(ARRAY[
    'subscriptions', 'entitlements']) t;
GRANT INSERT, UPDATE, DELETE ON subscriptions, entitlements TO app_worker;
SELECT secure_workspace_table(t) FROM unnest(ARRAY[
    'support_requests', 'support_messages', 'support_grants']) t;
SELECT secure_workspace_table('support_actions', 'append', 'SELECT, INSERT');

RESET ROLE;
