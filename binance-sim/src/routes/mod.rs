//! Rutas HTTP del simulador Binance Spot.

use std::sync::Arc;

use axum::{
    extract::{Query, State},
    middleware,
    routing::get,
    Json, Router,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::auth::require_api_key;
use crate::config::AppConfig;
use crate::error::AppError;

/// Crea el router con healthcheck y balance Spot interno protegido.
pub fn create_router(config: Arc<AppConfig>) -> Router {
    let public = Router::new().route("/health", get(health));

    let internal = Router::new()
        .route("/internal/v1/spot/balance", get(spot_balance))
        .layer(middleware::from_fn_with_state(
            config.clone(),
            require_api_key,
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

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "binance-sim-service",
    })
}

#[derive(Debug, Deserialize)]
struct BalanceQuery {
    currency: String,
}

#[derive(Debug, Serialize)]
struct BalanceResponse {
    currency: String,
    available: Decimal,
}

/// Retorna saldo Spot simulado para el Oracle (UC-04, riel BinanceCex).
async fn spot_balance(
    State(config): State<Arc<AppConfig>>,
    Query(query): Query<BalanceQuery>,
) -> Result<Json<BalanceResponse>, AppError> {
    let normalized = query.currency.to_ascii_uppercase();
    if !matches!(normalized.as_str(), "USD" | "USDC" | "USDT") {
        return Err(AppError::UnsupportedCurrency);
    }

    let available = config.balance_for(&normalized);

    tracing::info!(currency = %normalized, %available, "balance Spot consultado");

    Ok(Json(BalanceResponse {
        currency: normalized,
        available,
    }))
}
