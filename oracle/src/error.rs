//! Errores del dominio del Oracle.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
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

    #[error("tarjeta inválida")]
    InvalidCard,

    #[error("fondos insuficientes")]
    InsufficientFunds,

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
            Self::InvalidCard => StatusCode::UNPROCESSABLE_ENTITY,
            Self::InsufficientFunds => StatusCode::PAYMENT_REQUIRED,
            Self::RailUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_code(&self) -> &'static str {
        match self {
            Self::Unauthorized => "UNAUTHORIZED",
            Self::Forbidden => "FORBIDDEN",
            Self::TooManyRequests => "TOO_MANY_REQUESTS",
            Self::InvalidCard => "INVALID_CARD",
            Self::InsufficientFunds => "INSUFFICIENT_FUNDS",
            Self::RailUnavailable => "RAIL_UNAVAILABLE",
            Self::NotFound => "NOT_FOUND",
            Self::Internal(_) => "INTERNAL_ERROR",
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
