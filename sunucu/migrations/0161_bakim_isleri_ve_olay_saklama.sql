-- T2.39: worker bakım işleri. maintenance_runs: işin son çalışması; birden çok worker kopyası
-- işi `INSERT ... ON CONFLICT DO UPDATE ... WHERE` ile sahiplenir, iş aralıkta bir kez çalışır.
-- Çalışma alanları arası tablo: RLS yok, hakla korunur (adr/0007 › 2.5).
-- external_events.payload_cleared_at (F2.1 karar 11): işlenmeyen olayın yükü 30 gün sonra
-- boşaltılır; sütun bunu işaretler ve işi tekrar çalıştırmayı zararsız yapar.
SET LOCAL ROLE app_owner;

CREATE TABLE maintenance_runs (
    job              text PRIMARY KEY CHECK (char_length(job) BETWEEN 1 AND 100),
    last_started_at  timestamptz NOT NULL,
    last_finished_at timestamptz,
    last_status      text NOT NULL DEFAULT 'running'
                     CHECK (last_status IN ('running', 'ok', 'findings', 'error')),
    last_detail      text
);
GRANT SELECT, INSERT, UPDATE ON maintenance_runs TO app_worker;

ALTER TABLE external_events ADD COLUMN payload_cleared_at timestamptz;

RESET ROLE;
