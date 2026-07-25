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

/// Estado compartido de la aplicación con pool, stores y cliente antifraude.
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub pool: PgPool,
    pub authorization_store: Arc<dyn AuthorizationStore>,
    pub hold_store: Arc<dyn HoldStore>,
    pub antifraud_client: Arc<dyn AntifraudClient>,
}

impl AppState {
    /// Construye el estado con stores PostgreSQL y cliente HTTP antifraude.
    pub fn new(config: Arc<AppConfig>, pool: PgPool) -> Result<Self, AntifraudClientError> {
        let antifraud_client = Arc::new(HttpAntifraudClient::new(
            config.antifraud_base_url.clone(),
            config.antifraud_api_key.clone(),
            config.antifraud_timeout_secs,
        )?);

        Ok(Self::with_antifraud_client(config, pool, antifraud_client))
    }

    /// Construye el estado con un cliente antifraude inyectado (tests).
    pub fn with_antifraud_client(
        config: Arc<AppConfig>,
        pool: PgPool,
        antifraud_client: Arc<dyn AntifraudClient>,
    ) -> Self {
        let authorization_store = Arc::new(PostgresAuthorizationStore::new(pool.clone()));
        let hold_store = Arc::new(PostgresHoldStore::new(pool.clone()));

        Self {
            config,
            pool,
            authorization_store,
            hold_store,
            antifraud_client,
        }
    }
}
