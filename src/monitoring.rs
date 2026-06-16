//! Endpoints de monitoring : `GET /health`, `GET /health/models`, `GET /metrics`.

use crate::client::Client;
use crate::error::Error;
use serde::{Deserialize, Serialize};

/// Statut de santé d'un modèle (escalade-only côté OGL).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum HealthStatus {
    /// Opérationnel.
    Green,
    /// Dégradé / sous charge (file d'attente amont non vide).
    Yellow,
    /// Indisponible.
    Red,
}

/// Réponse de `GET /health` : liveness simple du gateway.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Health {
    /// Statut global (`"ok"` lorsque le gateway répond).
    pub status: String,
}

/// Santé d'un modèle individuel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ModelHealthStatus {
    /// Identifiant du modèle.
    pub id: String,
    /// Statut de santé.
    pub status: HealthStatus,
}

/// Réponse de `GET /health/models`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ModelsHealthResponse {
    /// Santé par modèle.
    pub data: Vec<ModelHealthStatus>,
}

impl Client {
    /// `GET /health` — liveness du gateway. Non authentifié côté OGL.
    pub async fn health(&self) -> Result<Health, Error> {
        let url = self.endpoint("/health")?;
        self.get_json(url).await
    }

    /// `GET /health/models` — santé par modèle (`green` / `yellow` / `red`).
    pub async fn health_models(&self) -> Result<ModelsHealthResponse, Error> {
        let url = self.endpoint("/health/models")?;
        self.get_json(url).await
    }

    /// `GET /metrics` — exposition Prometheus brute (format texte `text/plain`).
    ///
    /// Renvoie le corps Prometheus tel quel (à parser avec un client Prometheus si
    /// besoin) ; ce n'est pas du JSON. Nécessite la permission `read_metric`.
    pub async fn metrics(&self) -> Result<String, Error> {
        let url = self.endpoint("/metrics")?;
        self.get_text(url).await
    }
}
