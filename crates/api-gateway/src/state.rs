//! Estado compartido del Gateway — config, clientes y motor de liquidación.

use std::sync::Arc;

use oracle_client::{HttpOracleClient, OracleClient};
use rail_switcher::RailSwitcher;
use settlement_adapters::SettlementEngine;
use uuid::Uuid;

use crate::config::AppConfig;
use crate::persistence::{
    init_database, load_merchant_registry, seed_merchants, GatewayStore, PersistenceError,
    TransactionRecord,
};
use crate::services::merchant::{MerchantRegistry, MerchantRegistryError};
use crate::services::rails::RailContext;

/// Error al inicializar el Gateway.
#[derive(Debug, thiserror::Error)]
pub enum BootstrapError {
    #[error("Oracle: {0}")]
    Oracle(#[from] oracle_client::OracleClientError),

    #[error("Comercios: {0}")]
    Merchant(#[from] MerchantRegistryError),

    #[error("Persistencia: {0}")]
    Persistence(#[from] PersistenceError),
}

/// Estado de la aplicación inyectado en handlers Axum.
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub settlement_engine: SettlementEngine,
    pub oracle_client: Arc<dyn OracleClient>,
    pub rail_switcher: RailSwitcher,
    pub rail_context: RailContext,
    pub merchant_registry: Arc<MerchantRegistry>,
    store: GatewayStore,
}

impl AppState {
    /// Construye el estado a partir de configuración cargada.
    ///
    /// Si `DATABASE_URL` está definida, conecta PostgreSQL, migra y persiste
    /// transacciones, settlements, auditoría e idempotencia. Sin DB usa memoria.
    pub async fn new(config: Arc<AppConfig>) -> Result<Self, BootstrapError> {
        let (store, merchant_registry) = if let Some(database_url) = &config.database_url {
            let pool = init_database(database_url).await?;
            seed_merchants(&pool, &config).await?;
            let registry = load_merchant_registry(&pool).await?;
            (GatewayStore::postgres(pool), Arc::new(registry))
        } else {
            let registry = config.build_merchant_registry()?;
            (GatewayStore::in_memory(), Arc::new(registry))
        };

        Self::from_parts_with_store(
            config.clone(),
            None,
            SettlementEngine::with_stub_adapters(),
            RailSwitcher,
            config.rail_context(),
            merchant_registry,
            store,
        )
        .await
    }

    /// Construye estado con dependencias inyectadas (tests).
    pub fn from_parts(
        config: Arc<AppConfig>,
        oracle_client: Arc<dyn OracleClient>,
        settlement_engine: SettlementEngine,
        rail_switcher: RailSwitcher,
        rail_context: RailContext,
        merchant_registry: Arc<MerchantRegistry>,
    ) -> Self {
        Self::from_parts_with_store_sync(
            config,
            oracle_client,
            settlement_engine,
            rail_switcher,
            rail_context,
            merchant_registry,
            GatewayStore::in_memory(),
        )
    }

    async fn from_parts_with_store(
        config: Arc<AppConfig>,
        oracle_client: Option<Arc<dyn OracleClient>>,
        settlement_engine: SettlementEngine,
        rail_switcher: RailSwitcher,
        rail_context: RailContext,
        merchant_registry: Arc<MerchantRegistry>,
        store: GatewayStore,
    ) -> Result<Self, BootstrapError> {
        let oracle_client = match oracle_client {
            Some(client) => client,
            None => Arc::new(HttpOracleClient::new(
                config.oracle_base_url.clone(),
                config.oracle_api_key.clone(),
                config.oracle_timeout_secs,
            )?),
        };

        if config.oracle_health_check {
            oracle_client.health().await?;
        }

        Ok(Self::from_parts_with_store_sync(
            config,
            oracle_client,
            settlement_engine,
            rail_switcher,
            rail_context,
            merchant_registry,
            store,
        ))
    }

    fn from_parts_with_store_sync(
        config: Arc<AppConfig>,
        oracle_client: Arc<dyn OracleClient>,
        settlement_engine: SettlementEngine,
        rail_switcher: RailSwitcher,
        rail_context: RailContext,
        merchant_registry: Arc<MerchantRegistry>,
        store: GatewayStore,
    ) -> Self {
        Self {
            config,
            settlement_engine,
            oracle_client,
            rail_switcher,
            rail_context,
            merchant_registry,
            store,
        }
    }

    pub fn store(&self) -> &GatewayStore {
        &self.store
    }

    /// Persiste o actualiza una transacción.
    pub async fn save_transaction(&self, record: TransactionRecord) -> Result<(), PersistenceError> {
        self.store.upsert_transaction(&record).await
    }

    /// Consulta una transacción almacenada (UC-09).
    pub async fn get_transaction(
        &self,
        id: Uuid,
        merchant_id: domain::MerchantId,
    ) -> Result<Option<TransactionRecord>, PersistenceError> {
        self.store.get_transaction(id, merchant_id).await
    }
}
