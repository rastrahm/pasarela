//! Errores HTTP del Gateway — alineados con Arquitectura §6.2.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use domain::{LiquidityError, RailError};
use oracle_client::{error_codes, OracleClientError};
use serde::Serialize;
use thiserror::Error;

/// Códigos de error expuestos al comercio / frontend.
pub mod codes {
    pub const UNAUTHORIZED: &str = "UNAUTHORIZED";
    pub const INSUFFICIENT_FUNDS: &str = "INSUFFICIENT_FUNDS";
    pub const INVALID_CARD: &str = "INVALID_CARD";
    pub const INVALID_REQUEST: &str = "INVALID_REQUEST";
    pub const CONFLICT: &str = "CONFLICT";
    pub const NOT_FOUND: &str = "NOT_FOUND";
    pub const RAIL_UNAVAILABLE: &str = "RAIL_UNAVAILABLE";
    pub const INTERNAL_ERROR: &str = "INTERNAL_ERROR";
}

/// Cuerpo de error JSON expuesto al comercio / frontend.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ErrorResponse {
    pub error_code: String,
    pub message: String,
}

/// Errores de la capa HTTP del Gateway.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum GatewayError {
    #[error("no autorizado")]
    Unauthorized,

    #[error("fondos insuficientes")]
    InsufficientFunds,

    #[error("tarjeta inválida")]
    InvalidCard,

    #[error("solicitud inválida")]
    InvalidRequest,

    #[error("Idempotency-Key requerido")]
    MissingIdempotencyKey,

    #[error("conflicto: {0}")]
    Conflict(String),

    #[error("recurso no encontrado")]
    NotFound,

    #[error("riel no disponible")]
    RailUnavailable,

    #[error("endpoint pendiente de implementación: {0}")]
    NotImplemented(&'static str),

    #[error("error interno: {0}")]
    Internal(String),
}

impl GatewayError {
    /// Código HTTP según §6.2.
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::InsufficientFunds => StatusCode::PAYMENT_REQUIRED,
            Self::InvalidCard | Self::InvalidRequest | Self::MissingIdempotencyKey => {
                StatusCode::UNPROCESSABLE_ENTITY
            }
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::RailUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            Self::NotImplemented(_) => StatusCode::NOT_IMPLEMENTED,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Código de negocio para el cuerpo JSON.
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::Unauthorized => codes::UNAUTHORIZED,
            Self::InsufficientFunds => codes::INSUFFICIENT_FUNDS,
            Self::InvalidCard => codes::INVALID_CARD,
            Self::InvalidRequest | Self::MissingIdempotencyKey => codes::INVALID_REQUEST,
            Self::Conflict(_) => codes::CONFLICT,
            Self::NotFound => codes::NOT_FOUND,
            Self::RailUnavailable => codes::RAIL_UNAVAILABLE,
            Self::NotImplemented(_) => "NOT_IMPLEMENTED",
            Self::Internal(_) => codes::INTERNAL_ERROR,
        }
    }

    /// Mapea errores del cliente Oracle al contrato HTTP del Gateway.
    pub fn from_oracle_error(error: OracleClientError) -> Self {
        match error {
            OracleClientError::Unauthorized | OracleClientError::Forbidden => Self::Unauthorized,
            OracleClientError::InvalidCard => Self::InvalidCard,
            OracleClientError::InsufficientFunds | OracleClientError::FraudDeclined => {
                Self::InsufficientFunds
            }
            OracleClientError::RailUnavailable | OracleClientError::Unavailable(_) => {
                Self::RailUnavailable
            }
            OracleClientError::Conflict => Self::Conflict("conflicto en Oracle".to_string()),
            OracleClientError::TooManyRequests => {
                Self::Internal("rate limit Oracle".to_string())
            }
            OracleClientError::NotFound => Self::NotFound,
            OracleClientError::InternalError => Self::Internal(error.to_string()),
            OracleClientError::InvalidResponse(detail) => Self::Internal(detail),
            OracleClientError::Api(body) => from_oracle_error_code(&body.error_code)
                .unwrap_or_else(|| Self::Internal(body.message)),
        }
    }

    /// Mapea errores de liquidez / settlement al contrato HTTP del Gateway.
    pub fn from_liquidity_error(error: LiquidityError) -> Self {
        match error {
            LiquidityError::InsufficientFunds { .. } => Self::InsufficientFunds,
            LiquidityError::RailUnavailable { .. } => Self::RailUnavailable,
            LiquidityError::HoldNotFound
            | LiquidityError::HoldFailed
            | LiquidityError::SettlementFailed => Self::Internal(error.to_string()),
        }
    }

    /// Mapea errores del Rail Switcher al contrato HTTP del Gateway.
    pub fn from_rail_error(error: RailError) -> Self {
        match error {
            RailError::InsufficientFunds { .. } => Self::InsufficientFunds,
            RailError::NoRailAvailable
            | RailError::PreferredUnavailableNoFallback { .. }
            | RailError::RailDisabled { .. }
            | RailError::Unavailable { .. } => Self::RailUnavailable,
        }
    }
}

