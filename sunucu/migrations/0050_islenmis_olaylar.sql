-- T2.15: tüketici çerçevesinin tekrar denetimi.
--
-- Bir tüketici (kalıcı ad) bir outbox olayını yalnız bir kez işler: işleyici işlemi ile
-- aynı işlemde (consumer, event_id) eklenir; ikinci ekleme çakışırsa olay zaten işlenmiştir.
-- NATS en az bir kez teslim eder; bu tablo etkiyi bir kereye indirir. Müşteri içeriği taşımaz,
-- workspace_id yoktur (outbox gibi çalışma alanları arası, hak ile korunur; ADR-0007 madde 2.5).

SET LOCAL ROLE app_owner;

CREATE TABLE processed_events (
    consumer      text NOT NULL CHECK (char_length(consumer) BETWEEN 1 AND 100),
    event_id      uuid NOT NULL,
    processed_at  timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (consumer, event_id)
);
CREATE INDEX processed_events_age_idx ON processed_events (processed_at);

GRANT SELECT, INSERT, DELETE ON processed_events TO app_worker;
RESET ROLE;
