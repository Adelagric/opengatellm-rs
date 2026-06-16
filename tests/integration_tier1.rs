#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
//! Tests d'intégration Tier 1 (monitoring + me/*) contre une instance OGL réelle.
//!
//! Activés via `--ignored`. Variables d'environnement :
//! - `OGL_TEST_URL` : URL de base du gateway (ex: `http://localhost:8000`)
//! - `OGL_TEST_TOKEN` : bearer token (requis pour `health/models`, `me/*`)
//!
//! Lancer via `make test-integration` ou
//! `OGL_TEST_URL=http://localhost:8000 cargo test --tests -- --ignored`.

use opengatellm::{Client, KeysQuery, UsageQuery};

fn client_from_env() -> Option<Client> {
    let url = std::env::var("OGL_TEST_URL").ok()?;
    let token = std::env::var("OGL_TEST_TOKEN").ok();
    Some(Client::new(&url, token).expect("OGL_TEST_URL must be a valid base URL"))
}

#[tokio::test]
#[ignore = "needs OGL instance — run via `make test-integration`"]
async fn health_reports_ok() {
    let Some(client) = client_from_env() else {
        panic!("OGL_TEST_URL must be set for integration tests");
    };
    let h = client.health().await.expect("health() should succeed");
    assert!(!h.status.is_empty(), "health status should not be empty");
}

#[tokio::test]
#[ignore = "needs OGL instance — run via `make test-integration`"]
async fn health_models_succeeds() {
    let Some(client) = client_from_env() else {
        panic!("OGL_TEST_URL must be set for integration tests");
    };
    // Le détail dépend des modèles déployés ; on vérifie seulement que l'appel aboutit.
    let _ = client
        .health_models()
        .await
        .expect("health_models() should succeed");
}

#[tokio::test]
#[ignore = "needs OGL instance — run via `make test-integration`"]
async fn me_info_returns_identity() {
    let Some(client) = client_from_env() else {
        panic!("OGL_TEST_URL must be set for integration tests");
    };
    let me = client.me_info().await.expect("me_info() should succeed");
    assert!(!me.email.is_empty(), "user email should not be empty");
}

#[tokio::test]
#[ignore = "needs OGL instance — run via `make test-integration`"]
async fn keys_and_usage_list() {
    let Some(client) = client_from_env() else {
        panic!("OGL_TEST_URL must be set for integration tests");
    };
    let _ = client
        .keys(&KeysQuery::new().limit(5))
        .await
        .expect("keys() should succeed");
    let _ = client
        .usage(&UsageQuery::new().limit(5))
        .await
        .expect("usage() should succeed");
}
