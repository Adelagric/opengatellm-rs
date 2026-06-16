//! Endpoints `/v1/documents` (ingestion et gestion des documents RAG).

use crate::client::Client;
use crate::error::Error;
use serde::{Deserialize, Serialize};

/// Un document ingéré dans une collection.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Document {
    /// Identifiant.
    pub id: i64,
    /// Nom.
    pub name: String,
    /// Collection de rattachement.
    pub collection_id: i64,
    /// Création (timestamp Unix).
    pub created: i64,
    /// Nombre de chunks produits.
    #[serde(default)]
    pub chunks: Option<i64>,
    /// Taille (octets).
    #[serde(default)]
    pub size: Option<i64>,
}

/// Réponse de `GET /v1/documents`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Documents {
    /// Documents.
    pub data: Vec<Document>,
}

/// Réponse de `POST /v1/documents` : identifiant du document créé.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DocumentResponse {
    /// Identifiant du document créé.
    pub id: i64,
}

/// Options d'ingestion d'un document (`POST /v1/documents`, multipart).
#[derive(Debug, Clone, Default)]
pub struct CreateDocument {
    /// Collection cible.
    pub collection_id: Option<i64>,
    /// Nom du document (défaut : nom de fichier).
    pub name: Option<String>,
    /// Désactive le découpage en chunks.
    pub disable_chunking: Option<bool>,
    /// Taille cible des chunks.
    pub chunk_size: Option<i64>,
    /// Chevauchement entre chunks consécutifs.
    pub chunk_overlap: Option<i64>,
}

impl CreateDocument {
    /// Options par défaut ciblant une collection donnée.
    #[must_use]
    pub fn new(collection_id: i64) -> Self {
        Self {
            collection_id: Some(collection_id),
            ..Self::default()
        }
    }

    /// Définit le nom du document.
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Désactive le découpage en chunks.
    #[must_use]
    pub fn disable_chunking(mut self, disable: bool) -> Self {
        self.disable_chunking = Some(disable);
        self
    }

    /// Définit la taille cible des chunks.
    #[must_use]
    pub fn chunk_size(mut self, size: i64) -> Self {
        self.chunk_size = Some(size);
        self
    }

    /// Définit le chevauchement entre chunks.
    #[must_use]
    pub fn chunk_overlap(mut self, overlap: i64) -> Self {
        self.chunk_overlap = Some(overlap);
        self
    }
}

/// Paramètres de `GET /v1/documents`.
#[derive(Debug, Clone, Default, Serialize)]
pub struct DocumentsQuery {
    /// Filtre par collection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_id: Option<i64>,
    /// Décalage de pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
    /// Taille de page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

impl DocumentsQuery {
    /// Construit des paramètres vides.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Filtre par collection.
    #[must_use]
    pub fn collection(mut self, collection_id: i64) -> Self {
        self.collection_id = Some(collection_id);
        self
    }

    /// Définit le décalage de pagination.
    #[must_use]
    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Définit la taille de page.
    #[must_use]
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
}

impl Client {
    /// `POST /v1/documents` — ingère un fichier (multipart) et renvoie l'id du document.
    pub async fn create_document(
        &self,
        file_name: impl Into<String>,
        file: Vec<u8>,
        opts: &CreateDocument,
    ) -> Result<DocumentResponse, Error> {
        let url = self.endpoint("/v1/documents")?;
        let mut form = reqwest::multipart::Form::new().part(
            "file",
            reqwest::multipart::Part::bytes(file).file_name(file_name.into()),
        );
        if let Some(collection_id) = opts.collection_id {
            form = form.text("collection_id", collection_id.to_string());
        }
        if let Some(name) = &opts.name {
            form = form.text("name", name.clone());
        }
        if let Some(disable) = opts.disable_chunking {
            form = form.text("disable_chunking", disable.to_string());
        }
        if let Some(size) = opts.chunk_size {
            form = form.text("chunk_size", size.to_string());
        }
        if let Some(overlap) = opts.chunk_overlap {
            form = form.text("chunk_overlap", overlap.to_string());
        }
        self.post_multipart(url, form).await
    }

    /// `GET /v1/documents` — liste les documents.
    pub async fn documents(&self, query: &DocumentsQuery) -> Result<Documents, Error> {
        let url = self.endpoint("/v1/documents")?;
        self.get_json_with_query(url, query).await
    }

    /// `GET /v1/documents/{id}` — récupère un document par identifiant.
    pub async fn document(&self, document_id: impl std::fmt::Display) -> Result<Document, Error> {
        let url = self.endpoint_with_segment("/v1/documents", &document_id.to_string())?;
        self.get_json(url).await
    }

    /// `DELETE /v1/documents/{id}` — supprime un document (renvoie `204`).
    pub async fn delete_document(&self, document_id: impl std::fmt::Display) -> Result<(), Error> {
        let url = self.endpoint_with_segment("/v1/documents", &document_id.to_string())?;
        self.delete_no_content(url).await
    }
}
