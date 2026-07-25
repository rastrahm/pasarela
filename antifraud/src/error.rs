//! Errores HTTP del servicio antifraude.

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
}

impl AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = Json(json!({
            "error_code": "UNAUTHORIZED",
            "message": self.to_string(),
        }));
        (self.status_code(), body).into_response()
    }
}
