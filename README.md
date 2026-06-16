# opengatellm

> Client Rust pour [OpenGateLLM](https://github.com/etalab-ia/OpenGateLLM), l'API gateway open-source pour LLM self-hosted (DINUM / Etalab).

> [!WARNING]
> **Statut : experimental.** Le crate suit la version OGL `>=0.4.5 <0.5.0`. L'API OGL elle-même est en beta et peut introduire des breaking changes. Ce crate ne fait **pas** de retry/backoff automatique — c'est à l'appelant de gérer.

## Endpoints couverts

- `POST /v1/chat/completions` (streaming + non-streaming)
- `POST /v1/embeddings`
- `POST /v1/rerank`
- `GET  /v1/models` + `GET /v1/models/{model}`
- Monitoring : `GET /health`, `GET /health/models`, `GET /metrics`
- Self-service `me/*` : `GET`/`PATCH /v1/me/info`, `/v1/me/keys` (liste, création, lecture, révocation), `GET /v1/me/usage`

Encore **hors scope** : `admin/*`, RAG (`collections/*`, `documents/*`, `chunks/*`, `search`), `audio/*`, `ocr`, `parse-beta`, `auth/login`.

## Installation

```toml
[dependencies]
opengatellm = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## Quickstart

### Lister les modèles

```rust,no_run
use opengatellm::Client;

# async fn run() -> Result<(), opengatellm::Error> {
let client = Client::new("https://albert.api.etalab.gouv.fr", Some("YOUR_TOKEN"))?;
let models = client.models().await?;
for m in &models.data {
    println!("{} ({:?})", m.id, m.kind);
}
# Ok(())
# }
```

### Chat (non-streaming)

```rust,no_run
use opengatellm::{Client, ChatMessage, CreateChatCompletion};

# async fn run() -> Result<(), opengatellm::Error> {
let client = Client::new("http://localhost:8000", Some("TOKEN"))?;
let req = CreateChatCompletion::new(
    vec![
        ChatMessage::system("Réponds en français."),
        ChatMessage::user("Quelle est la capitale de la France ?"),
    ],
    "qwen3-coder",
)
.temperature(0.0)
.max_completion_tokens(64);
let resp = client.chat_completion(&req).await?;
println!("{:?}", resp.choices[0].message.content);
# Ok(())
# }
```

### Chat (streaming SSE)

```rust,no_run
use futures_util::StreamExt;
use opengatellm::{Client, ChatMessage, CreateChatCompletion};

# async fn run() -> Result<(), opengatellm::Error> {
let client = Client::new("http://localhost:8000", Some("TOKEN"))?;
let req = CreateChatCompletion::new(
    vec![ChatMessage::user("Raconte-moi une blague.")],
    "qwen3-coder",
);
let mut stream = client.chat_completion_stream(&req).await?;
while let Some(chunk) = stream.next().await {
    let chunk = chunk?;
    if let Some(delta) = chunk.choices.first().and_then(|c| c.delta.content.as_ref()) {
        print!("{delta}");
    }
}
# Ok(())
# }
```

### Embeddings

```rust,no_run
use opengatellm::{Client, EmbeddingsRequest};

# async fn run() -> Result<(), opengatellm::Error> {
let client = Client::new("http://localhost:8000", Some("TOKEN"))?;
let req = EmbeddingsRequest::new("Paris est la capitale de la France.", "nomic-embed-text");
let resp = client.embeddings(&req).await?;
println!("dim={}", resp.data[0].embedding.len());
# Ok(())
# }
```

### Rerank

```rust,no_run
use opengatellm::{Client, CreateRerankBody};

# async fn run() -> Result<(), opengatellm::Error> {
let client = Client::new("http://localhost:8000", Some("TOKEN"))?;
let req = CreateRerankBody::new(
    "aides à l'irrigation",
    ["Prêt à taux zéro agricole", "Subvention matériel d'irrigation"],
    "bge-reranker",
)
.top_n(1);
let resp = client.rerank(&req).await?;
println!("meilleur document = index {}", resp.results[0].index);
# Ok(())
# }
```

### Monitoring & self-service

```rust,no_run
use opengatellm::{Client, UsageQuery};

# async fn run() -> Result<(), opengatellm::Error> {
let client = Client::new("http://localhost:8000", Some("TOKEN"))?;
// Sonde de liveness (non authentifiée côté OGL).
println!("santé : {}", client.health().await?.status);
// Suivi de sa propre consommation de tokens.
let usage = client.usage(&UsageQuery::new().limit(10)).await?;
println!("{} lignes d'usage", usage.data.len());
# Ok(())
# }
```

## Examples runnable

```bash
# Pré-requis : une instance OGL accessible (cf. section ci-dessous pour le stack de test).
export OGL_URL=http://localhost:8000
export OGL_TOKEN=mytoken
export OGL_CHAT_MODEL=qwen3-coder
export OGL_EMBED_MODEL=nomic-embed-text

cargo run --example chat_completion
cargo run --example chat_streaming
cargo run --example embeddings
```

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
