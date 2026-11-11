# Kök Makefile: tek komut girişi (adr/0001-depo-duzeni.md › Tek komut).
# Yeni modüller kendi hedeflerini buraya ekler.

COMPOSE_FILE := altyapi/compose/compose.yaml
COMPOSE      := docker compose -f $(COMPOSE_FILE)

# TLS'i araya giren bir proxy arkasında derlemek için: make up BUILD_CA_FILE=/yol/ca.crt
BUILD_CA := altyapi/compose/.build-ca.pem

.DEFAULT_GOAL := help
.PHONY: help up down logs ps smoke test lint

help: ## Hedefleri listeler
	@grep -E '^[a-z-]+:.*## ' $(MAKEFILE_LIST) | awk -F':.*## ' '{printf "  make %-8s %s\n", $$1, $$2}'

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

test: ## Sunucu testleri: cargo test --workspace
	cd sunucu && cargo test --workspace

lint: ## Sunucu biçim ve clippy denetimi
	cd sunucu && cargo fmt --all -- --check
	cd sunucu && cargo clippy --workspace --all-targets -- -D warnings
