//! Tests de integración HTTP del API Gateway.

use std::sync::Arc;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use api_gateway::{
    build_app,
    config::AppConfig,
    services::{rails::default_rail_configs, RailContext},
    state::AppState,
};
use domain::MerchantId;
use http_body_util::BodyExt;
use oracle_client::{
    AuthorizeResponse, HealthResponse, OracleClient, OracleClientError, ReleaseHoldResponse,
};
use rail_switcher::RailSwitcher;
use settlement_adapters::SettlementEngine;
use tower::ServiceExt;
use uuid::Uuid;

struct StubOracle;

#[async_trait]
impl OracleClient for StubOracle {
    async fn health(&self) -> Result<HealthResponse, OracleClientError> {
        Ok(HealthResponse {
            status: "ok".to_string(),
            service: "stub".to_string(),
        })
    }

    async fn authorize(
        &self,
        _request: oracle_client::AuthorizeRequest,
        _options: oracle_client::RequestOptions,
    ) -> Result<AuthorizeResponse, OracleClientError> {
        Err(OracleClientError::InvalidCard)
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

fn test_state() -> AppState {
    let config = Arc::new(AppConfig {
        host: "127.0.0.1".to_string(),
        port: 8080,
        oracle_base_url: "http://127.0.0.1:8081".to_string(),
        oracle_api_key: "test-gateway-key".to_string(),
        oracle_timeout_secs: 2,
        oracle_health_check: false,
        default_merchant_id: MerchantId::new(Uuid::new_v4()),
        merchant_default_funding_type: None,
        rail_fallback_enabled: true,
        rail_configs: default_rail_configs(),
        database_url: None,
    });

    AppState::from_parts(
        config,
        Arc::new(StubOracle),
        SettlementEngine::with_stub_adapters(),
        RailSwitcher,
        RailContext::default(),
    )
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
