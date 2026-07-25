//! Tests de integración de adapters de riel (Plan 2.6).

mod common;

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::routing::get;
use axum::{Json, Router};
use oracle_authorization::antifraud_client::MockAntifraudClient;
use oracle_authorization::funds::FundingType;
use oracle_authorization::rail_adapters::{CompositeRailProvider, MockRailBalanceProvider};
use oracle_authorization::{build_app, init_database, persistence::AppState};
use rust_decimal::Decimal;
use serde::Deserialize;
use std::str::FromStr;
use tokio::net::TcpListener;
use tower::ServiceExt;

#[tokio::test]
async fn authorize_fail_closed_when_rail_unavailable() {
    let config = common::test_config();
    let rail = Arc::new(
        MockRailBalanceProvider::from_config(&config)
            .with_unavailable_rail(FundingType::TraditionalBank),
    );

    let app = common::setup_app_with_clients(
        Arc::new(MockAntifraudClient::approve()),
        rail,
        config,
    )
    .await;

    let body = common::sample_authorize_body(
        "aa0e8400-e29b-41d4-a716-446655440001",
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

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn authorize_with_binance_rail_uses_http_balance() {
    let binance_url = spawn_mock_binance("3000").await;

    let mut config = common::test_config();
    config.binance_cex_base_url = binance_url;
    config.binance_spread_buffer_pct = Decimal::ZERO;

    let config = Arc::new(config);
    let pool = init_database(&config.database_url)
        .await
        .expect("postgres");
    common::truncate_test_tables_public(&pool).await;

    let rail = Arc::new(
        CompositeRailProvider::new(config.clone()).expect("composite rail"),
    );
    let state = AppState::with_clients(
        config.clone(),
        pool,
        Arc::new(MockAntifraudClient::approve()),
        rail,
    );
    let app = build_app(state);

    let body = serde_json::json!({
        "gateway_request_id": "bb0e8400-e29b-41d4-a716-446655440001",
        "card": {
            "pan": "4111111111111111",
            "expiry_month": "12",
            "expiry_year": "30",
            "cvv": "123",
            "cardholder": "Demo"
        },
        "amount": 2500.0,
        "currency": "USD",
        "funding_type": "binance_cex"
    });

    let ok_response = app
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

    assert_eq!(ok_response.status(), StatusCode::OK);

    let reject_body = serde_json::json!({
        "gateway_request_id": "cc0e8400-e29b-41d4-a716-446655440001",
        "card": {
            "pan": "4111111111111111",
            "expiry_month": "12",
            "expiry_year": "30",
            "cvv": "123",
            "cardholder": "Demo"
        },
        "amount": 3001.0,
        "currency": "USD",
        "funding_type": "binance_cex"
    });

    let reject_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/internal/v1/authorize")
                .header("content-type", "application/json")
                .header("x-api-key", "test-secret-key")
                .body(Body::from(reject_body.to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(reject_response.status(), StatusCode::PAYMENT_REQUIRED);
}

async fn spawn_mock_binance(available: &str) -> String {
    #[derive(Deserialize)]
    struct BalanceQuery {
        currency: String,
    }

    #[derive(serde::Serialize)]
    struct BalanceBody {
        currency: String,
        available: Decimal,
    }

    let available = Decimal::from_str(available).expect("decimal");
    let app = Router::new().route(
        "/internal/v1/spot/balance",
        get(move |axum::extract::Query(query): axum::extract::Query<BalanceQuery>| {
            let available = available;
            async move {
                Json(BalanceBody {
                    currency: query.currency,
                    available,
                })
            }
        }),
    );

    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("addr");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve");
    });

    format!("http://{addr}")
}
