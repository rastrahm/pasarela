//! Mock HTTP del Oracle para tests de integración del Gateway.

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};

use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use oracle_client::{
    error_codes, AuthorizeRequest, AuthorizeResponse, ErrorResponse, FundingType as OracleFundingType,
    HealthResponse, ReleaseHoldRequest, ReleaseHoldResponse,
};
use tokio::net::TcpListener;
use uuid::Uuid;

/// Métricas capturadas del mock Oracle.
#[derive(Default)]
pub struct OracleCapture {
    pub authorize_calls: AtomicUsize,
    pub release_calls: AtomicUsize,
    pub last_funding_type: Mutex<Option<OracleFundingType>>,
    pub last_hold_id: Mutex<Option<Uuid>>,
    pub released_hold_ids: Mutex<Vec<Uuid>>,
}

/// Opciones al levantar el mock Oracle.
pub struct MockOracleOpts {
    pub hold_id: Uuid,
    pub insufficient_funds: bool,
}

impl Default for MockOracleOpts {
    fn default() -> Self {
        Self {
            hold_id: Uuid::new_v4(),
            insufficient_funds: false,
        }
    }
}

/// Levanta un Oracle simulado con authorize + release + health.
pub async fn spawn_mock_oracle(opts: MockOracleOpts) -> (String, Arc<OracleCapture>) {
    let capture = Arc::new(OracleCapture::default());
    let hold_id = opts.hold_id;

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
                        capture.authorize_calls.fetch_add(1, Ordering::SeqCst);
                        *capture.last_funding_type.lock().expect("lock") = Some(body.funding_type);
                        *capture.last_hold_id.lock().expect("lock") = Some(hold_id);

                        if opts.insufficient_funds {
                            (
                                axum::http::StatusCode::PAYMENT_REQUIRED,
                                Json(ErrorResponse {
                                    error_code: error_codes::INSUFFICIENT_FUNDS.to_string(),
                                    message: "fondos insuficientes".to_string(),
                                }),
                            )
                                .into_response()
                        } else {
                            (
                                axum::http::StatusCode::OK,
                                Json(AuthorizeResponse {
                                    hold_id,
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
            post({
                let capture = capture.clone();
                move |Json(body): Json<ReleaseHoldRequest>| {
                    let capture = capture.clone();
                    async move {
                        capture.release_calls.fetch_add(1, Ordering::SeqCst);
                        capture
                            .released_hold_ids
                            .lock()
                            .expect("lock")
                            .push(body.hold_id);
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

    (format!("http://{addr}"), capture)
}
