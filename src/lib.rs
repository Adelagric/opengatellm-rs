//! Client Rust pour [OpenGateLLM](https://github.com/etalab-ia/OpenGateLLM),
//! l'API gateway open-source pour LLM self-hosted (DINUM / Etalab).
//!
//! # Statut
//!
//! Crate en cours de construction (v0.1 minimal : chat, embeddings, models).
//! Compatible OGL `>=0.4.5 <0.5.0`.
//!
//! # Exemple
//!
//! ```no_run
//! use opengatellm::Client;
//!
//! # async fn run() -> Result<(), opengatellm::Error> {
//! let client = Client::new("https://albert.api.etalab.gouv.fr", Some("TOKEN"))?;
//! let models = client.models().await?;
//! for m in &models.data {
//!     println!("{} ({:?})", m.id, m.kind);
//! }
//! # Ok(())
//! # }
//! ```
#![doc(html_root_url = "https://docs.rs/opengatellm/0.1.0")]

pub mod chat;
pub mod client;
pub mod common;
pub mod embeddings;
pub mod error;
pub mod models;
pub mod stream;

pub use chat::{
    Annotation, AnnotationUrlCitation, ChatCompletion, ChatContent, ChatMessage, Choice,
    ChoiceLogprobs, ContentPart, CreateChatCompletion, CustomDetails, FinishReason,
    FunctionDetails, ImageUrl, Role, ServiceTier, Stop, TokenLogprob, ToolCall, TopLogprob,
};
pub use client::{Client, ClientBuilder};
pub use common::{CarbonFootprintRange, CarbonFootprintUsage, EnvironmentalImpacts, Usage};
pub use embeddings::{Embedding, Embeddings, EmbeddingsInput, EmbeddingsRequest, EncodingFormat};
pub use error::Error;
pub use models::{Model, ModelCosts, ModelType, ModelsResponse};
pub use stream::{
    ChatCompletionChunk, ChoiceDelta, ChoiceDeltaToolCall, ChoiceDeltaToolCallFunction, ChunkChoice,
};
