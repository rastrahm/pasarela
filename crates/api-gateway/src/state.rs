//! Estado compartido del Gateway — config, clientes y motor de liquidación.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use domain::{FundingType, MerchantId, TransactionStatus};
use oracle_client::{HttpOracleClient, OracleClient};
use rail_switcher::RailSwitcher;
use settlement_adapters::SettlementEngine;
use uuid::Uuid;

use crate::config::AppConfig;
use crate::services::rails::RailContext;

/// Registro en memoria de una transacción (persistencia real en paso 4.12).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionRecord {
    pub transaction_id: Uuid,
    pub status: TransactionStatus,
    pub rail_used: Option<FundingType>,
    pub settlement_proof: Option<String>,
}

/// Estado de la aplicación inyectado en handlers Axum.
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub settlement_engine: SettlementEngine,
    pub oracle_client: Arc<dyn OracleClient>,
    pub rail_switcher: RailSwitcher,
    pub rail_context: RailContext,
    pub default_merchant_id: MerchantId,
    transactions: Arc<RwLock<HashMap<Uuid, TransactionRecord>>>,
}

impl AppState {
    /// Construye el estado a partir de configuración cargada.
    ///
    /// Verifica conectividad con el Oracle (`GET /health`) si
    /// `GATEWAY_ORACLE_HEALTH_CHECK` está activo.
    pub async fn new(config: Arc<AppConfig>) -> Result<Self, oracle_client::OracleClientError> {
        let oracle_client = Arc::new(HttpOracleClient::new(
            config.oracle_base_url.clone(),
            config.oracle_api_key.clone(),
            config.oracle_timeout_secs,
        )?);

        if config.oracle_health_check {
            oracle_client.health().await?;
        }

        Ok(Self::from_parts(
            config.clone(),
            oracle_client,
            SettlementEngine::with_stub_adapters(),
            RailSwitcher,
            config.rail_context(),
        ))
    }

    /// Construye estado con dependencias inyectadas (tests).
    pub fn from_parts(
        config: Arc<AppConfig>,
        oracle_client: Arc<dyn OracleClient>,
        settlement_engine: SettlementEngine,
        rail_switcher: RailSwitcher,
        rail_context: RailContext,
    ) -> Self {
        Self {
            default_merchant_id: config.default_merchant_id,
            config,
            settlement_engine,
            oracle_client,
            rail_switcher,
            rail_context,
            transactions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Persiste el resultado de checkout en memoria.
    pub fn store_transaction(&self, record: TransactionRecord) {
        if let Ok(mut map) = self.transactions.write() {
            map.insert(record.transaction_id, record);
        }
    }

    /// Consulta una transacción almacenada (paso 4.8).
    pub fn get_transaction(&self, id: Uuid) -> Option<TransactionRecord> {
        self.transactions
            .read()
            .ok()
            .and_then(|map| map.get(&id).cloned())
    }
}
