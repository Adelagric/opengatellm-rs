#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
//! Tests unitaires du parser SSE de `chat_completion_stream`.
//!
//! Utilise wiremock pour simuler un endpoint SSE avec différentes formes
//! (frames normaux + sentinelle `[DONE]`, frame mal formé, erreur HTTP).

use futures_util::StreamExt;
use opengatellm::{ChatMessage, Client, CreateChatCompletion, Error, FinishReason, Role};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn chunk(content: &str, finish: Option<&str>) -> String {
    let finish_part = finish
        .map(|f| format!(r#","finish_reason":"{f}""#))
        .unwrap_or_default();
    format!(
        r#"data: {{"id":"c1","object":"chat.completion.chunk","created":1,"model":"m","choices":[{{"index":0,"delta":{{"content":"{content}"}}{finish_part}}}]}}

"#
    )
}

#[tokio::test]
async fn stream_parses_chunks_and_stops_on_done() {
    let server = MockServer::start().await;
    let body = format!(
        "{}{}{}data: [DONE]\n\n",
        // premier chunk pose le role
        r#"data: {"id":"c1","object":"chat.completion.chunk","created":1,"model":"m","choices":[{"index":0,"delta":{"role":"assistant","content":"Hel"}}]}

"#,
        chunk("lo", None),
        // finishreason sur le dernier vrai chunk
        r#"data: {"id":"c1","object":"chat.completion.chunk","created":1,"model":"m","choices":[{"index":0,"delta":{},"finish_reason":"stop"}]}

"#,
    );

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(body.into_bytes(), "text/event-stream"),
        )
        .mount(&server)
        .await;

    let client = Client::new(server.uri(), Option::<&str>::None).unwrap();
    let req = CreateChatCompletion::new(vec![ChatMessage::user("hi")], "m");
    let mut stream = client.chat_completion_stream(&req).await.unwrap();

    let mut chunks = Vec::new();
    while let Some(c) = stream.next().await {
        chunks.push(c.unwrap());
    }

    assert_eq!(
        chunks.len(),
        3,
        "[DONE] must terminate before being emitted"
    );
    assert_eq!(chunks[0].choices[0].delta.role, Some(Role::Assistant));
    assert_eq!(chunks[0].choices[0].delta.content.as_deref(), Some("Hel"));
    assert_eq!(chunks[1].choices[0].delta.content.as_deref(), Some("lo"));
    assert_eq!(chunks[2].choices[0].finish_reason, Some(FinishReason::Stop));
}

#[tokio::test]
async fn stream_done_sentinel_with_extra_whitespace_still_terminates() {
    let server = MockServer::start().await;
    let body = format!("{}data:  [DONE]  \n\n", chunk("ok", Some("stop")));

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(body.into_bytes(), "text/event-stream"),
        )
        .mount(&server)
        .await;

    let client = Client::new(server.uri(), Option::<&str>::None).unwrap();
    let req = CreateChatCompletion::new(vec![ChatMessage::user("hi")], "m");
    let mut stream = client.chat_completion_stream(&req).await.unwrap();

    let mut count = 0_usize;
    while let Some(c) = stream.next().await {
        assert!(c.is_ok());
        count += 1;
    }
    assert_eq!(count, 1, "1 real chunk before [DONE]");
}

#[tokio::test]
async fn stream_malformed_json_surfaces_decode_error() {
    let server = MockServer::start().await;
    let body = "data: {not json}\n\ndata: [DONE]\n\n";

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(body.as_bytes(), "text/event-stream"))
        .mount(&server)
        .await;

    let client = Client::new(server.uri(), Option::<&str>::None).unwrap();
    let req = CreateChatCompletion::new(vec![ChatMessage::user("hi")], "m");
    let mut stream = client.chat_completion_stream(&req).await.unwrap();

    let first = stream.next().await.unwrap();
    assert!(matches!(first, Err(Error::Decode(_))));
}

#[tokio::test]
async fn stream_http_error_returns_api_error_before_streaming() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(401).set_body_string("unauthorized"))
        .mount(&server)
        .await;

    let client = Client::new(server.uri(), Option::<&str>::None).unwrap();
    let req = CreateChatCompletion::new(vec![ChatMessage::user("hi")], "m");
    let err = client.chat_completion_stream(&req).await.err().unwrap();
    match err {
        Error::Api { status, detail } => {
            assert_eq!(status, 401);
            assert_eq!(detail, "unauthorized");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}
