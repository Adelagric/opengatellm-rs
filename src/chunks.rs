//! Endpoints chunks : `GET /v1/chunks/{document}` et `GET /v1/chunks/{document}/{chunk}`.

use crate::client::Client;
use crate::error::Error;
use serde::{Deserialize, Serialize};

/// Un fragment (chunk) indexé d'un document.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Chunk {
    /// Identifiant du chunk.
    pub id: i64,
    /// Collection contenant le chunk.
    pub collection_id: i64,
    /// Document source.
    pub document_id: i64,
    /// Contenu textuel du chunk.
    pub content: String,
    /// Métadonnées libres associées (forme dépendante de l'ingestion).
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
    /// Création (timestamp Unix).
    #[serde(default)]
    pub created: Option<i64>,
}

/// Réponse listant des chunks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Chunks {
    /// Chunks retournés.
    pub data: Vec<Chunk>,
}

impl Client {
    /// `GET /v1/chunks/{document}` — liste les chunks d'un document.
    pub async fn document_chunks(
        &self,
        document_id: impl std::fmt::Display,
    ) -> Result<Chunks, Error> {
        let url = self.endpoint_with_segment("/v1/chunks", &document_id.to_string())?;
        self.get_json(url).await
    }

    /// `GET /v1/chunks/{document}/{chunk}` — récupère un chunk précis.
    pub async fn chunk(
        &self,
        document_id: impl std::fmt::Display,
        chunk_id: impl std::fmt::Display,
    ) -> Result<Chunk, Error> {
        let url = self.endpoint_with_segments(
            "/v1/chunks",
            &[&document_id.to_string(), &chunk_id.to_string()],
        )?;
        self.get_json(url).await
    }
}
