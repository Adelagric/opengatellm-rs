# opengatellm

> Client Rust pour [OpenGateLLM](https://github.com/etalab-ia/OpenGateLLM), l'API gateway open-source pour LLM self-hosted (DINUM / Etalab).

> [!WARNING]
> **Statut : experimental.** Le crate suit la version OGL `>=0.4.5 <0.5.0`. L'API OGL elle-même est en beta et peut introduire des breaking changes. Ce crate ne fait **pas** de retry/backoff automatique — c'est à l'appelant de gérer.

## Scope v0.1

- `POST /v1/chat/completions` (streaming + non-streaming)
- `POST /v1/embeddings`
- `GET  /v1/models` + `GET /v1/models/{model}`

Les endpoints `admin/*`, `files/*`, `collections/*`, `rerank`, `audio/*`, `ocr` sont **hors scope** v0.1.

## Installation

```toml
[dependencies]
opengatellm = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## Quickstart

```rust,no_run
use opengatellm::Client;

# async fn run() -> Result<(), opengatellm::Error> {
let client = Client::new("https://albert.api.etalab.gouv.fr", Some("YOUR_TOKEN"))?;
let models = client.models().await?;
for m in &models.data {
    println!("{}", m.id);
}
# Ok(())
# }
```

Voir [`examples/`](examples) pour des cas plus complets (chat streaming, embeddings).

## Tests d'intégration locaux

Le repo fournit un stack OGL reproductible (`compose.test.yml`) pointant vers Ollama natif local pour le provider LLM. Pré-requis : Docker (ou Colima), Ollama installé.

```bash
ollama pull qwen3-coder:30b
ollama pull nomic-embed-text
make test-integration-up      # démarre OGL + Postgres + Redis + Elasticsearch
make test-integration-wait    # attend que GET /v1/models réponde 200
make test-integration         # lance cargo test --tests -- --ignored
make test-integration-down    # arrête le stack
```

> [!NOTE]
> **Colima** : `host.docker.internal` n'est pas toujours résolu par défaut dans la VM Colima. Si le provider Ollama n'est pas joignable depuis le conteneur OGL, configurer Colima avec `colima start --network-address` ou substituer l'IP réseau interne accessible depuis la VM dans `config.test.yml`.

## Référence

La spec OpenAPI 3.1.0 de la version d'OGL ciblée est commitée dans [`spec/openapi-0.4.5.json`](spec/openapi-0.4.5.json). Toute évolution du client doit rester cohérente avec cette référence jusqu'au prochain bump.

## Licence

Double licence MIT OR Apache-2.0, au choix de l'utilisateur. Voir [`LICENSE-MIT`](LICENSE-MIT) et [`LICENSE-APACHE`](LICENSE-APACHE).
