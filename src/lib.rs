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

pub mod client;
pub mod error;
pub mod models;

pub use client::Client;
pub use error::Error;
pub use models::{Model, ModelCosts, ModelType, ModelsResponse};
