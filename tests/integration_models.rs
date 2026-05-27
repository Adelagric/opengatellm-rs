#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
//! Tests d'intégration contre une instance OGL réelle.
//!
//! Activés via `--ignored`. Variable d'environnement requise :
//! - `OGL_TEST_URL` : URL de base du gateway (ex: `http://localhost:8000`)
//! - `OGL_TEST_TOKEN` : optionnel, bearer token si requis par l'instance
//!
//! Lancer via `make test-integration` ou
//! `OGL_TEST_URL=http://localhost:8000 cargo test --tests -- --ignored`.

use opengatellm::Client;

fn client_from_env() -> Option<Client> {
    let url = std::env::var("OGL_TEST_URL").ok()?;
    let token = std::env::var("OGL_TEST_TOKEN").ok();
    Some(Client::new(&url, token).expect("OGL_TEST_URL must be a valid base URL"))
}

#[tokio::test]
#[ignore = "needs OGL instance — run via `make test-integration`"]
async fn models_list_returns_non_empty() {
    let Some(client) = client_from_env() else {
        panic!("OGL_TEST_URL must be set for integration tests");
    };
    let resp = client.models().await.expect("models() should succeed");
    assert!(!resp.data.is_empty(), "expected at least one model");
}

#[tokio::test]
#[ignore = "needs OGL instance — run via `make test-integration`"]
async fn model_lookup_first_model() {
    let Some(client) = client_from_env() else {
        panic!("OGL_TEST_URL must be set for integration tests");
    };
    let list = client.models().await.expect("models() should succeed");
    let first_id = &list.data.first().expect("at least one model").id;
    let m = client
        .model(first_id)
        .await
        .expect("model() should succeed");
    assert_eq!(&m.id, first_id);
}
