//! Rutas HTTP públicas e internas del Oracle.

use std::sync::Arc;

use axum::{
    middleware,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::require_gateway_auth;
use crate::config::AppConfig;
use crate::error::AppError;
use crate::funds::{create_hold, FundingType};
use crate::validation::{validate_card, CardPayload};

/// Crea el router con healthcheck público e internal API protegida.
pub fn create_router(config: Arc<AppConfig>) -> Router {
    let public = Router::new().route("/health", get(health));

    let internal = Router::new()
        .route("/internal/v1/authorize", post(authorize))
        .route("/internal/v1/hold/release", post(release_hold))
        .layer(middleware::from_fn_with_state(
            config.clone(),
            require_gateway_auth,
        ));

    Router::new()
        .merge(public)
        .merge(internal)
        .with_state(config)
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
}

/// Healthcheck sin autenticación para orquestación de contenedores.
async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "oracle-authorization",
    })
}

#[derive(Debug, Deserialize)]
struct AuthorizeRequest {
    gateway_request_id: Uuid,
    card: CardPayload,
    amount: f64,
    currency: String,
    funding_type: FundingType,
}

#[derive(Debug, Serialize)]
struct AuthorizeResponse {
    hold_id: Uuid,
    brand: String,
    brand_code: u8,
    last_four: String,
    gateway_request_id: Uuid,
}

/// Autoriza tarjeta, evalúa fondos y crea hold (UC-03 + UC-04).
async fn authorize(
    Json(body): Json<AuthorizeRequest>,
) -> Result<Json<AuthorizeResponse>, AppError> {
    let validation = validate_card(&body.card)?;
    let hold = create_hold(body.amount, &body.currency, body.funding_type)?;

    Ok(Json(AuthorizeResponse {
        hold_id: hold.hold_id,
        brand: format!("{:?}", validation.brand).to_lowercase(),
        brand_code: validation.brand_code,
        last_four: validation.last_four,
        gateway_request_id: body.gateway_request_id,
    }))
}

#[derive(Debug, Deserialize)]
struct ReleaseHoldRequest {
    hold_id: Uuid,
}

#[derive(Debug, Serialize)]
struct ReleaseHoldResponse {
    hold_id: Uuid,
    status: &'static str,
}

/// Libera un hold cuando el settlement del Gateway falla.
async fn release_hold(
    Json(body): Json<ReleaseHoldRequest>,
) -> Result<Json<ReleaseHoldResponse>, AppError> {
    // Persistencia de holds: pendiente de Fase 2.
    Ok(Json(ReleaseHoldResponse {
        hold_id: body.hold_id,
        status: "released",
    }))
}
