-- T2.24: bildirim teslimi, tercihler, sessiz saat ve çalışma alanları arası "bekleyen iş" işlevi.
-- Bildirim satırı API'de (bildirim kutusu) ya da worker'da (uyarı kuralı, hatırlatıcı, destek yanıtı)
-- doğar; worker teslim eder (push, e-posta). Sistem metni tutulmaz: metin anahtarı ve parametre.
SET LOCAL ROLE app_owner;

-- dedupe_key: aynı olayın uyarıları birleşir (kural + zaman penceresi + alıcı). deliver_after:
-- sessiz saat teslimi öteler. delivered_at: push ve e-posta kararı verildi.
ALTER TABLE notifications
    ADD COLUMN dedupe_key    text CHECK (char_length(dedupe_key) <= 300),
    ADD COLUMN deliver_after timestamptz NOT NULL DEFAULT now(),
    ADD COLUMN delivered_at  timestamptz;
CREATE UNIQUE INDEX notifications_dedupe_idx ON notifications (workspace_id, dedupe_key)
    WHERE dedupe_key IS NOT NULL;
CREATE INDEX notifications_pending_idx ON notifications (deliver_after)
    WHERE delivered_at IS NULL AND deleted_at IS NULL AND status = 'open';
CREATE INDEX notifications_snoozed_idx ON notifications (snoozed_until)
    WHERE status = 'snoozed' AND deleted_at IS NULL;

ALTER TABLE notification_events DROP CONSTRAINT notification_events_kind_check;
ALTER TABLE notification_events ADD CONSTRAINT notification_events_kind_check CHECK (kind IN (
    'sent', 'read', 'acknowledged', 'snoozed', 'dismissed', 'email_sent', 'suppressed'));

-- Kullanıcı başına, çalışma alanı başına bildirim tercihi. Satır yoksa varsayılanlar geçerlidir
-- (push açık, e-posta açık, sessiz saat ve günlük özet kapalı, saat dilimi çalışma alanınınki).
CREATE TABLE notification_preferences (
    id             uuid PRIMARY KEY,
    workspace_id   uuid NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    user_id        uuid NOT NULL REFERENCES users (id),
    timezone       text NOT NULL DEFAULT 'UTC' CHECK (char_length(timezone) BETWEEN 1 AND 64),
    quiet_start    time,
    quiet_end      time,
    push_enabled   boolean NOT NULL DEFAULT true,
    email_enabled  boolean NOT NULL DEFAULT true,
    daily_summary  boolean NOT NULL DEFAULT false,
    summary_time   time NOT NULL DEFAULT '08:00',
    last_summary_on date,
    created_at     timestamptz NOT NULL DEFAULT now(),
    updated_at     timestamptz NOT NULL DEFAULT now(),
    deleted_at     timestamptz,
    version        bigint NOT NULL DEFAULT 1,
    created_by     uuid REFERENCES users (id),
    updated_by     uuid REFERENCES users (id),
    UNIQUE (workspace_id, id),
    UNIQUE (workspace_id, user_id),
    CHECK ((quiet_start IS NULL) = (quiet_end IS NULL))
);
SELECT secure_workspace_table('notification_preferences');

-- Sessiz saatin bitişi: şimdi sessiz saatteyse bitiş anı, değilse NULL. Başlangıç bitişten büyükse
-- aralık gece yarısını aşar (22:00-07:00).
CREATE FUNCTION notify_quiet_until(p_now timestamptz, p_tz text, p_start time, p_end time)
    RETURNS timestamptz
    LANGUAGE plpgsql STABLE
    SET search_path = pg_catalog, pg_temp
    AS $$
DECLARE
    local_ts timestamp := p_now AT TIME ZONE p_tz;
    t time := local_ts::time;
    end_local timestamp;
    inside boolean;
BEGIN
    IF p_start IS NULL OR p_end IS NULL OR p_start = p_end THEN
        RETURN NULL;
    END IF;
    IF p_start < p_end THEN
        inside := t >= p_start AND t < p_end;
    ELSE
        inside := t >= p_start OR t < p_end;
    END IF;
    IF NOT inside THEN
        RETURN NULL;
    END IF;
    end_local := local_ts::date + p_end;
    IF end_local <= local_ts THEN
        end_local := end_local + interval '1 day';
    END IF;
    RETURN end_local AT TIME ZONE p_tz;
END
$$;
REVOKE ALL ON FUNCTION notify_quiet_until(timestamptz, text, time, time) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION notify_quiet_until(timestamptz, text, time, time) TO app_api, app_worker;

-- Worker her dakika çalışma alanları arası bekleyen işi sorar: teslim edilecek bildirim, uyanacak
-- ertelenmiş bildirim, tetiklenecek hatırlatıcı, gönderilecek günlük özet. Yalnız kimlik döner
-- (müşteri içeriği yok). Satırı sonra çalışma alanı bağlamında işler.
CREATE POLICY owner_read ON notifications FOR SELECT TO app_owner USING (true);
CREATE POLICY owner_read ON reminders FOR SELECT TO app_owner USING (true);
CREATE POLICY owner_read ON notification_preferences FOR SELECT TO app_owner USING (true);

CREATE FUNCTION notify_due_work(p_now timestamptz, p_limit integer DEFAULT 100)
    RETURNS TABLE (kind text, workspace_id uuid, id uuid)
    LANGUAGE sql STABLE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
    SELECT * FROM (
        (SELECT 'deliver'::text, n.workspace_id, n.id FROM notifications n
         WHERE n.status = 'open' AND n.delivered_at IS NULL AND n.deleted_at IS NULL
           AND n.deliver_after <= p_now
         ORDER BY n.deliver_after LIMIT p_limit)
        UNION ALL
        (SELECT 'wake'::text, n.workspace_id, n.id FROM notifications n
         WHERE n.status = 'snoozed' AND n.snoozed_until <= p_now AND n.deleted_at IS NULL
         ORDER BY n.snoozed_until LIMIT p_limit)
        UNION ALL
        (SELECT 'reminder'::text, r.workspace_id, r.id FROM reminders r
         WHERE r.status = 'scheduled' AND r.remind_at <= p_now AND r.deleted_at IS NULL
         ORDER BY r.remind_at LIMIT p_limit)
        UNION ALL
        (SELECT 'summary'::text, p.workspace_id, p.id FROM notification_preferences p
         WHERE p.daily_summary AND p.deleted_at IS NULL
           AND (p.last_summary_on IS NULL
                OR p.last_summary_on < (p_now AT TIME ZONE p.timezone)::date)
           AND (p_now AT TIME ZONE p.timezone)::time >= p.summary_time
         LIMIT p_limit)
    ) due LIMIT p_limit
$$;
REVOKE ALL ON FUNCTION notify_due_work(timestamptz, integer) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION notify_due_work(timestamptz, integer) TO app_worker;

RESET ROLE;
