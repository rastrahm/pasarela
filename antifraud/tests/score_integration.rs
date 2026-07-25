//! Tests de integración — scoring antifraude.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use antifraud_service::build_app;
use antifraud_service::config::AppConfig;
use rust_decimal::Decimal;
use std::str::FromStr;
use std::sync::Arc;
use tower::ServiceExt;

fn test_config() -> AppConfig {
    AppConfig {
        host: "127.0.0.1".to_string(),
        port: 0,
        api_key: "test-key".to_string(),
        max_amount: Decimal::from_str("1000").expect("decimal"),
        blocked_token_hashes: vec![],
        decline_score_threshold: 0.85,
    }
}

#[tokio::test]
async fn score_requires_api_key() {
    let app = build_app(test_config());

    let body = serde_json::json!({
        "gateway_request_id": "550e8400-e29b-41d4-a716-446655440000",
        "amount": "100",
        "currency": "USD",
        "funding_type": "traditional_bank",
        "token_hash": "tok_ok"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/internal/v1/score")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn score_approves_valid_request() {
    let app = build_app(test_config());

    let body = serde_json::json!({
        "gateway_request_id": "550e8400-e29b-41d4-a716-446655440001",
        "amount": "100",
        "currency": "USD",
        "funding_type": "traditional_bank",
        "token_hash": "tok_ok"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/internal/v1/score")
                .header("content-type", "application/json")
                .header("x-api-key", "test-key")
                .body(Body::from(body.to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);

    let bytes = response.into_body().collect().await.expect("body").to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
    assert_eq!(json["approved"], true);
}

#[tokio::test]
async fn score_declines_high_amount() {
    let app = build_app(test_config());

    let body = serde_json::json!({
        "gateway_request_id": "550e8400-e29b-41d4-a716-446655440002",
        "amount": "99999",
        "currency": "USD",
        "funding_type": "traditional_bank",
        "token_hash": "tok_ok"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/internal/v1/score")
                .header("content-type", "application/json")
                .header("x-api-key", "test-key")
                .body(Body::from(body.to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);

    let bytes = response.into_body().collect().await.expect("body").to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
    assert_eq!(json["approved"], false);
}
