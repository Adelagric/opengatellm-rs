//! Endpoint `POST /v1/search` (recherche RAG).

use crate::chunks::Chunk;
use crate::client::Client;
use crate::common::Usage;
use crate::error::Error;
use serde::{Deserialize, Serialize};

/// Méthode de recherche.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum SearchMethod {
    /// Fusion sémantique + lexicale (RRF).
    Hybrid,
    /// Recherche vectorielle (défaut).
    #[default]
    Semantic,
    /// Recherche lexicale (BM25).
    Lexical,
}

/// Corps de `POST /v1/search`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CreateSearch {
    /// Requête en langage naturel.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    /// Restreint la recherche à ces collections.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub collection_ids: Vec<i64>,
    /// Restreint la recherche à ces documents.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub document_ids: Vec<i64>,
    /// Filtres de métadonnées (DSL `ComparisonFilter`|`CompoundFilter` ; laissé en JSON brut).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata_filters: Option<serde_json::Value>,
    /// Nombre maximal de résultats.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// Décalage de pagination.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
    /// Méthode de recherche.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<SearchMethod>,
    /// Constante k du Reciprocal Rank Fusion (mode hybride).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rff_k: Option<i64>,
    /// Seuil de score minimal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score_threshold: Option<f64>,
}

impl CreateSearch {
    /// Construit une recherche minimale à partir d'une requête.
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: Some(query.into()),
            ..Self::default()
        }
    }

    /// Restreint la recherche à ces collections.
    #[must_use]
    pub fn collections(mut self, ids: impl IntoIterator<Item = i64>) -> Self {
        self.collection_ids = ids.into_iter().collect();
        self
    }

    /// Restreint la recherche à ces documents.
    #[must_use]
    pub fn documents(mut self, ids: impl IntoIterator<Item = i64>) -> Self {
        self.document_ids = ids.into_iter().collect();
        self
    }

    /// Limite le nombre de résultats.
    #[must_use]
    pub fn limit(mut self, limit: i64) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Définit la méthode de recherche.
    #[must_use]
    pub fn method(mut self, method: SearchMethod) -> Self {
        self.method = Some(method);
        self
    }

    /// Définit le seuil de score minimal.
    #[must_use]
    pub fn score_threshold(mut self, threshold: f64) -> Self {
        self.score_threshold = Some(threshold);
        self
    }

    /// Définit les filtres de métadonnées (JSON brut, DSL OGL).
    #[must_use]
    pub fn metadata_filters(mut self, filters: serde_json::Value) -> Self {
        self.metadata_filters = Some(filters);
        self
    }
}

/// Un résultat de recherche : un chunk retrouvé et son score.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Search {
    /// Méthode ayant produit ce résultat.
    pub method: SearchMethod,
    /// Score de pertinence.
    pub score: f64,
    /// Chunk retrouvé.
    pub chunk: Chunk,
}

/// Réponse de `POST /v1/search`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Searches {
    /// Résultats triés par pertinence décroissante.
    pub data: Vec<Search>,
    /// Compteurs d'usage (embeddings de la requête).
    #[serde(default)]
    pub usage: Option<Usage>,
}

impl Client {
    /// `POST /v1/search` — recherche sémantique / lexicale / hybride dans les collections.
    pub async fn search(&self, req: &CreateSearch) -> Result<Searches, Error> {
        let url = self.endpoint("/v1/search")?;
        self.post_json(url, req).await
    }
}
