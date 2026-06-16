#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
//! Round-trips de (dé)sérialisation des schémas `/v1/me/*`.

use opengatellm::{
    CreateKey, CreateKeyResponse, EndpointUsage, Key, Keys, KeysQuery, LimitType, PermissionType,
    UpdateUserInfo, UsageQuery, Usages, UserInfo,
};

#[test]
fn update_user_info_skips_unset_fields() {
    let empty = serde_json::to_value(UpdateUserInfo::new()).unwrap();
    assert_eq!(empty, serde_json::json!({}), "empty patch must be {{}}");

    let v = serde_json::to_value(UpdateUserInfo::new().name("Alice").email("a@b.fr")).unwrap();
    assert_eq!(v["name"], "Alice");
    assert_eq!(v["email"], "a@b.fr");
    assert!(v.get("password").is_none());
    assert!(v.get("current_password").is_none());
}

#[test]
fn update_user_info_password_sets_both_fields() {
    let v = serde_json::to_value(UpdateUserInfo::new().password("old", "new")).unwrap();
    assert_eq!(v["current_password"], "old");
    assert_eq!(v["password"], "new");
}

#[test]
fn create_key_serialization() {
    let v = serde_json::to_value(CreateKey::new("ci-key")).unwrap();
    assert_eq!(v["name"], "ci-key");
    assert!(v.get("expires").is_none());

    let v = serde_json::to_value(CreateKey::new("ci-key").expires(1_000)).unwrap();
    assert_eq!(v["expires"], 1_000);
}

#[test]
fn deserialize_user_info_full() {
    let json = r#"{
      "object":"user","id":1,"email":"a@b.fr","name":"Alice","organization":2,
      "budget":12.5,"permissions":["admin","read_metric"],
      "limits":[{"router_id":3,"type":"rpm","value":60}],
      "expires":null,"priority":0,"created":100,"updated":200
    }"#;
    let u: UserInfo = serde_json::from_str(json).unwrap();
    assert_eq!(u.id, 1);
    assert_eq!(u.email, "a@b.fr");
    assert_eq!(u.name.as_deref(), Some("Alice"));
    assert_eq!(
        u.permissions,
        vec![PermissionType::Admin, PermissionType::ReadMetric]
    );
    assert_eq!(u.limits.len(), 1);
    assert_eq!(u.limits[0].kind, LimitType::Rpm);
    assert_eq!(u.limits[0].value, Some(60));
    assert!(u.expires.is_none());
}

#[test]
fn deserialize_user_info_minimal() {
    let json = r#"{"id":7,"email":"x@y.fr","permissions":[],"limits":[],"created":1,"updated":1}"#;
    let u: UserInfo = serde_json::from_str(json).unwrap();
    assert_eq!(u.id, 7);
    assert!(u.name.is_none());
    assert!(u.budget.is_none());
    assert!(u.permissions.is_empty());
}

#[test]
fn deserialize_keys_and_create_response() {
    let keys: Keys = serde_json::from_str(
        r#"{"object":"list","data":[{"object":"key","id":1,"name":"k","token":"sk-xxx","expires":null,"created":100}]}"#,
    )
    .unwrap();
    assert_eq!(keys.data.len(), 1);
    let k: &Key = &keys.data[0];
    assert_eq!(k.id, 1);
    assert_eq!(k.name, "k");

    let created: CreateKeyResponse =
        serde_json::from_str(r#"{"id":9,"token":"sk-secret"}"#).unwrap();
    assert_eq!(created.id, 9);
    assert_eq!(created.token, "sk-secret");
}

#[test]
fn deserialize_usages_nested_shape() {
    // Forme réelle du serveur : compteurs imbriqués sous `usage`, `created` requis.
    let json = r#"{"object":"list","data":[
      {"object":"me.usage","model":"albert-large","endpoint":"/v1/rerank","method":"POST","status":200,
       "usage":{"prompt_tokens":10,"completion_tokens":5,"total_tokens":15,"cost":0.01,
                "impacts":{"kWh":0.001,"kgCO2eq":0.0005},"metrics":{"latency":120,"ttft":40}},
       "created":1700000000}
    ]}"#;
    let u: Usages = serde_json::from_str(json).unwrap();
    assert_eq!(u.data.len(), 1);
    let rec = &u.data[0];
    assert_eq!(rec.created, 1_700_000_000);
    assert_eq!(rec.endpoint.as_deref(), Some("/v1/rerank"));
    assert_eq!(rec.usage.total_tokens, Some(15));
    assert_eq!(rec.usage.metrics.ttft, Some(40));
}

#[test]
fn endpoint_usage_renames_to_path() {
    assert_eq!(
        serde_json::to_value(EndpointUsage::Rerank).unwrap(),
        "/v1/rerank"
    );
    let e: EndpointUsage = serde_json::from_str("\"/v1/chat/completions\"").unwrap();
    assert_eq!(e, EndpointUsage::ChatCompletions);
}

#[test]
fn query_params_skip_unset() {
    let q = serde_json::to_value(KeysQuery::new().limit(50)).unwrap();
    assert_eq!(q["limit"], 50);
    assert!(q.get("offset").is_none());
    assert!(q.get("order_by").is_none());

    let q = serde_json::to_value(UsageQuery::new().endpoint(EndpointUsage::Embeddings)).unwrap();
    assert_eq!(q["endpoint"], "/v1/embeddings");
    assert!(q.get("start_time").is_none());
}
