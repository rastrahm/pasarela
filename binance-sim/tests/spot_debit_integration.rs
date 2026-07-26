//! Tests de integración del débito Spot simulado.

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
async fn spot_debit_requires_api_key() {
    let app = build_app(test_config());

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/internal/v1/spot/debit")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"amount":"100","currency":"USDC","client_order_id":"hold-1","spread_buffer_pct":"0.02"}"#,
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn spot_debit_returns_order_id_and_reduces_balance() {
    let app = build_app(test_config());

    let debit = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/internal/v1/spot/debit")
                .header("content-type", "application/json")
                .header("x-api-key", "test-binance-key")
                .body(Body::from(
                    r#"{"amount":"100","currency":"USDC","client_order_id":"hold-1","spread_buffer_pct":"0.02"}"#,
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(debit.status(), StatusCode::OK);

    let bytes = debit
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
    assert_eq!(json["status"], "filled");
    assert!(json["order_id"]
        .as_str()
        .expect("order_id")
        .starts_with("CEX-"));

    let balance = app
        .oneshot(
            Request::builder()
                .uri("/internal/v1/spot/balance?currency=USDC")
                .header("x-api-key", "test-binance-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(balance.status(), StatusCode::OK);

    let bytes = balance
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
    assert_eq!(json["available"], "4900");
}

#[tokio::test]
async fn spot_debit_rejects_when_spread_buffer_blocks_amount() {
    let app = build_app(AppConfig {
        default_balance: Decimal::new(100, 0),
        ..test_config()
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/internal/v1/spot/debit")
                .header("content-type", "application/json")
                .header("x-api-key", "test-binance-key")
                .body(Body::from(
                    r#"{"amount":"100","currency":"USDC","client_order_id":"hold-2","spread_buffer_pct":"0.02"}"#,
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::PAYMENT_REQUIRED);
}
