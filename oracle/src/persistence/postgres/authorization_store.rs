//! Implementación PostgreSQL de `AuthorizationStore`.

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use super::super::authorization_store::AuthorizationStore;
use super::super::error::StoreError;
use super::super::models::{
    AuditLogEntry, AuthorizationRequestRecord, CreateAuthorizationRequest,
};
use sqlx::PgTransaction;

/// Store PostgreSQL para solicitudes de autorización y audit log.
#[derive(Debug, Clone)]
pub struct PostgresAuthorizationStore {
    pool: PgPool,
}

impl PostgresAuthorizationStore {
    /// Crea el store con un pool de conexiones existente.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuthorizationStore for PostgresAuthorizationStore {
    async fn create_request(
        &self,
        tx: &mut PgTransaction<'_>,
        request: CreateAuthorizationRequest,
    ) -> Result<AuthorizationRequestRecord, StoreError> {
        let record = sqlx::query_as::<_, AuthorizationRequestRecord>(
            r#"
            INSERT INTO authorization_request (
                id, gateway_request_id, funding_type,
                amount, currency, brand, brand_code,
                card_token_hash, result
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING
                id, gateway_request_id, funding_type,
                amount, currency, brand, brand_code,
                card_token_hash, result, created_at
            "#,
        )
        .bind(request.id)
        .bind(request.gateway_request_id)
        .bind(request.funding_type)
        .bind(request.amount)
        .bind(&request.currency)
        .bind(request.brand)
        .bind(request.brand_code)
        .bind(&request.card_token_hash)
        .bind(request.result)
        .fetch_one(&mut **tx)
        .await?;

        Ok(record)
    }

    async fn append_audit_log(
        &self,
        tx: &mut PgTransaction<'_>,
        entry: AuditLogEntry,
    ) -> Result<(), StoreError> {
        insert_audit_log(&mut **tx, &entry).await
    }

    async fn append_audit_log_standalone(&self, entry: AuditLogEntry) -> Result<(), StoreError> {
        insert_audit_log(&self.pool, &entry).await
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<AuthorizationRequestRecord>, StoreError> {
        let record = sqlx::query_as::<_, AuthorizationRequestRecord>(
            r#"
            SELECT
                id, gateway_request_id, funding_type,
                amount, currency, brand, brand_code,
                card_token_hash, result, created_at
            FROM authorization_request
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(record)
    }
}

async fn insert_audit_log<'e, E>(executor: E, entry: &AuditLogEntry) -> Result<(), StoreError>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query(
        r#"
        INSERT INTO oracle_audit_log (
            id, hold_id, authorization_request_id,
            event_type, detail, caller_ip
        )
        VALUES ($1, $2, $3, $4, $5, $6::inet)
        "#,
    )
    .bind(entry.id)
    .bind(entry.hold_id)
    .bind(entry.authorization_request_id)
    .bind(&entry.event_type)
    .bind(&entry.detail)
    .bind(entry.caller_ip.as_deref())
    .execute(executor)
    .await?;

    Ok(())
}
