//! Exemple : chat streaming SSE.
//!
//! Lancer :
//! ```sh
//! OGL_URL=http://localhost:8000 \
//! OGL_TOKEN=mytoken \
//! cargo run --example chat_streaming
//! ```

#![allow(clippy::expect_used, clippy::unwrap_used)]

use futures_util::StreamExt;
use opengatellm::{ChatMessage, Client, CreateChatCompletion};
use std::io::Write;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = std::env::var("OGL_URL").unwrap_or_else(|_| "http://localhost:8000".into());
    let token = std::env::var("OGL_TOKEN").ok();
    let model = std::env::var("OGL_CHAT_MODEL").unwrap_or_else(|_| "qwen3-coder".into());

    let client = Client::new(url, token)?;

    let req = CreateChatCompletion::new(
        vec![
            ChatMessage::system("Réponds en français, en quelques phrases."),
            ChatMessage::user("Raconte-moi une courte histoire sur un châtaignier."),
        ],
        model,
    )
    .temperature(0.7)
    .max_completion_tokens(256);

    let mut stream = client.chat_completion_stream(&req).await?;
    let mut stdout = std::io::stdout();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        if let Some(content) = chunk.choices.first().and_then(|c| c.delta.content.as_ref()) {
            write!(stdout, "{content}")?;
            stdout.flush()?;
        }
    }
    println!();
    Ok(())
}
