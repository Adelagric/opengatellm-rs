#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
//! Tests d'intégration de `POST /v1/chat/completions` (non-streaming) contre une instance OGL.

use opengatellm::{ChatContent, ChatMessage, Client, CreateChatCompletion};

const DEFAULT_CHAT_MODEL: &str = "qwen3-coder";

fn client_from_env() -> Option<Client> {
    let url = std::env::var("OGL_TEST_URL").ok()?;
    let token = std::env::var("OGL_TEST_TOKEN").ok();
    Some(Client::new(&url, token).expect("OGL_TEST_URL must be a valid base URL"))
}

#[tokio::test]
#[ignore = "needs OGL instance — run via `make test-integration`"]
async fn chat_completion_basic_prompt() {
    let Some(client) = client_from_env() else {
        panic!("OGL_TEST_URL must be set for integration tests");
    };
    let model = std::env::var("OGL_TEST_CHAT_MODEL").unwrap_or_else(|_| DEFAULT_CHAT_MODEL.into());
    let req = CreateChatCompletion::new(
        vec![
            ChatMessage::system("Réponds en une phrase, en français."),
            ChatMessage::user("Quelle est la capitale de la France ?"),
        ],
        model,
    )
    .temperature(0.0)
    .max_completion_tokens(64);
    let resp = client
        .chat_completion(&req)
        .await
        .expect("chat_completion() should succeed");
    assert!(!resp.choices.is_empty(), "at least one choice expected");
    let msg = &resp.choices[0].message;
    let content = msg
        .content
        .as_ref()
        .expect("assistant should return content");
    let text = match content {
        ChatContent::Text(t) => t.clone(),
        ChatContent::Parts(_) => panic!("text-only response expected for this model"),
    };
    assert!(
        text.to_lowercase().contains("paris"),
        "expected 'Paris' in answer, got: {text}"
    );
}
