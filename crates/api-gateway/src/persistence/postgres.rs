//! Persistencia PostgreSQL del Gateway.

use domain::{FundingType, MerchantId, TransactionStatus};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::GatewayError;
use crate::routes::CheckoutResponse;
use crate::services::idempotency::CachedCheckoutResult;

use super::error::PersistenceError;
use super::models::{AuditEvent, SettlementRecord, TransactionRecord};
use super::{funding_type_to_db, settlement_status_to_db, status_to_db};

/// Store PostgreSQL para transacciones, settlements, auditoría e idempotencia.
#[derive(Clone)]
pub struct PostgresStore {
    pool: PgPool,
}

impl PostgresStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn upsert_transaction(&self, record: &TransactionRecord) -> Result<(), PersistenceError> {
        sqlx::query(
            r#"
            INSERT INTO gateway_transaction (
                id, merchant_id, status, amount, currency,
                funding_type, settlement_proof, oracle_hold_id
            )
            VALUES ($1, $2, $3::transaction_status, $4, $5, $6::gateway_funding_type, $7, $8)
            ON CONFLICT (id) DO UPDATE SET
                status = EXCLUDED.status,
                funding_type = EXCLUDED.funding_type,
                settlement_proof = EXCLUDED.settlement_proof,
                oracle_hold_id = EXCLUDED.oracle_hold_id,
                updated_at = NOW()
            "#,
        )
        .bind(record.transaction_id)
        .bind(record.merchant_id.0)
        .bind(status_to_db(record.status))
        .bind(record.amount)
        .bind(&record.currency)
        .bind(record.rail_used.map(funding_type_to_db))
        .bind(&record.settlement_proof)
        .bind(record.oracle_hold_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_transaction(
        &self,
        id: Uuid,
    ) -> Result<Option<TransactionRecord>, PersistenceError> {
        let row = sqlx::query_as::<_, TransactionRow>(
            r#"
            SELECT
                id,
                merchant_id,
                status,
                amount,
                currency,
                funding_type,
                settlement_proof,
                oracle_hold_id
            FROM gateway_transaction
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|entry| entry.into_record()))
    }

