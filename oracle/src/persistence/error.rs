//! Errores de la capa de persistencia.

use thiserror::Error;

/// Errores al interactuar con el almacén de datos.
#[derive(Debug, Error)]
pub enum StoreError {
    #[error("recurso no encontrado")]
    NotFound,

    #[error("hold no puede liberarse en estado {0:?}")]
    InvalidHoldState(String),

    #[error("error de base de datos: {0}")]
    Database(#[from] sqlx::Error),
}
