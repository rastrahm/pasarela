//! Tests de integración — cliente antifraude en flujo authorize (UC-12).

mod common;

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use oracle_authorization::antifraud_client::{MockAntifraudClient, MockBehavior};
use tower::ServiceExt;

#[tokio::test]
async fn authorize_rejects_when_antifraud_declines() {
    let app = common::setup_app_with_antifraud(Arc::new(MockAntifraudClient::with_behavior(
        MockBehavior::Decline,
    )))
    .await;

    let body = common::sample_authorize_body(
        "990e8400-e29b-41d4-a716-446655440001",
        100.0,
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/internal/v1/authorize")
                .header("content-type", "application/json")
                .header("x-api-key", "test-secret-key")
                .body(Body::from(body.to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::PAYMENT_REQUIRED);

    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
    assert_eq!(json["error_code"], "FRAUD_DECLINED");
}

#[tokio::test]
async fn authorize_fail_closed_when_antifraud_unavailable() {
    let app = common::setup_app_with_antifraud(Arc::new(MockAntifraudClient::with_behavior(
        MockBehavior::Unavailable("timeout".to_string()),
    )))
    .await;

    let body = common::sample_authorize_body(
        "aa0e8400-e29b-41d4-a716-446655440001",
        100.0,
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/internal/v1/authorize")
                .header("content-type", "application/json")
                .header("x-api-key", "test-secret-key")
                .body(Body::from(body.to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
    assert_eq!(json["error_code"], "RAIL_UNAVAILABLE");
}
