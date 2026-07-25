//! DTOs del contrato Oracle ↔ Antifraude.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::funds::FundingType;

/// Solicitud de scoring antifraude (UC-12).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScoreRequest {
    pub gateway_request_id: Uuid,
    pub amount: Decimal,
    pub currency: String,
    pub funding_type: FundingType,
    pub token_hash: String,
}

/// Respuesta del servicio antifraude.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScoreResponse {
    pub approved: bool,
    pub score: f64,
    #[serde(default)]
    pub reasons: Vec<String>,
}

impl ScoreResponse {
    /// Respuesta de aprobación para tests y mocks.
    pub fn approved() -> Self {
        Self {
            approved: true,
            score: 0.1,
            reasons: vec![],
        }
    }

    /// Respuesta de rechazo con razones explícitas.
    pub fn declined(reasons: Vec<String>) -> Self {
        Self {
            approved: false,
            score: 0.99,
            reasons,
        }
    }
}
