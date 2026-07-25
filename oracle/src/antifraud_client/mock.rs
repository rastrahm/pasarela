//! Cliente mock configurable para tests de integración.

use async_trait::async_trait;

use super::client::AntifraudClient;
use super::error::AntifraudClientError;
use super::types::{ScoreRequest, ScoreResponse};

/// Comportamiento del mock antifraude en tests.
#[derive(Debug, Clone)]
pub enum MockBehavior {
    Approve,
    Decline,
    Unavailable(String),
}

/// Mock del cliente antifraude sin red.
#[derive(Debug, Clone)]
pub struct MockAntifraudClient {
    behavior: MockBehavior,
}

impl MockAntifraudClient {
    /// Crea un mock que siempre aprueba.
    pub fn approve() -> Self {
        Self {
            behavior: MockBehavior::Approve,
        }
    }

    /// Crea un mock con comportamiento explícito.
    pub fn with_behavior(behavior: MockBehavior) -> Self {
        Self { behavior }
    }
}

#[async_trait]
impl AntifraudClient for MockAntifraudClient {
    async fn score(&self, _request: ScoreRequest) -> Result<ScoreResponse, AntifraudClientError> {
        match &self.behavior {
            MockBehavior::Approve => Ok(ScoreResponse::approved()),
            MockBehavior::Decline => Err(AntifraudClientError::Declined),
            MockBehavior::Unavailable(reason) => {
                Err(AntifraudClientError::Unavailable(reason.clone()))
            }
        }
    }
}
