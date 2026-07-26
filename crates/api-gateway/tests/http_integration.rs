//! Tests de integración HTTP del API Gateway.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use api_gateway::{build_app, config::AppConfig, state::AppState};
use domain::MerchantId;
use http_body_util::BodyExt;
use tower::ServiceExt;
use uuid::Uuid;

fn test_state() -> AppState {
    let config = Arc::new(AppConfig {
        host: "127.0.0.1".to_string(),
        port: 8080,
        oracle_base_url: "http://127.0.0.1:8081".to_string(),
        oracle_api_key: "test-gateway-key".to_string(),
        oracle_timeout_secs: 2,
        default_merchant_id: MerchantId::new(Uuid::new_v4()),
        database_url: None,
    });
    AppState::new(config).expect("state")
}

#[tokio::test]
async fn health_returns_ok() {
    let app = build_app(test_state());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .expect("request"),
        )
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
    assert_eq!(json["status"], "ok");
    assert_eq!(json["service"], "api-gateway");
}

#[tokio::test]
async fn get_transaction_route_is_registered() {
    let app = build_app(test_state());
    let tx_id = "550e8400-e29b-41d4-a716-446655440000";

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/transactions/{tx_id}"))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);
}
