//! Errores de persistencia del Gateway.

use thiserror::Error;

/// Error al acceder al almacén del Gateway.
#[derive(Debug, Error)]
pub enum PersistenceError {
    #[error("base de datos: {0}")]
    Database(#[from] sqlx::Error),

    #[error("almacén bloqueado")]
    LockPoisoned,

    #[error("{0}")]
    Internal(String),
}
