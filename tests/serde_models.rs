#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
//! Round-trips de désérialisation des schémas `/v1/models`.

use opengatellm::{Model, ModelType, ModelsResponse};

const SAMPLE_LIST: &str = r#"
{
  "object": "list",
  "data": [
    {
      "id": "albert-large",
      "object": "model",
      "type": "text-generation",
      "aliases": ["albert"],
      "created": 1700000000,
      "owned_by": "etalab",
      "max_context_length": 128000,
      "costs": {"prompt_tokens": 0.10, "completion_tokens": 0.20}
    },
    {
      "id": "albert-embed",
      "object": "model",
      "type": "text-embeddings-inference",
      "aliases": [],
      "created": 1700000000,
      "owned_by": "etalab"
    }
  ]
}
"#;

#[test]
fn deserialize_models_list() {
    let resp: ModelsResponse = serde_json::from_str(SAMPLE_LIST).unwrap();
    assert_eq!(resp.data.len(), 2);
    let first = &resp.data[0];
    assert_eq!(first.id, "albert-large");
    assert_eq!(first.kind, Some(ModelType::TextGeneration));
    assert_eq!(first.aliases, vec!["albert"]);
    assert_eq!(first.max_context_length, Some(128_000));
    let costs = first.costs.as_ref().unwrap();
    assert!((costs.prompt_tokens - 0.10).abs() < 1e-9);
    assert!((costs.completion_tokens - 0.20).abs() < 1e-9);

    let second = &resp.data[1];
    assert_eq!(second.kind, Some(ModelType::TextEmbeddingsInference));
    assert!(second.aliases.is_empty());
    assert!(second.costs.is_none());
}

#[test]
fn deserialize_model_with_null_type_and_missing_optionals() {
    let json = r#"{"id":"x","created":1,"owned_by":"o","type":null}"#;
    let m: Model = serde_json::from_str(json).unwrap();
    assert_eq!(m.id, "x");
    assert!(m.kind.is_none());
    assert!(m.costs.is_none());
    assert!(m.aliases.is_empty());
    assert!(m.max_context_length.is_none());
}

#[test]
fn deserialize_all_model_types() {
    for s in [
        "automatic-speech-recognition",
        "image-text-to-text",
        "image-to-text",
        "text-classification",
        "text-embeddings-inference",
        "text-generation",
    ] {
        let json = format!(r#""{s}""#);
        let _: ModelType = serde_json::from_str(&json).unwrap();
    }
}

#[test]
fn ignores_unknown_fields() {
    let json = r#"{
        "id":"x","created":1,"owned_by":"o",
        "future_field":"future_value",
        "another":{"nested":true}
    }"#;
    let m: Model = serde_json::from_str(json).unwrap();
    assert_eq!(m.id, "x");
}
