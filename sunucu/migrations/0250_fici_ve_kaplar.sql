-- K1: reusable containers, scan codes and immutable event history.
SET LOCAL ROLE app_owner;

CREATE TABLE container (
    id                uuid PRIMARY KEY,
    workspace_id      uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    serial_number     text NOT NULL CHECK (char_length(serial_number) BETWEEN 1 AND 100),
    package_type_id   uuid NOT NULL REFERENCES package_types (id),
    status            text NOT NULL DEFAULT 'clean'
                      CHECK (status IN ('clean', 'filled', 'out', 'dirty', 'maintenance', 'retired')),
    -- Independent nullable references: the packaging line and the traceable stock lot.
    last_fill_line_id uuid,
    last_lot_id       uuid,
    out_note          text CHECK (out_note IS NULL OR char_length(out_note) <= 2000),
    created_at        timestamptz NOT NULL DEFAULT now(),
    updated_at        timestamptz NOT NULL DEFAULT now(),
    deleted_at        timestamptz,
    version           bigint NOT NULL DEFAULT 1,
    created_by        uuid REFERENCES users (id),
    updated_by        uuid REFERENCES users (id),
    UNIQUE (workspace_id, id),
    UNIQUE (workspace_id, serial_number),
    FOREIGN KEY (workspace_id, last_fill_line_id)
        REFERENCES packaging_run_lines (workspace_id, id) ON DELETE SET NULL (last_fill_line_id),
    FOREIGN KEY (workspace_id, last_lot_id)
        REFERENCES stock_lots (workspace_id, id) ON DELETE SET NULL (last_lot_id)
);
CREATE INDEX container_last_lot_idx ON container (workspace_id, last_lot_id)
    WHERE last_lot_id IS NOT NULL;
CREATE TRIGGER shared_ref_package_type_id
    BEFORE INSERT OR UPDATE OF package_type_id, workspace_id ON container
    FOR EACH ROW EXECUTE FUNCTION check_shared_ref('package_types', 'package_type_id');

-- Store only the SHA-256 digest of a cryptographically random scan token. The API returns
-- the opaque token once; it must not derive it from the serial number.
CREATE TABLE scan_code (
    id           uuid PRIMARY KEY,
    workspace_id uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    code_hash    bytea NOT NULL CHECK (octet_length(code_hash) = 32),
    container_id uuid,
    vessel_id    uuid,
    lot_id       uuid,
    revoked_at   timestamptz,
    created_at   timestamptz NOT NULL DEFAULT now(),
    created_by   uuid REFERENCES users (id),
    UNIQUE (workspace_id, id),
    UNIQUE (workspace_id, code_hash),
    CHECK (num_nonnulls(container_id, vessel_id, lot_id) = 1),
    FOREIGN KEY (workspace_id, container_id)
        REFERENCES container (workspace_id, id) ON DELETE CASCADE,
    FOREIGN KEY (workspace_id, vessel_id)
        REFERENCES vessels (workspace_id, id) ON DELETE CASCADE,
    FOREIGN KEY (workspace_id, lot_id)
        REFERENCES stock_lots (workspace_id, id) ON DELETE CASCADE
);
CREATE INDEX scan_code_target_container_idx ON scan_code (workspace_id, container_id)
    WHERE container_id IS NOT NULL;
CREATE INDEX scan_code_target_vessel_idx ON scan_code (workspace_id, vessel_id)
    WHERE vessel_id IS NOT NULL;
CREATE INDEX scan_code_target_lot_idx ON scan_code (workspace_id, lot_id)
    WHERE lot_id IS NOT NULL;

