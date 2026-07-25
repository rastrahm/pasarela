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
}

impl AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::UnsupportedCurrency => StatusCode::BAD_REQUEST,
        }
    }

    fn error_code(&self) -> &'static str {
        match self {
            Self::Unauthorized => "UNAUTHORIZED",
            Self::UnsupportedCurrency => "UNSUPPORTED_CURRENCY",
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
