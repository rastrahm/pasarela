//! Errores HTTP del simulador Binance.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// Errores de la capa HTTP.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("no autorizado")]
    Unauthorized,

    #[error("moneda no soportada")]
    UnsupportedCurrency,

    #[error("fondos insuficientes")]
    InsufficientFunds,

    #[error("monto inválido")]
    InvalidAmount,
}

impl AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::UnsupportedCurrency => StatusCode::BAD_REQUEST,
            Self::InsufficientFunds => StatusCode::PAYMENT_REQUIRED,
            Self::InvalidAmount => StatusCode::UNPROCESSABLE_ENTITY,
        }
    }

    fn error_code(&self) -> &'static str {
        match self {
            Self::Unauthorized => "UNAUTHORIZED",
            Self::UnsupportedCurrency => "UNSUPPORTED_CURRENCY",
            Self::InsufficientFunds => "INSUFFICIENT_FUNDS",
            Self::InvalidAmount => "INVALID_AMOUNT",
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = Json(json!({
            "error_code": self.error_code(),
            "message": self.to_string(),
        }));
        (self.status_code(), body).into_response()
    }
}
