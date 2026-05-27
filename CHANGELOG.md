# Changelog

Toutes les modifications notables sont consignées ici. Le format suit [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/), et le projet adhère au [versioning sémantique](https://semver.org/lang/fr/).

## [Non publié]

### Ajouté
- Scaffold initial du crate.
- Spec OGL 0.4.5 pinnée dans `spec/openapi-0.4.5.json` (référence canonique).
- `compose.test.yml`, `config.test.yml`, `.env.test` : stack reproductible pour tests d'intégration (OGL + Postgres + Redis + Elasticsearch, pointant vers Ollama natif local).
- `Makefile` : cibles `test-integration-up/down/wait`, `pull-ollama-models`, `clippy`, `fmt`, `doc`.
- `Client::new(base_url, api_key)` + module `error` (`Transport`, `Api`, `Decode`, `InvalidUrl`).
- Endpoint `GET /v1/models` (`client.models()`) et `GET /v1/models/{model}` (`client.model(id)`).
- Types `Model`, `ModelsResponse`, `ModelType`, `ModelCosts` (tous `#[non_exhaustive]`).
