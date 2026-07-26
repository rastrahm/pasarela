//! Tests de integración del checkout con Oracle simulado.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::routing::post;
use axum::{Json, Router};
use domain::MerchantId;
use http_body_util::BodyExt;
use oracle_client::AuthorizeResponse;
use tokio::net::TcpListener;
use tower::ServiceExt;
use uuid::Uuid;

use api_gateway::{build_app, config::AppConfig, state::AppState};

async fn spawn_mock_oracle() -> String {
    let hold_id = Uuid::new_v4();
    let gateway_request_id = Uuid::new_v4();

    let app = Router::new().route(
        "/internal/v1/authorize",
        post(move || async move {
            Json(AuthorizeResponse {
                hold_id,
                brand: "visa".to_string(),
                brand_code: 1,
                last_four: "1111".to_string(),
                gateway_request_id,
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

fn test_state(oracle_base_url: String) -> AppState {
    let config = Arc::new(AppConfig {
        host: "127.0.0.1".to_string(),
        port: 8080,
        oracle_base_url,
        oracle_api_key: "test-gateway-key".to_string(),
        oracle_timeout_secs: 2,
        default_merchant_id: MerchantId::new(Uuid::new_v4()),
        database_url: None,
    });
    AppState::new(config).expect("state")
}

#[tokio::test]
async fn checkout_returns_settled_with_mock_oracle() {
    let oracle_url = spawn_mock_oracle().await;
    let app = build_app(test_state(oracle_url));

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/checkout")
                .header("content-type", "application/json")
                .header("x-forwarded-for", "127.0.0.1")
                .body(Body::from(
                    r#"{"amount":100.0,"currency":"USD","funding_type":"traditional_bank","card":{"pan":"4111111111111111","expiry_month":"12","expiry_year":"2030","cvv":"123","cardholder":"Test User"}}"#,
                ))
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
    assert_eq!(json["status"], "settled");
    assert_eq!(json["rail_used"], "traditional_bank");
    assert!(json["settlement_proof"]
        .as_str()
        .expect("proof")
        .starts_with("ACH-"));
}

#[tokio::test]
async fn checkout_rejects_invalid_amount() {
    let app = build_app(test_state(spawn_mock_oracle().await));

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/checkout")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"amount":0,"currency":"USD","card":{"pan":"4111111111111111","expiry_month":"12","expiry_year":"2030","cvv":"123","cardholder":"Test User"}}"#,
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}
