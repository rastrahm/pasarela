//! Tests de integración de idempotencia (D9).

mod common;

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::routing::get;
use axum::{Json, Router};
use http_body_util::BodyExt;
use oracle_client::{
    AuthorizeResponse, HealthResponse, OracleClient, OracleClientError, ReleaseHoldResponse,
};
use tokio::net::TcpListener;
use tower::ServiceExt;
use uuid::Uuid;

use api_gateway::{
    build_app,
    services::{IDEMPOTENCY_KEY_HEADER, RailContext},
    state::AppState,
};
use common::{bearer_header, test_app_config, test_merchant_id, test_merchant_registry};
use rail_switcher::RailSwitcher;
use settlement_adapters::SettlementEngine;

struct CountingOracle {
    calls: Arc<AtomicUsize>,
}

#[async_trait]
impl OracleClient for CountingOracle {
    async fn health(&self) -> Result<HealthResponse, OracleClientError> {
        Ok(HealthResponse {
            status: "ok".to_string(),
            service: "mock".to_string(),
        })
    }

    async fn authorize(
        &self,
        _request: oracle_client::AuthorizeRequest,
        _options: oracle_client::RequestOptions,
    ) -> Result<AuthorizeResponse, OracleClientError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(AuthorizeResponse {
            hold_id: Uuid::new_v4(),
            brand: "visa".to_string(),
            brand_code: 1,
            last_four: "1111".to_string(),
            gateway_request_id: Uuid::new_v4(),
        })
    }

    async fn release_hold_request(
        &self,
        _request: oracle_client::ReleaseHoldRequest,
        _options: oracle_client::RequestOptions,
    ) -> Result<ReleaseHoldResponse, OracleClientError> {
        Ok(ReleaseHoldResponse {
            hold_id: Uuid::new_v4(),
            status: "released".to_string(),
        })
    }
}

async fn spawn_mock_oracle() -> String {
    let app = Router::new().route(
        "/health",
        get(|| async {
            Json(HealthResponse {
                status: "ok".to_string(),
                service: "mock-oracle".to_string(),
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

async fn test_state(calls: Arc<AtomicUsize>) -> AppState {
    let merchant_id = test_merchant_id();
    let mut config = (*test_app_config(spawn_mock_oracle().await, merchant_id)).clone();
    config.oracle_health_check = true;

    AppState::from_parts(
        Arc::new(config),
        Arc::new(CountingOracle { calls }),
        SettlementEngine::with_stub_adapters(),
        RailSwitcher,
        RailContext::default(),
        test_merchant_registry(merchant_id),
    )
}

const VALID_CHECKOUT: &str = r#"{"amount":100.0,"currency":"USD","funding_type":"traditional_bank","card":{"pan":"4111111111111111","expiry_month":"12","expiry_year":"2030","cvv":"123","cardholder":"Test User"}}"#;

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
    let calls = Arc::new(AtomicUsize::new(0));
    let app = build_app(test_state(calls).await);

    let mut builder = Request::builder()
        .method("POST")
        .uri("/api/v1/checkout")
        .header("content-type", "application/json");
    let (auth_key, auth_value) = bearer_header();
    builder = builder.header(auth_key, auth_value);

    let response = app
        .oneshot(
            builder
                .body(Body::from(VALID_CHECKOUT.to_string()))
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
    let calls = Arc::new(AtomicUsize::new(0));
    let app = build_app(test_state(calls.clone()).await);

    let (status1, json1) = post_checkout(app.clone(), Some("idem-001"), VALID_CHECKOUT).await;
    let (status2, json2) = post_checkout(app, Some("idem-001"), VALID_CHECKOUT).await;

    assert_eq!(status1, StatusCode::OK);
    assert_eq!(status2, StatusCode::OK);
    assert_eq!(json1["transaction_id"], json2["transaction_id"]);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn same_idempotency_key_with_different_body_returns_409() {
    let calls = Arc::new(AtomicUsize::new(0));
    let app = build_app(test_state(calls).await);

    let (status1, _) = post_checkout(app.clone(), Some("idem-002"), VALID_CHECKOUT).await;
    assert_eq!(status1, StatusCode::OK);

    let different_body = r#"{"amount":200.0,"currency":"USD","funding_type":"traditional_bank","card":{"pan":"4111111111111111","expiry_month":"12","expiry_year":"2030","cvv":"123","cardholder":"Test User"}}"#;
    let (status2, json2) = post_checkout(app, Some("idem-002"), different_body).await;

    assert_eq!(status2, StatusCode::CONFLICT);
    assert_eq!(json2["error_code"], "CONFLICT");
}
