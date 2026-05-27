#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
//! Round-trips de désérialisation des schémas `/v1/embeddings`.

use opengatellm::{Embeddings, EmbeddingsInput, EmbeddingsRequest, EncodingFormat};

#[test]
fn serialize_request_text_input() {
    let req = EmbeddingsRequest::new("hello world", "nomic-embed-text");
    let v = serde_json::to_value(&req).unwrap();
    assert_eq!(v["input"], "hello world");
    assert_eq!(v["model"], "nomic-embed-text");
    assert!(v.get("dimensions").is_none(), "None must be skipped");
    assert!(v.get("encoding_format").is_none());
}

#[test]
fn serialize_request_batch_input_with_dimensions() {
    let req = EmbeddingsRequest::new(vec!["a".to_owned(), "b".to_owned()], "m")
        .dimensions(512)
        .encoding_format(EncodingFormat::Base64);
    let v = serde_json::to_value(&req).unwrap();
    assert_eq!(v["input"], serde_json::json!(["a", "b"]));
    assert_eq!(v["dimensions"], 512);
    assert_eq!(v["encoding_format"], "base64");
}

#[test]
fn serialize_tokens_input() {
    let req = EmbeddingsRequest {
        input: EmbeddingsInput::Tokens(vec![1, 2, 3]),
        model: "m".to_owned(),
        dimensions: None,
        encoding_format: None,
    };
    let v = serde_json::to_value(&req).unwrap();
    assert_eq!(v["input"], serde_json::json!([1, 2, 3]));
}

const SAMPLE_RESPONSE: &str = r#"
{
  "object": "list",
  "data": [
    {"object": "embedding", "index": 0, "embedding": [0.1, 0.2, 0.3]},
    {"object": "embedding", "index": 1, "embedding": [0.4, 0.5, 0.6]}
  ],
  "model": "nomic-embed-text",
  "id": "embd-abc123",
  "usage": {"prompt_tokens": 4, "total_tokens": 4}
}
"#;

#[test]
fn deserialize_response() {
    let r: Embeddings = serde_json::from_str(SAMPLE_RESPONSE).unwrap();
    assert_eq!(r.data.len(), 2);
    assert_eq!(r.data[0].embedding.len(), 3);
    assert_eq!(r.data[1].index, 1);
    assert_eq!(r.model, "nomic-embed-text");
    assert_eq!(r.id.as_deref(), Some("embd-abc123"));
    assert!(r.usage.is_some());
}

#[test]
fn deserialize_response_without_id_and_usage() {
    let json = r#"{"object":"list","data":[],"model":"m"}"#;
    let r: Embeddings = serde_json::from_str(json).unwrap();
    assert!(r.data.is_empty());
    assert!(r.id.is_none());
    assert!(r.usage.is_none());
}

#[test]
fn encoding_format_default_is_float() {
    let f: EncodingFormat = EncodingFormat::default();
    assert_eq!(f, EncodingFormat::Float);
}
