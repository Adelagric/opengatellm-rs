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

### Corrigé — revue Tier 1
- `Usages` : items de `data` correctement modélisés (`UsageRecord` avec compteurs imbriqués sous `usage` + `created` requis) ; ajout de `MetricsUsage` ; `UsageDetail.impacts`/`metrics` typés et toujours présents.
- Construction d'URL : `endpoint_with_segment` retire le slash final pour éviter `…//{id}` (corrige `key()`/`delete_key()` et le `model()` pré-existant).
- Tests `wiremock` (`wire_me`) : chemins réels, 204/201, query-string via reqwest, réponse texte `/metrics`.

### Ajouté — Tier 2 (RAG)
- Cœur : `post_multipart` (feature `multipart` de reqwest) + `endpoint_with_segments` (chemins multi-segments).
- `POST /v1/search` (`client.search()`) ; `CreateSearch` (builder), `SearchMethod`, `Search`, `Searches`.
- Collections (CRUD) : `create_collection()` (→ id), `collections()`, `collection()`, `update_collection()` (PATCH 204), `delete_collection()` (204) ; `Collection`, `Collections`, `CollectionRequest`, `CollectionUpdateRequest`, `CollectionVisibility`, `CollectionsQuery`.
- Documents : `create_document()` (multipart → `DocumentResponse`), `documents()`, `document()`, `delete_document()` (204) ; `Document`, `Documents`, `CreateDocument` (builder), `DocumentsQuery`.
- Chunks (lecture) : `document_chunks()`, `chunk()` ; `Chunk`, `Chunks`.
- Tests : 6 round-trips serde + 8 tests `wiremock` (`wire_rag`) — 201 `{id}`, 204, multipart, chemins multi-segments, body search.
