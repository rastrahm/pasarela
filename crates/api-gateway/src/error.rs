//! Errores HTTP del Gateway — alineados con Arquitectura §6.2.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

/// Cuerpo de error JSON expuesto al comercio / frontend.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ErrorResponse {
    pub error_code: String,
    pub message: String,
}

/// Errores de la capa HTTP del Gateway.
#[derive(Debug, Error)]
pub enum GatewayError {
    #[error("no autorizado")]
    Unauthorized,

    #[error("fondos insuficientes")]
    InsufficientFunds,

    #[error("tarjeta inválida")]
    InvalidCard,

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
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::InsufficientFunds => StatusCode::PAYMENT_REQUIRED,
            Self::InvalidCard => StatusCode::UNPROCESSABLE_ENTITY,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::RailUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            Self::NotImplemented(_) => StatusCode::NOT_IMPLEMENTED,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_code(&self) -> &'static str {
        match self {
            Self::Unauthorized => "UNAUTHORIZED",
            Self::InsufficientFunds => "INSUFFICIENT_FUNDS",
            Self::InvalidCard => "INVALID_CARD",
            Self::Conflict(_) => "CONFLICT",
            Self::NotFound => "NOT_FOUND",
            Self::RailUnavailable => "RAIL_UNAVAILABLE",
            Self::NotImplemented(_) => "NOT_IMPLEMENTED",
            Self::Internal(_) => "INTERNAL_ERROR",
        }
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
    fn insufficient_funds_maps_to_402() {
        assert_eq!(
            GatewayError::InsufficientFunds.status_code(),
            StatusCode::PAYMENT_REQUIRED
        );
    }
}
