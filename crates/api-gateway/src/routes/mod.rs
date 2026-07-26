//! Rutas HTTP públicas del API Gateway.

mod dto;

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use uuid::Uuid;

use crate::error::GatewayError;
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

/// Checkout completo — orquestador (implementación en paso 4.6).
async fn checkout(
    State(_state): State<Arc<AppState>>,
    Json(_body): Json<CheckoutRequest>,
) -> Result<Json<CheckoutResponse>, GatewayError> {
    Err(GatewayError::NotImplemented("checkout"))
}

/// Consulta de estado de transacción (implementación en paso 4.8).
async fn get_transaction(
    State(_state): State<Arc<AppState>>,
    Path(_id): Path<Uuid>,
) -> Result<Json<TransactionResponse>, GatewayError> {
    Err(GatewayError::NotImplemented("get_transaction"))
}
