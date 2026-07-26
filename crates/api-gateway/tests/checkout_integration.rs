//! Tests de integración del checkout con Oracle simulado.

mod common;

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use http_body_util::BodyExt;
use oracle_client::{AuthorizeResponse, HealthResponse};
use tokio::net::TcpListener;
use tower::ServiceExt;
use uuid::Uuid;

use api_gateway::{
    build_app,
    services::{IDEMPOTENCY_KEY_HEADER, RailContext},
    state::AppState,
};
use common::{bearer_header, test_app_config, test_merchant_id, test_merchant_registry, TEST_API_KEY};
use rail_switcher::RailSwitcher;
use settlement_adapters::SettlementEngine;

async fn spawn_mock_oracle() -> String {
    let hold_id = Uuid::new_v4();
    let gateway_request_id = Uuid::new_v4();

    let app = Router::new()
        .route(
            "/health",
            get(|| async {
                Json(HealthResponse {
                    status: "ok".to_string(),
                    service: "mock-oracle".to_string(),
                })
            }),
        )
        .route(
            "/internal/v1/authorize",
            post(move || async move {
                Json(AuthorizeResponse {
                    hold_id,
                    brand: "visa".to_string(),
                    brand_code: 1,
                    last_four: "1111".to_string(),
                    gateway_request_id,
                })
            }),
        );

    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("addr");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve");
    });

    format!("http://{addr}")
}

async fn test_state(oracle_base_url: String) -> AppState {
    let merchant_id = test_merchant_id();
    let config = test_app_config(oracle_base_url, merchant_id);
    let oracle_client = Arc::new(
        oracle_client::HttpOracleClient::new(
            config.oracle_base_url.clone(),
            config.oracle_api_key.clone(),
            config.oracle_timeout_secs,
        )
        .expect("client"),
    );

    AppState::from_parts(
        config,
        oracle_client,
        SettlementEngine::with_stub_adapters(),
        RailSwitcher,
        RailContext::default(),
        test_merchant_registry(merchant_id),
    )
}

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
    let app = build_app(test_state(spawn_mock_oracle().await).await);

    let response = app
        .oneshot(checkout_request(
            r#"{"amount":100.0,"currency":"USD","funding_type":"traditional_bank","card":{"pan":"4111111111111111","expiry_month":"12","expiry_year":"2030","cvv":"123","cardholder":"Test User"}}"#,
            "checkout-int-001",
        ))
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
    let app = build_app(test_state(spawn_mock_oracle().await).await);

    let checkout_response = app
        .clone()
        .oneshot(checkout_request(
            r#"{"amount":100.0,"currency":"USD","funding_type":"traditional_bank","card":{"pan":"4111111111111111","expiry_month":"12","expiry_year":"2030","cvv":"123","cardholder":"Test User"}}"#,
            "checkout-int-get-tx",
        ))
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
    let app = build_app(test_state(spawn_mock_oracle().await).await);

    let response = app
        .oneshot(checkout_request(
            r#"{"amount":0,"currency":"USD","card":{"pan":"4111111111111111","expiry_month":"12","expiry_year":"2030","cvv":"123","cardholder":"Test User"}}"#,
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
    let app = build_app(test_state(spawn_mock_oracle().await).await);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/checkout")
                .header("content-type", "application/json")
                .header(IDEMPOTENCY_KEY_HEADER, "no-auth")
                .body(Body::from(
                    r#"{"amount":100.0,"currency":"USD","card":{"pan":"4111111111111111","expiry_month":"12","expiry_year":"2030","cvv":"123","cardholder":"Test User"}}"#,
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn checkout_rejects_invalid_api_key() {
    let app = build_app(test_state(spawn_mock_oracle().await).await);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/checkout")
                .header("content-type", "application/json")
                .header("authorization", "Bearer sk_test_unknown1")
                .header(IDEMPOTENCY_KEY_HEADER, "bad-auth")
                .body(Body::from(
                    r#"{"amount":100.0,"currency":"USD","card":{"pan":"4111111111111111","expiry_month":"12","expiry_year":"2030","cvv":"123","cardholder":"Test User"}}"#,
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}