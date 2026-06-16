//! Endpoints self-service `/v1/me/*` : informations, clés d'API, usage.

use crate::client::Client;
use crate::common::EnvironmentalImpacts;
use crate::error::Error;
use serde::{Deserialize, Serialize};

/// Permission accordée à un utilisateur / une clé.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum PermissionType {
    /// Administration complète du gateway.
    Admin,
    /// Création de collections publiques.
    CreatePublicCollection,
    /// Lecture des métriques (`/metrics`).
    ReadMetric,
    /// Déclaration de modèles (provider).
    ProvideModels,
}

/// Type de limite de débit appliquée à un router (modèle).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum LimitType {
    /// Tokens par minute.
    Tpm,
    /// Tokens par jour.
    Tpd,
    /// Requêtes par minute.
    Rpm,
    /// Requêtes par jour.
    Rpd,
}

/// Limite de débit sur un router (modèle) donné.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Limit {
    /// Identifiant du router (modèle) concerné.
    pub router_id: i64,
    /// Nature de la limite.
    #[serde(rename = "type")]
    pub kind: LimitType,
    /// Valeur de la limite (`None` = pas de plafond).
    #[serde(default)]
    pub value: Option<i64>,
}

/// Informations sur l'utilisateur courant (`GET /v1/me/info`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct UserInfo {
    /// Identifiant interne.
    pub id: i64,
    /// Adresse e-mail.
    pub email: String,
    /// Nom d'affichage.
    #[serde(default)]
    pub name: Option<String>,
    /// Organisation de rattachement.
    #[serde(default)]
    pub organization: Option<i64>,
    /// Budget restant (unités OGL).
    #[serde(default)]
    pub budget: Option<f64>,
    /// Permissions accordées.
    #[serde(default)]
    pub permissions: Vec<PermissionType>,
    /// Limites de débit appliquées.
    #[serde(default)]
    pub limits: Vec<Limit>,
    /// Expiration du compte (timestamp Unix).
    #[serde(default)]
    pub expires: Option<i64>,
    /// Priorité d'ordonnancement des requêtes.
    #[serde(default)]
    pub priority: i64,
    /// Création du compte (timestamp Unix).
    pub created: i64,
    /// Dernière mise à jour (timestamp Unix).
    pub updated: i64,
}

/// Corps de `PATCH /v1/me/info` — champs à mettre à jour (tous optionnels).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateUserInfo {
    /// Nouveau nom d'affichage.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Nouvelle adresse e-mail.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// Mot de passe actuel (requis pour changer le mot de passe).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_password: Option<String>,
    /// Nouveau mot de passe.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

impl UpdateUserInfo {
    /// Construit un patch vide (aucun champ modifié).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Définit le nouveau nom d'affichage.
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Définit la nouvelle adresse e-mail.
    #[must_use]
    pub fn email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    /// Définit le couple (mot de passe actuel, nouveau mot de passe).
    #[must_use]
    pub fn password(mut self, current: impl Into<String>, new: impl Into<String>) -> Self {
        self.current_password = Some(current.into());
        self.password = Some(new.into());
        self
    }
}

/// Corps de `POST /v1/me/keys` — création d'une clé d'API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateKey {
    /// Nom lisible de la clé.
    pub name: String,
    /// Expiration optionnelle (timestamp Unix).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires: Option<i64>,
}

impl CreateKey {
    /// Construit une demande de clé nommée, sans expiration.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            expires: None,
        }
    }

    /// Définit une expiration (timestamp Unix).
    #[must_use]
    pub fn expires(mut self, expires: i64) -> Self {
        self.expires = Some(expires);
        self
    }
}

/// Réponse de `POST /v1/me/keys` : seule occasion où le token complet est révélé.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CreateKeyResponse {
    /// Identifiant de la clé créée.
    pub id: i64,
    /// Token complet — à stocker immédiatement (non récupérable ensuite).
    pub token: String,
}

/// Une clé d'API de l'utilisateur.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Key {
    /// Identifiant de la clé.
    pub id: i64,
    /// Nom lisible.
    pub name: String,
    /// Token (potentiellement masqué selon l'endpoint).
    pub token: String,
    /// Expiration (timestamp Unix), si définie.
    #[serde(default)]
    pub expires: Option<i64>,
    /// Création (timestamp Unix).
    pub created: i64,
}

/// Réponse de `GET /v1/me/keys` : liste des clés.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Keys {
    /// Clés de l'utilisateur.
    pub data: Vec<Key>,
}

/// Détail d'usage agrégé (une entrée par ligne d'usage).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct UsageDetail {
    /// Tokens de prompt consommés.
    #[serde(default)]
    pub prompt_tokens: Option<i64>,
    /// Tokens de complétion générés.
    #[serde(default)]
    pub completion_tokens: Option<i64>,
    /// Total de tokens.
    #[serde(default)]
    pub total_tokens: Option<i64>,
    /// Coût estimé.
    #[serde(default)]
    pub cost: Option<f64>,
    /// Empreinte environnementale.
    #[serde(default)]
    pub impacts: Option<EnvironmentalImpacts>,
    /// Métriques additionnelles dépendantes de l'endpoint (surface instable :
    /// laissée en JSON brut tant qu'OGL est en beta).
    #[serde(default)]
    pub metrics: Option<serde_json::Value>,
}

