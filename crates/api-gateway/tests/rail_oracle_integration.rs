//! Integración Rail Switcher + oracle-client — selección de riel y autorización.

mod common;

use std::sync::{Arc, Mutex};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use domain::FundingType;
use http_body_util::BodyExt;
use oracle_client::{
    error_codes, AuthorizeRequest, AuthorizeResponse, ErrorResponse,
    FundingType as OracleFundingType, HealthResponse, ReleaseHoldRequest, ReleaseHoldResponse,
};
use rail_switcher::{RailAvailability, RailSwitcher};
use settlement_adapters::SettlementEngine;
use tokio::net::TcpListener;
use tower::ServiceExt;
use uuid::Uuid;

use api_gateway::{
    build_app,
    services::{RailContext, IDEMPOTENCY_KEY_HEADER},
    state::AppState,
};
use common::{bearer_header, test_app_config, test_merchant_id, test_merchant_registry};

#[derive(Default)]
struct OracleCapture {
    last_funding_type: Mutex<Option<OracleFundingType>>,
}

async fn spawn_oracle(capture: Arc<OracleCapture>, insufficient: bool) -> String {
    let app = Router::new()
        .route(
            "/health",
            get(|| async {
                Json(HealthResponse {
                    status: "ok".to_string(),
                    service: "mock-oracle".to_string(),
                })
            }),
        )
        .route(
            "/internal/v1/authorize",
            post({
                let capture = capture.clone();
                move |Json(body): Json<AuthorizeRequest>| {
                    let capture = capture.clone();
                    async move {
                        *capture.last_funding_type.lock().expect("lock") =
                            Some(body.funding_type);

                        if insufficient {
                            (
                                StatusCode::PAYMENT_REQUIRED,
                                Json(ErrorResponse {
                                    error_code: error_codes::INSUFFICIENT_FUNDS.to_string(),
                                    message: "fondos insuficientes".to_string(),
                                }),
                            )
                                .into_response()
                        } else {
                            (
                                StatusCode::OK,
                                Json(AuthorizeResponse {
                                    hold_id: Uuid::new_v4(),
                                    brand: "visa".to_string(),
                                    brand_code: 1,
                                    last_four: "1111".to_string(),
                                    gateway_request_id: body.gateway_request_id,
                                }),
                            )
                                .into_response()
                        }
                    }
                }
            }),
        )
        .route(
            "/internal/v1/hold/release",
            post(|Json(body): Json<ReleaseHoldRequest>| async move {
                Json(ReleaseHoldResponse {
                    hold_id: body.hold_id,
                    status: "released".to_string(),
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

async fn state_with_rail_context(
    oracle_url: String,
    rail_context: RailContext,
) -> AppState {
    let merchant_id = test_merchant_id();
    let config = test_app_config(oracle_url, merchant_id);
    let oracle_client = Arc::new(
        oracle_client::HttpOracleClient::new(
            config.oracle_base_url.clone(),
            config.oracle_api_key.clone(),
            config.oracle_timeout_secs,
        )
        .expect("client"),
    );

    AppState::from_parts(
        config,
        oracle_client,
        SettlementEngine::with_stub_adapters(),
        RailSwitcher,
        rail_context,
        test_merchant_registry(merchant_id),
    )
}

fn card_payload_json(funding_type: &str) -> String {
    format!(
        r#"{{"amount":100.0,"currency":"USD","funding_type":"{funding_type}","card":{{"pan":"4111111111111111","expiry_month":"12","expiry_year":"2030","cvv":"123","cardholder":"Test User"}}}}"#
    )
}

async fn checkout_json(app: axum::Router, body: String) -> (StatusCode, serde_json::Value) {
    let mut builder = Request::builder()
        .method("POST")
        .uri("/api/v1/checkout")
        .header("content-type", "application/json")
        .header("x-forwarded-for", "127.0.0.1")
        .header(IDEMPOTENCY_KEY_HEADER, "rail-oracle-test-key");
    let (auth_key, auth_value) = bearer_header();
    builder = builder.header(auth_key, auth_value);

    let response = app
        .oneshot(
            builder.body(Body::from(body)).expect("request"),
        )
        .await
        .expect("response");

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

#[tokio::test]
async fn oracle_receives_funding_type_from_rail_selection() {
    let capture = Arc::new(OracleCapture::default());
    let oracle_url = spawn_oracle(capture.clone(), false).await;
    let app = build_app(
        state_with_rail_context(oracle_url, RailContext::default()).await,
    );

    let (status, json) = checkout_json(app, card_payload_json("binance_cex")).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["rail_used"], "binance_cex");
    assert_eq!(
        *capture.last_funding_type.lock().expect("lock"),
        Some(OracleFundingType::BinanceCex)
    );
}

#[tokio::test]
async fn fallback_selects_next_rail_and_authorizes_with_it() {
    let capture = Arc::new(OracleCapture::default());
    let oracle_url = spawn_oracle(capture.clone(), false).await;
    let rail_context = RailContext::default().with_availability(vec![
        RailAvailability {
            rail: FundingType::TraditionalBank,
            operational: true,
            funds_sufficient: false,
        },
        RailAvailability {
            rail: FundingType::BinanceCex,
            operational: true,
            funds_sufficient: true,
        },
        RailAvailability {
            rail: FundingType::SolanaWallet,
            operational: true,
            funds_sufficient: true,
        },
    ]);
    let app = build_app(state_with_rail_context(oracle_url, rail_context).await);

    let (status, json) =
        checkout_json(app, card_payload_json("traditional_bank")).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["rail_used"], "binance_cex");
    assert_eq!(
        *capture.last_funding_type.lock().expect("lock"),
        Some(OracleFundingType::BinanceCex)
    );
}

#[tokio::test]
async fn three_rails_return_distinct_settlement_proofs() {
    let cases = [
        ("traditional_bank", "ACH-"),
        ("binance_cex", "CEX-MEM-"),
        ("solana_wallet", "SOL-MEM-"),
    ];

    for (funding_type, proof_prefix) in cases {
        let capture = Arc::new(OracleCapture::default());
        let oracle_url = spawn_oracle(capture, false).await;
        let app = build_app(
            state_with_rail_context(oracle_url, RailContext::default()).await,
        );

        let (status, json) = checkout_json(app, card_payload_json(funding_type)).await;

        assert_eq!(status, StatusCode::OK, "rail {funding_type}");
        assert_eq!(json["rail_used"], funding_type);
        assert!(
            json["settlement_proof"]
                .as_str()
                .expect("proof")
                .starts_with(proof_prefix),
            "rail {funding_type}"
        );
    }
}

#[tokio::test]
async fn oracle_insufficient_funds_maps_to_402() {
    let capture = Arc::new(OracleCapture::default());
    let oracle_url = spawn_oracle(capture, true).await;
    let app = build_app(
        state_with_rail_context(oracle_url, RailContext::default()).await,
    );

    let (status, json) = checkout_json(app, card_payload_json("traditional_bank")).await;

    assert_eq!(status, StatusCode::PAYMENT_REQUIRED);
    assert_eq!(json["error_code"], "INSUFFICIENT_FUNDS");
}

#[tokio::test]
async fn merchant_default_rail_used_when_checkout_has_no_preference() {
    let capture = Arc::new(OracleCapture::default());
    let oracle_url = spawn_oracle(capture.clone(), false).await;
    let rail_context = RailContext {
        merchant_default: Some(FundingType::SolanaWallet),
        ..RailContext::default()
    };
    let app = build_app(state_with_rail_context(oracle_url, rail_context).await);

    let body = r#"{"amount":100.0,"currency":"USD","card":{"pan":"4111111111111111","expiry_month":"12","expiry_year":"2030","cvv":"123","cardholder":"Test User"}}"#.to_string();
    let (status, json) = checkout_json(app, body).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["rail_used"], "solana_wallet");
    assert_eq!(
        *capture.last_funding_type.lock().expect("lock"),
        Some(OracleFundingType::SolanaWallet)
    );
}