fn from_oracle_error_code(code: &str) -> Option<GatewayError> {
    match code {
        error_codes::UNAUTHORIZED | error_codes::FORBIDDEN => Some(GatewayError::Unauthorized),
        error_codes::INVALID_CARD => Some(GatewayError::InvalidCard),
        error_codes::INSUFFICIENT_FUNDS | error_codes::FRAUD_DECLINED => {
            Some(GatewayError::InsufficientFunds)
        }
        error_codes::RAIL_UNAVAILABLE => Some(GatewayError::RailUnavailable),
        error_codes::NOT_FOUND => Some(GatewayError::NotFound),
        error_codes::CONFLICT => Some(GatewayError::Conflict("conflicto en Oracle".to_string())),
        error_codes::INTERNAL_ERROR => Some(GatewayError::Internal("error interno Oracle".to_string())),
        _ => None,
    }
}

impl IntoResponse for GatewayError {
    fn into_response(self) -> Response {
        let body = Json(ErrorResponse {
            error_code: self.error_code().to_string(),
            message: self.to_string(),
        });
        (self.status_code(), body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_checkout_success_codes() {
        assert_eq!(
            GatewayError::InsufficientFunds.status_code(),
            StatusCode::PAYMENT_REQUIRED
        );
        assert_eq!(
            GatewayError::InvalidCard.status_code(),
            StatusCode::UNPROCESSABLE_ENTITY
        );
        assert_eq!(
            GatewayError::Unauthorized.status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            GatewayError::RailUnavailable.status_code(),
            StatusCode::SERVICE_UNAVAILABLE
        );
        assert_eq!(
            GatewayError::Internal("x".to_string()).status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn maps_oracle_unauthorized_to_401() {
        assert_eq!(
            GatewayError::from_oracle_error(OracleClientError::Unauthorized),
            GatewayError::Unauthorized
        );
    }

    #[test]
    fn maps_oracle_invalid_card_to_422() {
        assert_eq!(
            GatewayError::from_oracle_error(OracleClientError::InvalidCard),
            GatewayError::InvalidCard
        );
    }

    #[test]
    fn maps_oracle_fraud_declined_to_402() {
        assert_eq!(
            GatewayError::from_oracle_error(OracleClientError::FraudDeclined),
            GatewayError::InsufficientFunds
        );
    }

    #[test]
    fn maps_oracle_unavailable_to_503() {
        assert_eq!(
            GatewayError::from_oracle_error(OracleClientError::Unavailable("timeout".to_string())),
            GatewayError::RailUnavailable
        );
    }

    #[test]
    fn maps_oracle_internal_to_500() {
        assert_eq!(
            GatewayError::from_oracle_error(OracleClientError::InternalError),
            GatewayError::Internal(OracleClientError::InternalError.to_string())
        );
    }

    #[test]
    fn maps_oracle_api_body_by_error_code() {
        assert_eq!(
            GatewayError::from_oracle_error(OracleClientError::Api(oracle_client::ErrorResponse {
                error_code: error_codes::RAIL_UNAVAILABLE.to_string(),
                message: "riel caído".to_string(),
            })),
            GatewayError::RailUnavailable
        );
    }

    #[test]
    fn maps_liquidity_insufficient_funds_to_402() {
        assert_eq!(
            GatewayError::from_liquidity_error(LiquidityError::InsufficientFunds {
                rail: domain::FundingType::BinanceCex,
            }),
            GatewayError::InsufficientFunds
        );
    }

    #[test]
    fn maps_liquidity_settlement_failed_to_500() {
        assert!(matches!(
            GatewayError::from_liquidity_error(LiquidityError::SettlementFailed),
            GatewayError::Internal(_)
        ));
    }

    #[test]
    fn maps_rail_insufficient_funds_to_402() {
        assert_eq!(
            GatewayError::from_rail_error(RailError::InsufficientFunds {
                rail: domain::FundingType::TraditionalBank,
            }),
            GatewayError::InsufficientFunds
        );
    }

    #[test]
    fn maps_rail_no_viable_to_503() {
        assert_eq!(
            GatewayError::from_rail_error(RailError::NoRailAvailable),
            GatewayError::RailUnavailable
        );
    }
}
