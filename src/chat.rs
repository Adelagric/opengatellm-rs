//! Endpoint `POST /v1/chat/completions` (non-streaming).
//!
//! Le streaming SSE est géré séparément par [`crate::stream`].

use crate::client::Client;
use crate::common::Usage;
use crate::error::Error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Rôle d'un message dans la conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// Instructions système (souvent injectées en tête de conversation).
    System,
    /// Message utilisateur.
    User,
    /// Réponse du modèle.
    Assistant,
    /// Résultat d'un appel d'outil.
    Tool,
    /// Rôle développeur (`OpenAI` Responses API).
    Developer,
}

/// Contenu d'un message : texte simple ou parties multimodales.
///
/// Côté input, OGL accepte les deux formes (compat `OpenAI`). Côté output,
/// les modèles texte-only ne renvoient qu'une `String`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ChatContent {
    /// Texte simple.
    Text(String),
    /// Parties multimodales (texte + images).
    Parts(Vec<ContentPart>),
}

impl From<String> for ChatContent {
    fn from(s: String) -> Self {
        Self::Text(s)
    }
}

impl From<&str> for ChatContent {
    fn from(s: &str) -> Self {
        Self::Text(s.to_owned())
    }
}

/// Une partie d'un contenu multimodal.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPart {
    /// Bloc de texte.
    Text {
        /// Texte de la partie.
        text: String,
    },
    /// Référence à une image (URL ou data URI base64).
    ImageUrl {
        /// Détails de l'image référencée.
        image_url: ImageUrl,
    },
}

/// URL d'une image (référence ou data-URI base64).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUrl {
    /// URL ou data-URI (`data:image/png;base64,...`).
    pub url: String,
    /// Hint de détail (`auto`, `low`, `high`) accepté par certains modèles vision.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Message d'une conversation (input ou output).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Rôle du locuteur.
    pub role: Role,
    /// Contenu du message (peut être `null` si le message est un appel d'outil pur).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<ChatContent>,
    /// Nom optionnel (pour distinguer plusieurs utilisateurs / outils).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Refus du modèle (renvoyé par certains modèles modérés).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refusal: Option<String>,
    /// Annotations attachées (citations URL, etc.).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Vec<Annotation>>,
    /// Identifiant de l'appel d'outil auquel ce message répond (rôle `tool`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    /// Appels d'outils émis par le modèle (rôle `assistant`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

impl ChatMessage {
    /// Raccourci : message texte simple avec rôle `user`.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: Some(ChatContent::Text(content.into())),
            name: None,
            refusal: None,
            annotations: None,
            tool_call_id: None,
            tool_calls: None,
        }
    }

    /// Raccourci : message texte simple avec rôle `system`.
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: Some(ChatContent::Text(content.into())),
            name: None,
            refusal: None,
            annotations: None,
            tool_call_id: None,
            tool_calls: None,
        }
    }

    /// Raccourci : message texte simple avec rôle `assistant`.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: Some(ChatContent::Text(content.into())),
            name: None,
            refusal: None,
            annotations: None,
            tool_call_id: None,
            tool_calls: None,
        }
    }
}

/// Citation source attachée à un message (URL annotée).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Annotation {
    /// Citation d'une URL avec plage de caractères concernée.
    UrlCitation {
        /// Détails de la citation.
        url_citation: AnnotationUrlCitation,
    },
}

/// Détails d'une citation URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationUrlCitation {
    /// Index de fin (exclusif) dans le contenu textuel.
    pub end_index: u32,
    /// Index de début (inclusif) dans le contenu textuel.
    pub start_index: u32,
    /// Titre de la page citée.
    pub title: String,
    /// URL citée.
    pub url: String,
}

/// Appel d'outil émis par le modèle.
///
/// OGL supporte deux variants : `function` (legacy / structured) et `custom`
/// (outil utilisateur libre).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToolCall {
    /// Appel de fonction structurée.
    Function {
        /// Identifiant unique de l'appel.
        id: String,
        /// Détails de la fonction appelée.
        function: FunctionDetails,
    },
    /// Appel d'outil custom.
    Custom {
        /// Identifiant unique de l'appel.
        id: String,
        /// Détails de l'outil custom.
        custom: CustomDetails,
    },
}

/// Détails d'un appel de fonction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDetails {
    /// Nom de la fonction.
    pub name: String,
    /// Arguments JSON-stringifiés (à parser côté appelant).
    pub arguments: String,
}

/// Détails d'un appel d'outil custom.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomDetails {
    /// Nom de l'outil.
    pub name: String,
    /// Entrée brute fournie par le modèle.
    pub input: String,
}

/// Raison de terminaison d'une génération.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    /// Stop naturel (EOS).
    Stop,
    /// Limite de tokens atteinte.
    Length,
    /// Appels d'outils retournés.
    ToolCalls,
    /// Bloqué par le filtre de contenu.
    ContentFilter,
    /// Appel de fonction (legacy).
    FunctionCall,
}

/// Conditions de stop : un mot ou une liste.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Stop {
    /// Un seul motif de stop.
    Single(String),
    /// Plusieurs motifs (le générateur s'arrête au premier rencontré).
    Many(Vec<String>),
}

/// Niveau de service côté `OpenAI` (rarement utilisé côté self-hosted).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServiceTier {
    /// Auto.
    Auto,
    /// Défaut.
    Default,
    /// Flex (best-effort, latence variable).
    Flex,
    /// Scale.
    Scale,
    /// Priorité.
    Priority,
}

