//! Tests de seguridad — IDOR, PCI idempotency (Fase 6.4).

mod common;

use std::sync::Arc;

use axum::http::StatusCode;
use axum::body::Body;
use axum::http::Request;
use domain::MerchantId;
use tower::ServiceExt;

use api_gateway::services::merchant::ApiKeyMode;
use api_gateway::services::{MerchantRegistry, RailContext};
use api_gateway::{build_app, state::AppState};
use oracle_client::HttpOracleClient;
use rail_switcher::RailSwitcher;
use settlement_adapters::SettlementEngine;

use common::{
    checkout_payload, cross_service_app_config, spawn_mock_oracle, test_merchant_id,
    MockOracleOpts, TEST_API_KEY,
};

async fn post_checkout_as(
    app: &axum::Router,
    api_key: &str,
    body: &str,
    idempotency_key: &str,
) -> (StatusCode, serde_json::Value) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/checkout")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {api_key}"))
                .header("idempotency-key", idempotency_key)
                .header("x-forwarded-for", "127.0.0.1")
                .body(Body::from(body.to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let json: serde_json::Value =
        serde_json::from_slice(&bytes).unwrap_or(serde_json::json!({}));
    (status, json)
}

async fn get_transaction_as(
    app: &axum::Router,
    api_key: &str,
    transaction_id: &str,
) -> StatusCode {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/transactions/{transaction_id}"))
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    response.status()
}

fn build_two_merchant_app(oracle_url: String) -> (axum::Router, MerchantId, MerchantId, String) {
    let merchant_a = test_merchant_id();
    let merchant_b = test_merchant_id();
    let api_key_b = "sk_test_merchant_b1";

    let config = cross_service_app_config(oracle_url.clone(), merchant_a);
    let oracle_client = Arc::new(
        HttpOracleClient::new(
            config.oracle_base_url.clone(),
            config.oracle_api_key.clone(),
            config.oracle_timeout_secs,
        )
        .expect("client"),
    );

    let registry = Arc::new(MerchantRegistry::default());
    registry.register(TEST_API_KEY, merchant_a, ApiKeyMode::Test);
    registry.register(api_key_b, merchant_b, ApiKeyMode::Test);

    let app = build_app(AppState::from_parts(
        config,
        oracle_client,
        SettlementEngine::with_stub_adapters(),
        RailSwitcher,
        RailContext::default(),
        registry,
    ));

    (app, merchant_a, merchant_b, api_key_b.to_string())
}

#[tokio::test]
async fn security_transaction_lookup_denies_cross_merchant_idor() {
    let (oracle_url, _) = spawn_mock_oracle(MockOracleOpts::default()).await;
    let (app, _merchant_a, _merchant_b, api_key_b) = build_two_merchant_app(oracle_url);

    let body = checkout_payload("traditional_bank");
    let (status, checkout) =
        post_checkout_as(&app, TEST_API_KEY, &body, "security-idor-checkout-a").await;

    assert_eq!(status, StatusCode::OK);
    let transaction_id = checkout["transaction_id"]
        .as_str()
        .expect("transaction_id");

    let own_status = get_transaction_as(&app, TEST_API_KEY, transaction_id).await;
    assert_eq!(own_status, StatusCode::OK);

    let other_status = get_transaction_as(&app, &api_key_b, transaction_id).await;
    assert_eq!(
        other_status,
        StatusCode::NOT_FOUND,
        "comercio B no debe leer transacciones de comercio A"
    );
}

#[tokio::test]
async fn security_idempotency_fingerprint_not_stored_in_conflict_message() {
    let (oracle_url, _) = spawn_mock_oracle(MockOracleOpts::default()).await;
    let app = common::build_default_test_app(oracle_url);
    let pan = "4111111111111111";

    let body_a = format!(
        r#"{{"amount":100.0,"currency":"USD","funding_type":"traditional_bank","card":{{"pan":"{pan}","expiry_month":"12","expiry_year":"2030","cvv":"123","cardholder":"Test User"}}}}"#
    );
    let body_b = format!(
        r#"{{"amount":200.0,"currency":"USD","funding_type":"traditional_bank","card":{{"pan":"{pan}","expiry_month":"12","expiry_year":"2030","cvv":"123","cardholder":"Test User"}}}}"#
    );

    let (first_status, _) =
        post_checkout_as(&app, TEST_API_KEY, &body_a, "security-fingerprint-key").await;
    assert_eq!(first_status, StatusCode::OK);

    let (conflict_status, conflict_body) =
        post_checkout_as(&app, TEST_API_KEY, &body_b, "security-fingerprint-key").await;

    assert_eq!(conflict_status, StatusCode::CONFLICT);
    let serialized = conflict_body.to_string();
    assert!(
        !serialized.contains(pan),
        "respuesta de conflicto no debe filtrar PAN"
    );
    assert!(!serialized.contains("123"), "respuesta no debe filtrar CVV");
}
