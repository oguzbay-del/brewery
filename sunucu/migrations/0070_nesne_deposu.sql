-- Nesne deposu yüklemesi (T2.18; adr/0008 › 11.1).
-- Kayıt, yükleme bağlantısı verilirken açılır; dosya doğrudan nesne deposuna gider. `uploaded_at`
-- NULL ise yükleme tamamlanmamıştır (istemci `complete` çağırana kadar ya da hiç): indirme
-- bağlantısı verilmez ve temizlik işi bu kayıtları süre dolunca siler.
SET LOCAL ROLE app_owner;

ALTER TABLE attachments ADD COLUMN uploaded_at timestamptz;
ALTER TABLE scanned_documents ADD COLUMN uploaded_at timestamptz;

CREATE INDEX attachments_pending_idx ON attachments (created_at) WHERE uploaded_at IS NULL;
CREATE INDEX scanned_documents_pending_idx ON scanned_documents (created_at) WHERE uploaded_at IS NULL;

-- Yasal tutma yalnız DELETE'i değil yumuşak silmeyi de (deleted_at) engeller (adr/0007 › 6.3).
CREATE FUNCTION scanned_documents_guard_soft_delete() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF OLD.deleted_at IS NULL AND NEW.deleted_at IS NOT NULL
       AND OLD.legal_hold AND (OLD.retention_until IS NULL OR OLD.retention_until > now()) THEN
        RAISE EXCEPTION 'scanned document % is under legal hold', OLD.id
            USING ERRCODE = 'integrity_constraint_violation';
    END IF;
    RETURN NEW;
END
$$;
CREATE TRIGGER legal_hold_soft BEFORE UPDATE OF deleted_at ON scanned_documents
    FOR EACH ROW EXECUTE FUNCTION scanned_documents_guard_soft_delete();

RESET ROLE;
