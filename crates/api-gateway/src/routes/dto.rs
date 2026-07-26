//! DTOs HTTP del Gateway — alineados con dominio y contrato API v1.

use domain::{FundingType, TransactionStatus};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Solicitud de checkout (`POST /api/v1/checkout`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CheckoutRequest {
    pub amount: f64,
    pub currency: String,
    pub card: CheckoutCardPayload,
    pub funding_type: Option<FundingType>,
}

/// Datos de tarjeta en checkout (PII en tránsito — no persistir en Gateway).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CheckoutCardPayload {
    pub pan: String,
    pub expiry_month: String,
    pub expiry_year: String,
    pub cvv: String,
    pub cardholder: String,
}

/// Respuesta exitosa de checkout.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CheckoutResponse {
    pub transaction_id: Uuid,
    pub status: TransactionStatus,
    pub rail_used: FundingType,
    pub settlement_proof: Option<String>,
}

/// Respuesta de consulta de transacción (`GET /api/v1/transactions/{id}`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransactionResponse {
    pub transaction_id: Uuid,
    pub status: TransactionStatus,
    pub rail_used: Option<FundingType>,
    pub settlement_proof: Option<String>,
}

/// Healthcheck público del Gateway.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
}
