# events

## Ne yapar
- Olay hattı (ADR-0007 madde 2.5, ADR-0008 madde 9): `db::outbox::enqueue` ile işlemde yazılan outbox satırları `relay` ile NATS JetStream'e (`EVENTS` akışı, 1 saat tutma) aktarılır; `consumer` çerçevesi kalıcı çekme tüketicileriyle olayı bir kez etkili işler; API aynı akıştan SSE sunar.
- Konu: `ev.<çalışma alanı|_>.<varlık türü|_>.<varlık kimliği|_>.<olay türü>`; ölü mektup: `EVENTS_DLQ` akışı, `dlq.<tüketici>.<konu>`.
- Aktarıcı: `FOR UPDATE SKIP LOCKED` + tek aktarıcı için danışma kilidi, sıralı yayın, `Nats-Msg-Id` = outbox kimliği, yalnız onaylanan önek `published_at` alır (en az bir kez; JetStream 2 dk içinde tekrarı atar).
- Tüketici: işlem içinde `processed_events (consumer, event_id)` eklenir; çakışırsa olay atlanır. `Retry` geri çekilmeyle yeniden denenir, `max_attempts` sonunda ya da `Permanent` ile ölü mektuba gider.

## Neye dokunmaz
- Müşteri içeriği taşımaz (olay yalnız kimlik, sürüm, küçük `data`).
- HTTP ve yetki burada değil: SSE ucu `api/src/events.rs`.

## Nasıl test edilir
- PostgreSQL ve NATS gerekir: `docker compose -f altyapi/compose/compose.yaml up -d --wait postgres nats`; `DATABASE_URL=... NATS_URL=nats://127.0.0.1:4222 cargo test -p events`.
- Yeniden başlatma testi kendi `nats:2-alpine` konteynerini açar (Docker yoksa atlanır); compose NATS'ı durdurmaz.

## Kurallar
- Kod İngilizce; belgeler Türkçe.
- Commit mesajı iş numarasıyla başlar: `T2.x: ...`.
