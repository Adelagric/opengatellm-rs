//! Streaming SSE pour `POST /v1/chat/completions` (`stream: true`).
//!
//! Le gateway OGL renvoie un flux d'événements Server-Sent Events au format
//! `OpenAI` : chaque event a une payload `data: <json>` et la fin du flux est
//! signalée par `data: [DONE]`.

use crate::chat::{ChoiceLogprobs, CreateChatCompletion, FinishReason, Role, ServiceTier};
use crate::client::Client;
use crate::common::Usage;
use crate::error::Error;
use eventsource_stream::Eventsource;
use futures_util::Stream;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};

/// Sentinelle de fin de flux SSE.
const DONE_SENTINEL: &str = "[DONE]";

/// Chunk renvoyé par le streaming chat (`POST /v1/chat/completions` avec `stream: true`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ChatCompletionChunk {
    /// Identifiant de la complétion (constant sur tout le flux).
    pub id: String,
    /// Type de l'objet (`chat.completion.chunk`).
    #[serde(default)]
    pub object: String,
    /// Timestamp Unix de création.
    pub created: i64,
    /// Modèle utilisé.
    pub model: String,
    /// Choix incrémentaux.
    pub choices: Vec<ChunkChoice>,
    /// Niveau de service.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<ServiceTier>,
    /// Empreinte système.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
    /// Usage cumulé (présent uniquement dans le dernier chunk si `stream_options.include_usage = true`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
    /// Résultats de recherche RAG (extension OGL).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub search_results: Option<Vec<serde_json::Value>>,
}

/// Choix incrémental dans un chunk de streaming.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ChunkChoice {
    /// Index du choix.
    pub index: u32,
    /// Delta de message (champs partiels à appliquer cumulativement).
    pub delta: ChoiceDelta,
    /// Raison de fin (présente uniquement sur le dernier chunk).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<FinishReason>,
    /// Logprobs partiels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<ChoiceLogprobs>,
}

/// Delta de message incrémental.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ChoiceDelta {
    /// Rôle (généralement présent dans le premier chunk uniquement).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<Role>,
    /// Fragment de contenu textuel.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// Fragment de refus (modèles modérés).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refusal: Option<String>,
    /// Appels d'outils en streaming (les `arguments` arrivent en deltas).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ChoiceDeltaToolCall>>,
}

/// Appel d'outil partiel dans un delta.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ChoiceDeltaToolCall {
    /// Index de l'appel d'outil dans la liste (clé d'agrégation).
    pub index: u32,
    /// Identifiant (présent au premier chunk de cet outil).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Type d'outil (présent au premier chunk).
    #[serde(default, rename = "type", skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// Détails de fonction (incrémentaux).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub function: Option<ChoiceDeltaToolCallFunction>,
}

/// Détails de fonction dans un delta `tool_call`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ChoiceDeltaToolCallFunction {
    /// Nom de la fonction (présent au premier chunk).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Fragment d'arguments JSON (à concaténer).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
}

impl Client {
    /// `POST /v1/chat/completions` avec `stream: true`.
    ///
    /// Force `stream = true` côté wire. Le flux retourné se termine
    /// automatiquement à la réception de `data: [DONE]`.
    ///
    /// # Exemple
    /// ```no_run
    /// use futures_util::StreamExt;
    /// use opengatellm::{Client, ChatMessage, CreateChatCompletion};
    ///
    /// # async fn run() -> Result<(), opengatellm::Error> {
    /// let client = Client::new("http://localhost:8000", Some("TOKEN"))?;
    /// let req = CreateChatCompletion::new(
    ///     vec![ChatMessage::user("Raconte-moi une blague.")],
    ///     "qwen3-coder",
    /// );
    /// let mut stream = client.chat_completion_stream(&req).await?;
    /// while let Some(chunk) = stream.next().await {
    ///     let chunk = chunk?;
    ///     if let Some(delta) = chunk.choices.first().and_then(|c| c.delta.content.as_ref()) {
    ///         print!("{delta}");
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn chat_completion_stream(
        &self,
        req: &CreateChatCompletion,
    ) -> Result<impl Stream<Item = Result<ChatCompletionChunk, Error>>, Error> {
        let mut owned = req.clone();
        owned.stream = Some(true);
        let url = self.endpoint("/v1/chat/completions")?;
        let mut request = self.http().post(url).json(&owned);
        if let Some(key) = self.api_key() {
            request = request.bearer_auth(key);
        }
        let response = request.send().await?;
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let detail = response.text().await.unwrap_or_default();
            return Err(Error::Api { status, detail });
        }
        let event_stream = response.bytes_stream().eventsource();
        let chunks = event_stream
            .take_while(|ev| {
                let keep = match ev {
                    Ok(e) => e.data.trim() != DONE_SENTINEL,
                    Err(_) => true,
                };
                futures_util::future::ready(keep)
            })
            .map(|res| match res {
                Ok(event) => serde_json::from_str(&event.data).map_err(Error::Decode),
                Err(e) => Err(Error::Stream(e.to_string())),
            });
        Ok(chunks)
    }
}
