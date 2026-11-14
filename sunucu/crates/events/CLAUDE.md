# events

## What it does
- Event pipeline (ADR-0007 item 2.5, ADR-0008 item 9). `db::outbox::enqueue` writes outbox rows in a transaction. `relay` sends them to NATS JetStream (stream `EVENTS`, retention 1 hour). The `consumer` framework uses durable pull consumers to process an event effectively once. The API gives SSE from the same stream.
- Subject: `ev.<çalışma alanı|_>.<varlık türü|_>.<varlık kimliği|_>.<olay türü>`. Dead letter: stream `EVENTS_DLQ`, `dlq.<tüketici>.<konu>`.
- Relay: `FOR UPDATE SKIP LOCKED` plus an advisory lock for a single relay. It publishes in order. `Nats-Msg-Id` is the outbox id. Only the acknowledged prefix gets `published_at` (at least once; JetStream drops a repeat within 2 minutes).
- Consumer: it inserts `processed_events (consumer, event_id)` in the transaction. If the insert conflicts, it skips the event. It tries `Retry` again with backoff. After `max_attempts`, or with `Permanent`, the event goes to the dead letter.

## What it does not touch
- It does not carry customer content. An event has only an id, a version and a small `data`.
- HTTP and authorization are not here. The SSE endpoint is in `api/src/events.rs`.

## How to test
- You need PostgreSQL and NATS: `docker compose -f altyapi/compose/compose.yaml up -d --wait postgres nats`. Then run `DATABASE_URL=... NATS_URL=nats://127.0.0.1:4222 cargo test -p events`.
- The restart test starts its own `nats:2-alpine` container (it skips if Docker is not available). It does not stop the compose NATS.

## Rules
- Code is in English. Notes for Claude (this file, task lists) use ASD-STE100. Other documents are in Turkish.
- Start the commit message with the task number: `T2.x: ...`.
