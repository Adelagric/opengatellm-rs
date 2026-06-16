#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
//! Round-trips de désérialisation des schémas monitoring (`/health*`).

use opengatellm::{Health, HealthStatus, ModelsHealthResponse};

#[test]
fn deserialize_health() {
    let h: Health = serde_json::from_str(r#"{"status":"ok"}"#).unwrap();
    assert_eq!(h.status, "ok");
}

#[test]
fn deserialize_models_health() {
    let json = r#"{"data":[{"id":"albert-large","status":"green"},{"id":"embed","status":"red"}]}"#;
    let r: ModelsHealthResponse = serde_json::from_str(json).unwrap();
    assert_eq!(r.data.len(), 2);
    assert_eq!(r.data[0].id, "albert-large");
    assert_eq!(r.data[0].status, HealthStatus::Green);
    assert_eq!(r.data[1].status, HealthStatus::Red);
}

#[test]
fn health_status_serializes_lowercase() {
    assert_eq!(
        serde_json::to_value(HealthStatus::Yellow).unwrap(),
        "yellow"
    );
    let s: HealthStatus = serde_json::from_str("\"green\"").unwrap();
    assert_eq!(s, HealthStatus::Green);
}
