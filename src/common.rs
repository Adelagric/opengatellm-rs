//! Types partagés entre modules (usage de tokens, empreinte environnementale).

use serde::{Deserialize, Serialize};

/// Compteurs de tokens et coût d'une requête.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Usage {
    /// Tokens du prompt facturés.
    #[serde(default)]
    pub prompt_tokens: u64,
    /// Tokens générés en réponse.
    #[serde(default)]
    pub completion_tokens: u64,
    /// Total `prompt + completion`.
    #[serde(default)]
    pub total_tokens: u64,
    /// Coût monétaire estimé (selon les `ModelCosts` configurés côté OGL).
    #[serde(default)]
    pub cost: f64,
    /// Nombre de sous-requêtes (utile pour le RAG ou les outils enchaînés).
    #[serde(default)]
    pub requests: u64,
    /// Empreinte carbone (deprecated dans OGL au profit de `impacts`).
    #[serde(default)]
    pub carbon: Option<CarbonFootprintUsage>,
    /// Impact environnemental (kWh, kgCO2eq).
    #[serde(default)]
    pub impacts: Option<EnvironmentalImpacts>,
}

/// Empreinte environnementale (énergie + émissions équivalentes CO₂).
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EnvironmentalImpacts {
    /// Énergie consommée en kWh.
    #[serde(default, rename = "kWh")]
    pub kwh: f64,
    /// Émissions équivalentes en kg de CO₂.
    #[serde(default, rename = "kgCO2eq")]
    pub kg_co2eq: f64,
}

/// (Deprecated côté OGL) Fourchette d'empreinte carbone.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CarbonFootprintUsage {
    /// Fourchette en kWh.
    #[serde(default, rename = "kWh")]
    pub kwh: Option<CarbonFootprintRange>,
    /// Fourchette en kgCO2eq.
    #[serde(default, rename = "kgCO2eq")]
    pub kg_co2eq: Option<CarbonFootprintRange>,
}

/// Fourchette `min..max` pour une métrique environnementale.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CarbonFootprintRange {
    /// Borne basse.
    #[serde(default)]
    pub min: f64,
    /// Borne haute.
    #[serde(default)]
    pub max: f64,
}
