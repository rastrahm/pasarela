//! Trait de persistencia para solicitudes de autorización y audit log.

use async_trait::async_trait;
use uuid::Uuid;

use super::error::StoreError;
use super::models::{AuditLogEntry, AuthorizationRequestRecord, CreateAuthorizationRequest};
use sqlx::PgTransaction;

/// Almacén de solicitudes de autorización y eventos de auditoría.
#[async_trait]
pub trait AuthorizationStore: Send + Sync {
    /// Inserta una solicitud de autorización dentro de una transacción abierta.
    ///
    /// # Inputs
    /// - `tx`: transacción PostgreSQL activa.
    /// - `request`: datos de la solicitud sin PAN.
    ///
    /// # Returns
    /// Registro persistido o error de base de datos.
    async fn create_request(
        &self,
        tx: &mut PgTransaction<'_>,
        request: CreateAuthorizationRequest,
    ) -> Result<AuthorizationRequestRecord, StoreError>;

    /// Registra un evento de auditoría sin PII dentro de una transacción.
    ///
    /// # Inputs
    /// - `tx`: transacción PostgreSQL activa.
    /// - `entry`: evento a registrar.
    ///
    /// # Returns
    /// `Ok(())` si se insertó correctamente.
    async fn append_audit_log(
        &self,
        tx: &mut PgTransaction<'_>,
        entry: AuditLogEntry,
    ) -> Result<(), StoreError>;

    /// Registra auditoría fuera de una transacción de negocio (p. ej. rechazos de auth).
    async fn append_audit_log_standalone(&self, entry: AuditLogEntry) -> Result<(), StoreError>;

    /// Obtiene una solicitud por identificador interno.
    async fn find_by_id(&self, id: Uuid) -> Result<Option<AuthorizationRequestRecord>, StoreError>;
}
