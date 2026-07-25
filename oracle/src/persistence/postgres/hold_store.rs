//! Implementación PostgreSQL de `HoldStore`.

use async_trait::async_trait;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use super::super::error::StoreError;
use super::super::hold_store::HoldStore;
use super::super::models::{CreateHold, HoldRecord, HoldStatus};
use crate::funds::FundingType;
use sqlx::PgTransaction;

/// Store PostgreSQL para holds y expiración TTL.
#[derive(Debug, Clone)]
pub struct PostgresHoldStore {
    pool: PgPool,
}

impl PostgresHoldStore {
    /// Crea el store con un pool de conexiones existente.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl HoldStore for PostgresHoldStore {
    async fn create_hold(
        &self,
        tx: &mut PgTransaction<'_>,
        hold: CreateHold,
    ) -> Result<HoldRecord, StoreError> {
        let record = sqlx::query_as::<_, HoldRecord>(
            r#"
            INSERT INTO hold (
                id, authorization_request_id, funding_type,
                amount, currency, status, expires_at
            )
            VALUES ($1, $2, $3, $4, $5, 'active', $6)
            RETURNING
                id, authorization_request_id, funding_type,
                amount, currency, status, expires_at, created_at
            "#,
        )
        .bind(hold.id)
        .bind(hold.authorization_request_id)
        .bind(hold.funding_type)
        .bind(hold.amount)
        .bind(&hold.currency)
        .bind(hold.expires_at)
        .fetch_one(&mut **tx)
        .await?;

        Ok(record)
    }

    async fn get_hold(&self, hold_id: Uuid) -> Result<Option<HoldRecord>, StoreError> {
        expire_hold_if_needed(&self.pool, hold_id).await?;
        fetch_hold(&self.pool, hold_id).await
    }

    async fn release_hold(&self, hold_id: Uuid) -> Result<HoldRecord, StoreError> {
        expire_hold_if_needed(&self.pool, hold_id).await?;

        let Some(hold) = fetch_hold(&self.pool, hold_id).await? else {
            return Err(StoreError::NotFound);
        };

        match hold.status {
            HoldStatus::Released | HoldStatus::Expired => Ok(hold),
            HoldStatus::Consumed => Err(StoreError::InvalidHoldState("consumed".to_string())),
            HoldStatus::Active => {
                let updated = sqlx::query_as::<_, HoldRecord>(
                    r#"
                    UPDATE hold
                    SET status = 'released'
                    WHERE id = $1 AND status = 'active'
                    RETURNING
                        id, authorization_request_id, funding_type,
                        amount, currency, status, expires_at, created_at
                    "#,
                )
                .bind(hold_id)
                .fetch_one(&self.pool)
                .await?;

                Ok(updated)
            }
        }
    }

    async fn sum_active_holds(&self, funding_type: FundingType) -> Result<Decimal, StoreError> {
        expire_stale_holds_internal(&self.pool).await?;

        let total: Decimal = sqlx::query_scalar(
            r#"
            SELECT COALESCE(SUM(amount), 0)
            FROM hold
            WHERE funding_type = $1 AND status = 'active'
            "#,
        )
        .bind(funding_type)
        .fetch_one(&self.pool)
        .await?;

        Ok(total)
    }

    async fn expire_stale_holds(&self) -> Result<u64, StoreError> {
        expire_stale_holds_internal(&self.pool).await
    }

    async fn update_status_in_tx(
        &self,
        tx: &mut PgTransaction<'_>,
        hold_id: Uuid,
        status: HoldStatus,
    ) -> Result<HoldRecord, StoreError> {
        let record = sqlx::query_as::<_, HoldRecord>(
            r#"
            UPDATE hold
            SET status = $2
            WHERE id = $1
            RETURNING
                id, authorization_request_id, funding_type,
                amount, currency, status, expires_at, created_at
            "#,
        )
        .bind(hold_id)
        .bind(status)
        .fetch_one(&mut **tx)
        .await?;

        Ok(record)
    }
}

async fn fetch_hold(pool: &PgPool, hold_id: Uuid) -> Result<Option<HoldRecord>, StoreError> {
    let record = sqlx::query_as::<_, HoldRecord>(
        r#"
        SELECT
            id, authorization_request_id, funding_type,
            amount, currency, status, expires_at, created_at
        FROM hold
        WHERE id = $1
        "#,
    )
    .bind(hold_id)
    .fetch_optional(pool)
    .await?;

    Ok(record)
}

async fn expire_hold_if_needed(pool: &PgPool, hold_id: Uuid) -> Result<(), StoreError> {
    sqlx::query(
        r#"
        UPDATE hold
        SET status = 'expired'
        WHERE id = $1
          AND status = 'active'
          AND expires_at < NOW()
        "#,
    )
    .bind(hold_id)
    .execute(pool)
    .await?;

    Ok(())
}

async fn expire_stale_holds_internal(pool: &PgPool) -> Result<u64, StoreError> {
    let result = sqlx::query(
        r#"
        UPDATE hold
        SET status = 'expired'
        WHERE status = 'active'
          AND expires_at < NOW()
        "#,
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}
