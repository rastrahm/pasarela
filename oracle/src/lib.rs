//! Oracle de autorización off-chain — biblioteca compartida con tests de integración.

pub mod antifraud_client;
pub mod api;
pub mod auth;
pub mod config;
pub mod error;
pub mod funds;
pub mod logging;
pub mod persistence;
pub mod rail_adapters;
pub mod routes;
pub mod services;
pub mod ttl;
pub mod validation;

use axum::Router;
use std::sync::Arc;

use crate::logging::http_trace_layer;
use crate::persistence::AppState;

/// Construye el router HTTP con rutas públicas e internas protegidas.
///
/// # Inputs
/// - `state`: pool PostgreSQL, stores y configuración.
///
/// # Returns
/// Router de Axum listo para servir con `axum::serve`.
pub fn build_app(state: AppState) -> Router {
    routes::create_router(Arc::new(state)).layer(http_trace_layer())
}

/// Conecta a PostgreSQL y ejecuta migraciones pendientes.
pub async fn init_database(database_url: &str) -> anyhow::Result<sqlx::PgPool> {
    let pool = sqlx::PgPool::connect(database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}
