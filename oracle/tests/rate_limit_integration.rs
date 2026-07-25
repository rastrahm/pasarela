//! Tests de integración — rate limiting por API key + IP (UC-11).

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

#[tokio::test]
async fn internal_route_returns_429_when_rate_limit_exceeded() {
    let app = common::setup_app_with_config(common::test_config_with_rate_limit(5)).await;

    let body = common::sample_release_body("550e8400-e29b-41d4-a716-446655440000");

    for i in 0..5 {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/internal/v1/hold/release")
                    .header("content-type", "application/json")
                    .header("x-api-key", "test-secret-key")
                    .body(Body::from(body.to_string()))
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_ne!(
            response.status(),
            StatusCode::TOO_MANY_REQUESTS,
            "request {i} should pass rate limit"
        );
    }

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/internal/v1/hold/release")
                .header("content-type", "application/json")
                .header("x-api-key", "test-secret-key")
                .body(Body::from(body.to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);

    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
    assert_eq!(json["error_code"], "TOO_MANY_REQUESTS");
}

#[tokio::test]
async fn rate_limit_is_per_api_key_and_ip() {
    let app = common::setup_app_with_config(common::test_config_with_rate_limit(1)).await;

    let body = common::sample_release_body("660e8400-e29b-41d4-a716-446655440000");

    let first = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/internal/v1/hold/release")
                .header("content-type", "application/json")
                .header("x-api-key", "test-secret-key")
                .body(Body::from(body.to_string()))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_ne!(first.status(), StatusCode::TOO_MANY_REQUESTS);

    let second_same_key = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/internal/v1/hold/release")
                .header("content-type", "application/json")
                .header("x-api-key", "test-secret-key")
                .body(Body::from(body.to_string()))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(second_same_key.status(), StatusCode::TOO_MANY_REQUESTS);

    let wrong_key = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/internal/v1/hold/release")
                .header("content-type", "application/json")
                .header("x-api-key", "other-key")
                .body(Body::from(body.to_string()))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(wrong_key.status(), StatusCode::UNAUTHORIZED);
}
