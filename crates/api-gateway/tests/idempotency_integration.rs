//! Tests de integración de idempotencia (D9).

mod common;

use std::sync::atomic::Ordering;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use api_gateway::services::IDEMPOTENCY_KEY_HEADER;
use common::{
    bearer_header, build_default_test_app, checkout_amount_200, checkout_bank, spawn_mock_oracle,
    MockOracleOpts,
};

async fn post_checkout(
    app: axum::Router,
    idempotency_key: Option<&str>,
    body: &str,
) -> (StatusCode, serde_json::Value) {
    let mut builder = Request::builder()
        .method("POST")
        .uri("/api/v1/checkout")
        .header("content-type", "application/json")
        .header("x-forwarded-for", "127.0.0.1");

    if let Some(key) = idempotency_key {
        builder = builder.header(IDEMPOTENCY_KEY_HEADER, key);
    }

    let (auth_key, auth_value) = bearer_header();
    builder = builder.header(auth_key, auth_value);

    let response = app
        .oneshot(
            builder
                .body(Body::from(body.to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or(serde_json::json!({}));
    (status, json)
}

#[tokio::test]
async fn checkout_requires_idempotency_key() {
    let (oracle_url, _) = spawn_mock_oracle(MockOracleOpts::default()).await;
    let app = build_default_test_app(oracle_url);

    let mut builder = Request::builder()
        .method("POST")
        .uri("/api/v1/checkout")
        .header("content-type", "application/json");
    let (auth_key, auth_value) = bearer_header();
    builder = builder.header(auth_key, auth_value);

    let response = app
        .oneshot(
            builder
                .body(Body::from(checkout_bank()))
                .expect("request"),
        )
        .await
        .expect("response");

    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or(serde_json::json!({}));

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(json["error_code"], "INVALID_REQUEST");
}

#[tokio::test]
async fn duplicate_idempotency_key_replays_success_without_double_charge() {
    let (oracle_url, capture) = spawn_mock_oracle(MockOracleOpts::default()).await;
    let app = build_default_test_app(oracle_url);
    let body = checkout_bank();

    let (status1, json1) = post_checkout(app.clone(), Some("idem-001"), &body).await;
    let (status2, json2) = post_checkout(app, Some("idem-001"), &body).await;

    assert_eq!(status1, StatusCode::OK);
    assert_eq!(status2, StatusCode::OK);
    assert_eq!(json1["transaction_id"], json2["transaction_id"]);
    assert_eq!(capture.authorize_calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn same_idempotency_key_with_different_body_returns_409() {
    let (oracle_url, capture) = spawn_mock_oracle(MockOracleOpts::default()).await;
    let app = build_default_test_app(oracle_url);

    let (status1, _) = post_checkout(app.clone(), Some("idem-002"), &checkout_bank()).await;
    assert_eq!(status1, StatusCode::OK);

    let (status2, json2) =
        post_checkout(app, Some("idem-002"), &checkout_amount_200()).await;

    assert_eq!(status2, StatusCode::CONFLICT);
    assert_eq!(json2["error_code"], "CONFLICT");
    assert_eq!(capture.authorize_calls.load(Ordering::SeqCst), 1);
}
