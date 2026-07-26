//! Tests de integración del mapeo HTTP §6.2 (401, 402, 422, 503, 500).

use std::sync::Arc;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Json, Router};
use domain::{FundingType, LiquidityError, MerchantId, SettlementReceipt};
use http_body_util::BodyExt;
use oracle_client::{error_codes, AuthorizeResponse, ErrorResponse};
use rail_switcher::{RailAvailability, RailSwitcher};
use settlement_adapters::{SettlementAdapter, SettlementContext, SettlementEngine};
use tokio::net::TcpListener;
use tower::ServiceExt;
use uuid::Uuid;

use api_gateway::{
    build_app,
    config::AppConfig,
    error::codes,
    services::{rails::default_rail_configs, RailContext},
    state::AppState,
};

struct FailingSettlementAdapter;

#[async_trait]
impl SettlementAdapter for FailingSettlementAdapter {
    fn rail(&self) -> FundingType {
        FundingType::TraditionalBank
    }

    async fn settle(
        &self,
        _context: SettlementContext,
    ) -> Result<SettlementReceipt, LiquidityError> {
        Err(LiquidityError::SettlementFailed)
    }
}

async fn spawn_oracle(status: StatusCode, error: Option<ErrorResponse>) -> String {
    let app = Router::new().route(
        "/internal/v1/authorize",
        post(move || {
            let error = error.clone();
            async move {
                if let Some(body) = error {
                    (status, Json(body)).into_response()
                } else {
                    (
                        StatusCode::OK,
                        Json(AuthorizeResponse {
                            hold_id: Uuid::new_v4(),
                            brand: "visa".to_string(),
                            brand_code: 1,
                            last_four: "1111".to_string(),
                            gateway_request_id: Uuid::new_v4(),
                        }),
                    )
                        .into_response()
                }
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

fn base_config(oracle_url: String) -> Arc<AppConfig> {
    Arc::new(AppConfig {
        host: "127.0.0.1".to_string(),
        port: 8080,
        oracle_base_url: oracle_url,
        oracle_api_key: "test-key".to_string(),
        oracle_timeout_secs: 2,
        oracle_health_check: false,
        default_merchant_id: MerchantId::new(Uuid::new_v4()),
        merchant_default_funding_type: None,
        rail_fallback_enabled: true,
        rail_configs: default_rail_configs(),
        database_url: None,
    })
}

async fn state_with_parts(
    oracle_url: String,
    rail_context: RailContext,
    settlement_engine: SettlementEngine,
) -> AppState {
    let config = base_config(oracle_url);
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
        settlement_engine,
        RailSwitcher,
        rail_context,
    )
}

async fn checkout(app: axum::Router, body: &str) -> (StatusCode, serde_json::Value) {
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/checkout")
                .header("content-type", "application/json")
                .header("x-forwarded-for", "127.0.0.1")
                .body(Body::from(body.to_string()))
                .expect("request"),
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

const VALID_CHECKOUT: &str = r#"{"amount":100.0,"currency":"USD","funding_type":"traditional_bank","card":{"pan":"4111111111111111","expiry_month":"12","expiry_year":"2030","cvv":"123","cardholder":"Test User"}}"#;

#[tokio::test]
async fn invalid_json_body_returns_422_invalid_request() {
    let oracle_url = spawn_oracle(StatusCode::OK, None).await;
    let app = build_app(
        state_with_parts(oracle_url, RailContext::default(), SettlementEngine::with_stub_adapters())
            .await,
    );

    let (status, json) = checkout(app, "{not-json").await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(json["error_code"], codes::INVALID_REQUEST);
}

#[tokio::test]
async fn oracle_unauthorized_returns_401() {
    let oracle_url = spawn_oracle(
        StatusCode::UNAUTHORIZED,
        Some(ErrorResponse {
            error_code: error_codes::UNAUTHORIZED.to_string(),
            message: "api key inválida".to_string(),
        }),
    )
    .await;
    let app = build_app(
        state_with_parts(oracle_url, RailContext::default(), SettlementEngine::with_stub_adapters())
            .await,
    );

    let (status, json) = checkout(app, VALID_CHECKOUT).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(json["error_code"], codes::UNAUTHORIZED);
}

#[tokio::test]
async fn oracle_invalid_card_returns_422() {
    let oracle_url = spawn_oracle(
        StatusCode::UNPROCESSABLE_ENTITY,
        Some(ErrorResponse {
            error_code: error_codes::INVALID_CARD.to_string(),
            message: "tarjeta inválida".to_string(),
        }),
    )
    .await;
    let app = build_app(
        state_with_parts(oracle_url, RailContext::default(), SettlementEngine::with_stub_adapters())
            .await,
    );

    let (status, json) = checkout(app, VALID_CHECKOUT).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(json["error_code"], codes::INVALID_CARD);
}

#[tokio::test]
async fn oracle_insufficient_funds_returns_402() {
    let oracle_url = spawn_oracle(
        StatusCode::PAYMENT_REQUIRED,
        Some(ErrorResponse {
            error_code: error_codes::INSUFFICIENT_FUNDS.to_string(),
            message: "fondos insuficientes".to_string(),
        }),
    )
    .await;
    let app = build_app(
        state_with_parts(oracle_url, RailContext::default(), SettlementEngine::with_stub_adapters())
            .await,
    );

    let (status, json) = checkout(app, VALID_CHECKOUT).await;

    assert_eq!(status, StatusCode::PAYMENT_REQUIRED);
    assert_eq!(json["error_code"], codes::INSUFFICIENT_FUNDS);
}

#[tokio::test]
async fn no_viable_rail_returns_503() {
    let oracle_url = spawn_oracle(StatusCode::OK, None).await;
    let rail_context = RailContext::default().with_availability(vec![
        RailAvailability {
            rail: FundingType::TraditionalBank,
            operational: false,
            funds_sufficient: true,
        },
        RailAvailability {
            rail: FundingType::BinanceCex,
            operational: false,
            funds_sufficient: true,
        },
        RailAvailability {
            rail: FundingType::SolanaWallet,
            operational: false,
            funds_sufficient: true,
        },
    ]);
    let app = build_app(
        state_with_parts(oracle_url, rail_context, SettlementEngine::with_stub_adapters()).await,
    );

    let (status, json) = checkout(app, VALID_CHECKOUT).await;

    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(json["error_code"], codes::RAIL_UNAVAILABLE);
}

#[tokio::test]
async fn settlement_failure_returns_500() {
    let oracle_url = spawn_oracle(StatusCode::OK, None).await;
    let engine = SettlementEngine::new([Arc::new(FailingSettlementAdapter)
        as Arc<dyn SettlementAdapter>]);
    let app = build_app(
        state_with_parts(oracle_url, RailContext::default(), engine).await,
    );

    let (status, json) = checkout(app, VALID_CHECKOUT).await;

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(json["error_code"], codes::INTERNAL_ERROR);
}

#[tokio::test]
async fn oracle_internal_error_returns_500() {
    let oracle_url = spawn_oracle(
        StatusCode::INTERNAL_SERVER_ERROR,
        Some(ErrorResponse {
            error_code: error_codes::INTERNAL_ERROR.to_string(),
            message: "fallo interno".to_string(),
        }),
    )
    .await;
    let app = build_app(
        state_with_parts(oracle_url, RailContext::default(), SettlementEngine::with_stub_adapters())
            .await,
    );

    let (status, json) = checkout(app, VALID_CHECKOUT).await;

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(json["error_code"], codes::INTERNAL_ERROR);
}

#[tokio::test]
async fn fallback_disabled_and_preferred_lacks_funds_returns_402() {
    let oracle_url = spawn_oracle(StatusCode::OK, None).await;
    let rail_context = RailContext {
        fallback_enabled: false,
        availability: vec![
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
        ],
        ..RailContext::default()
    };
    let app = build_app(
        state_with_parts(oracle_url, rail_context, SettlementEngine::with_stub_adapters()).await,
    );

    let (status, json) = checkout(app, VALID_CHECKOUT).await;

    assert_eq!(status, StatusCode::PAYMENT_REQUIRED);
    assert_eq!(json["error_code"], codes::INSUFFICIENT_FUNDS);
}
