//! Exemple : chat completion non-streaming.
//!
//! Lancer :
//! ```sh
//! OGL_URL=http://localhost:8000 \
//! OGL_TOKEN=mytoken \
//! cargo run --example chat_completion
//! ```

#![allow(clippy::expect_used, clippy::unwrap_used)]

use opengatellm::{ChatContent, ChatMessage, Client, CreateChatCompletion};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = std::env::var("OGL_URL").unwrap_or_else(|_| "http://localhost:8000".into());
    let token = std::env::var("OGL_TOKEN").ok();
    let model = std::env::var("OGL_CHAT_MODEL").unwrap_or_else(|_| "qwen3-coder".into());

    let client = Client::new(url, token)?;

    let req = CreateChatCompletion::new(
        vec![
            ChatMessage::system("Réponds en une phrase, en français."),
            ChatMessage::user("Quelle est la capitale de la France ?"),
        ],
        model,
    )
    .temperature(0.0)
    .max_completion_tokens(64);

    let resp = client.chat_completion(&req).await?;

    let choice = resp.choices.first().ok_or("réponse sans choix")?;
    let text = match choice.message.content.as_ref() {
        Some(ChatContent::Text(t)) => t.clone(),
        Some(ChatContent::Parts(_)) => "(contenu multimodal non-textuel)".into(),
        None => "(pas de contenu)".into(),
    };
    println!("Modèle : {}", resp.model);
    println!("Réponse : {text}");
    if let Some(usage) = &resp.usage {
        println!(
            "Tokens : prompt={} completion={} total={}",
            usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
        );
    }
    Ok(())
}