CREATE TABLE container_event (
    id                uuid PRIMARY KEY,
    workspace_id      uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    container_id      uuid NOT NULL,
    event_type        text NOT NULL CHECK (event_type IN ('scan', 'status_changed', 'fill', 'note')),
    scan_code_id      uuid,
    user_id           uuid REFERENCES users (id),
    occurred_at       timestamptz NOT NULL DEFAULT now(),
    old_status        text CHECK (old_status IS NULL OR old_status IN (
                          'clean', 'filled', 'out', 'dirty', 'maintenance', 'retired')),
    new_status        text CHECK (new_status IS NULL OR new_status IN (
                          'clean', 'filled', 'out', 'dirty', 'maintenance', 'retired')),
    last_fill_line_id uuid,
    last_lot_id       uuid,
    note              text CHECK (note IS NULL OR char_length(note) <= 2000),
    created_at        timestamptz NOT NULL DEFAULT now(),
    UNIQUE (workspace_id, id),
    FOREIGN KEY (workspace_id, container_id)
        REFERENCES container (workspace_id, id) ON DELETE CASCADE,
    FOREIGN KEY (workspace_id, scan_code_id)
        REFERENCES scan_code (workspace_id, id) ON DELETE SET NULL (scan_code_id),
    FOREIGN KEY (workspace_id, last_fill_line_id)
        REFERENCES packaging_run_lines (workspace_id, id) ON DELETE SET NULL (last_fill_line_id),
    FOREIGN KEY (workspace_id, last_lot_id)
        REFERENCES stock_lots (workspace_id, id) ON DELETE SET NULL (last_lot_id)
);
CREATE INDEX container_event_history_idx
    ON container_event (workspace_id, container_id, occurred_at DESC);

CREATE FUNCTION container_fill_guard() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF NEW.status = 'filled' THEN
        IF TG_OP = 'INSERT' THEN
            RAISE EXCEPTION 'a new container cannot start filled'
                USING ERRCODE = 'integrity_constraint_violation';
        ELSIF OLD.status IN ('dirty', 'maintenance', 'retired') THEN
            RAISE EXCEPTION 'a dirty, maintenance or retired container cannot be filled'
                USING ERRCODE = 'integrity_constraint_violation';
        END IF;
    END IF;
    RETURN NEW;
END
$$;
CREATE TRIGGER container_fill_allowed
    BEFORE INSERT OR UPDATE OF status ON container
    FOR EACH ROW EXECUTE FUNCTION container_fill_guard();

-- Event rows are append-only. The only permitted nested updates are FK SET NULL actions;
-- workspace purge may cascade-delete the history together with its workspace.
CREATE FUNCTION container_event_reject_change() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF TG_OP = 'UPDATE' AND pg_trigger_depth() > 1
       AND (to_jsonb(NEW) - ARRAY['scan_code_id', 'last_fill_line_id', 'last_lot_id'])
           = (to_jsonb(OLD) - ARRAY['scan_code_id', 'last_fill_line_id', 'last_lot_id'])
       AND (NEW.scan_code_id IS NULL OR NEW.scan_code_id IS NOT DISTINCT FROM OLD.scan_code_id)
       AND (NEW.last_fill_line_id IS NULL OR NEW.last_fill_line_id IS NOT DISTINCT FROM OLD.last_fill_line_id)
       AND (NEW.last_lot_id IS NULL OR NEW.last_lot_id IS NOT DISTINCT FROM OLD.last_lot_id) THEN
        RETURN NEW;
    END IF;
    IF TG_OP = 'DELETE' AND pg_trigger_depth() > 1 THEN
        RETURN OLD;
    END IF;
    RAISE EXCEPTION 'container_event is append-only (% rejected)', TG_OP
        USING ERRCODE = 'integrity_constraint_violation';
END
$$;
CREATE TRIGGER container_event_no_change
    BEFORE UPDATE OR DELETE ON container
    FOR EACH ROW EXECUTE FUNCTION container_event_reject_change();
CREATE TRIGGER container_event_no_truncate
    BEFORE TRUNCATE ON container_event
    FOR EACH STATEMENT EXECUTE FUNCTION container_event_reject_change();

SELECT secure_workspace_table('container', 'standard', 'SELECT, INSERT, UPDATE');
SELECT secure_workspace_table('scan_code');
SELECT secure_workspace_table('container_event', 'append', 'SELECT, INSERT');

RESET ROLE;