/// Réponse de `GET /v1/me/usage`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Usages {
    /// Détails d'usage.
    pub data: Vec<UsageDetail>,
}

/// Endpoint facturable, pour filtrer `GET /v1/me/usage`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum EndpointUsage {
    /// `/v1/audio/transcriptions`.
    #[serde(rename = "/v1/audio/transcriptions")]
    AudioTranscriptions,
    /// `/v1/chat/completions`.
    #[serde(rename = "/v1/chat/completions")]
    ChatCompletions,
    /// `/v1/embeddings`.
    #[serde(rename = "/v1/embeddings")]
    Embeddings,
    /// `/v1/ocr`.
    #[serde(rename = "/v1/ocr")]
    Ocr,
    /// `/v1/rerank`.
    #[serde(rename = "/v1/rerank")]
    Rerank,
    /// `/v1/search`.
    #[serde(rename = "/v1/search")]
    Search,
}

/// Paramètres de pagination / tri pour `GET /v1/me/keys`.
#[derive(Debug, Clone, Default, Serialize)]
pub struct KeysQuery {
    /// Décalage de pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
    /// Taille de page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Champ de tri.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_by: Option<String>,
    /// Direction de tri (`asc` / `desc`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_direction: Option<String>,
}

impl KeysQuery {
    /// Construit des paramètres vides (aucun filtre).
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

    /// Définit le champ et la direction de tri.
    #[must_use]
    pub fn order(mut self, by: impl Into<String>, direction: impl Into<String>) -> Self {
        self.order_by = Some(by.into());
        self.order_direction = Some(direction.into());
        self
    }
}

/// Paramètres de filtrage pour `GET /v1/me/usage`.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UsageQuery {
    /// Décalage de pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
    /// Taille de page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Borne basse de la fenêtre (timestamp Unix).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<i64>,
    /// Borne haute de la fenêtre (timestamp Unix).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<i64>,
    /// Restreint à un endpoint facturable donné.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<EndpointUsage>,
}

impl UsageQuery {
    /// Construit des paramètres vides (aucun filtre).
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

    /// Définit la fenêtre temporelle (timestamps Unix).
    #[must_use]
    pub fn time_range(mut self, start: i64, end: i64) -> Self {
        self.start_time = Some(start);
        self.end_time = Some(end);
        self
    }

    /// Restreint à un endpoint facturable.
    #[must_use]
    pub fn endpoint(mut self, endpoint: EndpointUsage) -> Self {
        self.endpoint = Some(endpoint);
        self
    }
}

impl Client {
    /// `GET /v1/me/info` — informations sur l'utilisateur courant.
    pub async fn me_info(&self) -> Result<UserInfo, Error> {
        let url = self.endpoint("/v1/me/info")?;
        self.get_json(url).await
    }

    /// `PATCH /v1/me/info` — met à jour le profil (renvoie `204 No Content`).
    pub async fn update_me_info(&self, update: &UpdateUserInfo) -> Result<(), Error> {
        let url = self.endpoint("/v1/me/info")?;
        self.patch_no_content(url, update).await
    }

    /// `POST /v1/me/keys` — crée une clé d'API et renvoie le token complet.
    pub async fn create_key(&self, key: &CreateKey) -> Result<CreateKeyResponse, Error> {
        let url = self.endpoint("/v1/me/keys")?;
        self.post_json(url, key).await
    }

    /// `GET /v1/me/keys` — liste les clés d'API de l'utilisateur.
    pub async fn keys(&self, query: &KeysQuery) -> Result<Keys, Error> {
        let url = self.endpoint("/v1/me/keys")?;
        self.get_json_with_query(url, query).await
    }

    /// `GET /v1/me/keys/{key}` — récupère une clé par identifiant.
    pub async fn key(&self, key_id: impl std::fmt::Display) -> Result<Key, Error> {
        let url = self.endpoint_with_segment("/v1/me/keys/", &key_id.to_string())?;
        self.get_json(url).await
    }

    /// `DELETE /v1/me/keys/{key}` — révoque une clé (renvoie `204 No Content`).
    pub async fn delete_key(&self, key_id: impl std::fmt::Display) -> Result<(), Error> {
        let url = self.endpoint_with_segment("/v1/me/keys/", &key_id.to_string())?;
        self.delete_no_content(url).await
    }

    /// `GET /v1/me/usage` — usage agrégé de l'utilisateur (tokens, coût, impacts).
    pub async fn usage(&self, query: &UsageQuery) -> Result<Usages, Error> {
        let url = self.endpoint("/v1/me/usage")?;
        self.get_json_with_query(url, query).await
    }
}
