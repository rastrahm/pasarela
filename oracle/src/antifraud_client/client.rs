//! Trait del cliente antifraude.

use async_trait::async_trait;

use super::error::AntifraudClientError;
use super::types::{ScoreRequest, ScoreResponse};

/// Cliente HTTP hacia el microservicio `antifraud/` (UC-12).
#[async_trait]
pub trait AntifraudClient: Send + Sync {
    /// Evalúa el riesgo de una transacción antes de crear el hold.
    ///
    /// # Inputs
    /// - `request`: monto, token hash, riel y correlación con el Gateway.
    ///
    /// # Returns
    /// Score aprobado o rechazado; error si el servicio no responde (fail closed).
    async fn score(&self, request: ScoreRequest) -> Result<ScoreResponse, AntifraudClientError>;
}
