//! Errores al consultar saldos en rieles externos.

use thiserror::Error;

/// Fallo al obtener balance de un riel (fail closed en autorización).
#[derive(Debug, Error)]
pub enum RailAdapterError {
    #[error("riel no disponible: {0}")]
    Unavailable(String),

    #[error("respuesta inválida del riel: {0}")]
    InvalidResponse(String),

    #[error("moneda no soportada: {0}")]
    UnsupportedCurrency(String),
}
