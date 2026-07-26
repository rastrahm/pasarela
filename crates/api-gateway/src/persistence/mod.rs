//! Persistencia del Gateway — PostgreSQL con fallback in-memory.

mod error;
mod memory;
mod models;
mod postgres;

pub use error::PersistenceError;
pub use memory::InMemoryStore;
pub use models::{AuditEvent, SettlementRecord, SettlementStatus, TransactionRecord};
pub use postgres::PostgresStore;

use std::sync::Arc;

use domain::{FundingType, MerchantId, TransactionStatus};
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::AppConfig;
use crate::error::GatewayError;
use crate::routes::CheckoutResponse;
use crate::services::idempotency::CachedCheckoutResult;
use crate::services::merchant::{hash_api_key, ApiKeyMode, MerchantApiKeyEntry, MerchantRegistry};

/// Backend de persistencia del Gateway.
#[derive(Clone)]
pub enum GatewayStore {
    Memory(Arc<InMemoryStore>),
    Postgres(PostgresStore),
}

impl Default for GatewayStore {
    fn default() -> Self {
        Self::Memory(Arc::new(InMemoryStore::default()))
    }
}

impl GatewayStore {
    pub fn in_memory() -> Self {
        Self::Memory(Arc::new(InMemoryStore::default()))
    }

    pub fn postgres(pool: PgPool) -> Self {
        Self::Postgres(PostgresStore::new(pool))
    }

    pub async fn upsert_transaction(&self, record: &TransactionRecord) -> Result<(), PersistenceError> {
        match self {
            Self::Memory(store) => store.upsert_transaction(record),
            Self::Postgres(store) => store.upsert_transaction(record).await,
        }
    }

    pub async fn get_transaction(
        &self,
        id: Uuid,
        merchant_id: MerchantId,
    ) -> Result<Option<TransactionRecord>, PersistenceError> {
        match self {
            Self::Memory(store) => store.get_transaction(id, merchant_id),
            Self::Postgres(store) => store.get_transaction(id, merchant_id).await,
        }
    }

    pub async fn insert_settlement(&self, record: &SettlementRecord) -> Result<(), PersistenceError> {
        match self {
            Self::Memory(store) => store.insert_settlement(record),
            Self::Postgres(store) => store.insert_settlement(record).await,
        }
    }

    pub async fn append_audit(&self, event: &AuditEvent) -> Result<(), PersistenceError> {
        match self {
            Self::Memory(store) => store.append_audit(event),
            Self::Postgres(store) => store.append_audit(event).await,
        }
    }

    pub async fn idempotency_begin(
        &self,
        merchant_id: MerchantId,
        key: &str,
        fingerprint: &str,
    ) -> Result<Option<CachedCheckoutResult>, PersistenceError> {
        match self {
            Self::Memory(store) => store.idempotency_begin(merchant_id, key, fingerprint),
            Self::Postgres(store) => store.idempotency_begin(merchant_id, key, fingerprint).await,
        }
    }

    pub async fn idempotency_complete(
        &self,
        merchant_id: MerchantId,
        key: &str,
        fingerprint: &str,
        result: &Result<CheckoutResponse, GatewayError>,
    ) -> Result<(), PersistenceError> {
        match self {
            Self::Memory(store) => store.idempotency_complete(merchant_id, key, fingerprint, result),
            Self::Postgres(store) => {
                store
                    .idempotency_complete(merchant_id, key, fingerprint, result)
                    .await
            }
        }
    }
}

/// Conecta a PostgreSQL y ejecuta migraciones pendientes.
pub async fn init_database(database_url: &str) -> Result<PgPool, PersistenceError> {
    let pool = PgPool::connect(database_url).await?;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|err| PersistenceError::Internal(format!("migración: {err}")))?;
    Ok(pool)
}

/// Inserta o actualiza comercios desde la configuración de arranque.
pub async fn seed_merchants(pool: &PgPool, config: &AppConfig) -> Result<(), PersistenceError> {
    for entry in merchant_entries_from_config(config) {
        upsert_merchant(pool, &entry).await?;
    }
    Ok(())
}

/// Carga el registro de comercios autenticables desde PostgreSQL.
pub async fn load_merchant_registry(pool: &PgPool) -> Result<MerchantRegistry, PersistenceError> {
    let rows = sqlx::query_as::<_, MerchantRow>(
        r#"
        SELECT id, api_key_hash, api_key_env
        FROM merchant
        "#,
    )
    .fetch_all(pool)
    .await?;

    let registry = MerchantRegistry::default();
    for row in rows {
        registry.register_hash(
            row.api_key_hash,
            MerchantId::new(row.id),
            api_key_env_from_db(&row.api_key_env)?,
        );
    }

    Ok(registry)
}

fn merchant_entries_from_config(config: &AppConfig) -> Vec<MerchantApiKeyEntry> {
    if !config.merchant_api_keys.is_empty() {
        return config.merchant_api_keys.clone();
    }

    config
        .bootstrap_test_api_key
        .as_ref()
        .map(|api_key| MerchantApiKeyEntry {
            api_key: api_key.clone(),
            merchant_id: config.default_merchant_id,
            mode: ApiKeyMode::Test,
        })
        .into_iter()
        .collect()
}

async fn upsert_merchant(pool: &PgPool, entry: &MerchantApiKeyEntry) -> Result<(), PersistenceError> {
    let api_key_hash = hash_api_key(&entry.api_key);
    let env = api_key_env_to_db(entry.mode);

    sqlx::query(
        r#"
        INSERT INTO merchant (id, name, default_currency, api_key_hash, api_key_env)
        VALUES ($1, $2, 'USD', $3, $4::api_key_env)
        ON CONFLICT (id) DO UPDATE SET
            api_key_hash = EXCLUDED.api_key_hash,
            api_key_env = EXCLUDED.api_key_env
        "#,
    )
    .bind(entry.merchant_id.0)
    .bind(format!("merchant-{}", entry.merchant_id.0))
    .bind(api_key_hash)
    .bind(env)
    .execute(pool)
    .await?;

    Ok(())
}

#[derive(sqlx::FromRow)]
struct MerchantRow {
    id: Uuid,
    api_key_hash: String,
    api_key_env: String,
}

pub(crate) fn status_to_db(status: TransactionStatus) -> &'static str {
    match status {
        TransactionStatus::Pending => "pending",
        TransactionStatus::Authorized => "authorized",
        TransactionStatus::Held => "held",
        TransactionStatus::Settled => "settled",
        TransactionStatus::Failed => "failed",
        TransactionStatus::Reversed => "reversed",
    }
}

pub(crate) fn funding_type_to_db(funding_type: FundingType) -> &'static str {
    match funding_type {
        FundingType::TraditionalBank => "traditional_bank",
        FundingType::BinanceCex => "binance_cex",
        FundingType::SolanaWallet => "solana_wallet",
    }
}

pub(crate) fn settlement_status_to_db(status: SettlementStatus) -> &'static str {
    match status {
        SettlementStatus::Completed => "completed",
        SettlementStatus::Failed => "failed",
    }
}

fn api_key_env_to_db(mode: ApiKeyMode) -> &'static str {
    match mode {
        ApiKeyMode::Test => "test",
        ApiKeyMode::Live => "live",
    }
}

fn api_key_env_from_db(raw: &str) -> Result<ApiKeyMode, PersistenceError> {
    match raw {
        "test" => Ok(ApiKeyMode::Test),
        "live" => Ok(ApiKeyMode::Live),
        other => Err(PersistenceError::Internal(format!(
            "api_key_env desconocido: {other}"
        ))),
    }
}
