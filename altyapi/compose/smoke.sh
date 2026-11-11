#!/usr/bin/env bash
# Yerel sistem duman testi: her servisin sağlığını denetler. Hata varsa 1 döner.
# Kullanım: kökten `make smoke`.
set -u

COMPOSE_FILE="$(cd "$(dirname "$0")" && pwd)/compose.yaml"
dc() { docker compose -f "$COMPOSE_FILE" "$@"; }

# Portlar compose.yaml ile aynı varsayılanları ve .env geçersiz kılmalarını kullanır.
set -a
# shellcheck disable=SC1091
[ -f "$(dirname "$COMPOSE_FILE")/.env" ] && . "$(dirname "$COMPOSE_FILE")/.env"
set +a

fail=0
ok()  { printf '  ok    %s\n' "$1"; }
bad() { printf '  HATA  %s\n' "$1"; fail=1; }

http_status() { curl -s -o /dev/null -m 5 -w '%{http_code}' "$@"; }

expect() { # ad beklenen gerçek
  if [ "$3" = "$2" ]; then ok "$1"; else bad "$1 (beklenen $2, gelen $3)"; fi
}

echo "HTTP servisleri"
expect "api /healthz"    200 "$(http_status "http://127.0.0.1:${API_PORT:-8080}/healthz")"
expect "worker /healthz" 200 "$(http_status "http://127.0.0.1:${WORKER_PORT:-8081}/healthz")"
expect "ingest /healthz" 200 "$(http_status "http://127.0.0.1:${INGEST_PORT:-8082}/healthz")"

echo "engine-service (gRPC)"
if out=$(dc exec -T engine-service grpc_health_probe -addr=127.0.0.1:50051 2>&1); then
  ok "gRPC sağlık: $out"
else
  bad "gRPC sağlık: $out"
fi

echo "Veri ve mesajlaşma"
if out=$(dc exec -T postgres sh -c 'pg_isready -U "$POSTGRES_USER" -d "$POSTGRES_DB"' 2>&1); then ok "postgres: $out"; else bad "postgres: $out"; fi
if [ "$(dc exec -T redis redis-cli ping 2>&1 | tr -d '\r')" = "PONG" ]; then ok "redis: PONG"; else bad "redis ping"; fi
expect "nats /healthz" 200 "$(http_status "http://127.0.0.1:${NATS_MONITOR_PORT:-8222}/healthz")"
if curl -s -m 5 "http://127.0.0.1:${NATS_MONITOR_PORT:-8222}/jsz" | grep -q '"config"'; then ok "nats JetStream açık"; else bad "nats JetStream kapalı"; fi

echo "Sahte servisler"
expect "mailpit arayüz/API" 200 "$(http_status "http://127.0.0.1:${MAILPIT_UI_PORT:-8025}/api/v1/info")"
expect "github-mock sağlık" 200 "$(http_status "http://127.0.0.1:${GITHUB_MOCK_PORT:-8090}/__admin/health")"
expect "github-mock issue oluştur" 201 "$(http_status -X POST -H 'Content-Type: application/json' -d '{"title":"t"}' "http://127.0.0.1:${GITHUB_MOCK_PORT:-8090}/repos/o/r/issues")"
expect "github-mock yorum ekle"    201 "$(http_status -X POST -H 'Content-Type: application/json' -d '{"body":"b"}' "http://127.0.0.1:${GITHUB_MOCK_PORT:-8090}/repos/o/r/issues/1/comments")"
expect "apns-mock sağlık" 200 "$(http_status "http://127.0.0.1:${APNS_MOCK_PORT:-8091}/__admin/health")"
expect "apns-mock bildirim gönder" 200 "$(http_status -X POST -H 'Content-Type: application/json' -d '{"aps":{"alert":"x"}}' "http://127.0.0.1:${APNS_MOCK_PORT:-8091}/3/device/abcdef0123456789")"

if [ "$fail" -eq 0 ]; then echo "smoke: TAMAM"; else echo "smoke: BAŞARISIZ"; fi
exit "$fail"
