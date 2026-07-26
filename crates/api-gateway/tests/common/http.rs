//! Helpers HTTP para tests de integración del Gateway.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use api_gateway::services::IDEMPOTENCY_KEY_HEADER;

use super::fixtures::{checkout_no_rail as fixture_checkout_no_rail, checkout_payload as fixture_checkout_payload};
use super::{bearer_header, TEST_API_KEY};

/// Payload JSON de checkout con riel explícito (fixture canónico).
pub fn checkout_payload(funding_type: &str) -> String {
    fixture_checkout_payload(funding_type)
}

/// Payload JSON de checkout sin preferencia de riel.
pub fn checkout_payload_no_rail() -> String {
    fixture_checkout_no_rail()
}

/// Ejecuta `POST /api/v1/checkout` y devuelve status + JSON.
pub async fn post_checkout(
    app: &axum::Router,
    body: &str,
    idempotency_key: &str,
) -> (StatusCode, serde_json::Value) {
    let mut builder = Request::builder()
        .method("POST")
        .uri("/api/v1/checkout")
        .header("content-type", "application/json")
        .header("x-forwarded-for", "127.0.0.1")
        .header(IDEMPOTENCY_KEY_HEADER, idempotency_key);
    let (auth_key, auth_value) = bearer_header();
    builder = builder.header(auth_key, auth_value);

    let response = app
        .clone()
        .oneshot(
            builder
                .body(Body::from(body.to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    parse_json_response(response).await
}

/// Ejecuta checkout sin autenticación (tests de auth).
pub async fn post_checkout_unauthenticated(
    app: &axum::Router,
    body: &str,
    idempotency_key: &str,
) -> StatusCode {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/checkout")
                .header("content-type", "application/json")
                .header(IDEMPOTENCY_KEY_HEADER, idempotency_key)
                .body(Body::from(body.to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    response.status()
}

/// Ejecuta checkout sin `Idempotency-Key` (tests D9).
pub async fn post_checkout_without_idempotency(app: &axum::Router, body: &str) -> StatusCode {
    let (auth_key, auth_value) = bearer_header();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/checkout")
                .header("content-type", "application/json")
                .header(auth_key, auth_value)
                .body(Body::from(body.to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    response.status()
}

/// Ejecuta `GET /api/v1/transactions/{id}`.
pub async fn get_transaction(
    app: &axum::Router,
    transaction_id: &str,
) -> (StatusCode, serde_json::Value) {
    let (auth_key, auth_value) = bearer_header();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/transactions/{transaction_id}"))
                .header(auth_key, auth_value)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    parse_json_response(response).await
}

/// Ejecuta `GET /health`.
pub async fn get_health(app: &axum::Router) -> (StatusCode, serde_json::Value) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    parse_json_response(response).await
}

/// Header Authorization con API key inválida.
pub fn invalid_bearer_header() -> (&'static str, String) {
    ("authorization", format!("Bearer sk_test_unknown1"))
}

/// API key válida de tests (re-export conveniencia).
pub fn valid_api_key() -> &'static str {
    TEST_API_KEY
}

async fn parse_json_response(
    response: axum::response::Response,
) -> (StatusCode, serde_json::Value) {
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
