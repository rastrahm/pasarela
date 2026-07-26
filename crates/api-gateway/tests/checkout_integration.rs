//! Tests de integración del checkout con Oracle simulado.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use domain::MerchantId;
use http_body_util::BodyExt;
use oracle_client::{AuthorizeResponse, HealthResponse};
use tokio::net::TcpListener;
use tower::ServiceExt;
use uuid::Uuid;

use api_gateway::{build_app, config::AppConfig, services::IDEMPOTENCY_KEY_HEADER, state::AppState};

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

fn test_config(oracle_base_url: String) -> Arc<AppConfig> {
    Arc::new(AppConfig {
        host: "127.0.0.1".to_string(),
        port: 8080,
        oracle_base_url,
        oracle_api_key: "test-gateway-key".to_string(),
        oracle_timeout_secs: 2,
        oracle_health_check: true,
        default_merchant_id: MerchantId::new(Uuid::new_v4()),
        merchant_default_funding_type: None,
        rail_fallback_enabled: true,
        rail_configs: api_gateway::services::rails::default_rail_configs(),
        database_url: None,
    })
}

async fn test_state(oracle_base_url: String) -> AppState {
    AppState::new(test_config(oracle_base_url))
        .await
        .expect("state")
}

#[tokio::test]
async fn checkout_returns_settled_with_mock_oracle() {
    let oracle_url = spawn_mock_oracle().await;
    let app = build_app(test_state(oracle_url).await);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/checkout")
                .header("content-type", "application/json")
                .header("x-forwarded-for", "127.0.0.1")
                .header(IDEMPOTENCY_KEY_HEADER, "checkout-int-001")
                .body(Body::from(
                    r#"{"amount":100.0,"currency":"USD","funding_type":"traditional_bank","card":{"pan":"4111111111111111","expiry_month":"12","expiry_year":"2030","cvv":"123","cardholder":"Test User"}}"#,
                ))
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
    assert_eq!(json["status"], "settled");
    assert_eq!(json["rail_used"], "traditional_bank");
    assert!(json["settlement_proof"]
        .as_str()
        .expect("proof")
        .starts_with("ACH-"));
}

#[tokio::test]
async fn get_transaction_returns_checkout_result() {
    let oracle_url = spawn_mock_oracle().await;
    let state = test_state(oracle_url).await;
    let app = build_app(state);

    let checkout_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/checkout")
                .header("content-type", "application/json")
                .header("x-forwarded-for", "127.0.0.1")
                .header(IDEMPOTENCY_KEY_HEADER, "checkout-int-001")
                .body(Body::from(
                    r#"{"amount":100.0,"currency":"USD","funding_type":"traditional_bank","card":{"pan":"4111111111111111","expiry_month":"12","expiry_year":"2030","cvv":"123","cardholder":"Test User"}}"#,
                ))
                .expect("request"),
        )
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

    let get_response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/transactions/{transaction_id}"))
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
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/checkout")
                .header("content-type", "application/json")
                .header(IDEMPOTENCY_KEY_HEADER, "checkout-int-invalid-amount")
                .body(Body::from(
                    r#"{"amount":0,"currency":"USD","card":{"pan":"4111111111111111","expiry_month":"12","expiry_year":"2030","cvv":"123","cardholder":"Test User"}}"#,
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn startup_fails_when_oracle_health_unreachable() {
    let config = test_config("http://127.0.0.1:1".to_string());
    let result = AppState::new(config).await;
    assert!(result.is_err());
}
