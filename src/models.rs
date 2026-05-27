//! Endpoints `/v1/models` et `/v1/models/{model}`.

use crate::client::Client;
use crate::error::Error;
use serde::{Deserialize, Serialize};

/// Type de modèle exposé par OGL.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum ModelType {
    /// Reconnaissance vocale (speech-to-text).
    AutomaticSpeechRecognition,
    /// Vision + texte → texte.
    ImageTextToText,
    /// Image → texte.
    ImageToText,
    /// Classification de texte.
    TextClassification,
    /// Génération d'embeddings.
    TextEmbeddingsInference,
    /// Génération de texte (chat / completion).
    TextGeneration,
}

/// Coûts par million de tokens (décrémentent le budget utilisateur OGL).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ModelCosts {
    /// Coût des tokens d'entrée par million.
    #[serde(default)]
    pub prompt_tokens: f64,
    /// Coût des tokens de sortie par million.
    #[serde(default)]
    pub completion_tokens: f64,
}

/// Description d'un modèle exposé par le gateway.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Model {
    /// Identifiant du modèle (utilisé dans les requêtes `model: ...`).
    pub id: String,
    /// Timestamp Unix de création.
    pub created: i64,
    /// Organisation propriétaire du modèle.
    pub owned_by: String,
    /// Alias additionnels permettant de référencer le modèle.
    #[serde(default)]
    pub aliases: Vec<String>,
    /// Type fonctionnel du modèle.
    #[serde(default, rename = "type")]
    pub kind: Option<ModelType>,
    /// Longueur maximale de contexte (tokens).
    #[serde(default)]
    pub max_context_length: Option<i64>,
    /// Coûts par million de tokens.
    #[serde(default)]
    pub costs: Option<ModelCosts>,
}

/// Réponse de `GET /v1/models` : enveloppe avec la liste des modèles.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ModelsResponse {
    /// Modèles exposés par le gateway.
    pub data: Vec<Model>,
}

impl Client {
    /// `GET /v1/models` — liste les modèles disponibles sur le gateway.
    pub async fn models(&self) -> Result<ModelsResponse, Error> {
        let url = self.endpoint("/v1/models")?;
        self.get_json(url).await
    }

    /// `GET /v1/models/{model_id}` — récupère un modèle par identifiant.
    pub async fn model(&self, model_id: &str) -> Result<Model, Error> {
        let url = self.endpoint_with_segment("/v1/models/", model_id)?;
        self.get_json(url).await
    }
}
