//! Tests de integración — healthcheck público.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use oracle_authorization::build_app;
use oracle_authorization::config::AppConfig;
use tower::ServiceExt;

fn test_config() -> AppConfig {
    AppConfig {
        host: "127.0.0.1".to_string(),
        port: 0,
        api_key: "test-secret-key".to_string(),
        allowed_callers: vec![oracle_authorization::config::CallerRule::Exact(
            "127.0.0.1".parse().expect("ip"),
        )],
        rate_limit_per_minute: 100,
        rail_timeout_secs: 5,
        hold_ttl_secs: 300,
    }
}

#[tokio::test]
async fn health_returns_ok_without_auth() {
    let app = build_app(test_config());

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

    let body = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();

    let json: serde_json::Value = serde_json::from_slice(&body).expect("json");
    assert_eq!(json["status"], "ok");
    assert_eq!(json["service"], "oracle-authorization");
}
