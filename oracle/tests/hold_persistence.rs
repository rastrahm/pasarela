//! Tests de persistencia de holds — creación, release e insuficiencia de fondos.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

#[tokio::test]
async fn authorize_persists_hold_and_release_is_idempotent() {
    let app = common::setup_app().await;

    let body = common::sample_authorize_body(
        "660e8400-e29b-41d4-a716-446655440001",
        250.0,
    );

    let auth_response = app
        .clone()
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

    assert_eq!(auth_response.status(), StatusCode::OK);

    let auth_bytes = auth_response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let auth_json: serde_json::Value =
        serde_json::from_slice(&auth_bytes).expect("json");
    let hold_id = auth_json["hold_id"].as_str().expect("hold_id");

    let release_body = serde_json::json!({ "hold_id": hold_id });

    let release_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/internal/v1/hold/release")
                .header("content-type", "application/json")
                .header("x-api-key", "test-secret-key")
                .body(Body::from(release_body.to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(release_response.status(), StatusCode::OK);

    let release_bytes = release_response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let release_json: serde_json::Value =
        serde_json::from_slice(&release_bytes).expect("json");
    assert_eq!(release_json["status"], "released");

    // Segundo release idempotente
    let release_again = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/internal/v1/hold/release")
                .header("content-type", "application/json")
                .header("x-api-key", "test-secret-key")
                .body(Body::from(release_body.to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(release_again.status(), StatusCode::OK);
}

#[tokio::test]
async fn authorize_rejects_insufficient_funds() {
    let app = common::setup_app().await;

    let body = common::sample_authorize_body(
        "770e8400-e29b-41d4-a716-446655440001",
        999_999.0,
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

    assert_eq!(response.status(), StatusCode::PAYMENT_REQUIRED);
}

#[tokio::test]
async fn release_unknown_hold_returns_not_found() {
    let app = common::setup_app().await;

    let release_body =
        serde_json::json!({ "hold_id": "880e8400-e29b-41d4-a716-446655440001" });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/internal/v1/hold/release")
                .header("content-type", "application/json")
                .header("x-api-key", "test-secret-key")
                .body(Body::from(release_body.to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
