//! Errores tipados del cliente Oracle.

use thiserror::Error;

use crate::dto::ErrorResponse;

/// Error al invocar el Oracle desde el Gateway.
#[derive(Debug, Error, PartialEq)]
pub enum OracleClientError {
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

    #[error("transacción rechazada por antifraude")]
    FraudDeclined,

    #[error("riel no disponible")]
    RailUnavailable,

    #[error("recurso no encontrado")]
    NotFound,

    #[error("conflicto de estado")]
    Conflict,

    #[error("error interno del Oracle")]
    InternalError,

    #[error("Oracle no disponible: {0}")]
    Unavailable(String),

    #[error("respuesta inválida del Oracle: {0}")]
    InvalidResponse(String),

    #[error("{0}")]
    Api(ErrorResponse),
}

impl OracleClientError {
    /// Mapea un código HTTP y cuerpo de error al enum tipado.
    pub fn from_http_status(status: reqwest::StatusCode, body: Option<ErrorResponse>) -> Self {
        if let Some(body) = body {
            if let Some(mapped) = map_error_code(&body.error_code) {
                return mapped;
            }
            return Self::Api(body);
        }

        match status.as_u16() {
            401 => Self::Unauthorized,
            403 => Self::Forbidden,
            429 => Self::TooManyRequests,
            422 => Self::InvalidCard,
            402 => Self::InsufficientFunds,
            503 => Self::RailUnavailable,
            404 => Self::NotFound,
            409 => Self::Conflict,
            _ if status.is_server_error() => Self::InternalError,
            _ => Self::InvalidResponse(format!("status {}", status)),
        }
    }
}

fn map_error_code(code: &str) -> Option<OracleClientError> {
    use crate::dto::error_codes;

    match code {
        error_codes::UNAUTHORIZED => Some(OracleClientError::Unauthorized),
        error_codes::FORBIDDEN => Some(OracleClientError::Forbidden),
        error_codes::TOO_MANY_REQUESTS => Some(OracleClientError::TooManyRequests),
        error_codes::INVALID_CARD => Some(OracleClientError::InvalidCard),
        error_codes::INSUFFICIENT_FUNDS => Some(OracleClientError::InsufficientFunds),
        error_codes::FRAUD_DECLINED => Some(OracleClientError::FraudDeclined),
        error_codes::RAIL_UNAVAILABLE => Some(OracleClientError::RailUnavailable),
        error_codes::NOT_FOUND => Some(OracleClientError::NotFound),
        error_codes::CONFLICT => Some(OracleClientError::Conflict),
        error_codes::INTERNAL_ERROR => Some(OracleClientError::InternalError),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_insufficient_funds_error_code() {
        let err = OracleClientError::from_http_status(
            reqwest::StatusCode::PAYMENT_REQUIRED,
            Some(ErrorResponse {
                error_code: "INSUFFICIENT_FUNDS".to_string(),
                message: "fondos insuficientes".to_string(),
            }),
        );
        assert_eq!(err, OracleClientError::InsufficientFunds);
    }

    #[test]
    fn maps_unknown_error_code_to_api_wrapper() {
        let body = ErrorResponse {
            error_code: "CUSTOM".to_string(),
            message: "detalle".to_string(),
        };
        let err = OracleClientError::from_http_status(reqwest::StatusCode::BAD_REQUEST, Some(body.clone()));
        assert_eq!(err, OracleClientError::Api(body));
    }
}
