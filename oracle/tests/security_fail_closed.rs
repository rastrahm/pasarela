//! Tests de seguridad ampliados — allowlist, rate limit y fail closed (Plan 2.9).

mod common;

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use oracle_authorization::antifraud_client::{MockAntifraudClient, MockBehavior};
use oracle_authorization::funds::FundingType;
use oracle_authorization::rail_adapters::MockRailBalanceProvider;
use serial_test::serial;
use tower::ServiceExt;

const TEST_PAN: &str = "4111111111111111";

async fn assert_error_code(response: axum::http::Response<Body>, expected: &str) {
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
    assert_eq!(json["error_code"], expected);
    assert!(json.get("message").and_then(|m| m.as_str()).is_some());
}

#[serial]
#[tokio::test]
async fn cidr_allowlist_accepts_ip_in_docker_prefix() {
    let app =
        common::setup_app_with_config(common::test_config_with_cidr_allowlist()).await;

    let response = common::post_authorize(
        &app,
        "f10e8400-e29b-41d4-a716-446655440001",
        50.0,
        Some("test-secret-key"),
        Some("172.17.0.5"),
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
}

#[serial]
#[tokio::test]
async fn unauthorized_authorize_does_not_create_hold() {
    let config = common::test_config();
    let (app, pool) = common::setup_app_with_clients_and_pool(
        Arc::new(MockAntifraudClient::approve()),
        Arc::new(MockRailBalanceProvider::from_config(&config)),
        config,
    )
    .await;

    let response = common::post_authorize(
        &app,
        "f20e8400-e29b-41d4-a716-446655440001",
        100.0,
        None,
        None,
    )
    .await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_error_code(response, "UNAUTHORIZED").await;

    assert_eq!(common::count_active_holds(&pool).await, 0);
    assert_eq!(common::count_holds(&pool).await, 0);
}

#[serial]
#[tokio::test]
async fn forbidden_ip_does_not_create_hold() {
    let config = common::test_config();
    let (app, pool) = common::setup_app_with_clients_and_pool(
        Arc::new(MockAntifraudClient::approve()),
        Arc::new(MockRailBalanceProvider::from_config(&config)),
        config,
    )
    .await;

    let response = common::post_authorize(
        &app,
        "f30e8400-e29b-41d4-a716-446655440001",
        100.0,
        Some("test-secret-key"),
        Some("10.0.0.99"),
    )
    .await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_error_code(response, "FORBIDDEN").await;

    assert_eq!(common::count_active_holds(&pool).await, 0);
    assert_eq!(common::count_holds(&pool).await, 0);
}

#[serial]
#[tokio::test]
async fn release_route_requires_api_key() {
    let app = common::setup_app().await;

    let body = common::sample_release_body("550e8400-e29b-41d4-a716-446655440099");

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/internal/v1/hold/release")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_error_code(response, "UNAUTHORIZED").await;
}

#[serial]
#[tokio::test]
async fn invalid_card_does_not_create_hold() {
    let config = common::test_config();
    let (app, pool) = common::setup_app_with_clients_and_pool(
        Arc::new(MockAntifraudClient::approve()),
        Arc::new(MockRailBalanceProvider::from_config(&config)),
        config,
    )
    .await;

    let body = common::sample_authorize_body_with_pan(
        "f40e8400-e29b-41d4-a716-446655440001",
        100.0,
        "4111111111111112",
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

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_error_code(response, "INVALID_CARD").await;

    assert_eq!(common::count_holds(&pool).await, 0);
}

#[serial]
#[tokio::test]
async fn insufficient_funds_does_not_create_active_hold() {
    let config = common::test_config();
    let (app, pool) = common::setup_app_with_clients_and_pool(
        Arc::new(MockAntifraudClient::approve()),
        Arc::new(MockRailBalanceProvider::from_config(&config)),
        config,
    )
    .await;

    let response = common::post_authorize(
        &app,
        "f50e8400-e29b-41d4-a716-446655440001",
        999_999.0,
        Some("test-secret-key"),
        None,
    )
    .await;

    assert_eq!(response.status(), StatusCode::PAYMENT_REQUIRED);
    assert_error_code(response, "INSUFFICIENT_FUNDS").await;

    assert_eq!(common::count_active_holds(&pool).await, 0);
}

#[serial]
#[tokio::test]
async fn antifraud_decline_does_not_create_active_hold() {
    let config = common::test_config();
    let (app, pool) = common::setup_app_with_clients_and_pool(
        Arc::new(MockAntifraudClient::with_behavior(MockBehavior::Decline)),
        Arc::new(MockRailBalanceProvider::from_config(&config)),
        config,
    )
    .await;

    let response = common::post_authorize(
        &app,
        "f60e8400-e29b-41d4-a716-446655440001",
        100.0,
        Some("test-secret-key"),
        None,
    )
    .await;

    assert_eq!(response.status(), StatusCode::PAYMENT_REQUIRED);
    assert_error_code(response, "FRAUD_DECLINED").await;

    assert_eq!(common::count_active_holds(&pool).await, 0);
}

#[serial]
#[tokio::test]
async fn rail_unavailable_does_not_create_active_hold() {
    let config = common::test_config();
    let rail = Arc::new(
        MockRailBalanceProvider::from_config(&config)
            .with_unavailable_rail(FundingType::TraditionalBank),
    );

    let (app, pool) = common::setup_app_with_clients_and_pool(
        Arc::new(MockAntifraudClient::approve()),
        rail,
        config,
    )
    .await;

    let response = common::post_authorize(
        &app,
        "f70e8400-e29b-41d4-a716-446655440001",
        100.0,
        Some("test-secret-key"),
        None,
    )
    .await;

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_error_code(response, "RAIL_UNAVAILABLE").await;

    assert_eq!(common::count_active_holds(&pool).await, 0);
}

#[serial]
#[tokio::test]
async fn rate_limit_blocks_authorize_without_creating_hold() {
    let config = common::test_config_with_rate_limit(1);
    let (app, pool) = common::setup_app_with_clients_and_pool(
        Arc::new(MockAntifraudClient::approve()),
        Arc::new(MockRailBalanceProvider::from_config(&config)),
        config,
    )
    .await;

    let first = common::post_authorize(
        &app,
        "f80e8400-e29b-41d4-a716-446655440001",
        10.0,
        Some("test-secret-key"),
        None,
    )
    .await;
    assert_eq!(first.status(), StatusCode::OK);

    let second = common::post_authorize(
        &app,
        "f81e8400-e29b-41d4-a716-446655440001",
        10.0,
        Some("test-secret-key"),
        None,
    )
    .await;
    assert_eq!(second.status(), StatusCode::TOO_MANY_REQUESTS);
    assert_error_code(second, "TOO_MANY_REQUESTS").await;

    assert_eq!(common::count_active_holds(&pool).await, 1);
}

#[serial]
#[tokio::test]
async fn approved_authorization_persists_token_hash_not_pan() {
    let config = common::test_config();
    let (app, pool) = common::setup_app_with_clients_and_pool(
        Arc::new(MockAntifraudClient::approve()),
        Arc::new(MockRailBalanceProvider::from_config(&config)),
        config,
    )
    .await;

    let gateway_id = "f90e8400-e29b-41d4-a716-446655440001";
    let response = common::post_authorize(
        &app,
        gateway_id,
        80.0,
        Some("test-secret-key"),
        None,
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);

    let token_hash: String = sqlx::query_scalar(
        "SELECT card_token_hash FROM authorization_request WHERE gateway_request_id = $1::uuid",
    )
    .bind(gateway_id)
    .fetch_one(&pool)
    .await
    .expect("auth request");

    assert!(token_hash.starts_with("tok_"));
    assert!(!token_hash.contains(TEST_PAN));

    let pan_in_db: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM authorization_request WHERE card_token_hash LIKE $1",
    )
    .bind(format!("%{TEST_PAN}%"))
    .fetch_one(&pool)
    .await
    .expect("count pan");
    assert_eq!(pan_in_db.0, 0);
}
