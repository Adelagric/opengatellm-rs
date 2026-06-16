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
- `ClientBuilder` (timeout + api_key configurables) ; `Client::builder()` et `Client::new()` cohérents (`impl Into<String>`).
- Endpoint `POST /v1/embeddings` (`client.embeddings()`).
- Types `EmbeddingsRequest` (builder fluent), `EmbeddingsInput` (untagged : Text / TextBatch / Tokens / TokensBatch), `Embeddings`, `Embedding`, `EncodingFormat`.
- Module `common` : `Usage`, `EnvironmentalImpacts`, `CarbonFootprintUsage`, `CarbonFootprintRange`.
- Endpoint `POST /v1/chat/completions` non-streaming (`client.chat_completion()` force `stream=false`).
- Types chat : `CreateChatCompletion` (builder), `ChatMessage` (raccourcis `user/system/assistant`), `ChatContent` (untagged texte|parts), `ContentPart` (texte + image_url multimodal), `ImageUrl`, `ToolCall` (Function|Custom), `FunctionDetails`, `CustomDetails`, `Annotation`, `Choice`, `ChatCompletion`, `FinishReason`, `Stop`, `Role`, `ServiceTier`, logprobs.
- Module `stream` : streaming SSE chat via `client.chat_completion_stream()` (force `stream=true`, retourne `impl Stream<Item=Result<ChatCompletionChunk>>`). Sentinelle `[DONE]` détectée et le stream se termine proprement.
- Types stream : `ChatCompletionChunk`, `ChunkChoice`, `ChoiceDelta`, `ChoiceDeltaToolCall`, `ChoiceDeltaToolCallFunction`.
- Variante `Error::Stream(String)` pour les erreurs de parsing SSE.
- Examples runnable : `chat_completion`, `chat_streaming`, `embeddings` (env `OGL_URL`, `OGL_TOKEN`, `OGL_CHAT_MODEL`, `OGL_EMBED_MODEL`).
- README quickstart enrichi (4 snippets : models, chat, streaming, embeddings).

### Ajouté — Tier 1 (vers v0.2)
- Cœur client : `get_json_with_query` (params de requête / pagination), `get_text` (réponses non-JSON), `patch_no_content` (PATCH → 204), `delete_no_content` (DELETE → 204).
- Endpoint `POST /v1/rerank` (`client.rerank()`) ; types `CreateRerankBody` (builder + `top_n`), `RerankResult`, `RerankResponse`.
- Monitoring : `GET /health` (`client.health()`), `GET /health/models` (`client.health_models()`), `GET /metrics` (`client.metrics()` → texte Prometheus brut) ; types `Health`, `HealthStatus`, `ModelHealthStatus`, `ModelsHealthResponse`.
- Self-service `me/*` : `me_info()`, `update_me_info()` (PATCH), `create_key()`, `keys()`, `key()`, `delete_key()`, `usage()` ; types `UserInfo`, `UpdateUserInfo` (builder), `PermissionType`, `Limit`/`LimitType`, `CreateKey`/`CreateKeyResponse`, `Key`/`Keys`, `UsageDetail`/`Usages`, `EndpointUsage`, et les filtres `KeysQuery`/`UsageQuery` (builders).
- Tests : 16 round-trips serde (rerank, monitoring, me/*) + 4 tests d'intégration gated (`integration_tier1`).
