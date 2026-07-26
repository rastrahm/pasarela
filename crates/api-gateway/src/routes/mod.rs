//! Rutas HTTP públicas del API Gateway.

mod dto;

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::HeaderMap,
    routing::{get, post},
    Json, Router,
};
use uuid::Uuid;

use crate::error::GatewayError;
use crate::services::{extract_caller_ip, lookup_transaction, process_checkout, CheckoutInput};
use crate::state::AppState;

pub use dto::{
    CheckoutCardPayload, CheckoutRequest, CheckoutResponse, HealthResponse, TransactionResponse,
};

/// Crea el router con healthcheck y API v1 del comercio.
pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/v1/checkout", post(checkout))
        .route("/api/v1/transactions/:id", get(get_transaction))
        .with_state(state)
}

/// Healthcheck sin autenticación para orquestación de contenedores.
async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        service: "api-gateway".to_string(),
    })
}

/// Checkout completo — autorización Oracle + liquidación en riel activo (UC-01).
async fn checkout(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Result<Json<CheckoutRequest>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<CheckoutResponse>, GatewayError> {
    let Json(body) = body.map_err(|_| GatewayError::InvalidRequest)?;
    let response = process_checkout(
        state.as_ref(),
        CheckoutInput {
            request: body,
            caller_ip: extract_caller_ip(&headers),
        },
    )
    .await?;

    Ok(Json(response))
}

/// Consulta de estado de transacción (UC-09).
async fn get_transaction(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<TransactionResponse>, GatewayError> {
    let response = lookup_transaction(state.as_ref(), id)?;
    Ok(Json(response))
}
