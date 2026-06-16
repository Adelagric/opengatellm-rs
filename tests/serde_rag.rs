#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
//! Round-trips de (dé)sérialisation des schémas RAG (search, collections, documents, chunks).

use opengatellm::{
    Chunks, Collection, CollectionRequest, CollectionUpdateRequest, CollectionVisibility,
    Collections, CreateSearch, Document, DocumentResponse, Documents, SearchMethod, Searches,
};

#[test]
fn create_search_serialization() {
    let v = serde_json::to_value(CreateSearch::new("aide irrigation")).unwrap();
    assert_eq!(v["query"], "aide irrigation");
    assert!(
        v.get("collection_ids").is_none(),
        "empty vec must be skipped"
    );
    assert!(v.get("method").is_none());

    let v = serde_json::to_value(
        CreateSearch::new("q")
            .collections([1, 2])
            .method(SearchMethod::Hybrid)
            .limit(5),
    )
    .unwrap();
    assert_eq!(v["collection_ids"], serde_json::json!([1, 2]));
    assert_eq!(v["method"], "hybrid");
    assert_eq!(v["limit"], 5);
}

#[test]
fn deserialize_searches_with_nested_chunk() {
    let json = r#"{"object":"list","data":[
      {"method":"semantic","score":0.83,
       "chunk":{"object":"chunk","id":1,"collection_id":2,"document_id":3,"content":"texte"}}
    ],"usage":{"prompt_tokens":5,"total_tokens":5}}"#;
    let s: Searches = serde_json::from_str(json).unwrap();
    assert_eq!(s.data.len(), 1);
    assert_eq!(s.data[0].method, SearchMethod::Semantic);
    assert_eq!(s.data[0].chunk.content, "texte");
    assert_eq!(s.data[0].chunk.document_id, 3);
    assert!(s.usage.is_some());
}

#[test]
fn collection_request_and_update_skip_unset() {
    let v = serde_json::to_value(CollectionRequest::new("docs")).unwrap();
    assert_eq!(v["name"], "docs");
    assert!(v.get("visibility").is_none());

    let v = serde_json::to_value(
        CollectionRequest::new("docs").visibility(CollectionVisibility::Public),
    )
    .unwrap();
    assert_eq!(v["visibility"], "public");

    let empty = serde_json::to_value(CollectionUpdateRequest::new()).unwrap();
    assert_eq!(empty, serde_json::json!({}));
}

#[test]
fn deserialize_collection_and_collections() {
    let json = r#"{"object":"list","data":[
      {"object":"collection","id":1,"name":"c","owner":"u","description":null,
       "visibility":"private","created":10,"updated":20,"documents":3,"size":1024}
    ]}"#;
    let c: Collections = serde_json::from_str(json).unwrap();
    assert_eq!(c.data.len(), 1);
    let first: &Collection = &c.data[0];
    assert_eq!(first.name, "c");
    assert_eq!(first.visibility, Some(CollectionVisibility::Private));
    assert_eq!(first.documents, Some(3));
}

#[test]
fn deserialize_documents_and_response() {
    let docs: Documents = serde_json::from_str(
        r#"{"data":[{"object":"document","id":1,"name":"d.pdf","collection_id":2,"created":10,"chunks":4}]}"#,
    )
    .unwrap();
    assert_eq!(docs.data.len(), 1);
    let d: &Document = &docs.data[0];
    assert_eq!(d.name, "d.pdf");
    assert_eq!(d.chunks, Some(4));

    let created: DocumentResponse = serde_json::from_str(r#"{"id":42}"#).unwrap();
    assert_eq!(created.id, 42);
}

#[test]
fn deserialize_chunks() {
    let chunks: Chunks = serde_json::from_str(
        r#"{"data":[{"object":"chunk","id":1,"collection_id":2,"document_id":3,"content":"x","metadata":{"page":1}}]}"#,
    )
    .unwrap();
    assert_eq!(chunks.data.len(), 1);
    assert_eq!(chunks.data[0].content, "x");
    assert!(chunks.data[0].metadata.is_some());
}
