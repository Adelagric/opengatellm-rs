//! Endpoints `/v1/collections` (CRUD des collections RAG).

use crate::client::Client;
use crate::error::Error;
use serde::{Deserialize, Serialize};

/// Visibilité d'une collection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum CollectionVisibility {
    /// Visible du seul propriétaire.
    Private,
    /// Publique.
    Public,
}

/// Une collection de documents.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Collection {
    /// Identifiant.
    pub id: i64,
    /// Nom.
    pub name: String,
    /// Propriétaire.
    pub owner: String,
    /// Description.
    #[serde(default)]
    pub description: Option<String>,
    /// Visibilité.
    #[serde(default)]
    pub visibility: Option<CollectionVisibility>,
    /// Création (timestamp Unix).
    pub created: i64,
    /// Dernière mise à jour (timestamp Unix).
    pub updated: i64,
    /// Nombre de documents.
    #[serde(default)]
    pub documents: Option<i64>,
    /// Taille cumulée (octets).
    #[serde(default)]
    pub size: Option<i64>,
}

/// Réponse de `GET /v1/collections`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Collections {
    /// Collections.
    pub data: Vec<Collection>,
}

/// Corps de `POST /v1/collections`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionRequest {
    /// Nom de la collection.
    pub name: String,
    /// Description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Visibilité.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<CollectionVisibility>,
}

impl CollectionRequest {
    /// Construit une demande de collection nommée.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            visibility: None,
        }
    }

    /// Définit la description.
    #[must_use]
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Définit la visibilité.
    #[must_use]
    pub fn visibility(mut self, visibility: CollectionVisibility) -> Self {
        self.visibility = Some(visibility);
        self
    }
}

/// Corps de `PATCH /v1/collections/{id}` — champs à mettre à jour (tous optionnels).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CollectionUpdateRequest {
    /// Nouveau nom.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Nouvelle description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Nouvelle visibilité.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<CollectionVisibility>,
}

impl CollectionUpdateRequest {
    /// Construit un patch vide (aucun champ modifié).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Définit le nouveau nom.
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Définit la nouvelle description.
    #[must_use]
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Définit la nouvelle visibilité.
    #[must_use]
    pub fn visibility(mut self, visibility: CollectionVisibility) -> Self {
        self.visibility = Some(visibility);
        self
    }
}

/// Paramètres de pagination pour `GET /v1/collections`.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CollectionsQuery {
    /// Décalage de pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
    /// Taille de page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

impl CollectionsQuery {
    /// Construit des paramètres vides.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
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

// Réponse interne de création : le serveur renvoie `{"id": N}`.
#[derive(Deserialize)]
struct CreatedId {
    id: i64,
}

impl Client {
    /// `POST /v1/collections` — crée une collection et renvoie son identifiant.
    pub async fn create_collection(&self, req: &CollectionRequest) -> Result<i64, Error> {
        let url = self.endpoint("/v1/collections")?;
        let created: CreatedId = self.post_json(url, req).await?;
        Ok(created.id)
    }

    /// `GET /v1/collections` — liste les collections accessibles.
    pub async fn collections(&self, query: &CollectionsQuery) -> Result<Collections, Error> {
        let url = self.endpoint("/v1/collections")?;
        self.get_json_with_query(url, query).await
    }

    /// `GET /v1/collections/{id}` — récupère une collection par identifiant.
    pub async fn collection(
        &self,
        collection_id: impl std::fmt::Display,
    ) -> Result<Collection, Error> {
        let url = self.endpoint_with_segment("/v1/collections", &collection_id.to_string())?;
        self.get_json(url).await
    }

    /// `PATCH /v1/collections/{id}` — met à jour une collection (renvoie `204`).
    pub async fn update_collection(
        &self,
        collection_id: impl std::fmt::Display,
        update: &CollectionUpdateRequest,
    ) -> Result<(), Error> {
        let url = self.endpoint_with_segment("/v1/collections", &collection_id.to_string())?;
        self.patch_no_content(url, update).await
    }

    /// `DELETE /v1/collections/{id}` — supprime une collection (renvoie `204`).
    pub async fn delete_collection(
        &self,
        collection_id: impl std::fmt::Display,
    ) -> Result<(), Error> {
        let url = self.endpoint_with_segment("/v1/collections", &collection_id.to_string())?;
        self.delete_no_content(url).await
    }
}
