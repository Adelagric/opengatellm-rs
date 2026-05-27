.PHONY: help build test test-unit test-integration test-integration-up test-integration-down test-integration-wait pull-ollama-models clippy fmt fmt-check doc clean

OGL_URL ?= http://localhost:8000
# Utilise `docker-compose` (legacy v5, p.ex. Colima) ou `docker compose` (plugin v2).
# Surcharge avec `make ... DOCKER_COMPOSE='docker compose'` au besoin.
DOCKER_COMPOSE ?= docker-compose
COMPOSE := $(DOCKER_COMPOSE) -f compose.test.yml --env-file .env.test

help: ## Affiche les cibles disponibles
	@awk 'BEGIN{FS=":.*##";printf "Usage: make <cible>\n\nCibles:\n"} /^[a-zA-Z_-]+:.*##/{printf "  \033[36m%-28s\033[0m %s\n",$$1,$$2}' $(MAKEFILE_LIST)

build: ## cargo build
	cargo build

test-unit: ## Tests unitaires uniquement
	cargo test --lib

test-integration: test-integration-up test-integration-wait ## Stack up + tests d'intégration (#[ignore]) lancés
	OGL_TEST_URL=$(OGL_URL) cargo test --tests -- --ignored

test-integration-up: ## Démarre le stack OGL local pour tests (Postgres + Redis + ES + api)
	$(COMPOSE) up -d

test-integration-down: ## Arrête le stack OGL local
	$(COMPOSE) down

test-integration-wait: ## Attend que GET /v1/models réponde 200 (max 120s)
	@echo "Attente de OGL sur $(OGL_URL)/v1/models..."
	@for i in $$(seq 1 60); do \
		if curl -fsSm 2 $(OGL_URL)/v1/models > /dev/null 2>&1; then \
			echo "OGL prêt ($$i*2s)"; exit 0; \
		fi; \
		sleep 2; \
	done; \
	echo "Timeout après 120s"; exit 1

pull-ollama-models: ## Pré-télécharge les modèles Ollama nécessaires aux tests
	ollama pull qwen3-coder:30b
	ollama pull nomic-embed-text

clippy: ## Lint strict
	cargo clippy --all-targets -- -D warnings

fmt: ## Formate le code
	cargo fmt

fmt-check: ## Vérifie le formatage sans modifier
	cargo fmt --check

doc: ## Génère la doc
	cargo doc --no-deps

clean: ## Nettoie
	cargo clean
