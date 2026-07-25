//! DTOs del contrato HTTP `/internal/v1/*` — fuente de verdad para `oracle-client`.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::funds::FundingType;
use crate::validation::CardPayload;

/// Solicitud de autorización enviada por el Gateway.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct AuthorizeRequest {
    pub gateway_request_id: Uuid,
    pub card: CardPayload,
    pub amount: f64,
    pub currency: String,
    pub funding_type: FundingType,
}

/// Respuesta exitosa de autorización con hold creado.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct AuthorizeResponse {
    pub hold_id: Uuid,
    pub brand: String,
    pub brand_code: u8,
    pub last_four: String,
    pub gateway_request_id: Uuid,
}

/// Solicitud de liberación de hold.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ReleaseHoldRequest {
    pub hold_id: Uuid,
}

/// Respuesta de liberación de hold.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ReleaseHoldResponse {
    pub hold_id: Uuid,
    pub status: String,
}

/// Respuesta del healthcheck público.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
}

/// Cuerpo estándar de error del Oracle.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ErrorResponse {
    pub error_code: String,
    pub message: String,
}

/// Códigos de error estables del contrato v1.
pub mod error_codes {
    pub const UNAUTHORIZED: &str = "UNAUTHORIZED";
    pub const FORBIDDEN: &str = "FORBIDDEN";
    pub const TOO_MANY_REQUESTS: &str = "TOO_MANY_REQUESTS";
    pub const CONFLICT: &str = "CONFLICT";
    pub const INVALID_CARD: &str = "INVALID_CARD";
    pub const INSUFFICIENT_FUNDS: &str = "INSUFFICIENT_FUNDS";
    pub const FRAUD_DECLINED: &str = "FRAUD_DECLINED";
    pub const RAIL_UNAVAILABLE: &str = "RAIL_UNAVAILABLE";
    pub const NOT_FOUND: &str = "NOT_FOUND";
    pub const INTERNAL_ERROR: &str = "INTERNAL_ERROR";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authorize_request_deserializes_from_contract_example() {
        let json = include_str!("../../tests/fixtures/authorize_request.json");
        let request: AuthorizeRequest = serde_json::from_str(json).expect("deserialize");
        assert_eq!(request.currency, "USD");
        assert_eq!(request.funding_type, FundingType::TraditionalBank);
    }

    #[test]
    fn authorize_response_deserializes_from_contract_example() {
        let json = include_str!("../../tests/fixtures/authorize_response.json");
        let response: AuthorizeResponse = serde_json::from_str(json).expect("deserialize");
        assert_eq!(response.brand_code, 1);
        assert_eq!(response.last_four, "1111");
    }
}
