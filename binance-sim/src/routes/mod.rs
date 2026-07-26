//! Rutas HTTP del simulador Binance Spot.

use std::sync::Arc;

use axum::{
    extract::{Query, State},
    middleware,
    routing::{get, post},
    Json, Router,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::require_api_key;
use crate::error::AppError;
use crate::state::AppState;

/// Crea el router con healthcheck, balance y débito Spot internos protegidos.
pub fn create_router(state: Arc<AppState>) -> Router {
    let public = Router::new().route("/health", get(health));

    let internal = Router::new()
        .route("/internal/v1/spot/balance", get(spot_balance))
        .route("/internal/v1/spot/debit", post(spot_debit))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            require_api_key,
        ));

    Router::new()
        .merge(public)
        .merge(internal)
        .with_state(state)
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
    State(state): State<Arc<AppState>>,
    Query(query): Query<BalanceQuery>,
) -> Result<Json<BalanceResponse>, AppError> {
    let normalized = normalize_currency(&query.currency)?;
    let available = state.balance_for(&normalized);

    tracing::info!(currency = %normalized, %available, "balance Spot consultado");

    Ok(Json(BalanceResponse {
        currency: normalized,
        available,
    }))
}

#[derive(Debug, Deserialize)]
struct DebitRequest {
    amount: Decimal,
    currency: String,
    client_order_id: String,
    spread_buffer_pct: Decimal,
}

#[derive(Debug, Serialize)]
struct DebitResponse {
    order_id: String,
    status: &'static str,
    debited_amount: Decimal,
    currency: String,
}

/// Debita saldo Spot simulado (UC-06, Settlement Engine).
async fn spot_debit(
    State(state): State<Arc<AppState>>,
    Json(request): Json<DebitRequest>,
) -> Result<Json<DebitResponse>, AppError> {
    if request.amount <= Decimal::ZERO {
        return Err(AppError::InvalidAmount);
    }

    if request.client_order_id.trim().is_empty() {
        return Err(AppError::InvalidAmount);
    }

    let normalized = normalize_currency(&request.currency)?;
    let remaining = state
        .try_debit(&normalized, request.amount, request.spread_buffer_pct)
        .ok_or(AppError::InsufficientFunds)?;

    let order_id = format!("CEX-{}", Uuid::new_v4().simple());

    tracing::info!(
        currency = %normalized,
        amount = %request.amount,
        %remaining,
        client_order_id = %request.client_order_id,
        %order_id,
        "débito Spot simulado"
    );

    Ok(Json(DebitResponse {
        order_id,
        status: "filled",
        debited_amount: request.amount,
        currency: normalized,
    }))
}

fn normalize_currency(currency: &str) -> Result<String, AppError> {
    let normalized = currency.to_ascii_uppercase();
    if matches!(normalized.as_str(), "USD" | "USDC" | "USDT") {
        Ok(normalized)
    } else {
        Err(AppError::UnsupportedCurrency)
    }
}
