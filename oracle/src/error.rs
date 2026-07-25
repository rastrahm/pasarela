//! Errores del dominio del Oracle.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use oracle_client::{error_codes, ErrorResponse};
use thiserror::Error;

/// Errores de la capa de aplicación del Oracle.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("no autorizado")]
    Unauthorized,

    #[error("acceso prohibido")]
    Forbidden,

    #[error("demasiadas solicitudes")]
    TooManyRequests,

    #[error("conflicto de estado: {0}")]
    Conflict(String),

    #[error("tarjeta inválida")]
    InvalidCard,

    #[error("fondos insuficientes")]
    InsufficientFunds,

    #[error("transacción rechazada por antifraude")]
    FraudDeclined,

    #[error("riel no disponible")]
    RailUnavailable,

    #[error("recurso no encontrado")]
    NotFound,

    #[error("error interno: {0}")]
    Internal(String),
}

impl AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::TooManyRequests => StatusCode::TOO_MANY_REQUESTS,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::InvalidCard => StatusCode::UNPROCESSABLE_ENTITY,
            Self::InsufficientFunds => StatusCode::PAYMENT_REQUIRED,
            Self::FraudDeclined => StatusCode::PAYMENT_REQUIRED,
            Self::RailUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_code(&self) -> &'static str {
        match self {
            Self::Unauthorized => error_codes::UNAUTHORIZED,
            Self::Forbidden => error_codes::FORBIDDEN,
            Self::TooManyRequests => error_codes::TOO_MANY_REQUESTS,
            Self::Conflict(_) => error_codes::CONFLICT,
            Self::InvalidCard => error_codes::INVALID_CARD,
            Self::InsufficientFunds => error_codes::INSUFFICIENT_FUNDS,
            Self::FraudDeclined => error_codes::FRAUD_DECLINED,
            Self::RailUnavailable => error_codes::RAIL_UNAVAILABLE,
            Self::NotFound => error_codes::NOT_FOUND,
            Self::Internal(_) => error_codes::INTERNAL_ERROR,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = Json(ErrorResponse {
            error_code: self.error_code().to_string(),
            message: self.to_string(),
        });
        (self.status_code(), body).into_response()
    }
}
