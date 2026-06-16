#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
//! Round-trips de désérialisation des schémas `/v1/rerank`.

use opengatellm::{CreateRerankBody, RerankResponse};

#[test]
fn serialize_request_minimal_skips_top_n() {
    let req = CreateRerankBody::new("quelle aide ?", ["doc a", "doc b"], "bge-reranker");
    let v = serde_json::to_value(&req).unwrap();
    assert_eq!(v["query"], "quelle aide ?");
    assert_eq!(v["documents"], serde_json::json!(["doc a", "doc b"]));
    assert_eq!(v["model"], "bge-reranker");
    assert!(v.get("top_n").is_none(), "None top_n must be skipped");
}

#[test]
fn serialize_request_with_top_n() {
    let req = CreateRerankBody::new("q", vec!["a".to_owned(), "b".to_owned()], "m").top_n(1);
    let v = serde_json::to_value(&req).unwrap();
    assert_eq!(v["top_n"], 1);
}

const SAMPLE_RESPONSE: &str = r#"
{
  "object": "list",
  "id": "rrk-abc123",
  "results": [
    {"relevance_score": 0.92, "index": 0},
    {"relevance_score": 0.08, "index": 1}
  ],
  "model": "bge-reranker",
  "usage": {"prompt_tokens": 12, "total_tokens": 12}
}
"#;

#[test]
fn deserialize_response() {
    let r: RerankResponse = serde_json::from_str(SAMPLE_RESPONSE).unwrap();
    assert_eq!(r.id, "rrk-abc123");
    assert_eq!(r.model, "bge-reranker");
    assert_eq!(r.results.len(), 2);
    assert_eq!(r.results[0].index, 0);
    assert!(
        r.results[0].relevance_score > r.results[1].relevance_score,
        "results should keep their relevance ordering"
    );
    assert!(r.usage.is_some());
}

#[test]
fn deserialize_response_without_usage() {
    let json = r#"{"id":"rrk-1","results":[],"model":"m"}"#;
    let r: RerankResponse = serde_json::from_str(json).unwrap();
    assert!(r.results.is_empty());
    assert!(r.usage.is_none());
}
