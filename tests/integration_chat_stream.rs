#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
//! Test d'intégration du streaming SSE contre une instance OGL réelle.

use futures_util::StreamExt;
use opengatellm::{ChatMessage, Client, CreateChatCompletion};

const DEFAULT_CHAT_MODEL: &str = "qwen3-coder";

fn client_from_env() -> Option<Client> {
    let url = std::env::var("OGL_TEST_URL").ok()?;
    let token = std::env::var("OGL_TEST_TOKEN").ok();
    Some(Client::new(&url, token).expect("OGL_TEST_URL must be a valid base URL"))
}

#[tokio::test]
#[ignore = "needs OGL instance — run via `make test-integration`"]
async fn chat_streaming_yields_chunks_and_terminates() {
    let Some(client) = client_from_env() else {
        panic!("OGL_TEST_URL must be set for integration tests");
    };
    let model = std::env::var("OGL_TEST_CHAT_MODEL").unwrap_or_else(|_| DEFAULT_CHAT_MODEL.into());
    let req = CreateChatCompletion::new(
        vec![
            ChatMessage::system("Réponds très brièvement en français."),
            ChatMessage::user("Salut, dis bonjour."),
        ],
        model,
    )
    .max_completion_tokens(32);

    let mut stream = client
        .chat_completion_stream(&req)
        .await
        .expect("chat_completion_stream() should start");

    let mut accumulated = String::new();
    let mut chunk_count = 0_usize;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.expect("chunk should decode");
        chunk_count += 1;
        if let Some(c) = chunk
            .choices
            .first()
            .and_then(|ch| ch.delta.content.as_ref())
        {
            accumulated.push_str(c);
        }
    }
    assert!(chunk_count > 0, "at least one chunk expected");
    assert!(!accumulated.is_empty(), "expected non-empty assembled text");
}
