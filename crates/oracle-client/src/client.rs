//! Trait del cliente HTTP hacia el Oracle.

use async_trait::async_trait;
use uuid::Uuid;

use crate::dto::{
    AuthorizeRequest, AuthorizeResponse, HealthResponse, ReleaseHoldRequest, ReleaseHoldResponse,
};
use crate::error::OracleClientError;

/// Opciones de transporte para una llamada al Oracle.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RequestOptions {
    /// IP del Gateway para allowlist y rate limiting (`X-Forwarded-For`).
    pub caller_ip: Option<String>,
}

/// Cliente tipado hacia el Oracle de autorización (contrato v1).
#[async_trait]
pub trait OracleClient: Send + Sync {
    /// Consulta el healthcheck público (`GET /health`).
    async fn health(&self) -> Result<HealthResponse, OracleClientError>;

    /// Autoriza tarjeta y crea hold (`POST /internal/v1/authorize`).
    async fn authorize(
        &self,
        request: AuthorizeRequest,
        options: RequestOptions,
    ) -> Result<AuthorizeResponse, OracleClientError>;

    /// Libera un hold (`POST /internal/v1/hold/release`).
    async fn release_hold(
        &self,
        hold_id: Uuid,
        options: RequestOptions,
    ) -> Result<ReleaseHoldResponse, OracleClientError> {
        self.release_hold_request(ReleaseHoldRequest { hold_id }, options)
            .await
    }

    /// Libera un hold con payload explícito.
    async fn release_hold_request(
        &self,
        request: ReleaseHoldRequest,
        options: RequestOptions,
    ) -> Result<ReleaseHoldResponse, OracleClientError>;
}
