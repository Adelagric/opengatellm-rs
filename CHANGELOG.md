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
