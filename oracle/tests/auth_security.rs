//! Tests de seguridad — rechazo fail closed sin credenciales válidas.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use oracle_authorization::build_app;
use oracle_authorization::config::{AppConfig, CallerRule};
use std::net::IpAddr;
use tower::ServiceExt;

fn test_config() -> AppConfig {
    AppConfig {
        host: "127.0.0.1".to_string(),
        port: 0,
        api_key: "test-secret-key".to_string(),
        allowed_callers: vec![CallerRule::Exact("127.0.0.1".parse().expect("ip"))],
        rate_limit_per_minute: 100,
        rail_timeout_secs: 5,
        hold_ttl_secs: 300,
    }
}

#[tokio::test]
async fn internal_route_rejects_missing_api_key() {
    let app = build_app(test_config());

    let body = serde_json::json!({
        "gateway_request_id": "550e8400-e29b-41d4-a716-446655440000",
        "card": {
            "pan": "4111111111111111",
            "expiry_month": "12",
            "expiry_year": "30",
            "cvv": "123",
            "cardholder": "Demo"
        },
        "amount": 100.0,
        "currency": "USD",
        "funding_type": "traditional_bank"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/internal/v1/authorize")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn internal_route_rejects_invalid_api_key() {
    let app = build_app(test_config());

    let body = serde_json::json!({
        "gateway_request_id": "550e8400-e29b-41d4-a716-446655440000",
        "card": {
            "pan": "4111111111111111",
            "expiry_month": "12",
            "expiry_year": "30",
            "cvv": "123",
            "cardholder": "Demo"
        },
        "amount": 100.0,
        "currency": "USD",
        "funding_type": "traditional_bank"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/internal/v1/authorize")
                .header("content-type", "application/json")
                .header("x-api-key", "wrong-key")
                .body(Body::from(body.to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn internal_route_accepts_valid_api_key() {
    let app = build_app(test_config());

    let body = serde_json::json!({
        "gateway_request_id": "550e8400-e29b-41d4-a716-446655440000",
        "card": {
            "pan": "4111111111111111",
            "expiry_month": "12",
            "expiry_year": "30",
            "cvv": "123",
            "cardholder": "Demo"
        },
        "amount": 100.0,
        "currency": "USD",
        "funding_type": "traditional_bank"
    });

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

    assert_eq!(response.status(), StatusCode::OK);

    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();

    let json: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
    assert!(json.get("hold_id").is_some());
    assert_eq!(json["brand_code"], 1);
}

#[allow(dead_code)]
fn _ip(addr: &str) -> IpAddr {
    addr.parse().expect("ip")
}
