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
.PHONY: help up down logs ps smoke test lint migrate sqlx-prepare kind-up kind-test kind-down vektor

help: ## Hedefleri listeler
	@grep -E '^[a-z-]+:.*## ' $(MAKEFILE_LIST) | awk -F':.*## ' '{printf "  make %-12s %s\n", $$1, $$2}'

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
