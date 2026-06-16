#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
//! Tests du chemin requête réel (wiremock) pour les endpoints RAG :
//! chemins (mono/multi-segments), 201 `{id}`, 204, query-string, multipart.

use opengatellm::{
    Client, CollectionRequest, CollectionUpdateRequest, CollectionsQuery, CreateDocument,
    CreateSearch, DocumentsQuery,
};
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> Client {
    Client::new(server.uri(), Option::<&str>::None).unwrap()
}

#[tokio::test]
async fn create_collection_returns_id_from_201() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/collections"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({"id": 5})))
        .mount(&server)
        .await;
    let id = client(&server)
        .create_collection(&CollectionRequest::new("docs"))
        .await
        .expect("create_collection should parse {id}");
    assert_eq!(id, 5);
}

#[tokio::test]
async fn collection_crud_paths_single_slash() {
    // GET /v1/collections/5
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/collections/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!(
            {"object":"collection","id":5,"name":"c","owner":"u","created":1,"updated":2}
        )))
        .mount(&server)
        .await;
    assert_eq!(client(&server).collection(5).await.unwrap().id, 5);

    // PATCH /v1/collections/5 -> 204
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/v1/collections/5"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;
    client(&server)
        .update_collection(5, &CollectionUpdateRequest::new().name("c2"))
        .await
        .expect("update should map 204 to Ok");

    // DELETE /v1/collections/5 -> 204
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/v1/collections/5"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;
    client(&server)
        .delete_collection(5)
        .await
        .expect("delete should map 204 to Ok");
}

#[tokio::test]
async fn collections_list_sends_pagination() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/collections"))
        .and(query_param("limit", "10"))
        .and(query_param("offset", "20"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": []})))
        .mount(&server)
        .await;
    client(&server)
        .collections(&CollectionsQuery::new().limit(10).offset(20))
        .await
        .expect("collections() should send pagination");
}

#[tokio::test]
async fn search_posts_body_and_parses_results() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/search"))
        .and(body_json(
            serde_json::json!({"query": "irrigation", "limit": 3}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "object": "list",
            "data": [{"method": "semantic", "score": 0.9,
                      "chunk": {"id": 1, "collection_id": 2, "document_id": 3, "content": "c"}}]
        })))
        .mount(&server)
        .await;
    let res = client(&server)
        .search(&CreateSearch::new("irrigation").limit(3))
        .await
        .expect("search() should succeed");
    assert_eq!(res.data.len(), 1);
    assert_eq!(res.data[0].chunk.document_id, 3);
}

#[tokio::test]
async fn chunk_two_segment_path() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/chunks/7/3")) // multi-segments, pas de double slash
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!(
            {"object":"chunk","id":3,"collection_id":2,"document_id":7,"content":"c"}
        )))
        .mount(&server)
        .await;
    let c = client(&server)
        .chunk(7, 3)
        .await
        .expect("chunk() should hit /v1/chunks/7/3");
    assert_eq!(c.id, 3);
}

#[tokio::test]
async fn document_chunks_single_segment_path() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/chunks/7"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": []})))
        .mount(&server)
        .await;
    client(&server)
        .document_chunks(7)
        .await
        .expect("document_chunks() should hit /v1/chunks/7");
}

#[tokio::test]
async fn create_document_posts_multipart() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/documents"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({"id": 9})))
        .mount(&server)
        .await;
    let resp = client(&server)
        .create_document(
            "f.txt",
            b"hello".to_vec(),
            &CreateDocument::new(2).chunk_size(512),
        )
        .await
        .expect("create_document multipart should parse {id}");
    assert_eq!(resp.id, 9);
}

#[tokio::test]
async fn documents_list_filters_by_collection() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/documents"))
        .and(query_param("collection_id", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": []})))
        .mount(&server)
        .await;
    client(&server)
        .documents(&DocumentsQuery::new().collection(2))
        .await
        .expect("documents() should send collection_id filter");
}
