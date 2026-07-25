//! Rutas HTTP públicas e internas del Oracle.

use std::net::IpAddr;
use std::sync::Arc;

use axum::{
    extract::State,
    http::HeaderMap,
    middleware,
    routing::{get, post},
    Json, Router,
};

use crate::api::{
    AuthorizeRequest, AuthorizeResponse, HealthResponse, ReleaseHoldRequest, ReleaseHoldResponse,
};
use crate::auth::{require_gateway_auth, GatewayAuthState};
use crate::error::AppError;
use crate::persistence::models::HoldStatus;
use crate::persistence::AppState;
use crate::services::authorization::{self, AuthorizeInput};

/// Crea el router con healthcheck público e internal API protegida.
pub fn create_router(state: Arc<AppState>) -> Router {
    let auth_state = GatewayAuthState::from_config(state.config.clone());

    let public = Router::new().route("/health", get(health));

    let internal = Router::new()
        .route("/internal/v1/authorize", post(authorize))
        .route("/internal/v1/hold/release", post(release_hold))
        .layer(middleware::from_fn_with_state(
            auth_state,
            require_gateway_auth,
        ));

    Router::new()
        .merge(public)
        .merge(internal)
        .with_state(state)
}

/// Healthcheck sin autenticación para orquestación de contenedores.
async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        service: "oracle-authorization".to_string(),
    })
}

/// Autoriza tarjeta, evalúa fondos y crea hold persistido (UC-03 + UC-04).
async fn authorize(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<AuthorizeRequest>,
) -> Result<Json<AuthorizeResponse>, AppError> {
    let caller_ip = extract_caller_ip(&headers);
    let amount = authorization::decimal_from_f64(body.amount)?;

    let output = authorization::authorize(
        &state,
        AuthorizeInput {
            gateway_request_id: body.gateway_request_id,
            card: body.card,
            amount,
            currency: body.currency,
            funding_type: body.funding_type,
            caller_ip,
        },
    )
    .await?;

    Ok(Json(AuthorizeResponse {
        hold_id: output.hold_id,
        brand: output.brand,
        brand_code: output.brand_code,
        last_four: output.last_four,
        gateway_request_id: output.gateway_request_id,
    }))
}

/// Libera un hold cuando el settlement del Gateway falla.
async fn release_hold(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ReleaseHoldRequest>,
) -> Result<Json<ReleaseHoldResponse>, AppError> {
    let output = authorization::release_hold(&state, body.hold_id).await?;

    Ok(Json(ReleaseHoldResponse {
        hold_id: output.hold_id,
        status: hold_status_label(output.status).to_string(),
    }))
}

fn hold_status_label(status: HoldStatus) -> &'static str {
    match status {
        HoldStatus::Active => "active",
        HoldStatus::Consumed => "consumed",
        HoldStatus::Released => "released",
        HoldStatus::Expired => "expired",
    }
}

fn extract_caller_ip(headers: &HeaderMap) -> Option<String> {
    if let Some(forwarded) = headers.get("x-forwarded-for") {
        if let Ok(value) = forwarded.to_str() {
            if let Some(first) = value.split(',').next() {
                return Some(first.trim().to_string());
            }
        }
    }

    Some(IpAddr::from([127, 0, 0, 1]).to_string())
}
