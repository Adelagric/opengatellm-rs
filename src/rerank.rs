//! Endpoint `POST /v1/rerank`.

use crate::client::Client;
use crate::common::Usage;
use crate::error::Error;
use serde::{Deserialize, Serialize};

/// Corps de requête de `POST /v1/rerank`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRerankBody {
    /// Requête de référence pour le scoring de pertinence.
    pub query: String,
    /// Documents à réordonner par pertinence vis-à-vis de `query`.
    pub documents: Vec<String>,
    /// Identifiant du modèle de reranking (cf. `client.models()`).
    pub model: String,
    /// Nombre maximal de résultats à retourner (les plus pertinents).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_n: Option<i64>,
}

impl CreateRerankBody {
    /// Construit une requête minimale (query + documents + model).
    pub fn new(
        query: impl Into<String>,
        documents: impl IntoIterator<Item = impl Into<String>>,
        model: impl Into<String>,
    ) -> Self {
        Self {
            query: query.into(),
            documents: documents.into_iter().map(Into::into).collect(),
            model: model.into(),
            top_n: None,
        }
    }

    /// Limite le nombre de résultats retournés aux `top_n` plus pertinents.
    #[must_use]
    pub fn top_n(mut self, top_n: i64) -> Self {
        self.top_n = Some(top_n);
        self
    }
}

/// Un résultat de reranking : score de pertinence + index du document d'entrée.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct RerankResult {
    /// Score de pertinence attribué au document (plus élevé = plus pertinent).
    pub relevance_score: f64,
    /// Index du document dans la liste `documents` d'entrée.
    pub index: u32,
}

/// Réponse de `POST /v1/rerank`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct RerankResponse {
    /// Identifiant unique de la requête.
    pub id: String,
    /// Résultats triés par pertinence décroissante.
    pub results: Vec<RerankResult>,
    /// Modèle ayant produit le reranking.
    pub model: String,
    /// Compteurs d'usage (tokens, coût, empreinte environnementale).
    #[serde(default)]
    pub usage: Option<Usage>,
}

impl Client {
    /// `POST /v1/rerank` — réordonne `documents` par pertinence vis-à-vis de `query`.
    pub async fn rerank(&self, req: &CreateRerankBody) -> Result<RerankResponse, Error> {
        let url = self.endpoint("/v1/rerank")?;
        self.post_json(url, req).await
    }
}
