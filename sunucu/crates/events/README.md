# events

Outbox -> NATS JetStream aktarıcısı, tüketici çerçevesi ve ortak akış/konu adları (T2.15). Ayrıntı ve komutlar: `CLAUDE.md`.

| Parça | Ne yapar |
|---|---|
| `connect`, `ensure_streams` | NATS bağlantısı (ilk bağlantıyı ve kopmayı yeniden dener), `EVENTS` ve `EVENTS_DLQ` akışlarını kurar |
| `subject`, `workspace_filter`, `envelope` | Konu adı ve SSE `data` gövdesi (içeriksiz bildirim) |
| `relay::run`, `relay::run_once` | Worker'da outbox aktarıcısı, eski yayınlanmış satırları siler |
| `consumer::run`, `Handler`, `HandlerError` | Bir kez etkili tüketici: tekrar denetimi, geri çekilme, ölü mektup |

Yazma tarafı `db::outbox::enqueue`'dadır. Tüketici örneği: `tests/events.rs`.
