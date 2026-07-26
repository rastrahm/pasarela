//! Integración UC-04 — liberación de hold en Oracle cuando falla el settlement (paso 4.13).

mod common;

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::routing::post;
use axum::{Json, Router};
use domain::{FundingType, LiquidityError};
use oracle_client::{
    AuthorizeResponse, ReleaseHoldRequest, ReleaseHoldResponse,
};
use rail_switcher::RailSwitcher;
use settlement_adapters::{MockSettlementAdapter, SettlementAdapter, SettlementEngine};
use tokio::net::TcpListener;
use tower::ServiceExt;
use uuid::Uuid;

use api_gateway::{
    build_app,
    services::{IDEMPOTENCY_KEY_HEADER, RailContext},
    state::AppState,
};
use common::{bearer_header, test_app_config, test_merchant_id, test_merchant_registry};

#[derive(Default)]
struct ReleaseCapture {
    calls: AtomicUsize,
    released_hold_id: Mutex<Option<Uuid>>,
}

async fn spawn_oracle_with_release(capture: Arc<ReleaseCapture>, hold_id: Uuid) -> String {
    let app = Router::new()
        .route(
            "/internal/v1/authorize",
            post(move || async move {
                Json(AuthorizeResponse {
                    hold_id,
                    brand: "visa".to_string(),
                    brand_code: 1,
                    last_four: "1111".to_string(),
                    gateway_request_id: Uuid::new_v4(),
                })
            }),
        )
        .route(
            "/internal/v1/hold/release",
            post({
                let capture = capture.clone();
                move |Json(body): Json<ReleaseHoldRequest>| {
                    let capture = capture.clone();
                    async move {
                        capture.calls.fetch_add(1, Ordering::SeqCst);
                        *capture.released_hold_id.lock().expect("lock") = Some(body.hold_id);
                        Json(ReleaseHoldResponse {
                            hold_id: body.hold_id,
                            status: "released".to_string(),
                        })
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

async fn state_with_failing_settlement(oracle_url: String) -> AppState {
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
    let engine = SettlementEngine::new([Arc::new(
        MockSettlementAdapter::new(FundingType::TraditionalBank, "unused")
            .with_error(LiquidityError::SettlementFailed),
    )
        as Arc<dyn SettlementAdapter>]);

    AppState::from_parts(
        config,
        oracle_client,
        engine,
        RailSwitcher,
        RailContext::default(),
        test_merchant_registry(merchant_id),
    )
}

const VALID_CHECKOUT: &str = r#"{"amount":100.0,"currency":"USD","funding_type":"traditional_bank","card":{"pan":"4111111111111111","expiry_month":"12","expiry_year":"2030","cvv":"123","cardholder":"Test User"}}"#;

async fn post_checkout(app: &axum::Router, idempotency_key: &str) -> StatusCode {
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
                .body(Body::from(VALID_CHECKOUT.to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    response.status()
}

#[tokio::test]
async fn settlement_failure_triggers_oracle_hold_release() {
    let hold_id = Uuid::new_v4();
    let capture = Arc::new(ReleaseCapture::default());
    let oracle_url = spawn_oracle_with_release(capture.clone(), hold_id).await;
    let app = build_app(state_with_failing_settlement(oracle_url).await);

    let status = post_checkout(&app, "hold-release-test-key").await;

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(capture.calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        *capture.released_hold_id.lock().expect("lock"),
        Some(hold_id)
    );
}

#[tokio::test]
async fn idempotent_replay_does_not_release_hold_twice() {
    let hold_id = Uuid::new_v4();
    let capture = Arc::new(ReleaseCapture::default());
    let oracle_url = spawn_oracle_with_release(capture.clone(), hold_id).await;
    let app = build_app(state_with_failing_settlement(oracle_url).await);

    assert_eq!(
        post_checkout(&app, "hold-release-replay-key").await,
        StatusCode::INTERNAL_SERVER_ERROR
    );
    assert_eq!(
        post_checkout(&app, "hold-release-replay-key").await,
        StatusCode::INTERNAL_SERVER_ERROR
    );
    assert_eq!(
        capture.calls.load(Ordering::SeqCst),
        1,
        "release solo en el intento original"
    );
}

struct SuccessfulSettlementOracle {
    hold_id: Uuid,
    release_calls: AtomicUsize,
}

#[async_trait]
impl oracle_client::OracleClient for SuccessfulSettlementOracle {
    async fn health(
        &self,
    ) -> Result<oracle_client::HealthResponse, oracle_client::OracleClientError> {
        Ok(oracle_client::HealthResponse {
            status: "ok".to_string(),
            service: "mock".to_string(),
        })
    }

    async fn authorize(
        &self,
        _request: oracle_client::AuthorizeRequest,
        _options: oracle_client::RequestOptions,
    ) -> Result<AuthorizeResponse, oracle_client::OracleClientError> {
        Ok(AuthorizeResponse {
            hold_id: self.hold_id,
            brand: "visa".to_string(),
            brand_code: 1,
            last_four: "1111".to_string(),
            gateway_request_id: Uuid::new_v4(),
        })
    }

    async fn release_hold_request(
        &self,
        _request: ReleaseHoldRequest,
        _options: oracle_client::RequestOptions,
    ) -> Result<ReleaseHoldResponse, oracle_client::OracleClientError> {
        self.release_calls.fetch_add(1, Ordering::SeqCst);
        Ok(ReleaseHoldResponse {
            hold_id: self.hold_id,
            status: "released".to_string(),
        })
    }
}

#[tokio::test]
async fn successful_settlement_does_not_release_hold() {
    let hold_id = Uuid::new_v4();
    let oracle = Arc::new(SuccessfulSettlementOracle {
        hold_id,
        release_calls: AtomicUsize::new(0),
    });
    let merchant_id = test_merchant_id();
    let config = test_app_config("http://mock".to_string(), merchant_id);
    let state = AppState::from_parts(
        config,
        oracle.clone(),
        SettlementEngine::with_stub_adapters(),
        RailSwitcher,
        RailContext::default(),
        test_merchant_registry(merchant_id),
    );
    let app = build_app(state);

    let status = post_checkout(&app, "hold-release-success-key").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(oracle.release_calls.load(Ordering::SeqCst), 0);
}
