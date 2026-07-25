//! Errores del cliente antifraude.

use thiserror::Error;

/// Errores al consultar el servicio antifraude.
#[derive(Debug, Error)]
pub enum AntifraudClientError {
    #[error("antifraude rechazó la transacción")]
    Declined,

    #[error("antifraude no disponible: {0}")]
    Unavailable(String),

    #[error("respuesta inválida del antifraude: {0}")]
    InvalidResponse(String),
}
