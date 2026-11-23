# Kök Makefile: tek komut girişi (adr/0001-depo-duzeni.md › Tek komut).
# Yeni modüller kendi hedeflerini buraya ekler.

COMPOSE_FILE := altyapi/compose/compose.yaml
COMPOSE      := docker compose -f $(COMPOSE_FILE)

# TLS'i araya giren bir proxy arkasında derlemek için: make up BUILD_CA_FILE=/yol/ca.crt
BUILD_CA := altyapi/compose/.build-ca.pem

# Yerel compose veritabanı (compose.yaml varsayılanlarıyla aynı); testler ve göç bunu kullanır.
DATABASE_URL ?= postgres://$(or $(POSTGRES_USER),demleme):$(or $(POSTGRES_PASSWORD),demleme-yerel)@127.0.0.1:$(or $(POSTGRES_PORT),5432)/$(or $(POSTGRES_DB),demleme)
export DATABASE_URL

.DEFAULT_GOAL := help
.PHONY: help up down logs ps smoke test e2e lint migrate sqlx-prepare kind-up kind-test kind-down vektor

help: ## Hedefleri listeler
	@grep -E '^[a-z-]+:.*## ' $(MAKEFILE_LIST) | awk -F':.*## ' '{printf "  make %-12s %s\\n", $$1, $$2}'

up: ## Derler, tüm servisleri başlatır, sağlık denetimini bekler
	@if [ -n "$(BUILD_CA_FILE)" ]; then cp "$(BUILD_CA_FILE)" $(BUILD_CA); else : > $(BUILD_CA); fi
	$(COMPOSE) up -d --build --wait --wait-timeout 600

down: ## Servisleri durdurur ve kaldırır (veri hacimleri kalır; silmek için: make down ARGS=-v)
	$(COMPOSE) down --remove-orphans $(ARGS)

logs: ## Günlükleri izler (make logs S=api ile tek servis)
	$(COMPOSE) logs -f --tail=100 $(S)

ps: ## Servis durumunu gösterir
	$(COMPOSE) ps

smoke: ## Her servisin sağlığını denetler (önce make up)
	@altyapi/compose/smoke.sh

test: ## Sunucu testleri: cargo test --workspace (PostgreSQL gerekir: make up)
	cd sunucu && cargo test --workspace

E2E_KEYED_API_PORT ?= 18081
E2E_BASE_URL ?= http://127.0.0.1:$(E2E_KEYED_API_PORT)
E2E_NO_KEY_BASE_URL ?= http://127.0.0.1:8080
E2E_FAKE_OIDC_URL ?= http://127.0.0.1:9000
E2E_MAILPIT_URL ?= http://127.0.0.1:8025

e2e: ## Compose uçtan uca testleri (önce make up)
	@set -eu; \
	api_id=$($(COMPOSE) run --no-deps -d \
		-p 127.0.0.1:$(E2E_KEYED_API_PORT):8080 \
		-e BREW_PACKAGE_KEY=gristora-e2e-only-signing-key-not-production \
		-e BREW_PACKAGE_KEY_ID=e2e \
		api); \
	cleanup() { docker rm -f "$$api_id" >/dev/null 2>&1 || true; }; \
	trap cleanup EXIT; \
	for attempt in $$(seq 1 60); do \
		if curl -fsS "$(E2E_BASE_URL)/healthz" >/dev/null; then break; fi; \
		sleep 1; \
	done; \
	if ! curl -fsS "$(E2E_BASE_URL)/healthz" >/dev/null; then \
		echo "E2E keyed API did not become healthy; container state and logs follow:" >&2; \
		docker inspect "$$api_id" --format 'state={{.State.Status}} exit={{.State.ExitCode}} error={{.State.Error}}' >&2 || true; \
		docker logs "$$api_id" >&2 || true; \
		exit 1; \
	fi; \
	cd sunucu && E2E_BASE_URL="$(E2E_BASE_URL)" E2E_NO_KEY_BASE_URL="$(E2E_NO_KEY_BASE_URL)" E2E_FAKE_OIDC_URL="$(E2E_FAKE_OIDC_URL)" E2E_MAILPIT_URL="$(E2E_MAILPIT_URL)" cargo test -p e2e -- --test-threads=1

vektor: ## Parite vektörlerini Python referansından üretir (ADR-0003)
	python3 -I formuller/testler/parite/uret.py

lint: ## Sunucu biçim ve clippy denetimi
	cd sunucu && cargo fmt --all -- --check
	cd sunucu && cargo clippy --workspace --all-targets -- -D warnings

migrate: ## Yerel compose veritabanına göçleri uygular (önce make up ya da yalnız postgres)
	cd sunucu && cargo run -q -p db --bin migrate

sqlx-prepare: ## Sorgu önbelleğini üretir: sunucu/.sqlx (commit edilir; PostgreSQL ve göç gerekir)
	cd sunucu && cargo sqlx prepare --workspace -- --all-targets

kind-up: ## kind kümesi kurar, imajları yükler, altyapi/k8s/overlays/yerel'i uygular
	altyapi/k8s/kind-up.sh

kind-test: ## kind duman testi: podlar, /healthz, gRPC sağlığı, ağ politikası
	altyapi/k8s/kind-test.sh

kind-down: ## kind kümesini siler
	altyapi/k8s/kind-down.sh

.PHONY: gen
gen: ## Sözleşme ve istemci kodunu üretir (OpenAPI, TS tipleri, alan/birim/metin kataloğu)
	bash sozlesme/scripts/gen.sh
