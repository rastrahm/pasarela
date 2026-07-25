//! DTOs del contrato HTTP `/internal/v1/*` — compartidos con el Oracle y el Gateway.

use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Payload de tarjeta ficticia enviado por el Gateway (PII en tránsito únicamente).
#[derive(Clone, Deserialize, Serialize, PartialEq)]
pub struct CardPayload {
    pub pan: String,
    pub expiry_month: String,
    pub expiry_year: String,
    pub cvv: String,
    pub cardholder: String,
}

impl fmt::Debug for CardPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CardPayload")
            .field("pan", &"[REDACTED]")
            .field("expiry_month", &self.expiry_month)
            .field("expiry_year", &self.expiry_year)
            .field("cvv", &"[REDACTED]")
            .field("cardholder", &"[REDACTED]")
            .finish()
    }
}

/// Tipo de riel de fondeo / liquidación.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FundingType {
    TraditionalBank,
    BinanceCex,
    SolanaWallet,
}

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

impl std::fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.error_code, self.message)
    }
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
    fn authorize_request_deserializes_from_contract_fixture() {
        let json = include_str!("../tests/fixtures/authorize_request.json");
        let request: AuthorizeRequest = serde_json::from_str(json).expect("deserialize");
        assert_eq!(request.currency, "USD");
        assert_eq!(request.funding_type, FundingType::TraditionalBank);
    }

    #[test]
    fn authorize_response_deserializes_from_contract_fixture() {
        let json = include_str!("../tests/fixtures/authorize_response.json");
        let response: AuthorizeResponse = serde_json::from_str(json).expect("deserialize");
        assert_eq!(response.brand_code, 1);
        assert_eq!(response.last_four, "1111");
    }

    #[test]
    fn card_payload_debug_redacts_pii() {
        let card = CardPayload {
            pan: "4111111111111111".to_string(),
            expiry_month: "12".to_string(),
            expiry_year: "30".to_string(),
            cvv: "123".to_string(),
            cardholder: "Demo".to_string(),
        };
        let rendered = format!("{card:?}");
        assert!(!rendered.contains("4111111111111111"));
        assert!(rendered.contains("[REDACTED]"));
    }
}
