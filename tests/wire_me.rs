#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
//! Tests du chemin requête réel (wiremock) : URLs, codes 204/201, query-string,
//! réponse texte. Ce que les tests serde ne peuvent pas voir.

use opengatellm::{Client, CreateKey, EndpointUsage, Error, KeysQuery, UpdateUserInfo, UsageQuery};
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> Client {
    Client::new(server.uri(), Option::<&str>::None).unwrap()
}

#[tokio::test]
async fn key_path_has_single_slash() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/me/keys/42")) // pas `//42`
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!(
            {"object":"key","id":42,"name":"k","token":"sk","created":1}
        )))
        .mount(&server)
        .await;
    let k = client(&server)
        .key(42)
        .await
        .expect("key() should hit /v1/me/keys/42");
    assert_eq!(k.id, 42);
}

#[tokio::test]
async fn model_path_has_single_slash() {
    // Régression : `model()` produisait `/v1/models//{id}` avant le fix.
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/models/gpt-4"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!(
            {"id":"gpt-4","created":1,"owned_by":"o"}
        )))
        .mount(&server)
        .await;
    let m = client(&server)
        .model("gpt-4")
        .await
        .expect("model() should hit /v1/models/gpt-4");
    assert_eq!(m.id, "gpt-4");
}

#[tokio::test]
async fn update_me_info_handles_204() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/v1/me/info"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;
    client(&server)
        .update_me_info(&UpdateUserInfo::new().name("Alice"))
        .await
        .expect("204 should map to Ok(())");
}

#[tokio::test]
async fn delete_key_handles_204_and_error() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/v1/me/keys/7"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;
    client(&server)
        .delete_key(7)
        .await
        .expect("204 should map to Ok(())");

    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/v1/me/keys/7"))
        .respond_with(ResponseTemplate::new(404).set_body_string("nope"))
        .mount(&server)
        .await;
    match client(&server).delete_key(7).await {
        Err(Error::Api { status, detail }) => {
            assert_eq!(status, 404);
            assert_eq!(detail, "nope");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}

#[tokio::test]
async fn create_key_handles_201() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/me/keys"))
        .respond_with(
            ResponseTemplate::new(201).set_body_json(serde_json::json!({"id":9,"token":"sk-x"})),
        )
        .mount(&server)
        .await;
    let created = client(&server)
        .create_key(&CreateKey::new("ci"))
        .await
        .expect("201 should parse CreateKeyResponse");
    assert_eq!(created.id, 9);
    assert_eq!(created.token, "sk-x");
}

#[tokio::test]
async fn keys_sends_query_params_on_the_wire() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/me/keys"))
        .and(query_param("limit", "5"))
        .and(query_param("order_by", "name"))
        .and(query_param("order_direction", "asc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"data":[]})))
        .mount(&server)
        .await;
    client(&server)
        .keys(&KeysQuery::new().limit(5).order("name", "asc"))
        .await
        .expect("query params must be sent through reqwest");
}

#[tokio::test]
async fn usage_sends_endpoint_filter_and_parses_nested() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/me/usage"))
        .and(query_param("endpoint", "/v1/rerank")) // l'enum doit s'encoder en le path
        .and(query_param("limit", "3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "object": "list",
            "data": [{
                "object": "me.usage", "endpoint": "/v1/rerank", "created": 1_700_000_000,
                "usage": {"total_tokens": 42, "metrics": {"ttft": 40}}
            }]
        })))
        .mount(&server)
        .await;
    let u = client(&server)
        .usage(&UsageQuery::new().endpoint(EndpointUsage::Rerank).limit(3))
        .await
        .expect("usage() should succeed");
    assert_eq!(u.data.len(), 1);
    assert_eq!(u.data[0].usage.total_tokens, Some(42));
    assert_eq!(u.data[0].created, 1_700_000_000);
}

#[tokio::test]
async fn metrics_returns_prometheus_text() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/metrics"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            b"# HELP ogl_inference_requests_total total\nogl_inference_requests_total 5\n".to_vec(),
            "text/plain",
        ))
        .mount(&server)
        .await;
    let text = client(&server)
        .metrics()
        .await
        .expect("metrics() should return text");
    assert!(text.contains("ogl_inference_requests_total"));
}