/// Logprob d'un token candidat.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TopLogprob {
    /// Texte du token.
    pub token: String,
    /// Bytes UTF-8 du token (utile si le token n'est pas du texte valide).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bytes: Option<Vec<i32>>,
    /// Log-probabilité.
    pub logprob: f64,
}

/// Logprob d'un token avec ses alternatives.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TokenLogprob {
    /// Texte du token.
    pub token: String,
    /// Bytes UTF-8 du token.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bytes: Option<Vec<i32>>,
    /// Log-probabilité.
    pub logprob: f64,
    /// Tokens alternatifs et leurs logprobs.
    pub top_logprobs: Vec<TopLogprob>,
}

/// Logprobs d'un `Choice`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ChoiceLogprobs {
    /// Logprobs du contenu généré.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<TokenLogprob>>,
    /// Logprobs du refus (si applicable).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refusal: Option<Vec<TokenLogprob>>,
}

/// Un choix retourné par le modèle (généralement un seul, sauf `n > 1`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Choice {
    /// Index du choix dans la liste (0-based).
    pub index: u32,
    /// Message complet du choix.
    pub message: ChatMessage,
    /// Raison de terminaison.
    pub finish_reason: FinishReason,
    /// Logprobs détaillés si demandés via `logprobs: true`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<ChoiceLogprobs>,
}

/// Réponse non-streamée de `POST /v1/chat/completions`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ChatCompletion {
    /// Identifiant de la complétion.
    #[serde(default)]
    pub id: String,
    /// Liste des choix.
    pub choices: Vec<Choice>,
    /// Timestamp Unix de création.
    pub created: i64,
    /// Modèle utilisé.
    pub model: String,
    /// Type de l'objet (`chat.completion`).
    #[serde(default)]
    pub object: String,
    /// Niveau de service.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<ServiceTier>,
    /// Empreinte système (config backend).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
    /// Compteurs d'usage / coût / empreinte.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
    /// Résultats de recherche RAG (extension OGL).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub search_results: Option<Vec<serde_json::Value>>,
}

/// Corps de requête de `POST /v1/chat/completions`.
///
/// Construire via [`CreateChatCompletion::new`] + builder, ou littéralement
/// via `CreateChatCompletion { ... }` si plus pratique.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateChatCompletion {
    /// Liste des messages constituant la conversation.
    pub messages: Vec<ChatMessage>,
    /// Identifiant du modèle (cf. `client.models()`).
    pub model: String,
    /// `[-2.0, 2.0]` — pénalise la fréquence de tokens déjà émis.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f64>,
    /// Biais de logit par token id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<HashMap<String, f64>>,
    /// Renvoie les logprobs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<bool>,
    /// Nombre d'alternatives top logprobs à inclure.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<u8>,
    /// `[-2.0, 2.0]` — pénalise la présence de tokens déjà émis.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f64>,
    /// Nombre max de tokens en sortie.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<u32>,
    /// Nombre de complétions à générer (souvent 1).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    /// Format de réponse (JSON Mode, Structured Outputs, etc.).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_format: Option<serde_json::Value>,
    /// Seed pour la génération déterministe.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    /// Motif(s) de stop.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop: Option<Stop>,
    /// Active le streaming SSE. Voir aussi
    /// [`Client::chat_completion_stream`](crate::client::Client) à l'étape 4.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    /// Options du streaming (`include_usage`, etc.).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<serde_json::Value>,
    /// `[0.0, 2.0]` — randomness.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    /// `(0.0, 1.0]` — nucleus sampling.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    /// Outils que le modèle peut appeler (functions + `SearchTool` OGL).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<serde_json::Value>>,
    /// Politique de choix d'outil.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<serde_json::Value>,
    /// Autorise les appels d'outils en parallèle.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,
    /// Identifiant utilisateur (pour le logging côté OGL).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    /// Active la recherche RAG OGL (extension propre à `OpenGateLLM`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub search: Option<bool>,
    /// Paramètres de recherche RAG (extension OGL).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub search_args: Option<serde_json::Value>,
}

impl CreateChatCompletion {
    /// Construit une requête minimale `(messages, model)`.
    pub fn new(messages: Vec<ChatMessage>, model: impl Into<String>) -> Self {
        Self {
            messages,
            model: model.into(),
            ..Self::default()
        }
    }

    /// Définit la temperature.
    #[must_use]
    pub fn temperature(mut self, t: f64) -> Self {
        self.temperature = Some(t);
        self
    }

    /// Définit le `top_p`.
    #[must_use]
    pub fn top_p(mut self, p: f64) -> Self {
        self.top_p = Some(p);
        self
    }

    /// Définit le nombre max de tokens en sortie.
    #[must_use]
    pub fn max_completion_tokens(mut self, n: u32) -> Self {
        self.max_completion_tokens = Some(n);
        self
    }

    /// Définit le seed déterministe.
    #[must_use]
    pub fn seed(mut self, seed: i64) -> Self {
        self.seed = Some(seed);
        self
    }
}

impl Client {
    /// `POST /v1/chat/completions` (non-streaming).
    ///
    /// Force `stream = false` quel que soit le champ de la requête fournie.
    /// Pour du streaming SSE, utiliser `Client::chat_completion_stream`
    /// (étape 4 — module `stream`).
    pub async fn chat_completion(
        &self,
        req: &CreateChatCompletion,
    ) -> Result<ChatCompletion, Error> {
        let mut owned = req.clone();
        owned.stream = Some(false);
        let url = self.endpoint("/v1/chat/completions")?;
        self.post_json(url, &owned).await
    }
}
