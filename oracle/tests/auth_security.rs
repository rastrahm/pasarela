//! Tests de seguridad — rechazo fail closed sin credenciales válidas.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

#[tokio::test]
async fn internal_route_rejects_missing_api_key() {
    let app = common::setup_app().await;

    let body = common::sample_authorize_body(
        "550e8400-e29b-41d4-a716-446655440000",
        100.0,
    );

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
    let app = common::setup_app().await;

    let body = common::sample_authorize_body(
        "550e8400-e29b-41d4-a716-446655440000",
        100.0,
    );

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
    let app = common::setup_app().await;

    let body = common::sample_authorize_body(
        "550e8400-e29b-41d4-a716-446655440001",
        100.0,
    );

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

#[tokio::test]
async fn internal_route_rejects_ip_not_in_allowlist() {
    let app = common::setup_app().await;

    let body = common::sample_release_body("550e8400-e29b-41d4-a716-446655440099");

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/internal/v1/hold/release")
                .header("content-type", "application/json")
                .header("x-api-key", "test-secret-key")
                .header("x-forwarded-for", "10.0.0.99")
                .body(Body::from(body.to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
