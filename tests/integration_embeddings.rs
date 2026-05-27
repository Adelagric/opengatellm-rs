#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
//! Tests d'intégration de `POST /v1/embeddings` contre une instance OGL réelle.

use opengatellm::{Client, EmbeddingsRequest};

const DEFAULT_EMBED_MODEL: &str = "nomic-embed-text";

fn client_from_env() -> Option<Client> {
    let url = std::env::var("OGL_TEST_URL").ok()?;
    let token = std::env::var("OGL_TEST_TOKEN").ok();
    Some(Client::new(&url, token).expect("OGL_TEST_URL must be a valid base URL"))
}

#[tokio::test]
#[ignore = "needs OGL instance — run via `make test-integration`"]
async fn embeddings_single_text() {
    let Some(client) = client_from_env() else {
        panic!("OGL_TEST_URL must be set for integration tests");
    };
    let model =
        std::env::var("OGL_TEST_EMBED_MODEL").unwrap_or_else(|_| DEFAULT_EMBED_MODEL.into());
    let req = EmbeddingsRequest::new("le château fort de Carcassonne", model);
    let resp = client
        .embeddings(&req)
        .await
        .expect("embeddings() should succeed");
    assert_eq!(resp.data.len(), 1, "one vector expected for one input");
    assert!(!resp.data[0].embedding.is_empty(), "vector dim must be > 0");
}

#[tokio::test]
#[ignore = "needs OGL instance — run via `make test-integration`"]
async fn embeddings_batch_input() {
    let Some(client) = client_from_env() else {
        panic!("OGL_TEST_URL must be set for integration tests");
    };
    let model =
        std::env::var("OGL_TEST_EMBED_MODEL").unwrap_or_else(|_| DEFAULT_EMBED_MODEL.into());
    let req = EmbeddingsRequest::new(
        vec![
            "Paris est la capitale.".to_owned(),
            "Lyon est en Rhône-Alpes.".to_owned(),
        ],
        model,
    );
    let resp = client
        .embeddings(&req)
        .await
        .expect("embeddings() should succeed");
    assert_eq!(resp.data.len(), 2);
    assert_eq!(resp.data[0].index, 0);
    assert_eq!(resp.data[1].index, 1);
}
