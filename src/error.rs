//! Erreurs renvoyées par le client.

use thiserror::Error;

/// Erreurs possibles d'un appel au gateway `OpenGateLLM`.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// Erreur de transport HTTP (réseau, TLS, timeout, etc.).
    #[error("HTTP transport error: {0}")]
    Transport(#[from] reqwest::Error),

    /// Réponse non-2xx du gateway. `detail` contient le corps brut.
    #[error("API error {status}: {detail}")]
    Api {
        /// Code HTTP retourné.
        status: u16,
        /// Corps brut de la réponse d'erreur (souvent JSON).
        detail: String,
    },

    /// Échec de désérialisation d'une réponse.
    #[error("JSON decode error: {0}")]
    Decode(#[from] serde_json::Error),

    /// URL invalide passée au client ou produite à partir d'une URL de base invalide.
    #[error("invalid URL: {0}")]
    InvalidUrl(String),

    /// Erreur de parsing d'un flux SSE (streaming chat).
    #[error("stream error: {0}")]
    Stream(String),
}

impl From<url::ParseError> for Error {
    fn from(e: url::ParseError) -> Self {
        Self::InvalidUrl(e.to_string())
    }
}
