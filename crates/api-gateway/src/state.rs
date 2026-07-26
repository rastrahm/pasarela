//! Estado compartido del Gateway — config, clientes y motor de liquidación.

use std::sync::Arc;

use oracle_client::HttpOracleClient;
use settlement_adapters::SettlementEngine;

use crate::config::AppConfig;

/// Estado de la aplicación inyectado en handlers Axum.
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub settlement_engine: SettlementEngine,
    pub oracle_client: HttpOracleClient,
}

impl AppState {
    /// Construye el estado a partir de configuración cargada.
    ///
    /// # Inputs
    /// - `config`: parámetros de red, Oracle y persistencia futura.
    ///
    /// # Returns
    /// Estado listo para rutas HTTP o error si el cliente Oracle no inicializa.
    pub fn new(config: Arc<AppConfig>) -> Result<Self, oracle_client::OracleClientError> {
        let oracle_client = HttpOracleClient::new(
            config.oracle_base_url.clone(),
            config.oracle_api_key.clone(),
            config.oracle_timeout_secs,
        )?;

        Ok(Self {
            config,
            settlement_engine: SettlementEngine::with_stub_adapters(),
            oracle_client,
        })
    }
}
