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
use crate::services::{
    authenticate_merchant, extract_caller_ip, extract_idempotency_key, lookup_transaction,
    process_checkout_idempotent, CheckoutInput,
};
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
    let merchant = authenticate_merchant(&headers, state.merchant_registry.as_ref())?;
    let idempotency_key = extract_idempotency_key(&headers)?;
    let response = process_checkout_idempotent(
        state.as_ref(),
        merchant.merchant_id,
        idempotency_key,
        CheckoutInput {
            merchant_id: merchant.merchant_id,
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
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<TransactionResponse>, GatewayError> {
    let _merchant = authenticate_merchant(&headers, state.merchant_registry.as_ref())?;
    let response = lookup_transaction(state.as_ref(), id).await?;
    Ok(Json(response))
}
