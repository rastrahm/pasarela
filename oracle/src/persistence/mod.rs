//! Capa de persistencia — traits e implementación PostgreSQL.

pub mod authorization_store;
pub mod error;
pub mod hold_store;
pub mod models;
pub mod postgres;

pub use authorization_store::AuthorizationStore;
pub use hold_store::HoldStore;
pub use postgres::{PostgresAuthorizationStore, PostgresHoldStore};

use std::sync::Arc;

use sqlx::PgPool;

use crate::antifraud_client::{AntifraudClient, AntifraudClientError, HttpAntifraudClient};
use crate::config::AppConfig;
use crate::rail_adapters::{CompositeRailProvider, RailAdapterError, RailBalanceProvider};

/// Estado compartido de la aplicación con pool, stores y clientes externos.
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub pool: PgPool,
    pub authorization_store: Arc<dyn AuthorizationStore>,
    pub hold_store: Arc<dyn HoldStore>,
    pub antifraud_client: Arc<dyn AntifraudClient>,
    pub rail_provider: Arc<dyn RailBalanceProvider>,
}

impl AppState {
    /// Construye el estado con stores PostgreSQL y clientes HTTP/RPC de producción.
    pub fn new(config: Arc<AppConfig>, pool: PgPool) -> Result<Self, StartupError> {
        let antifraud_client = Arc::new(HttpAntifraudClient::new(
            config.antifraud_base_url.clone(),
            config.antifraud_api_key.clone(),
            config.antifraud_timeout_secs,
        )?);
        let rail_provider = Arc::new(CompositeRailProvider::new(config.clone())?);

        Ok(Self::with_clients(
            config,
            pool,
            antifraud_client,
            rail_provider,
        ))
    }

    /// Construye el estado con clientes inyectados (tests).
    pub fn with_clients(
        config: Arc<AppConfig>,
        pool: PgPool,
        antifraud_client: Arc<dyn AntifraudClient>,
        rail_provider: Arc<dyn RailBalanceProvider>,
    ) -> Self {
        let authorization_store = Arc::new(PostgresAuthorizationStore::new(pool.clone()));
        let hold_store = Arc::new(PostgresHoldStore::new(pool.clone()));

        Self {
            config,
            pool,
            authorization_store,
            hold_store,
            antifraud_client,
            rail_provider,
        }
    }

    /// Atajo para tests: mock antifraude que aprueba + mock de rieles.
    pub fn with_test_clients(
        config: Arc<AppConfig>,
        pool: PgPool,
        antifraud_client: Arc<dyn AntifraudClient>,
    ) -> Self {
        use crate::rail_adapters::MockRailBalanceProvider;

        let rail_provider = Arc::new(MockRailBalanceProvider::from_config(&config));
        Self::with_clients(config, pool, antifraud_client, rail_provider)
    }
}

/// Error al inicializar clientes externos en el arranque.
#[derive(Debug, thiserror::Error)]
pub enum StartupError {
    #[error("cliente antifraude: {0}")]
    Antifraud(#[from] AntifraudClientError),

    #[error("adapter de riel: {0}")]
    Rail(#[from] RailAdapterError),
}
