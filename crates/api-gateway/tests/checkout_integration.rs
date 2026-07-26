//! Tests de integración del checkout con Oracle simulado.

mod common;

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use api_gateway::{
    services::IDEMPOTENCY_KEY_HEADER,
    state::AppState,
};
use common::{
    bearer_header, build_default_test_app, checkout_bank, checkout_invalid_amount,
    checkout_no_rail, spawn_mock_oracle, test_app_config, test_merchant_id, MockOracleOpts,
    TEST_API_KEY,
};

fn checkout_request(body: &str, idempotency_key: &str) -> Request<Body> {
    let (auth_key, auth_value) = bearer_header();
    Request::builder()
        .method("POST")
        .uri("/api/v1/checkout")
        .header("content-type", "application/json")
        .header("x-forwarded-for", "127.0.0.1")
        .header(auth_key, auth_value)
        .header(IDEMPOTENCY_KEY_HEADER, idempotency_key)
        .body(Body::from(body.to_string()))
        .expect("request")
}

#[tokio::test]
async fn checkout_returns_settled_with_mock_oracle() {
    let (oracle_url, _) = spawn_mock_oracle(MockOracleOpts::default()).await;
    let app = build_default_test_app(oracle_url);

    let response = app
        .oneshot(checkout_request(&checkout_bank(), "checkout-int-001"))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);

    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
    assert_eq!(json["status"], "settled");
    assert_eq!(json["rail_used"], "traditional_bank");
    assert!(json["settlement_proof"]
        .as_str()
        .expect("proof")
        .starts_with("ACH-"));
}

#[tokio::test]
async fn get_transaction_returns_checkout_result() {
    let (oracle_url, _) = spawn_mock_oracle(MockOracleOpts::default()).await;
    let app = build_default_test_app(oracle_url);

    let checkout_response = app
        .clone()
        .oneshot(checkout_request(&checkout_bank(), "checkout-int-get-tx"))
        .await
        .expect("response");

    assert_eq!(checkout_response.status(), StatusCode::OK);

    let checkout_bytes = checkout_response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let checkout_json: serde_json::Value =
        serde_json::from_slice(&checkout_bytes).expect("json");
    let transaction_id = checkout_json["transaction_id"]
        .as_str()
        .expect("transaction_id");

    let (auth_key, auth_value) = bearer_header();
    let get_response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/transactions/{transaction_id}"))
                .header(auth_key, auth_value)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(get_response.status(), StatusCode::OK);

    let get_bytes = get_response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let get_json: serde_json::Value = serde_json::from_slice(&get_bytes).expect("json");
    assert_eq!(get_json["transaction_id"], transaction_id);
    assert_eq!(get_json["status"], "settled");
    assert_eq!(get_json["rail_used"], "traditional_bank");
    assert_eq!(
        get_json["settlement_proof"],
        checkout_json["settlement_proof"]
    );
}

#[tokio::test]
async fn checkout_rejects_invalid_amount() {
    let (oracle_url, _) = spawn_mock_oracle(MockOracleOpts::default()).await;
    let app = build_default_test_app(oracle_url);

    let response = app
        .oneshot(checkout_request(
            &checkout_invalid_amount(),
            "checkout-int-invalid-amount",
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn startup_fails_when_oracle_health_unreachable() {
    let merchant_id = test_merchant_id();
    let mut config = (*test_app_config("http://127.0.0.1:1".to_string(), merchant_id)).clone();
    config.oracle_health_check = true;
    config.bootstrap_test_api_key = Some(TEST_API_KEY.to_string());
    let result = AppState::new(Arc::new(config)).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn checkout_rejects_missing_api_key() {
    let (oracle_url, _) = spawn_mock_oracle(MockOracleOpts::default()).await;
    let app = build_default_test_app(oracle_url);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/checkout")
                .header("content-type", "application/json")
                .header(IDEMPOTENCY_KEY_HEADER, "no-auth")
                .body(Body::from(checkout_no_rail()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn checkout_rejects_invalid_api_key() {
    let (oracle_url, _) = spawn_mock_oracle(MockOracleOpts::default()).await;
    let app = build_default_test_app(oracle_url);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/checkout")
                .header("content-type", "application/json")
                .header("authorization", "Bearer sk_test_unknown1")
                .header(IDEMPOTENCY_KEY_HEADER, "bad-auth")
                .body(Body::from(checkout_no_rail()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
