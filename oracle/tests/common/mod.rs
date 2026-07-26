//! Utilidades compartidas para tests de integración con PostgreSQL.

pub use oracle_authorization::test_support::{
    count_active_holds, count_holds, default_test_database_url, postgres_available,
    setup_app, setup_app_with_antifraud, setup_app_with_clients,
    setup_app_with_clients_and_pool, setup_app_with_config, spawn_oracle_server,
    spawn_oracle_server_with_config, test_config, test_config_with_cidr_allowlist,
    test_config_with_rate_limit, test_pool, truncate_test_tables_public, TEST_ORACLE_API_KEY,
};

use axum::body::Body;
use axum::http::Request;
use tower::ServiceExt;

/// Envía POST /internal/v1/authorize con headers opcionales.
pub async fn post_authorize(
    app: &axum::Router,
    gateway_request_id: &str,
    amount: f64,
    api_key: Option<&str>,
    forwarded_for: Option<&str>,
) -> axum::http::Response<axum::body::Body> {
    let body = sample_authorize_body(gateway_request_id, amount);
    let mut builder = Request::builder()
        .method("POST")
        .uri("/internal/v1/authorize")
        .header("content-type", "application/json");

    if let Some(key) = api_key {
        builder = builder.header("x-api-key", key);
    }
    if let Some(ip) = forwarded_for {
        builder = builder.header("x-forwarded-for", ip);
    }

    app.clone()
        .oneshot(
            builder
                .body(Body::from(body.to_string()))
                .expect("request"),
        )
        .await
        .expect("response")
}

pub fn sample_authorize_body_with_pan(
    gateway_request_id: &str,
    amount: f64,
    pan: &str,
) -> serde_json::Value {
    serde_json::json!({
        "gateway_request_id": gateway_request_id,
        "card": {
            "pan": pan,
            "expiry_month": "12",
            "expiry_year": "30",
            "cvv": "123",
            "cardholder": "Demo"
        },
        "amount": amount,
        "currency": "USD",
        "funding_type": "traditional_bank"
    })
}

pub fn sample_authorize_body(gateway_request_id: &str, amount: f64) -> serde_json::Value {
    sample_authorize_body_with_pan(gateway_request_id, amount, "4111111111111111")
}

pub fn sample_release_body(hold_id: &str) -> serde_json::Value {
    serde_json::json!({ "hold_id": hold_id })
}
