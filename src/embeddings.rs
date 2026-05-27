//! Endpoint `POST /v1/embeddings`.

use crate::client::Client;
use crate::common::Usage;
use crate::error::Error;
use serde::{Deserialize, Serialize};

/// Format d'encodage des vecteurs retournés.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum EncodingFormat {
    /// Vecteurs sous forme de tableau de `f32` (défaut).
    #[default]
    Float,
    /// Vecteurs encodés en base64 (compact).
    Base64,
}

/// Entrée d'une requête embeddings.
///
/// Quatre formes acceptées par OGL (compat `OpenAI`) : texte simple, batch de textes,
/// séquence de tokens pré-encodés, ou batch de séquences de tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EmbeddingsInput {
    /// Un texte unique à embedder.
    Text(String),
    /// Plusieurs textes en une seule requête.
    TextBatch(Vec<String>),
    /// Une séquence de tokens déjà encodés.
    Tokens(Vec<i64>),
    /// Plusieurs séquences de tokens.
    TokensBatch(Vec<Vec<i64>>),
}

impl From<String> for EmbeddingsInput {
    fn from(s: String) -> Self {
        Self::Text(s)
    }
}

impl From<&str> for EmbeddingsInput {
    fn from(s: &str) -> Self {
        Self::Text(s.to_owned())
    }
}

impl From<Vec<String>> for EmbeddingsInput {
    fn from(v: Vec<String>) -> Self {
        Self::TextBatch(v)
    }
}

/// Corps de requête de `POST /v1/embeddings`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingsRequest {
    /// Texte(s) ou tokens à embedder.
    pub input: EmbeddingsInput,
    /// Identifiant du modèle d'embedding (cf. `client.models()`).
    pub model: String,
    /// Nombre de dimensions souhaitées (si le modèle le supporte).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<u32>,
    /// Format d'encodage.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding_format: Option<EncodingFormat>,
}

impl EmbeddingsRequest {
    /// Construit une requête minimale (input + model).
    pub fn new(input: impl Into<EmbeddingsInput>, model: impl Into<String>) -> Self {
        Self {
            input: input.into(),
            model: model.into(),
            dimensions: None,
            encoding_format: None,
        }
    }

    /// Définit le nombre de dimensions souhaitées (Matryoshka-style).
    #[must_use]
    pub fn dimensions(mut self, dimensions: u32) -> Self {
        self.dimensions = Some(dimensions);
        self
    }

    /// Définit le format d'encodage.
    #[must_use]
    pub fn encoding_format(mut self, format: EncodingFormat) -> Self {
        self.encoding_format = Some(format);
        self
    }
}

/// Un vecteur d'embedding renvoyé par le gateway.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Embedding {
    /// Le vecteur lui-même (format `float`). En mode `base64`, la chaîne décodée
    /// retombe ici aussi (les implémentations OpenAI-compat servent typiquement
    /// du float ; base64 reste optionnel côté OGL).
    pub embedding: Vec<f64>,
    /// Index dans le batch d'entrée.
    pub index: u32,
}

/// Réponse de `POST /v1/embeddings`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Embeddings {
    /// Liste des vecteurs (un par entrée).
    pub data: Vec<Embedding>,
    /// Identifiant du modèle ayant produit les vecteurs.
    pub model: String,
    /// Identifiant unique de la requête (si fourni par OGL).
    #[serde(default)]
    pub id: Option<String>,
    /// Compteurs d'usage (tokens, coût, empreinte environnementale).
    #[serde(default)]
    pub usage: Option<Usage>,
}

impl Client {
    /// `POST /v1/embeddings` — calcule les vecteurs d'embedding d'une ou plusieurs entrées.
    pub async fn embeddings(&self, req: &EmbeddingsRequest) -> Result<Embeddings, Error> {
        let url = self.endpoint("/v1/embeddings")?;
        self.post_json(url, req).await
    }
}
