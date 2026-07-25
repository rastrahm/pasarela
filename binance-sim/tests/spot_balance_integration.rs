//! Tests de integración del simulador Binance Spot.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use binance_sim_service::{build_app, config::AppConfig};
use http_body_util::BodyExt;
use rust_decimal::Decimal;
use std::collections::HashMap;
use tower::ServiceExt;

fn test_config() -> AppConfig {
    AppConfig {
        host: "127.0.0.1".to_string(),
        port: 0,
        api_key: "test-binance-key".to_string(),
        default_balance: Decimal::new(5_000, 0),
        balances_by_currency: HashMap::new(),
    }
}

#[tokio::test]
async fn spot_balance_requires_api_key() {
    let app = build_app(test_config());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/internal/v1/spot/balance?currency=USDC")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn spot_balance_returns_configured_amount() {
    let app = build_app(test_config());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/internal/v1/spot/balance?currency=USDC")
                .header("x-api-key", "test-binance-key")
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
    assert_eq!(json["available"], "5000");
}