    pub async fn insert_settlement(&self, record: &SettlementRecord) -> Result<(), PersistenceError> {
        sqlx::query(
            r#"
            INSERT INTO settlement (id, transaction_id, rail_type, proof, status)
            VALUES ($1, $2, $3::gateway_funding_type, $4, $5::settlement_status)
            ON CONFLICT (transaction_id) DO NOTHING
            "#,
        )
        .bind(record.id)
        .bind(record.transaction_id)
        .bind(funding_type_to_db(record.rail_type))
        .bind(&record.proof)
        .bind(settlement_status_to_db(record.status))
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn append_audit(&self, event: &AuditEvent) -> Result<(), PersistenceError> {
        sqlx::query(
            r#"
            INSERT INTO gateway_audit_log (id, transaction_id, event_type, detail)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(event.transaction_id)
        .bind(&event.event_type)
        .bind(&event.detail)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn idempotency_begin(
        &self,
        merchant_id: MerchantId,
        key: &str,
        fingerprint: &str,
    ) -> Result<Option<CachedCheckoutResult>, PersistenceError> {
        let existing = sqlx::query_as::<_, IdempotencyRow>(
            r#"
            SELECT request_fingerprint, status, response_status, response_body
            FROM idempotency_record
            WHERE merchant_id = $1 AND idempotency_key = $2
            "#,
        )
        .bind(merchant_id.0)
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = existing {
            return row.into_begin_result(fingerprint);
        }

        let inserted = sqlx::query(
            r#"
            INSERT INTO idempotency_record (
                merchant_id, idempotency_key, request_fingerprint, status
            )
            VALUES ($1, $2, $3, 'in_flight')
            ON CONFLICT (merchant_id, idempotency_key) DO NOTHING
            "#,
        )
        .bind(merchant_id.0)
        .bind(key)
        .bind(fingerprint)
        .execute(&self.pool)
        .await?;

        if inserted.rows_affected() == 0 {
            let row = sqlx::query_as::<_, IdempotencyRow>(
                r#"
                SELECT request_fingerprint, status, response_status, response_body
                FROM idempotency_record
                WHERE merchant_id = $1 AND idempotency_key = $2
                "#,
            )
            .bind(merchant_id.0)
            .bind(key)
            .fetch_one(&self.pool)
            .await?;

            return row.into_begin_result(fingerprint);
        }

        Ok(None)
    }

    pub async fn idempotency_complete(
        &self,
        merchant_id: MerchantId,
        key: &str,
        fingerprint: &str,
        result: &Result<CheckoutResponse, GatewayError>,
    ) -> Result<(), PersistenceError> {
        let (response_status, response_body) = serialize_checkout_result(result)?;

        sqlx::query(
            r#"
            UPDATE idempotency_record
            SET
                status = 'completed',
                request_fingerprint = $3,
                response_status = $4,
                response_body = $5
            WHERE merchant_id = $1 AND idempotency_key = $2
            "#,
        )
        .bind(merchant_id.0)
        .bind(key)
        .bind(fingerprint)
        .bind(i16::try_from(response_status).unwrap_or(i16::MAX))
        .bind(response_body)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct TransactionRow {
    id: Uuid,
    merchant_id: Uuid,
    status: String,
    amount: rust_decimal::Decimal,
    currency: String,
    funding_type: Option<String>,
    settlement_proof: Option<String>,
    oracle_hold_id: Option<Uuid>,
}

impl TransactionRow {
    fn into_record(self) -> TransactionRecord {
        TransactionRecord {
            transaction_id: self.id,
            merchant_id: MerchantId::new(self.merchant_id),
            status: status_from_db(&self.status).unwrap_or(TransactionStatus::Failed),
            amount: self.amount,
            currency: self.currency,
            rail_used: self.funding_type.as_deref().and_then(funding_type_from_db),
            settlement_proof: self.settlement_proof,
            oracle_hold_id: self.oracle_hold_id,
        }
    }
}

#[derive(sqlx::FromRow)]
struct IdempotencyRow {
    request_fingerprint: String,
    status: String,
    response_status: i16,
    response_body: serde_json::Value,
}

impl IdempotencyRow {
    fn into_begin_result(
        self,
        fingerprint: &str,
    ) -> Result<Option<CachedCheckoutResult>, PersistenceError> {
        if self.request_fingerprint != fingerprint {
            return Err(PersistenceError::Internal(
                "Idempotency-Key reutilizada con payload distinto".to_string(),
            ));
        }

        if self.status == "in_flight" {
            return Err(PersistenceError::Internal(
                "checkout idempotente en curso".to_string(),
            ));
        }

        deserialize_checkout_result(self.response_status, &self.response_body).map(Some)
    }
}

fn serialize_checkout_result(
    result: &Result<CheckoutResponse, GatewayError>,
) -> Result<(u16, serde_json::Value), PersistenceError> {
    match result {
        Ok(response) => {
            let body = serde_json::to_value(response)
                .map_err(|err| PersistenceError::Internal(err.to_string()))?;
            Ok((200, body))
        }
        Err(error) => {
            let body = serde_json::json!({
                "error_code": error.error_code(),
                "message": error.to_string(),
                "kind": "gateway_error",
            });
            Ok((error.status_code().as_u16(), body))
        }
    }
}

fn deserialize_checkout_result(
    status_code: i16,
    body: &serde_json::Value,
) -> Result<CachedCheckoutResult, PersistenceError> {
    if status_code == 200 {
        let response: CheckoutResponse = serde_json::from_value(body.clone())
            .map_err(|err| PersistenceError::Internal(err.to_string()))?;
        return Ok(CachedCheckoutResult::Success(response));
    }

    let error_code = body
        .get("error_code")
        .and_then(|value| value.as_str())
        .unwrap_or("INTERNAL_ERROR");

    let gateway_error = match error_code {
        "UNAUTHORIZED" => GatewayError::Unauthorized,
        "INSUFFICIENT_FUNDS" => GatewayError::InsufficientFunds,
        "INVALID_CARD" => GatewayError::InvalidCard,
        "INVALID_REQUEST" => GatewayError::InvalidRequest,
        "CONFLICT" => GatewayError::Conflict(
            body.get("message")
                .and_then(|value| value.as_str())
                .unwrap_or("conflicto")
                .to_string(),
        ),
        "NOT_FOUND" => GatewayError::NotFound,
        "RAIL_UNAVAILABLE" => GatewayError::RailUnavailable,
        other => GatewayError::Internal(format!("error idempotente: {other}")),
    };

    Ok(CachedCheckoutResult::Error(gateway_error))
}

fn status_from_db(raw: &str) -> Option<TransactionStatus> {
    match raw {
        "pending" => Some(TransactionStatus::Pending),
        "authorized" => Some(TransactionStatus::Authorized),
        "held" => Some(TransactionStatus::Held),
        "settled" => Some(TransactionStatus::Settled),
        "failed" => Some(TransactionStatus::Failed),
        "reversed" => Some(TransactionStatus::Reversed),
        _ => None,
    }
}

fn funding_type_from_db(raw: &str) -> Option<FundingType> {
    match raw {
        "traditional_bank" => Some(FundingType::TraditionalBank),
        "binance_cex" => Some(FundingType::BinanceCex),
        "solana_wallet" => Some(FundingType::SolanaWallet),
        _ => None,
    }
}
